use super::InstDispatchError;
use crate::{
    base::slabref::SlabRef,
    ir::{
        inst::{InstData, InstRef, usedef::UseData},
        module::Module,
    },
    mir::{
        inst::{IMirSubInst, impls::*, inst::MirInst, opcode::MirOP},
        module::{MirGlobalRef, stack::VirtRegAlloc},
        operand::{
            MirOperand,
            compound::MirSymbolOp,
            imm::{ImmLoad32, ImmLoad64},
            reg::{FPR32, FPR64, GPR32, GPR64, GPReg, RegID, RegOperand},
            subop::IMirSubOperand,
        },
        translate::mirgen::operandgen::{OperandMap, OperandMapError},
    },
};
use slab::Slab;
use std::{cell::Ref, collections::VecDeque};

#[derive(Debug, Clone, Copy)]
enum StrDest {
    G64(GPR64),
    Global(MirGlobalRef),
}

pub(crate) fn generate_store_inst(
    ir_module: &Module,
    operand_map: &OperandMap<'_>,
    vreg_alloc: &mut VirtRegAlloc,
    out_insts: &mut VecDeque<MirInst>,
    ir_ref: InstRef,
    alloc_inst: &Slab<InstData>,
    alloc_use: Ref<'_, Slab<UseData>>,
) -> Option<Result<(), InstDispatchError>> {
    type StrSrc = crate::mir::translate::mirgen::operandgen::DispatchedReg;

    let InstData::Store(_, store) = ir_ref.to_slabref_unwrap(alloc_inst) else {
        panic!("Expected Store instruction");
    };

    let src_ir = store.source.get_operand(&alloc_use);
    let src_mir = StrSrc::from_valuessa(
        operand_map,
        &ir_module.type_ctx,
        vreg_alloc,
        out_insts,
        &src_ir,
        false,
    );
    let src_mir = match src_mir {
        Ok(x) => x,
        Err(OperandMapError::OperandUndefined) => return Some(Ok(())),
        Err(e) => panic!("Failed to find source operand for store: {e:?}"),
    };

    let dst_ir = store.target.get_operand(&alloc_use);
    let dst_mir = match operand_map.find_operand_no_constdata(&dst_ir) {
        Ok(MirOperand::GPReg(gpreg)) => StrDest::G64(GPR64::from_real(gpreg)),
        Ok(MirOperand::Global(globl)) => StrDest::Global(globl),
        Ok(m) => panic!("Invalid dest for store: {m:?}"),
        Err(e) => panic!("Failed to find dest for store: {e:?}"),
    };
    let zoff32 = ImmLoad32::new(0);
    let zoff64 = ImmLoad64::new(0);
    match dst_mir {
        StrDest::G64(dst_ptr) => {
            let store_inst = match src_mir {
                StrSrc::F32(fpr32) => {
                    StoreF32Base::new(MirOP::StrF32Base, fpr32, dst_ptr, zoff32).into_mir()
                }
                StrSrc::F64(fpr64) => {
                    StoreF64Base::new(MirOP::StrF64Base, fpr64, dst_ptr, zoff64).into_mir()
                }
                StrSrc::G32(gpr32) => {
                    StoreGr32Base::new(MirOP::StrGr32Base, gpr32, dst_ptr, zoff32).into_mir()
                }
                StrSrc::G64(gpr64) => {
                    StoreGr64Base::new(MirOP::StrGr64Base, gpr64, dst_ptr, zoff64).into_mir()
                }
            };
            out_insts.push_back(store_inst);
        }
        StrDest::Global(global) => {
            let global = MirSymbolOp::Global(global);
            let hi20_addr_vreg = vreg_alloc.insert_gp(GPReg::new_long(RegID::Virt(0)));
            let hi20_addr_vreg = GPR64::from_real(hi20_addr_vreg);
            let adr_hi20 = Adr::new(MirOP::Adr, hi20_addr_vreg, global);
            out_insts.push_back(adr_hi20.into_mir());

            let store_inst = match src_mir {
                StrSrc::F32(src) => {
                    StoreF32BaseS::new(MirOP::StrF32BaseS, src, hi20_addr_vreg, global).into_mir()
                }
                StrSrc::F64(src) => {
                    StoreF64BaseS::new(MirOP::StrF64BaseS, src, hi20_addr_vreg, global).into_mir()
                }
                StrSrc::G32(src) => {
                    StoreGr32BaseS::new(MirOP::StrGr32BaseS, src, hi20_addr_vreg, global).into_mir()
                }
                StrSrc::G64(src) => {
                    StoreGr64BaseS::new(MirOP::StrGr64BaseS, src, hi20_addr_vreg, global).into_mir()
                }
            };
            out_insts.push_back(store_inst);
        }
    };
    None
}

#[derive(Debug, Clone, Copy)]
enum LdrDest {
    F32(FPR32),
    F64(FPR64),
    G32(GPR32),
    G64(GPR64),
}

impl LdrDest {
    fn from_reg_operand(op: RegOperand) -> Self {
        let RegOperand(id, si, uf, is_fp) = op;
        let bits_log2 = si.get_bits_log2();
        match (is_fp, bits_log2) {
            (true, 5) => LdrDest::F32(FPR32(id, uf)),
            (true, 6) => LdrDest::F64(FPR64(id, uf)),
            (false, 5) => LdrDest::G32(GPR32(id, uf)),
            (false, 6) => LdrDest::G64(GPR64(id, uf)),
            _ => panic!("Unsupported size for load: {bits_log2}"),
        }
    }
}

pub(crate) fn dispatch_load(
    operand_map: &OperandMap<'_>,
    ir_ref: InstRef,
    alloc_inst: &Slab<InstData>,
    alloc_use: Ref<Slab<UseData>>,
    out_insts: &mut VecDeque<MirInst>,
) {
    let InstData::Load(_, load) = ir_ref.to_slabref_unwrap(alloc_inst) else {
        panic!("Expected Load instruction");
    };
    let src_ir = load.source.get_operand(&alloc_use);
    let src_mir = operand_map
        .find_operand_no_constdata(&src_ir)
        .expect("Failed to find source operand for load");
    let dst_mir = operand_map.find_operand_for_inst(ir_ref).unwrap();
    let dst_kind = LdrDest::from_reg_operand(dst_mir);
    let ldr_inst = match dst_kind {
        LdrDest::F32(fpr32) => match src_mir {
            MirOperand::GPReg(gpreg) => LoadF32::new(
                MirOP::LdrF32,
                fpr32,
                GPR64::from_real(gpreg),
                GPR64::zr(),
                None,
            )
            .into_mir(),
            MirOperand::Label(label) => {
                LoadF32Literal::new(MirOP::LdrF32Literal, fpr32, MirSymbolOp::Label(label))
                    .into_mir()
            }
            MirOperand::Global(globl) => {
                LoadF32Literal::new(MirOP::LdrF32Literal, fpr32, MirSymbolOp::Global(globl))
                    .into_mir()
            }
            _ => panic!("Invalid source operand load from {src_mir:?} to {dst_mir:?}"),
        },
        LdrDest::F64(fpr64) => match src_mir {
            MirOperand::GPReg(gpreg) => LoadF64Base::new(
                MirOP::LdrF64Base,
                fpr64,
                GPR64::from_real(gpreg),
                ImmLoad64::new(0),
            )
            .into_mir(),
            MirOperand::Label(label) => {
                LoadF64Literal::new(MirOP::LdrF64Literal, fpr64, MirSymbolOp::Label(label))
                    .into_mir()
            }
            MirOperand::Global(globl) => {
                LoadF64Literal::new(MirOP::LdrF64Literal, fpr64, MirSymbolOp::Global(globl))
                    .into_mir()
            }
            _ => panic!("Invalid source operand load from {src_mir:?} to {dst_mir:?}"),
        },
        LdrDest::G32(gpr32) => match src_mir {
            MirOperand::GPReg(gpreg) => LoadGr32Base::new(
                MirOP::LdrGr32Base,
                gpr32,
                GPR64::from_real(gpreg),
                ImmLoad32::new(0),
            )
            .into_mir(),
            MirOperand::Label(label) => {
                LoadGr32Literal::new(MirOP::LdrGr32Literal, gpr32, MirSymbolOp::Label(label))
                    .into_mir()
            }
            MirOperand::Global(globl) => {
                LoadGr32Literal::new(MirOP::LdrGr32Literal, gpr32, MirSymbolOp::Global(globl))
                    .into_mir()
            }
            _ => panic!("Invalid source operand load from {src_mir:?} to {dst_mir:?}"),
        },
        LdrDest::G64(gpr64) => match src_mir {
            MirOperand::GPReg(gpreg) => LoadGr64Base::new(
                MirOP::LdrGr64Base,
                gpr64,
                GPR64::from_real(gpreg),
                ImmLoad64::new(0),
            )
            .into_mir(),
            MirOperand::Label(label) => {
                LoadGr64Literal::new(MirOP::LdrGr64Literal, gpr64, MirSymbolOp::Label(label))
                    .into_mir()
            }
            MirOperand::Global(globl) => {
                LoadGr64Literal::new(MirOP::LdrGr64Literal, gpr64, MirSymbolOp::Global(globl))
                    .into_mir()
            }
            MirOperand::SwitchTab(index) => {
                LoadGr64Literal::new(MirOP::LdrGr64Literal, gpr64, MirSymbolOp::SwitchTab(index))
                    .into_mir()
            }
            _ => panic!("Invalid source operand load from {src_mir:?} to {dst_mir:?}"),
        },
    };
    out_insts.push_back(ldr_inst);
}

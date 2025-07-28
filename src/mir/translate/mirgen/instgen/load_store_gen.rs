use super::InstDispatchError;
use crate::{
    base::slabref::SlabRef,
    ir::{
        inst::{InstData, InstRef, usedef::UseData},
        module::Module,
    },
    mir::{
        inst::{IMirSubInst, impls::*, inst::MirInst, opcode::MirOP},
        module::{MirGlobalRef, vreg_alloc::VirtRegAlloc},
        operand::{
            MirOperand,
            compound::MirSymbolOp,
            imm::{ImmLSP32, ImmLSP64},
            reg::{FPR32, FPR64, GPR32, GPR64, RegOperand},
            subop::IMirSubOperand,
        },
        translate::mirgen::operandgen::{InstRetval, OperandMap, OperandMapError},
    },
};
use log::debug;
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
    let zoff32 = ImmLSP32::new(0);
    let zoff64 = ImmLSP64::new(0);
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
            let dst = MirSymbolOp::Global(global);
            let hi20_addr = vreg_alloc.insert_gpr64(GPR64::new_empty());

            let store_inst = match src_mir {
                StrSrc::F32(src) => {
                    MirStrLitF32::new(MirOP::MirStrLitF32, hi20_addr, src, dst).into_mir()
                }
                StrSrc::F64(src) => {
                    MirStrLitF64::new(MirOP::MirStrLitF64, hi20_addr, src, dst).into_mir()
                }
                StrSrc::G32(src) => {
                    MirStrLitG32::new(MirOP::MirStrLitG32, hi20_addr, src, dst).into_mir()
                }
                StrSrc::G64(src) => {
                    MirStrLitG64::new(MirOP::MirStrLitG64, hi20_addr, src, dst).into_mir()
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
    vreg_alloc: &mut VirtRegAlloc,
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
    let dst_mir = match dst_mir {
        InstRetval::Reg(reg) => reg,
        InstRetval::Wasted => {
            debug!("Load instruction {ir_ref:?} is wasted, skipping load generation");
            return;
        }
    };
    let dst_kind = LdrDest::from_reg_operand(dst_mir);
    let ldr_inst = match dst_kind {
        LdrDest::F32(dst) => match src_mir {
            MirOperand::GPReg(gpreg) => LoadF32::new(
                MirOP::LdrF32,
                dst,
                GPR64::from_real(gpreg),
                GPR64::zr(),
                None,
            )
            .into_mir(),
            MirOperand::Label(label) => {
                LoadF32Literal::new(MirOP::LdrF32Literal, dst, MirSymbolOp::Label(label)).into_mir()
            }
            MirOperand::Global(globl) => {
                let tmp_addr = vreg_alloc.insert_gpr64(GPR64::new_empty());
                let src: MirSymbolOp = MirSymbolOp::Global(globl);
                MirLdrLitF32::new(MirOP::MirLdrLitF32, dst, tmp_addr, src).into_mir()
            }
            _ => panic!("Invalid source operand load from {src_mir:?} to {dst_mir:?}"),
        },
        LdrDest::F64(fpr64) => match src_mir {
            MirOperand::GPReg(gpreg) => LoadF64Base::new(
                MirOP::LdrF64Base,
                fpr64,
                GPR64::from_real(gpreg),
                ImmLSP64::new(0),
            )
            .into_mir(),
            MirOperand::Label(label) => {
                LoadF64Literal::new(MirOP::LdrF64Literal, fpr64, MirSymbolOp::Label(label))
                    .into_mir()
            }
            MirOperand::Global(globl) => {
                let tmp_addr = vreg_alloc.insert_gpr64(GPR64::new_empty());
                let src = MirSymbolOp::Global(globl);
                MirLdrLitF64::new(MirOP::MirLdrLitF64, fpr64, tmp_addr, src).into_mir()
            }
            _ => panic!("Invalid source operand load from {src_mir:?} to {dst_mir:?}"),
        },
        LdrDest::G32(gpr32) => match src_mir {
            MirOperand::GPReg(gpreg) => LoadGr32Base::new(
                MirOP::LdrGr32Base,
                gpr32,
                GPR64::from_real(gpreg),
                ImmLSP32::new(0),
            )
            .into_mir(),
            MirOperand::Label(label) => {
                LoadGr32Literal::new(MirOP::LdrGr32Literal, gpr32, MirSymbolOp::Label(label))
                    .into_mir()
            }
            MirOperand::Global(globl) => {
                let tmp_addr = GPR64::new(gpr32.get_id());
                let src = MirSymbolOp::Global(globl);
                MirLdrLitG32::new(MirOP::MirLdrLitG32, gpr32, tmp_addr, src).into_mir()
            }
            _ => panic!("Invalid source operand load from {src_mir:?} to {dst_mir:?}"),
        },
        LdrDest::G64(dst) => match src_mir {
            MirOperand::GPReg(gpreg) => LoadGr64Base::new(
                MirOP::LdrGr64Base,
                dst,
                GPR64::from_real(gpreg),
                ImmLSP64::new(0),
            )
            .into_mir(),
            MirOperand::Label(label) => {
                LoadGr64Literal::new(MirOP::LdrGr64Literal, dst, MirSymbolOp::Label(label))
                    .into_mir()
            }
            MirOperand::Global(globl) => {
                let src = MirSymbolOp::Global(globl);
                MirLdrLitG64::new(MirOP::MirLdrLitG64, dst, dst, src).into_mir()
            }
            MirOperand::SwitchTab(index) => {
                let src = MirSymbolOp::SwitchTab(index);
                MirLdrLitG64::new(MirOP::MirLdrLitG64, dst, dst, src).into_mir()
            }
            _ => panic!("Invalid source operand load from {src_mir:?} to {dst_mir:?}"),
        },
    };
    out_insts.push_back(ldr_inst);
}

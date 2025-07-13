use super::InstDispatchError;
use crate::{
    base::slabref::SlabRef,
    ir::{
        inst::{InstData, InstRef, usedef::UseData},
        module::Module,
    },
    mir::{
        inst::{
            IMirSubInst,
            impls::{
                LoadStoreF32, LoadStoreF32Base, LoadStoreF32Literal, LoadStoreF64Base,
                LoadStoreF64Literal, LoadStoreGr32Base, LoadStoreGr32Literal, LoadStoreGr64Base,
                LoadStoreGr64Literal,
            },
            inst::MirInst,
            opcode::MirOP,
        },
        module::{MirGlobalRef, stack::VirtRegAlloc},
        operand::{
            MirOperand,
            compound::MirSymbolOp,
            imm::{ImmLoad32, ImmLoad64},
            reg::{FPR32, FPR64, GPR32, GPR64, RegOperand},
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
    type StrSrc = crate::mir::translate::mirgen::operandgen::PureSourceReg;

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
    let store_inst = match dst_mir {
        StrDest::G64(dst_ptr) => match src_mir {
            StrSrc::F32(fpr32) => {
                LoadStoreF32Base::new(MirOP::StrF32, fpr32, dst_ptr, zoff32).into_mir()
            }
            StrSrc::F64(fpr64) => {
                LoadStoreF64Base::new(MirOP::StrF64, fpr64, dst_ptr, zoff64).into_mir()
            }
            StrSrc::G32(gpr32) => {
                LoadStoreGr32Base::new(MirOP::StrGr32, gpr32, dst_ptr, zoff32).into_mir()
            }
            StrSrc::G64(gpr64) => {
                LoadStoreGr64Base::new(MirOP::StrGr64, gpr64, dst_ptr, zoff64).into_mir()
            }
        },
        StrDest::Global(global) => {
            let global = MirSymbolOp::Global(global);
            match src_mir {
                StrSrc::F32(fpr32) => {
                    LoadStoreF32Literal::new(MirOP::StrF32Literal, fpr32, global).into_mir()
                }
                StrSrc::F64(fpr64) => {
                    LoadStoreF64Literal::new(MirOP::StrF64Literal, fpr64, global).into_mir()
                }
                StrSrc::G32(gpr32) => {
                    LoadStoreGr32Literal::new(MirOP::StrGr32Literal, gpr32, global).into_mir()
                }
                StrSrc::G64(gpr64) => {
                    LoadStoreGr64Literal::new(MirOP::StrGr64Literal, gpr64, global).into_mir()
                }
            }
        }
    };
    out_insts.push_back(store_inst);
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
            MirOperand::GPReg(gpreg) => LoadStoreF32::new(
                MirOP::LdrF32,
                fpr32,
                GPR64::from_real(gpreg),
                GPR64::zr(),
                None,
            )
            .into_mir(),
            MirOperand::Label(label) => {
                LoadStoreF32Literal::new(MirOP::LdrF32Literal, fpr32, MirSymbolOp::Label(label))
                    .into_mir()
            }
            MirOperand::Global(globl) => {
                LoadStoreF32Literal::new(MirOP::LdrF32Literal, fpr32, MirSymbolOp::Global(globl))
                    .into_mir()
            }
            _ => panic!("Invalid source operand load from {src_mir:?} to {dst_mir:?}"),
        },
        LdrDest::F64(fpr64) => match src_mir {
            MirOperand::GPReg(gpreg) => LoadStoreF64Base::new(
                MirOP::LdrF64Base,
                fpr64,
                GPR64::from_real(gpreg),
                ImmLoad64::new(0),
            )
            .into_mir(),
            MirOperand::Label(label) => {
                LoadStoreF64Literal::new(MirOP::LdrF64Literal, fpr64, MirSymbolOp::Label(label))
                    .into_mir()
            }
            MirOperand::Global(globl) => {
                LoadStoreF64Literal::new(MirOP::LdrF64Literal, fpr64, MirSymbolOp::Global(globl))
                    .into_mir()
            }
            _ => panic!("Invalid source operand load from {src_mir:?} to {dst_mir:?}"),
        },
        LdrDest::G32(gpr32) => match src_mir {
            MirOperand::GPReg(gpreg) => LoadStoreGr32Base::new(
                MirOP::LdrGr32Base,
                gpr32,
                GPR64::from_real(gpreg),
                ImmLoad32::new(0),
            )
            .into_mir(),
            MirOperand::Label(label) => {
                LoadStoreGr32Literal::new(MirOP::LdrGr32Literal, gpr32, MirSymbolOp::Label(label))
                    .into_mir()
            }
            MirOperand::Global(globl) => {
                LoadStoreGr32Literal::new(MirOP::LdrGr32Literal, gpr32, MirSymbolOp::Global(globl))
                    .into_mir()
            }
            _ => panic!("Invalid source operand load from {src_mir:?} to {dst_mir:?}"),
        },
        LdrDest::G64(gpr64) => match src_mir {
            MirOperand::GPReg(gpreg) => LoadStoreGr64Base::new(
                MirOP::LdrGr64Base,
                gpr64,
                GPR64::from_real(gpreg),
                ImmLoad64::new(0),
            )
            .into_mir(),
            MirOperand::Label(label) => {
                LoadStoreGr64Literal::new(MirOP::LdrGr64Literal, gpr64, MirSymbolOp::Label(label))
                    .into_mir()
            }
            MirOperand::Global(globl) => {
                LoadStoreGr64Literal::new(MirOP::LdrGr64Literal, gpr64, MirSymbolOp::Global(globl))
                    .into_mir()
            }
            MirOperand::SwitchTab(index) => LoadStoreGr64Literal::new(
                MirOP::LdrGr64Literal,
                gpr64,
                MirSymbolOp::SwitchTab(index),
            )
            .into_mir(),
            _ => panic!("Invalid source operand load from {src_mir:?} to {dst_mir:?}"),
        },
    };
    out_insts.push_back(ldr_inst);
}

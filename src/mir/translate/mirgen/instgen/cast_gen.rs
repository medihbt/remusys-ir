use super::InstDispatchError;
use crate::{
    base::{slabref::SlabRef, NullableValue},
    ir::{
        constant::data::ConstData, inst::{cmp::CmpOp, usedef::UseData, InstData, InstRef}, module::Module, opcode::Opcode, ValueSSA
    },
    mir::{
        inst::{cond::MirCondFlag, impls::*, inst::MirInst, opcode::MirOP, IMirSubInst},
        module::vreg_alloc::VirtRegAlloc,
        operand::{
            imm::ImmLogic, reg::{PState, GPR32, GPR64}, IMirSubOperand
        },
        translate::mirgen::{
            instgen::ir_value_as_cmp,
            operandgen::{DispatchedReg, InstRetval, OperandMap},
        },
    },
    typing::{id::ValTypeID, types::FloatTypeKind},
};
use slab::Slab;
use std::{cell::Ref, collections::VecDeque};

pub(crate) fn dispatch_casts(
    ir_module: &Module,
    operand_map: &OperandMap,
    vreg_alloc: &mut VirtRegAlloc,
    out_insts: &mut VecDeque<MirInst>,
    ir_ref: InstRef,
    alloc_inst: &Slab<InstData>,
    alloc_use: Ref<Slab<UseData>>,
    last_pstate_modifier: Option<InstRef>,
) -> Option<Result<(), InstDispatchError>> {
    use DispatchedReg::*;
    fn int_cast_get_bits(src_ty: ValTypeID, dst_ty: ValTypeID) -> (u8, u8) {
        use ValTypeID::Int;
        match (src_ty, dst_ty) {
            (Int(s), Int(d)) => (s, d),
            _ => panic!("Invalid types for integer cast: {src_ty:?} to {dst_ty:?}"),
        }
    }
    let (opcode, dst_ty, inst) = match ir_ref.to_slabref_unwrap(alloc_inst) {
        InstData::Cast(c, i) => (c.opcode, c.ret_type, i),
        _ => panic!("Expected Cast instruction"),
    };
    let src_ir = inst.from_op.get_operand(&alloc_use);
    let src_ty = src_ir.get_value_type(ir_module);
    if src_ty == dst_ty {
        return Some(Ok(())); // No cast needed if types match
    }
    let dst_mir = operand_map
        .find_operand_for_inst(ir_ref)
        .expect("Failed to find destination operand for cast instruction");
    let dst_mir = match dst_mir {
        InstRetval::Reg(reg) => reg,
        InstRetval::Wasted => return Some(Ok(())), // If the result is wasted, no need to generate a cast
    };
    let dst_mir = DispatchedReg::from_reg(dst_mir);

    if let Some(cmp) = ir_value_as_cmp(src_ir, alloc_inst) {
        let cmp_ref = match src_ir {
            ValueSSA::Inst(i) => i,
            _ => unreachable!("Expected source operand to be an instruction"),
        };
        if InstRef::from_option(last_pstate_modifier) == cmp_ref {
            return dispach_cast_cmp_to_int(ir_module, vreg_alloc, out_insts, dst_mir, cmp);
        }
    }

    let src_mir = DispatchedReg::from_valuessa(
        operand_map,
        &ir_module.type_ctx,
        vreg_alloc,
        out_insts,
        &src_ir,
        true,
    )
    .expect("Failed to convert source operand to MIR");
    let castinst = match opcode {
        Opcode::Zext => match (dst_mir, src_mir) {
            (G32(_), G32(_)) | (G32(_), G64(_)) | (G64(_), G64(_)) => {
                return Some(Ok(()));
            }
            (G64(dst), G32(src)) => {
                let (src_bits, dst_bits) = int_cast_get_bits(src_ty, dst_ty);
                if src_bits != 32 || dst_bits != 64 {
                    panic!("Unsupported Zext cast: {src_ty:?} to {dst_ty:?}");
                }
                let GPR32(src_id, src_uf) = src;
                Bin64RL::new(
                    MirOP::And64I,
                    dst,
                    GPR64(src_id, src_uf),
                    ImmLogic::new(0xFFFFFFFF),
                )
                .into_mir()
            }
            _ => panic!("Invalid Zext cast: {src_mir:?} to {dst_mir:?}"),
        },
        Opcode::Sext => match (dst_mir, src_mir) {
            (G32(_), G32(_)) | (G32(_), G64(_)) | (G64(_), G64(_)) => {
                return Some(Ok(()));
            }
            (G64(dst), G32(src)) => ExtR::new(MirOP::SXTW64, dst, src).into_mir(),
            _ => panic!("Invalid Sext cast: {src_mir:?} to {dst_mir:?}"),
        },
        Opcode::Trunc => {
            let (src_bits, dst_bits) = match (src_ty, dst_ty) {
                (ValTypeID::Int(s), ValTypeID::Int(d)) => (s, d),
                _ => panic!("Invalid types for truncation: {src_ty:?} to {dst_ty:?}"),
            };
            let (dst_id, dst_uf, src_id, src_uf) = match (dst_mir, src_mir) {
                (G32(dst), G32(src)) => (dst.0, dst.1, src.0, src.1),
                (G32(dst), G64(src)) => (dst.0, dst.1, src.0, src.1),
                (G64(dst), G32(src)) => (dst.0, dst.1, src.0, src.1),
                (G64(dst), G64(src)) => (dst.0, dst.1, src.0, src.1),
                _ => panic!("Invalid truncation operands: {src_mir:?} to {dst_mir:?}"),
            };
            assert!(src_bits > dst_bits, "Truncation must reduce bit width");
            let imm = ImmLogic::new((1 << dst_bits) - 1);
            let dst = GPR64(dst_id, dst_uf);
            let src = GPR64(src_id, src_uf);
            Bin64RL::new(MirOP::And64I, dst, src, imm).into_mir()
        }
        Opcode::Fpext => {
            // FP 寄存器的类型直接对应 FloatTypeKind 的几个变体, 不需要像 int
            // 那样处理位宽
            match (dst_mir, src_mir) {
                (F64(dst), F32(src)) => UnaryF64F32::new(MirOP::FCvt64F32, dst, src).into_mir(),
                _ => panic!("Invalid Fpext cast: {src_mir:?} to {dst_mir:?}"),
            }
        }
        Opcode::Fptrunc => match (dst_mir, src_mir) {
            (F32(dst), F64(src)) => UnaryF32F64::new(MirOP::FCvt32F64, dst, src).into_mir(),
            _ => panic!("Invalid Fptrunc cast: {src_mir:?} to {dst_mir:?}"),
        },
        Opcode::Sitofp => match (dst_mir, src_mir) {
            (F32(dst), G32(src)) => UnaFG32::new(MirOP::SCvtF32, dst, src).into_mir(),
            (F32(dst), G64(src)) => UnaF32G64::new(MirOP::SCvtF32G64, dst, src).into_mir(),
            (F64(dst), G32(src)) => UnaF64G32::new(MirOP::SCvtF64G32, dst, src).into_mir(),
            (F64(dst), G64(src)) => UnaFG64::new(MirOP::SCvtF64, dst, src).into_mir(),
            _ => panic!("Invalid Sitofp cast: {src_mir:?} to {dst_mir:?}"),
        },
        Opcode::Uitofp => match (dst_mir, src_mir) {
            (F32(dst), G32(src)) => UnaFG32::new(MirOP::UCvtF32, dst, src).into_mir(),
            (F32(dst), G64(src)) => UnaF32G64::new(MirOP::UCvtF32G64, dst, src).into_mir(),
            (F64(dst), G32(src)) => UnaF64G32::new(MirOP::UCvtF64G32, dst, src).into_mir(),
            (F64(dst), G64(src)) => UnaFG64::new(MirOP::UCvtF64, dst, src).into_mir(),
            _ => panic!("Invalid Uitofp cast: {src_mir:?} to {dst_mir:?}"),
        },
        Opcode::Fptosi => match (dst_mir, src_mir) {
            (G32(dst), F32(src)) => UnaGF32::new(MirOP::FCvtAS32, dst, src).into_mir(),
            (G64(dst), F32(src)) => UnaG64F32::new(MirOP::FCvtAS64F32, dst, src).into_mir(),
            (G32(dst), F64(src)) => UnaG32F64::new(MirOP::FCvtAS32F64, dst, src).into_mir(),
            (G64(dst), F64(src)) => UnaGF64::new(MirOP::FCvtAS64, dst, src).into_mir(),
            _ => panic!("Invalid Fptosi cast: {src_mir:?} to {dst_mir:?}"),
        },
        _ => panic!("Unexpected opcode for Cast instruction: {opcode:?}"),
    };
    out_insts.push_back(castinst);
    None
}

fn dispach_cast_cmp_to_int(
    ir_module: &Module,
    vreg_alloc: &mut VirtRegAlloc,
    out_insts: &mut VecDeque<MirInst>,
    dst_mir: DispatchedReg,
    cmp: &CmpOp,
) -> Option<Result<(), InstDispatchError>> {
    use DispatchedReg::*;

    let cmp_cond = cmp.cond;
    let select_cond = MirCondFlag::from_cmp_cond(cmp_cond);
    let cset_inst = match dst_mir {
        F32(fpr32) => {
            let rn = match DispatchedReg::from_constdata(
                &ConstData::Float(FloatTypeKind::Ieee32, 0.0),
                &ir_module.type_ctx,
                vreg_alloc,
                out_insts,
                true,
            ) {
                DispatchedReg::F32(fpr32) => fpr32,
                _ => panic!("Expected source operand to be F32"),
            };
            let rm = match DispatchedReg::from_constdata(
                &ConstData::Float(FloatTypeKind::Ieee32, 1.0),
                &ir_module.type_ctx,
                vreg_alloc,
                out_insts,
                true,
            ) {
                DispatchedReg::F32(fpr32) => fpr32,
                _ => panic!("Expected source operand to be F32"),
            };
            CSelF32::new(
                MirOP::CSelF32,
                fpr32,
                rn,
                rm,
                PState::new_empty(),
                select_cond,
            )
            .into_mir()
        }
        F64(fpr64) => {
            let rn = match DispatchedReg::from_constdata(
                &ConstData::Float(FloatTypeKind::Ieee64, 0.0),
                &ir_module.type_ctx,
                vreg_alloc,
                out_insts,
                true,
            ) {
                DispatchedReg::F64(fpr64) => fpr64,
                _ => panic!("Expected source operand to be F64"),
            };
            let rm = match DispatchedReg::from_constdata(
                &ConstData::Float(FloatTypeKind::Ieee64, 1.0),
                &ir_module.type_ctx,
                vreg_alloc,
                out_insts,
                true,
            ) {
                DispatchedReg::F64(fpr64) => fpr64,
                _ => panic!("Expected source operand to be F64"),
            };
            CSelF64::new(
                MirOP::CSelF64,
                fpr64,
                rn,
                rm,
                PState::new_empty(),
                select_cond,
            )
            .into_mir()
        }
        G32(gpr32) => {
            CSet32::new(MirOP::CSet32, gpr32, PState::new_empty(), select_cond).into_mir()
        }
        G64(gpr64) => {
            CSet64::new(MirOP::CSet64, gpr64, PState::new_empty(), select_cond).into_mir()
        }
    };
    out_insts.push_back(cset_inst);
    Some(Ok(()))
}

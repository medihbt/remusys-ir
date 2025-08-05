use crate::{
    base::SlabRef,
    ir::{
        ValueSSA,
        constant::data::ConstData,
        inst::{InstData, InstRef, UseData},
        module::Module,
        opcode::Opcode as O,
    },
    mir::{
        inst::{IMirSubInst, impls::*, inst::MirInst, opcode::MirOP},
        module::vreg_alloc::VirtRegAlloc,
        operand::{IMirSubOperand, imm::ImmCalc, imm_traits, reg::*},
        translate::mirgen::operandgen::{DispatchedReg, InstRetval, OperandMap},
    },
    typing::{context::TypeContext, id::ValTypeID},
};
use log::debug;
use slab::Slab;
use std::{cell::Ref, collections::VecDeque};

type BinLHS = crate::mir::translate::mirgen::operandgen::DispatchedReg;

pub(super) fn dispatch_binaries(
    operand_map: &OperandMap,
    ir_module: &Module,
    vreg_alloc: &mut VirtRegAlloc,
    out_insts: &mut VecDeque<MirInst>,
    ir_ref: InstRef,
    alloc_inst: &Slab<InstData>,
    alloc_use: Ref<Slab<UseData>>,
) {
    let (opcode, inst) = match ir_ref.to_data(alloc_inst) {
        InstData::BinOp(c, b) => (c.opcode, b),
        _ => panic!("Expected BinOp instruction"),
    };
    let lhs_ir = inst.lhs.get_operand(&alloc_use);
    let rhs_ir = inst.rhs.get_operand(&alloc_use);

    let res = operand_map
        .find_operand_for_inst(ir_ref)
        .expect("Failed to find MIR operand for binary instruction");
    let res = match res {
        InstRetval::Reg(res) => res,
        InstRetval::Wasted => {
            debug!("Binary operation {ir_ref:?} is wasted, skipping generation.");
            return;
        }
    };

    let lhs_mir = BinLHS::from_valuessa(
        operand_map,
        &ir_module.type_ctx,
        vreg_alloc,
        out_insts,
        &lhs_ir,
        true,
    )
    .expect("Failed to find LHS operand for binary operation");

    let mut gen_ctx = BinGenContext {
        res,
        opcode,
        lhs_mir,
        rhs_ir,
        vreg_alloc,
        out_insts,
        operand_map,
        type_ctx: &ir_module.type_ctx,
    };

    match opcode {
        O::Add | O::Sub => gen_ctx.generate_iaddsub(),
        O::Fadd | O::Fsub => gen_ctx.generate_faddsub(),
        O::Mul | O::Sdiv | O::Udiv => gen_ctx.generate_imuldiv(),
        O::Fmul | O::Fdiv => gen_ctx.generate_fmuldiv(),
        O::Srem | O::Urem => gen_ctx.generate_irem(),
        O::Frem => gen_ctx.generate_frem(),
        O::BitAnd | O::BitOr | O::BitXor => gen_ctx.generate_bitwise(),
        O::Shl | O::Lshr | O::Ashr => gen_ctx.generate_shift(),
        _ => panic!("Unsupported binary operation: {opcode:?}"),
    };
}

struct BinGenContext<'a> {
    res: RegOperand,
    opcode: O,
    lhs_mir: BinLHS,
    rhs_ir: ValueSSA,
    vreg_alloc: &'a mut VirtRegAlloc,
    out_insts: &'a mut VecDeque<MirInst>,
    operand_map: &'a OperandMap<'a>,
    type_ctx: &'a TypeContext,
}

impl<'a> BinGenContext<'a> {
    fn generate_iaddsub(mut self) {
        let res: GPReg = self.res.into();
        let opcode = self.opcode;
        let inst = match Self::value_as_imm_calc(&self.rhs_ir) {
            Some(value) => match self.lhs_mir {
                BinLHS::G32(lhs) => {
                    let opcode = match self.opcode {
                        O::Add => MirOP::Add32I,
                        O::Sub => MirOP::Sub32I,
                        _ => panic!("Unsupported integer binary operation: {opcode:?}"),
                    };
                    Bin32RC::new(opcode, GPR32::from_real(res), lhs, value).into_mir()
                }
                BinLHS::G64(lhs) => {
                    let opcode = match self.opcode {
                        O::Add => MirOP::Add64I,
                        O::Sub => MirOP::Sub64I,
                        _ => panic!("Unsupported integer binary operation: {opcode:?}"),
                    };
                    Bin64RC::new(opcode, GPR64::from_real(res), lhs, value).into_mir()
                }
                _ => panic!(
                    "Unsupported binary operation for operand: {:?}",
                    self.lhs_mir
                ),
            },
            None => {
                let lhs = self.lhs_mir;
                let rhs = self.make_rhs_mir();
                self.do_generate_iaddsub_by_mir(res, opcode, lhs, rhs)
            }
        };
        self.out_insts.push_back(inst);
    }

    fn do_generate_iaddsub_by_mir(
        &mut self,
        res: GPReg,
        opcode: O,
        lhs: DispatchedReg,
        rhs: DispatchedReg,
    ) -> MirInst {
        use DispatchedReg::*;
        match (lhs, rhs) {
            (G32(lhs), G32(rhs)) => {
                let opcode = match self.opcode {
                    O::Add => MirOP::Add32R,
                    O::Sub => MirOP::Sub32R,
                    _ => panic!("Unsupported integer binary operation: {opcode:?}"),
                };
                Bin32R::new(opcode, GPR32::from_real(res), lhs, rhs, None).into_mir()
            }
            (G64(lhs), G64(rhs)) => {
                let opcode = match self.opcode {
                    O::Add => MirOP::Add64R,
                    O::Sub => MirOP::Sub64R,
                    _ => panic!("Unsupported integer binary operation: {opcode:?}"),
                };
                Bin64R::new(opcode, GPR64::from_real(res), lhs, rhs, None).into_mir()
            }
            _ => {
                panic!("Unsupported binary operation for operands: {lhs:?}, {rhs:?}");
            }
        }
    }

    fn generate_faddsub(mut self) {
        let lhs = self.lhs_mir;
        let rhs = self.make_rhs_mir();
        let res: VFReg = self.res.into();
        let opcode = self.opcode;

        let inst = Self::do_generate_faddsub_by_mir(lhs, rhs, res, opcode);
        self.out_insts.push_back(inst);
    }
    fn do_generate_faddsub_by_mir(
        lhs: DispatchedReg,
        rhs: DispatchedReg,
        res: VFReg,
        opcode: O,
    ) -> MirInst {
        use DispatchedReg::*;
        let inst = match (lhs, rhs) {
            (F32(lhs), F32(rhs)) => match opcode {
                O::Fadd => BinF32R::new(MirOP::FAdd32, FPR32::from_real(res), lhs, rhs).into_mir(),
                O::Fsub => BinF32R::new(MirOP::FSub32, FPR32::from_real(res), lhs, rhs).into_mir(),
                _ => panic!("Unsupported floating-point binary operation: {opcode:?}"),
            },
            (F64(lhs), F64(rhs)) => match opcode {
                O::Fadd => BinF64R::new(MirOP::FAdd64, FPR64::from_real(res), lhs, rhs).into_mir(),
                O::Fsub => BinF64R::new(MirOP::FSub64, FPR64::from_real(res), lhs, rhs).into_mir(),
                _ => panic!("Unsupported floating-point binary operation: {opcode:?}"),
            },
            _ => panic!("Unsupported binary operation for operands: {lhs:?}, {rhs:?}"),
        };
        inst
    }

    fn generate_imuldiv(&mut self) {
        let lhs = self.lhs_mir;
        let rhs = self.make_rhs_mir();
        let res: GPReg = self.res.into();
        let opcode = self.opcode;

        let inst = match (lhs, rhs) {
            (BinLHS::G32(lhs), BinLHS::G32(rhs)) => {
                let opcode = match opcode {
                    O::Mul => MirOP::Mul32,
                    O::Sdiv => MirOP::SDiv32,
                    O::Udiv => MirOP::UDiv32,
                    _ => panic!("Unsupported integer binary operation: {opcode:?}"),
                };
                Bin32R::new(opcode, GPR32::from_real(res), lhs, rhs, None).into_mir()
            }
            (BinLHS::G64(lhs), BinLHS::G64(rhs)) => {
                let opcode = match opcode {
                    O::Mul => MirOP::Mul64,
                    O::Sdiv => MirOP::SDiv64,
                    O::Udiv => MirOP::UDiv64,
                    _ => panic!("Unsupported integer binary operation: {opcode:?}"),
                };
                Bin64R::new(opcode, GPR64::from_real(res), lhs, rhs, None).into_mir()
            }
            _ => panic!("Unsupported binary operation for operands: {lhs:?}, {rhs:?}"),
        };
        self.out_insts.push_back(inst);
    }

    fn generate_fmuldiv(&mut self) {
        use DispatchedReg::*;
        let lhs = self.lhs_mir;
        let rhs = self.make_rhs_mir();
        let res: VFReg = self.res.into();
        let opcode = self.opcode;

        let inst = match (lhs, rhs) {
            (F32(lhs), F32(rhs)) => match opcode {
                O::Fmul => BinF32R::new(MirOP::FMul32, FPR32::from_real(res), lhs, rhs).into_mir(),
                O::Fdiv => BinF32R::new(MirOP::FDiv32, FPR32::from_real(res), lhs, rhs).into_mir(),
                _ => panic!("Unsupported floating-point binary operation: {opcode:?}"),
            },
            (F64(lhs), F64(rhs)) => match opcode {
                O::Fmul => BinF64R::new(MirOP::FMul64, FPR64::from_real(res), lhs, rhs).into_mir(),
                O::Fdiv => BinF64R::new(MirOP::FDiv64, FPR64::from_real(res), lhs, rhs).into_mir(),
                _ => panic!("Unsupported floating-point binary operation: {opcode:?}"),
            },
            _ => panic!("Unsupported binary operation for operands: {lhs:?}, {rhs:?}"),
        };
        self.out_insts.push_back(inst);
    }

    fn generate_irem(mut self) {
        // AArch64 has no instruction for integer remainder,
        // so we use a division followed by a multiplication and subtraction.

        // Step 1: Generate `div res, lhs, rhs`
        let old_opcode = self.opcode;
        self.opcode = match old_opcode {
            O::Srem => O::Sdiv,
            O::Urem => O::Udiv,
            _ => panic!("Unsupported integer remainder operation: {old_opcode:?}"),
        };
        self.generate_imuldiv();
        let res = self.res;
        let lhs = self.lhs_mir;
        let res_pure = DispatchedReg::from_reg(res);

        // Step 2: Generate `mul res, res, rhs`
        self.opcode = O::Mul;
        self.lhs_mir = res_pure;
        self.generate_imuldiv();

        // Step 3: Generate `sub res, lhs, res`
        self.opcode = O::Sub;
        let inst = self.do_generate_iaddsub_by_mir(res.into(), O::Sub, lhs, res_pure);
        self.out_insts.push_back(inst);
    }

    fn generate_frem(mut self) {
        // AArch64 has no instruction for floating-point remainder,
        // so we use a division followed by a multiplication and subtraction.

        // Step 1: Generate `fdiv res, lhs, rhs`
        let old_opcode = self.opcode;
        self.opcode = match old_opcode {
            O::Frem => O::Fdiv,
            _ => panic!("Unsupported floating-point remainder operation: {old_opcode:?}"),
        };
        self.generate_fmuldiv();
        let res = self.res;
        let lhs = self.lhs_mir;
        let res_pure = DispatchedReg::from_reg(res);

        // Step 2: Generate `fmul res, res, rhs`
        self.opcode = O::Fmul;
        self.lhs_mir = res_pure;
        self.generate_fmuldiv();

        // Step 3: Generate `fsub res, lhs, res`
        self.opcode = O::Fsub;
        let inst = Self::do_generate_faddsub_by_mir(lhs, res_pure, res.into(), O::Fsub);
        self.out_insts.push_back(inst);
    }

    fn generate_bitwise(mut self) {
        use DispatchedReg::*;
        let lhs = self.lhs_mir;
        let rhs = self.make_rhs_mir();
        let res: GPReg = self.res.into();
        let opcode = self.opcode;

        let inst = match (lhs, rhs) {
            (G32(lhs), G32(rhs)) => {
                let opcode = match opcode {
                    O::BitAnd => MirOP::And32R,
                    O::BitOr => MirOP::ORR32R,
                    O::BitXor => MirOP::EOR32R,
                    _ => panic!("Unsupported bitwise operation: {opcode:?}"),
                };
                Bin32R::new(opcode, GPR32::from_real(res), lhs, rhs, None).into_mir()
            }
            (G64(lhs), G64(rhs)) => {
                let opcode = match opcode {
                    O::BitAnd => MirOP::And64R,
                    O::BitOr => MirOP::ORR64R,
                    O::BitXor => MirOP::EOR64R,
                    _ => panic!("Unsupported bitwise operation: {opcode:?}"),
                };
                Bin64R::new(opcode, GPR64::from_real(res), lhs, rhs, None).into_mir()
            }
            _ => panic!("Unsupported binary operation for operands: {lhs:?}, {rhs:?}"),
        };
        self.out_insts.push_back(inst);
    }

    fn generate_shift(mut self) {
        use DispatchedReg::*;
        let lhs = self.lhs_mir;
        let rhs = self.make_rhs_mir();
        let res: GPReg = self.res.into();
        let opcode = self.opcode;

        let inst = match (lhs, rhs) {
            (G32(lhs), G32(rhs)) => {
                let opcode = match opcode {
                    O::Shl => MirOP::Lsl32R,
                    O::Lshr => MirOP::Lsr32R,
                    O::Ashr => MirOP::Asr32R,
                    _ => panic!("Unsupported shift operation: {opcode:?}"),
                };
                Bin32R::new(opcode, GPR32::from_real(res), lhs, rhs, None).into_mir()
            }
            (G64(lhs), G64(rhs)) => {
                let opcode = match opcode {
                    O::Shl => MirOP::Lsl64R,
                    O::Lshr => MirOP::Lsr64R,
                    O::Ashr => MirOP::Asr64R,
                    _ => panic!("Unsupported shift operation: {opcode:?}"),
                };
                Bin64R::new(opcode, GPR64::from_real(res), lhs, rhs, None).into_mir()
            }
            _ => panic!("Unsupported binary operation for operands: {lhs:?}, {rhs:?}"),
        };
        self.out_insts.push_back(inst);
    }

    fn make_rhs_mir(&mut self) -> BinLHS {
        BinLHS::from_valuessa(
            self.operand_map,
            self.type_ctx,
            self.vreg_alloc,
            self.out_insts,
            &self.rhs_ir,
            true,
        )
        .expect("Failed to find RHS operand for binary operation")
    }

    fn value_as_constdata(value: &ValueSSA) -> Option<&ConstData> {
        match value {
            ValueSSA::ConstData(c) => Some(c),
            _ => None,
        }
    }
    fn constdata_as_imm_calc(data: &ConstData) -> Option<ImmCalc> {
        match data {
            ConstData::Zero(ValTypeID::Int(bits)) if *bits <= 64 => Some(ImmCalc(0)),
            ConstData::PtrNull(_) => Some(ImmCalc(0)),
            ConstData::Int(bits, value) if *bits <= 64 => {
                let value = ConstData::iconst_value_get_real_signed(*bits, *value) as u64;
                if imm_traits::is_calc_imm(value) { Some(ImmCalc(value as u32)) } else { None }
            }
            _ => None,
        }
    }
    fn value_as_imm_calc(value: &ValueSSA) -> Option<ImmCalc> {
        Self::value_as_constdata(value).and_then(Self::constdata_as_imm_calc)
    }
}

use std::{collections::VecDeque, rc::Rc};

use crate::{
    ir::{ValueSSA, block::BlockRef, constant::data::ConstData, global::GlobalRef, inst::InstRef},
    mir::{
        inst::{IMirSubInst, impls::*, inst::MirInst, opcode::MirOP},
        module::{MirGlobalRef, block::MirBlockRef, func::MirFunc, stack::VirtRegAlloc},
        operand::{
            IMirSubOperand, MirOperand,
            imm::{Imm32, Imm64, ImmFMov32, ImmFMov64, ImmKind},
            imm_traits::{try_cast_f32_to_aarch8, try_cast_f64_to_aarch8},
            reg::{FPR32, FPR64, GPR32, GPR64, GPReg, RegOperand, RegUseFlags, SubRegIndex, VFReg},
        },
        translate::mirgen::{MirBlockInfo, globalgen::MirGlobalItems, instgen::make_copy_inst},
    },
    typing::{context::TypeContext, types::FloatTypeKind},
};

pub struct OperandMap<'a> {
    pub args: Vec<(u32, RegOperand)>,
    pub func: Rc<MirFunc>,
    pub globals: &'a MirGlobalItems,
    pub insts: Vec<(InstRef, RegOperand)>,
    pub blocks: Vec<MirBlockInfo>,
}

#[derive(Debug, Clone)]
pub enum OperandMapError {
    IsConstData(ConstData),
    OperandUndefined,
    IsNone,
    IsUnsupported(ValueSSA),
    IsNotFound(ValueSSA),
}

impl<'a> OperandMap<'a> {
    pub fn new(
        func: Rc<MirFunc>,
        globals: &'a MirGlobalItems,
        insts: Vec<(InstRef, RegOperand)>,
        blocks: Vec<MirBlockInfo>,
    ) -> Self {
        debug_assert!(insts.is_sorted_by_key(|(inst, _)| *inst));
        debug_assert!(blocks.is_sorted_by_key(|b| b.ir));

        let nargs = func.arg_ir_types.len();
        let mut args = Vec::with_capacity(nargs);
        let mut arg_id = 0u32;
        for &preg in &func.arg_regs {
            args.push((arg_id, preg));
            arg_id += 1;
        }
        for spilled_arg in func.borrow_spilled_args().iter() {
            args.push((arg_id, spilled_arg.virtreg));
            arg_id += 1;
        }
        Self {
            args,
            func,
            globals,
            insts,
            blocks,
        }
    }

    pub fn find_operand_for_inst(&self, inst: InstRef) -> Option<RegOperand> {
        self.insts
            .binary_search_by_key(&inst, |(i, _)| *i)
            .ok()
            .map(|idx| self.insts[idx].1)
    }
    pub fn find_operand_for_arg(&self, arg_id: u32) -> Option<RegOperand> {
        self.args
            .binary_search_by_key(&arg_id, |(id, _)| *id)
            .ok()
            .map(|idx| self.args[idx].1)
    }
    pub fn find_operand_for_global(&self, gref: GlobalRef) -> Option<MirGlobalRef> {
        self.globals.find_mir_ref(gref)
    }
    pub fn find_function(&self, func: GlobalRef) -> Option<(Rc<MirFunc>, MirGlobalRef)> {
        self.globals.find_func(func).map(|f| (f.rc.clone(), f.mir))
    }
    pub fn find_operand_for_block(&self, block: BlockRef) -> Option<MirBlockRef> {
        self.blocks
            .binary_search_by_key(&block, |b| b.ir)
            .ok()
            .map(|idx| self.blocks[idx].mir)
    }

    pub fn find_operand_no_constdata(
        &self,
        operand: &ValueSSA,
    ) -> Result<MirOperand, OperandMapError> {
        match operand {
            ValueSSA::FuncArg(_, n) => self
                .find_operand_for_arg(*n)
                .map(RegOperand::into)
                .ok_or(OperandMapError::IsNotFound(operand.clone())),
            ValueSSA::Block(b) => self
                .find_operand_for_block(*b)
                .map(MirOperand::Label)
                .ok_or(OperandMapError::IsNotFound(operand.clone())),
            ValueSSA::Inst(i) => self
                .find_operand_for_inst(*i)
                .map(RegOperand::into)
                .ok_or(OperandMapError::IsNotFound(operand.clone())),
            ValueSSA::Global(g) => self
                .find_operand_for_global(*g)
                .map(MirOperand::Global)
                .ok_or(OperandMapError::IsNotFound(operand.clone())),
            ValueSSA::ConstExpr(_) | ValueSSA::None => {
                Err(OperandMapError::IsUnsupported(operand.clone()))
            }
            ValueSSA::ConstData(c) => Err(OperandMapError::IsConstData(*c)),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum PureSourceReg {
    F32(FPR32),
    F64(FPR64),
    G32(GPR32),
    G64(GPR64),
}

impl PureSourceReg {
    pub fn from_reg_full(id: u32, si: SubRegIndex, uf: RegUseFlags, is_fp: bool) -> Self {
        let bits_log2 = si.get_bits_log2();
        match (is_fp, bits_log2) {
            (true, 5) => PureSourceReg::F32(FPR32(id, uf)),
            (true, 6) => PureSourceReg::F64(FPR64(id, uf)),
            (false, 5) => PureSourceReg::G32(GPR32(id, uf)),
            (false, 6) => PureSourceReg::G64(GPR64(id, uf)),
            _ => panic!("Unsupported size for store: {bits_log2}"),
        }
    }
    pub fn from_reg(op: RegOperand) -> Self {
        let RegOperand(id, si, uf, is_fp) = op;
        Self::from_reg_full(id, si, uf, is_fp)
    }

    pub fn from_constdata(
        constdata: &ConstData,
        type_ctx: &TypeContext,
        alloc_reg: &mut VirtRegAlloc,
        out_insts: &mut VecDeque<MirInst>,
        fpconst_force_float: bool,
    ) -> Self {
        match constdata {
            ConstData::Zero(ty) => {
                let ty_size = ty
                    .get_instance_size(type_ctx)
                    .expect("Failed to get type size");
                match ty_size {
                    4 => PureSourceReg::G32(Self::make_ldr_for_imm32(0, alloc_reg, out_insts)),
                    8 => PureSourceReg::G64(Self::make_ldr_for_imm64(0, alloc_reg, out_insts)),
                    _ => panic!("Unsupported zero-sized type for store: {ty:?}"),
                }
            }
            ConstData::PtrNull(_) => {
                PureSourceReg::G64(Self::make_ldr_for_imm64(0, alloc_reg, out_insts))
            }
            ConstData::Int(64, value) => {
                let value = *value as u64;
                PureSourceReg::G64(Self::make_ldr_for_imm64(value, alloc_reg, out_insts))
            }
            ConstData::Int(32, value) => {
                let value = *value as u32;
                PureSourceReg::G32(Self::make_ldr_for_imm32(value, alloc_reg, out_insts))
            }
            ConstData::Int(bits, value) => {
                let value = if *bits == 1 {
                    *value as u64
                } else {
                    ConstData::iconst_value_get_real_signed(*bits, *value) as u64
                };
                PureSourceReg::G64(Self::make_ldr_for_imm64(value, alloc_reg, out_insts))
            }
            ConstData::Float(FloatTypeKind::Ieee32, f) => {
                Self::f32const_to_reg(alloc_reg, out_insts, fpconst_force_float, *f as f32)
            }
            ConstData::Float(FloatTypeKind::Ieee64, f) => {
                Self::f64const_to_reg(alloc_reg, out_insts, fpconst_force_float, *f)
            }
            _ => panic!("Unsupported constant data for store: {constdata:?}"),
        }
    }

    pub fn from_valuessa(
        operand_map: &OperandMap,
        type_ctx: &TypeContext,
        vreg_alloc: &mut VirtRegAlloc,
        out_insts: &mut VecDeque<MirInst>,
        value: &ValueSSA,
        fpconst_force_float: bool,
    ) -> Result<Self, OperandMapError> {
        match operand_map.find_operand_no_constdata(value) {
            Ok(value) => match value {
                MirOperand::GPReg(GPReg(id, si, uf)) => Ok(Self::from_reg_full(id, si, uf, false)),
                MirOperand::VFReg(VFReg(id, si, uf)) => Ok(Self::from_reg_full(id, si, uf, true)),
                _ => panic!("Invalid source operand for store: {value:?}"),
            },
            Err(OperandMapError::IsConstData(c)) => {
                if let ConstData::Undef(_) = c {
                    return Err(OperandMapError::OperandUndefined);
                }
                Ok(Self::from_constdata(
                    &c,
                    type_ctx,
                    vreg_alloc,
                    out_insts,
                    fpconst_force_float,
                ))
            }
            Err(e) => panic!("Failed to find source operand for store: {e:?}"),
        }
    }

    pub fn into_mir(self) -> MirOperand {
        match self {
            PureSourceReg::F32(fpr32) => fpr32.into_mir(),
            PureSourceReg::F64(fpr64) => fpr64.into_mir(),
            PureSourceReg::G32(gpr32) => gpr32.into_mir(),
            PureSourceReg::G64(gpr64) => gpr64.into_mir(),
        }
    }

    fn make_ldr_for_imm32(
        imm32: u32,
        alloc_reg: &mut VirtRegAlloc,
        out_insts: &mut VecDeque<MirInst>,
    ) -> GPR32 {
        let reg = alloc_reg.insert_gp(GPR32(0, RegUseFlags::DEF).into_real());
        make_copy_inst(
            RegOperand::from(reg),
            MirOperand::Imm32(Imm32(imm32, ImmKind::Full)),
            out_insts,
        );
        GPR32::from_real(reg)
    }
    fn make_ldr_for_imm64(
        imm64: u64,
        alloc_reg: &mut VirtRegAlloc,
        out_insts: &mut VecDeque<MirInst>,
    ) -> GPR64 {
        let reg = alloc_reg.insert_gp(GPR64(0, RegUseFlags::DEF).into_real());
        make_copy_inst(
            RegOperand::from(reg),
            MirOperand::Imm64(Imm64(imm64, ImmKind::Full)),
            out_insts,
        );
        GPR64::from_real(reg)
    }

    fn f32const_to_reg(
        alloc_reg: &mut VirtRegAlloc,
        out_insts: &mut VecDeque<MirInst>,
        fpconst_force_float: bool,
        f: f32,
    ) -> Self {
        if try_cast_f32_to_aarch8(f).is_some() {
            let immf32 = ImmFMov32::new(f);
            let f32 = alloc_reg.insert_float(FPR32(0, RegUseFlags::DEF).into_real());
            let f32 = FPR32::from_real(f32);
            out_insts.push_back(FMov32I::new(MirOP::FMov32I, f32, immf32).into_mir());
            Self::F32(f32)
        } else if fpconst_force_float {
            let fbits = f.to_bits();
            let g32 = Self::make_ldr_for_imm32(fbits, alloc_reg, out_insts);
            let f32 = alloc_reg.insert_float(FPR32(0, RegUseFlags::DEF).into_real());
            let f32 = FPR32::from_real(f32);
            out_insts.push_back(UnaFG32::new(MirOP::FMovFG32, f32, g32).into_mir());
            Self::F32(f32)
        } else {
            let fbits = f.to_bits();
            Self::G32(Self::make_ldr_for_imm32(fbits, alloc_reg, out_insts))
        }
    }

    fn f64const_to_reg(
        alloc_reg: &mut VirtRegAlloc,
        out_insts: &mut VecDeque<MirInst>,
        fpconst_force_float: bool,
        f: f64,
    ) -> PureSourceReg {
        if try_cast_f64_to_aarch8(f).is_some() {
            let immf64 = ImmFMov64::new(f);
            let f64 = alloc_reg.insert_float(FPR64(0, RegUseFlags::DEF).into_real());
            let f64 = FPR64::from_real(f64);
            out_insts.push_back(FMov64I::new(MirOP::FMov32I, f64, immf64).into_mir());
            Self::F64(f64)
        } else if fpconst_force_float {
            let fbits = f.to_bits();
            let g64 = Self::make_ldr_for_imm64(fbits, alloc_reg, out_insts);
            let f64 = alloc_reg.insert_float(FPR64(0, RegUseFlags::DEF).into_real());
            let f64 = FPR64::from_real(f64);
            out_insts.push_back(UnaFG64::new(MirOP::FMovFG64, f64, g64).into_mir());
            Self::F64(f64)
        } else {
            let fbits = f.to_bits();
            Self::G64(Self::make_ldr_for_imm64(fbits, alloc_reg, out_insts))
        }
    }
}

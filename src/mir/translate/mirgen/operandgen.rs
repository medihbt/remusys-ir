use std::rc::Rc;

use crate::{
    ir::{ValueSSA, block::BlockRef, constant::data::ConstData, global::GlobalRef, inst::InstRef},
    mir::{
        module::{MirGlobalRef, block::MirBlockRef, func::MirFunc},
        operand::{MirOperand, reg::RegOperand},
        translate::mirgen::{MirBlockInfo, datagen::DataUnit, globalgen::MirGlobalItems},
    },
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

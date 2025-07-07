use std::rc::Rc;

use crate::{
    ir::{ValueSSA, block::BlockRef, global::GlobalRef, inst::InstRef},
    mir::{
        module::{MirGlobalRef, block::MirBlockRef, func::MirFunc},
        operand::{
            MirOperand,
            reg::VReg,
            suboperand::{IMirSubOperand, RegOperand},
        },
        translate::mirgen::{MirBlockInfo, datagen::DataUnit, globalgen::MirGlobalItems},
    },
};

pub struct OperandMap<'a> {
    pub args: Vec<(u32, RegOperand)>,
    pub func: Rc<MirFunc>,
    pub globals: &'a MirGlobalItems,
    pub insts: Vec<(InstRef, VReg)>,
    pub blocks: Vec<MirBlockInfo>,
}

impl<'a> OperandMap<'a> {
    pub fn new(
        func: Rc<MirFunc>,
        globals: &'a MirGlobalItems,
        insts: Vec<(InstRef, VReg)>,
        blocks: Vec<MirBlockInfo>,
    ) -> Self {
        debug_assert!(insts.is_sorted_by_key(|(inst, _)| *inst));
        debug_assert!(blocks.is_sorted_by_key(|b| b.ir));

        let nargs = func.arg_ir_types.len();
        let mut args = Vec::with_capacity(nargs);
        let mut arg_id = 0u32;
        for &preg in &func.arg_regs {
            args.push((arg_id, RegOperand::P(preg)));
            arg_id += 1;
        }
        for spilled_arg in func.borrow_spilled_args().iter() {
            args.push((arg_id, RegOperand::V(spilled_arg.virtreg)));
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
            .map(|idx| RegOperand::V(self.insts[idx].1))
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

    pub fn find_operand(&self, operand: &ValueSSA) -> Option<MirOperand> {
        match operand {
            ValueSSA::ConstData(data) => match DataUnit::from_const_primitive_data(*data) {
                DataUnit::Byte(x) => Some(MirOperand::Imm(x as i64)),
                DataUnit::Half(x) => Some(MirOperand::Imm(x as i64)),
                DataUnit::Word(x) => Some(MirOperand::Imm(x as i32 as i64)),
                DataUnit::DWord(x) => Some(MirOperand::Imm(x as i64)),
                _ => unreachable!("Unsupported data unit for MIR generation"),
            },
            ValueSSA::FuncArg(_, n) => self.find_operand_for_arg(*n).map(RegOperand::into_mirop),
            ValueSSA::Block(b) => self.find_operand_for_block(*b).map(MirOperand::Label),
            ValueSSA::Inst(i) => self.find_operand_for_inst(*i).map(RegOperand::into_mirop),
            ValueSSA::Global(g) => self.find_operand_for_global(*g).map(MirOperand::Global),
            ValueSSA::ConstExpr(_) | ValueSSA::None => {
                panic!("Unsupported value type in OperandMap: {operand:?}")
            }
        }
    }
}

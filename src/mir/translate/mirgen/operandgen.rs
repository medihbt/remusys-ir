use std::rc::Rc;

use crate::{
    ir::{block::BlockRef, global::GlobalRef, inst::InstRef},
    mir::{
        module::{block::MirBlockRef, func::MirFunc, MirGlobalRef},
        operand::{reg::VReg, suboperand::RegOperand},
        translate::mirgen::{globalgen::MirGlobalItems, MirBlockInfo},
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
        Self { args, func, globals, insts, blocks }
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
}

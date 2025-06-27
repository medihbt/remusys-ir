//! ## PHI Node Elimination Pass
//! 
//! This pass removes critical edges from the IR module by breaking them, operated after the
//! "critical edge elimination" pass.
//! 
//! Well... Remusys-IR is a SSA-only architecture, so we cannot just remove PHI nodes and
//! insert the `copy` instructions directly -- this IR even cannot support `copy` instructions.
//! Instead, we'll generate a map containing all related PHI nodes for each basic block, each
//! entry stands fot a `copy` instruction.
//! 
//! In MIR generation, each PHI set entry is equalvent to a `copy` instruction.

use std::collections::BTreeMap;

use crate::ir::{block::BlockRef, inst::phi::PhiOpRef, module::Module, ValueSSA};

pub(super) struct CopyInstNode {
    pub bb:   BlockRef,
    pub phi:  PhiOpRef,
    pub from: ValueSSA,
}

/// A read-only map that contains all `copy` instructions that should be inserted into the
/// basic blocks of the IR module.
pub(super) struct CopyMap {
    /// Items sorted by basic block.
    /// Each item contains a `copy` instruction that should be inserted into the block.
    pub insts: Vec<CopyInstNode>,
}

impl CopyMap {
    pub(super) fn find_copies(&self, bb: BlockRef) -> Option<&[CopyInstNode]> {
        let lower = self.lower_bound(bb);
        let upper = self.upper_bound(bb);
        let ret = &self.insts[lower..upper];
        if ret.is_empty() {
            None
        } else {
            Some(ret)
        }
    }
    pub(super) fn has_copy(&self, bb: BlockRef) -> bool {
        let lower = self.lower_bound(bb);
        let upper = self.upper_bound(bb);
        lower < upper
    }

    fn lower_bound(&self, bb: BlockRef) -> usize {
        self.insts.partition_point(|item| item.bb < bb)
    }
    fn upper_bound(&self, bb: BlockRef) -> usize {
        self.insts.partition_point(|item| item.bb <= bb)
    }

    pub(super) fn build_from_module(ir_module: &Module) -> Self {
    }
}
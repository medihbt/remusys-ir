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

use crate::{
    ir::{
        BlockRef, GlobalRef, InstData, Module, ValueSSA,
        inst::{ISubInstRef, PhiRef},
    },
    opt::analysis::cfg::snapshot::CfgSnapshot,
};
use std::ops::ControlFlow;

#[derive(Debug, Clone)]
pub struct CopyInstNode {
    /// The value that should be copied.
    pub from: ValueSSA,
    /// The basic block where the `copy` instruction should be inserted at.
    pub bb_from: BlockRef,
    /// The `phi` instruction that should be replaced with a `copy` instruction.
    pub phi: PhiRef,
    /// The basic block where the `phi` node is located.
    pub bb_to: BlockRef,
}

/// A read-only map that contains all `copy` instructions that should be inserted into the
/// basic blocks of the IR module.
pub struct CopyMap {
    /// Items sorted by basic block.
    /// Each item contains a `copy` instruction that should be inserted into the block.
    pub insts: Vec<CopyInstNode>,
}

impl CopyMap {
    pub fn find_copies(&self, bb: BlockRef) -> &[CopyInstNode] {
        let lower = self.lower_bound(bb);
        let upper = self.upper_bound(bb);
        &self.insts[lower..upper]
    }
    pub fn find_option_copies(&self, bb: BlockRef) -> Option<&[CopyInstNode]> {
        let ret = self.find_copies(bb);
        if ret.is_empty() { None } else { Some(ret) }
    }
    pub fn find_copies_unwrap(&self, bb: BlockRef) -> &[CopyInstNode] {
        let ret = self.find_copies(bb);
        assert!(
            !ret.is_empty(),
            "No copy instructions found for block {bb:?}"
        );
        ret
    }
    pub fn has_copy(&self, bb: BlockRef) -> bool {
        let lower = self.lower_bound(bb);
        let upper = self.upper_bound(bb);
        lower < upper
    }

    fn lower_bound(&self, bb: BlockRef) -> usize {
        self.insts.partition_point(|item| item.bb_from < bb)
    }
    fn upper_bound(&self, bb: BlockRef) -> usize {
        self.insts.partition_point(|item| item.bb_from <= bb)
    }

    pub fn from_module(ir_module: &Module) -> Self {
        let mut ret = Self { insts: Vec::new() };
        ir_module.forall_funcs(false, |fref, _| {
            ret.add_from_func(fref.0, ir_module);
            ControlFlow::Continue(())
        });
        ret.insts.sort_by_key(|node| node.bb_from);
        ret
    }
    pub fn new_and_cfg(ir_module: &Module) -> (Self, Vec<CfgSnapshot>) {
        let mut ret = Self { insts: Vec::new() };
        let mut cfg_snapshots = Vec::new();
        ir_module.forall_funcs(false, |fref, _| {
            let cfg_snapshot = ret.add_from_func(fref.0, ir_module);
            cfg_snapshots.push(cfg_snapshot);
            ControlFlow::Continue(())
        });
        ret.insts.sort_by_key(|node| node.bb_from);
        cfg_snapshots.sort_by_key(|snap| snap.func);
        (ret, cfg_snapshots)
    }

    fn add_from_func(&mut self, func_ref: GlobalRef, ir_module: &Module) -> CfgSnapshot {
        // Create a new snapshot of the CFG for the function.
        // Snapshot time: After the critical edge elimination pass; all critical edges are broken.
        let allocs = ir_module.borrow_allocs();
        let cfg_snapshot = CfgSnapshot::new(&allocs, func_ref);

        for node in cfg_snapshot.nodes.iter() {
            let bb = node.block;
            for (iref, inst) in bb.view_insts(&allocs) {
                let InstData::Phi(phi) = inst else { break };
                for (from, bb_from) in phi {
                    self.insts.push(CopyInstNode {
                        bb_from,
                        bb_to: node.block,
                        phi: PhiRef::from_raw_nocheck(iref),
                        from,
                    });
                }
            }
        }
        cfg_snapshot
    }
}

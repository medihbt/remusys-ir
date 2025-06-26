//! IR pass: key_edge_destroy

use crate::{base::NullableValue, ir::{block::{jump_target::JumpTargetRef, BlockRef}, module::{Module, ModuleError}}};

pub mod cfg_ops;

/// Removes key edges from the IR module.
///
/// ### How to scan
///
/// Suppose there are two basic blocks Ba and Bb with a directed edge `Jt = Ba -> Bb`.
/// If Ba has multiple successors and Bb has multiple predecessors, then Jt is a "critical edge".
///
/// ### How to remove
///
/// To remove a critical edge, we need to split the edge into two edges:
///
/// 1. Create a new basic block Bc.
/// 2. Insert Bc between Ba and Bb.
/// 3. Redirect the edge from Ba to Bc.
/// 4. Redirect the edge from Bc to Bb.
pub fn break_key_edges(ir_module: &Module) {
    todo!(
        "Implement key edge destruction for IR module: {}",
        ir_module.name
    );
}

//! IR pass: key_edge_destroy

use crate::ir::module::Module;

pub mod critical_edge;

/// Removes key edges from the IR module. Used in PHI-Node elimination.
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
/// 
/// ### Notes
///
/// This pass has less memory usage if we perform a "mark and compact"
/// garbage collection before this pass.
pub fn break_key_edges(ir_module: &Module) {
    use critical_edge::{break_critical_edge, collect_critical_edges};
    let critical_edges = collect_critical_edges(ir_module);
    if critical_edges.is_empty() {
        return;
    }
    for edge in critical_edges {
        break_critical_edge(ir_module, &edge);
    }
}

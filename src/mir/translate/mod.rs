//! Translate IR to MIR.
//!
//! This module includes some IR passes and MIR passes.
//!
//! ### IR Passes
//!
//! * `key_edge_destroy`: Removes key edges from the IR.
//! * `phi_elimination_generate`: Generates operations for phi elimination.
//!
//! ### MIR Passes
//!
//! * `translate_ir_to_mir`: Translates an IR module to a MIR module. PHI elimination is done in this pass.
//! * `lower_pseudo_ops`: Lowers pseudo operations in the MIR module.
//! * `reg_alloc`: Allocates registers for the MIR module.

use crate::{ir::module::Module, mir::module::MirModule};

pub mod ir_pass;
pub mod mirgen;

pub fn translate_ir_to_mir(ir_module: &Module) -> MirModule {
    use ir_pass::phi_node_ellimination::CopyMap;

    // Pass: Critical Edge Elimination
    ir_pass::critical_edge::break_critical_edges(ir_module);

    // Pass: PHI Node Elimination
    let (copy_map, cfgs) = CopyMap::new_and_cfg(ir_module);

    // Pass: Generate MIR from IR
    let mir_module = mirgen::codegen_ir_to_mir(ir_module, &copy_map, cfgs.as_slice());

    // Passes...
    todo!(
        "Implement MIR translation for IR module: {}",
        ir_module.name
    );
    mir_module
}

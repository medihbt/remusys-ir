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

pub fn translate_ir_to_mir(ir_module: &Module) -> MirModule {
    todo!(
        "Implement MIR translation for IR module: {}",
        ir_module.name
    )
}

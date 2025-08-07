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

use crate::{
    ir::Module,
    mir::{module::MirModule, translate::ir_pass::phi_node_ellimination::CopyMap},
};
use std::rc::Rc;

pub mod ir_pass;
pub mod mir_pass;
pub mod mirgen;

pub fn translate_ir_to_mir(ir_module: &Rc<Module>) -> MirModule {
    ir_pass::critical_edge::break_critical_edges(ir_module);
    let (copy_map, cfgs) = CopyMap::new_and_cfg(ir_module);
    let mut mir_module = mirgen::codegen_ir_to_mir(Rc::clone(ir_module), copy_map, cfgs);
    let name = mir_module.name.clone();
    eprintln!("Generated MIR module: {name}");

    // Perform additional MIR passes
    let sp_adjustments = mir_pass::inst_lower::pre_lower_a_module(&mir_module);
    eprintln!("Lowered MIR module: {name}");

    mir_pass::simple_reg_alloc::roughly_allocate_register(&mut mir_module);
    eprintln!("Roughly allocated registers in MIR module: {name}");

    mir_pass::inst_lower::post_lower_a_module(&mut mir_module);
    eprintln!("Post-lowered MIR module: {name}");

    // Final pass: lower stack operations
    mir_pass::stack_lower::lower_stack_for_module(&mut mir_module, sp_adjustments);
    eprintln!("Lowered stack operations in MIR module: {name}");

    mir_pass::preasm::preasm_pass_for_module(&mut mir_module);
    eprintln!("Pre-assembly pass completed for MIR module: {name}");

    // return the generated MIR module
    mir_module
}

#[cfg(test)]
mod testing {
    use super::*;
    use crate::{mir::util::asm_writer::AsmWriter, testing::cases::test_case_cfg_deep_while_br};

    #[test]
    fn test_translate_ir_to_mir() {
        let builder = test_case_cfg_deep_while_br();
        let ir_module = Rc::new(builder.module);
        let mir_module = translate_ir_to_mir(&ir_module);
        let mut stdout = std::io::stdout();
        let mut asm_writer = AsmWriter::new(&mut stdout);
        asm_writer.write_module(&mir_module);
    }
}

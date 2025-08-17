//! Optimizers of Remusys-IR compiler framework.

use crate::{ir::Module, testing::cases::write_ir_to_file_quiet};

pub mod analysis;
pub mod pass;
pub mod transform;
pub mod util;

pub fn optimize_module(module: &Module) {
    transform::dce::dce_pass(module);
    write_ir_to_file_quiet(module, "optimize_module");
}

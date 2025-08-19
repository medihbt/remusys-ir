//! Optimizers of Remusys-IR compiler framework.

use std::path::Path;

use crate::{ir::Module, testing::cases::write_ir_to_file_quiet};

pub mod analysis;
pub mod pass;
pub mod transform;
pub mod util;

pub fn optimize_module(module: &Module) {
    transform::dce::dce_pass(module);
    if log::log_enabled!(log::Level::Debug) {
        let module_name = {
            let name = Path::new(module.name.as_str());
            let filename = name.file_stem().unwrap().to_str().unwrap();
            format!("debug.optimize_module.{filename}")
        };
        write_ir_to_file_quiet(module, &module_name);
    }
}

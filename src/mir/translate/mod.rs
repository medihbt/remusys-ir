use crate::{ir::module::Module, mir::module::MirModule};

pub fn translate_ir_to_mir(ir_module: &Module) -> MirModule {
    todo!(
        "Implement MIR translation for IR module: {}",
        ir_module.name
    )
}

use crate::ir::FuncID;
use std::sync::Arc;

pub mod basic_dce;
pub mod mem2reg;

pub trait IFuncTransformPass {
    fn get_name(&self) -> Arc<str>;
    fn run_on_func(&mut self, func: FuncID);
}

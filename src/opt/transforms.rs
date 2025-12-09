use crate::ir::{FuncID, InstOrdering};
use std::sync::Arc;

pub mod basic_dce;

pub trait IFuncTransformPass {
    fn get_name(&self) -> Arc<str>;
    fn run_on_func(&mut self, order: &dyn InstOrdering, func: FuncID);
}

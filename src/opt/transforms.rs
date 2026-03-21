use crate::{SymbolStr, ir::FuncID};

pub mod basic_dce;
pub mod mem2reg;

pub trait IFuncTransformPass {
    fn get_name(&self) -> SymbolStr;
    fn run_on_func(&mut self, func: FuncID);
}

use std::rc::Rc;

use crate::ir::module::Module;

pub struct IRBuilder {
    pub module: Rc<Module>,
    pub focus: IRBuilderFocus,
}

pub struct IRBuilderFocus {
    //
}
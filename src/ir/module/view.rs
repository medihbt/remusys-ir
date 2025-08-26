use super::{
    IModuleEditable, IModuleReadable, Module,
    allocs::{IRAllocs, IRAllocsEditable, IRAllocsReadable},
};
use crate::typing::TypeContext;
use std::rc::Rc;

#[derive(Clone, Copy)]
pub struct ModuleView<'a>(pub &'a Rc<TypeContext>, pub &'a IRAllocs);

impl<'a> std::fmt::Debug for ModuleView<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(tctx, allocs) = self;
        write!(f, "ModuleView {{ type_ctx: {tctx:p}, allocs: {allocs:p} }}")
    }
}
impl<'a> IRAllocsReadable for ModuleView<'a> {
    fn get_allocs_ref(&self) -> &IRAllocs {
        self.1
    }
}
impl<'a> IModuleReadable for ModuleView<'a> {
    fn get_type_ctx(&self) -> &Rc<TypeContext> {
        self.0
    }
}

impl<'a> ModuleView<'a> {
    pub fn new(type_ctx: &'a Rc<TypeContext>, allocs: &'a IRAllocs) -> Self {
        Self(type_ctx, allocs)
    }

    pub fn from_module(module: &'a Module) -> Self {
        Self(&module.type_ctx, &module.allocs)
    }
}

pub struct ModuleEdit<'a>(pub &'a Rc<TypeContext>, pub &'a mut IRAllocs);

impl<'a> std::fmt::Debug for ModuleEdit<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(tctx, allocs) = self;
        write!(f, "ModuleEdit {{ type_ctx: {tctx:p}, allocs: {allocs:p} }}")
    }
}

impl<'a> IRAllocsReadable for ModuleEdit<'a> {
    fn get_allocs_ref(&self) -> &IRAllocs {
        self.1
    }
}
impl<'a> IRAllocsEditable for ModuleEdit<'a> {
    fn get_allocs_mutref(&mut self) -> &mut IRAllocs {
        self.1
    }
}
impl<'a> IModuleReadable for ModuleEdit<'a> {
    fn get_type_ctx(&self) -> &Rc<TypeContext> {
        self.0
    }
}
impl<'a> IModuleEditable for ModuleEdit<'a> {}

impl<'a> ModuleEdit<'a> {
    pub fn new(type_ctx: &'a Rc<TypeContext>, allocs: &'a mut IRAllocs) -> Self {
        Self(type_ctx, allocs)
    }

    pub fn from_module(module: &'a mut Module) -> Self {
        Self(&module.type_ctx, &mut module.allocs)
    }
}

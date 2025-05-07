use std::cell::Cell;

use func::FuncData;

use crate::{impl_slabref, typing::id::ValTypeID};

use super::{PtrStorage, ValueSSA};

pub mod func;

pub enum GlobalData {
    Alias(Alias),
    Var(Var),
    Func(FuncData),
}

pub struct GlobalDataCommon {
    pub name: String,
    pub content_ty: ValTypeID,
}

pub struct Alias {
    pub common: GlobalDataCommon,
    pub target: Cell<GlobalRef>,
}

pub struct Var {
    pub common: GlobalDataCommon,
    pub init: Cell<ValueSSA>,
}

impl PtrStorage for GlobalData {
    fn get_stored_pointee_type(&self) -> ValTypeID {
        self.get_common().content_ty.clone()
    }
}
impl GlobalData {
    pub fn get_common(&self) -> &GlobalDataCommon {
        match self {
            GlobalData::Alias(alias) => &alias.common,
            GlobalData::Var(var) => &var.common,
            GlobalData::Func(func) => &func.common,
        }
    }

    pub fn new_variable(name: String, content_ty: ValTypeID, init: ValueSSA) -> Self {
        GlobalData::Var(Var {
            common: GlobalDataCommon { name, content_ty },
            init: Cell::new(init),
        })
    }
    pub fn new_alias(name: String, content_ty: ValTypeID, target: GlobalRef) -> Self {
        GlobalData::Alias(Alias {
            common: GlobalDataCommon { name, content_ty },
            target: Cell::new(target),
        })
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlobalRef(usize);
impl_slabref!(GlobalRef, GlobalData);

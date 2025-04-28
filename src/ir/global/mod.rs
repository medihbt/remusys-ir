pub mod func;

use crate::{base::slabref::SlabRef, typing::id::ValTypeID};

use super::ValueRef;

pub enum Global {
    Var  (GlobalVar),
    Alias(GlobalAlias),
    Func (func::Func),
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct GlobalRef(pub(crate) usize);

impl SlabRef for GlobalRef {
    type Item = Global;

    fn from_handle(handle: usize) -> Self { GlobalRef(handle) }
    fn get_handle(&self) -> usize { self.0 }
}

pub struct GlobalDataCommon {
    pub name:       String,
    pub pointee_ty: ValTypeID,
}
pub struct GlobalAlias {
    pub data:    GlobalDataCommon,
    pub aliased: GlobalRef,
}
pub struct GlobalVar {
    pub data: GlobalDataCommon,
    pub init: Option<ValueRef>,
}

impl Global {
    pub fn get_common(&self) -> &GlobalDataCommon {
        match self {
            Global::Var(data) => &data.data,
            Global::Alias(data) => &data.data,
            Global::Func(data) => &data.global,
        }
    }
    pub fn common_mut(&mut self) -> &mut GlobalDataCommon {
        match self {
            Global::Var(data) => &mut data.data,
            Global::Alias(data) => &mut data.data,
            Global::Func(data) => &mut data.global,
        }
    }

    pub fn get_pointee_ty(&self) -> ValTypeID {
        self.get_common().pointee_ty.clone()
    }
    pub fn get_name(&self) -> &str {
        self.get_common().name.as_str()
    }

    pub fn try_as_func(&self) -> Option<&func::Func> {
        match self {
            Global::Func(data) => Some(data),
            _ => None,
        }
    }
    pub fn try_as_func_mut(&mut self) -> Option<&mut func::Func> {
        match self {
            Global::Func(data) => Some(data),
            _ => None,
        }
    }
}
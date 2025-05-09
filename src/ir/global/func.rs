use std::cell::RefCell;

use crate::{
    base::slablist::SlabRefList,
    ir::{PtrStorage, PtrUser, block::BlockRef},
    typing::{id::ValTypeID, types::FuncTypeRef},
};

use super::{GlobalDataCommon, GlobalRef};

pub trait FuncStorage: PtrStorage {
    fn get_stored_func_type(&self) -> FuncTypeRef {
        match self.get_stored_pointee_type() {
            ValTypeID::Func(func_type) => func_type,
            _ => panic!("Expected a function type"),
        }
    }
}
pub trait FuncUser: PtrUser {
    fn get_operand_func_type(&self) -> FuncTypeRef {
        match self.get_operand_pointee_type() {
            ValTypeID::Func(func_type) => func_type,
            _ => panic!("Expected a function type"),
        }
    }
}

pub struct FuncData {
    pub(super) common: GlobalDataCommon,
    pub(super) body: RefCell<Option<FuncBody>>,
}

pub struct FuncBody {
    pub func: GlobalRef,
    pub body: SlabRefList<BlockRef>,
}

impl PtrStorage for FuncData {
    fn get_stored_pointee_type(&self) -> ValTypeID {
        self.common.content_ty.clone()
    }
}
impl FuncStorage for FuncData {}

impl FuncData {
    pub fn is_extern(&self) -> bool {
        self.body.borrow().is_none()
    }
}

use std::cell::{Cell, Ref};

use func::FuncData;
use slab::Slab;

use crate::{
    base::{NullableValue, slablist::SlabRefListNode, slabref::SlabRef},
    impl_slabref,
    typing::id::ValTypeID,
};

use super::{
    block::{BlockData, BlockRef}, module::Module, PtrStorage, ValueSSA
};

pub mod func;

pub enum GlobalData {
    Alias(Alias),
    Var(Var),
    Func(FuncData),
}

pub struct GlobalDataCommon {
    pub name: String,
    pub content_ty: ValTypeID,
    pub self_ref: Cell<GlobalRef>,
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
    pub fn get_name(&self) -> &str {
        self.get_common().name.as_str()
    }

    pub fn new_variable(name: String, content_ty: ValTypeID, init: ValueSSA) -> Self {
        GlobalData::Var(Var {
            common: GlobalDataCommon {
                name,
                content_ty,
                self_ref: Cell::new(GlobalRef::new_null()),
            },
            init: Cell::new(init),
        })
    }
    pub fn new_alias(name: String, content_ty: ValTypeID, target: GlobalRef) -> Self {
        GlobalData::Alias(Alias {
            common: GlobalDataCommon {
                name,
                content_ty,
                self_ref: Cell::new(GlobalRef::new_null()),
            },
            target: Cell::new(target),
        })
    }

    pub(super) fn _init_set_self_reference(&self, alloc_bb: &Slab<BlockData>, self_ref: GlobalRef) {
        self.get_common().self_ref.set(self_ref);

        let func = if let GlobalData::Func(func) = self {
            func
        } else {
            return;
        };

        if let Some(body) = func.body.borrow().as_ref() {
            let mut curr_node = body.body._head;
            while curr_node.is_nonnull() {
                let bb = curr_node.to_slabref_unwrap(alloc_bb);
                bb._inner
                    .get()
                    .insert_parent_func(self_ref)
                    .assign_to(&bb._inner);
                curr_node = BlockRef::from_handle(bb.load_node_head().next);
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GlobalRef(usize);
impl_slabref!(GlobalRef, GlobalData);

impl GlobalRef {
    pub fn get_name_with_alloc<'a>(&self, slab: &'a Slab<GlobalData>) -> &'a str {
        self.to_slabref_unwrap(slab).get_name()
    }
    pub fn get_name_with_module<'a>(&self, module: &'a Module) -> Ref<'a, str> {
        Ref::map(module.get_global(*self), |g| g.get_name())
    }

    pub fn accept_read_visitor(&self, alloc: &Slab<GlobalData>, visitor: impl IGlobalObjectVisitor) {
        let global_data = self.to_slabref_unwrap(alloc);
        match global_data {
            GlobalData::Var(v) => visitor.read_global_variable(*self, v),
            GlobalData::Alias(a) => visitor.read_global_alias(*self, a),
            GlobalData::Func(f) => visitor.read_func(*self, f),
        }
    }
}

pub trait IGlobalObjectVisitor {
    fn read_global_variable(&self, global_ref: GlobalRef, gvar: &Var);
    fn read_global_alias(&self, global_ref: GlobalRef, galias: &Alias);
    fn read_func(&self, global_ref: GlobalRef, gfunc: &func::FuncData);
}

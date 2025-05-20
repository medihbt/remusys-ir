use std::{
    cell::{Cell, Ref},
    num::NonZero,
};

use func::FuncData;
use slab::Slab;

use crate::{
    base::{NullableValue, slablist::SlabRefListNode, slabref::SlabRef},
    impl_slabref,
    typing::id::ValTypeID,
};

use super::{
    PtrStorage, ValueSSA,
    block::{BlockData, BlockRef},
    module::Module,
};

pub mod func;
pub mod intrin;

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
    pub inner: Cell<VarInner>,
}

#[derive(Debug, Clone, Copy)]
pub struct VarInner {
    pub readonly: bool,
    pub align_log2: u8,
    pub init: ValueSSA,
}

impl Var {
    pub fn is_extern(&self) -> bool {
        self.inner.get().init.is_null()
    }
    pub fn get_init(&self) -> Option<ValueSSA> {
        self.inner.get().init.to_option()
    }
    pub fn set_init(&self, init: ValueSSA) {
        let mut inner = self.inner.get();
        inner.init = init;
        self.inner.set(inner);
    }
    pub fn is_readonly(&self) -> bool {
        self.inner.get().readonly
    }
    pub fn set_readonly(&self, readonly: bool) {
        let mut inner = self.inner.get();
        inner.readonly = readonly;
        self.inner.set(inner);
    }
    pub fn get_stored_pointee_align(&self) -> usize {
        1 << self.inner.get().align_log2
    }
    pub fn set_stored_pointee_align(&self, align: u64) {
        if align.is_power_of_two() {
            let mut inner = self.inner.get();
            inner.align_log2 = align.trailing_zeros() as u8;
            self.inner.set(inner);
        } else {
            panic!("Align {} NOT power of 2", align)
        }
    }
}

impl PtrStorage for GlobalData {
    fn get_stored_pointee_type(&self) -> ValTypeID {
        self.get_common().content_ty.clone()
    }

    fn get_stored_pointee_align(&self) -> Option<NonZero<usize>> {
        match self {
            GlobalData::Alias(_) => None,
            GlobalData::Func(_) => None,
            GlobalData::Var(v) => NonZero::new(v.get_stored_pointee_align()),
        }
    }
}
impl GlobalData {
    pub fn get_common(&self) -> &GlobalDataCommon {
        match self {
            GlobalData::Alias(alias) => &alias.common,
            GlobalData::Var(var) => &var.common,
            GlobalData::Func(func) => &func._common,
        }
    }
    pub fn common_mut(&mut self) -> &mut GlobalDataCommon {
        match self {
            GlobalData::Alias(alias) => &mut alias.common,
            GlobalData::Var(var) => &mut var.common,
            GlobalData::Func(func) => &mut func._common,
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
            inner: Cell::new(VarInner {
                readonly: false,
                align_log2: 3,
                init,
            }),
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

        if let Some(body) = func._body.borrow().as_ref() {
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
}

pub trait IGlobalObjectVisitor {
    fn read_global_variable(&self, global_ref: GlobalRef, gvar: &Var);
    fn read_global_alias(&self, global_ref: GlobalRef, galias: &Alias);
    fn read_func(&self, global_ref: GlobalRef, gfunc: &func::FuncData);

    fn global_object_visitor_dispatch(
        &self,
        global_ref: GlobalRef,
        alloc_global: &Slab<GlobalData>,
    ) {
        let global_data = global_ref.to_slabref_unwrap(alloc_global);
        match global_data {
            GlobalData::Alias(galias) => self.read_global_alias(global_ref, galias),
            GlobalData::Var(gvar) => self.read_global_variable(global_ref, gvar),
            GlobalData::Func(gfunc) => self.read_func(global_ref, gfunc),
        }
    }
}

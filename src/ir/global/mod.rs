use std::{
    cell::{Cell, Ref},
    num::NonZero,
};

use func::FuncData;
use slab::Slab;

use crate::{
    base::{INullableValue, SlabListNode, SlabRef},
    impl_slabref,
    ir::global::func::FuncStorage,
    typing::id::ValTypeID,
};

use super::{
    PtrStorage, ValueSSA,
    block::{BlockData, BlockRef},
    module::Module,
};

pub mod func;
pub mod intrin;

#[derive(Debug)]
pub enum GlobalData {
    Alias(Alias),
    Var(Var),
    Func(FuncData),
}

#[derive(Debug)]
pub struct GlobalDataCommon {
    pub name: String,
    pub content_ty: ValTypeID,
    pub self_ref: Cell<GlobalRef>,
}

#[derive(Debug)]
pub struct Alias {
    pub common: GlobalDataCommon,
    pub target: Cell<GlobalRef>,
}

#[derive(Debug)]
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

    pub fn get_stored_pointee_type(&self) -> ValTypeID {
        self.common.content_ty.clone()
    }

    pub fn get_name(&self) -> &str {
        self.common.name.as_str()
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

    pub fn is_readonly(&self) -> bool {
        match self {
            GlobalData::Alias(_) => todo!("Alias is not readonly"),
            GlobalData::Var(var) => var.is_readonly(),
            GlobalData::Func(_) => true,
        }
    }

    pub fn new_variable(
        name: String,
        is_const: bool,
        content_ty: ValTypeID,
        init: ValueSSA,
    ) -> Self {
        GlobalData::Var(Var {
            common: GlobalDataCommon {
                name,
                content_ty,
                self_ref: Cell::new(GlobalRef::new_null()),
            },
            inner: Cell::new(VarInner { readonly: is_const, align_log2: 3, init }),
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
        let GlobalData::Func(func) = self else {
            return;
        };

        if let Some(body) = func._body.borrow().as_ref() {
            let mut curr_node = body.body._head;
            while curr_node.is_nonnull() {
                let bb = curr_node.to_data(alloc_bb);
                bb._inner
                    .get()
                    .insert_parent_func(self_ref)
                    .assign_to(&bb._inner);
                curr_node = BlockRef::from_handle(bb.load_node_head().next);
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GlobalRef(usize);
impl_slabref!(GlobalRef, GlobalData);

impl GlobalRef {
    fn from_alloc_raw(alloc_global: &mut Slab<GlobalData>, data: GlobalData) -> Self {
        let handle = alloc_global.vacant_key();
        let ret = GlobalRef(handle);
        data.get_common().self_ref.set(ret);
        alloc_global.insert(data);
        ret
    }
    pub fn var_from_alloc(alloc: &mut Slab<GlobalData>, var: Var) -> Self {
        Self::from_alloc_raw(alloc, GlobalData::Var(var))
    }
    pub fn alias_from_alloc(alloc: &mut Slab<GlobalData>, alias: Alias) -> Self {
        Self::from_alloc_raw(alloc, GlobalData::Alias(alias))
    }
    pub fn func_from_allocs(
        alloc_global: &mut Slab<GlobalData>,
        alloc_block: &Slab<BlockData>,
        mut data: FuncData,
    ) -> Self {
        let ret = GlobalRef(alloc_global.vacant_key());
        data._common.self_ref.set(ret);

        let Some(body) = data._body.get_mut() else {
            alloc_global.insert(GlobalData::Func(data));
            return ret;
        };

        body.func = ret;
        let mut curr_node = body.body._head;
        while curr_node.is_nonnull() {
            let bb = curr_node.to_data(alloc_block);
            bb.set_parent_func(ret);
            let Some(next) = bb.get_next() else { break };
            curr_node = BlockRef::from_handle(next);
        }

        alloc_global.insert(GlobalData::Func(data));
        ret
    }
    pub fn from_allocs(
        alloc_global: &mut Slab<GlobalData>,
        alloc_block: &Slab<BlockData>,
        data: GlobalData,
    ) -> Self {
        match data {
            GlobalData::Alias(alias) => Self::alias_from_alloc(alloc_global, alias),
            GlobalData::Var(var) => Self::var_from_alloc(alloc_global, var),
            GlobalData::Func(func) => Self::func_from_allocs(alloc_global, alloc_block, func),
        }
    }

    /// 把自己注册到模块中
    /// 这会把自己注册到全局量表中，并且如果 RDFG 启用了, 就注册到 RDFG 中，维护 RDFG 的一致性
    pub fn register_to_mut_module(self, module: &mut Module) {
        let allocs = module._alloc_value.get_mut();
        let maybe_functy = match self.to_data(&allocs.globals) {
            GlobalData::Func(f) => Some(f.get_stored_func_type()),
            _ => None,
        };

        // 若 RDFG 启用了, 就注册到 RDFG 中，维护 RDFG 的一致性
        if let Some(rdfg) = module._rdfg_alloc.get_mut() {
            let type_ctx = module.type_ctx.clone();
            rdfg.alloc_node(ValueSSA::Global(self), maybe_functy, &type_ctx)
                .expect("Failed to register global to RDFG");
        }

        // 把自己注册到全局量表中
        let name = self.get_name_with_alloc(&allocs.globals);
        module.global_defs.get_mut().insert(name.into(), self);
    }

    pub fn register_to_module(self, module: &Module) {
        let allocs = module.borrow_value_alloc();
        let maybe_func = match self.to_data(&allocs.globals) {
            GlobalData::Func(f) => Some(f.get_stored_func_type()),
            _ => None,
        };

        // 若 RDFG 启用了, 就注册到 RDFG 中，维护 RDFG 的一致性
        if let Some(mut rdfg) = module.borrow_rdfg_alloc_mut() {
            rdfg.alloc_node(ValueSSA::Global(self), maybe_func, &module.type_ctx)
                .expect("Failed to register global to RDFG");
        }

        // 把自己注册到全局量表中
        let name = self.get_name_with_alloc(&allocs.globals);
        module.global_defs.borrow_mut().insert(name.into(), self);
    }

    pub fn from_mut_module(module: &mut Module, data: GlobalData) -> Self {
        let ret = {
            let allocs = module._alloc_value.get_mut();
            Self::from_allocs(&mut allocs.globals, &allocs.blocks, data)
        };
        ret.register_to_mut_module(module);
        ret
    }

    pub fn from_module(module: &Module, data: GlobalData) -> Self {
        let ret = {
            let mut allocs = module.borrow_value_alloc_mut();
            let allocs = &mut *allocs;
            Self::from_allocs(&mut allocs.globals, &allocs.blocks, data)
        };
        ret.register_to_module(module);
        ret
    }

    pub fn get_name_with_alloc<'a>(self, slab: &'a Slab<GlobalData>) -> &'a str {
        self.to_data(slab).get_name()
    }
    pub fn get_name_with_module<'a>(self, module: &'a Module) -> Ref<'a, str> {
        Ref::map(module.get_global(self), |g| g.get_name())
    }

    pub fn is_extern(self, alloc_global: &Slab<GlobalData>) -> bool {
        match self.to_data(alloc_global) {
            GlobalData::Alias(_) => false,
            GlobalData::Var(gvar) => gvar.is_extern(),
            GlobalData::Func(f) => f.is_extern(),
        }
    }
    pub fn is_extern_with_module(self, module: &Module) -> bool {
        let alloc_value = module.borrow_value_alloc();
        let alloc_global = &alloc_value.globals;
        self.is_extern(alloc_global)
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
        let global_data = global_ref.to_data(alloc_global);
        match global_data {
            GlobalData::Alias(galias) => self.read_global_alias(global_ref, galias),
            GlobalData::Var(gvar) => self.read_global_variable(global_ref, gvar),
            GlobalData::Func(gfunc) => self.read_func(global_ref, gfunc),
        }
    }
}

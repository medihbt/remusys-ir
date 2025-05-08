use std::{
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    rc::Rc,
};

use slab::Slab;

use crate::{base::slabref::SlabRef, typing::context::TypeContext};

use super::{
    ValueSSA,
    block::{
        BlockData, BlockRef,
        jump_target::{JumpTargetData, JumpTargetRef},
    },
    constant::expr::{ConstExprData, ConstExprRef},
    global::{GlobalData, GlobalRef},
    inst::{
        InstData, InstRef,
        usedef::{UseData, UseRef},
    },
};

pub struct Module {
    pub name: String,
    pub type_ctx: Rc<TypeContext>,
    pub global_defs: RefCell<HashMap<String, GlobalRef>>,
    pub(super) _alloc_value: RefCell<ModulAllocatorInner>,
    pub(super) _alloc_use: RefCell<Slab<UseData>>,
    pub(super) _alloc_jt: RefCell<Slab<JumpTargetData>>,
}

pub struct ModulAllocatorInner {
    pub(super) _alloc_global: Slab<GlobalData>,
    pub(super) _alloc_expr: Slab<ConstExprData>,
    pub(super) _alloc_inst: Slab<InstData>,
    pub(super) _alloc_block: Slab<BlockData>,
}

impl Module {
    pub fn new(name: String, type_ctx: Rc<TypeContext>) -> Self {
        let inner = ModulAllocatorInner {
            _alloc_global: Slab::with_capacity(32),
            _alloc_expr: Slab::with_capacity(4096),
            _alloc_inst: Slab::with_capacity(1024),
            _alloc_block: Slab::with_capacity(512),
        };
        Self {
            name,
            type_ctx,
            global_defs: RefCell::new(HashMap::new()),
            _alloc_value: RefCell::new(inner),
            _alloc_use: RefCell::new(Slab::with_capacity(4096)),
            _alloc_jt: RefCell::new(Slab::with_capacity(1024)),
        }
    }

    pub fn borrow_value_alloc<'a>(&'a self) -> Ref<'a, ModulAllocatorInner> {
        self._alloc_value.borrow()
    }
    pub fn borrow_value_alloc_mut<'a>(&'a self) -> RefMut<'a, ModulAllocatorInner> {
        self._alloc_value.borrow_mut()
    }

    pub fn borrow_use_alloc<'a>(&'a self) -> Ref<'a, Slab<UseData>> {
        self._alloc_use.borrow()
    }
    pub fn borrow_use_alloc_mut<'a>(&'a self) -> RefMut<'a, Slab<UseData>> {
        self._alloc_use.borrow_mut()
    }

    pub fn borrow_jt_alloc<'a>(&'a self) -> Ref<'a, Slab<JumpTargetData>> {
        self._alloc_jt.borrow()
    }
    pub fn borrow_jt_alloc_mut<'a>(&'a self) -> RefMut<'a, Slab<JumpTargetData>> {
        self._alloc_jt.borrow_mut()
    }
}

/// Adding and removing allocated items.
/// well... removing allocated items is prohibited; you can only use GC to remove them.
impl Module {
    pub fn get_global(&self, global: GlobalRef) -> Ref<GlobalData> {
        let inner = self.borrow_value_alloc();
        Ref::map(inner, |inner| {
            inner._alloc_global.get(global.get_handle()).unwrap()
        })
    }
    pub fn mut_global(&self, global: GlobalRef) -> RefMut<GlobalData> {
        let inner = self.borrow_value_alloc_mut();
        RefMut::map(inner, |inner| {
            inner._alloc_global.get_mut(global.get_handle()).unwrap()
        })
    }
    pub fn insert_global(&self, data: GlobalData) -> GlobalRef {
        let mut inner = self.borrow_value_alloc_mut();
        let name = data.get_common().name.clone();
        let id = inner._alloc_global.insert(data);
        let ret = GlobalRef::from_handle(id);
        self.global_defs.borrow_mut().insert(name, ret);
        ret
    }

    pub fn get_expr(&self, expr: ConstExprRef) -> Ref<ConstExprData> {
        let inner = self.borrow_value_alloc();
        Ref::map(inner, |inner| {
            inner._alloc_expr.get(expr.get_handle()).unwrap()
        })
    }
    pub fn mut_expr(&self, expr: ConstExprRef) -> RefMut<ConstExprData> {
        let inner = self.borrow_value_alloc_mut();
        RefMut::map(inner, |inner| {
            inner._alloc_expr.get_mut(expr.get_handle()).unwrap()
        })
    }
    pub fn insert_expr(&self, data: ConstExprData) -> ConstExprRef {
        let mut inner = self.borrow_value_alloc_mut();
        let id = inner._alloc_expr.insert(data);
        ConstExprRef::from_handle(id)
    }

    pub fn get_inst(&self, inst: InstRef) -> Ref<InstData> {
        let inner = self.borrow_value_alloc();
        Ref::map(inner, |inner| {
            inner._alloc_inst.get(inst.get_handle()).unwrap()
        })
    }
    pub fn mut_inst(&self, inst: InstRef) -> RefMut<InstData> {
        let inner = self.borrow_value_alloc_mut();
        RefMut::map(inner, |inner| {
            inner._alloc_inst.get_mut(inst.get_handle()).unwrap()
        })
    }
    pub fn insert_inst(&self, data: InstData) -> InstRef {
        let mut inner = self.borrow_value_alloc_mut();
        let id = inner._alloc_inst.insert(data);
        let ret = InstRef::from_handle(id);

        /* Modify the slab reference to point to this */
        ret.to_slabref_unwrap_mut(&mut inner._alloc_inst)
            .common_mut()
            .map(|c| c.self_ref = ret.clone());
        ret
    }

    pub fn get_block(&self, block: BlockRef) -> Ref<BlockData> {
        let inner = self.borrow_value_alloc();
        Ref::map(inner, |inner| {
            inner._alloc_block.get(block.get_handle()).unwrap()
        })
    }
    pub fn mut_block(&self, block: BlockRef) -> RefMut<BlockData> {
        let inner = self.borrow_value_alloc_mut();
        RefMut::map(inner, |inner| {
            inner._alloc_block.get_mut(block.get_handle()).unwrap()
        })
    }
    pub fn insert_block(&self, data: BlockData) -> BlockRef {
        let mut inner = self.borrow_value_alloc_mut();
        let id = inner._alloc_block.insert(data);
        BlockRef::from_handle(id)
    }

    pub fn get_use(&self, use_ref: UseRef) -> Ref<UseData> {
        let inner = self.borrow_use_alloc();
        Ref::map(inner, |inner| use_ref.to_slabref_unwrap(inner))
    }
    pub fn mut_use(&self, use_ref: UseRef) -> RefMut<UseData> {
        let inner = self.borrow_use_alloc_mut();
        RefMut::map(inner, |inner| use_ref.to_slabref_unwrap_mut(inner))
    }
    pub fn insert_use(&self, data: UseData) -> UseRef {
        let mut inner = self.borrow_use_alloc_mut();
        let id = inner.insert(data);
        UseRef::from_handle(id)
    }

    pub fn get_jt(&self, use_ref: JumpTargetRef) -> Ref<JumpTargetData> {
        let inner = self.borrow_jt_alloc();
        Ref::map(inner, |inner| use_ref.to_slabref_unwrap(inner))
    }
    pub fn mut_jt(&self, use_ref: JumpTargetRef) -> RefMut<JumpTargetData> {
        let inner = self.borrow_jt_alloc_mut();
        RefMut::map(inner, |inner| use_ref.to_slabref_unwrap_mut(inner))
    }
    pub fn insert_jt(&self, data: JumpTargetData) -> JumpTargetRef {
        let mut inner = self.borrow_jt_alloc_mut();
        let id = inner.insert(data);
        JumpTargetRef::from_handle(id)
    }

    /// Implement a 'mark-sweep' algorithm to reduce usage of those allocators.
    /// If the module owns its type context uniquely, it also collects garbages in
    /// its type context.
    ///
    /// This function cannot change the reference addresses of `Value`.
    pub fn gc_mark_sweep(&self, _external_live_set: impl Iterator<Item = ValueSSA>) {
        todo!()
    }
}

#[cfg(test)]
mod testing {

    use crate::{
        ir::{ValueSSA, constant::data::ConstData, global::GlobalData},
        typing::{context::PlatformPolicy, id::ValTypeID},
    };

    #[test]
    fn test_module() {
        use super::Module;
        use crate::typing::context::TypeContext;

        let type_ctx = TypeContext::new_rc(PlatformPolicy::new_host());
        let module = Module::new("test_module".to_string(), type_ctx.clone());
        assert_eq!(module.name, "test_module");

        // translate SysY source `int a = 0;` to IR: Create an integer global variable `a` and initialize it to 0.
        let global_data = GlobalData::new_variable(
            "a".to_string(),
            ValTypeID::Int(32),
            ValueSSA::ConstData(ConstData::Int(32, 0)),
        );

        module.insert_global(global_data);

        assert!(module.global_defs.borrow().contains_key("a"));
    }
}

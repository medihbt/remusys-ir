use std::{collections::HashMap, rc::Rc};

use block::{BlockData, BlockRef};
use global::{Global, GlobalRef};
use inst::InstRef;
use slab::Slab;

use crate::{base::{slabref::SlabRef, NullableValue}, typing::{context::TypeContext, id::{ValTypeID, ValTypeUnion}, subtypes::FuncType}};

pub mod opcode;
pub mod constant;
pub mod global;
pub mod block;
pub mod inst;
pub mod util;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum ValueRef {
    None,
    Block (BlockRef),
    Global(GlobalRef),
    Inst  (InstRef),
}

pub trait PtrStorage {
    fn get_pointee_ty(&self) -> ValTypeID;

    fn read_pointee_ty<R>(&self, module: &Module, f: impl FnOnce(&ValTypeUnion) -> R) -> Option<R> {
        let ty = self.get_pointee_ty();
        let tctx = module.get_type_ctx().borrow();
        if let Some(ty) = tctx.find_type(&ty) {
            Some(f(ty))
        } else {
            None
        }
    }
}

pub trait FuncStorage: PtrStorage {
    fn read_func_ty<R>(&self, module: &Module, f: impl FnOnce(&FuncType) -> R) -> Option<R> {
        self.read_pointee_ty(module, |ty| {
            if let ValTypeUnion::Func(func_ty) = ty {
                Some(f(func_ty))
            } else {
                panic!("Invalid type: requires function type but got {:?}", ty);
            }
        }).unwrap_or(None)
    }

    fn get_rettype(&self, module: &Module) -> Option<ValTypeID> {
        self.read_func_ty(module, |func_ty| Some(func_ty.ret_ty.clone()))
            .unwrap_or(None)
    }
    fn get_argtype(&self, module: &Module, index: usize) -> Option<ValTypeID> {
        self.read_func_ty(module, |func_ty| {
            if index < func_ty.args.len() {
                Some(func_ty.args[index].clone())
            } else {
                None
            }
        }).unwrap_or(None)
    }
}

impl NullableValue for ValueRef {
    fn new_null() -> Self { ValueRef::None }
    fn is_null(&self) -> bool { matches!(self, ValueRef::None) }
}
impl ValueRef {
    pub fn is_none(&self)   -> bool { matches!(self, ValueRef::None) }
    pub fn is_block(&self)  -> bool { matches!(self, ValueRef::Block(_)) }
    pub fn is_global(&self) -> bool { matches!(self, ValueRef::Global(_)) }
    pub fn is_inst(&self)   -> bool { matches!(self, ValueRef::Inst(_)) }

    pub fn get_value_ty(&self, module: &Module) -> ValTypeID {
        let type_ctx = module.get_rc_type_ctx();
        match self {
            ValueRef::None | ValueRef::Block(_) => type_ctx.get_void_type(),
            ValueRef::Global(_) => type_ctx.get_ptr_type(),
            ValueRef::Inst(inst_ref) => {
                inst_ref.read_slabref(
                    &module._alloc_inst,
                    |inst| {
                        inst.get_ty()
                    }).unwrap_or(type_ctx.get_void_type())
            }
        }
    }
}


// ============================[ Module definition ]==============================

pub struct Module {
    pub name: String,

    _type_ctx:     Rc<TypeContext>,
    _alloc_global: Slab<Global>,
    _alloc_block:  Slab<BlockData>,
    _alloc_inst:   Slab<inst::Inst>,
    _alloc_use:    Slab<inst::usedef::UseData>,
    _alloc_jt:     Slab<inst::jump_targets::JumpTargetData>,
    _global_map:   HashMap<String, GlobalRef>,
}

impl Module {
    pub fn new(name: String) -> Self {
        Self {
            name,
            _type_ctx:     TypeContext::new(),
            _alloc_global: Slab::new(),
            _alloc_block:  Slab::new(),
            _alloc_inst:   Slab::new(),
            _alloc_use:    Slab::new(),
            _alloc_jt:     Slab::new(),
            _global_map:   HashMap::new(),
        }
    }

    pub fn get_type_ctx   (&self) -> &TypeContext     { &self._type_ctx }
    pub fn get_rc_type_ctx(&self) -> &Rc<TypeContext> { &self._type_ctx }

    pub fn alloc_global(&mut self, global: Global) -> Option<GlobalRef> {
        if self._global_map.contains_key(global.get_name()) {
            None
        } else {
            let global_ref = GlobalRef::from_handle(self._alloc_global.insert(global));
            let global = global_ref.to_slabref(&self._alloc_global).unwrap();
            self._global_map.insert(global.get_name().to_string(), global_ref);
            Some(global_ref)
        }
    }
    pub fn find_global(&self, name: &str) -> Option<GlobalRef> {
        self._global_map.get(name).cloned()
    }
    pub fn edit_global<R>(&mut self, name: &str, f: impl FnOnce(&mut Global) -> R) -> Option<R> {
        if let Some(global_ref) = self._global_map.get(name) {
            let global = global_ref.to_slabref_mut(&mut self._alloc_global).unwrap();
            Some(f(global))
        } else {
            None
        }
    }
    pub fn read_global<R>(&self, name: &str, f: impl FnOnce(&Global) -> R) -> Option<R> {
        if let Some(global_ref) = self._global_map.get(name) {
            let global = global_ref.to_slabref(&self._alloc_global).unwrap();
            Some(f(global))
        } else {
            None
        }
    }
    pub fn remove_global(&mut self, name: &str) -> Option<GlobalRef>
    {
        if let Some(global_ref) = self._global_map.remove(name) {
            self._alloc_global.remove(global_ref.get_handle());
            Some(global_ref)
        } else {
            None
        }
    }

    pub fn alloc_block(&mut self, block: BlockData) -> BlockRef {
        BlockRef::from_handle(self._alloc_block.insert(block))
    }
    pub fn alloc_inst(&mut self, inst: inst::Inst) -> inst::InstRef {
        inst::InstRef::from_handle(self._alloc_inst.insert(inst))
    }
    pub fn alloc_use(&mut self, use_data: inst::usedef::UseData) -> inst::usedef::UseRef {
        inst::usedef::UseRef::from_handle(self._alloc_use.insert(use_data))
    }
    pub fn alloc_jt(&mut self, jt_data: inst::jump_targets::JumpTargetData) -> inst::jump_targets::JumpTargetRef {
        inst::jump_targets::JumpTargetRef::from_handle(self._alloc_jt.insert(jt_data))
    }

    pub fn gc(&mut self) {
        todo!("Implement garbage collection for Module");
    }
}
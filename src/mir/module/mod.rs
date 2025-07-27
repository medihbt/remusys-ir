use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};

use slab::Slab;

use crate::{
    base::slabref::SlabRef,
    mir::{
        inst::inst::MirInst,
        module::{
            block::MirBlock,
            func::MirFunc,
            global::{Linkage, MirGlobalCommon, MirGlobalData, MirGlobalVariable},
        },
    },
};

pub(super) mod block;
pub(super) mod func;
pub(super) mod global;
pub(super) mod stack;
pub mod vreg_alloc;

/// Represents an item in a MIR module, which can be a global variable, unnamed data, or a function.
#[derive(Debug, Clone)]
pub enum MirGlobal {
    Variable(Rc<MirGlobalVariable>),
    UnnamedData(MirGlobalData),
    Function(Rc<MirFunc>),
    Useless,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MirGlobalRef(usize);

impl SlabRef for MirGlobalRef {
    type RefObject = MirGlobal;
    fn from_handle(handle: usize) -> Self {
        MirGlobalRef(handle)
    }
    fn get_handle(&self) -> usize {
        self.0
    }
}

impl MirGlobalRef {
    pub fn from_alloc(alloc: &mut Slab<MirGlobal>, data: MirGlobal) -> Self {
        let index = alloc.insert(data);
        if index == usize::MAX {
            panic!("Failed to allocate ModuleItem in slab");
        }
        let ret = Self(index);
        ret.read_slabref(alloc, |data| {
            data.get_common().set_self_ref(ret);
        });
        ret
    }
    pub fn from_module(module: &MirModule, data: MirGlobal) -> Self {
        let mut alloc = module.borrow_alloc_item_mut();
        MirGlobalRef::from_alloc(&mut alloc, data)
    }

    pub fn data_from_module(self, module: &MirModule) -> Ref<MirGlobal> {
        let alloc = module.borrow_alloc_item();
        Ref::map(alloc, |a| {
            a.get(self.0 as usize).expect("Invalid ModuleItemRef")
        })
    }
    pub fn force_as_func(self, module: &MirModule) -> Rc<MirFunc> {
        let item = self.data_from_module(module);
        match &*item {
            MirGlobal::Function(func) => func.clone(),
            _ => panic!("Expected a function, but found a different item type"),
        }
    }
    pub fn force_as_variable(self, module: &MirModule) -> Rc<MirGlobalVariable> {
        let item = self.data_from_module(module);
        match &*item {
            MirGlobal::Variable(var) => var.clone(),
            _ => panic!("Expected a variable, but found a different item type"),
        }
    }

    pub fn get_common(self, module: &MirModule) -> Ref<MirGlobalCommon> {
        let item = self.data_from_module(module);
        Ref::map(item, |item| item.get_common())
    }
    pub fn get_name(self, module: &MirModule) -> Option<String> {
        self.data_from_module(module).get_name().map(str::to_string)
    }

    pub fn is_extern(self, module: &MirModule) -> bool {
        self.get_common(module).linkage == Linkage::Extern
    }
}

impl MirGlobal {
    pub fn get_common(&self) -> &MirGlobalCommon {
        match self {
            MirGlobal::Variable(var) => &var.common,
            MirGlobal::UnnamedData(data) => &data.common,
            MirGlobal::Function(func) => &func.common,
            _ => panic!("Attempted to access common data of a useless item"),
        }
    }
    pub fn get_name(&self) -> Option<&str> {
        let name = self.get_common().name.as_str();
        if name.is_empty() { None } else { Some(name) }
    }
    pub fn is_extern(&self) -> bool {
        self.get_common().linkage == Linkage::Extern
    }
    pub fn is_uselsss(&self) -> bool {
        matches!(self, MirGlobal::Useless)
    }
}

#[derive(Debug)]
pub struct MirModule {
    pub name: String,
    pub items: Vec<MirGlobalRef>,
    pub allocs: RefCell<MirAllocs>,
}

#[derive(Debug)]
pub struct MirAllocs {
    pub block: Slab<MirBlock>,
    pub inst: Slab<MirInst>,
    pub item: Slab<MirGlobal>,
}

impl MirModule {
    pub fn new(name: String) -> Self {
        MirModule {
            name,
            items: Vec::new(),
            allocs: RefCell::new(MirAllocs {
                block: Slab::new(),
                inst: Slab::new(),
                item: Slab::new(),
            }),
        }
    }

    pub fn add_item(&mut self, item: MirGlobal) -> MirGlobalRef {
        let alloc_item = &mut self.allocs.get_mut().item;
        let item_ref = MirGlobalRef::from_alloc(alloc_item, item);
        self.items.push(item_ref);
        item_ref
    }

    pub fn borrow_alloc_block(&self) -> Ref<Slab<MirBlock>> {
        Ref::map(self.allocs.borrow(), |allocs| &allocs.block)
    }
    pub fn borrow_alloc_block_mut(&self) -> RefMut<Slab<MirBlock>> {
        RefMut::map(self.allocs.borrow_mut(), |allocs| &mut allocs.block)
    }
    pub fn borrow_alloc_inst(&self) -> Ref<Slab<MirInst>> {
        Ref::map(self.allocs.borrow(), |allocs| &allocs.inst)
    }
    pub fn borrow_alloc_inst_mut(&self) -> RefMut<Slab<MirInst>> {
        RefMut::map(self.allocs.borrow_mut(), |allocs| &mut allocs.inst)
    }
    pub fn borrow_alloc_item(&self) -> Ref<Slab<MirGlobal>> {
        Ref::map(self.allocs.borrow(), |allocs| &allocs.item)
    }
    pub fn borrow_alloc_item_mut(&self) -> RefMut<Slab<MirGlobal>> {
        RefMut::map(self.allocs.borrow_mut(), |allocs| &mut allocs.item)
    }
}

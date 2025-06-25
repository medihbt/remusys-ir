use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};

use slab::Slab;

use crate::{
    base::slabref::SlabRef,
    mir::module::{
        block::{MirBlock, MirBlockRef},
        func::MirFunc,
        global::{Linkage, MirGlobalCommon, MirGlobalData, MirGlobalVariable},
    },
};

pub mod block;
pub mod func;
pub mod global;
pub mod stack;

/// Represents an item in a MIR module, which can be a global variable, unnamed data, or a function.
#[derive(Debug)]
pub enum ModuleItem {
    Variable(Rc<MirGlobalVariable>),
    UnnamedData(MirGlobalData),
    Function(Rc<MirFunc>),
    Useless,
}

impl ModuleItem {
    pub fn get_common(&self) -> &MirGlobalCommon {
        match self {
            ModuleItem::Variable(var) => &var.common,
            ModuleItem::UnnamedData(data) => &data.common,
            ModuleItem::Function(func) => &func.common,
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
}

#[derive(Debug)]
pub struct MirModule {
    pub name: String,
    pub items: Vec<ModuleItem>,
    pub allocs: RefCell<MirAllocs>,
}

#[derive(Debug, Clone)]
pub struct MirAllocs {
    pub block: Slab<MirBlock>,
}

impl MirModule {
    pub fn new(name: String) -> Self {
        MirModule {
            name,
            items: Vec::new(),
            allocs: RefCell::new(MirAllocs { block: Slab::new() }),
        }
    }

    pub fn add_item(&mut self, item: ModuleItem) {
        item.get_common().index.set(self.items.len() as u32);
        self.items.push(item);
    }

    pub fn refresh_indices(&self) {
        for (index, item) in self.items.iter().enumerate() {
            let index_cell = &item.get_common().index;
            index_cell.set(index as u32);
        }
    }

    pub fn borrow_alloc_block(&self) -> Ref<Slab<MirBlock>> {
        Ref::map(self.allocs.borrow(), |allocs| &allocs.block)
    }
    pub fn borrow_alloc_block_mut(&self) -> RefMut<Slab<MirBlock>> {
        RefMut::map(self.allocs.borrow_mut(), |allocs| &mut allocs.block)
    }

    pub fn insert_block(&self, block: MirBlock) -> MirBlockRef {
        let mut allocs = self.borrow_alloc_block_mut();
        let index = allocs.insert(block);
        MirBlockRef::from_handle(index)
    }
    pub fn borrow_block(&self, block_ref: MirBlockRef) -> Ref<MirBlock> {
        Ref::map(self.borrow_alloc_block(), |allocs| {
            allocs
                .get(block_ref.get_handle())
                .expect("Block reference is invalid")
        })
    }
    pub fn borrow_block_mut(&self, block_ref: MirBlockRef) -> RefMut<MirBlock> {
        RefMut::map(self.borrow_alloc_block_mut(), |allocs| {
            allocs
                .get_mut(block_ref.get_handle())
                .expect("Block reference is invalid")
        })
    }
}

//! Reverse CFG (RCFG) module

use std::
    cell::{Ref, RefCell, RefMut}
;

use slab::Slab;

use crate::{
    base::{slabref::SlabRef, NullableValue},
    ir::block::{
        jump_target::{JumpTargetData, JumpTargetRef}, BlockRef
    },
};


#[derive(Clone, Debug)]
pub struct RcfgPerBlock {
    pub block: BlockRef,
    pub preds: RefCell<Vec<JumpTargetRef>>,
}

impl RcfgPerBlock {
    pub fn new(block: BlockRef) -> Self {
        Self {
            block,
            preds: RefCell::new(Vec::new()),
        }
    }

    pub fn add_predecessor(&self, pred: JumpTargetRef) {
        let mut comes_from = self.preds.borrow_mut();
        if !comes_from.contains(&pred) {
            comes_from.push(pred);
        }
    }
    pub fn remove_predecessor(&self, pred: JumpTargetRef) {
        let mut comes_from = self.preds.borrow_mut();
        if let Some(pos) = comes_from.iter().position(|x| *x == pred) {
            comes_from.remove(pos);
        }
    }

    pub fn dump_blocks(&self, alloc_jt: &Slab<JumpTargetData>) -> Box<[BlockRef]> {
        let mut blocks = Vec::new();
        for jt in self.preds.borrow().iter() {
            let jt = jt.get_block(alloc_jt);
            if blocks.contains(&jt) {
                continue;
            }
            blocks.push(jt);
        }
        blocks.into_boxed_slice()
    }
}

pub struct RcfgAllocs {
    pub per_bb: Vec<RcfgPerBlock>,
}

impl RcfgAllocs {
    pub fn new_with_capacity(block: usize) -> Self {
        Self {
            per_bb: vec![RcfgPerBlock::new(BlockRef::new_null()); block],
        }
    }
    pub fn alloc_node(&mut self, block: BlockRef) {
        let per_bb = &mut self.per_bb;
        if per_bb.len() <= block.get_handle() {
            per_bb.resize(
                block.get_handle() + 1,
                RcfgPerBlock::new(BlockRef::new_null()),
            );
        }

        let per_block = &mut per_bb[block.get_handle()];
        if per_block.block.is_null() {
            per_block.block = block;
        } else {
            panic!("RcfgPerBlock already initialized");
        }
    }

    pub fn free_node(&mut self, block: BlockRef) {
        let per_bb = &mut self.per_bb;
        if per_bb.len() <= block.get_handle() {
            panic!("RcfgPerBlock not initialized");
        }
        per_bb[block.get_handle()].block = BlockRef::new_null();
    }

    pub fn get_node(&self, block: BlockRef) -> &RcfgPerBlock {
        let per_bb = &self.per_bb;
        if per_bb.len() <= block.get_handle() {
            panic!("RcfgPerBlock not initialized");
        }
        &per_bb[block.get_handle()]
    }
    pub fn get_node_mut(&mut self, block: BlockRef) -> &mut RcfgPerBlock {
        let per_bb = &mut self.per_bb;
        if per_bb.len() <= block.get_handle() {
            panic!("RcfgPerBlock not initialized");
        }
        &mut per_bb[block.get_handle()]
    }

    pub fn option_borrow_node(
        alloc: &RefCell<Option<Self>>,
        block: BlockRef,
    ) -> Option<Ref<RcfgPerBlock>> {
        let alloc = alloc.borrow();
        if let None = *alloc {
            return None;
        }
        Some(Ref::map(alloc, |alloc| {
            let per_bb = &alloc.as_ref().unwrap().per_bb;
            if per_bb.len() <= block.get_handle() {
                panic!("RcfgPerBlock not initialized");
            }
            &per_bb[block.get_handle()]
        }))
    }
    pub fn option_borrow_node_mut(
        alloc: &RefCell<Option<Self>>,
        block: BlockRef,
    ) -> Option<RefMut<RcfgPerBlock>> {
        let alloc = alloc.borrow_mut();
        if let None = *alloc {
            return None;
        }
        Some(RefMut::map(alloc, |alloc| {
            let per_bb = &mut alloc.as_mut().unwrap().per_bb;
            if per_bb.len() <= block.get_handle() {
                panic!("RcfgPerBlock not initialized");
            }
            &mut per_bb[block.get_handle()]
        }))
    }
}

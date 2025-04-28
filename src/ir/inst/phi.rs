use slab::Slab;

use crate::ir::{block::BlockRef, ValueRef};

use super::{usedef::{UseData, UseRef}, InstCommon, InstDataTrait, InstRef};

pub struct PhiNode {
    pub incoming: Vec<(UseRef, BlockRef)>,
}

impl PhiNode {
    pub fn new() -> Self {
        Self {
            incoming: Vec::new(),
        }
    }

    pub fn add_incoming(&mut self,
                        self_common: &InstCommon,
                        self_ref:     InstRef,
                        alloc:       &mut Slab<UseData>,
                        incoming: (BlockRef, ValueRef)) {
        let (block, value) = incoming;
        let use_data = UseData::new_with_operand(self_ref, value);
        let use_ref = self_common.add_use(use_data, alloc);
        self.incoming.push((use_ref, block));
    }

    pub fn find_incoming(&self, block: BlockRef) -> Option<UseRef> {
        self.incoming.iter()
            .find(|(_, b)| *b == block)
            .map(|(u, _)| *u)
    }
    pub fn find_incoming_mut(&mut self, block: BlockRef) -> Option<&mut UseRef> {
        self.incoming.iter_mut()
            .find(|(_, b)| *b == block)
            .map(|(u, _)| u)
    }
    pub fn remove_incoming(&mut self, block: BlockRef) -> Option<UseRef>
    {
        self.incoming.iter()
            .position(|(_, b)| *b == block)
            .map(|i| {
                let (u, b) = self.incoming.remove(i);
                assert_eq!(b, block);
                u
            })
    }
}

impl InstDataTrait for PhiNode {}
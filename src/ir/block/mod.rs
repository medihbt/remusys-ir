use slab::Slab;

use crate::base::slabref::SlabRef;

#[derive(Clone, Copy, Debug, PartialEq, Eq, Hash)]
pub struct BlockRef(pub(crate) usize);

pub struct BlockData {
    // TODO
}

impl SlabRef for BlockRef {
    type Item = BlockData;

    fn from_handle(handle: usize) -> Self {
        BlockRef(handle)
    }

    fn get_handle(&self) -> usize {
        self.0
    }
}

impl BlockRef {
    pub fn split(&self, _alloc: &mut Slab<BlockData>) -> BlockRef {
        // TODO
        BlockRef(0)
    }
}

impl BlockData {
    pub fn new() -> Self {
        Self {}
    }
}
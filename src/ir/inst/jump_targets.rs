use crate::{base::slabref::SlabRef, ir::block::BlockRef};

use super::InstRef;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct JumpTargetRef(pub(crate) usize);

pub struct JumpTargetData {
    pub terminator: InstRef,
    pub target: BlockRef,
    pub prev:   Option<JumpTargetRef>,
    pub next:   Option<JumpTargetRef>,
}

impl SlabRef for JumpTargetRef {
    type Item = JumpTargetData;

    fn from_handle(handle: usize) -> Self {
        Self(handle)
    }
    fn get_handle(&self) -> usize {
        self.0
    }
}

impl JumpTargetData {
}
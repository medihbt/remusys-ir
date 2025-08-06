use crate::{
    base::{IWeakListNode, SlabRef},
    ir::{
        BlockRef, ISubValueSSA, InstRef, JumpTarget, JumpTargetKind, Use, ValueSSA,
        block::jump_target::TerminatorRef,
        inst::{BrRef, ISubInstRef, JumpRef, SwitchRef},
    },
};

pub trait IRGraphEdge: IWeakListNode {
    type SourceHolderT;
    type SourceVertexT: ISubValueSSA + SlabRef;
    type TargetVertexT: ISubValueSSA;

    fn get_source_holder(&self) -> Self::SourceHolderT;
    fn get_target_vertex(&self) -> Self::TargetVertexT;
}

impl IRGraphEdge for Use {
    type SourceHolderT = InstRef;
    type SourceVertexT = InstRef;
    type TargetVertexT = ValueSSA;

    fn get_source_holder(&self) -> InstRef {
        self.inst.get()
    }

    fn get_target_vertex(&self) -> ValueSSA {
        self.get_operand()
    }
}

impl IRGraphEdge for JumpTarget {
    type SourceHolderT = TerminatorRef;
    type SourceVertexT = BlockRef;
    type TargetVertexT = BlockRef;

    fn get_source_holder(&self) -> TerminatorRef {
        let inst = self.get_terminator();
        match self.kind {
            JumpTargetKind::None => TerminatorRef::Unreachable(inst),
            JumpTargetKind::Jump => TerminatorRef::Jump(JumpRef::from_raw_nocheck(inst)),
            JumpTargetKind::BrTrue | JumpTargetKind::BrFalse => {
                TerminatorRef::Br(BrRef::from_raw_nocheck(inst))
            }
            JumpTargetKind::SwitchDefault | JumpTargetKind::SwitchCase(_) => {
                TerminatorRef::Switch(SwitchRef::from_raw_nocheck(inst))
            }
        }
    }

    fn get_target_vertex(&self) -> BlockRef {
        self.get_block()
    }
}

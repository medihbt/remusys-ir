use crate::{
    base::NullableValue,
    mir::{
        module::{block::MirBlockRef, MirGlobalRef},
        operand::{subop::SwitchTab, IMirSubOperand, MirOperand},
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MirSymbolOp {
    Label(MirBlockRef),
    Global(MirGlobalRef),
    SwitchTab(u32),
}

impl IMirSubOperand for MirSymbolOp {
    type RealRepresents = MirSymbolOp;
    fn new_empty() -> Self {
        MirSymbolOp::Label(MirBlockRef::new_null())
    }
    fn from_mir(mir: MirOperand) -> Self {
        match mir {
            MirOperand::Label(block_ref) => MirSymbolOp::Label(block_ref),
            MirOperand::Global(global_ref) => MirSymbolOp::Global(global_ref),
            MirOperand::SwitchTab(tab) => MirSymbolOp::SwitchTab(tab),
            _ => panic!(
                "Expected MirOperand::Label, MirOperand::Global, or MirOperand::SwitchTab, found {mir:?}"
            ),
        }
    }

    fn into_mir(self) -> MirOperand {
        match self {
            MirSymbolOp::Label(block_ref) => MirOperand::Label(block_ref),
            MirSymbolOp::Global(global_ref) => MirOperand::Global(global_ref),
            MirSymbolOp::SwitchTab(tab) => MirOperand::SwitchTab(tab),
        }
    }

    fn from_real(real: Self) -> Self {
        real
    }
    fn into_real(self) -> Self {
        self
    }
    fn insert_to_real(self, _: Self) -> Self {
        self
    }

    fn fmt_asm(&self, formatter: &mut crate::mir::fmt::FormatContext<'_>) -> std::fmt::Result {
        match self {
            MirSymbolOp::Label(x) => x.fmt_asm(formatter),
            MirSymbolOp::Global(x) => x.fmt_asm(formatter),
            MirSymbolOp::SwitchTab(tab) => SwitchTab(*tab).fmt_asm(formatter),
        }
    }
}

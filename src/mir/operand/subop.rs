use crate::{
    base::NullableValue,
    mir::{
        fmt::FormatContext,
        module::{MirGlobalRef, block::MirBlockRef},
        operand::MirOperand,
    },
};
use std::fmt::Write;

pub trait IMirSubOperand {
    type RealRepresents;

    fn new_empty() -> Self;

    fn from_mir(mir: MirOperand) -> Self;
    fn into_mir(self) -> MirOperand;

    fn from_real(real: Self::RealRepresents) -> Self;
    fn into_real(self) -> Self::RealRepresents;

    fn insert_to_real(self, real: Self::RealRepresents) -> Self::RealRepresents;

    fn fmt_asm(&self, _formatter: &mut FormatContext<'_>) -> std::fmt::Result {
        todo!(
            "Format context has not been implemented. Implement this after the context is ready."
        );
    }
}

impl IMirSubOperand for MirBlockRef {
    type RealRepresents = MirBlockRef;

    fn new_empty() -> Self {
        MirBlockRef::new_null()
    }

    fn from_mir(mir: MirOperand) -> Self {
        if let MirOperand::Label(block_ref) = mir {
            block_ref
        } else {
            panic!("Expected MirOperand::Label, found {mir:?}");
        }
    }
    fn into_mir(self) -> MirOperand {
        MirOperand::Label(self)
    }
    fn from_real(real: Self) -> Self {
        real
    }
    fn into_real(self) -> Self {
        self
    }
    fn insert_to_real(self, real: Self) -> Self {
        real
    }
}

impl IMirSubOperand for MirGlobalRef {
    type RealRepresents = MirGlobalRef;
    fn new_empty() -> Self {
        MirGlobalRef::new_null()
    }
    fn from_mir(mir: MirOperand) -> Self {
        if let MirOperand::Global(global_ref) = mir {
            global_ref
        } else {
            panic!("Expected MirOperand::Global, found {mir:?}");
        }
    }
    fn into_mir(self) -> MirOperand {
        MirOperand::Global(self)
    }
    fn from_real(real: Self) -> Self {
        real
    }
    fn into_real(self) -> Self {
        self
    }
    fn insert_to_real(self, real: Self) -> Self {
        real
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SwitchTab(pub u32);

impl IMirSubOperand for SwitchTab {
    type RealRepresents = SwitchTab;

    fn new_empty() -> Self {
        SwitchTab(0)
    }

    fn from_mir(mir: MirOperand) -> Self {
        if let MirOperand::SwitchTab(tab) = mir {
            SwitchTab(tab)
        } else {
            panic!("Expected MirOperand::SwitchTab, found {mir:?}");
        }
    }
    fn into_mir(self) -> MirOperand {
        MirOperand::SwitchTab(self.0)
    }
    fn from_real(real: Self) -> Self {
        real
    }
    fn into_real(self) -> Self {
        self
    }
    fn insert_to_real(self, real: Self) -> Self {
        real
    }
}

impl IMirSubOperand for MirOperand {
    type RealRepresents = MirOperand;

    fn new_empty() -> Self {
        MirOperand::None
    }

    fn from_mir(mir: MirOperand) -> Self {
        mir
    }
    fn into_mir(self) -> MirOperand {
        self
    }
    fn from_real(real: Self) -> Self {
        real
    }
    fn into_real(self) -> Self {
        self
    }
    fn insert_to_real(self, real: Self) -> Self {
        real
    }

    fn fmt_asm(&self, formatter: &mut FormatContext<'_>) -> std::fmt::Result {
        match self {
            MirOperand::None => write!(formatter, "None"),
            MirOperand::GPReg(gpreg) => gpreg.fmt_asm(formatter),
            MirOperand::VFReg(vfreg) => vfreg.fmt_asm(formatter),
            MirOperand::PState(pstate) => pstate.fmt_asm(formatter),
            MirOperand::Imm64(imm64) => imm64.fmt_asm(formatter),
            MirOperand::Imm32(imm32) => imm32.fmt_asm(formatter),
            MirOperand::Label(bb) => bb.fmt_asm(formatter),
            MirOperand::Global(global) => global.fmt_asm(formatter),
            MirOperand::SwitchTab(id) => SwitchTab(*id).fmt_asm(formatter),
        }
    }
}

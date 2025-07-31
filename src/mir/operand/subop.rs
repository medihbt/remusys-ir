use crate::{
    base::{INullableValue, SlabRef},
    mir::{
        fmt::FuncFormatContext,
        module::{MirGlobalRef, block::MirBlockRef},
        operand::MirOperand,
    },
};
use std::fmt::{Debug, Write};

pub trait IMirSubOperand: Sized {
    type RealRepresents: Debug + Clone;

    fn new_empty() -> Self;

    fn from_mir(mir: MirOperand) -> Self;
    fn into_mir(self) -> MirOperand;

    fn try_from_real(real: Self::RealRepresents) -> Option<Self>;
    fn into_real(self) -> Self::RealRepresents;
    fn from_real(real: Self::RealRepresents) -> Self {
        match Self::try_from_real(real.clone()) {
            Some(x) => x,
            None => panic!("Failed to convert from real representation: {real:?}"),
        }
    }

    fn insert_to_real(self, real: Self::RealRepresents) -> Self::RealRepresents;
    fn fmt_asm(&self, _formatter: &mut FuncFormatContext<'_>) -> std::fmt::Result;
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
    fn try_from_real(real: Self) -> Option<Self> {
        real.to_option()
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

    fn fmt_asm(&self, formatter: &mut FuncFormatContext<'_>) -> std::fmt::Result {
        let alloc_block = formatter.mir_module.borrow_alloc_block();
        let bb_name = self.to_data(&alloc_block).name.as_str();
        write!(formatter, "{bb_name}")
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
    fn try_from_real(real: Self) -> Option<Self> {
        real.to_option()
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

    fn fmt_asm(&self, formatter: &mut FuncFormatContext<'_>) -> std::fmt::Result {
        let alloc_global = formatter.mir_module.borrow_alloc_item();
        let name = self.to_data(&alloc_global).get_name().unwrap_or("");
        formatter.write_str(name)
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
    fn try_from_real(real: Self) -> Option<Self> {
        Some(real)
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

    fn fmt_asm(&self, formatter: &mut FuncFormatContext<'_>) -> std::fmt::Result {
        let func = formatter.get_current_func();
        let func_name = func.get_name();
        let Self(index) = *self;
        match func.get_vec_switch_tab(index as usize) {
            Some(_) => {
                write!(formatter, ".{func_name}.switch.{index}")
            }
            None => return Err(std::fmt::Error),
        }
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
    fn try_from_real(real: Self) -> Option<Self> {
        Some(real)
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

    fn fmt_asm(&self, formatter: &mut FuncFormatContext<'_>) -> std::fmt::Result {
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
            MirOperand::F32(f) => write!(formatter, "{f:e}f32"),
            MirOperand::F64(f) => write!(formatter, "{f:e}f64"),
        }
    }
}

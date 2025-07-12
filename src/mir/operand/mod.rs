use crate::mir::{
    module::{block::MirBlockRef, MirGlobalRef},
    operand::{
        imm::{Imm32, Imm64},
        reg::{GPReg, PState, VFReg},
    },
};

pub mod compound;
pub mod imm;
pub mod imm_traits;
pub mod reg;
pub mod subop;

pub use subop::IMirSubOperand;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MirOperand {
    None,
    GPReg(GPReg),
    VFReg(VFReg),
    PState(PState),
    Imm64(Imm64),
    Imm32(Imm32),
    Label(MirBlockRef),
    Global(MirGlobalRef),
    SwitchTab(u32),
}

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

#[derive(Debug, Clone, Copy)]
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

    /// pesudo operands used for internal purposes
    F32(f32),
    F64(f64),
}

impl PartialEq for MirOperand {
    fn eq(&self, other: &Self) -> bool {
        use MirOperand::*;
        match (self, other) {
            (None, None) => true,
            (GPReg(a), GPReg(b)) => a == b,
            (VFReg(a), VFReg(b)) => a == b,
            (PState(a), PState(b)) => a == b,
            (Imm64(a), Imm64(b)) => a == b,
            (Imm32(a), Imm32(b)) => a == b,
            (Label(a), Label(b)) => a == b,
            (Global(a), Global(b)) => a == b,
            (SwitchTab(a), SwitchTab(b)) => a == b,
            (F32(a), F32(b)) => a.to_bits() == b.to_bits(),
            (F64(a), F64(b)) => a.to_bits() == b.to_bits(),
            _ => false,
        }
    }
}

impl Eq for MirOperand { }

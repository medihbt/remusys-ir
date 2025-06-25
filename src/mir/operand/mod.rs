use crate::mir::{
    module::block::MirBlockRef,
    operand::{
        reg::{PhysReg, RegOperand, VirtReg},
        symbol::SymbolOperand,
    },
};

pub mod immediate;
pub mod reg;
pub mod symbol;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MirOperand {
    VirtReg(VirtReg),
    PhysReg(PhysReg),
    ImmConst(i64),
    Symbol(SymbolOperand),
    Label(MirBlockRef),
    /// Index of array switch table collections. Represents a switch table.
    VecSwitchTab(usize),
    /// Index of binary switch table collections. Represents a switch table.
    BinSwitchTab(usize),
    None,
}

impl From<RegOperand> for MirOperand {
    fn from(operand: RegOperand) -> Self {
        match operand {
            RegOperand::Virt(vr) => MirOperand::VirtReg(vr),
            RegOperand::Phys(pr) => MirOperand::PhysReg(pr),
        }
    }
}

impl From<SymbolOperand> for MirOperand {
    fn from(operand: SymbolOperand) -> Self {
        match operand {
            SymbolOperand::Label(label) => MirOperand::Label(label),
            SymbolOperand::Global(index) => MirOperand::Symbol(SymbolOperand::Global(index)),
        }
    }
}
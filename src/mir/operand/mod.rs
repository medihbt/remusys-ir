use physreg::PhysReg;
use virtreg::VirtReg;
use constant::ImmConst;

use super::block::MachineBlockRef;

pub mod constant;
pub mod virtreg;
pub mod physreg;

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum MachineOperand {
    VirtReg(VirtReg),
    PhysReg(PhysReg),
    ImmConst(ImmConst),
    ImmSymbol,
    Label(MachineBlockRef),
    SwitchEntry,
    ConstPoolIndex(u32),
}

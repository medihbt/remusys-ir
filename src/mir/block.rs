use std::collections::{BTreeSet, HashSet};

use crate::impl_slabref;

use super::operand::{MachineOperand, virtreg::VirtReg};

#[derive(Debug, Clone)]
pub struct MachineBlock {
    pub successors: BTreeSet<MachineBlockRef>,
    pub livein_regs: HashSet<VirtReg>,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MachineBlockRef(pub usize);
impl_slabref!(MachineBlockRef, MachineBlock);

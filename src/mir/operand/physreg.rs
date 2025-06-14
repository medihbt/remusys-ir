use crate::mir::operand::RegUseFlags;

use super::SubRegIndex;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PhysReg {
    X(u8, SubRegIndex, RegUseFlags), // General-purpose registers, e.g., X0, X1, ..., X31
    V(u8, SubRegIndex, RegUseFlags), // Vector registers: V[0:31], D[0:31], S[0:31]
    PC(SubRegIndex, RegUseFlags),    // Program Counter
    PState(RegUseFlags),             // Processor state register (e.g., PSTATE)
}

impl PhysReg {
    pub const fn sp() -> Self {
        PhysReg::X(31, SubRegIndex::new(6, 0), RegUseFlags::NONE)
    }
    pub const fn is_sp(self) -> bool {
        matches!(self, PhysReg::X(31, ..))
    }

    pub const fn x(reg_id: u8) -> Self {
        assert!(reg_id < 32);
        PhysReg::X(reg_id, SubRegIndex::new(6, 0), RegUseFlags::NONE)
    }
    pub const fn is_x(self) -> bool {
        if let PhysReg::X(_, si, _) = self {
            si.get_bits_log2() == 6
        } else {
            false
        }
    }
    pub const fn w(reg_id: u8) -> Self {
        assert!(reg_id < 32);
        PhysReg::X(reg_id, SubRegIndex::new(5, 0), RegUseFlags::NONE)
    }
    pub const fn is_w(self) -> bool {
        if let PhysReg::X(_, si, _) = self {
            si.get_bits_log2() == 5
        } else {
            false
        }
    }
    pub const fn fp_d(reg_id: u8) -> Self {
        assert!(reg_id < 32);
        PhysReg::V(reg_id, SubRegIndex::new(6, 0), RegUseFlags::NONE)
    }
    pub const fn is_fp_d(self) -> bool {
        if let PhysReg::V(_, si, _) = self {
            si.get_bits_log2() == 6
        } else {
            false
        }
    }
    pub const fn fp_s(reg_id: u8) -> Self {
        assert!(reg_id < 32);
        PhysReg::V(reg_id, SubRegIndex::new(5, 0), RegUseFlags::NONE)
    }
    pub const fn is_fp_s(self) -> bool {
        if let PhysReg::V(_, si, _) = self {
            si.get_bits_log2() == 5
        } else {
            false
        }
    }
    pub const fn return_addr() -> Self {
        PhysReg::X(30, SubRegIndex::new(6, 0), RegUseFlags::NONE)
    }
    pub const fn is_return_addr(self) -> bool {
        matches!(self, PhysReg::X(30, ..))
    }
    pub const fn pc() -> Self {
        PhysReg::PC(SubRegIndex::new(6, 0), RegUseFlags::NONE)
    }
    pub const fn is_pc(self) -> bool {
        matches!(self, PhysReg::PC(..))
    }

    pub fn use_flags_mut(&mut self) -> &mut RegUseFlags {
        match self {
            PhysReg::X(_, _, flags)
            | PhysReg::V(_, _, flags)
            | PhysReg::PC(_, flags)
            | PhysReg::PState(flags) => flags,
        }
    }
    pub fn get_use_flags(&self) -> RegUseFlags {
        match self {
            PhysReg::X(_, _, flags)
            | PhysReg::V(_, _, flags)
            | PhysReg::PC(_, flags)
            | PhysReg::PState(flags) => *flags,
        }
    }
    pub fn add_use_flag(&mut self, flag: RegUseFlags) {
        self.use_flags_mut().insert(flag);
    }

    pub fn get_bits(&self) -> u8 {
        match self {
            PhysReg::X(_, si, _) | PhysReg::V(_, si, _) | PhysReg::PC(si, _) => {
                1 << si.get_bits_log2()
            }
            PhysReg::PState(_) => 0, // PState is not a register with bits
        }
    }
}

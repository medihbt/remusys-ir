use std::fmt::Debug;

use bitflags::bitflags;
use constant::ImmConst;
use physreg::PhysReg;
use virtreg::VirtReg;

use super::block::MachineBlockRef;

pub mod constant;
pub mod physreg;
pub mod virtreg;

#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub enum MachineOperand {
    None,
    VirtReg(VirtReg),
    PhysReg(PhysReg),
    ImmConst(ImmConst),
    ImmSymbol,
    Label(MachineBlockRef),
    SwitchEntry,
    ConstPoolIndex(u32),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegOperand {
    VirtReg(VirtReg),
    PhysReg(PhysReg),
}

impl RegOperand {
    pub fn from_machine_operand_unwrap(op: MachineOperand) -> Self {
        match op {
            MachineOperand::VirtReg(vreg) => RegOperand::VirtReg(vreg),
            MachineOperand::PhysReg(preg) => RegOperand::PhysReg(preg),
            _ => panic!("Expected a register operand, got: {:?}", op),
        }
    }

    pub fn from_machine_operand(op: MachineOperand) -> Option<Self> {
        match op {
            MachineOperand::VirtReg(vreg) => Some(RegOperand::VirtReg(vreg)),
            MachineOperand::PhysReg(preg) => Some(RegOperand::PhysReg(preg)),
            _ => None,
        }
    }

    pub fn use_flags_mut(&mut self) -> &mut RegUseFlags {
        match self {
            RegOperand::VirtReg(vreg) => vreg.use_flags_mut(),
            RegOperand::PhysReg(preg) => preg.use_flags_mut(),
        }
    }
    pub fn set_use_flags(&mut self, flags: RegUseFlags, value: bool) {
        self.use_flags_mut().set(flags, value);
    }
    pub fn add_use_flags(&mut self, flags: RegUseFlags) {
        self.use_flags_mut().insert(flags);
    }
    pub fn del_use_flags(&mut self, flags: RegUseFlags) {
        self.use_flags_mut().remove(flags);
    }

    pub fn get_bits(&self) -> u8 {
        match self {
            RegOperand::VirtReg(vreg) => vreg.get_bits(),
            RegOperand::PhysReg(preg) => preg.get_bits(),
        }
    }
}

impl Into<MachineOperand> for RegOperand {
    fn into(self) -> MachineOperand {
        match self {
            RegOperand::VirtReg(vreg) => MachineOperand::VirtReg(vreg),
            RegOperand::PhysReg(preg) => MachineOperand::PhysReg(preg),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubRegIndex(pub u8);

impl SubRegIndex {
    pub const fn new(bits_log2: u8, index: u8) -> Self {
        assert!(
            bits_log2 >= 3 && bits_log2 <= 7,
            "bits_log2 must be in range [3, 7]"
        );
        assert!(index < 64, "index must be less than 64");
        let bits_log2_flag = bits_log2 - 3; // Convert to [0,4] range
        SubRegIndex((bits_log2_flag & 0b111) | ((index as u8) << 3))
    }

    /// `bits[0..3]` is sub-register binary bits flag:
    ///
    /// * 000 => 8
    /// * 001 => 16
    /// * 010 => 32   (Wn in Xn,  Sn in Vn)
    /// * 011 => 64   (Xn,        Dn in Vn)
    /// * 100 => 128  (Vn)
    pub const fn get_bits_log2(self) -> u8 {
        let bits_log2_flag = self.0 & 0b111;
        bits_log2_flag + 3
    }
    pub const fn insert_bits_log2(self, bits_log2: u8) -> Self {
        assert!(bits_log2 >= 3 && bits_log2 <= 7);
        let bits_log2_flag = bits_log2 - 3;
        SubRegIndex((self.0 & !0b111) | (bits_log2_flag & 0b111))
    }
    pub fn set_bits_log2(&mut self, bits_log2: u8) {
        *self = self.insert_bits_log2(bits_log2);
    }

    /// `bits[3..8]` is sub-register index. ranged from 0 to 31.
    /// This is used to represent sub-registers like Wn, Sn, Dn, Vn.
    pub const fn get_index(self) -> u8 {
        (self.0 >> 3) & 0b0001_1111
    }
    pub const fn insert_index(self, index: u8) -> Self {
        assert!(index < 32, "index must be less than 32");
        let index_bits = (index & 0b0001_1111) << 3;
        SubRegIndex((self.0 & !0b1111_1000) | index_bits)
    }
    pub fn set_index(&mut self, index: u8) {
        *self = self.insert_index(index);
    }
}

impl Debug for SubRegIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "SubRegIndex(bits_log2: {}, index: {}, value: {})",
            self.get_bits_log2(),
            self.get_index(),
            self.0
        )
    }
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct RegUseFlags: u16 {
        const NONE = 0b0000_0000_0000_0000;
        /// This register is defined in this instruction
        const DEF  = 0b0000_0000_0000_0001;
        /// This register is defined but not used in this instruction
        const DEAD = 0b0000_0000_0000_0010;
        /// The last use of this register in this instruction
        const KILL = 0b0000_0000_0000_0100;
        /// This register is defined implicitly in this instruction
        const IMPLICIT_DEF = 0b0000_0000_0000_1000;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum RegOP {
    LSL(u8),
    LSR(u8),
    ASR(u8),
    UXTB,
    UXTH,
    UXTW,
    SXTB,
    SXTH,
    SXTW,
    SXTX,
}

impl RegOP {
    pub const fn get_shift_bits(self) -> u8 {
        match self {
            Self::LSL(bits) | Self::LSR(bits) | Self::ASR(bits) => bits,
            _ => 0,
        }
    }

    pub const fn is_shift(self) -> bool {
        matches!(self, Self::LSL(_) | Self::LSR(_) | Self::ASR(_))
    }
    pub const fn is_ext(self) -> bool {
        matches!(
            self,
            Self::UXTB
                | Self::UXTH
                | Self::UXTW
                | Self::SXTW
                | Self::SXTX
                | Self::SXTB
                | Self::SXTH
        )
    }
}

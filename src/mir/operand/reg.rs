use crate::mir::{
    fmt::FuncFormatContext,
    operand::{IMirSubOperand, MirOperand},
};
use bitflags::bitflags;
use std::fmt::{Debug, Write};

/// Represents a sub-register index with a specific bit width and index.
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

    pub fn format_mir(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[:b{}:{}]", self.get_bits_log2(), self.get_index())
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

impl std::fmt::Display for SubRegIndex {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "[:b{}:{}]", self.get_bits_log2(), self.get_index())
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
        /// This register is used in this instruction
        const USE  = 0b0000_0000_0000_1000;
        /// This register is defined implicitly in this instruction
        const IMPLICIT_DEF = 0b0000_0000_0000_1000;
    }
}

impl std::fmt::Display for RegUseFlags {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut flags = String::new();
        if self.contains(RegUseFlags::DEF) {
            flags.push_str("def ");
        }
        if self.contains(RegUseFlags::DEAD) {
            flags.push_str("dead ");
        }
        if self.contains(RegUseFlags::KILL) {
            flags.push_str("kill ");
        }
        if self.contains(RegUseFlags::IMPLICIT_DEF) {
            flags.push_str("implicit-def ");
        }
        write!(f, "{}", flags.trim_end())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
#[rustfmt::skip]
pub enum RegOP {
    LSL(u8), LSR(u8), ASR(u8),
    UXTB, UXTH, UXTW, SXTB, SXTH, SXTW, SXTX,
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
        #[rustfmt::skip]
        return matches!(
            self,
            Self::UXTB | Self::UXTH | Self::UXTW |
            Self::SXTW | Self::SXTX | Self::SXTB | Self::SXTH
        );
    }
}

impl std::fmt::Display for RegOP {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            RegOP::LSL(bits) => write!(f, "LSL #{}", bits),
            RegOP::LSR(bits) => write!(f, "LSR #{}", bits),
            RegOP::ASR(bits) => write!(f, "ASR #{}", bits),
            RegOP::UXTB => write!(f, "UXTB"),
            RegOP::UXTH => write!(f, "UXTH"),
            RegOP::UXTW => write!(f, "UXTW"),
            RegOP::SXTB => write!(f, "SXTB"),
            RegOP::SXTH => write!(f, "SXTH"),
            RegOP::SXTW => write!(f, "SXTW"),
            RegOP::SXTX => write!(f, "SXTX"),
        }
    }
}

impl std::str::FromStr for RegOP {
    type Err = &'static str;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "LSL" => Ok(RegOP::LSL(0)),
            "LSR" => Ok(RegOP::LSR(0)),
            "ASR" => Ok(RegOP::ASR(0)),
            "UXTB" => Ok(RegOP::UXTB),
            "UXTH" => Ok(RegOP::UXTH),
            "UXTW" => Ok(RegOP::UXTW),
            "SXTB" => Ok(RegOP::SXTB),
            "SXTH" => Ok(RegOP::SXTH),
            "SXTW" => Ok(RegOP::SXTW),
            "SXTX" => Ok(RegOP::SXTX),
            _ => {
                if s.starts_with("LSL #") {
                    let bits: u8 = s[5..].parse().map_err(|_| "Invalid LSL bits")?;
                    Ok(RegOP::LSL(bits))
                } else if s.starts_with("LSR #") {
                    let bits: u8 = s[5..].parse().map_err(|_| "Invalid LSR bits")?;
                    Ok(RegOP::LSR(bits))
                } else if s.starts_with("ASR #") {
                    let bits: u8 = s[5..].parse().map_err(|_| "Invalid ASR bits")?;
                    Ok(RegOP::ASR(bits))
                } else {
                    Err("Unknown RegOP")
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegID {
    Virt(u32), // Virtual register ID (ID >= 33)
    SP,        // Stack Pointer, ID = 32
    ZR,        // Zero Register, ID = 31
    Phys(u32), // Physical register ID (ID < 31)
}

impl RegID {
    pub fn get_real(self) -> u32 {
        match self {
            RegID::Virt(id) => id + 33,
            RegID::SP => 32,
            RegID::ZR => 31,
            RegID::Phys(id) => id,
        }
    }

    pub fn from_real(id: u32) -> Self {
        if id < 31 {
            RegID::Phys(id)
        } else if id == 31 {
            RegID::ZR
        } else if id == 32 {
            RegID::SP
        } else {
            RegID::Virt(id - 33)
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct GPReg(pub u32, pub SubRegIndex, pub RegUseFlags);

impl GPReg {
    pub const RETVAL_POS:  u32 = 0;
    pub const RETADDR_POS: u32 = 30;

    pub fn is_virtual(self) -> bool {
        matches!(RegID::from_real(self.0), RegID::Virt(_))
    }
    pub fn get_id(self) -> RegID {
        RegID::from_real(self.0)
    }
    pub fn get_id_raw(self) -> u32 {
        self.0
    }
    pub fn insert_id_raw(self, id: u32) -> Self {
        let Self(_, si, uf) = self;
        Self(id, si, uf)
    }
    pub fn set_id_raw(&mut self, id: u32) {
        *self = self.insert_id_raw(id)
    }
    pub fn insert_id(self, id: RegID) -> Self {
        self.insert_id_raw(id.get_real())
    }
    pub fn set_id(&mut self, id: RegID) {
        self.set_id_raw(id.get_real())
    }

    pub fn get_subreg_index(self) -> SubRegIndex {
        self.1
    }
    pub fn insert_subreg_index(self, subreg_index: SubRegIndex) -> Self {
        let Self(id, _, uf) = self;
        Self(id, subreg_index, uf)
    }
    pub fn set_subreg_index(&mut self, subreg_index: SubRegIndex) {
        *self = self.insert_subreg_index(subreg_index)
    }
    pub fn get_bits_log2(self) -> u8 {
        self.1.get_bits_log2()
    }
    pub fn insert_bits_log2(self, bits_log2: u8) -> Self {
        let Self(id, si, uf) = self;
        Self(id, si.insert_bits_log2(bits_log2), uf)
    }
    pub fn set_bits_log2(&mut self, bits_log2: u8) {
        *self = self.insert_bits_log2(bits_log2)
    }

    pub fn get_use_flags(self) -> RegUseFlags {
        self.2
    }
    pub fn insert_use_flags(self, use_flags: RegUseFlags) -> Self {
        let Self(id, si, _) = self;
        Self(id, si, use_flags)
    }
    pub fn set_use_flags(&mut self, use_flags: RegUseFlags) {
        *self = self.insert_use_flags(use_flags)
    }

    pub fn new_long(id: RegID) -> Self {
        GPReg(id.get_real(), SubRegIndex::new(6, 0), RegUseFlags::empty())
    }
    pub fn new_word(id: RegID) -> Self {
        GPReg(id.get_real(), SubRegIndex::new(5, 0), RegUseFlags::empty())
    }
    pub fn new_ra() -> Self {
        GPReg(30, SubRegIndex::new(6, 0), RegUseFlags::empty())
    }
}

impl IMirSubOperand for GPReg {
    type RealRepresents = GPReg;

    fn new_empty() -> Self {
        GPReg(0, SubRegIndex::new(6, 0), RegUseFlags::empty())
    }

    fn from_mir(mir: MirOperand) -> Self {
        if let MirOperand::GPReg(reg) = mir {
            reg
        } else {
            panic!("Expected MirOperand::GPReg, found {:?}", mir);
        }
    }
    fn into_mir(self) -> MirOperand {
        MirOperand::GPReg(self)
    }
    fn from_real(real: GPReg) -> Self {
        real
    }
    fn into_real(self) -> GPReg {
        self
    }
    fn insert_to_real(self, real: Self) -> Self {
        let Self(_, _, uf) = real;
        let Self(id, si, _) = self;
        Self(id, si, uf)
    }

    fn fmt_asm(&self, formatter: &mut FuncFormatContext<'_>) -> std::fmt::Result {
        let id_str = match self.get_id() {
            RegID::Phys(id) => id.to_string(),
            RegID::Virt(id) => format!("v{}", id),
            RegID::SP => "sp".to_string(),
            RegID::ZR => "zr".to_string(),
        };
        match self.get_subreg_index().get_bits_log2() {
            5 => write!(formatter, "w{}", id_str),
            _ => write!(formatter, "x{}", id_str),
        }
    }
}

/// vector or floating-point register
/// This is used for both vector registers (Vn) and floating-point registers (Dn).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct VFReg(pub u32, pub SubRegIndex, pub RegUseFlags);

impl VFReg {
    pub const RETVAL_POS: u32 = 0;

    pub fn is_virtual(self) -> bool {
        self.0 >= 31
    }
    pub fn get_id(self) -> RegID {
        if self.0 < 31 {
            RegID::Phys(self.0)
        } else {
            RegID::Virt(self.0)
        }
    }

    pub fn get_id_raw(self) -> u32 {
        self.0
    }
    pub fn insert_id_raw(self, id: u32) -> Self {
        let Self(_, si, uf) = self;
        Self(id, si, uf)
    }
    pub fn set_id_raw(&mut self, id: u32) {
        *self = self.insert_id_raw(id)
    }
    pub fn insert_id(self, id: RegID) -> Self {
        self.insert_id_raw(id.get_real())
    }
    pub fn set_id(&mut self, id: RegID) {
        self.set_id_raw(id.get_real())
    }

    pub fn get_subreg_index(self) -> SubRegIndex {
        self.1
    }
    pub fn insert_subreg_index(self, subreg_index: SubRegIndex) -> Self {
        let Self(id, _, uf) = self;
        Self(id, subreg_index, uf)
    }
    pub fn set_subreg_index(&mut self, subreg_index: SubRegIndex) {
        *self = self.insert_subreg_index(subreg_index)
    }
    pub fn get_bits_log2(self) -> u8 {
        self.1.get_bits_log2()
    }
    pub fn insert_bits_log2(self, bits_log2: u8) -> Self {
        let Self(id, si, uf) = self;
        Self(id, si.insert_bits_log2(bits_log2), uf)
    }
    pub fn set_bits_log2(&mut self, bits_log2: u8) {
        *self = self.insert_bits_log2(bits_log2)
    }

    pub fn get_use_flags(self) -> RegUseFlags {
        self.2
    }
    pub fn insert_use_flags(self, use_flags: RegUseFlags) -> Self {
        let Self(id, si, _) = self;
        Self(id, si, use_flags)
    }
    pub fn set_use_flags(&mut self, use_flags: RegUseFlags) {
        *self = self.insert_use_flags(use_flags)
    }

    pub fn new_double(id: RegID) -> Self {
        VFReg(id.get_real(), SubRegIndex::new(6, 0), RegUseFlags::empty())
    }
    pub fn new_single(id: RegID) -> Self {
        VFReg(id.get_real(), SubRegIndex::new(5, 0), RegUseFlags::empty())
    }
}

impl IMirSubOperand for VFReg {
    type RealRepresents = VFReg;

    fn new_empty() -> Self {
        VFReg(0, SubRegIndex::new(6, 0), RegUseFlags::empty())
    }

    fn from_mir(mir: MirOperand) -> Self {
        if let MirOperand::VFReg(reg) = mir {
            reg
        } else {
            panic!("Expected MirOperand::VFReg, found {:?}", mir);
        }
    }

    fn into_mir(self) -> MirOperand {
        MirOperand::VFReg(self)
    }
    fn from_real(real: Self) -> Self {
        real
    }
    fn into_real(self) -> Self {
        self
    }
    fn insert_to_real(self, real: Self) -> Self {
        let Self(_, _, uf) = real;
        let Self(id, si, _) = self;
        Self(id, si, uf)
    }

    fn fmt_asm(&self, _formatter: &mut FuncFormatContext<'_>) -> std::fmt::Result {
        let id_str = match self.get_id() {
            RegID::Phys(id) => id.to_string(),
            RegID::Virt(id) => format!("v{}", id + 33),
            RegID::SP | RegID::ZR => panic!("VFReg cannot be SP or ZR"),
        };
        match self.get_subreg_index().get_bits_log2() {
            5 => write!(_formatter, "s{}", id_str),
            _ => write!(_formatter, "d{}", id_str),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PState(pub RegUseFlags);

impl PState {
    pub fn in_cmp() -> Self {
        PState(RegUseFlags::DEF | RegUseFlags::IMPLICIT_DEF)
    }
}

impl IMirSubOperand for PState {
    type RealRepresents = PState;

    fn new_empty() -> Self {
        PState(RegUseFlags::empty())
    }
    fn from_mir(mir: MirOperand) -> Self {
        if let MirOperand::PState(pstate) = mir {
            pstate
        } else {
            panic!("Expected MirOperand::PState, found {:?}", mir);
        }
    }
    fn into_mir(self) -> MirOperand {
        MirOperand::PState(self)
    }

    fn from_real(real: Self) -> Self {
        real
    }
    fn into_real(self) -> Self {
        self
    }
    fn insert_to_real(self, real: Self) -> Self {
        real // PState does not have an ID or sub-register index, so we can return it directly
    }
    fn fmt_asm(&self, _formatter: &mut FuncFormatContext<'_>) -> std::fmt::Result {
        write!(_formatter, "PSTATE")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GPR32(pub u32, pub RegUseFlags);

impl IMirSubOperand for GPR32 {
    type RealRepresents = GPReg;

    fn new_empty() -> Self {
        Self(0, RegUseFlags::empty())
    }
    fn from_mir(mir: MirOperand) -> Self {
        Self::from_real(GPReg::from_mir(mir))
    }
    fn into_mir(self) -> MirOperand {
        MirOperand::GPReg(self.into_real())
    }
    fn from_real(real: GPReg) -> Self {
        Self(real.0, real.2)
    }
    fn into_real(self) -> GPReg {
        GPReg(self.0, SubRegIndex::new(5, 0), self.1)
    }
    fn insert_to_real(self, real: GPReg) -> GPReg {
        let Self(_, uf) = self;
        let GPReg(id, _, _) = real;
        GPReg(id, SubRegIndex::new(5, 0), uf)
    }
    fn fmt_asm(&self, _formatter: &mut FuncFormatContext<'_>) -> std::fmt::Result {
        self.into_real().fmt_asm(_formatter)
    }
}

impl GPR32 {
    pub fn retval() -> Self {
        GPR32(0, RegUseFlags::empty())
    }
    pub fn zr() -> Self {
        GPR32(31, RegUseFlags::empty())
    }
    pub fn sp() -> Self {
        GPR32(32, RegUseFlags::empty())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GPR64(pub u32, pub RegUseFlags);

impl IMirSubOperand for GPR64 {
    type RealRepresents = GPReg;

    fn new_empty() -> Self {
        Self(0, RegUseFlags::empty())
    }
    fn from_mir(mir: MirOperand) -> Self {
        Self::from_real(GPReg::from_mir(mir))
    }
    fn into_mir(self) -> MirOperand {
        MirOperand::GPReg(self.into_real())
    }
    fn from_real(real: GPReg) -> Self {
        Self(real.0, real.2)
    }
    fn into_real(self) -> GPReg {
        GPReg(self.0, SubRegIndex::new(6, 0), self.1)
    }
    fn insert_to_real(self, real: GPReg) -> GPReg {
        let Self(_, uf) = self;
        let GPReg(id, _, _) = real;
        GPReg(id, SubRegIndex::new(6, 0), uf)
    }
    fn fmt_asm(&self, _formatter: &mut FuncFormatContext<'_>) -> std::fmt::Result {
        self.into_real().fmt_asm(_formatter)
    }
}

impl GPR64 {
    pub fn retval() -> Self {
        GPR64(0, RegUseFlags::empty())
    }
    pub fn zr() -> Self {
        GPR64(31, RegUseFlags::empty())
    }
    pub fn sp() -> Self {
        GPR64(32, RegUseFlags::empty())
    }
    pub fn ra() -> Self {
        GPR64(30, RegUseFlags::empty())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FPR32(pub u32, pub RegUseFlags);

impl FPR32 {
    pub fn retval() -> Self {
        FPR32(0, RegUseFlags::empty())
    }
}

impl IMirSubOperand for FPR32 {
    type RealRepresents = VFReg;

    fn new_empty() -> Self {
        Self(0, RegUseFlags::empty())
    }
    fn from_mir(mir: MirOperand) -> Self {
        Self::from_real(VFReg::from_mir(mir))
    }
    fn into_mir(self) -> MirOperand {
        MirOperand::VFReg(self.into_real())
    }
    fn from_real(real: VFReg) -> Self {
        Self(real.0, real.2)
    }
    fn into_real(self) -> VFReg {
        VFReg(self.0, SubRegIndex::new(5, 0), self.1)
    }
    fn insert_to_real(self, real: VFReg) -> VFReg {
        let Self(_, uf) = self;
        let VFReg(id, _, _) = real;
        VFReg(id, SubRegIndex::new(5, 0), uf)
    }
    fn fmt_asm(&self, _formatter: &mut FuncFormatContext<'_>) -> std::fmt::Result {
        self.into_real().fmt_asm(_formatter)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FPR64(pub u32, pub RegUseFlags);

impl FPR64 {
    pub fn retval() -> Self {
        FPR64(0, RegUseFlags::empty())
    }
}

impl IMirSubOperand for FPR64 {
    type RealRepresents = VFReg;

    fn new_empty() -> Self {
        Self(0, RegUseFlags::empty())
    }
    fn from_mir(mir: MirOperand) -> Self {
        Self::from_real(VFReg::from_mir(mir))
    }
    fn into_mir(self) -> MirOperand {
        MirOperand::VFReg(self.into_real())
    }
    fn from_real(real: VFReg) -> Self {
        Self(real.0, real.2)
    }
    fn into_real(self) -> VFReg {
        VFReg(self.0, SubRegIndex::new(6, 0), self.1)
    }
    fn insert_to_real(self, real: VFReg) -> VFReg {
        let Self(_, uf) = self;
        let VFReg(id, _, _) = real;
        VFReg(id, SubRegIndex::new(6, 0), uf)
    }
    fn fmt_asm(&self, _formatter: &mut FuncFormatContext<'_>) -> std::fmt::Result {
        self.into_real().fmt_asm(_formatter)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RegOperand(pub u32, pub SubRegIndex, pub RegUseFlags, pub bool);

impl RegOperand {
    pub fn is_fp(&self) -> bool {
        self.3
    }
    pub fn get_id(&self) -> RegID {
        RegID::from_real(self.0)
    }
    pub fn insert_id(&self, id: RegID) -> Self {
        self.insert_id_raw(id.get_real())
    }
    pub fn set_id(&mut self, id: RegID) {
        self.set_id_raw(id.get_real())
    }

    pub fn get_id_raw(&self) -> u32 {
        self.0
    }
    pub fn insert_id_raw(&self, id: u32) -> Self {
        let Self(_, si, uf, is_fp) = self;
        Self(id, *si, *uf, *is_fp)
    }
    pub fn set_id_raw(&mut self, id: u32) {
        *self = self.insert_id_raw(id)
    }

    pub fn get_subreg_index(&self) -> SubRegIndex {
        self.1
    }
    pub fn insert_subreg_index(&self, subreg_index: SubRegIndex) -> Self {
        let Self(id, _, uf, is_fp) = self;
        Self(*id, subreg_index, *uf, *is_fp)
    }
    pub fn set_subreg_index(&mut self, subreg_index: SubRegIndex) {
        *self = self.insert_subreg_index(subreg_index)
    }
    pub fn get_bits_log2(&self) -> u8 {
        self.1.get_bits_log2()
    }
    pub fn insert_bits_log2(&self, bits_log2: u8) -> Self {
        let Self(id, si, uf, is_fp) = self;
        Self(*id, si.insert_bits_log2(bits_log2), *uf, *is_fp)
    }
    pub fn set_bits_log2(&mut self, bits_log2: u8) {
        *self = self.insert_bits_log2(bits_log2)
    }

    pub fn get_use_flags(&self) -> RegUseFlags {
        self.2
    }
    pub fn insert_use_flags(&self, use_flags: RegUseFlags) -> Self {
        let Self(id, si, _, is_fp) = self;
        Self(*id, *si, use_flags, *is_fp)
    }
    pub fn set_use_flags(&mut self, use_flags: RegUseFlags) {
        *self = self.insert_use_flags(use_flags)
    }

    pub fn as_physical(&self) -> Option<u32> {
        match self.get_id() {
            RegID::Phys(x) => Some(x),
            _ => None,
        }
    }
    pub fn is_physical(&self) -> bool {
        matches!(self.get_id(), RegID::Phys(_))
    }

    pub fn as_virtual(&self) -> Option<u32> {
        match self.get_id() {
            RegID::Virt(x) => Some(x),
            _ => None,
        }
    }
    pub fn is_virtual(&self) -> bool {
        matches!(self.get_id(), RegID::Virt(_))
    }

    pub fn same_pos_as<T>(&self, other: T) -> bool
    where
        Self: From<T>,
    {
        let other = Self::from(other);
        self.get_id() == other.get_id() && self.get_use_flags() == other.get_use_flags()
    }
}

impl From<GPReg> for RegOperand {
    fn from(reg: GPReg) -> Self {
        let GPReg(id, si, uf) = reg;
        RegOperand(id, si, uf, false)
    }
}

impl From<VFReg> for RegOperand {
    fn from(reg: VFReg) -> Self {
        let VFReg(id, si, uf) = reg;
        RegOperand(id, si, uf, true)
    }
}

impl Into<GPReg> for RegOperand {
    fn into(self) -> GPReg {
        let RegOperand(id, si, uf, is_fp) = self;
        if is_fp {
            panic!("Cannot convert RegOperand to GPReg, it is a VFReg");
        }
        GPReg(id, si, uf)
    }
}

impl Into<VFReg> for RegOperand {
    fn into(self) -> VFReg {
        let RegOperand(id, si, uf, is_fp) = self;
        if !is_fp {
            panic!("Cannot convert RegOperand to VFReg, it is a GPReg");
        }
        VFReg(id, si, uf)
    }
}

impl Into<MirOperand> for RegOperand {
    fn into(self) -> MirOperand {
        let RegOperand(id, si, uf, is_fp) = self;
        if is_fp {
            MirOperand::VFReg(VFReg(id, si, uf))
        } else {
            MirOperand::GPReg(GPReg(id, si, uf))
        }
    }
}

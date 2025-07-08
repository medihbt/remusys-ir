use std::fmt::Debug;

use bitflags::bitflags;

use crate::{
    mir::operand::{
        MirOperand,
        suboperand::{IMirSubOperand, RegOperand},
    },
    typing::{id::ValTypeID, types::FloatTypeKind},
};

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

/// 虚拟寄存器, 在寄存器分配之前使用.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VReg {
    General(u32, SubRegIndex, RegUseFlags),
    Float(u32, SubRegIndex, RegUseFlags),
}

impl VReg {
    pub fn new_long(reg_id: u32) -> Self {
        VReg::General(reg_id, SubRegIndex::new(6, 0), RegUseFlags::NONE)
    }
    pub fn new_int(reg_id: u32) -> Self {
        VReg::General(reg_id, SubRegIndex::new(5, 0), RegUseFlags::NONE)
    }
    pub fn new_double(reg_id: u32) -> Self {
        VReg::Float(reg_id, SubRegIndex::new(6, 0), RegUseFlags::NONE)
    }
    pub fn new_float(reg_id: u32) -> Self {
        VReg::Float(reg_id, SubRegIndex::new(5, 0), RegUseFlags::NONE)
    }

    pub fn new_from_type(ir_type: ValTypeID, reg_id: u32) -> Self {
        match ir_type {
            ValTypeID::Ptr => VReg::new_long(reg_id),
            ValTypeID::Int(bits) => {
                if bits <= 32 {
                    VReg::new_int(reg_id)
                } else {
                    VReg::new_long(reg_id)
                }
            }
            ValTypeID::Float(fp_kind) => match fp_kind {
                FloatTypeKind::Ieee32 => VReg::new_float(reg_id),
                FloatTypeKind::Ieee64 => VReg::new_double(reg_id),
            },
            ValTypeID::Void
            | ValTypeID::Array(_)
            | ValTypeID::Struct(_)
            | ValTypeID::StructAlias(_)
            | ValTypeID::Func(_) => panic!(
                "Cannot create VirtReg from non-primitive type: {:?}",
                ir_type
            ),
        }
    }

    pub fn get_subreg_index(self) -> SubRegIndex {
        match self {
            VReg::General(_, si, _) | VReg::Float(_, si, _) => si,
        }
    }
    pub fn subreg_index_mut(&mut self) -> &mut SubRegIndex {
        match self {
            VReg::General(_, si, _) | VReg::Float(_, si, _) => si,
        }
    }

    pub fn get_use_flags(&self) -> RegUseFlags {
        match self {
            VReg::General(_, _, uf) | VReg::Float(_, _, uf) => *uf,
        }
    }
    pub fn use_flags_mut(&mut self) -> &mut RegUseFlags {
        match self {
            VReg::General(_, _, uf) | VReg::Float(_, _, uf) => uf,
        }
    }
    pub fn add_use_flag(&mut self, flag: RegUseFlags) {
        self.use_flags_mut().insert(flag);
    }
    pub fn insert_use_flags(mut self, flag: RegUseFlags) -> Self {
        self.add_use_flag(flag);
        self
    }
    pub fn del_use_flag(&mut self, flag: RegUseFlags) {
        self.use_flags_mut().remove(flag);
    }
    pub fn extract_use_flag(mut self, flag: RegUseFlags) -> Self {
        self.del_use_flag(flag);
        self
    }

    pub fn get_bits(self) -> u8 {
        match self {
            VReg::General(_, si, _) | VReg::Float(_, si, _) => 1 << si.get_bits_log2(),
        }
    }
    pub fn get_bits_log2(self) -> u8 {
        match self {
            VReg::General(_, si, _) | VReg::Float(_, si, _) => si.get_bits_log2(),
        }
    }
    pub fn get_id(self) -> u32 {
        match self {
            VReg::General(id, _, _) | VReg::Float(id, _, _) => id,
        }
    }
    pub fn insert_id(self, id: u32) -> Self {
        match self {
            VReg::General(_, si, uf) => VReg::General(id, si, uf),
            VReg::Float(_, si, uf) => VReg::Float(id, si, uf),
        }
    }
}

impl std::fmt::Display for VReg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (leading, id, si, uf) = match self {
            VReg::General(id, si, uf) => ("%vg", *id, *si, *uf),
            VReg::Float(id, si, uf) => ("%vf", *id, *si, *uf),
        };
        write!(f, "{} {leading}{}{}", uf.to_string(), id, si)
    }
}

/// Represents a physical register in the ARM architecture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PReg {
    X(u8, SubRegIndex, RegUseFlags), // 64-bit general purpose register (Xn | Wn; n in 0..31)
    V(u8, SubRegIndex, RegUseFlags), // 128-bit vector register (Vn | Dn | Sn)
    SP(SubRegIndex, RegUseFlags),    // Stack pointer (SP)
    ZR(SubRegIndex, RegUseFlags),    // Zero register (ZR)
    PState(RegUseFlags),             // Processor state register (PSTATE)
    PC(SubRegIndex, RegUseFlags),    // Program counter (PC)
}

impl PReg {
    pub const fn sp() -> Self {
        PReg::SP(SubRegIndex::new(6, 0), RegUseFlags::NONE)
    }
    pub const fn is_sp(self) -> bool {
        matches!(self, PReg::SP(..))
    }

    pub const fn x(reg_id: u8) -> Self {
        assert!(reg_id < 32);
        PReg::X(reg_id, SubRegIndex::new(6, 0), RegUseFlags::NONE)
    }
    pub const fn is_x(self) -> bool {
        if let PReg::X(_, si, _) = self {
            si.get_bits_log2() == 6
        } else {
            false
        }
    }
    pub const fn w(reg_id: u8) -> Self {
        assert!(reg_id < 32);
        PReg::X(reg_id, SubRegIndex::new(5, 0), RegUseFlags::NONE)
    }
    pub const fn is_w(self) -> bool {
        if let PReg::X(_, si, _) = self {
            si.get_bits_log2() == 5
        } else {
            false
        }
    }
    pub const fn fp_d(reg_id: u8) -> Self {
        assert!(reg_id < 32);
        PReg::V(reg_id, SubRegIndex::new(6, 0), RegUseFlags::NONE)
    }
    pub const fn is_fp_d(self) -> bool {
        if let PReg::V(_, si, _) = self {
            si.get_bits_log2() == 6
        } else {
            false
        }
    }
    pub const fn fp_s(reg_id: u8) -> Self {
        assert!(reg_id < 32);
        PReg::V(reg_id, SubRegIndex::new(5, 0), RegUseFlags::NONE)
    }
    pub const fn is_fp_s(self) -> bool {
        if let PReg::V(_, si, _) = self {
            si.get_bits_log2() == 5
        } else {
            false
        }
    }
    pub const fn return_addr() -> Self {
        PReg::X(30, SubRegIndex::new(6, 0), RegUseFlags::NONE)
    }
    pub const fn is_return_addr(self) -> bool {
        matches!(self, PReg::X(30, ..))
    }
    pub const fn pc() -> Self {
        PReg::PC(SubRegIndex::new(6, 0), RegUseFlags::NONE)
    }
    pub const fn is_pc(self) -> bool {
        matches!(self, PReg::PC(..))
    }

    pub const fn pstate() -> Self {
        PReg::PState(RegUseFlags::NONE)
    }
    pub const fn is_pstate(self) -> bool {
        matches!(self, PReg::PState(_))
    }

    pub const fn zr() -> Self {
        PReg::ZR(SubRegIndex::new(6, 0), RegUseFlags::NONE)
    }
    pub const fn is_zr(self) -> bool {
        matches!(self, PReg::ZR(..))
    }

    pub fn use_flags_mut(&mut self) -> &mut RegUseFlags {
        match self {
            PReg::X(_, _, uf)
            | PReg::V(_, _, uf)
            | PReg::SP(_, uf)
            | PReg::ZR(_, uf)
            | PReg::PState(uf)
            | PReg::PC(_, uf) => uf,
        }
    }
    pub fn get_use_flags(&self) -> RegUseFlags {
        match self {
            PReg::X(_, _, uf)
            | PReg::V(_, _, uf)
            | PReg::SP(_, uf)
            | PReg::ZR(_, uf)
            | PReg::PState(uf)
            | PReg::PC(_, uf) => *uf,
        }
    }
    pub fn add_use_flag(&mut self, flag: RegUseFlags) {
        self.use_flags_mut().insert(flag);
    }
    pub fn insert_use_flags(mut self, flag: RegUseFlags) -> Self {
        self.add_use_flag(flag);
        self
    }

    pub fn subreg_index_mut(&mut self) -> Option<&mut SubRegIndex> {
        match self {
            PReg::X(_, si, _)
            | PReg::V(_, si, _)
            | PReg::SP(si, _)
            | PReg::ZR(si, _)
            | PReg::PC(si, _) => Some(si),
            PReg::PState(_) => None,
        }
    }
    pub fn id_mut(&mut self) -> Option<&mut u8> {
        match self {
            PReg::X(id, _, _) | PReg::V(id, _, _) => Some(id),
            _ => None,
        }
    }
    pub fn get_bits(&self) -> u8 {
        match self {
            PReg::X(_, subr, _)
            | PReg::V(_, subr, _)
            | PReg::SP(subr, _)
            | PReg::ZR(subr, _)
            | PReg::PC(subr, _) => 1 << subr.get_bits_log2(),
            PReg::PState(_) => 64,
        }
    }
    pub fn get_bits_log2(&self) -> u8 {
        match self {
            PReg::X(_, subr, _)
            | PReg::V(_, subr, _)
            | PReg::SP(subr, _)
            | PReg::ZR(subr, _)
            | PReg::PC(subr, _) => subr.get_bits_log2(),
            PReg::PState(_) => 6, // PState is considered as 64-bit register
        }
    }
}

impl std::fmt::Display for PReg {
    /// Formats the physical register in AArch64 assembly syntax.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PReg::X(id, si, _) => {
                let bits_log2_str = if si.get_bits_log2() == 5 { "w" } else { "x" };
                write!(f, "{}{}", bits_log2_str, id)
            }
            PReg::V(id, si, _) => {
                let bits_log2 = match si.get_bits_log2() {
                    5 => "s",
                    6 => "d",
                    7 => "v",
                    _ => unreachable!("Invalid bits_log2 for vector register"),
                };
                write!(f, "{}{}", bits_log2, id)
            }
            PReg::SP(si, _) => {
                let bits_log2 = match si.get_bits_log2() {
                    5 => "wsp",
                    _ => "sp",
                };
                write!(f, "{}", bits_log2)
            }
            PReg::ZR(si, _) => {
                let bits_log2 = match si.get_bits_log2() {
                    5 => "wzr",
                    _ => "xzr",
                };
                write!(f, "{}", bits_log2)
            }
            PReg::PState(_) => write!(f, "pstate"),
            PReg::PC(si, _) => {
                let bits_log2 = if si.get_bits_log2() == 5 { "wpc" } else { "pc" };
                write!(f, "{}", bits_log2)
            }
        }
    }
}

impl IMirSubOperand for VReg {
    fn from_mirop(operand: MirOperand) -> Self {
        match operand {
            MirOperand::VReg(r) => r,
            _ => panic!("Expected a VReg operand, found: {operand:?}"),
        }
    }
    fn into_mirop(self) -> MirOperand {
        MirOperand::VReg(self)
    }

    fn insert_to_mirop(self, op: MirOperand) -> MirOperand {
        RegOperand::V(self).insert_to_mirop(op)
    }

    fn new_empty_mirsubop() -> Self {
        VReg::new_long(0)
    }
}

impl IMirSubOperand for PReg {
    fn from_mirop(operand: MirOperand) -> Self {
        match operand {
            MirOperand::PReg(r) => r,
            _ => panic!("Expected a PReg operand, found: {operand:?}"),
        }
    }
    fn into_mirop(self) -> MirOperand {
        MirOperand::PReg(self)
    }
    fn insert_to_mirop(self, op: MirOperand) -> MirOperand {
        RegOperand::P(self).insert_to_mirop(op)
    }
    fn new_empty_mirsubop() -> Self {
        PReg::x(0)
    }
}

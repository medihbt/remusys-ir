use std::fmt::Debug;

use bitflags::bitflags;

use crate::typing::{id::ValTypeID, types::FloatTypeKind};

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

impl ToString for RegUseFlags {
    fn to_string(&self) -> String {
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
        flags.trim_end().to_string()
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VirtReg {
    General(u32, SubRegIndex, RegUseFlags),
    Float(u32, SubRegIndex, RegUseFlags),
}

impl VirtReg {
    pub fn new_long(reg_id: u32) -> Self {
        VirtReg::General(reg_id, SubRegIndex::new(6, 0), RegUseFlags::NONE)
    }
    pub fn new_int(reg_id: u32) -> Self {
        VirtReg::General(reg_id, SubRegIndex::new(5, 0), RegUseFlags::NONE)
    }
    pub fn new_double(reg_id: u32) -> Self {
        VirtReg::Float(reg_id, SubRegIndex::new(6, 0), RegUseFlags::NONE)
    }
    pub fn new_float(reg_id: u32) -> Self {
        VirtReg::Float(reg_id, SubRegIndex::new(5, 0), RegUseFlags::NONE)
    }

    pub fn new_from_type(ir_type: ValTypeID, reg_id: u32) -> Self {
        match ir_type {
            ValTypeID::Ptr => VirtReg::new_long(reg_id),
            ValTypeID::Int(bits) => {
                if bits <= 32 {
                    VirtReg::new_int(reg_id)
                } else {
                    VirtReg::new_long(reg_id)
                }
            }
            ValTypeID::Float(fp_kind) => match fp_kind {
                FloatTypeKind::Ieee32 => VirtReg::new_float(reg_id),
                FloatTypeKind::Ieee64 => VirtReg::new_double(reg_id),
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
            VirtReg::General(_, si, _) | VirtReg::Float(_, si, _) => si,
        }
    }
    pub fn subreg_index_mut(&mut self) -> &mut SubRegIndex {
        match self {
            VirtReg::General(_, si, _) | VirtReg::Float(_, si, _) => si,
        }
    }

    pub fn use_flags_mut(&mut self) -> &mut RegUseFlags {
        match self {
            VirtReg::General(_, _, uf) | VirtReg::Float(_, _, uf) => uf,
        }
    }
    pub fn get_use_flags(&self) -> RegUseFlags {
        match self {
            VirtReg::General(_, _, uf) | VirtReg::Float(_, _, uf) => *uf,
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
            VirtReg::General(_, si, _) | VirtReg::Float(_, si, _) => 1 << si.get_bits_log2(),
        }
    }
    pub fn get_id(self) -> u32 {
        match self {
            VirtReg::General(id, _, _) | VirtReg::Float(id, _, _) => id,
        }
    }
}

impl std::fmt::Display for VirtReg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let (leading, id, si, uf) = match self {
            VirtReg::General(id, si, uf) => ("%vg", *id, *si, *uf),
            VirtReg::Float(id, si, uf) => ("%vf", *id, *si, *uf),
        };
        write!(f, "{} {leading}{}{}", uf.to_string(), id, si)
    }
}

/// Represents a physical register in the ARM architecture.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PhysReg {
    X(u8, SubRegIndex, RegUseFlags), // 64-bit general purpose register (Xn | Wn; n in 0..31)
    V(u8, SubRegIndex, RegUseFlags), // 128-bit vector register (Vn | Dn | Sn)
    SP(SubRegIndex, RegUseFlags),    // Stack pointer (SP)
    ZR(SubRegIndex, RegUseFlags),    // Zero register (ZR)
    PState(RegUseFlags),             // Processor state register (PSTATE)
    PC(SubRegIndex, RegUseFlags),    // Program counter (PC)
}

impl PhysReg {
    pub const fn sp() -> Self {
        PhysReg::SP(SubRegIndex::new(6, 0), RegUseFlags::NONE)
    }
    pub const fn is_sp(self) -> bool {
        matches!(self, PhysReg::SP(..))
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

    pub const fn pstate() -> Self {
        PhysReg::PState(RegUseFlags::NONE)
    }
    pub const fn is_pstate(self) -> bool {
        matches!(self, PhysReg::PState(_))
    }

    pub const fn zr() -> Self {
        PhysReg::ZR(SubRegIndex::new(6, 0), RegUseFlags::NONE)
    }
    pub const fn is_zr(self) -> bool {
        matches!(self, PhysReg::ZR(..))
    }

    pub fn use_flags_mut(&mut self) -> &mut RegUseFlags {
        match self {
            PhysReg::X(_, _, uf)
            | PhysReg::V(_, _, uf)
            | PhysReg::SP(_, uf)
            | PhysReg::ZR(_, uf)
            | PhysReg::PState(uf)
            | PhysReg::PC(_, uf) => uf,
        }
    }
    pub fn get_use_flags(&self) -> RegUseFlags {
        match self {
            PhysReg::X(_, _, uf)
            | PhysReg::V(_, _, uf)
            | PhysReg::SP(_, uf)
            | PhysReg::ZR(_, uf)
            | PhysReg::PState(uf)
            | PhysReg::PC(_, uf) => *uf,
        }
    }
    pub fn add_use_flag(&mut self, flag: RegUseFlags) {
        self.use_flags_mut().insert(flag);
    }

    pub fn subreg_index_mut(&mut self) -> Option<&mut SubRegIndex> {
        match self {
            PhysReg::X(_, si, _)
            | PhysReg::V(_, si, _)
            | PhysReg::SP(si, _)
            | PhysReg::ZR(si, _)
            | PhysReg::PC(si, _) => Some(si),
            PhysReg::PState(_) => None,
        }
    }
    pub fn id_mut(&mut self) -> Option<&mut u8> {
        match self {
            PhysReg::X(id, _, _) | PhysReg::V(id, _, _) => Some(id),
            _ => None,
        }
    }
    pub fn get_bits(&self) -> u8 {
        match self {
            PhysReg::X(_, subr, _)
            | PhysReg::V(_, subr, _)
            | PhysReg::SP(subr, _)
            | PhysReg::ZR(subr, _)
            | PhysReg::PC(subr, _) => 1 << subr.get_bits_log2(),
            PhysReg::PState(_) => 64,
        }
    }
}

impl std::fmt::Display for PhysReg {
    /// Formats the physical register in AArch64 assembly syntax.
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            PhysReg::X(id, si, _) => {
                let bits_log2_str = if si.get_bits_log2() == 5 { "w" } else { "x" };
                write!(f, "{}{}", bits_log2_str, id)
            }
            PhysReg::V(id, si, _) => {
                let bits_log2 = match si.get_bits_log2() {
                    5 => "s",
                    6 => "d",
                    7 => "v",
                    _ => unreachable!("Invalid bits_log2 for vector register"),
                };
                write!(f, "{}{}", bits_log2, id)
            }
            PhysReg::SP(si, _) => {
                let bits_log2 = match si.get_bits_log2() {
                    5 => "wsp",
                    _ => "sp",
                };
                write!(f, "{}", bits_log2)
            }
            PhysReg::ZR(si, _) => {
                let bits_log2 = match si.get_bits_log2() {
                    5 => "wzr",
                    _ => "xzr",
                };
                write!(f, "{}", bits_log2)
            }
            PhysReg::PState(_) => write!(f, "pstate"),
            PhysReg::PC(si, _) => {
                let bits_log2 = if si.get_bits_log2() == 5 { "wpc" } else { "pc" };
                write!(f, "{}", bits_log2)
            }
        }
    }
}

/// Represents a register operand, which can be either a virtual or physical register.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegOperand {
    Virt(VirtReg),
    Phys(PhysReg),
}

impl RegOperand {
    pub fn is_phys(&self) -> bool {
        matches!(self, RegOperand::Phys(_))
    }
    pub fn is_virt(&self) -> bool {
        matches!(self, RegOperand::Virt(_))
    }

    pub fn get_bits(&self) -> u8 {
        match self {
            RegOperand::Virt(vr) => vr.get_bits(),
            RegOperand::Phys(pr) => pr.get_bits(),
        }
    }

    pub fn get_use_flags(&self) -> RegUseFlags {
        match self {
            RegOperand::Virt(vr) => vr.get_use_flags(),
            RegOperand::Phys(pr) => pr.get_use_flags(),
        }
    }
    pub fn use_flags_mut(&mut self) -> &mut RegUseFlags {
        match self {
            RegOperand::Virt(vr) => vr.use_flags_mut(),
            RegOperand::Phys(pr) => pr.use_flags_mut(),
        }
    }
}

use std::fmt::Debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VirtReg {
    General(u32, SubRegIndex),
    Float  (u32, SubRegIndex),
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct SubRegIndex(pub u8);

impl SubRegIndex {
    pub fn new(bits_log2: u8, index: u8) -> Self {
        assert!(bits_log2 >= 3 && bits_log2 <= 6);
        assert!(index < 64);
        let bits_log2_flag = bits_log2 - 3;
        SubRegIndex((bits_log2_flag & 0b11) | ((index as u8) << 2))
    }

    /// bits[0..2] is sub-register binary bits flag:
    ///
    /// 00 => 8
    /// 01 => 16
    /// 10 => 32
    /// 11 => 64
    pub const fn get_bits_log2(self) -> u8 {
        let bits_log2_flag = self.0 & 0b11;
        bits_log2_flag + 3
    }
    pub const fn insert_bits_log2(self, bits_log2: u8) -> Self {
        assert!(bits_log2 >= 3 && bits_log2 <= 6);
        let bits_log2_flag = bits_log2 - 3;
        SubRegIndex((self.0 & !0b11) | bits_log2_flag)
    }
    pub fn set_bits_log2(&mut self, bits_log2: u8) {
        *self = self.insert_bits_log2(bits_log2);
    }

    /// bits[2..8] is sub-register index.
    pub const fn get_index(self) -> u8 {
        (self.0 >> 2) & 0b111111
    }
    pub const fn insert_index(self, index: u8) -> Self {
        assert!(index < 64);
        SubRegIndex((self.0 & !0b11111100) | ((index as u8) << 2))
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ShiftOpcode {
    LSL, // Logical Shift Left
    LSR, // Logical Shift Right
    ASR, // Arithmetic Shift Right
    ROR, // Rotate Right
}

pub struct RegShift {
    pub reg: VirtReg,
    pub shift_opcode: ShiftOpcode,
    pub shift_amount: u8, // 0-63
}

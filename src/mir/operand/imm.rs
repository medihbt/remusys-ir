use crate::{
    mir::operand::{MirOperand, suboperand::IMirSubOperand},
    typing::{id::ValTypeID, types::FloatTypeKind},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ImmKind {
    /// 没有任何限制的立即数
    Full,
    /// 计算相关的立即数, 规则是: 要么只有 [0:12] 位有值, 要么只有 [12:24] 位有值.
    Calc,
    /// `ldr r, [r, #i]` 等使用的立即数, 规则是: 一个 9 位的整数, 通过有符号扩展得到原数.
    Load,
    /// 逻辑相关的立即数, 规则是: 立即数应该是一个循环节的位模式.
    Logic,
    /// 条件比较相关的立即数, 规则是: 只有 [0:5] 位有值, 无符号.
    CCmp,
}

impl ImmKind {
    pub const fn verify_imm64(self, value: u64) -> bool {
        match self {
            ImmKind::Full => true,
            ImmKind::Calc => imm_traits::is_calc_imm(value),
            ImmKind::Load => imm_traits::is_load_imm(value),
            ImmKind::Logic => imm_traits::is_logical_imm64(value),
            ImmKind::CCmp => imm_traits::is_condcmp_imm(value),
        }
    }

    pub const fn verify_imm32(self, value: u32) -> bool {
        match self {
            ImmKind::Full => true,
            ImmKind::Calc => imm_traits::is_calc_imm(value as u64),
            ImmKind::Load => imm_traits::is_load_imm(value as u64),
            ImmKind::Logic => imm_traits::is_logical_imm32(value),
            ImmKind::CCmp => imm_traits::is_condcmp_imm(value as u64),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum ImmConst {
    Word(u32, ImmKind),
    Long(u64, ImmKind),
    FP32(f32),
    FP64(f64),
    FMov(u8),
}

#[derive(Debug)]
pub enum ImmVerifyErr {
    NotFeed(ImmKind, u64),
    InvalidKind(ImmKind),
}

impl std::fmt::Display for ImmConst {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ImmConst::Word(value, ImmKind::Load) => {
                // 这里做的是有符号扩展, 使得立即数可以被正确地解释为负数.
                let pmask = if *value & 0x100 != 0 {
                    0xFFFF_FF00i32
                } else {
                    0x0000_0000i32
                };
                let value = *value as i32 | pmask;
                write!(f, "{value}")
            }
            ImmConst::Long(value, ImmKind::Load) => {
                // 这里做的是有符号扩展, 使得立即数可以被正确地解释为负数.
                let pmask = if *value & 0x100 != 0 {
                    0xFFFF_FFFF_FFFF_FF00i64
                } else {
                    0x0000_0000_0000_0000i64
                };
                let value = *value as i64 | pmask;
                write!(f, "{value}")
            }
            ImmConst::Word(value, _) => write!(f, "{value:#x}"),
            ImmConst::Long(value, _) => write!(f, "{value:#x}"),
            ImmConst::FP32(fp) => write!(f, "{fp:e}"),
            ImmConst::FP64(fp) => write!(f, "{fp:e}"),
            ImmConst::FMov(x) => {
                let real_fp = imm_traits::fp8aarch_to_fp32(*x);
                write!(f, "{real_fp:e}")
            }
        }
    }
}

impl PartialEq for ImmConst {
    fn eq(&self, other: &Self) -> bool {
        match (self, other) {
            (Self::Word(l0, l1), Self::Word(r0, r1)) => l0 == r0 && l1 == r1,
            (Self::Long(l0, l1), Self::Long(r0, r1)) => l0 == r0 && l1 == r1,
            (Self::FP32(l0), Self::FP32(r0)) => l0.to_bits() == r0.to_bits(),
            (Self::FP64(l0), Self::FP64(r0)) => l0.to_bits() == r0.to_bits(),
            (Self::FMov(l0), Self::FMov(r0)) => l0 == r0,
            _ => false,
        }
    }
}

impl Eq for ImmConst {}

impl std::hash::Hash for ImmConst {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        match self {
            ImmConst::Word(value, kind) => {
                state.write_u32(*value);
                state.write_u8(*kind as u8);
            }
            ImmConst::Long(value, kind) => {
                state.write_u64(*value);
                state.write_u8(*kind as u8);
            }
            ImmConst::FP32(fp) => state.write_u32(fp.to_bits()),
            ImmConst::FP64(fp) => state.write_u64(fp.to_bits()),
            ImmConst::FMov(x) => state.write_u8(*x),
        }
    }
}

impl IMirSubOperand for ImmConst {
    fn new_empty_mirsubop() -> Self {
        ImmConst::Word(0, ImmKind::Full)
    }
    fn from_mirop(operand: MirOperand) -> Self {
        match operand {
            MirOperand::ImmLimit(imm) => imm,
            MirOperand::Imm(value) => ImmConst::Long(value as u64, ImmKind::Full),
            MirOperand::None => ImmConst::Word(0, ImmKind::Full),
            _ => panic!("Cannot convert {operand:?} to ImmConst"),
        }
    }
    fn into_mirop(self) -> MirOperand {
        match self {
            ImmConst::Word(value, ImmKind::Full) => MirOperand::Imm(value as i64),
            ImmConst::Long(value, ImmKind::Full) => MirOperand::Imm(value as i64),
            _ => MirOperand::ImmLimit(self),
        }
    }
    fn insert_to_mirop(self, _: MirOperand) -> MirOperand {
        self.into_mirop()
    }
}

impl ImmConst {
    /// Returns the immediate constant as a 32-bit integer if it is a Word or FMov.
    pub fn as_float(&self) -> Option<f64> {
        match self {
            ImmConst::FP32(fp) => Some(*fp as f64),
            ImmConst::FP64(fp) => Some(*fp),
            ImmConst::FMov(x) => Some(imm_traits::fp8aarch_to_fp64(*x)),
            _ => None,
        }
    }

    /// Returns the immediate constant as a 64-bit integer if it is a Long or Word.
    pub fn as_bits(&self) -> (u64, u8) {
        match self {
            ImmConst::Word(value, ImmKind::Load) => {
                // 这里做的是有符号扩展, 使得立即数可以被正确地解释为负数.
                let pmask = if *value & 0x100 != 0 {
                    0xFFFF_FF00i32
                } else {
                    0x0000_0000i32
                };
                let out = (*value as i32) | pmask;
                (out as u64, 32)
            }
            ImmConst::Long(value, ImmKind::Load) => {
                // 这里做的是有符号扩展, 使得立即数可以被正确地解释为负数.
                let pmask = if *value & 0x100 != 0 {
                    0xFFFF_FFFF_FFFF_FF00i64
                } else {
                    0x0000_0000_0000_0000i64
                };
                let out = (*value as i64) | pmask;
                (out as u64, 64)
            }
            ImmConst::Word(value, _) => (*value as u64, 32),
            ImmConst::Long(value, _) => (*value, 64),
            ImmConst::FP32(fp) => (fp.to_bits() as u64, 32),
            ImmConst::FP64(fp) => (fp.to_bits(), 64),
            ImmConst::FMov(x) => (*x as u64, 8),
        }
    }

    /// Creates an immediate constant from a 64-bit integer and a type.
    pub fn from_bits(bits: impl Into<u64>, ty: &ValTypeID) -> Self {
        Self::from_bits_u64(bits.into(), ty)
    }
    fn from_bits_u64(bits: u64, ty: &ValTypeID) -> Self {
        match ty {
            ValTypeID::Int(32) => Self::Word(bits as u32, ImmKind::Full),
            ValTypeID::Ptr | ValTypeID::Int(64) => Self::Long(bits, ImmKind::Full),
            ValTypeID::Int(x) => {
                panic!("Unsupported integer type for immediate constant: i{x}")
            }
            ValTypeID::Float(FloatTypeKind::Ieee32) => Self::FP32(f32::from_bits(bits as u32)),
            ValTypeID::Float(FloatTypeKind::Ieee64) => Self::FP64(f64::from_bits(bits)),
            _ => {
                panic!("Unsupported type for immediate constant: {ty:?}")
            }
        }
    }

    /// Creates an immediate constant with a value of zero for the given type.
    pub fn new_zero(ty: &ValTypeID) -> Self {
        Self::from_bits_u64(0, ty)
    }

    pub fn new_u32(value: u32, kind: ImmKind) -> Result<Self, ImmVerifyErr> {
        if !kind.verify_imm32(value) {
            Err(ImmVerifyErr::NotFeed(kind, value as u64))
        } else {
            Ok(Self::Word(value, kind))
        }
    }
    pub fn new_u64(value: u64, kind: ImmKind) -> Result<Self, ImmVerifyErr> {
        if !kind.verify_imm64(value) {
            Err(ImmVerifyErr::NotFeed(kind, value))
        } else {
            Ok(Self::Long(value, kind))
        }
    }
    pub fn new_f32(value: f32, kind: ImmKind) -> Result<Self, ImmVerifyErr> {
        if !matches!(kind, ImmKind::Full | ImmKind::Calc) {
            Err(ImmVerifyErr::InvalidKind(kind))
        } else if !kind.verify_imm64(value.to_bits() as u64) {
            Err(ImmVerifyErr::NotFeed(kind, value.to_bits() as u64))
        } else {
            Ok(Self::FP32(value))
        }
    }
    pub fn new_f64(value: f64, kind: ImmKind) -> Result<Self, ImmVerifyErr> {
        if !matches!(kind, ImmKind::Full | ImmKind::Calc) {
            Err(ImmVerifyErr::InvalidKind(kind))
        } else if !kind.verify_imm64(value.to_bits()) {
            Err(ImmVerifyErr::NotFeed(kind, value.to_bits()))
        } else {
            Ok(Self::FP64(value))
        }
    }
}

pub mod imm_traits {
    /// 检查该 32 位立即数是否满足逻辑 and 指令的立即数条件;
    /// 具体来说, 立即数应该是一段重复循环的位模式.
    pub const fn is_logical_imm32(imm32: u32) -> bool {
        use super::loop_pattern::*;
        // 先检查一些简单的重复模式
        match imm32 {
            // and 指令不接受循环节为 1 位的立即数.
            // 这个可能是因为这玩意可以直接算出来.
            0x0000_0000 | 0xFFFF_FFFF => return false,
            // 2 位循环节的模式
            0x5555_5555 | 0xAAAA_AAAA => return true,
            // 4 位循环节的模式
            0x1111_1111 | 0x2222_2222 | 0x4444_4444 | 0x8888_8888 | 0x3333_3333 | 0x6666_6666
            | 0xCCCC_CCCC | 0x9999_9999 | 0x7777_7777 | 0xEEEE_EEEE | 0xDDDD_DDDD | 0xBBBB_BBBB => {
                return true;
            }
            // 8 位及以上循环节模式情况比较多了, 不能再做这种匹配了.
            _ => {}
        };

        // 检查是否满足循环节为 8 位的模式
        let imm32 = imm32.rotate_right(imm32.trailing_ones());
        let imm32 = imm32.rotate_right(imm32.trailing_zeros());
        is_loop8_pattern32(imm32) || is_loop16_pattern32(imm32) || is_loop32_pattern32(imm32)
    }

    /// 检查该 64 位立即数是否满足逻辑 and 指令的立即数条件;
    /// 具体来说, 立即数应该是一段重复循环的位模式.
    pub const fn is_logical_imm64(imm64: u64) -> bool {
        use super::loop_pattern::*;
        // 先检查一些简单的重复模式
        match imm64 {
            // and 指令不接受循环节为 1 位的立即数.
            // 这个可能是因为这玩意可以直接算出来.
            0x0000_0000_0000_0000 | 0xFFFF_FFFF_FFFF_FFFF => return false,
            // 2 位循环节的模式
            0x5555_5555_5555_5555 | 0xAAAA_AAAA_AAAA_AAAA => return true,
            // 4 位循环节的模式
            0x1111_1111_1111_1111
            | 0x2222_2222_2222_2222
            | 0x4444_4444_4444_4444
            | 0x8888_8888_8888_8888
            | 0x3333_3333_3333_3333
            | 0x6666_6666_6666_6666
            | 0xCCCC_CCCC_CCCC_CCCC
            | 0x9999_9999_9999_9999
            | 0x7777_7777_7777_7777
            | 0xEEEE_EEEE_EEEE_EEEE
            | 0xDDDD_DDDD_DDDD_DDDD
            | 0xBBBB_BBBB_BBBB_BBBB => {
                return true;
            }
            // 8 位及以上循环节模式情况比较多了, 不能再做这种匹配了.
            _ => {}
        };

        // 检查是否满足循环节为 8 位的模式
        let imm64 = imm64.rotate_right(imm64.trailing_ones());
        let imm64 = imm64.rotate_right(imm64.trailing_zeros());
        is_loop8_pattern64(imm64)
            || is_loop16_pattern64(imm64)
            || is_loop32_pattern64(imm64)
            || is_loop64_pattern64(imm64)
    }

    pub const fn is_calc_imm(imm: u64) -> bool {
        if imm < 4096 {
            // 12 位以内的立即数可以直接计算
            return true;
        }
        if imm & 0xFFF != 0 {
            // 12 位以上的立即数, 需要满足 imm & 0xFFF == 0
            return false;
        }
        let imm = imm >> 12;
        imm < 4096
    }

    /// Converts aarch64 `fmov` instruction immediate format
    /// to host float `f32`
    pub const fn fp8aarch_to_fp32(imm: u8) -> f32 {
        let sign = imm >> 7;
        let exp = (imm >> 4) & 0b111;
        let mantissa = imm & 0b1111;

        // [1,2,3,4,-3,-2,-1,0] -> [128, 129, 130, 131, 124, 125, 126, 127]
        let fp32_exp = if exp < 4 { 128 } else { 120 } + exp as u32;
        let fp32_mantissa = (mantissa as u32) << 19;
        let fp32_sign = (sign as u32) << 31;
        let fp32 = fp32_sign | (fp32_exp << 23) | fp32_mantissa;
        f32::from_bits(fp32)
    }

    /// Converts aarch64 `fmov` instruction immediate format
    /// to host float `f64`.
    pub const fn fp8aarch_to_fp64(imm: u8) -> f64 {
        let sign = imm >> 7;
        let exp = (imm >> 4) & 0b111;
        let mantissa = imm & 0b1111;

        // [1,2,3,4,-3,-2,-1,0] -> [1024, 1025, 1026, 1027, 1020, 1021, 1022, 1023]
        let fp64_exp = if exp < 4 { 1024 } else { 1016 } + exp as u64;
        let fp64_mantissa = (mantissa as u64) << 48;
        let fp64_sign = (sign as u64) << 63;
        let fp64 = fp64_sign | (fp64_exp << 52) | fp64_mantissa;
        f64::from_bits(fp64)
    }

    pub const fn try_cast_f32_to_aarch8(imm: f32) -> Option<u8> {
        let bits = imm.to_bits();
        let sign = (bits >> 31) as u8;
        let exp = ((bits >> 23) & 0xFF) as u8;
        let mantissa = bits & 0x7F_FFFF;
        if exp < 124 || exp > 131 {
            return None; // Exponent out of range for fp8
        }
        if mantissa & 0x07_FFFF != 0 {
            return None;
        }
        let exp = if exp >= 128 { exp - 128 } else { exp - 120 };
        let mantissa = (mantissa >> 19) as u8;
        let imm8 = (sign << 7) | (exp << 4) | mantissa;
        Some(imm8)
    }

    pub const fn try_cast_f64_to_aarch8(imm: f64) -> Option<u8> {
        let bits = imm.to_bits();
        let sign = (bits >> 63) as u8;
        let exp = (bits >> 52) & 0x7FF;
        let mantissa = bits & 0x000F_FFFF_FFFF_FFFF; // 52 bits of mantissa
        if exp < 1020 || exp > 1027 {
            return None; // Exponent out of range for fp8
        }
        if mantissa & 0x0000_FFFF_FFFF_FFFF != 0 {
            // Mantissa must be 4 bits
            return None;
        }
        let exp = if exp >= 1024 { exp - 1024 } else { exp - 1016 };
        let mantissa = (mantissa >> 48) as u8;
        let imm8 = (sign << 7) | ((exp as u8) << 4) | mantissa;
        Some(imm8)
    }

    /// 检查该 64 位立即数是否满足条件比较指令的立即数条件;
    /// 具体来说, 立即数应该是一个无符号的 5 位数.
    pub const fn is_condcmp_imm(imm: u64) -> bool {
        imm & 0xFFFF_FFFF_FFFF_FFE0 != 0
    }

    /// 检查该 64 位立即数是否满足 `ldr r, [r, #i]` 等指令的立即数条件;
    /// 具体来说, 立即数应该是一个 9 位的整数.
    pub const fn is_load_imm(imm: u64) -> bool {
        // 检查是否是一个 9 位的整数, 通过有符号扩展得到原数
        (imm & 0x1FF) == imm && imm <= 0x1FF
    }
}

mod loop_pattern {
    pub(super) const fn is_loop8_pattern32(imm: u32) -> bool {
        let imm8 = (imm & 0xFF) as u8;
        if !(imm8 + 1).is_power_of_two() {
            return false;
        }
        let should_be = imm8 as u32;
        let should_be = should_be | (should_be << 8);
        let should_be = should_be | (should_be << 16);
        imm == should_be
    }

    pub(super) const fn is_loop16_pattern32(imm: u32) -> bool {
        let imm16 = (imm & 0xFFFF) as u16;
        if !(imm16 + 1).is_power_of_two() {
            return false;
        }
        let should_be = imm16 as u32;
        let should_be = should_be | (should_be << 16);
        imm == should_be
    }

    pub(super) const fn is_loop32_pattern32(imm: u32) -> bool {
        (imm + 1).is_power_of_two()
    }

    pub(super) const fn is_loop8_pattern64(imm: u64) -> bool {
        let imm8 = (imm & 0xFF) as u8;
        if !(imm8 + 1).is_power_of_two() {
            return false;
        }
        let should_be = imm8 as u64;
        let should_be = should_be | (should_be << 8);
        let should_be = should_be | (should_be << 16);
        let should_be = should_be | (should_be << 32);
        imm == should_be
    }

    pub(super) const fn is_loop16_pattern64(imm: u64) -> bool {
        let imm16 = (imm & 0xFFFF) as u16;
        if !(imm16 + 1).is_power_of_two() {
            return false;
        }
        let should_be = imm16 as u64;
        let should_be = should_be | (should_be << 16);
        let should_be = should_be | (should_be << 32);
        imm == should_be
    }

    pub(super) const fn is_loop32_pattern64(imm: u64) -> bool {
        let imm32 = (imm & 0xFFFFFFFF) as u32;
        if !(imm32 + 1).is_power_of_two() {
            return false;
        }
        let should_be = imm32 as u64;
        let should_be = should_be | (should_be << 32);
        imm == should_be
    }

    pub(super) const fn is_loop64_pattern64(imm: u64) -> bool {
        (imm + 1).is_power_of_two()
    }
}

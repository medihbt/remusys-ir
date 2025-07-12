use crate::mir::operand::{IMirSubOperand, MirOperand, imm_traits};
use std::fmt::Write;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImmKind {
    Full,
    /// 计算相关的立即数, 规则是: 要么只有 [0:12] 位有值, 要么只有 [12:24] 位有值.
    Calc,
    /// 逻辑相关的立即数, 规则是: 立即数应该是一个循环节的位模式.
    Logic,
    /// `ldr r, [r, #i]` 等使用的立即数, 规则是: 一个 9 位的整数, 通过有符号扩展得到原数.
    Load,
    /// 条件比较相关的立即数, 规则是: 只有 [0:5] 位有值, 无符号.
    CCmp,
    /// 移动指令相关的立即数.
    Mov,
    /// 浮点移动指令相关的立即数.
    FMov,
    /// 最大/最小值相关的立即数, 有符号变体
    SMax,
    /// 最大/最小值相关的立即数, 无符号变体
    UMax,
    /// 移位操作相关的立即数.
    Shift,
}

impl ImmKind {
    pub fn check_u32(&self, imm: u32) -> bool {
        match self {
            ImmKind::Full => true,
            ImmKind::Calc => imm_traits::is_calc_imm(imm as u64),
            ImmKind::Logic => imm_traits::is_logical_imm32(imm),
            ImmKind::Load => imm_traits::is_load32_imm(imm as i64),
            ImmKind::CCmp => imm_traits::is_condcmp_imm(imm as u64),
            ImmKind::Mov => imm_traits::is_mov_imm(imm as u64),
            ImmKind::FMov => imm_traits::try_cast_f32_to_aarch8(f32::from_bits(imm)).is_some(),
            ImmKind::SMax => imm_traits::is_smax_imm(imm as i32 as i64),
            ImmKind::UMax => imm_traits::is_umax_imm(imm as u64),
            ImmKind::Shift => imm_traits::is_shift_imm(imm as u64),
        }
    }

    pub fn check_u64(&self, imm: u64) -> bool {
        match self {
            ImmKind::Full => true,
            ImmKind::Calc => imm_traits::is_calc_imm(imm),
            ImmKind::Logic => imm_traits::is_logical_imm64(imm),
            ImmKind::Load => imm_traits::is_load64_imm(imm as i64),
            ImmKind::CCmp => imm_traits::is_condcmp_imm(imm),
            ImmKind::Mov => imm_traits::is_mov_imm(imm),
            ImmKind::FMov => imm_traits::try_cast_f64_to_aarch8(f64::from_bits(imm)).is_some(),
            ImmKind::SMax => imm_traits::is_smax_imm(imm as i64),
            ImmKind::UMax => imm_traits::is_umax_imm(imm),
            ImmKind::Shift => imm_traits::is_shift_imm(imm),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Imm64(pub u64, pub ImmKind);

impl Imm64 {
    pub fn new(value: u64, flags: ImmKind) -> Self {
        Self(value, flags)
    }

    pub fn get_value(&self) -> u64 {
        self.0
    }
    pub fn set_value(&mut self, value: u64) {
        self.0 = value;
    }

    pub fn get_kind(&self) -> ImmKind {
        self.1
    }
    pub fn set_kind(&mut self, kind: ImmKind) {
        self.1 = kind;
    }
}

impl IMirSubOperand for Imm64 {
    type RealRepresents = Imm64;

    fn new_empty() -> Self {
        Self(0, ImmKind::Full)
    }

    fn from_mir(mir: MirOperand) -> Self {
        if let MirOperand::Imm64(imm) = mir {
            imm
        } else {
            panic!("Expected MirOperand::Imm64, found {:?}", mir);
        }
    }
    fn into_mir(self) -> MirOperand {
        MirOperand::Imm64(self)
    }

    fn from_real(real: Self) -> Self {
        real
    }
    fn into_real(self) -> Self {
        self
    }

    fn insert_to_real(self, real: Self) -> Self {
        if real.1.check_u64(self.0) {
            Self(self.0, real.1)
        } else {
            panic!(
                "Invalid immediate value: {} for flags: {:?}",
                self.0, real.1
            );
        }
    }

    fn fmt_asm(&self, formatter: &mut crate::mir::fmt::FormatContext<'_>) -> std::fmt::Result {
        let is_fp = formatter.operand_context.is_fp;
        match self.get_kind() {
            ImmKind::Full => {
                if is_fp {
                    write!(formatter.writer, "#{:e}", f64::from_bits(self.0))
                } else {
                    write!(formatter.writer, "#{:#X}", self.0)
                }
            }
            ImmKind::Calc => write!(formatter.writer, "#{:#X}", self.0),
            ImmKind::Logic => write!(formatter.writer, "#{:#X}", self.0),
            ImmKind::Load => write!(formatter.writer, "#{:#X}", self.0),
            ImmKind::CCmp => write!(formatter.writer, "#{:#X}", self.0),
            ImmKind::Mov => write!(formatter.writer, "#{:#X}", self.0),
            ImmKind::FMov => write!(formatter.writer, "#{:e}", f64::from_bits(self.0)),
            ImmKind::SMax => write!(formatter.writer, "#{:#X}", self.0 as i64),
            ImmKind::UMax => write!(formatter.writer, "#{:#X}", self.0 as u8 as u64),
            ImmKind::Shift => write!(formatter.writer, "#{:#X}", self.0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct Imm32(pub u32, pub ImmKind);

impl Imm32 {
    pub fn new(value: u32, flags: ImmKind) -> Self {
        Self(value, flags)
    }

    pub fn get_value(&self) -> u32 {
        self.0
    }
    pub fn set_value(&mut self, value: u32) {
        self.0 = value;
    }

    pub fn get_kind(&self) -> ImmKind {
        self.1
    }
    pub fn set_kind(&mut self, kind: ImmKind) {
        self.1 = kind;
    }
}

impl IMirSubOperand for Imm32 {
    type RealRepresents = Imm32;

    fn new_empty() -> Self {
        Self(0, ImmKind::Full)
    }

    fn from_mir(mir: MirOperand) -> Self {
        if let MirOperand::Imm32(imm) = mir {
            imm
        } else {
            panic!("Expected MirOperand::Imm64, found {:?}", mir);
        }
    }
    fn into_mir(self) -> MirOperand {
        MirOperand::Imm32(self)
    }

    fn from_real(real: Self) -> Self {
        real
    }
    fn into_real(self) -> Self {
        self
    }

    fn insert_to_real(self, real: Self) -> Self {
        if real.1.check_u32(self.0) {
            Self(self.0, real.1)
        } else {
            panic!(
                "Invalid immediate value: {} for flags: {:?}",
                self.0, real.1
            );
        }
    }

    fn fmt_asm(&self, formatter: &mut crate::mir::fmt::FormatContext<'_>) -> std::fmt::Result {
        let is_fp = formatter.operand_context.is_fp;
        match self.get_kind() {
            ImmKind::Full => {
                if is_fp {
                    write!(formatter.writer, "#{:e}", f32::from_bits(self.0))
                } else {
                    write!(formatter.writer, "#{:#X}", self.0)
                }
            }
            ImmKind::Calc => write!(formatter.writer, "#0x{:#X}", self.0),
            ImmKind::Logic => write!(formatter.writer, "#0x{:#X}", self.0),
            ImmKind::Load => write!(formatter.writer, "#0x{:#X}", self.0),
            ImmKind::CCmp => write!(formatter.writer, "#0x{:#X}", self.0),
            ImmKind::Mov => write!(formatter.writer, "#0x{:#X}", self.0),
            ImmKind::FMov => write!(formatter.writer, "#{:e}", f32::from_bits(self.0)),
            ImmKind::SMax => write!(formatter.writer, "#{:#X}", self.0 as i32 as i64),
            ImmKind::UMax => write!(formatter.writer, "#{:#X}", self.0 as u8 as u64),
            ImmKind::Shift => write!(formatter.writer, "#{:#X}", self.0),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImmCalc(pub u32);

impl ImmCalc {
    pub fn new(value: u32) -> Self {
        if imm_traits::is_calc_imm(value as u64) {
            Self(value)
        } else {
            panic!("Invalid immediate value: {} for Calc kind", value);
        }
    }
}

impl IMirSubOperand for ImmCalc {
    type RealRepresents = Imm64;

    fn new_empty() -> Self {
        Self(0)
    }

    fn from_mir(mir: MirOperand) -> Self {
        let (value, _) = match mir {
            MirOperand::Imm64(Imm64(value, flag)) => (
                value,
                match flag {
                    ImmKind::Calc => ImmKind::Calc,
                    _ => panic!("Expected Imm64 with Calc kind, found {:?}", flag),
                },
            ),
            MirOperand::Imm32(Imm32(value, flag)) => (
                value as u64,
                match flag {
                    ImmKind::Calc => ImmKind::Calc,
                    _ => panic!("Expected Imm32 with Calc kind, found {:?}", flag),
                },
            ),
            _ => panic!(
                "Expected MirOperand::Imm64 or MirOperand::Imm32, found {:?}",
                mir
            ),
        };
        Self(value as u32)
    }

    fn into_mir(self) -> MirOperand {
        MirOperand::Imm64(Imm64(self.0 as u64, ImmKind::Calc))
    }

    fn from_real(real: Imm64) -> Self {
        if real.get_kind() == ImmKind::Calc {
            Self(real.get_value() as u32)
        } else {
            panic!("Expected Imm64 with Calc kind, found {:?}", real.get_kind());
        }
    }

    fn into_real(self) -> Imm64 {
        Imm64(self.0 as u64, ImmKind::Calc)
    }

    fn insert_to_real(self, real: Imm64) -> Imm64 {
        if real.get_kind() == ImmKind::Calc {
            Imm64(self.0 as u64, ImmKind::Calc)
        } else {
            panic!(
                "Invalid immediate value: {} for flags: {:?}",
                self.0,
                real.get_kind()
            );
        }
    }

    fn fmt_asm(&self, formatter: &mut crate::mir::fmt::FormatContext<'_>) -> std::fmt::Result {
        write!(formatter, "#0x{:X}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImmLogic(pub u64);

impl ImmLogic {
    pub fn new(value: u64) -> Self {
        if imm_traits::is_logical_imm64(value) {
            Self(value)
        } else {
            panic!("Invalid immediate value: {} for Logic kind", value);
        }
    }
}

impl IMirSubOperand for ImmLogic {
    type RealRepresents = Imm64;

    fn new_empty() -> Self {
        Self(0)
    }

    fn from_mir(mir: MirOperand) -> Self {
        let (value, _) = match mir {
            MirOperand::Imm64(Imm64(value, flag)) => (
                value,
                match flag {
                    ImmKind::Logic => ImmKind::Logic,
                    _ => panic!("Expected Imm64 with Logic kind, found {:?}", flag),
                },
            ),
            MirOperand::Imm32(Imm32(value, flag)) => (
                value as u64,
                match flag {
                    ImmKind::Logic => ImmKind::Logic,
                    _ => panic!("Expected Imm32 with Logic kind, found {:?}", flag),
                },
            ),
            _ => panic!(
                "Expected MirOperand::Imm64 or MirOperand::Imm32, found {:?}",
                mir
            ),
        };
        Self(value)
    }

    fn into_mir(self) -> MirOperand {
        MirOperand::Imm64(Imm64(self.0, ImmKind::Logic))
    }

    fn from_real(real: Imm64) -> Self {
        if real.get_kind() == ImmKind::Logic {
            Self(real.get_value())
        } else {
            panic!(
                "Expected Imm64 with Logic kind, found {:?}",
                real.get_kind()
            );
        }
    }

    fn into_real(self) -> Imm64 {
        Imm64(self.0, ImmKind::Logic)
    }

    fn insert_to_real(self, real: Imm64) -> Imm64 {
        if real.get_kind() == ImmKind::Logic {
            Imm64(self.0, ImmKind::Logic)
        } else {
            panic!(
                "Invalid immediate value: {} for flags: {:?}",
                self.0,
                real.get_kind()
            );
        }
    }

    fn fmt_asm(&self, formatter: &mut crate::mir::fmt::FormatContext<'_>) -> std::fmt::Result {
        write!(formatter.writer, "#0x{:X}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImmSMax(pub i64);

impl ImmSMax {
    pub fn new(value: i64) -> Self {
        if imm_traits::is_smax_imm(value) {
            Self(value)
        } else {
            panic!("Invalid immediate value: {} for SMax kind", value);
        }
    }
}

impl IMirSubOperand for ImmSMax {
    type RealRepresents = Imm64;

    fn new_empty() -> Self {
        Self(0)
    }

    fn from_mir(mir: MirOperand) -> Self {
        let (value, _) = match mir {
            MirOperand::Imm64(Imm64(value, flag)) => (
                value as i64,
                match flag {
                    ImmKind::SMax => ImmKind::SMax,
                    _ => panic!("Expected Imm64 with SMax kind, found {:?}", flag),
                },
            ),
            MirOperand::Imm32(Imm32(value, flag)) => (
                value as i32 as i64,
                match flag {
                    ImmKind::SMax => ImmKind::SMax,
                    _ => panic!("Expected Imm32 with SMax kind, found {:?}", flag),
                },
            ),
            _ => panic!(
                "Expected MirOperand::Imm64 or MirOperand::Imm32, found {:?}",
                mir
            ),
        };
        Self(value)
    }

    fn into_mir(self) -> MirOperand {
        MirOperand::Imm64(Imm64(self.0 as u64, ImmKind::SMax))
    }

    fn from_real(real: Imm64) -> Self {
        if real.get_kind() == ImmKind::SMax {
            Self(real.get_value() as i64)
        } else {
            panic!("Expected Imm64 with SMax kind, found {:?}", real.get_kind());
        }
    }

    fn into_real(self) -> Imm64 {
        Imm64(self.0 as u64, ImmKind::SMax)
    }

    fn insert_to_real(self, real: Imm64) -> Imm64 {
        let Self(imm_value) = self;
        if real.get_kind() == ImmKind::SMax {
            Imm64(imm_value as u64, ImmKind::SMax)
        } else {
            panic!(
                "Invalid immediate value: {} for flags: {:?}",
                imm_value,
                real.get_kind()
            );
        }
    }

    fn fmt_asm(&self, formatter: &mut crate::mir::fmt::FormatContext<'_>) -> std::fmt::Result {
        write!(formatter.writer, "#{:#X}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImmUMax(pub u64);

impl ImmUMax {
    pub fn new(value: u64) -> Self {
        if imm_traits::is_umax_imm(value) {
            Self(value)
        } else {
            panic!("Invalid immediate value: {} for UMax kind", value);
        }
    }
}

impl IMirSubOperand for ImmUMax {
    type RealRepresents = Imm64;

    fn new_empty() -> Self {
        Self(0)
    }

    fn from_mir(mir: MirOperand) -> Self {
        let (value, _) = match mir {
            MirOperand::Imm64(Imm64(value, flag)) => (
                value as u64,
                match flag {
                    ImmKind::UMax => ImmKind::UMax,
                    _ => panic!("Expected Imm64 with UMax kind, found {:?}", flag),
                },
            ),
            MirOperand::Imm32(Imm32(value, flag)) => (
                value as u32 as u64,
                match flag {
                    ImmKind::UMax => ImmKind::UMax,
                    _ => panic!("Expected Imm32 with UMax kind, found {:?}", flag),
                },
            ),
            _ => panic!(
                "Expected MirOperand::Imm64 or MirOperand::Imm32, found {:?}",
                mir
            ),
        };
        Self(value)
    }

    fn into_mir(self) -> MirOperand {
        MirOperand::Imm64(Imm64(self.0, ImmKind::UMax))
    }

    fn from_real(real: Imm64) -> Self {
        if real.get_kind() == ImmKind::UMax {
            Self(real.get_value())
        } else {
            panic!("Expected Imm64 with UMax kind, found {:?}", real.get_kind());
        }
    }

    fn into_real(self) -> Imm64 {
        Imm64(self.0, ImmKind::UMax)
    }

    fn insert_to_real(self, real: Imm64) -> Imm64 {
        if real.get_kind() == ImmKind::UMax {
            Imm64(self.0, ImmKind::UMax)
        } else {
            panic!(
                "Invalid immediate value: {} for flags: {:?}",
                self.0,
                real.get_kind()
            );
        }
    }

    fn fmt_asm(&self, formatter: &mut crate::mir::fmt::FormatContext<'_>) -> std::fmt::Result {
        write!(formatter.writer, "#{:#X}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImmShift(pub u64);

impl ImmShift {
    pub fn new(value: u64) -> Self {
        if imm_traits::is_shift_imm(value) {
            Self(value)
        } else {
            panic!("Invalid immediate value: {} for Shift kind", value);
        }
    }
}

impl IMirSubOperand for ImmShift {
    type RealRepresents = Imm64;

    fn new_empty() -> Self {
        Self(0)
    }

    fn from_mir(mir: MirOperand) -> Self {
        let (value, _) = match mir {
            MirOperand::Imm64(Imm64(value, flag)) => (
                value,
                match flag {
                    ImmKind::Shift => ImmKind::Shift,
                    _ => panic!("Expected Imm64 with Shift kind, found {:?}", flag),
                },
            ),
            MirOperand::Imm32(Imm32(value, flag)) => (
                value as u64,
                match flag {
                    ImmKind::Shift => ImmKind::Shift,
                    _ => panic!("Expected Imm32 with Shift kind, found {:?}", flag),
                },
            ),
            _ => panic!(
                "Expected MirOperand::Imm64 or MirOperand::Imm32, found {:?}",
                mir
            ),
        };
        Self(value)
    }

    fn into_mir(self) -> MirOperand {
        MirOperand::Imm64(Imm64(self.0, ImmKind::Shift))
    }

    fn from_real(real: Imm64) -> Self {
        if real.get_kind() == ImmKind::Shift {
            Self(real.get_value())
        } else {
            panic!(
                "Expected Imm64 with Shift kind, found {:?}",
                real.get_kind()
            );
        }
    }

    fn into_real(self) -> Imm64 {
        Imm64(self.0, ImmKind::Shift)
    }

    fn insert_to_real(self, real: Imm64) -> Imm64 {
        if real.get_kind() == ImmKind::Shift {
            Imm64(self.0, ImmKind::Shift)
        } else {
            panic!(
                "Invalid immediate value: {} for flags: {:?}",
                self.0,
                real.get_kind()
            );
        }
    }

    fn fmt_asm(&self, formatter: &mut crate::mir::fmt::FormatContext<'_>) -> std::fmt::Result {
        write!(formatter.writer, "#{:#X}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImmLoad32(pub i32);

impl ImmLoad32 {
    pub fn new(value: i32) -> Self {
        if imm_traits::is_load32_imm(value as i64) {
            Self(value)
        } else {
            panic!("Invalid immediate value: {} for Load32 kind", value);
        }
    }
}

impl IMirSubOperand for ImmLoad32 {
    type RealRepresents = Imm64;

    fn new_empty() -> Self {
        Self(0)
    }

    fn from_mir(mir: MirOperand) -> Self {
        let (value, flag) = match mir {
            MirOperand::Imm64(Imm64(value, flag)) => (value as i32, flag),
            MirOperand::Imm32(Imm32(value, flag)) => (value as i32, flag),
            _ => panic!("Expected Imm64 or Imm32, found {mir:?}"),
        };
        if flag != ImmKind::Load {
            panic!("Expected Imm64 or Imm32 with Load kind, found {:?}", flag);
        }
        if imm_traits::is_load32_imm(value as i64) {
            Self(value)
        } else {
            panic!("Invalid immediate value: {} for Load kind", value);
        }
    }

    fn into_mir(self) -> MirOperand {
        MirOperand::Imm64(Imm64(self.0 as u64, ImmKind::Load))
    }

    fn from_real(real: Imm64) -> Self {
        if real.get_kind() == ImmKind::Load {
            Self(real.get_value() as i32)
        } else {
            panic!("Expected Imm64 with Load kind, found {:?}", real.get_kind());
        }
    }

    fn into_real(self) -> Imm64 {
        Imm64(self.0 as u64, ImmKind::Load)
    }

    fn insert_to_real(self, real: Imm64) -> Imm64 {
        if real.get_kind() == ImmKind::Load {
            Imm64(self.0 as u64, ImmKind::Load)
        } else {
            panic!(
                "Invalid immediate value: {} for flags: {:?}",
                self.0,
                real.get_kind()
            );
        }
    }

    fn fmt_asm(&self, formatter: &mut crate::mir::fmt::FormatContext<'_>) -> std::fmt::Result {
        write!(formatter.writer, "#{:#X}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImmLoad64(pub i64);

impl ImmLoad64 {
    pub fn new(value: i64) -> Self {
        if imm_traits::is_load64_imm(value) {
            Self(value)
        } else {
            panic!("Invalid immediate value: {} for Load64 kind", value);
        }
    }
}

impl IMirSubOperand for ImmLoad64 {
    type RealRepresents = Imm64;

    fn new_empty() -> Self {
        Self(0)
    }

    fn from_mir(mir: MirOperand) -> Self {
        let (value, flag) = match mir {
            MirOperand::Imm64(Imm64(value, flag)) => (value as i64, flag),
            MirOperand::Imm32(Imm32(value, flag)) => (value as i64, flag),
            _ => panic!("Expected Imm64 or Imm32, found {mir:?}"),
        };
        if flag != ImmKind::Load {
            panic!("Expected Imm64 or Imm32 with Load kind, found {:?}", flag);
        }
        if imm_traits::is_load64_imm(value) {
            Self(value)
        } else {
            panic!("Invalid immediate value: {} for Load kind", value);
        }
    }

    fn into_mir(self) -> MirOperand {
        MirOperand::Imm64(Imm64(self.0 as u64, ImmKind::Load))
    }

    fn from_real(real: Imm64) -> Self {
        if real.get_kind() == ImmKind::Load {
            Self(real.get_value() as i64)
        } else {
            panic!("Expected Imm64 with Load kind, found {:?}", real.get_kind());
        }
    }

    fn into_real(self) -> Imm64 {
        Imm64(self.0 as u64, ImmKind::Load)
    }

    fn insert_to_real(self, real: Imm64) -> Imm64 {
        if real.get_kind() == ImmKind::Load {
            Imm64(self.0 as u64, ImmKind::Load)
        } else {
            panic!(
                "Invalid immediate value: {} for flags: {:?}",
                self.0,
                real.get_kind()
            );
        }
    }

    fn fmt_asm(&self, formatter: &mut crate::mir::fmt::FormatContext<'_>) -> std::fmt::Result {
        write!(formatter.writer, "#{:#X}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImmCCmp(pub u32);

impl ImmCCmp {
    pub fn new(value: u32) -> Self {
        if imm_traits::is_condcmp_imm(value as u64) {
            Self(value)
        } else {
            panic!("Invalid immediate value: {} for CCmp kind", value);
        }
    }
}

impl IMirSubOperand for ImmCCmp {
    type RealRepresents = Imm64;

    fn new_empty() -> Self {
        Self(0)
    }

    fn from_mir(mir: MirOperand) -> Self {
        let (value, flag) = match mir {
            MirOperand::Imm64(Imm64(value, flag)) => (value as u32, flag),
            MirOperand::Imm32(Imm32(value, flag)) => (value, flag),
            _ => panic!("Expected Imm64 or Imm32, found {mir:?}"),
        };
        if flag != ImmKind::CCmp {
            panic!("Expected Imm64 or Imm32 with CCmp kind, found {:?}", flag);
        }
        if imm_traits::is_condcmp_imm(value as u64) {
            Self(value)
        } else {
            panic!("Invalid immediate value: {} for CCmp kind", value);
        }
    }

    fn into_mir(self) -> MirOperand {
        MirOperand::Imm64(Imm64(self.0 as u64, ImmKind::CCmp))
    }

    fn from_real(real: Imm64) -> Self {
        if real.get_kind() == ImmKind::CCmp {
            Self(real.get_value() as u32)
        } else {
            panic!("Expected Imm64 with CCmp kind, found {:?}", real.get_kind());
        }
    }

    fn into_real(self) -> Imm64 {
        Imm64(self.0 as u64, ImmKind::CCmp)
    }

    fn insert_to_real(self, real: Imm64) -> Imm64 {
        if real.get_kind() == ImmKind::CCmp {
            Imm64(self.0 as u64, ImmKind::CCmp)
        } else {
            panic!(
                "Invalid immediate value: {} for flags: {:?}",
                self.0,
                real.get_kind()
            );
        }
    }

    fn fmt_asm(&self, formatter: &mut crate::mir::fmt::FormatContext<'_>) -> std::fmt::Result {
        write!(formatter.writer, "#{:#X}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImmMov(pub u32);

impl ImmMov {
    pub fn new(value: u32) -> Self {
        if imm_traits::is_mov_imm(value as u64) {
            Self(value)
        } else {
            panic!("Invalid immediate value: {} for Mov kind", value);
        }
    }
}

impl IMirSubOperand for ImmMov {
    type RealRepresents = Imm64;

    fn new_empty() -> Self {
        Self(0)
    }

    fn from_mir(mir: MirOperand) -> Self {
        let (value, flag) = match mir {
            MirOperand::Imm64(Imm64(value, flag)) => (value as u32, flag),
            MirOperand::Imm32(Imm32(value, flag)) => (value, flag),
            _ => panic!("Expected Imm64 or Imm32, found {mir:?}"),
        };
        if flag != ImmKind::Mov {
            panic!("Expected Imm64 or Imm32 with Mov kind, found {:?}", flag);
        }
        if imm_traits::is_mov_imm(value as u64) {
            Self(value)
        } else {
            panic!("Invalid immediate value: {} for Mov kind", value);
        }
    }

    fn into_mir(self) -> MirOperand {
        MirOperand::Imm64(Imm64(self.0 as u64, ImmKind::Mov))
    }

    fn from_real(real: Imm64) -> Self {
        if real.get_kind() == ImmKind::Mov {
            Self(real.get_value() as u32)
        } else {
            panic!("Expected Imm64 with Mov kind, found {:?}", real.get_kind());
        }
    }

    fn into_real(self) -> Imm64 {
        Imm64(self.0 as u64, ImmKind::Mov)
    }

    fn insert_to_real(self, real: Imm64) -> Imm64 {
        if real.get_kind() == ImmKind::Mov {
            Imm64(self.0 as u64, ImmKind::Mov)
        } else {
            panic!(
                "Invalid immediate value: {} for flags: {:?}",
                self.0,
                real.get_kind()
            );
        }
    }

    fn fmt_asm(&self, formatter: &mut crate::mir::fmt::FormatContext<'_>) -> std::fmt::Result {
        write!(formatter.writer, "#{:#X}", self.0)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ImmFMov32(pub f32);

impl ImmFMov32 {
    pub fn new(value: f32) -> Self {
        if imm_traits::try_cast_f32_to_aarch8(value).is_some() {
            Self(value)
        } else {
            panic!("Invalid immediate value: {} for FMov32 kind", value);
        }
    }
}

impl PartialEq for ImmFMov32 {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}

impl Eq for ImmFMov32 {}

impl std::hash::Hash for ImmFMov32 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

impl IMirSubOperand for ImmFMov32 {
    type RealRepresents = Imm64;

    fn new_empty() -> Self {
        Self(0.0)
    }

    fn from_mir(mir: MirOperand) -> Self {
        if let MirOperand::Imm32(Imm32(value, flag)) = mir {
            if flag == ImmKind::FMov {
                Self(f32::from_bits(value))
            } else {
                panic!("Expected Imm32 with FMov kind, found {:?}", flag);
            }
        } else {
            panic!("Expected MirOperand::Imm32, found {:?}", mir);
        }
    }

    fn into_mir(self) -> MirOperand {
        MirOperand::Imm32(Imm32(self.0.to_bits(), ImmKind::FMov))
    }

    fn from_real(real: Imm64) -> Self {
        if real.get_kind() == ImmKind::FMov {
            Self(f32::from_bits(real.get_value() as u32))
        } else {
            panic!("Expected Imm64 with FMov kind, found {:?}", real.get_kind());
        }
    }

    fn into_real(self) -> Imm64 {
        Imm64(self.0.to_bits() as u64, ImmKind::FMov)
    }

    fn insert_to_real(self, real: Imm64) -> Imm64 {
        if real.get_kind() == ImmKind::FMov {
            Imm64(self.0.to_bits() as u64, ImmKind::FMov)
        } else {
            panic!(
                "Invalid immediate value: {} for flags: {:?}",
                self.0,
                real.get_kind()
            );
        }
    }

    fn fmt_asm(&self, formatter: &mut crate::mir::fmt::FormatContext<'_>) -> std::fmt::Result {
        write!(formatter.writer, "#{:e}", self.0)
    }
}

#[derive(Debug, Clone, Copy)]
pub struct ImmFMov64(pub f64);

impl ImmFMov64 {
    pub fn new(value: f64) -> Self {
        if imm_traits::try_cast_f64_to_aarch8(value).is_some() {
            Self(value)
        } else {
            panic!("Invalid immediate value: {} for FMov64 kind", value);
        }
    }
}

impl PartialEq for ImmFMov64 {
    fn eq(&self, other: &Self) -> bool {
        self.0.to_bits() == other.0.to_bits()
    }
}

impl Eq for ImmFMov64 {}

impl std::hash::Hash for ImmFMov64 {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.0.to_bits().hash(state);
    }
}

impl IMirSubOperand for ImmFMov64 {
    type RealRepresents = Imm64;

    fn new_empty() -> Self {
        Self(0.0)
    }

    fn from_mir(mir: MirOperand) -> Self {
        if let MirOperand::Imm64(Imm64(value, flag)) = mir {
            if flag == ImmKind::FMov {
                Self(f64::from_bits(value))
            } else {
                panic!("Expected Imm64 with FMov kind, found {:?}", flag);
            }
        } else {
            panic!("Expected MirOperand::Imm64, found {:?}", mir);
        }
    }

    fn into_mir(self) -> MirOperand {
        MirOperand::Imm64(Imm64(self.0.to_bits(), ImmKind::FMov))
    }

    fn from_real(real: Imm64) -> Self {
        if real.get_kind() == ImmKind::FMov {
            Self(f64::from_bits(real.get_value()))
        } else {
            panic!("Expected Imm64 with FMov kind, found {:?}", real.get_kind());
        }
    }

    fn into_real(self) -> Imm64 {
        Imm64(self.0.to_bits(), ImmKind::FMov)
    }

    fn insert_to_real(self, real: Imm64) -> Imm64 {
        if real.get_kind() == ImmKind::FMov {
            Imm64(self.0.to_bits(), ImmKind::FMov)
        } else {
            panic!(
                "Invalid immediate value: {} for flags: {:?}",
                self.0,
                real.get_kind()
            );
        }
    }

    fn fmt_asm(&self, formatter: &mut crate::mir::fmt::FormatContext<'_>) -> std::fmt::Result {
        write!(formatter.writer, "#{:e}", self.0)
    }
}

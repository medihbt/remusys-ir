use crate::mir::{
    fmt::FuncFormatContext,
    operand::{
        imm_traits::{self, fp8aarch_to_fp32, fp8aarch_to_fp64}, IMirSubOperand, MirOperand
    },
};
use std::fmt::Debug;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImmKind {
    Full,
    /// 计算相关的立即数, 规则是: 要么只有 [0:12] 位有值, 要么只有 [12:24] 位有值.
    Calc,
    /// 逻辑相关的立即数, 规则是: 立即数应该是一个循环节的位模式.
    Logic,
    /// Load & Store 在偏移模式下的立即数模式. 规则是:
    ///
    /// * 32 位变体: 4 的倍数, 范围 `[0, 16380]`
    /// * 64 位变体: 8 的倍数, 范围 `[0, 32760]`
    LSP,
    /// 条件比较相关的立即数, 规则是: 只有 [0:5] 位有值, 无符号.
    CCmp,
    /// 移动指令相关的立即数.
    Mov,
    /// 常量加载相关的立即数, 规则是: 16 位整数, 通过移位和无符号扩展得到原数.
    MovZNK,
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
            ImmKind::LSP => imm_traits::is_lsp32_imm(imm),
            ImmKind::CCmp => imm_traits::is_condcmp_imm(imm as u64),
            ImmKind::Mov => imm_traits::is_mov_imm(imm as u64),
            ImmKind::MovZNK => ImmMovZNK::try_from_u64(imm as u64).is_some(),
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
            ImmKind::LSP => imm_traits::is_lsp64_imm(imm),
            ImmKind::CCmp => imm_traits::is_condcmp_imm(imm),
            ImmKind::Mov => imm_traits::is_mov_imm(imm),
            ImmKind::MovZNK => ImmMovZNK::try_from_u64(imm).is_some(),
            ImmKind::FMov => imm_traits::try_cast_f64_to_aarch8(f64::from_bits(imm)).is_some(),
            ImmKind::SMax => imm_traits::is_smax_imm(imm as i64),
            ImmKind::UMax => imm_traits::is_umax_imm(imm),
            ImmKind::Shift => imm_traits::is_shift_imm(imm),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Imm64(pub u64, pub ImmKind);

impl Debug for Imm64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let &Self(value, kind) = self;
        write!(f, "Imm64({value:#x}:{kind:?})")
    }
}

impl Imm64 {
    pub fn new(value: u64, flags: ImmKind) -> Self {
        Self(value, flags)
    }
    pub fn full(value: u64) -> Self {
        Self(value, ImmKind::Full)
    }
    pub fn from_fp_bits(value: f64) -> Self {
        Self(value.to_bits(), ImmKind::Full)
    }
    pub fn to_fp_bits(self) -> f64 {
        f64::from_bits(self.0)
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

    fn try_from_real(real: Self) -> Option<Self> {
        Some(real)
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

    fn fmt_asm(&self, formatter: &mut crate::mir::fmt::FuncFormatContext<'_>) -> std::fmt::Result {
        let is_fp = formatter.operand_context.is_fp;
        let &Self(value, kind) = self;
        match kind {
            ImmKind::Full => {
                if is_fp {
                    write!(formatter.writer, "{:e}", f64::from_bits(value))
                } else {
                    write!(formatter.writer, "{:#X}", value)
                }
            }
            ImmKind::Calc => ImmCalc::new(value as u32).fmt_asm(formatter),
            ImmKind::Logic => ImmLogic::new(value).fmt_asm(formatter),
            ImmKind::LSP => ImmLSP64::new(value).fmt_asm(formatter),
            ImmKind::CCmp => ImmCCmp::new(value as u32).fmt_asm(formatter),
            ImmKind::Mov => ImmMov::new(value).fmt_asm(formatter),
            ImmKind::MovZNK => ImmMovZNK::from_raw(value as u32).fmt_asm(formatter),
            ImmKind::FMov => ImmFMov64::from_real(*self).fmt_asm(formatter),
            ImmKind::SMax => ImmSMax::new(value as i64).fmt_asm(formatter),
            ImmKind::UMax => ImmUMax::new(value).fmt_asm(formatter),
            ImmKind::Shift => ImmShift::new(value).fmt_asm(formatter),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct Imm32(pub u32, pub ImmKind);

impl Debug for Imm32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let &Self(value, kind) = self;
        write!(f, "Imm32({value:#x}:{kind:?})")
    }
}

impl Imm32 {
    pub fn new(value: u32, flags: ImmKind) -> Self {
        Self(value, flags)
    }
    pub fn full(value: u32) -> Self {
        Self(value, ImmKind::Full)
    }
    pub fn from_fp_bits(value: f32) -> Self {
        Self(value.to_bits(), ImmKind::Full)
    }
    pub fn to_fp_bits(self) -> f32 {
        f32::from_bits(self.0)
    }

    pub fn zext_to_imm64(self) -> Imm64 {
        Imm64(self.0 as u64, ImmKind::Full)
    }
    pub fn sext_to_imm64(self) -> Imm64 {
        Imm64(self.0 as i32 as u64, ImmKind::Full)
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

    fn try_from_real(real: Self) -> Option<Self> {
        Some(real)
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

    fn fmt_asm(&self, formatter: &mut crate::mir::fmt::FuncFormatContext<'_>) -> std::fmt::Result {
        let is_fp = formatter.operand_context.is_fp;
        let &Self(imm, kind) = self;
        match kind {
            ImmKind::Full => {
                if is_fp {
                    write!(formatter.writer, "{:e}", f32::from_bits(imm))
                } else {
                    write!(formatter.writer, "{:#X}", imm)
                }
            }
            ImmKind::Calc => ImmCalc::new(imm as u32).fmt_asm(formatter),
            ImmKind::Logic => ImmLogic::new(imm as u64).fmt_asm(formatter),
            ImmKind::LSP => ImmLSP32::new(imm).fmt_asm(formatter),
            ImmKind::CCmp => ImmCCmp::new(imm as u32).fmt_asm(formatter),
            ImmKind::Mov => ImmMov::new(imm as u64).fmt_asm(formatter),
            ImmKind::MovZNK => ImmMovZNK::from_raw(imm).fmt_asm(formatter),
            ImmKind::FMov => ImmFMov32::from_real(*self).fmt_asm(formatter),
            ImmKind::SMax => ImmSMax::new(imm as i32 as i64).fmt_asm(formatter),
            ImmKind::UMax => ImmUMax::new(imm as u64).fmt_asm(formatter),
            ImmKind::Shift => ImmShift::new(imm as u64).fmt_asm(formatter),
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImmCalc(pub u32);

impl Debug for ImmCalc {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(value) = self;
        write!(f, "Imm:Calc({value:#x})")
    }
}

impl ImmCalc {
    pub fn new(value: u32) -> Self {
        if imm_traits::is_calc_imm(value as u64) {
            Self(value)
        } else {
            panic!("Invalid immediate value: {} for Calc kind", value);
        }
    }
    pub fn try_new(value: impl Into<u64>) -> Option<Self> {
        let value = value.into();
        if imm_traits::is_calc_imm(value) { Some(Self(value as u32)) } else { None }
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

    fn try_from_real(real: Imm64) -> Option<Self> {
        if real.get_kind() == ImmKind::Calc { Some(Self(real.get_value() as u32)) } else { None }
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

    fn fmt_asm(&self, formatter: &mut crate::mir::fmt::FuncFormatContext<'_>) -> std::fmt::Result {
        let &Self(imm) = self;
        if imm < 4096 {
            write!(formatter.writer, "#{imm:#x}")
        } else {
            let imm = imm >> 12;
            write!(formatter.writer, "#{imm:#x}, LSL #12")
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImmLogic(pub u64);

impl Debug for ImmLogic {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(value) = self;
        write!(f, "Imm:Logic({value:#x})")
    }
}

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

    fn try_from_real(real: Imm64) -> Option<Self> {
        if real.get_kind() == ImmKind::Logic { Some(Self(real.get_value())) } else { None }
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

    fn fmt_asm(&self, formatter: &mut crate::mir::fmt::FuncFormatContext<'_>) -> std::fmt::Result {
        write!(formatter.writer, "#0x{:X}", self.0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImmSMax(pub i64);

impl Debug for ImmSMax {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(value) = self;
        write!(f, "Imm:SMax({value:#x})")
    }
}

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

    fn try_from_real(real: Imm64) -> Option<Self> {
        if real.get_kind() == ImmKind::SMax { Some(Self(real.get_value() as i64)) } else { None }
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

    fn fmt_asm(&self, formatter: &mut crate::mir::fmt::FuncFormatContext<'_>) -> std::fmt::Result {
        write!(formatter.writer, "#{:#X}", self.0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImmUMax(pub u64);

impl Debug for ImmUMax {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(value) = self;
        write!(f, "Imm:UMax({value:#x})")
    }
}

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

    fn try_from_real(real: Imm64) -> Option<Self> {
        if real.get_kind() == ImmKind::UMax { Some(Self(real.get_value())) } else { None }
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

    fn fmt_asm(&self, formatter: &mut crate::mir::fmt::FuncFormatContext<'_>) -> std::fmt::Result {
        write!(formatter.writer, "#{:#X}", self.0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImmShift(pub u64);

impl Debug for ImmShift {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(value) = self;
        write!(f, "Imm:Shift({value:#x})")
    }
}

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

    fn try_from_real(real: Imm64) -> Option<Self> {
        if real.get_kind() == ImmKind::Shift { Some(Self(real.get_value())) } else { None }
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

    fn fmt_asm(&self, formatter: &mut crate::mir::fmt::FuncFormatContext<'_>) -> std::fmt::Result {
        write!(formatter.writer, "#{:#X}", self.0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImmLSP32(pub u32);

impl Debug for ImmLSP32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(value) = self;
        write!(f, "Imm:Load32({value:#x})")
    }
}

impl ImmLSP32 {
    pub fn new(value: u32) -> Self {
        if imm_traits::is_lsp32_imm(value) {
            Self(value)
        } else {
            panic!("Invalid immediate value: {} for Load32 kind", value);
        }
    }
    pub fn try_new(value: u32) -> Option<Self> {
        if imm_traits::is_lsp32_imm(value) { Some(Self(value)) } else { None }
    }
}

impl IMirSubOperand for ImmLSP32 {
    type RealRepresents = Imm32;

    fn new_empty() -> Self {
        Self(0)
    }

    fn from_mir(mir: MirOperand) -> Self {
        let (value, flag) = match mir {
            MirOperand::Imm64(Imm64(value, flag)) => {
                if value < u32::MAX as u64 {
                    (value as u32, flag)
                } else {
                    panic!("Expected Imm64 with Load kind, found value too large: {value}")
                }
            }
            MirOperand::Imm32(Imm32(value, flag)) => (value as u32, flag),
            _ => panic!("Expected Imm64 or Imm32, found {mir:?}"),
        };
        if flag != ImmKind::LSP {
            panic!("Expected Imm64 or Imm32 with Load kind, found {:?}", flag);
        }
        if imm_traits::is_lsp32_imm(value) {
            Self(value)
        } else {
            panic!("Invalid immediate value: {} for Load kind", value);
        }
    }

    fn into_mir(self) -> MirOperand {
        MirOperand::Imm64(Imm64(self.0 as u64, ImmKind::LSP))
    }

    fn try_from_real(real: Imm32) -> Option<Self> {
        if real.get_kind() == ImmKind::LSP { Some(Self(real.get_value())) } else { None }
    }

    fn from_real(real: Imm32) -> Self {
        if real.get_kind() == ImmKind::LSP {
            Self(real.get_value())
        } else {
            panic!("Expected Imm64 with Load kind, found {:?}", real.get_kind());
        }
    }
    fn into_real(self) -> Imm32 {
        Imm32(self.0, ImmKind::LSP)
    }
    fn insert_to_real(self, _: Imm32) -> Imm32 {
        Imm32(self.0, ImmKind::LSP)
    }
    fn fmt_asm(&self, formatter: &mut FuncFormatContext<'_>) -> std::fmt::Result {
        write!(formatter.writer, "#{:#x}", self.0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImmLSP64(pub u64);

impl Debug for ImmLSP64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(value) = self;
        write!(f, "Imm:Load64({value:#x})")
    }
}

impl ImmLSP64 {
    pub fn new(value: u64) -> Self {
        if imm_traits::is_lsp64_imm(value) {
            Self(value)
        } else {
            panic!("Invalid immediate value: {} for Load64 kind", value);
        }
    }
    pub fn try_new(value: u64) -> Option<Self> {
        if imm_traits::is_lsp64_imm(value) { Some(Self(value)) } else { None }
    }
}

impl IMirSubOperand for ImmLSP64 {
    type RealRepresents = Imm64;

    fn new_empty() -> Self {
        Self(0)
    }

    fn from_mir(mir: MirOperand) -> Self {
        let (value, flag) = match mir {
            MirOperand::Imm64(Imm64(value, flag)) => (value, flag),
            MirOperand::Imm32(Imm32(value, flag)) => (value as u64, flag),
            _ => panic!("Expected Imm64 or Imm32, found {mir:?}"),
        };
        if flag != ImmKind::LSP {
            panic!("Expected Imm64 or Imm32 with Load kind, found {:?}", flag);
        }
        if imm_traits::is_lsp64_imm(value) {
            Self(value)
        } else {
            panic!("Invalid immediate value: {} for Load kind", value);
        }
    }

    fn into_mir(self) -> MirOperand {
        MirOperand::Imm64(Imm64(self.0 as u64, ImmKind::LSP))
    }

    fn try_from_real(real: Imm64) -> Option<Self> {
        if real.get_kind() == ImmKind::LSP { Some(Self(real.get_value())) } else { None }
    }

    fn from_real(real: Imm64) -> Self {
        if real.get_kind() == ImmKind::LSP {
            Self(real.get_value())
        } else {
            panic!("Expected Imm64 with Load kind, found {:?}", real.get_kind());
        }
    }

    fn into_real(self) -> Imm64 {
        Imm64(self.0 as u64, ImmKind::LSP)
    }

    fn insert_to_real(self, real: Imm64) -> Imm64 {
        if real.get_kind() == ImmKind::LSP {
            Imm64(self.0 as u64, ImmKind::LSP)
        } else {
            panic!(
                "Invalid immediate value: {} for flags: {:?}",
                self.0,
                real.get_kind()
            );
        }
    }

    fn fmt_asm(&self, formatter: &mut crate::mir::fmt::FuncFormatContext<'_>) -> std::fmt::Result {
        write!(formatter.writer, "#{:#X}", self.0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImmCCmp(pub u32);

impl Debug for ImmCCmp {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(value) = self;
        write!(f, "Imm:CCmp({value:#x})")
    }
}

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

    fn try_from_real(real: Imm64) -> Option<Self> {
        if real.get_kind() == ImmKind::CCmp { Some(Self(real.get_value() as u32)) } else { None }
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

    fn fmt_asm(&self, formatter: &mut crate::mir::fmt::FuncFormatContext<'_>) -> std::fmt::Result {
        write!(formatter.writer, "#{:#X}", self.0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct ImmMov(pub u16);

impl Debug for ImmMov {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(value) = self;
        write!(f, "Imm:Mov({value:#x})")
    }
}

impl ImmMov {
    pub fn new(value: u64) -> Self {
        if imm_traits::is_mov_imm(value) {
            Self(value as u16)
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
            MirOperand::Imm64(Imm64(value, flag)) => (value, flag),
            MirOperand::Imm32(Imm32(value, flag)) => (value as u64, flag),
            _ => panic!("Expected Imm64 or Imm32, found {mir:?}"),
        };
        if flag != ImmKind::Mov {
            panic!("Expected Imm64 or Imm32 with Mov kind, found {:?}", flag);
        }
        if imm_traits::is_mov_imm(value) {
            Self(value as u16)
        } else {
            panic!("Invalid immediate value: {} for Mov kind", value);
        }
    }

    fn into_mir(self) -> MirOperand {
        MirOperand::Imm64(Imm64(self.0 as u64, ImmKind::Mov))
    }

    fn try_from_real(real: Imm64) -> Option<Self> {
        if real.get_kind() == ImmKind::Mov { Some(Self(real.get_value() as u16)) } else { None }
    }

    fn from_real(real: Imm64) -> Self {
        if real.get_kind() == ImmKind::Mov {
            Self(real.get_value() as u16)
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

    fn fmt_asm(&self, formatter: &mut crate::mir::fmt::FuncFormatContext<'_>) -> std::fmt::Result {
        write!(formatter.writer, "#{:#X}", self.0)
    }
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub struct ImmMovZNK(pub u16, pub u8);

impl ImmMovZNK {
    pub const fn try_from_u64(value: u64) -> Option<Self> {
        if imm_traits::is_mov_imm(value) {
            let (v, s) = if imm_traits::is_mov_imm(value >> 16) {
                (value as u16, 16)
            } else if imm_traits::is_mov_imm(value >> 32) {
                (value as u16, 32)
            } else if imm_traits::is_mov_imm(value >> 48) {
                (value as u16, 48)
            } else {
                (value as u16, 0)
            };
            Some(Self(v, s))
        } else {
            None
        }
    }
    pub fn from_u64(value: u64) -> Self {
        let Some(imm) = Self::try_from_u64(value) else {
            panic!("Invalid immediate value: {value} for MovZNK kind");
        };
        imm
    }

    pub fn from_raw<T: Into<u64>>(value: T) -> Self {
        let value: u64 = value.into();
        if imm_traits::is_mov_imm(value) {
            Self::from_u64(value)
        } else {
            panic!("Invalid immediate value: {value} for MovZNK kind");
        }
    }

    pub fn new(value: u16, shift: u8) -> Self {
        // u16 一定是合法的 mov 立即数
        if matches!(shift, 0 | 16 | 32 | 48) {
            Self(value, shift)
        } else {
            panic!("Invalid shift value: {shift} for MovZNK kind");
        }
    }
}

impl Debug for ImmMovZNK {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let &Self(value, shift) = self;
        let real_value = value as u64;
        let real_value = real_value << shift;
        write!(f, "Imm:MovZNK({real_value:#x} = {value:#x} << {shift})")
    }
}

impl IMirSubOperand for ImmMovZNK {
    type RealRepresents = Imm64;

    fn new_empty() -> Self {
        Self(0, 0)
    }

    fn from_mir(mir: MirOperand) -> Self {
        let MirOperand::Imm64(imm) = mir else {
            panic!("Expected MirOperand::Imm64, found {mir:?}");
        };
        Self::from_real(imm)
    }

    fn into_mir(self) -> MirOperand {
        MirOperand::Imm64(self.into_real())
    }

    fn try_from_real(real: Imm64) -> Option<Self> {
        let Imm64(value, _) = real;
        let (v, s) = if imm_traits::is_mov_imm(value) {
            (value as u16, 0)
        } else if imm_traits::is_mov_imm(value >> 16) {
            ((value >> 16) as u16, 16)
        } else if imm_traits::is_mov_imm(value >> 32) {
            ((value >> 32) as u16, 32)
        } else if imm_traits::is_mov_imm(value >> 48) {
            ((value >> 48) as u16, 48)
        } else {
            return None;
        };
        Some(Self(v, s))
    }

    fn from_real(real: Imm64) -> Self {
        let Some(imm) = Self::try_from_real(real) else {
            panic!(
                "Expected Imm64 with MovZNK kind, found {:?}",
                real.get_kind()
            );
        };
        imm
    }

    fn into_real(self) -> Imm64 {
        let Self(value, shift) = self;
        Imm64((value as u64) << shift, ImmKind::MovZNK)
    }

    fn insert_to_real(self, _: Imm64) -> Imm64 {
        self.into_real()
    }

    fn fmt_asm(&self, formatter: &mut FuncFormatContext<'_>) -> std::fmt::Result {
        let &Self(value, shift) = self;
        if shift != 0 {
            write!(formatter.writer, "#{value:#x}, LSL #{shift}")
        } else {
            write!(formatter.writer, "#{value:#x}")
        }
    }
}

#[derive(Clone, Copy)]
pub struct ImmFMov32(pub f32);

impl TryFrom<f32> for ImmFMov32 {
    type Error = &'static str;

    fn try_from(value: f32) -> Result<Self, Self::Error> {
        if imm_traits::try_cast_f32_to_aarch8(value).is_some() {
            Ok(Self(value))
        } else {
            Err("Invalid immediate value for FMov32 kind")
        }
    }
}

impl Debug for ImmFMov32 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(value) = self;
        write!(f, "Imm:FMov32({value})")
    }
}

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
    type RealRepresents = Imm32;

    fn new_empty() -> Self {
        Self(0.0)
    }

    fn from_mir(mir: MirOperand) -> Self {
        let MirOperand::Imm32(imm) = mir else {
            panic!("Expected MirOperand::Imm32, found {mir:?}");
        };
        Self::from_real(imm)
    }

    fn into_mir(self) -> MirOperand {
        self.into_real().into_mir()
    }

    fn try_from_real(real: Imm32) -> Option<Self> {
        if real.get_kind() != ImmKind::FMov {
            return None;
        }
        let imm = real.get_value();
        if imm & 0xFFu32 != imm {
            return None;
        }
        Some(Self(fp8aarch_to_fp32(imm as u8)))
    }

    fn from_real(real: Imm32) -> Self {
        if real.get_kind() != ImmKind::FMov {
            panic!("Expected Imm32 with FMov kind, found {:?}", real.get_kind());
        }
        let value = real.get_value();
        if value & 0xFFu32 != value {
            panic!("Value format error: Found {value:#x}");
        }
        Self(fp8aarch_to_fp32(value as u8))
    }

    fn into_real(self) -> Imm32 {
        let Self(value) = self;
        let Some(x) = imm_traits::try_cast_f32_to_aarch8(value) else {
            panic!("Value error: {value} cannot cast to FMovImm32");
        };
        Imm32(x as u32, ImmKind::FMov)
    }

    fn insert_to_real(self, real: Imm32) -> Imm32 {
        if real.get_kind() != ImmKind::FMov {
            panic!("Expected Imm32 with FMov kind, but found {real:?}");
        }
        self.into_real()
    }

    fn fmt_asm(&self, formatter: &mut FuncFormatContext<'_>) -> std::fmt::Result {
        write!(formatter.writer, "#{:.20e}", self.0)
    }
}

#[derive(Clone, Copy)]
pub struct ImmFMov64(pub f64);

impl TryFrom<f64> for ImmFMov64 {
    type Error = &'static str;

    fn try_from(value: f64) -> Result<Self, Self::Error> {
        if imm_traits::try_cast_f64_to_aarch8(value).is_some() {
            Ok(Self(value))
        } else {
            Err("Invalid immediate value for FMov64 kind")
        }
    }
}

impl Debug for ImmFMov64 {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(value) = self;
        write!(f, "Imm:FMov64({value})")
    }
}

impl ImmFMov64 {
    pub fn new(value: f64) -> Self {
        if imm_traits::try_cast_f64_to_aarch8(value).is_some() {
            Self(value)
        } else {
            panic!("Invalid immediate value: {value} for FMov64 kind");
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
        let MirOperand::Imm64(x) = mir else {
            panic!("Expected MirOperand::Imm64, found {mir:?}");
        };
        Self::from_real(x)
    }

    fn into_mir(self) -> MirOperand {
        self.into_real().into_mir()
    }

    fn try_from_real(real: Imm64) -> Option<Self> {
        if real.get_kind() != ImmKind::FMov {
            return None;
        }
        let imm = real.get_value();
        if imm & 0xFFu64 != imm {
            return None;
        }
        Some(Self(fp8aarch_to_fp64(imm as u8)))
    }

    fn from_real(real: Imm64) -> Self {
        if real.get_kind() != ImmKind::FMov {
            panic!("Expected Imm64 with FMov kind, found {real:?}");
        }
        let imm = real.get_value();
        if imm & 0xFFu64 != imm {
            panic!("Immediate {imm:#x} bits not available for FMov64");
        }
        Self(fp8aarch_to_fp64(imm as u8))
    }

    fn into_real(self) -> Imm64 {
        let Self(value) = self;
        let Some(x) = imm_traits::try_cast_f64_to_aarch8(value) else {
            panic!("Value error: {value} cannot cast to FMovImm64");
        };
        Imm64(x as u64, ImmKind::FMov)
    }

    fn insert_to_real(self, real: Imm64) -> Imm64 {
        if real.get_kind() != ImmKind::FMov {
            panic!("Expected Imm64 with FMov kind, but found {real:?}");
        }
        self.into_real()
    }

    fn fmt_asm(&self, formatter: &mut crate::mir::fmt::FuncFormatContext<'_>) -> std::fmt::Result {
        write!(formatter.writer, "#{:e}", self.0)
    }
}

#[cfg(test)]
mod testing {
    use crate::mir::operand::{imm::ImmFMov32, IMirSubOperand};

    #[test]
    fn test_imm_fmov32() {
        let imm = ImmFMov32::new(1.0);
        println!("{imm:?}");
        println!("{:?}", imm.into_mir());
        println!("{:?}", imm.into_real());

        println!("{:.20e}", f32::from_bits(0x40b00000))
    }
}
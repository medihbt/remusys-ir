use crate::ir::cmp_cond::CmpCond;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MirCondFlag {
    EQ = 0b0000,
    NE = 0b0001,
    CS = 0b0010, // Carry Set
    CC = 0b0011, // Carry Clear
    MI = 0b0100, // Minus
    PL = 0b0101, // Plus
    VS = 0b0110, // Overflow Set
    VC = 0b0111, // Overflow Clear
    HI = 0b1000, // Unsigned Higher
    LS = 0b1001, // Unsigned Lower or Same
    GE = 0b1010, // Signed Greater or Equal
    LT = 0b1011, // Signed Less Than
    GT = 0b1100, // Signed Greater Than
    LE = 0b1101, // Signed Less Than or Equal
    AL = 0b1110, // Always
    NV = 0b1111, // Never
}

impl MirCondFlag {
    pub fn from_cmp_cond(cond: CmpCond) -> Self {
        let signed = if cond.is_float() {
            true
        } else {
            cond.is_signed_ordered()
        };
        #[rustfmt::skip]
        return match cond.get_basic_cond() {
            CmpCond::LT => if signed { Self::LT } else { Self::CC },
            CmpCond::EQ => Self::EQ,
            CmpCond::GT => if signed { Self::GT } else { Self::HI },
            CmpCond::LE => if signed { Self::LE } else { Self::LS },
            CmpCond::NE => Self::NE,
            CmpCond::GE => if signed { Self::GE } else { Self::CS },
            CmpCond::ALWAYS => Self::AL,
            CmpCond::NEVER  => Self::NV,
            _ => unreachable!(),
        };
    }

    pub fn get_name(self) -> &'static str {
        #[rustfmt::skip]
        return match self {
            Self::EQ => "EQ", Self::NE => "NE", Self::CS => "CS", Self::CC => "CC",
            Self::MI => "MI", Self::PL => "PL", Self::VS => "VS", Self::VC => "VC",
            Self::HI => "HI", Self::LS => "LS", Self::GE => "GE", Self::LT => "LT",
            Self::GT => "GT", Self::LE => "LE", Self::AL => "AL", Self::NV => "NV",
        };
    }
}

impl ToString for MirCondFlag {
    fn to_string(&self) -> String {
        self.get_name().to_string()
    }
}

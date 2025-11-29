use bitflags::bitflags;

bitflags! {
    /// IR 比较条件.
    #[derive(Clone, Copy, PartialEq, Eq, Debug)]
    pub struct CmpCond: u8 {
        const LT = 0b00_001;
        const EQ = 0b00_010;
        const GT = 0b00_100;
        const LE = 0b00_011;
        const NE = 0b00_101;
        const GE = 0b00_110;

        const ALWAYS = 0b00_111;
        const NEVER  = 0b00_000;

        /// 取决于 `FLOAT_SWITCH`.
        ///
        /// * `FLOAT_SWITCH = false`: 为 true 则为有符号比较, false 则为无符号比较.
        /// * `FLOAT_SWITCH = true`: 为 true 则为有序比较, false 则为无序比较.
        const SIGNED_ORDERED = 0b01_000;
        const SLT = 0b01_001;
        const SEQ = 0b01_010;
        const SGT = 0b01_100;
        const SLE = 0b01_011;
        const SNE = 0b01_101;
        const SGE = 0b01_110;
        const SALWAYS = 0b01_111;
        const SNEVER  = 0b01_000;

        /// 浮点数开关.
        ///
        /// * `FLOAT_SWITCH = false`: 整数比较.
        /// * `FLOAT_SWITCH = true`: 浮点数比较.
        const FLOAT_SWITCH = 0b10_000;
        const FULT = 0b10_001;
        const FUEQ = 0b10_010;
        const FUGT = 0b10_100;
        const FULE = 0b10_011;
        const FUNE = 0b10_101;
        const FUGE = 0b10_110;
        const FUALWAYS = 0b10_111;
        const FUNEVER  = 0b10_000;

        const FOLT = 0b11_001;
        const FOEQ = 0b11_010;
        const FOGT = 0b11_100;
        const FOLE = 0b11_011;
        const FONE = 0b11_101;
        const FOGE = 0b11_110;
        const FOALWAYS = 0b11_111;
        const FONEVER  = 0b11_000;
    }
}

impl CmpCond {
    pub fn is_signed(self) -> Option<bool> {
        if self.contains(Self::FLOAT_SWITCH) {
            None
        } else {
            Some(self.contains(Self::SIGNED_ORDERED))
        }
    }
    pub fn is_float(self) -> bool {
        self.contains(Self::FLOAT_SWITCH)
    }
    pub fn is_int(self) -> bool {
        !self.contains(Self::FLOAT_SWITCH)
    }

    pub fn is_signed_ordered(self) -> bool {
        self.contains(Self::SIGNED_ORDERED)
    }
    pub fn switch_to_float(self) -> Self {
        self | Self::FLOAT_SWITCH
    }
    pub fn switch_to_int(self) -> Self {
        self & !Self::FLOAT_SWITCH
    }

    /// 获取不包含符号和浮点信息的基本比较条件.
    pub fn get_basic_cond(mut self) -> Self {
        self.remove(Self::SIGNED_ORDERED | Self::FLOAT_SWITCH);
        self
    }

    pub fn as_str(self) -> &'static str {
        #[rustfmt::skip]
        return match self {
            Self::LT => "lt",
            Self::EQ | Self::SEQ => "eq",
            Self::GT => "gt",
            Self::LE => "le",
            Self::NE | Self::SNE => "ne",
            Self::GE => "ge",
            Self::ALWAYS | Self::SALWAYS | Self::FUALWAYS | Self::FOALWAYS => "true",
            Self::NEVER | Self::SNEVER | Self::FUNEVER | Self::FONEVER => "false",

            Self::SLT => "slt", Self::SGT => "sgt",
            Self::SLE => "sle", Self::SGE => "sge",

            Self::FULT => "ult", Self::FUEQ => "ueq",
            Self::FUGT => "ugt", Self::FULE => "ule",
            Self::FUNE => "une", Self::FUGE => "uge",

            Self::FOLT => "olt", Self::FOEQ => "oeq",
            Self::FOGT => "ogt", Self::FOLE => "ole",
            Self::FONE => "one", Self::FOGE => "oge",

            _ => panic!("Unknown CmpCond: {:?}", self),
        };
    }
}

impl std::fmt::Display for CmpCond {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

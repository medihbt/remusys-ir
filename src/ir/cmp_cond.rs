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

        /// 浮点数开关.
        ///
        /// * `FLOAT_SWITCH = false`: 整数比较.
        /// * `FLOAT_SWITCH = true`: 浮点数比较.
        const FLOAT_SWITCH = 0b10_000;
    }
}

impl CmpCond {
    pub fn is_signed(&self) -> Option<bool> {
        if self.contains(Self::FLOAT_SWITCH) {
            None
        } else {
            Some(self.contains(Self::SIGNED_ORDERED))
        }
    }
    pub fn is_float(&self) -> bool {
        self.contains(Self::FLOAT_SWITCH)
    }
    pub fn is_int(&self) -> bool {
        !self.contains(Self::FLOAT_SWITCH)
    }

    pub fn is_signed_ordered(&self) -> bool {
        self.contains(Self::SIGNED_ORDERED)
    }
    pub fn switch_to_float(&self) -> Self {
        *self | Self::FLOAT_SWITCH
    }
    pub fn switch_to_int(&self) -> Self {
        *self & !Self::FLOAT_SWITCH
    }

    /// 获取不包含符号和浮点信息的基本比较条件.
    pub fn get_basic_cond(&self) -> Self {
        let mut ret = self.clone();
        ret.remove(Self::SIGNED_ORDERED | Self::FLOAT_SWITCH);
        ret
    }
}

impl std::fmt::Display for CmpCond {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let basic_name = match self.get_basic_cond() {
            Self::LT => "lt",
            Self::EQ => return write!(f, "eq"),
            Self::GT => "gt",
            Self::LE => "le",
            Self::NE => return write!(f, "ne"),
            Self::GE => "ge",
            Self::ALWAYS => return write!(f, "true"),
            Self::NEVER => return write!(f, "false"),
            _ => unreachable!(),
        };
        if self.is_float() && self.is_signed_ordered() {
            write!(f, "o{basic_name}")
        } else if self.is_int() && self.is_signed_ordered() {
            write!(f, "s{basic_name}")
        } else if self.is_float() {
            write!(f, "f{basic_name}")
        } else {
            write!(f, "{basic_name}")
        }
    }
}

use bitflags::bitflags;

bitflags! {
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

        const SIGNED_ORDERED = 0b01_000;
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

    pub fn get_basic_cond(&self) -> Self {
        let mut ret = self.clone();
        ret.remove(Self::SIGNED_ORDERED | Self::FLOAT_SWITCH);
        ret
    }
}

impl ToString for CmpCond {
    fn to_string(&self) -> String {
        let basic_name = match self.get_basic_cond() {
            Self::LT => "lt",
            Self::EQ => return "eq".into(),
            Self::GT => "ge",
            Self::LE => "le",
            Self::NE => return "ne".into(),
            Self::GE => "ge",
            Self::ALWAYS => "true",
            Self::NEVER => "false",
            _ => unreachable!(),
        };
        if self.is_float() && self.is_signed_ordered() {
            format!("o{}", basic_name)
        } else if self.is_int() && self.is_signed_ordered() {
            format!("s{}", basic_name)
        } else if self.is_float() {
            format!("f{}", basic_name)
        } else {
            basic_name.to_string()
        }
    }
}

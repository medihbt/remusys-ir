use crate::typing::{
    IValType, TypeAllocs, TypeContext, TypeFormatter, TypeMismatchErr, TypingRes, ValTypeClass,
    ValTypeID,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FPKind {
    Ieee32,
    Ieee64,
}

impl std::fmt::Debug for FPKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_name())
    }
}
impl std::fmt::Display for FPKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_name())
    }
}
impl core::str::FromStr for FPKind {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "float" => Ok(FPKind::Ieee32),
            "double" => Ok(FPKind::Ieee64),
            _ => Err(()),
        }
    }
}

impl IValType for FPKind {
    fn try_from_ir(ty: ValTypeID) -> TypingRes<Self> {
        if let ValTypeID::Float(kind) = ty {
            Ok(kind)
        } else {
            Err(TypeMismatchErr::NotClass(ty, ValTypeClass::Float))
        }
    }

    fn into_ir(self) -> ValTypeID {
        ValTypeID::Float(self)
    }

    fn makes_instance(self) -> bool {
        true
    }

    fn class_id(self) -> ValTypeClass {
        ValTypeClass::Float
    }

    fn try_get_size_full(self, _: &TypeAllocs, _: &TypeContext) -> Option<usize> {
        let size = match self {
            FPKind::Ieee32 => 4,
            FPKind::Ieee64 => 8,
        };
        Some(size)
    }

    fn try_get_align_full(self, _: &TypeAllocs, _: &TypeContext) -> Option<usize> {
        let align = match self {
            FPKind::Ieee32 => 4,
            FPKind::Ieee64 => 8,
        };
        Some(align)
    }

    fn format_ir<T: std::io::Write>(self, f: &TypeFormatter<T>) -> std::io::Result<()> {
        match self {
            FPKind::Ieee32 => f.write_str("float"),
            FPKind::Ieee64 => f.write_str("double"),
        }
    }
}

impl FPKind {
    pub fn get_name(self) -> &'static str {
        match self {
            FPKind::Ieee32 => "float",
            FPKind::Ieee64 => "double",
        }
    }
}

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct IntType(pub u8);

impl std::fmt::Debug for IntType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "i{}", self.0)
    }
}
impl std::fmt::Display for IntType {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "i{}", self.0)
    }
}
impl core::str::FromStr for IntType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if let Some(bits_str) = s.strip_prefix('i')
            && let Ok(bits) = bits_str.parse::<u8>()
        {
            Ok(IntType(bits))
        } else {
            Err(())
        }
    }
}

impl IValType for IntType {
    fn try_from_ir(ty: ValTypeID) -> TypingRes<Self> {
        if let ValTypeID::Int(bits) = ty {
            Ok(IntType(bits))
        } else {
            Err(TypeMismatchErr::NotClass(ty, ValTypeClass::Int))
        }
    }

    fn into_ir(self) -> ValTypeID {
        ValTypeID::Int(self.0)
    }

    fn makes_instance(self) -> bool {
        true
    }

    fn class_id(self) -> ValTypeClass {
        ValTypeClass::Int
    }

    fn try_get_size_full(self, _: &TypeAllocs, _: &TypeContext) -> Option<usize> {
        Some(self.0.div_ceil(8) as usize)
    }

    fn try_get_align_full(self, _: &TypeAllocs, _: &TypeContext) -> Option<usize> {
        let closing = self.0.next_power_of_two();
        Some(closing.div_ceil(8) as usize)
    }

    fn format_ir<T: std::io::Write>(self, f: &TypeFormatter<T>) -> std::io::Result<()> {
        write!(f, "i{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PtrType;

impl core::str::FromStr for PtrType {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "ptr" { Ok(PtrType) } else { Err(()) }
    }
}

impl IValType for PtrType {
    fn try_from_ir(ty: ValTypeID) -> TypingRes<Self> {
        if ty == ValTypeID::Ptr {
            Ok(PtrType)
        } else {
            Err(TypeMismatchErr::NotClass(ty, ValTypeClass::Ptr))
        }
    }

    fn into_ir(self) -> ValTypeID {
        ValTypeID::Ptr
    }

    fn makes_instance(self) -> bool {
        true
    }

    fn class_id(self) -> ValTypeClass {
        ValTypeClass::Ptr
    }

    fn try_get_size_full(self, _: &TypeAllocs, tctx: &TypeContext) -> Option<usize> {
        Some(tctx.arch.ptr_nbits.div_ceil(8) as usize)
    }

    fn try_get_align_full(self, _: &TypeAllocs, tctx: &TypeContext) -> Option<usize> {
        Some(tctx.arch.ptr_nbits.div_ceil(8) as usize)
    }

    fn format_ir<T: std::io::Write>(self, f: &TypeFormatter<T>) -> std::io::Result<()> {
        f.write_str("ptr")
    }
}

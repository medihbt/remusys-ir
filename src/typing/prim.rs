use crate::typing::{
    IValType, TypeAllocs, TypeContext, TypeFormatter, TypeMismatchError, TypingRes, ValTypeClass,
    ValTypeID,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum FPKind {
    Ieee32,
    Ieee64,
}

impl IValType for FPKind {
    fn try_from_ir(ty: ValTypeID) -> TypingRes<Self> {
        if let ValTypeID::Float(kind) = ty {
            Ok(kind)
        } else {
            Err(TypeMismatchError::NotClass(ty, ValTypeClass::Float))
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

    fn serialize<T: std::io::Write>(self, f: &TypeFormatter<T>) -> std::io::Result<()> {
        let name = match self {
            FPKind::Ieee32 => "float",
            FPKind::Ieee64 => "double",
        };
        f.write_str(name)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IntType(pub u8);

impl IValType for IntType {
    fn try_from_ir(ty: ValTypeID) -> TypingRes<Self> {
        if let ValTypeID::Int(bits) = ty {
            Ok(IntType(bits))
        } else {
            Err(TypeMismatchError::NotClass(ty, ValTypeClass::Int))
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

    fn serialize<T: std::io::Write>(self, f: &TypeFormatter<T>) -> std::io::Result<()> {
        write!(f, "i{}", self.0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PtrType;

impl IValType for PtrType {
    fn try_from_ir(ty: ValTypeID) -> TypingRes<Self> {
        if ty == ValTypeID::Ptr {
            Ok(PtrType)
        } else {
            Err(TypeMismatchError::NotClass(ty, ValTypeClass::Ptr))
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

    fn serialize<T: std::io::Write>(self, f: &TypeFormatter<T>) -> std::io::Result<()> {
        f.write_str("ptr")
    }
}

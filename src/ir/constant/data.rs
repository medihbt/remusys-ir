use crate::{
    base::APInt,
    ir::{IRAllocs, IRWriter, ISubValueSSA, ValueSSA},
    typing::{FPKind, IValType, ScalarType, ValTypeID},
};
use std::hash::Hash;

#[derive(Debug, Clone, Copy)]
pub enum ConstData {
    Undef(ValTypeID),
    Zero(ScalarType),
    PtrNull(ValTypeID),
    Int(APInt),
    Float(FPKind, f64),
}

impl PartialEq for ConstData {
    fn eq(&self, other: &Self) -> bool {
        use crate::typing::FPKind::*;
        use ConstData::*;
        match (self, other) {
            (Undef(l0), Undef(r0)) => l0 == r0,
            (Zero(l0), Zero(r0)) => l0 == r0,
            (PtrNull(l0), PtrNull(r0)) => l0 == r0,
            (Int(l), Int(r)) => l == r,
            (Float(Ieee32, l1), Float(Ieee32, r1)) => {
                (*l1 as f32).to_bits() == (*r1 as f32).to_bits()
            }
            (Float(Ieee64, l1), Float(Ieee64, r1)) => {
                (*l1 as f64).to_bits() == (*r1 as f64).to_bits()
            }
            _ => false,
        }
    }
}

impl Hash for ConstData {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        core::mem::discriminant(self).hash(state);
        use crate::typing::FPKind::*;
        use ConstData::*;
        match self {
            Undef(ty) => ty.hash(state),
            Zero(ty) => ty.hash(state),
            PtrNull(ty) => ty.hash(state),
            Int(apint) => apint.hash(state),
            Float(Ieee32, value) => {
                (*value as f32).to_bits().hash(state);
            }
            Float(Ieee64, value) => {
                (*value as f64).to_bits().hash(state);
            }
        }
    }
}

impl Eq for ConstData {
    /* 在上面重写的 PartialEq 中, 我们能确定这个 Eq 没问题. */
}

impl ISubValueSSA for ConstData {
    fn try_from_ir(value: ValueSSA) -> Option<Self> {
        match value {
            ValueSSA::ConstData(data) => Some(data),
            _ => None,
        }
    }
    fn into_ir(self) -> ValueSSA {
        ValueSSA::ConstData(self)
    }

    fn get_valtype(self, _: &IRAllocs) -> ValTypeID {
        match self {
            ConstData::Undef(ty) => ty,
            ConstData::Zero(ty) => ty.into_ir(),
            ConstData::PtrNull(_) => ValTypeID::Ptr,
            ConstData::Int(apint) => ValTypeID::Int(apint.bits()),
            ConstData::Float(kind, _) => ValTypeID::Float(kind),
        }
    }

    fn try_gettype_noalloc(self) -> Option<ValTypeID> {
        Some((&self).get_valtype_noalloc())
    }

    fn is_zero(&self, _: &IRAllocs) -> bool {
        self.is_zero()
    }

    fn fmt_ir(&self, writer: &IRWriter) -> std::io::Result<()> {
        match self {
            ConstData::Undef(_) => writer.write_str("undef"),
            ConstData::Zero(ty) => match ty {
                ScalarType::Ptr => writer.write_str("null"),
                ScalarType::Int(_) => writer.write_str("0"),
                ScalarType::Float(_) => writer.write_str("0.0"),
            },
            ConstData::PtrNull(_) => writer.write_str("null"),
            ConstData::Int(apint) => {
                write!(writer.output.borrow_mut(), "{}", apint.as_signed())
            }
            ConstData::Float(FPKind::Ieee32, fp) => {
                write!(writer.output.borrow_mut(), "{:.20e}", *fp as f32)
            }
            ConstData::Float(FPKind::Ieee64, fp) => {
                write!(writer.output.borrow_mut(), "{:.20e}", *fp)
            }
        }
    }
}

impl From<APInt> for ConstData {
    fn from(value: APInt) -> Self {
        ConstData::Int(value)
    }
}

impl From<f64> for ConstData {
    fn from(value: f64) -> Self {
        ConstData::Float(FPKind::Ieee64, value)
    }
}

impl From<f32> for ConstData {
    fn from(value: f32) -> Self {
        ConstData::Float(FPKind::Ieee32, value as f64)
    }
}
impl ConstData {
    pub fn get_valtype_noalloc(&self) -> ValTypeID {
        match self {
            ConstData::Undef(ty) => *ty,
            ConstData::Zero(ty) => ty.into_ir(),
            ConstData::PtrNull(_) => ValTypeID::Ptr,
            ConstData::Int(apint) => ValTypeID::Int(apint.bits()),
            ConstData::Float(kind, _) => ValTypeID::Float(*kind),
        }
    }

    pub fn is_zero(&self) -> bool {
        use ConstData::*;
        use FPKind::*;
        match self {
            Zero(_) | PtrNull(_) => true,
            Int(x) => *x == 0,
            Float(Ieee32, f) => (*f as f32).to_bits() == 0,
            Float(Ieee64, f) => (*f as f64).to_bits() == 0,
            _ => false,
        }
    }

    pub fn as_apint(&self) -> Option<APInt> {
        match self {
            ConstData::Int(apint) => Some(*apint),
            _ => None,
        }
    }
}

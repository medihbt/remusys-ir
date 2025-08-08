use crate::{
    base::APInt,
    ir::{IRAllocs, IRWriter, ISubValueSSA, ValueSSA},
    typing::{id::ValTypeID, types::FloatTypeKind},
};
use std::hash::Hash;

#[derive(Debug, Clone, Copy)]
pub enum ConstData {
    Undef(ValTypeID),
    Zero(ValTypeID),
    PtrNull(ValTypeID),
    Int(u8, u128),
    Float(FloatTypeKind, f64),
}

impl PartialEq for ConstData {
    fn eq(&self, other: &Self) -> bool {
        use crate::typing::types::FloatTypeKind::*;
        use ConstData::*;
        match (self, other) {
            (Undef(l0), Undef(r0)) => l0 == r0,
            (Zero(l0), Zero(r0)) => l0 == r0,
            (PtrNull(l0), PtrNull(r0)) => l0 == r0,
            (Int(l0, l1), Int(r0, r1)) => l0 == r0 && l1 == r1,
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
        use crate::typing::types::FloatTypeKind::*;
        use ConstData::*;
        match self {
            Undef(ty) => ty.hash(state),
            Zero(ty) => ty.hash(state),
            PtrNull(ty) => ty.hash(state),
            Int(bits, value) => {
                bits.hash(state);
                value.hash(state);
            }
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
    fn try_from_ir(value: &ValueSSA) -> Option<&Self> {
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
            ConstData::Zero(ty) => ty,
            ConstData::PtrNull(_) => ValTypeID::Ptr,
            ConstData::Int(bits, _) => ValTypeID::Int(bits),
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
                ValTypeID::Ptr => writer.write_str("null"),
                ValTypeID::Int(_) => writer.write_str("0"),
                ValTypeID::Float(_) => writer.write_str("0.0"),
                ValTypeID::Array(_) | ValTypeID::Struct(_) | ValTypeID::StructAlias(_) => {
                    writer.write_str("zeroinitializer")
                }
                _ => panic!("Unsupported type {ty:?} for zero constant"),
            },
            ConstData::PtrNull(_) => writer.write_str("null"),
            ConstData::Int(bit, val) => {
                let val = APInt::new(*val, *bit).as_signed();
                write!(writer.output.borrow_mut(), "{val}")
            }
            ConstData::Float(FloatTypeKind::Ieee32, fp) => {
                write!(writer.output.borrow_mut(), "{:.20e}", *fp as f32)
            }
            ConstData::Float(FloatTypeKind::Ieee64, fp) => {
                write!(writer.output.borrow_mut(), "{:.20e}", *fp)
            }
        }
    }
}

impl From<APInt> for ConstData {
    fn from(value: APInt) -> Self {
        ConstData::Int(value.bits(), value.as_unsigned())
    }
}

impl From<f64> for ConstData {
    fn from(value: f64) -> Self {
        ConstData::Float(FloatTypeKind::Ieee64, value)
    }
}

impl From<f32> for ConstData {
    fn from(value: f32) -> Self {
        ConstData::Float(FloatTypeKind::Ieee32, value as f64)
    }
}
impl ConstData {
    pub fn get_valtype_noalloc(&self) -> ValTypeID {
        match self {
            ConstData::Undef(ty) => *ty,
            ConstData::Zero(ty) => *ty,
            ConstData::PtrNull(_) => ValTypeID::Ptr,
            ConstData::Int(bits, _) => ValTypeID::Int(*bits),
            ConstData::Float(kind, _) => ValTypeID::Float(*kind),
        }
    }

    pub fn is_zero(&self) -> bool {
        use ConstData::*;
        use FloatTypeKind::*;
        match self {
            Zero(_) | PtrNull(_) | Int(_, 0) => true,
            Float(Ieee32, f) => (*f as f32).to_bits() == 0,
            Float(Ieee64, f) => (*f as f64).to_bits() == 0,
            _ => false,
        }
    }

    pub fn as_apint(&self) -> Option<APInt> {
        match self {
            ConstData::Int(bits, value) => Some(APInt::new(*value, *bits)),
            _ => None,
        }
    }
}

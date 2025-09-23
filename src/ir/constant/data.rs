use crate::{
    base::APInt,
    ir::{IRAllocs, IRWriter, ISubValueSSA, ValueSSA},
    typing::{FPKind, IValType, ScalarType, ValTypeID},
};
use std::{
    hash::Hash,
    ops::{Neg, Rem, Shl},
};

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

/// 常量计算错误.
#[derive(Debug, thiserror::Error, Clone, Copy, PartialEq, Eq)]
pub enum ConstCalcError {
    #[error("constant value type mismatch")]
    TypeMismatch,
    #[error("constant value operation not supported")]
    UnsupportedOp,
    #[error("constant value division by zero")]
    DivByZero,
}
pub type ConstCalcRes<T = ()> = Result<T, ConstCalcError>;

impl ConstData {
    pub fn add(&self, other: &Self) -> ConstCalcRes<Self> {
        if self.get_valtype_noalloc() != other.get_valtype_noalloc() {
            return Err(ConstCalcError::TypeMismatch);
        }
        match (self, other) {
            (Self::Undef(_), _) | (_, Self::Zero(_)) | (_, Self::PtrNull(_)) => Ok(self.clone()),
            (_, Self::Undef(_)) | (Self::Zero(_), _) | (Self::PtrNull(_), _) => Ok(other.clone()),
            (Self::Int(lv), Self::Int(rv)) if lv.bits() == rv.bits() => {
                Ok(ConstData::Int(*lv + *rv))
            }
            (Self::Float(lk, lv), ConstData::Float(rk, rv)) if lk == rk => {
                Ok(ConstData::Float(*lk, *lv + *rv))
            }
            _ => Err(ConstCalcError::UnsupportedOp),
        }
    }

    pub fn neg(&self) -> Self {
        match self {
            Self::Undef(ty) => Self::Undef(*ty),
            Self::Zero(ty) => Self::Zero(*ty),
            Self::PtrNull(ty) => Self::PtrNull(*ty),
            Self::Int(v) => Self::Int(v.neg()),
            Self::Float(kind, v) => Self::Float(*kind, -*v),
        }
    }

    pub fn sub(&self, other: &Self) -> ConstCalcRes<Self> {
        self.add(&other.neg())
    }

    pub fn mul(&self, other: &Self) -> ConstCalcRes<Self> {
        if self.get_valtype_noalloc() != other.get_valtype_noalloc() {
            return Err(ConstCalcError::TypeMismatch);
        }
        match (self, other) {
            (Self::Undef(_), _) | (Self::Zero(_), _) | (Self::PtrNull(_), _) => Ok(self.clone()),
            (_, Self::Undef(_)) | (_, Self::Zero(_)) | (_, Self::PtrNull(_)) => Ok(other.clone()),
            (Self::Int(lv), Self::Int(rv)) if lv.bits() == rv.bits() => {
                Ok(ConstData::Int(*lv * *rv))
            }
            (Self::Float(lk, lv), ConstData::Float(rk, rv)) if lk == rk => {
                Ok(ConstData::Float(*lk, *lv * *rv))
            }
            _ => Err(ConstCalcError::UnsupportedOp),
        }
    }

    pub fn sdiv(&self, other: &Self) -> ConstCalcRes<Self> {
        if self.get_valtype_noalloc() != other.get_valtype_noalloc() {
            return Err(ConstCalcError::TypeMismatch);
        }
        match (self, other) {
            (Self::Undef(_), _) => Ok(self.clone()),
            (_, Self::Undef(_)) => Ok(other.clone()),
            (_, Self::Zero(_)) | (_, Self::PtrNull(_)) => Err(ConstCalcError::DivByZero),
            (Self::Zero(_), _) | (Self::PtrNull(_), _) => Ok(self.clone()),
            (Self::Int(lv), Self::Int(rv)) if lv.bits() == rv.bits() => {
                if rv.is_nonzero() {
                    Ok(ConstData::Int(lv.sdiv(*rv)))
                } else {
                    Err(ConstCalcError::DivByZero)
                }
            }
            (Self::Float(lk, lv), ConstData::Float(rk, rv)) if lk == rk => {
                if *rv != 0.0 {
                    Ok(ConstData::Float(*lk, *lv / *rv))
                } else {
                    Err(ConstCalcError::DivByZero)
                }
            }
            _ => Err(ConstCalcError::UnsupportedOp),
        }
    }

    pub fn udiv(&self, other: &Self) -> ConstCalcRes<Self> {
        if self.get_valtype_noalloc() != other.get_valtype_noalloc() {
            return Err(ConstCalcError::TypeMismatch);
        }
        match (self, other) {
            (Self::Undef(_), _) => Ok(self.clone()),
            (_, Self::Undef(_)) => Ok(other.clone()),
            (_, Self::Zero(_)) | (_, Self::PtrNull(_)) => Err(ConstCalcError::DivByZero),
            (Self::Zero(_), _) | (Self::PtrNull(_), _) => Ok(self.clone()),
            (Self::Int(lv), Self::Int(rv)) if lv.bits() == rv.bits() => {
                if rv.is_nonzero() {
                    Ok(ConstData::Int(lv.udiv(*rv)))
                } else {
                    Err(ConstCalcError::DivByZero)
                }
            }
            (Self::Float(lk, lv), ConstData::Float(rk, rv)) if lk == rk => {
                if *rv != 0.0 {
                    Ok(ConstData::Float(*lk, *lv / *rv))
                } else {
                    Err(ConstCalcError::DivByZero)
                }
            }
            _ => Err(ConstCalcError::UnsupportedOp),
        }
    }

    pub fn srem(&self, other: &Self) -> ConstCalcRes<Self> {
        if self.get_valtype_noalloc() != other.get_valtype_noalloc() {
            return Err(ConstCalcError::TypeMismatch);
        }
        match (self, other) {
            (Self::Undef(_), _) => Ok(self.clone()),
            (_, Self::Undef(_)) => Ok(other.clone()),
            (_, Self::Zero(_)) | (_, Self::PtrNull(_)) => Err(ConstCalcError::DivByZero),
            (Self::Zero(_), _) | (Self::PtrNull(_), _) => Ok(self.clone()),
            (Self::Int(lv), Self::Int(rv)) if lv.bits() == rv.bits() => {
                if rv.is_nonzero() {
                    Ok(ConstData::Int(lv.srem(*rv)))
                } else {
                    Err(ConstCalcError::DivByZero)
                }
            }
            (Self::Float(lk, lv), ConstData::Float(rk, rv)) if lk == rk => {
                if *rv != 0.0 {
                    Ok(ConstData::Float(*lk, lv.rem(*rv)))
                } else {
                    Err(ConstCalcError::DivByZero)
                }
            }
            _ => Err(ConstCalcError::UnsupportedOp),
        }
    }

    pub fn urem(&self, other: &Self) -> ConstCalcRes<Self> {
        if self.get_valtype_noalloc() != other.get_valtype_noalloc() {
            return Err(ConstCalcError::TypeMismatch);
        }
        match (self, other) {
            (Self::Undef(_), _) => Ok(self.clone()),
            (_, Self::Undef(_)) => Ok(other.clone()),
            (_, Self::Zero(_)) | (_, Self::PtrNull(_)) => Err(ConstCalcError::DivByZero),
            (Self::Zero(_), _) | (Self::PtrNull(_), _) => Ok(self.clone()),
            (Self::Int(lv), Self::Int(rv)) if lv.bits() == rv.bits() => {
                if rv.is_nonzero() {
                    Ok(ConstData::Int(lv.urem(*rv)))
                } else {
                    Err(ConstCalcError::DivByZero)
                }
            }
            (Self::Float(lk, lv), ConstData::Float(rk, rv)) if lk == rk => {
                if *rv != 0.0 {
                    Ok(ConstData::Float(*lk, lv.rem(*rv)))
                } else {
                    Err(ConstCalcError::DivByZero)
                }
            }
            _ => Err(ConstCalcError::UnsupportedOp),
        }
    }

    pub fn shl(&self, other: &Self) -> ConstCalcRes<Self> {
        if self.get_valtype_noalloc() != other.get_valtype_noalloc() {
            return Err(ConstCalcError::TypeMismatch);
        }
        match (self, other) {
            (Self::Undef(_), _) => Ok(self.clone()),
            (_, Self::Undef(_)) => Ok(other.clone()),
            (_, Self::Zero(_)) | (_, Self::PtrNull(_)) => Ok(self.clone()),
            (Self::Zero(_), _) | (Self::PtrNull(_), _) => Ok(self.clone()),
            (Self::Int(lv), Self::Int(rv)) if lv.bits() == rv.bits() => {
                if rv.is_nonzero() {
                    Ok(ConstData::Int(lv.shl(*rv)))
                } else {
                    Ok(self.clone())
                }
            }
            _ => Err(ConstCalcError::UnsupportedOp),
        }
    }

    pub fn lshr(&self, other: &Self) -> ConstCalcRes<Self> {
        if self.get_valtype_noalloc() != other.get_valtype_noalloc() {
            return Err(ConstCalcError::TypeMismatch);
        }
        match (self, other) {
            (Self::Undef(_), _) => Ok(self.clone()),
            (_, Self::Undef(_)) => Ok(other.clone()),
            (_, Self::Zero(_)) | (_, Self::PtrNull(_)) => Ok(self.clone()),
            (Self::Zero(_), _) | (Self::PtrNull(_), _) => Ok(self.clone()),
            (Self::Int(lv), Self::Int(rv)) if lv.bits() == rv.bits() => {
                if rv.is_nonzero() {
                    Ok(ConstData::Int(lv.lshr_with(*rv)))
                } else {
                    Ok(self.clone())
                }
            }
            _ => Err(ConstCalcError::UnsupportedOp),
        }
    }
    pub fn ashr(&self, other: &Self) -> ConstCalcRes<Self> {
        if self.get_valtype_noalloc() != other.get_valtype_noalloc() {
            return Err(ConstCalcError::TypeMismatch);
        }
        match (self, other) {
            (Self::Undef(_), _) => Ok(self.clone()),
            (_, Self::Undef(_)) => Ok(other.clone()),
            (_, Self::Zero(_)) | (_, Self::PtrNull(_)) => Ok(self.clone()),
            (Self::Zero(_), _) | (Self::PtrNull(_), _) => Ok(self.clone()),
            (Self::Int(lv), Self::Int(rv)) if lv.bits() == rv.bits() => {
                if rv.is_nonzero() {
                    Ok(ConstData::Int(lv.ashr_with(*rv)))
                } else {
                    Ok(self.clone())
                }
            }
            _ => Err(ConstCalcError::UnsupportedOp),
        }
    }
}

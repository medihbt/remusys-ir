use std::ops::{Add, BitAnd, BitOr, BitXor, Div, Mul, Rem, Shl, Shr, Sub};

use crate::typing::{id::ValTypeID, types::FloatTypeKind};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ConstData {
    Undef(ValTypeID),
    Zero(ValTypeID),
    PtrNull(ValTypeID),
    Int(u8, i128),
    Float(FloatTypeKind, f64),
}

#[derive(Debug, Clone, Copy)]
pub enum ConstDataErr {
    ValueTypeMismatch(ValTypeID, ValTypeID),
    ClassMismatch(ConstData, ConstData),
    DivideByZero,
}

impl Eq for ConstData {}

impl ConstData {
    pub fn get_value_type(&self) -> ValTypeID {
        match self {
            ConstData::Undef(ty) => ty.clone(),
            ConstData::Zero(ty) => ty.clone(),
            ConstData::PtrNull(_) => ValTypeID::Ptr,
            ConstData::Int(bin_bits, _) => ValTypeID::Int(bin_bits.clone()),
            ConstData::Float(fp, _) => ValTypeID::Float(fp.clone()),
        }
    }

    pub fn value_cast_int(&self) -> Option<i128> {
        match self {
            ConstData::Int(_, value) => Some(*value),
            ConstData::Float(_, value) => Some(*value as i128),
            ConstData::Zero(_) => Some(0),
            _ => None,
        }
    }
    pub fn value_cast_float(&self) -> Option<f64> {
        match self {
            ConstData::Float(_, value) => Some(*value),
            ConstData::Int(_, value) => Some(*value as f64),
            ConstData::Zero(_) => Some(0.0),
            _ => None,
        }
    }
}

/// 用来补四则运算中两个操作数有一个为 0 的情况.
macro_rules! const_dasta_calculation_zero_action {
    ($left:ident, $right:ident, add) => {
        match ($left, $right) {
            (ConstData::Zero(_), _) => return Ok($right.clone()),
            (_, ConstData::Zero(_)) => return Ok($left.clone()),
            _ => {}
        }
    };
    ($left:ident, $right:ident, sub) => {
        match ($left, $right) {
            (ConstData::Zero(_), _) => return Ok($right.neg()),
            (_, ConstData::Zero(_)) => return Ok($left.clone()),
            _ => {}
        }
    };
    ($left:ident, $right:ident, mul) => {
        match ($left, $right) {
            (ConstData::Zero(_), _) => return Ok($left.make_zero()),
            (_, ConstData::Zero(_)) => return Ok($right.make_zero()),
            _ => {}
        }
    };
    ($left:ident, $right:ident, div) => {
        match ($left, $right) {
            (_, ConstData::Zero(_)) => return Err(ConstDataErr::DivideByZero),
            (ConstData::Zero(_), _) => {
                return if $right.is_zero() {
                    Ok($right.make_zero())
                } else {
                    Err(ConstDataErr::DivideByZero)
                }
            }
            _ => {}
        }
    };
    ($left:ident, $right:ident, rem) => {
        match ($left, $right) {
            (_, ConstData::Zero(_)) => return Err(ConstDataErr::DivideByZero),
            (ConstData::Zero(_), _) => {
                return if $right.is_zero() {
                    Ok($right.make_zero())
                } else {
                    Err(ConstDataErr::DivideByZero)
                }
            }
            _ => {}
        }
    };
}

/// INPUT: Calculator(+, -, *, /, %); calculator name (add, sub, mul, div, rem)
/// OUTPUT: Function name (add, sub, mul, div, rem)
macro_rules! const_data_calculation {
    ($calculator:ident, $calculator_name:ident) => {
        pub fn $calculator_name(&self, other: &Self) -> Result<Self, ConstDataErr> {
            if self.get_value_type() != other.get_value_type() {
                return Err(ConstDataErr::ValueTypeMismatch(
                    self.get_value_type(),
                    other.get_value_type(),
                ));
            }
            const_dasta_calculation_zero_action!(self, other, $calculator);
            match (self, other) {
                (ConstData::Int(lbit, a), ConstData::Int(rbit, b)) => {
                    Ok(ConstData::Int(*lbit.max(rbit), a.$calculator(b)))
                }
                (ConstData::Float(lkind, a), ConstData::Float(rkind, b)) => {
                    if lkind == rkind {
                        Ok(ConstData::Float(*lkind, a.$calculator(b)))
                    } else {
                        Err(ConstDataErr::ValueTypeMismatch(
                            self.get_value_type(),
                            other.get_value_type(),
                        ))
                    }
                }
                _ => Err(ConstDataErr::ClassMismatch(self.clone(), other.clone())),
            }
        }
    };
}

/// INPUT: Logic & Shift Operator(<<, >>, &, |, ^); calculator name (shl, shr, and, or, xor)
/// OUTPUT: Function name (shl, shr, and, or, xor)
macro_rules! int_const_binary_calculaton {
    ($calculator:ident, $calculator_name:ident) => {
        pub fn $calculator_name(&self, other: &Self) -> Result<Self, ConstDataErr> {
            if self.get_value_type() != other.get_value_type() {
                return Err(ConstDataErr::ValueTypeMismatch(
                    self.get_value_type(),
                    other.get_value_type(),
                ));
            }
            match (self, other) {
                (ConstData::Int(lbit, a), ConstData::Int(_, b)) => {
                    Ok(ConstData::Int(*lbit, a.$calculator(b)))
                }
                _ => Err(ConstDataErr::ClassMismatch(self.clone(), other.clone())),
            }
        }
    };
}

/// Math calculation
impl ConstData {
    pub fn neg(&self) -> Self {
        match self {
            Self::Int(bin_bits, value) => Self::Int(*bin_bits, -value),
            Self::Float(fp_kind, value) => Self::Float(*fp_kind, -value),
            _ => self.clone(),
        }
    }
    pub fn make_zero(&self) -> Self {
        match self {
            Self::Int(bin_bits, _) => Self::Int(*bin_bits, 0),
            Self::Float(fp_kind, _) => Self::Float(*fp_kind, 0.0),
            _ => self.clone(),
        }
    }
    pub fn is_zero(&self) -> bool {
        match self {
            Self::Zero(_) => true,
            Self::Int(_, value) => *value == 0,
            Self::Float(_, value) => *value == 0.0,
            _ => false,
        }
    }

    const_data_calculation!(add, add);
    const_data_calculation!(sub, sub);
    const_data_calculation!(mul, mul);
    const_data_calculation!(div, div);
    const_data_calculation!(rem, rem);

    int_const_binary_calculaton!(shl, shl);
    int_const_binary_calculaton!(shr, shr);
    int_const_binary_calculaton!(bitand, bitand);
    int_const_binary_calculaton!(bitor, bitor);
    int_const_binary_calculaton!(bitxor, bitxor);
}

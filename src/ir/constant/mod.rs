use crate::{base::slabref::SlabRef, typing::id::ValTypeID};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// A reference to a constant value.
pub struct ConstantRef(pub(crate) usize);

pub enum Constant {
    Undef(ConstantCommon),
    Zero (ConstantCommon),
    Int  (ConstantCommon, i128),
    Float(ConstantCommon, f64),

    PtrNull(ConstantCommon),
    Array (ConstantCommon, Vec<ConstantRef>),
    Struct(ConstantCommon, Vec<ConstantRef>),
}

impl SlabRef for ConstantRef {
    type RefObject = Constant;

    fn from_handle(handle: usize) -> Self { ConstantRef(handle) }
    fn get_handle (&self) -> usize { self.0 }
}

pub struct ConstantCommon {
    pub ty: ValTypeID,
}

impl Constant {
    pub fn get_common(&self) -> &ConstantCommon {
        match self {
            Constant::Undef(data) => data,
            Constant::Zero(data) => data,
            Constant::Int(data, _) => data,
            Constant::Float(data, _) => data,

            Constant::PtrNull(data) => data,
            Constant::Array(data, _) => data,
            Constant::Struct(data, _) => data,
        }
    }
    pub fn common_mut(&mut self) -> &mut ConstantCommon {
        match self {
            Constant::Undef(data) => data,
            Constant::Zero(data) => data,
            Constant::Int(data, _) => data,
            Constant::Float(data, _) => data,

            Constant::PtrNull(data) => data,
            Constant::Array(data, _) => data,
            Constant::Struct(data, _) => data,
        }
    }

    pub fn get_int(&self) -> Option<i128> {
        match self {
            Constant::Int  (_, value) => Some(*value),
            Constant::Float(_, value) => Some(*value as i128),
            _ => None,
        }
    }
    pub fn get_float(&self) -> Option<f64> {
        match self {
            Constant::Float(_, value) => Some(*value),
            Constant::Int  (_, value) => Some(*value as f64),
            _ => None,
        }
    }

    pub fn get_nelements(&self) -> Option<usize> {
        match self {
            Constant::Array(_, values) => Some(values.len()),
            Constant::Struct(_, values) => Some(values.len()),
            _ => None,
        }
    }
}

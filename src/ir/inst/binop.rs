//! Binary operation.

use super::usedef::UseRef;

pub struct BinOp {
    pub lhs: UseRef,
    pub rhs: UseRef,
}
use std::hash::{Hash, Hasher};

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum ImmConst {
    Zero,
    I32(i32),
    I64(i64),
    F32(f32),
    F64(f64),
}

impl Hash for ImmConst {
    fn hash<H: Hasher>(&self, state: &mut H) {
        std::mem::discriminant(self).hash(state);
        match self {
            ImmConst::Zero => 0.hash(state),
            ImmConst::I32(v) => v.hash(state),
            ImmConst::I64(v) => v.hash(state),
            ImmConst::F32(v) => v.to_bits().hash(state),
            ImmConst::F64(v) => v.to_bits().hash(state),
        }
    }
}

#[derive(Debug, Clone, PartialEq, Hash)]
pub enum ImmAggregate {
    Array(Vec<ImmConst>),
    ByteArray(Vec<u8>),
}

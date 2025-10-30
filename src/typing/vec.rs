use crate::typing::*;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FixVecType(pub ScalarType, pub u8);

impl IValType for FixVecType {
    fn try_from_ir(ty: ValTypeID) -> TypingRes<Self> {
        match ty {
            ValTypeID::FixVec(FixVecType(scalar, len_log2)) => Ok(FixVecType(scalar, len_log2)),
            _ => Err(TypeMismatchError::NotClass(ty, ValTypeClass::FixVec)),
        }
    }

    fn into_ir(self) -> ValTypeID {
        ValTypeID::FixVec(self)
    }

    fn makes_instance(self) -> bool {
        true
    }

    fn class_id(self) -> ValTypeClass {
        ValTypeClass::FixVec
    }

    fn serialize<T: Write>(self, f: &TypeFormatter<T>) -> std::io::Result<()> {
        f.write_str("<")?;
        self.0.serialize(f)?;
        write!(f, "x {}>", 1 << self.1)
    }

    fn try_get_size_full(self, alloc: &TypeAllocs, tctx: &TypeContext) -> Option<usize> {
        let scalar_size = self.0.try_get_size_full(alloc, tctx)?;
        Some(scalar_size.checked_mul(1 << self.1)?)
    }

    fn try_get_align_full(self, alloc: &TypeAllocs, tctx: &TypeContext) -> Option<usize> {
        let elem_align = self.0.try_get_align_full(alloc, tctx)?;
        elem_align.checked_mul(1 << self.1)
    }
}

impl FixVecType {
    pub fn get_len(self) -> usize {
        1 << self.1
    }
    pub fn get_len_log2(self) -> u8 {
        self.1
    }
    pub fn get_elem(self) -> ScalarType {
        self.0
    }

    pub fn try_get_offset(self, index: usize, tctx: &TypeContext) -> Option<usize> {
        if index >= self.get_len() {
            return None;
        }
        let elem_size = self.get_elem().get_size(tctx);
        Some(elem_size.checked_mul(index)?)
    }
    pub fn get_offset(self, index: usize, tctx: &TypeContext) -> usize {
        self.try_get_offset(index, tctx)
            .expect("Index out of bounds or size overflow")
    }
}

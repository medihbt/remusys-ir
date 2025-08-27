use crate::typing::{
    IValType, ScalarType, TypeAllocs, TypeContext, TypeFormatter, TypeMismatchError, TypingRes,
    ValTypeClass, ValTypeID,
};
use std::io::Write;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub struct FixVecType(pub ScalarType, pub u32);

impl IValType for FixVecType {
    fn try_from_ir(ty: ValTypeID) -> TypingRes<Self> {
        if let ValTypeID::FixVec(fv) = ty {
            Ok(fv)
        } else {
            Err(TypeMismatchError::NotClass(ty, ValTypeClass::FixVec))
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
        let Self(elemty, nelems) = self;
        write!(f, "<")?;
        elemty.serialize(f)?;
        write!(f, ", {nelems}>")?;
        Ok(())
    }

    fn try_get_size_full(self, alloc: &TypeAllocs, tctx: &TypeContext) -> Option<usize> {
        let Self(elemty, nelems) = self;
        let elemsize = elemty.try_get_size_full(alloc, tctx)?;
        Some(elemsize * nelems as usize)
    }

    fn try_get_align_full(self, alloc: &TypeAllocs, tctx: &TypeContext) -> Option<usize> {
        let Self(elemty, nelems) = self;
        let elemsize = elemty.try_get_align_full(alloc, tctx)?;
        Some(elemsize * nelems as usize)
    }
}

impl FixVecType {
    pub fn get_elemty(self) -> ScalarType {
        self.0
    }

    pub fn num_elems(self) -> u32 {
        self.1
    }

    pub fn try_num_elems_log2(self) -> Result<u8, u32> {
        let Self(_, n) = self;
        if n.is_power_of_two() { Ok(n.ilog2() as u8) } else { Err(n) }
    }
    pub fn num_elems_log2(self) -> u8 {
        self.try_num_elems_log2()
            .expect("Number of elements in FixVecType is not a power of two")
    }

    pub fn try_get_offset(self, index: u32, tctx: &TypeContext) -> Option<usize> {
        if index >= self.1 {
            return None;
        }
        let elemty = self.0;
        let elemsize = elemty.try_get_size(tctx)?;
        Some((index as usize) * elemsize)
    }
    pub fn get_offset(self, index: u32, tctx: &TypeContext) -> usize {
        self.try_get_offset(index, tctx)
            .expect("Failed to get offset from FixVecType")
    }
}

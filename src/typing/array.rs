use crate::{
    base::SlabRef,
    typing::{
        IValType, TypeAllocs, TypeContext, TypeFormatter, TypeMismatchError, TypingRes,
        ValTypeClass, ValTypeID,
    },
};
use std::{cell::Cell, io::Write};

#[derive(Debug, Clone)]
pub struct ArrayTypeData {
    pub elemty: ValTypeID,
    pub nelems: usize,
    elem_size_cache: Cell<usize>,
    elem_align_cache: Cell<usize>,
}

impl ArrayTypeData {
    pub fn new(elemty: ValTypeID, nelems: usize) -> Self {
        Self {
            elemty,
            nelems,
            elem_size_cache: Cell::new(0),
            elem_align_cache: Cell::new(0),
        }
    }

    fn update_size(&self, allocs: &TypeAllocs, tctx: &TypeContext) -> usize {
        let elem_size = self.elem_size_cache.get();
        if elem_size != 0 {
            return elem_size;
        }
        let elem_size = self.elemty.try_get_aligned_size_full(allocs, tctx);
        let Some(elem_size) = elem_size else {
            panic!(
                "Array type element should make instance but got {:?}",
                self.elemty
            );
        };
        self.elem_size_cache.set(elem_size);
        elem_size
    }

    fn update_align(&self, allocs: &TypeAllocs, tctx: &TypeContext) -> usize {
        let elem_align = self.elem_align_cache.get();
        if elem_align != 0 {
            return elem_align;
        }
        let elem_align = self.elemty.try_get_align_full(allocs, tctx);
        let Some(elem_align) = elem_align else {
            panic!(
                "Array type element should make instance but got {:?}",
                self.elemty
            );
        };
        self.elem_align_cache.set(elem_align);
        elem_align
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArrayTypeRef(pub usize);

impl SlabRef for ArrayTypeRef {
    type RefObject = ArrayTypeData;
    fn from_handle(handle: usize) -> Self {
        Self(handle)
    }
    fn get_handle(&self) -> usize {
        self.0
    }
}

impl IValType for ArrayTypeRef {
    fn try_from_ir(ty: ValTypeID) -> TypingRes<Self> {
        if let ValTypeID::Array(arr) = ty {
            Ok(arr)
        } else {
            Err(TypeMismatchError::NotClass(ty, ValTypeClass::Array))
        }
    }

    fn into_ir(self) -> ValTypeID {
        ValTypeID::Array(self)
    }

    fn makes_instance(self) -> bool {
        true
    }

    fn class_id(self) -> ValTypeClass {
        ValTypeClass::Array
    }

    fn try_get_size_full(self, alloc: &TypeAllocs, tctx: &TypeContext) -> Option<usize> {
        let data = self.to_data(&alloc.array);
        let elem_size = data.update_size(alloc, tctx);
        let elem_align = data.update_align(alloc, tctx);
        let aligned_size = elem_size.next_multiple_of(elem_align);
        let nelems = data.nelems;
        Some(aligned_size * nelems)
    }

    fn try_get_align_full(self, alloc: &TypeAllocs, tctx: &TypeContext) -> Option<usize> {
        let data = self.to_data(&alloc.array);
        let elem_align = data.update_align(alloc, tctx);
        Some(elem_align)
    }

    fn serialize<T: Write>(self, f: &TypeFormatter<T>) -> std::io::Result<()> {
        let (nelems, elemty) = {
            let data = self.to_data(&f.allocs.array);
            (data.nelems, data.elemty)
        };
        write!(f, "[ {nelems} x ")?;
        elemty.serialize(f)?;
        write!(f, " ]")
    }
}

impl ArrayTypeRef {
    pub fn get_element_type(self, tctx: &TypeContext) -> ValTypeID {
        self.to_data(&tctx.allocs.borrow().array).elemty
    }
    pub fn get_num_elements(self, tctx: &TypeContext) -> usize {
        self.to_data(&tctx.allocs.borrow().array).nelems
    }

    pub fn get_element_size(self, tctx: &TypeContext) -> usize {
        let allocs = tctx.allocs.borrow();
        let data = self.to_data(&allocs.array);
        data.update_size(&allocs, tctx)
    }
    pub fn get_element_align(self, tctx: &TypeContext) -> usize {
        let allocs = tctx.allocs.borrow();
        let data = self.to_data(&allocs.array);
        data.update_align(&allocs, tctx)
    }
    pub fn get_unit_size(self, tctx: &TypeContext) -> usize {
        let allocs = tctx.allocs.borrow();
        let data = self.to_data(&allocs.array);
        let elem_size = data.update_size(&allocs, tctx);
        let elem_align = data.update_align(&allocs, tctx);
        elem_size.next_multiple_of(elem_align)
    }

    pub fn get_offset(self, tctx: &TypeContext, index: usize) -> usize {
        index * self.get_unit_size(tctx)
    }

    pub fn new(tctx: &TypeContext, elemty: ValTypeID, nelems: usize) -> Self {
        let mut allocs = tctx.allocs.borrow_mut();
        let alloc_arr = &mut allocs.array;
        for (handle, arr) in alloc_arr.iter_mut() {
            if arr.elemty == elemty && arr.nelems == nelems {
                return ArrayTypeRef(handle);
            }
        }
        let new_arr = ArrayTypeData::new(elemty, nelems);
        let handle = alloc_arr.insert(new_arr);
        ArrayTypeRef(handle)
    }
}

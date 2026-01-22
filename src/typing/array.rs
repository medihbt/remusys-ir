use crate::{
    base::ISlabID,
    typing::{
        IValType, TypeAllocs, TypeContext, TypeFormatter, TypeMismatchErr, TypingRes, ValTypeClass,
        ValTypeID,
    },
};
use std::{
    cell::{Cell, Ref},
    io::Write,
};

#[derive(Debug, Clone)]
pub struct ArrayTypeObj {
    pub elemty: ValTypeID,
    pub nelems: usize,
    elem_size_cache: Cell<usize>,
    elem_align_cache: Cell<usize>,
}

impl ArrayTypeObj {
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
pub struct ArrayTypeID(pub u32);

impl ISlabID for ArrayTypeID {
    type RefObject = ArrayTypeObj;

    fn from_handle(handle: u32) -> Self {
        ArrayTypeID(handle)
    }

    fn into_handle(self) -> u32 {
        self.0
    }
}

impl IValType for ArrayTypeID {
    fn try_from_ir(ty: ValTypeID) -> TypingRes<Self> {
        let ValTypeID::Array(arr_id) = ty else {
            return Err(TypeMismatchErr::NotClass(ty, ValTypeClass::Array));
        };
        Ok(arr_id)
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

    fn serialize<T: Write>(self, f: &TypeFormatter<T>) -> std::io::Result<()> {
        let (nelems, elemty) = {
            let obj = self.deref(&f.allocs.arrays);
            (obj.nelems, obj.elemty)
        };
        write!(f, "[ {nelems} x ")?;
        elemty.serialize(f)?;
        write!(f, " ]")
    }

    fn try_get_size_full(self, alloc: &TypeAllocs, tctx: &TypeContext) -> Option<usize> {
        let obj = self.deref(&alloc.arrays);
        let elem_size = obj.update_size(alloc, tctx);
        Some(elem_size * obj.nelems)
    }

    fn try_get_align_full(self, alloc: &TypeAllocs, tctx: &TypeContext) -> Option<usize> {
        let obj = self.deref(&alloc.arrays);
        let elem_align = obj.update_align(alloc, tctx);
        Some(elem_align)
    }
}

impl ArrayTypeID {
    pub fn deref_ir(self, tctx: &TypeContext) -> Ref<'_, ArrayTypeObj> {
        let allocs = tctx.allocs.borrow();
        Ref::map(allocs, |a| self.deref(&a.arrays))
    }

    pub fn get_element_type(self, tctx: &TypeContext) -> ValTypeID {
        self.deref(&tctx.allocs.borrow().arrays).elemty
    }
    pub fn get_num_elements(self, tctx: &TypeContext) -> usize {
        self.deref(&tctx.allocs.borrow().arrays).nelems
    }

    pub fn get_element_size(self, tctx: &TypeContext) -> usize {
        let allocs = tctx.allocs.borrow();
        let data = self.deref(&allocs.arrays);
        data.update_size(&allocs, tctx)
    }
    pub fn get_element_align(self, tctx: &TypeContext) -> usize {
        let allocs = tctx.allocs.borrow();
        let data = self.deref(&allocs.arrays);
        data.update_align(&allocs, tctx)
    }
    pub fn get_unit_size(self, tctx: &TypeContext) -> usize {
        let allocs = tctx.allocs.borrow();
        let data = self.deref(&allocs.arrays);
        let elem_size = data.update_size(&allocs, tctx);
        let elem_align = data.update_align(&allocs, tctx);
        elem_size.next_multiple_of(elem_align)
    }

    pub fn get_offset(self, tctx: &TypeContext, index: usize) -> usize {
        index * self.get_unit_size(tctx)
    }

    pub fn new(tctx: &TypeContext, elemty: ValTypeID, nelems: usize) -> Self {
        let mut allocs = tctx.allocs.borrow_mut();
        let alloc_arr = &mut allocs.arrays;
        for (handle, arr) in alloc_arr.iter_mut() {
            if arr.elemty == elemty && arr.nelems == nelems {
                return Self(handle as u32);
            }
        }
        let new_arr = ArrayTypeObj::new(elemty, nelems);
        let handle = alloc_arr.insert(new_arr);
        ArrayTypeID(handle as u32)
    }
    /// # Safety
    ///
    /// this function does not check for duplicate types.
    pub unsafe fn new_nodedup(tctx: &TypeContext, elemty: ValTypeID, nelems: usize) -> Self {
        let mut allocs = tctx.allocs.borrow_mut();
        let alloc_arr = &mut allocs.arrays;
        let new_arr = ArrayTypeObj::new(elemty, nelems);
        let handle = alloc_arr.insert(new_arr);
        ArrayTypeID(handle as u32)
    }
}

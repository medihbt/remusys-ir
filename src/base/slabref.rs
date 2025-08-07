use slab::Slab;

use super::INullableValue;

pub trait SlabRef: Clone + Eq + INullableValue + std::fmt::Debug {
    type RefObject: Sized;

    fn from_handle(handle: usize) -> Self;
    fn get_handle(&self) -> usize;

    fn as_data<'a>(&self, slab: &'a Slab<Self::RefObject>) -> Option<&'a Self::RefObject> {
        slab.get(self.get_handle())
    }
    fn as_data_mut<'a>(
        &self,
        slab: &'a mut Slab<Self::RefObject>,
    ) -> Option<&'a mut Self::RefObject> {
        slab.get_mut(self.get_handle())
    }
    fn to_data<'a>(&self, slab: &'a Slab<Self::RefObject>) -> &'a Self::RefObject {
        if self.is_null() {
            panic!("Cannot convert null reference to data");
        }
        slab.get(self.get_handle())
            .expect(format!("Invalid reference {} (Use after free?)", self.get_handle()).as_str())
    }
    fn to_data_mut<'a>(&self, slab: &'a mut Slab<Self::RefObject>) -> &'a mut Self::RefObject {
        if self.is_null() {
            panic!("Cannot convert null reference to data");
        }
        slab.get_mut(self.get_handle())
            .expect(format!("Invalid reference {} (Use after free?)", self.get_handle()).as_str())
    }

    fn modify_slabref<'a, R>(
        &self,
        slab: &'a mut Slab<Self::RefObject>,
        modify: impl FnOnce(&mut Self::RefObject) -> R,
    ) -> Option<R> {
        if let Some(v) = slab.get_mut(self.get_handle()) { Some(modify(v)) } else { None }
    }
    fn read_slabref<'a, R>(
        &self,
        slab: &'a Slab<Self::RefObject>,
        read: impl FnOnce(&Self::RefObject) -> R,
    ) -> Option<R> {
        if let Some(v) = slab.get(self.get_handle()) { Some(read(v)) } else { None }
    }
}

impl<T: SlabRef> INullableValue for T {
    fn new_null() -> Self {
        Self::from_handle(usize::MAX)
    }
    fn is_null(&self) -> bool {
        self.get_handle() == usize::MAX
    }
}

#[macro_export]
macro_rules! impl_slabref {
    ($ref_typename:ident, $data_typename:ident) => {
        impl $crate::base::SlabRef for $ref_typename {
            type RefObject = $data_typename;

            fn from_handle(handle: usize) -> Self {
                Self(handle)
            }
            fn get_handle(&self) -> usize {
                self.0
            }
        }
    };
}

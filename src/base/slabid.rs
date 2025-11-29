use super::INullableValue;
use slab::Slab;

pub trait ISlabID: Copy + Eq {
    type RefObject: Sized;

    fn from_handle(handle: u32) -> Self;
    fn into_handle(self) -> u32;

    fn try_deref(self, slab: &Slab<Self::RefObject>) -> Option<&Self::RefObject> {
        slab.get(self.into_handle() as usize)
    }
    fn try_deref_mut(self, slab: &mut Slab<Self::RefObject>) -> Option<&mut Self::RefObject> {
        slab.get_mut(self.into_handle() as usize)
    }

    fn deref(self, slab: &Slab<Self::RefObject>) -> &Self::RefObject {
        self.try_deref(slab).expect("Tried to deref invalid SlabID")
    }
    fn deref_mut(self, slab: &mut Slab<Self::RefObject>) -> &mut Self::RefObject {
        self.try_deref_mut(slab)
            .expect("Tried to deref invalid SlabID")
    }

    fn free(self, slab: &mut Slab<Self::RefObject>) -> Option<Self::RefObject> {
        slab.try_remove(self.into_handle() as usize)
    }
}

impl<T: ISlabID> INullableValue for T {
    fn new_null() -> Self {
        T::from_handle(u32::MAX)
    }

    fn is_null(&self) -> bool {
        self.into_handle() == u32::MAX
    }
}

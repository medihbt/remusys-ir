use slab::Slab;

use super::NullableValue;

pub trait SlabRef: Clone + Eq + NullableValue + std::fmt::Debug {
    type Item: Sized;

    fn from_handle(handle: usize) -> Self;
    fn get_handle (&self) -> usize;

    fn to_slabref    <'a>(&self, slab: &'a Slab<Self::Item>)     -> Option<&'a Self::Item> {
        slab.get(self.get_handle())
    }
    fn to_slabref_mut<'a>(&self, slab: &'a mut Slab<Self::Item>) -> Option<&'a mut Self::Item> {
        slab.get_mut(self.get_handle())
    }

    fn modify_slabref<'a, R>(&self,
                              slab:   &'a mut Slab<Self::Item>,
                              modify: impl FnOnce(&mut Self::Item) -> R) -> Option<R> {
        if let Some(v) = slab.get_mut(self.get_handle()) {
            Some(modify(v))
        } else {
            None
        }
    }
    fn read_slabref<'a, R>(&self,
                            slab: &'a Slab<Self::Item>,
                            read: impl FnOnce(&Self::Item) -> R) -> Option<R> {
        if let Some(v) = slab.get(self.get_handle()) {
            Some(read(v))
        } else {
            None
        }
    }
}

impl<T: SlabRef> NullableValue for T {
    fn new_null() -> Self {
        Self::from_handle(usize::MAX)
    }
    fn is_null(&self) -> bool {
        self.get_handle() == usize::MAX
    }
}
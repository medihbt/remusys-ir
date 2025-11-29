use mtb_entity_slab::{IAllocPolicy, IndexedID};
use std::marker::PhantomData;

mod apint;
mod bitset;
mod dsu;
mod mixref;
mod slabid;
mod weak_list;

pub use {
    apint::APInt,
    bitset::{FixBitSet, FixBitSetIter},
    dsu::DSU,
    mixref::{MixMutRef, MixRef, MixRefIter},
    slabid::ISlabID,
    weak_list::*,
};

pub trait INullableValue: Copy + Eq {
    fn new_null() -> Self;
    fn is_null(&self) -> bool;

    fn is_nonnull(&self) -> bool {
        !self.is_null()
    }
    fn from_option(opt: Option<Self>) -> Self {
        opt.unwrap_or_else(Self::new_null)
    }
    fn to_option(&self) -> Option<Self> {
        if self.is_null() { None } else { Some(*self) }
    }

    fn unwrap(&self) -> Self {
        if self.is_null() { panic!("Tried to unwrap a null value") } else { *self }
    }
}

impl<E, P: IAllocPolicy> INullableValue for IndexedID<E, P> {
    fn new_null() -> Self {
        IndexedID(usize::MAX, PhantomData)
    }

    fn is_null(&self) -> bool {
        self.0 == usize::MAX
    }
}

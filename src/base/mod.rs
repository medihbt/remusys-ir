mod apint;
mod bitset;
mod dsu;
mod slablist;
mod slabref;
mod weak_list;

pub use {
    apint::APInt,
    bitset::{FixBitSet, FixBitSetIter},
    dsu::DSU,
    slablist::{
        SlabListError, SlabListIterator, SlabListNode, SlabListNodeHead, SlabListNodeRef,
        SlabListRange, SlabListRes, SlabListView, SlabRefList,
    },
    slabref::SlabRef,
    weak_list::*,
};

pub trait INullableValue: Clone + Eq {
    fn new_null() -> Self;
    fn is_null(&self) -> bool;

    fn is_nonnull(&self) -> bool {
        !self.is_null()
    }
    fn from_option(opt: Option<Self>) -> Self {
        opt.unwrap_or_else(Self::new_null)
    }
    fn to_option(&self) -> Option<Self> {
        if self.is_null() { None } else { Some(self.clone()) }
    }

    fn unwrap(&self) -> Self {
        if self.is_null() { panic!("Tried to unwrap a null value") } else { self.clone() }
    }
}

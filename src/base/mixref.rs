use std::{
    cell::{Ref, RefMut},
    fmt::Display,
    ops::{Deref, DerefMut},
};

pub enum MixRef<'a, T: ?Sized> {
    Fix(&'a T),
    Dyn(Ref<'a, T>),
}

impl<'a, T: ?Sized> Deref for MixRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}
impl<'a, T: ?Sized + Display> Display for MixRef<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.get().fmt(f)
    }
}
impl<'a, T: ?Sized> Clone for MixRef<'a, T> {
    fn clone(&self) -> Self {
        match self {
            MixRef::Fix(val) => MixRef::Fix(val),
            MixRef::Dyn(val) => MixRef::Dyn(Ref::clone(val)),
        }
    }
}
impl<'a, T> IntoIterator for &'a MixRef<'a, T>
where
    T: ?Sized + 'a,
    &'a T: IntoIterator,
{
    type Item = <&'a T as IntoIterator>::Item;
    type IntoIter = <&'a T as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.get().into_iter()
    }
}
impl<'a, T: ?Sized> MixRef<'a, T> {
    pub fn get(&self) -> &T {
        match self {
            MixRef::Fix(val) => val,
            MixRef::Dyn(val) => val.deref(),
        }
    }

    pub fn map<U: ?Sized>(self, f: impl FnOnce(&T) -> &U) -> MixRef<'a, U> {
        match self {
            MixRef::Fix(val) => MixRef::Fix(f(val)),
            MixRef::Dyn(val) => MixRef::Dyn(Ref::map(val, f)),
        }
    }
}

pub struct MixRefIter<'a, E: Clone> {
    inner: MixRef<'a, [E]>,
    index: usize,
}
impl<'a, E: Clone> Iterator for MixRefIter<'a, E> {
    type Item = E;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.inner.get().len() {
            return None;
        }
        let item = self.inner.get()[self.index].clone();
        self.index += 1;
        Some(item)
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.inner.get().len() - self.index;
        (len, Some(len))
    }
}
impl<'a, E: Clone> ExactSizeIterator for MixRefIter<'a, E> {
    fn len(&self) -> usize {
        self.inner.get().len() - self.index
    }
}
impl<'a, E: Clone> DoubleEndedIterator for MixRefIter<'a, E> {
    fn next_back(&mut self) -> Option<Self::Item> {
        if self.index >= self.inner.get().len() {
            return None;
        }
        let real_index = self.inner.get().len() - 1 - self.index;
        let item = self.inner.get()[real_index].clone();
        self.index += 1;
        Some(item)
    }
}
impl<'a, E: Clone> IntoIterator for MixRef<'a, [E]> {
    type Item = E;
    type IntoIter = MixRefIter<'a, E>;

    fn into_iter(self) -> Self::IntoIter {
        MixRefIter { inner: self, index: 0 }
    }
}

pub enum MixMutRef<'a, T: ?Sized> {
    Fix(&'a mut T),
    Dyn(RefMut<'a, T>),
}

impl<'a, T: ?Sized> Deref for MixMutRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}
impl<'a, T: ?Sized> DerefMut for MixMutRef<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_mut()
    }
}
impl<'a, T: ?Sized + Display> Display for MixMutRef<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.get().fmt(f)
    }
}
impl<'a, T: ?Sized> MixMutRef<'a, T> {
    pub fn get(&self) -> &T {
        match self {
            MixMutRef::Fix(val) => val,
            MixMutRef::Dyn(val) => val.deref(),
        }
    }
    pub fn get_mut(&mut self) -> &mut T {
        match self {
            MixMutRef::Fix(val) => val,
            MixMutRef::Dyn(val) => val.deref_mut(),
        }
    }

    pub fn map<U: ?Sized>(self, f: impl FnOnce(&mut T) -> &mut U) -> MixMutRef<'a, U> {
        match self {
            MixMutRef::Fix(val) => MixMutRef::Fix(f(val)),
            MixMutRef::Dyn(val) => MixMutRef::Dyn(RefMut::map(val, f)),
        }
    }
}

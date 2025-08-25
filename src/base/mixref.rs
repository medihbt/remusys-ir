use std::{
    cell::{Ref, RefMut},
    fmt::Display,
    ops::{Deref, DerefMut},
};

pub enum MixRef<'a, T: ?Sized + 'a> {
    Fix(&'a T),
    Dyn(Ref<'a, T>),
}

impl<'a, T: ?Sized + 'a> Deref for MixRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}
impl<'a, T: ?Sized + Display + 'a> Display for MixRef<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.get().fmt(f)
    }
}
impl<'a, T: ?Sized + 'a> Clone for MixRef<'a, T> {
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
impl<'a, T: ?Sized + 'a> MixRef<'a, T> {
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

pub enum MixMutRef<'a, T: ?Sized + 'a> {
    Fix(&'a mut T),
    Dyn(RefMut<'a, T>),
}

impl<'a, T: ?Sized + 'a> Deref for MixMutRef<'a, T> {
    type Target = T;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}
impl<'a, T: ?Sized + 'a> DerefMut for MixMutRef<'a, T> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.get_mut()
    }
}
impl<'a, T: ?Sized + Display + 'a> Display for MixMutRef<'a, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.get().fmt(f)
    }
}
impl<'a, T: ?Sized + 'a> MixMutRef<'a, T> {
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

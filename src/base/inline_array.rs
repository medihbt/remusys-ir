use std::ops::{Deref, DerefMut};

#[derive(Debug, Clone)]
pub enum InlineArray<T: Copy + Default, const N: usize> {
    Small([T; N], u32),
    Large(Box<[T]>),
}
impl<T: Copy + Default, const N: usize> Deref for InlineArray<T, N> {
    type Target = [T];

    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}
impl<T: Copy + Default, const N: usize> DerefMut for InlineArray<T, N> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.as_mut_slice()
    }
}
impl<T: Copy + Default, const N: usize> InlineArray<T, N> {
    pub fn as_slice(&self) -> &[T] {
        match self {
            InlineArray::Small(arr, len) => &arr[..*len as usize],
            InlineArray::Large(boxed) => boxed,
        }
    }
    pub fn as_mut_slice(&mut self) -> &mut [T] {
        match self {
            InlineArray::Small(arr, len) => &mut arr[..*len as usize],
            InlineArray::Large(boxed) => boxed,
        }
    }
    pub fn len(&self) -> usize {
        match self {
            InlineArray::Small(_, len) => *len as usize,
            InlineArray::Large(boxed) => boxed.len(),
        }
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn is_inlined(&self) -> bool {
        match self {
            InlineArray::Small(_, _) => true,
            InlineArray::Large(_) => false,
        }
    }

    pub fn from_slice(slice: &[T]) -> Self {
        if slice.len() <= N {
            let mut arr = [T::default(); N];
            arr[..slice.len()].copy_from_slice(slice);
            InlineArray::Small(arr, slice.len() as u32)
        } else {
            InlineArray::Large(slice.to_vec().into_boxed_slice())
        }
    }
    pub fn with_len(len: usize) -> Self {
        if len <= N {
            InlineArray::Small([T::default(); N], len as u32)
        } else {
            InlineArray::Large(vec![T::default(); len].into_boxed_slice())
        }
    }
}

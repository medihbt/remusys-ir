use crate::{
    base::ISlabID,
    typing::{
        IValType, TypeAllocs, TypeContext, TypeFormatter, TypeMismatchErr, TypingRes, ValTypeClass,
        ValTypeID,
    },
};
use std::{
    cell::{Cell, Ref},
    fmt::Write,
    hash::{DefaultHasher, Hash, Hasher},
};

#[derive(Debug, Clone)]
pub struct FuncTypeObj {
    pub is_vararg: bool,
    pub ret_type: ValTypeID,
    pub args: Box<[ValTypeID]>,
    hash_cache: Cell<usize>,
}

impl FuncTypeObj {
    pub fn new(is_vararg: bool, ret_type: ValTypeID, args: Box<[ValTypeID]>) -> Self {
        Self { is_vararg, ret_type, args, hash_cache: Cell::new(0) }
    }

    fn make_hash_and_arg_len(
        is_vararg: bool,
        ret_type: ValTypeID,
        args: impl Iterator<Item = ValTypeID>,
    ) -> (usize, usize) {
        let mut hasher = DefaultHasher::new();
        is_vararg.hash(&mut hasher);
        ret_type.hash(&mut hasher);
        let mut arg_len = 0;
        for arg in args {
            arg.hash(&mut hasher);
            arg_len += 1;
        }
        (hasher.finish() as usize, arg_len)
    }

    pub fn get_hash(&self) -> usize {
        let cached = self.hash_cache.get();
        if cached != 0 {
            return cached;
        }
        let (hash, _) =
            Self::make_hash_and_arg_len(self.is_vararg, self.ret_type, self.args.iter().copied());
        self.hash_cache.set(hash);
        hash
    }

    pub fn accepts_arg(&self, index: usize, arg_type: ValTypeID) -> bool {
        if index >= self.args.len() {
            return self.is_vararg;
        }
        self.args[index] == arg_type
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FuncTypeID(pub u32);

impl ISlabID for FuncTypeID {
    type RefObject = FuncTypeObj;

    fn from_handle(handle: u32) -> Self {
        FuncTypeID(handle)
    }
    fn into_handle(self) -> u32 {
        self.0
    }
}

impl IValType for FuncTypeID {
    fn try_from_ir(ty: ValTypeID) -> TypingRes<Self> {
        let ValTypeID::Func(f) = ty else {
            return Err(TypeMismatchErr::NotClass(ty, ValTypeClass::Func));
        };
        Ok(f)
    }

    fn into_ir(self) -> ValTypeID {
        ValTypeID::Func(self)
    }

    fn makes_instance(self) -> bool {
        false
    }

    fn class_id(self) -> ValTypeClass {
        ValTypeClass::Func
    }

    fn format_ir<T: Write>(self, f: &TypeFormatter<T>) -> std::fmt::Result {
        let func_obj = self.deref(&f.allocs.funcs);
        f.write_str("func(")?;
        for (i, arg) in func_obj.args.iter().enumerate() {
            if i > 0 {
                f.write_str(", ")?;
            }
            arg.format_ir(f)?;
        }
        if func_obj.is_vararg {
            if !func_obj.args.is_empty() {
                f.write_str(", ")?;
            }
            f.write_str("...")?;
        }
        f.write_str(") -> ")?;
        func_obj.ret_type.format_ir(f)
    }

    fn try_get_size_full(self, _: &TypeAllocs, _: &TypeContext) -> Option<usize> {
        None
    }

    fn try_get_align_full(self, _: &TypeAllocs, _: &TypeContext) -> Option<usize> {
        None
    }
}

impl FuncTypeID {
    pub fn deref_ir(self, tctx: &TypeContext) -> Ref<'_, FuncTypeObj> {
        let allocs = tctx.allocs.borrow();
        Ref::map(allocs, |allocs| self.deref(&allocs.funcs))
    }

    pub fn get_args(self, tctx: &TypeContext) -> Ref<'_, [ValTypeID]> {
        Ref::map(self.deref_ir(tctx), |func| &func.args[..])
    }
    pub fn get_nargs(self, tctx: &TypeContext) -> usize {
        self.deref_ir(tctx).args.len()
    }
    pub fn get_ret_type(self, tctx: &TypeContext) -> ValTypeID {
        self.deref_ir(tctx).ret_type
    }
    pub fn is_vararg(self, tctx: &TypeContext) -> bool {
        self.deref_ir(tctx).is_vararg
    }
    pub fn accepts_arg(self, tctx: &TypeContext, index: usize, arg_type: ValTypeID) -> bool {
        self.deref_ir(tctx).accepts_arg(index, arg_type)
    }
    pub fn get_hash(self, tctx: &TypeContext) -> usize {
        self.deref_ir(tctx).get_hash()
    }

    pub fn new<T>(tctx: &TypeContext, ret: ValTypeID, is_vararg: bool, args: T) -> Self
    where
        T: IntoIterator<Item = ValTypeID>,
        T::IntoIter: Clone,
    {
        let mut allocs = tctx.allocs.borrow_mut();
        let iter = args.into_iter();
        let (hash, arg_len) = FuncTypeObj::make_hash_and_arg_len(is_vararg, ret, iter.clone());
        let alloc_funcs = &allocs.funcs;
        for (handle, ft) in alloc_funcs {
            if ft.get_hash() != hash || ft.args.len() != arg_len {
                continue;
            }
            if ft.is_vararg != is_vararg || ft.ret_type != ret {
                continue;
            }
            if ft.args.iter().zip(iter.clone()).all(|(a, b)| *a == b) {
                return Self(handle as u32);
            }
        }
        let new_func = FuncTypeObj::new(is_vararg, ret, iter.collect());
        let handle = allocs.funcs.insert(new_func);
        Self(handle as u32)
    }
    pub fn from_arg_slice(
        tctx: &TypeContext,
        ret: ValTypeID,
        is_vararg: bool,
        args: &[ValTypeID],
    ) -> Self {
        Self::new(tctx, ret, is_vararg, args.iter().copied())
    }

    /// # Safety
    ///
    /// this function may create duplicate function types
    pub unsafe fn new_nodedup(
        tctx: &TypeContext,
        ret: ValTypeID,
        is_vararg: bool,
        args: &[ValTypeID],
    ) -> Self {
        let mut allocs = tctx.allocs.borrow_mut();
        let new_func = FuncTypeObj::new(is_vararg, ret, Box::from(args));
        let handle = allocs.funcs.insert(new_func);
        Self(handle as u32)
    }
}

use crate::{
    base::SlabRef,
    typing::{
        IValType, TypeAllocs, TypeContext, TypeFormatter, TypingRes, ValTypeClass, ValTypeID,
    },
};
use std::{
    cell::{Cell, Ref},
    hash::{Hash, Hasher},
    io::Write,
};

#[derive(Debug, Clone)]
pub struct FuncType {
    pub is_vararg: bool,
    pub ret_type: ValTypeID,
    pub args: Box<[ValTypeID]>,
    hash_cache: Cell<usize>,
}

impl FuncType {
    pub fn new(is_vararg: bool, ret_type: ValTypeID, args: Box<[ValTypeID]>) -> Self {
        Self { is_vararg, ret_type, args, hash_cache: Cell::new(0) }
    }

    fn make_hash_and_arg_len(
        is_vararg: bool,
        ret_type: ValTypeID,
        args: impl Iterator<Item = ValTypeID>,
    ) -> (usize, usize) {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        is_vararg.hash(&mut hasher);
        ret_type.hash(&mut hasher);
        let mut len = 0;
        for arg in args {
            arg.hash(&mut hasher);
            len += 1;
        }
        (hasher.finish() as usize, len)
    }

    fn get_hash(&self) -> usize {
        if self.hash_cache.get() != 0 {
            return self.hash_cache.get();
        }
        let (hash, _) =
            Self::make_hash_and_arg_len(self.is_vararg, self.ret_type, self.args.iter().cloned());
        self.hash_cache.set(hash);
        hash
    }

    pub fn accepts_arg(&self, arg_index: usize, arg_type: ValTypeID) -> bool {
        if arg_index >= self.args.len() {
            return self.is_vararg;
        }
        self.args[arg_index] == arg_type
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FuncTypeRef(pub usize);

impl SlabRef for FuncTypeRef {
    type RefObject = FuncType;
    fn from_handle(handle: usize) -> Self {
        Self(handle)
    }
    fn get_handle(&self) -> usize {
        self.0
    }
}

impl IValType for FuncTypeRef {
    fn try_from_ir(ty: ValTypeID) -> TypingRes<Self> {
        use crate::typing::TypeMismatchError::*;
        use crate::typing::ValTypeClass::*;
        match ty {
            ValTypeID::Func(func) => Ok(func),
            _ => Err(NotClass(ty, Func)),
        }
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

    /// Syntax Example:
    ///
    /// * `<ret ty> ()`
    /// * `<ret ty> (...)`
    /// * `<ret ty> (argty1)`
    /// * `<ret ty> (argty1, argty2)`
    /// * `<ret ty> (argty1, argty2, ...)`
    fn serialize<T: Write>(self, f: &TypeFormatter<T>) -> std::io::Result<()> {
        let data = self.allocs_to_data(&f.allocs);
        data.ret_type.serialize(f)?;
        f.write_str(" (")?;
        let mut count = 0;
        for arg in &data.args {
            if count > 0 {
                f.write_str(", ")?;
            }
            count += 1;
            arg.serialize(f)?;
        }
        if data.is_vararg {
            f.write_str(if count > 0 { ", ..." } else { "..." })?;
        }
        f.write_str(")")
    }

    fn try_get_size_full(self, _: &TypeAllocs, _: &TypeContext) -> Option<usize> {
        None
    }

    fn try_get_align_full(self, _: &TypeAllocs, _: &TypeContext) -> Option<usize> {
        None
    }
}

impl FuncTypeRef {
    fn allocs_to_data(self, allocs: &TypeAllocs) -> &FuncType {
        self.to_data(&allocs.funcs)
    }

    fn typectx_to_data(self, tctx: &TypeContext) -> Ref<FuncType> {
        let allocs = tctx.allocs.borrow();
        Ref::map(allocs, |allocs| self.to_data(&allocs.funcs))
    }

    pub fn ret_type(self, tctx: &TypeContext) -> ValTypeID {
        self.typectx_to_data(tctx).ret_type
    }

    pub fn args(self, tctx: &TypeContext) -> Ref<[ValTypeID]> {
        Ref::map(self.typectx_to_data(tctx), |s| &*s.args)
    }

    pub fn nargs(self, tctx: &TypeContext) -> usize {
        self.args(tctx).len()
    }

    pub fn try_get_arg(self, tctx: &TypeContext, index: usize) -> Option<ValTypeID> {
        self.args(tctx).get(index).cloned()
    }
    pub fn get_arg(self, tctx: &TypeContext, index: usize) -> ValTypeID {
        self.try_get_arg(tctx, index)
            .expect("Failed to get argument type from function type")
    }

    pub fn is_vararg(self, tctx: &TypeContext) -> bool {
        self.typectx_to_data(tctx).is_vararg
    }

    pub fn accepts_arg(self, tctx: &TypeContext, arg_index: usize, arg_type: ValTypeID) -> bool {
        self.typectx_to_data(tctx).accepts_arg(arg_index, arg_type)
    }

    pub fn new<T>(tctx: &TypeContext, ret_ty: ValTypeID, is_vararg: bool, args: T) -> FuncTypeRef
    where
        T: IntoIterator<Item = ValTypeID>,
        T::IntoIter: Clone,
    {
        let args = args.into_iter();
        let (hash, arg_len) = FuncType::make_hash_and_arg_len(is_vararg, ret_ty, args.clone());
        for (handle, func) in tctx.allocs.borrow().funcs.iter() {
            if arg_len != func.args.len() || is_vararg != func.is_vararg || hash != func.get_hash()
            {
                continue;
            }
            if func.ret_type != ret_ty {
                continue;
            }
            if func.args.iter().zip(args.clone()).all(|(a, b)| *a == b) {
                return FuncTypeRef(handle);
            }
        }

        let func = FuncType::new(is_vararg, ret_ty, args.collect());
        let mut allocs = tctx.allocs.borrow_mut();
        let handle = allocs.funcs.insert(func);
        FuncTypeRef(handle)
    }
}

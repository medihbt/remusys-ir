use crate::{
    base::{MixMutRef, MixRef},
    ir::{BlockData, ConstExprData, GlobalData, IRValueMarker, InstData, Module},
};
use slab::Slab;
use std::{
    cell::{BorrowError, BorrowMutError, Ref, RefMut},
};

#[derive(Debug)]
pub struct IRAllocs {
    pub exprs: Slab<ConstExprData>,
    pub insts: Slab<InstData>,
    pub blocks: Slab<BlockData>,
    pub globals: Slab<GlobalData>,
}

impl IRAllocs {
    pub fn new() -> Self {
        Self {
            exprs: Slab::new(),
            insts: Slab::new(),
            blocks: Slab::new(),
            globals: Slab::new(),
        }
    }

    pub fn with_capacity(base_capacity: usize) -> Self {
        Self {
            exprs: Slab::with_capacity(base_capacity * 2),
            insts: Slab::with_capacity(base_capacity * 8),
            blocks: Slab::with_capacity(base_capacity * 2),
            globals: Slab::with_capacity(base_capacity),
        }
    }

    /// 执行垃圾回收，清理未使用的 IR 对象
    ///
    /// # 参数
    /// - `roots`: 根对象集合，通常包括模块的全局变量和函数入口点
    pub fn gc_mark_sweep(&mut self, roots: impl IntoIterator<Item = crate::ir::ValueSSA>) {
        let mut marker = IRValueMarker::from_allocs(self);
        marker.mark_and_sweep(roots);
    }
}

pub type IRAllocsRef<'a> = MixRef<'a, IRAllocs>;
pub type IRAllocsMutRef<'a> = MixMutRef<'a, IRAllocs>;

#[derive(Debug)]
pub enum IRAllocsErr {
    Borrow(BorrowError),
    BorrowMut(BorrowMutError),
    Other(String),
}

pub type IRAllocsRes<T> = Result<T, IRAllocsErr>;

pub trait IRAllocsReadable<'a>: Sized + 'a {
    fn try_get_allocs_ref(self) -> IRAllocsRes<IRAllocsRef<'a>>;

    fn get_allocs_ref(self) -> IRAllocsRef<'a> {
        self.try_get_allocs_ref().unwrap()
    }
}

pub trait IRAllocsEditable<'a>: Sized + 'a {
    fn try_get_allocs_mutref(self) -> IRAllocsRes<IRAllocsMutRef<'a>>;

    fn get_allocs_mutref(self) -> IRAllocsMutRef<'a> {
        self.try_get_allocs_mutref().unwrap()
    }
}

impl<'a> IRAllocsReadable<'a> for &'a IRAllocs {
    fn try_get_allocs_ref(self) -> IRAllocsRes<IRAllocsRef<'a>> {
        Ok(IRAllocsRef::Fix(self))
    }

    fn get_allocs_ref(self) -> IRAllocsRef<'a> {
        IRAllocsRef::Fix(self)
    }
}

impl<'a> IRAllocsReadable<'a> for &'a mut IRAllocs {
    fn try_get_allocs_ref(self) -> IRAllocsRes<IRAllocsRef<'a>> {
        Ok(IRAllocsRef::Fix(self))
    }

    fn get_allocs_ref(self) -> IRAllocsRef<'a> {
        IRAllocsRef::Fix(self)
    }
}
impl<'a> IRAllocsEditable<'a> for &'a mut IRAllocs {
    fn try_get_allocs_mutref(self) -> IRAllocsRes<IRAllocsMutRef<'a>> {
        Ok(IRAllocsMutRef::Fix(self))
    }

    fn get_allocs_mutref(self) -> IRAllocsMutRef<'a> {
        IRAllocsMutRef::Fix(self)
    }
}

impl<'a> IRAllocsReadable<'a> for Ref<'a, IRAllocs> {
    fn try_get_allocs_ref(self) -> IRAllocsRes<IRAllocsRef<'a>> {
        Ok(IRAllocsRef::Dyn(self))
    }

    fn get_allocs_ref(self) -> IRAllocsRef<'a> {
        IRAllocsRef::Dyn(self)
    }
}

impl<'a> IRAllocsEditable<'a> for RefMut<'a, IRAllocs> {
    fn try_get_allocs_mutref(self) -> IRAllocsRes<IRAllocsMutRef<'a>> {
        Ok(IRAllocsMutRef::Dyn(self))
    }

    fn get_allocs_mutref(self) -> IRAllocsMutRef<'a> {
        IRAllocsMutRef::Dyn(self)
    }
}

impl<'a> IRAllocsReadable<'a> for &'a Module {
    fn try_get_allocs_ref(self) -> IRAllocsRes<IRAllocsRef<'a>> {
        self.allocs
            .try_borrow()
            .map(IRAllocsRef::Dyn)
            .map_err(IRAllocsErr::Borrow)
    }
}
impl<'a> IRAllocsEditable<'a> for &'a Module {
    fn try_get_allocs_mutref(self) -> IRAllocsRes<IRAllocsMutRef<'a>> {
        self.allocs
            .try_borrow_mut()
            .map(IRAllocsMutRef::Dyn)
            .map_err(IRAllocsErr::BorrowMut)
    }
}

impl<'a> IRAllocsReadable<'a> for &'a mut Module {
    fn try_get_allocs_ref(self) -> IRAllocsRes<IRAllocsRef<'a>> {
        Ok(IRAllocsRef::Fix(self.allocs_mut()))
    }

    fn get_allocs_ref(self) -> IRAllocsRef<'a> {
        IRAllocsRef::Fix(self.allocs_mut())
    }
}
impl<'a> IRAllocsEditable<'a> for &'a mut Module {
    fn try_get_allocs_mutref(self) -> IRAllocsRes<IRAllocsMutRef<'a>> {
        Ok(IRAllocsMutRef::Fix(self.allocs_mut()))
    }

    fn get_allocs_mutref(self) -> IRAllocsMutRef<'a> {
        IRAllocsMutRef::Fix(self.allocs_mut())
    }
}

impl<'a> IRAllocsReadable<'a> for IRAllocsRef<'a> {
    fn try_get_allocs_ref(self) -> IRAllocsRes<IRAllocsRef<'a>> {
        Ok(self)
    }

    fn get_allocs_ref(self) -> IRAllocsRef<'a> {
        self
    }
}
impl<'a> IRAllocsEditable<'a> for IRAllocsMutRef<'a> {
    fn try_get_allocs_mutref(self) -> IRAllocsRes<IRAllocsMutRef<'a>> {
        Ok(self)
    }

    fn get_allocs_mutref(self) -> IRAllocsMutRef<'a> {
        self
    }
}
impl<'a> IRAllocsReadable<'a> for &'a IRAllocsRef<'a> {
    fn try_get_allocs_ref(self) -> IRAllocsRes<IRAllocsRef<'a>> {
        Ok(IRAllocsRef::Fix(self.get()))
    }

    fn get_allocs_ref(self) -> IRAllocsRef<'a> {
        IRAllocsRef::Fix(self.get())
    }
}
impl<'a> IRAllocsReadable<'a> for &'a IRAllocsMutRef<'a> {
    fn try_get_allocs_ref(self) -> IRAllocsRes<IRAllocsRef<'a>> {
        Ok(IRAllocsRef::Fix(self.get()))
    }

    fn get_allocs_ref(self) -> IRAllocsRef<'a> {
        IRAllocsRef::Fix(self.get())
    }
}
impl<'a> IRAllocsEditable<'a> for &'a mut IRAllocsMutRef<'a> {
    fn try_get_allocs_mutref(self) -> IRAllocsRes<IRAllocsMutRef<'a>> {
        Ok(IRAllocsMutRef::Fix(self.get_mut()))
    }

    fn get_allocs_mutref(self) -> IRAllocsMutRef<'a> {
        IRAllocsMutRef::Fix(self.get_mut())
    }
}

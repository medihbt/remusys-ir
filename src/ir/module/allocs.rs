use crate::ir::{BlockData, ConstExprData, GlobalData, IRValueMarker, InstData};
use slab::Slab;
use std::{cell::Ref, ops::Deref};

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

pub enum IRAllocsRef<'a> {
    Fix(&'a IRAllocs),
    Mut(&'a mut IRAllocs),
    Dyn(Ref<'a, IRAllocs>),
}

impl<'a> IRAllocsRef<'a> {
    pub fn get(&self) -> &IRAllocs {
        match self {
            IRAllocsRef::Fix(x) => *x,
            IRAllocsRef::Mut(x) => x,
            IRAllocsRef::Dyn(x) => &*x,
        }
    }
}

impl<'a> Deref for IRAllocsRef<'a> {
    type Target = IRAllocs;

    fn deref(&self) -> &Self::Target {
        self.get()
    }
}

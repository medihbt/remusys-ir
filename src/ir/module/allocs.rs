use crate::{
    base::{MixMutRef, MixRef},
    ir::{
        AttrList, BlockData, ConstExprData, GlobalData, IRValueMarker, InstData, Module, ValueSSA,
    },
};
use slab::Slab;

#[derive(Debug)]
pub struct IRAllocs {
    pub exprs: Slab<ConstExprData>,
    pub insts: Slab<InstData>,
    pub blocks: Slab<BlockData>,
    pub globals: Slab<GlobalData>,
    pub attrs: Slab<AttrList>,
}

impl IRAllocs {
    pub fn new() -> Self {
        Self {
            exprs: Slab::new(),
            insts: Slab::new(),
            blocks: Slab::new(),
            globals: Slab::new(),
            attrs: Slab::new(),
        }
    }

    pub fn with_capacity(base_capacity: usize) -> Self {
        Self {
            exprs: Slab::with_capacity(base_capacity * 2),
            insts: Slab::with_capacity(base_capacity * 8),
            blocks: Slab::with_capacity(base_capacity * 2),
            globals: Slab::with_capacity(base_capacity),
            attrs: Slab::with_capacity(base_capacity),
        }
    }

    /// 执行垃圾回收，清理未使用的 IR 对象
    ///
    /// # 参数
    /// - `roots`: 根对象集合，通常包括模块的全局变量和函数入口点
    pub fn gc_mark_sweep(&mut self, roots: impl IntoIterator<Item = ValueSSA>) {
        let mut marker = IRValueMarker::from_allocs(self);
        marker.mark_and_sweep(roots);
    }
}

pub trait IRAllocsReadable {
    fn get_allocs_ref(&self) -> &IRAllocs;
}
pub trait IRAllocsEditable {
    fn get_allocs_mutref(&mut self) -> &mut IRAllocs;
}

impl IRAllocsReadable for IRAllocs {
    fn get_allocs_ref(&self) -> &IRAllocs {
        self
    }
}
impl IRAllocsEditable for IRAllocs {
    fn get_allocs_mutref(&mut self) -> &mut IRAllocs {
        self
    }
}

impl IRAllocsReadable for MixRef<'_, IRAllocs> {
    fn get_allocs_ref(&self) -> &IRAllocs {
        self.get()
    }
}
impl IRAllocsReadable for MixMutRef<'_, IRAllocs> {
    fn get_allocs_ref(&self) -> &IRAllocs {
        self.get()
    }
}
impl IRAllocsEditable for MixMutRef<'_, IRAllocs> {
    fn get_allocs_mutref(&mut self) -> &mut IRAllocs {
        self.get_mut()
    }
}

impl IRAllocsReadable for Module {
    fn get_allocs_ref(&self) -> &IRAllocs {
        &self.allocs
    }
}
impl IRAllocsEditable for Module {
    fn get_allocs_mutref(&mut self) -> &mut IRAllocs {
        &mut self.allocs
    }
}

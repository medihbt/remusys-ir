use crate::{
    base::SlabRef,
    ir::{
        BlockData, ConstExprData, Func, FuncRef, GlobalData, GlobalRef, ISubGlobal, InstData,
        ValueSSA,
    },
    typing::context::TypeContext,
};
use slab::Slab;
use std::{
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    ops::{ControlFlow, Deref},
    rc::Rc,
};

pub(super) mod gc;

pub struct Module {
    pub name: String,
    pub allocs: RefCell<IRAllocs>,
    pub globals: RefCell<HashMap<String, GlobalRef>>,
    pub type_ctx: Rc<TypeContext>,
}

impl Module {
    pub fn new(name: String, type_ctx: Rc<TypeContext>) -> Self {
        Self {
            name,
            allocs: RefCell::new(IRAllocs::new()),
            globals: RefCell::new(HashMap::new()),
            type_ctx,
        }
    }
    pub fn with_capacity(name: String, type_ctx: Rc<TypeContext>, base_capacity: usize) -> Self {
        Self {
            name,
            allocs: RefCell::new(IRAllocs::with_capacity(base_capacity)),
            globals: RefCell::new(HashMap::new()),
            type_ctx,
        }
    }

    pub fn borrow_allocs(&self) -> Ref<IRAllocs> {
        self.allocs.borrow()
    }
    pub fn borrow_allocs_mut(&self) -> RefMut<IRAllocs> {
        self.allocs.borrow_mut()
    }
    pub fn allocs_mut(&mut self) -> &mut IRAllocs {
        self.allocs.get_mut()
    }

    pub fn gc_mark_sweep_mut(&mut self, roots: impl IntoIterator<Item = ValueSSA>) {
        let Self { allocs, globals, .. } = self;
        let mut marker = gc::IRValueMarker::from_allocs(allocs.get_mut());
        for (_, &global) in globals.get_mut().iter() {
            marker.push_mark(global);
        }
        marker.mark_and_sweep(roots);
    }
    pub fn gc_mark_sweep(&self, roots: impl IntoIterator<Item = ValueSSA>) {
        let Self { allocs, globals, .. } = self;
        let mut allocs = allocs.borrow_mut();
        let mut marker = gc::IRValueMarker::from_allocs(&mut allocs);
        let globals = globals.borrow();
        for (_, &global) in globals.iter() {
            marker.push_mark(global);
        }
        marker.mark_and_sweep(roots);
    }

    pub fn forall_funcs(&self, has_extern: bool, f: impl FnMut(FuncRef, &Func) -> ControlFlow<()>) {
        let allocs = self.allocs.borrow();
        let globals = self.globals.borrow();
        let mut f = f;
        for (_, &global) in globals.iter() {
            let GlobalData::Func(func) = global.to_data(&allocs.globals) else {
                continue;
            };
            if !has_extern && func.is_extern() {
                continue;
            }
            if let ControlFlow::Break(()) = f(FuncRef(global), func) {
                break;
            }
        }
    }
    pub fn dump_funcs(&self, has_extern: bool) -> Vec<FuncRef> {
        let mut ret = Vec::new();
        self.forall_funcs(has_extern, |func_ref, _| {
            ret.push(func_ref);
            ControlFlow::Continue(())
        });
        ret
    }
    pub fn forall_globals(
        &self,
        has_extern: bool,
        f: impl FnMut(GlobalRef, &GlobalData) -> ControlFlow<()>,
    ) {
        let allocs = self.allocs.borrow();
        let globals = self.globals.borrow();
        let mut f = f;
        for (_, &global) in globals.iter() {
            if !has_extern && global.is_extern(&allocs) {
                continue;
            }
            if let ControlFlow::Break(()) = f(global, global.to_data(&allocs.globals)) {
                break;
            }
        }
    }
}

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
        let mut marker = gc::IRValueMarker::from_allocs(self);
        marker.mark_and_sweep(roots);
    }
}

pub enum IRAllocsRef<'a> {
    Fix(&'a IRAllocs),
    Dyn(Ref<'a, IRAllocs>),
}

impl<'a> IRAllocsRef<'a> {
    pub fn get(&self) -> &IRAllocs {
        match self {
            IRAllocsRef::Fix(x) => *x,
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

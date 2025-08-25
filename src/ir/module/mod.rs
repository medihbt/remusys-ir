use crate::{
    base::SlabRef,
    ir::{Func, FuncRef, GlobalData, GlobalRef, ISubGlobal, ValueSSA, module::allocs::IRAllocs},
    typing::TypeContext,
};
use std::{
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    ops::ControlFlow,
    rc::Rc,
};

pub(super) mod allocs;
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

use crate::{
    base::{INullableValue, SlabRef},
    ir::{
        Func, FuncRef, GlobalData, GlobalRef, IRAllocsEditable, IRAllocsReadable,
        IRValueCompactMap, IRValueMarker, ISubGlobal, ISubValueSSA, ValueSSA,
        module::allocs::IRAllocs,
    },
    typing::{ArchInfo, TypeContext},
};
use std::{
    cell::RefCell,
    collections::HashMap,
    ops::{ControlFlow, Deref, DerefMut},
    rc::Rc,
};

pub(super) mod allocs;
pub(super) mod gc;
pub(super) mod view;

pub struct Module {
    pub name: String,
    pub allocs: IRAllocs,
    pub globals: RefCell<HashMap<String, GlobalRef>>,
    pub type_ctx: Rc<TypeContext>,
}

impl Module {
    pub fn new(name: String, type_ctx: Rc<TypeContext>) -> Self {
        Self {
            name,
            allocs: IRAllocs::new(),
            globals: RefCell::new(HashMap::new()),
            type_ctx,
        }
    }
    pub fn with_capacity(name: String, type_ctx: Rc<TypeContext>, base_capacity: usize) -> Self {
        Self {
            name,
            allocs: IRAllocs::with_capacity(base_capacity),
            globals: RefCell::new(HashMap::new()),
            type_ctx,
        }
    }
    pub fn new_host_arch(name: impl Into<String>) -> Self {
        let type_ctx = TypeContext::new_rc(ArchInfo::new_host());
        Self::new(name.into(), type_ctx)
    }

    pub fn gc_cleaner(&mut self) -> IRModuleCleaner {
        IRModuleCleaner::new(self)
    }
    pub fn gc_mark_sweep(&mut self, roots: impl IntoIterator<Item = ValueSSA>) {
        self.gc_cleaner().sweep(roots);
    }
    pub fn gc_mark_compact(
        &mut self,
        roots: impl IntoIterator<Item = ValueSSA>,
    ) -> IRValueCompactMap {
        self.gc_cleaner().compact(roots)
    }

    pub fn forall_funcs(&self, has_extern: bool, f: impl FnMut(FuncRef, &Func) -> ControlFlow<()>) {
        let allocs = &self.allocs;
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
        let allocs = &self.allocs;
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

pub trait IModuleReadable: IRAllocsReadable {
    fn get_type_ctx(&self) -> &Rc<TypeContext>;
}
pub trait IModuleEditable: IModuleReadable + IRAllocsEditable {}

impl IModuleReadable for Module {
    fn get_type_ctx(&self) -> &Rc<TypeContext> {
        &self.type_ctx
    }
}
impl IModuleEditable for Module {}

/// 用于垃圾回收的代理引用. 在垃圾回收时会自动管理 Module 内部引用的数据结构.
///
/// ### 示例
///
/// ```
/// use remusys_ir::ir::Module;
/// let mut module = Module::new_host_arch("my_module");
/// /* 对 Module 进行大量操作，如生成 IR、做大量的优化, 此时产生了许多垃圾 */
/// module.gc_cleaner().sweep([]);
/// ```
pub struct IRModuleCleaner<'ir> {
    globals: &'ir mut HashMap<String, GlobalRef>,
    marker: IRValueMarker<'ir>,
}

impl<'ir> Deref for IRModuleCleaner<'ir> {
    type Target = IRValueMarker<'ir>;
    fn deref(&self) -> &Self::Target {
        &self.marker
    }
}
impl<'ir> DerefMut for IRModuleCleaner<'ir> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.marker
    }
}
impl<'ir> IRModuleCleaner<'ir> {
    pub fn new(module: &'ir mut Module) -> Self {
        let Module { allocs, globals, .. } = module;

        let globals = globals.get_mut();
        let mut marker = IRValueMarker::from_allocs(allocs);

        for (_, &global) in globals.iter() {
            marker.push_mark(global);
        }
        Self { globals, marker }
    }

    pub fn push_mark(&mut self, value: impl ISubValueSSA) -> &mut Self {
        self.marker.push_mark(value);
        self
    }
    pub fn mark_leaf(&mut self, value: impl ISubValueSSA) -> &mut Self {
        self.marker.mark_leaf(value);
        self
    }

    pub fn sweep(self, roots: impl IntoIterator<Item = ValueSSA>) {
        let Self { globals, mut marker } = self;
        marker.mark_and_sweep(roots);
        globals.retain(|_, &mut g| marker.live_set.is_live(g));
    }

    pub fn compact(self, roots: impl IntoIterator<Item = ValueSSA>) -> IRValueCompactMap {
        let Self { globals, marker } = self;
        let map = marker.mark_and_compact(roots);
        for (_, global) in globals.iter_mut() {
            let g = map.redirect_global(*global);
            assert!(g.is_nonnull(), "global should not be null");
            *global = g;
        }
        map
    }
}

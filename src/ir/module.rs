use crate::{
    ir::{
        FuncID, GlobalID, GlobalObj, GlobalVarID, IRAllocs, IRMarker, ISubGlobal, ISubGlobalID,
        utils::module_clone::ModuleClone,
    },
    typing::{ArchInfo, TypeContext},
};
#[cfg(not(target_arch = "wasm32"))]
use std::sync::Mutex;
use std::{
    borrow::Borrow,
    cell::RefCell,
    collections::{HashMap, HashSet},
    rc::Rc,
    sync::Arc,
};

pub mod allocs;
pub mod gc;
pub mod managing;

pub struct SymbolPool {
    pub(super) exported: HashMap<Arc<str>, GlobalID>,
    pub(super) func_pool: HashSet<FuncID>,
    pub(super) var_pool: HashSet<GlobalVarID>,
}
impl SymbolPool {
    fn new() -> RefCell<Self> {
        RefCell::new(Self {
            exported: HashMap::new(),
            func_pool: HashSet::new(),
            var_pool: HashSet::new(),
        })
    }
    pub fn func_pool(&self) -> &HashSet<FuncID> {
        &self.func_pool
    }
    pub fn var_pool(&self) -> &HashSet<GlobalVarID> {
        &self.var_pool
    }
    pub(super) fn pool_add(&mut self, allocs: &IRAllocs, id: GlobalID) -> bool {
        match id.deref_ir(allocs) {
            GlobalObj::Func(_) => self.func_pool.insert(FuncID::raw_from(id)),
            GlobalObj::Var(_) => self.var_pool.insert(GlobalVarID::raw_from(id)),
        }
    }
    pub fn get_symbol_by_name(&self, name: impl Borrow<str>) -> Option<GlobalID> {
        self.exported.get(name.borrow()).copied()
    }
    /// Return true if the given GlobalID is present in the exported symbol table.
    pub fn is_id_exported(&self, id: GlobalID) -> bool {
        self.exported.values().any(|&v| v == id)
    }
    pub fn symbol_pinned(&self, id: GlobalID, allocs: &IRAllocs) -> bool {
        match id.deref_ir(allocs) {
            GlobalObj::Func(_) => self.func_pool.contains(&FuncID::raw_from(id)),
            GlobalObj::Var(_) => self.var_pool.contains(&GlobalVarID::raw_from(id)),
        }
    }
    pub fn unpin_symbol(&mut self, id: GlobalID, allocs: &IRAllocs) -> bool {
        self.exported.remove(id.get_name(allocs));
        match id.deref_ir(allocs) {
            GlobalObj::Func(_) => self.func_pool.remove(&FuncID::raw_from(id)),
            GlobalObj::Var(_) => self.var_pool.remove(&GlobalVarID::raw_from(id)),
        }
    }

    pub(super) fn try_export_symbol(
        &mut self,
        id: GlobalID,
        allocs: &IRAllocs,
    ) -> Result<GlobalID, GlobalID> {
        use std::collections::hash_map::Entry;
        let obj = id.deref_ir(allocs);
        debug_assert!(self.symbol_pinned(id, allocs), "Exporting unpinned symbol");
        let name_arc = obj.name_arc();
        match self.exported.entry(name_arc) {
            Entry::Occupied(existed) => Err(*existed.get()),
            Entry::Vacant(v) => {
                v.insert(id);
                Ok(id)
            }
        }
    }

    fn gc_mark(&self, marker: &mut IRMarker<'_>) {
        if cfg!(debug_assertions) {
            for id in self.exported.values() {
                debug_assert!(
                    self.symbol_pinned(*id, marker.ir_allocs),
                    "Symbol table contains unpinned symbol during GC marking"
                );
            }
        }
        for &fid in &self.func_pool {
            marker.push_mark(fid.raw_into());
        }
        for &gid in &self.var_pool {
            marker.push_mark(gid.raw_into());
        }
    }
}

pub struct Module {
    pub allocs: IRAllocs,
    pub tctx: TypeContext,
    pub symbols: RefCell<SymbolPool>,
    pub name: String,
}

impl AsRef<IRAllocs> for Module {
    fn as_ref(&self) -> &IRAllocs {
        &self.allocs
    }
}
impl AsRef<Module> for Module {
    fn as_ref(&self) -> &Module {
        self
    }
}
impl AsMut<IRAllocs> for Module {
    fn as_mut(&mut self) -> &mut IRAllocs {
        &mut self.allocs
    }
}
impl AsMut<Module> for Module {
    fn as_mut(&mut self) -> &mut Module {
        self
    }
}
impl Clone for Module {
    fn clone(&self) -> Self {
        ModuleClone::new(self).clone_and_release()
    }
}

impl Module {
    pub fn new(arch: ArchInfo, name: impl Into<String>) -> Self {
        Self {
            allocs: IRAllocs::new(),
            tctx: TypeContext::new(arch),
            symbols: SymbolPool::new(),
            name: name.into(),
        }
    }
    #[inline(never)]
    #[cfg(not(target_arch = "wasm32"))]
    pub fn shared_new(arch: ArchInfo, name: impl Into<String>) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self::new(arch, name)))
    }
    #[inline(never)]
    pub fn new_rc(arch: ArchInfo, name: impl Into<String>) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self::new(arch, name)))
    }

    pub fn with_capacity(arch: ArchInfo, name: impl Into<String>, base_cap: usize) -> Self {
        Self {
            allocs: IRAllocs::with_capacity(base_cap),
            tctx: TypeContext::new(arch),
            symbols: SymbolPool::new(),
            name: name.into(),
        }
    }
    #[inline(never)]
    #[cfg(not(target_arch = "wasm32"))]
    pub fn shared_with_capacity(
        arch: ArchInfo,
        name: impl Into<String>,
        base_cap: usize,
    ) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self::with_capacity(arch, name, base_cap)))
    }
    #[inline(never)]
    pub fn with_capacity_rc(
        arch: ArchInfo,
        name: impl Into<String>,
        base_cap: usize,
    ) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self::with_capacity(arch, name, base_cap)))
    }

    pub fn get_global_by_name(&self, name: &str) -> Option<GlobalID> {
        self.symbols.borrow().get_symbol_by_name(name)
    }

    pub fn symbol_pinned(&self, id: GlobalID) -> bool {
        self.symbols.borrow().symbol_pinned(id, &self.allocs)
    }
    pub fn symbol_pinned_by_name(&self, name: &str) -> bool {
        let id = match self.get_global_by_name(name) {
            Some(id) => id,
            None => return false,
        };
        assert!(
            self.symbol_pinned(id),
            "Internal error: symbol not pinned but found in symbol table"
        );
        true
    }
    pub fn unpin_symbol(&self, id: GlobalID) -> bool {
        let mut symbols = match self.symbols.try_borrow_mut() {
            Ok(s) => s,
            Err(e) => panic!("Are you trying to unpin a symbol while traversing symtab? {e}"),
        };
        symbols.unpin_symbol(id, &self.allocs)
    }
    pub fn unpin_symbol_by_name(&self, name: &str) -> Option<GlobalID> {
        let id = self.get_global_by_name(name)?;
        self.unpin_symbol(id);
        Some(id)
    }

    /// List all pinned global IDs that are not exported.
    /// Intended to be used by lowering to ensure emission completeness.
    pub fn list_unexported_pinned(&self) -> Vec<GlobalID> {
        let symbols = self.symbols.borrow();
        let mut missing = Vec::new();
        // Functions
        for &fid in symbols.func_pool.iter() {
            let gid = fid.raw_into();
            if !symbols.is_id_exported(gid) {
                missing.push(gid);
            }
        }
        // Global variables
        for &gid_t in symbols.var_pool.iter() {
            let gid = gid_t.raw_into();
            if !symbols.is_id_exported(gid) {
                missing.push(gid);
            }
        }
        missing
    }

    /// Check whether all pinned globals are exported.
    /// Returns Ok(()) if satisfied, or Err(Vec<GlobalID>) with the list of missing ones.
    pub fn check_all_pinned_exported(&self) -> Result<(), Vec<GlobalID>> {
        let missing = self.list_unexported_pinned();
        if missing.is_empty() { Ok(()) } else { Err(missing) }
    }

    /// Begin a garbage collection cycle.
    /// This will free disposed allocations and return an IRMarker to mark live allocations.
    pub fn begin_gc(&mut self) -> IRMarker<'_> {
        self.allocs.free_disposed();
        let Self { allocs, symbols, .. } = self;
        let mut marker = IRMarker::new(allocs);
        symbols.get_mut().gc_mark(&mut marker);
        marker
    }

    /// Free disposed allocations without starting a GC cycle.
    pub fn free_disposed(&mut self) {
        self.allocs.free_disposed();
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ir::{IRWriteOption, IRWriter},
        testing::cases::test_case_cfg_deep_while_br,
    };

    #[test]
    fn test_thread_safety() {
        fn assert_send<T: Send>() {}
        assert_send::<Module>();
        // Module is Send and not Sync
    }

    #[test]
    fn test_gc() {
        let mut module = test_case_cfg_deep_while_br().module;
        module.begin_gc().finish();
        write_module(&module, "target/test_output_gc.ll");
    }

    fn write_module(module: &Module, path: &str) {
        let file = std::fs::File::create(path).expect("Failed to create output file");
        let mut file_writer = std::io::BufWriter::new(file);
        let mut writer = IRWriter::from_module(&mut file_writer, module);
        writer.option = IRWriteOption::loud();
        writer.write_module();
    }
}

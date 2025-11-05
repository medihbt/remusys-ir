use crate::{
    ir::{GlobalID, IRAllocs, IRMarker},
    typing::{ArchInfo, TypeContext},
};
#[cfg(not(target_arch = "wasm32"))]
use std::sync::Mutex;
use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::Arc};

pub mod allocs;
pub mod gc;
pub mod managing;

pub struct Module {
    pub allocs: IRAllocs,
    pub tctx: TypeContext,
    pub symbols: RefCell<HashMap<Arc<str>, GlobalID>>,
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

impl Module {
    pub fn new(arch: ArchInfo, name: impl Into<String>) -> Self {
        Self {
            allocs: IRAllocs::new(),
            tctx: TypeContext::new(arch),
            symbols: RefCell::new(HashMap::new()),
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
            symbols: RefCell::new(HashMap::new()),
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
        self.symbols.borrow().get(name).copied()
    }

    /// Begin a garbage collection cycle.
    /// This will free disposed allocations and return an IRMarker to mark live allocations.
    pub fn begin_gc(&mut self) -> IRMarker<'_> {
        self.allocs.free_disposed();
        let Self { allocs, symbols, .. } = self;
        let mut marker = IRMarker::new(allocs);
        for (_, &gid) in symbols.get_mut().iter() {
            marker.push_mark(gid);
        }
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

use crate::{
    ir::{GlobalID, IRAllocs},
    typing::{ArchInfo, TypeContext},
};
#[cfg(not(target_arch = "wasm32"))]
use std::sync::Mutex;
use std::{cell::RefCell, collections::HashMap, rc::Rc, sync::Arc};

pub mod allocs;
pub mod gc;

pub struct Module {
    pub allocs: IRAllocs,
    pub tctx: TypeContext,
    pub symbols: RefCell<HashMap<Arc<str>, GlobalID>>,
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
    pub fn new(arch: ArchInfo) -> Self {
        Self {
            allocs: IRAllocs::new(),
            tctx: TypeContext::new(arch),
            symbols: RefCell::new(HashMap::new()),
        }
    }
    #[inline(never)]
    #[cfg(not(target_arch = "wasm32"))]
    pub fn shared_new(arch: ArchInfo) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self::new(arch)))
    }
    #[inline(never)]
    pub fn new_rc(arch: ArchInfo) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self::new(arch)))
    }

    pub fn with_capacity(arch: ArchInfo, base_cap: usize) -> Self {
        Self {
            allocs: IRAllocs::with_capacity(base_cap),
            tctx: TypeContext::new(arch),
            symbols: RefCell::new(HashMap::new()),
        }
    }
    #[inline(never)]
    #[cfg(not(target_arch = "wasm32"))]
    pub fn shared_with_capacity(arch: ArchInfo, base_cap: usize) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self::with_capacity(arch, base_cap)))
    }
    #[inline(never)]
    pub fn with_capacity_rc(arch: ArchInfo, base_cap: usize) -> Rc<RefCell<Self>> {
        Rc::new(RefCell::new(Self::with_capacity(arch, base_cap)))
    }

    pub fn get_global_by_name(&self, name: &str) -> Option<GlobalID> {
        self.symbols.borrow().get(name).copied()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_thread_safety() {
        fn assert_send<T: Send>() {}
        assert_send::<Module>();
        // Module is Send and not Sync
    }
}

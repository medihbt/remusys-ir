use crate::{
    ir::{GlobalID, IRAllocs},
    typing::{ArchInfo, TypeContext},
};
use std::{
    cell::RefCell,
    collections::HashMap,
    sync::{Arc, Mutex},
};

pub mod allocs;

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
    pub fn shared_new(arch: ArchInfo) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self::new(arch)))
    }

    pub fn with_capacity(arch: ArchInfo, base_cap: usize) -> Self {
        Self {
            allocs: allocs::IRAllocs::with_capacity(base_cap),
            tctx: TypeContext::new(arch),
            symbols: RefCell::new(HashMap::new()),
        }
    }
    #[inline(never)]
    pub fn shared_with_capacity(arch: ArchInfo, base_cap: usize) -> Arc<Mutex<Self>> {
        Arc::new(Mutex::new(Self::with_capacity(arch, base_cap)))
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

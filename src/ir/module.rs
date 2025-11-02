use crate::{
    ir::GlobalID,
    typing::{ArchInfo, TypeContext},
};
use std::{cell::RefCell, collections::HashMap, sync::Arc};

pub mod allocs;

pub struct Module {
    pub allocs: allocs::IRAllocs,
    pub tctx: TypeContext,
    pub symbols: RefCell<HashMap<Arc<str>, GlobalID>>,
}

impl AsRef<allocs::IRAllocs> for Module {
    fn as_ref(&self) -> &allocs::IRAllocs {
        &self.allocs
    }
}
impl AsMut<allocs::IRAllocs> for Module {
    fn as_mut(&mut self) -> &mut allocs::IRAllocs {
        &mut self.allocs
    }
}

impl Module {
    pub fn new(arch: ArchInfo) -> Self {
        Self {
            allocs: allocs::IRAllocs::new(),
            tctx: TypeContext::new(arch),
            symbols: RefCell::new(HashMap::new()),
        }
    }

    pub fn with_capacity(arch: ArchInfo, base_cap: usize) -> Self {
        Self {
            allocs: allocs::IRAllocs::with_capacity(base_cap),
            tctx: TypeContext::new(arch),
            symbols: RefCell::new(HashMap::new()),
        }
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

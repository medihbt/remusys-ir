use crate::{
    base::SlabRef,
    typing::{
        ArrayTypeData, FuncType, StructAliasData, StructAliasRef, StructTypeData, StructTypeRef,
    },
};
use slab::Slab;
use std::{cell::RefCell, collections::HashMap, rc::Rc};

#[derive(Debug, Clone)]
pub struct ArchInfo {
    pub ptr_nbits: usize,
    pub reg_nbits: usize,
}

impl ArchInfo {
    pub fn new_host() -> Self {
        use core::mem::size_of;
        Self {
            ptr_nbits: size_of::<*const usize>() * 8,
            reg_nbits: size_of::<usize>() * 8,
        }
    }
}

#[derive(Debug, Clone)]
pub struct TypeAllocs {
    pub array: Slab<ArrayTypeData>,
    pub structs: Slab<StructTypeData>,
    pub aliases: Slab<StructAliasData>,
    pub funcs: Slab<FuncType>,
}

impl TypeAllocs {
    pub fn new() -> Self {
        Self {
            array: Slab::new(),
            structs: Slab::new(),
            aliases: Slab::new(),
            funcs: Slab::new(),
        }
    }
}

pub struct TypeContext {
    pub arch: ArchInfo,
    pub allocs: RefCell<TypeAllocs>,
    alias_map: RefCell<HashMap<String, StructAliasRef>>,
}

impl TypeContext {
    pub fn new(arch: ArchInfo) -> Self {
        Self {
            arch,
            allocs: RefCell::new(TypeAllocs::new()),
            alias_map: RefCell::new(HashMap::new()),
        }
    }

    pub fn new_rc(arch: ArchInfo) -> Rc<Self> {
        Rc::new(Self::new(arch))
    }

    pub fn try_get_alias(&self, name: &str) -> Option<StructTypeRef> {
        let sa = self.alias_map.borrow().get(name).cloned();
        sa.map(|alias| alias.get_aliasee(self))
    }
    pub fn get_alias(&self, name: &str) -> StructTypeRef {
        self.try_get_alias(name)
            .expect("Failed to get struct alias from type context")
    }

    pub fn set_alias(&self, name: impl Into<String>, aliasee: StructTypeRef) {
        let name: String = name.into();
        let mut aliases = self.alias_map.borrow_mut();
        let mut allocs = self.allocs.borrow_mut();
        if let Some(existing) = aliases.get(&name) {
            let alias_data = existing.to_data_mut(&mut allocs.aliases);
            alias_data.aliasee = aliasee;
        } else {
            let new_alias = StructAliasData { name: name.clone(), aliasee };
            let alias_ref = StructAliasRef(allocs.aliases.insert(new_alias));
            aliases.insert(name, alias_ref);
        }
    }

    pub fn read_struct_aliases(&self, mut reader: impl FnMut(&str, StructTypeRef)) {
        let aliases = self.alias_map.borrow();
        for (name, alias_ref) in aliases.iter() {
            let aliasee = alias_ref.get_aliasee(self);
            reader(name, aliasee);
        }
    }
}

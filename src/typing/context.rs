use crate::{
    base::ISlabID,
    typing::{
        ArrayTypeObj, FuncTypeObj, IntType, StructAliasID, StructAliasObj, StructTypeID,
        StructTypeObj,
    },
};
use slab::Slab;
use std::{cell::RefCell, collections::HashMap};

#[derive(Debug, Clone)]
pub struct ArchInfo {
    pub ptr_nbits: u32,
    pub reg_nbits: u32,
}

impl ArchInfo {
    pub fn get_intptr_type(&self) -> IntType {
        IntType(self.ptr_nbits as u8)
    }
    pub fn new_host() -> Self {
        let ptr_nbits = std::mem::size_of::<usize>() as u32 * 8;
        Self { ptr_nbits, reg_nbits: ptr_nbits }
    }
}

#[derive(Debug, Clone)]
pub struct TypeAllocs {
    pub arrays: Slab<ArrayTypeObj>,
    pub structs: Slab<StructTypeObj>,
    pub aliases: Slab<StructAliasObj>,
    pub funcs: Slab<FuncTypeObj>,
}

impl TypeAllocs {
    pub fn new() -> Self {
        Self {
            arrays: Slab::new(),
            structs: Slab::new(),
            aliases: Slab::new(),
            funcs: Slab::new(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct TypeContext {
    pub arch: ArchInfo,
    pub allocs: RefCell<TypeAllocs>,
    alias_map: RefCell<HashMap<String, StructAliasID>>,
}

impl TypeContext {
    pub fn new(arch: ArchInfo) -> Self {
        Self {
            arch,
            allocs: RefCell::new(TypeAllocs::new()),
            alias_map: RefCell::new(HashMap::new()),
        }
    }

    pub fn try_get_alias(&self, name: &str) -> Option<StructTypeID> {
        let alias_id = *self.alias_map.borrow().get(name)?;
        let aliasee = alias_id.deref(&self.allocs.borrow().aliases).aliasee;
        Some(aliasee)
    }
    pub fn get_alias(&self, name: &str) -> StructTypeID {
        if let Some(ty) = self.try_get_alias(name) {
            return ty;
        }
        panic!("Alias %{name} not found");
    }

    pub fn set_alias(&self, name: impl Into<String>, aliasee: StructTypeID) -> StructAliasID {
        let name = name.into();
        if let Some(existing) = self.alias_map.borrow().get(&name) {
            return *existing;
        }
        let alias_obj = StructAliasObj { name: name.clone(), aliasee };
        let mut allocs = self.allocs.borrow_mut();
        let alias_id = StructAliasID(allocs.aliases.insert(alias_obj) as u32);
        self.alias_map.borrow_mut().insert(name, alias_id);
        alias_id
    }

    pub fn foreach_aliases(&self, mut f: impl FnMut(&String, StructAliasID, StructTypeID)) {
        for (name, &alias_id) in self.alias_map.borrow().iter() {
            let aliasee = alias_id.deref(&self.allocs.borrow().aliases).aliasee;
            f(name, alias_id, aliasee);
        }
    }
}

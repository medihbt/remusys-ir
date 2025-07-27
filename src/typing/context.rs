use std::{cell::RefCell, collections::HashMap, rc::Rc};

use slab::Slab;

use crate::base::slabref::SlabRef;

use super::{
    IValType,
    id::ValTypeID,
    types::{
        ArrayTypeData, ArrayTypeRef, FuncTypeData, FuncTypeRef, StructAliasData, StructAliasRef,
        StructTypeData, StructTypeRef,
    },
};

#[derive(Debug, Clone)]
pub struct PlatformPolicy {
    pub ptr_nbits: usize,
    pub reg_nbits: usize,
}

impl PlatformPolicy {
    pub fn new_host() -> Self {
        Self {
            ptr_nbits: core::mem::size_of::<usize>() * 8,
            reg_nbits: core::mem::size_of::<usize>() * 8,
        }
    }
}

pub struct TypeContext {
    pub platform_policy: PlatformPolicy,
    pub(super) _inner: RefCell<TypeContextInner>,
    pub(super) _struct_alias_map: RefCell<HashMap<String, StructAliasRef>>,
}

pub(super) struct TypeContextInner {
    pub(super) _alloc_array: Slab<ArrayTypeData>,
    pub(super) _alloc_struct: Slab<StructTypeData>,
    pub(super) _alloc_struct_alias: Slab<StructAliasData>,
    pub(super) _alloc_func: Slab<FuncTypeData>,
}

impl TypeContext {
    pub fn new(platform: PlatformPolicy) -> Self {
        Self {
            platform_policy: platform,
            _inner: RefCell::new(TypeContextInner {
                _alloc_array: Slab::new(),
                _alloc_struct: Slab::new(),
                _alloc_func: Slab::new(),
                _alloc_struct_alias: Slab::new(),
            }),
            _struct_alias_map: RefCell::new(HashMap::new()),
        }
    }
    pub fn new_rc(platform: PlatformPolicy) -> Rc<Self> {
        Rc::new(Self::new(platform))
    }

    pub fn get_array_type(&self, arrty: ArrayTypeData) -> ArrayTypeRef {
        let option_array = self
            ._inner
            .borrow()
            ._alloc_array
            .iter()
            .find(|(_, arr)| arr.length == arrty.length && arr.elemty == arrty.elemty)
            .map(|(idx, _)| idx);

        match option_array {
            Some(index) => ArrayTypeRef::from_handle(index),
            None => {
                let index = self._inner.borrow_mut()._alloc_array.insert(arrty);
                ArrayTypeRef::from_handle(index)
            }
        }
    }
    pub fn make_array_type(&self, length: usize, elemty: ValTypeID) -> ArrayTypeRef {
        self.get_array_type(ArrayTypeData { length, elemty })
    }

    pub fn get_struct_type(&self, struct_ty: StructTypeData) -> StructTypeRef {
        let option_struct = self
            ._inner
            .borrow()
            ._alloc_struct
            .iter()
            .find(|(_, st)| st.deep_eq(&struct_ty))
            .map(|(idx, _)| idx);

        match option_struct {
            Some(index) => StructTypeRef::from_handle(index),
            None => {
                let index = self._inner.borrow_mut()._alloc_struct.insert(struct_ty);
                StructTypeRef::from_handle(index)
            }
        }
    }
    pub fn make_struct_type(&self, elems: &[ValTypeID]) -> StructTypeRef {
        self.get_struct_type(StructTypeData {
            elemty: Box::from(elems),
        })
    }

    pub fn read_struct_aliases(&self, mut reader: impl FnMut(&str, StructTypeRef)) {
        let inner = self._inner.borrow();
        for (_, alias) in inner._alloc_struct_alias.iter() {
            let name = alias.name.clone();
            let aliasee = alias.aliasee.clone();
            reader(name.as_str(), aliasee);
        }
    }
    pub fn get_struct_alias_by_name(&self, name: &str) -> Option<StructAliasRef> {
        self._struct_alias_map
            .borrow()
            .get(name)
            .map(|sa| sa.clone())
    }
    pub fn make_struct_alias_lazy(&self, name: &str, aliasee: StructTypeRef) -> StructAliasRef {
        if let Some(alias) = self.get_struct_alias_by_name(name) {
            alias
        } else {
            self._force_insert_struct_alias(name, aliasee)
        }
    }
    pub fn make_struct_alias_force(&self, name: &str, aliasee: StructTypeRef) -> StructAliasRef {
        if let Some(alias) = self.get_struct_alias_by_name(name) {
            if alias
                .to_slabref_unwrap(&self._inner.borrow()._alloc_struct_alias)
                .aliasee
                .eq(&aliasee)
            {
                return alias;
            }
        }
        self._force_insert_struct_alias(name, aliasee)
    }
    fn _force_insert_struct_alias(&self, name: &str, aliasee: StructTypeRef) -> StructAliasRef {
        let handle = self
            ._inner
            .borrow_mut()
            ._alloc_struct_alias
            .insert(StructAliasData {
                name: name.to_string(),
                aliasee,
            });
        let ret = StructAliasRef::from_handle(handle);
        self._struct_alias_map
            .borrow_mut()
            .insert(name.to_string(), ret.clone());
        ret
    }

    pub fn get_func_type(&self, functy: FuncTypeData) -> FuncTypeRef {
        let option_func = self
            ._inner
            .borrow()
            ._alloc_func
            .iter()
            .find(|(_, func)| func.deep_eq(&functy))
            .map(|(idx, _)| idx);

        match option_func {
            Some(index) => FuncTypeRef::from_handle(index),
            None => {
                let index = self._inner.borrow_mut()._alloc_func.insert(functy);
                FuncTypeRef::from_handle(index)
            }
        }
    }
    pub fn make_func_type(
        &self,
        argtys: &[ValTypeID],
        ret_ty: ValTypeID,
        is_vararg: bool,
    ) -> FuncTypeRef {
        self.get_func_type(FuncTypeData {
            args: Box::from(argtys),
            ret_ty,
            is_vararg,
        })
    }
}

pub(super) const fn binary_bits_to_bytes(binary_bits: usize) -> usize {
    (binary_bits - 1) / 8 + 1
}

use std::{cell::{Ref, RefCell}, collections::HashMap, rc::{Rc, Weak}};

use super::{
    subtypes::{FloatTypeKind, IntType, StructAliasType},
    id::{ValTypeID, ValTypeUnion}
};

#[derive(Debug)]
pub struct TypeContextInner {
    _type_storage: slab::Slab<ValTypeUnion>,
    _struct_alias: HashMap<String, ValTypeID>,
    _int_types:    [Option<ValTypeID>; u8::MAX as usize],
    _float_types:  [Option<ValTypeID>; FloatTypeKind::NELEMS],
    _void_type:    ValTypeID,
    _ptr_type:     ValTypeID,
}

#[derive(Debug)]
pub struct TypeContext {
    pub inner: RefCell<TypeContextInner>,
}

impl TypeContextInner {
    pub fn find_type<'a>(&'a self, vtyid: &ValTypeID) -> Option<&'a ValTypeUnion> {
        self._type_storage.get(vtyid.handle())
    }
    pub fn find_struct_alias(&self, name: &str) -> Option<&ValTypeID> {
        self._struct_alias.get(name)
    }

    pub fn get_ptr_type(&self) -> ValTypeID {
        self._ptr_type.clone()
    }
    pub fn get_void_type(&self) -> ValTypeID {
        self._void_type.clone()
    }

    pub fn intptr_size(&self) -> usize {
        std::mem::size_of::<usize>()
    }
    pub fn register_size(&self) -> usize {
        std::mem::size_of::<usize>()
    }

    fn _try_find_unique_type_index(&self, vtyid: &ValTypeUnion) -> Option<usize>
    {
        for (idx, vty) in self._type_storage.iter() {
            if vtyid.deep_eq(vty) {
                return Some(idx);
            }
        }
        None
    }
    fn _find_or_register_type(&mut self, vty: ValTypeUnion) -> usize
    {
        match self._try_find_unique_type_index(&vty) {
            Some(idx) => idx,
            None => self._type_storage.insert(vty)
        }
    }

    fn _get_int_type(&mut self, weak_ref: Weak<TypeContext>, binary_bits: u8) -> ValTypeID
    {
        if let Some(vtyid) = self._int_types[binary_bits as usize].clone() {
            return vtyid;
        }
        let vty = ValTypeUnion::Int(IntType { bin_bits: binary_bits });
        let idx = self._find_or_register_type(vty);
        let vtyid = ValTypeID(idx, weak_ref);
        self._int_types[binary_bits as usize] = Some(vtyid.clone());
        vtyid
    }
    fn _get_float_type(&mut self, weak_ref: Weak<TypeContext>, kind: FloatTypeKind) -> ValTypeID
    {
        if let Some(vtyid) = self._float_types[kind as usize].clone() {
            return vtyid;
        }
        let vty = ValTypeUnion::Float(kind);
        let idx = self._find_or_register_type(vty);
        let vtyid = ValTypeID(idx, weak_ref);
        self._float_types[kind as usize] = Some(vtyid.clone());
        vtyid
    }
    fn _get_struct_alias(&mut self, weak_ref: Weak<TypeContext>, name: String, aliasee: ValTypeID) -> ValTypeID
    {
        if !matches!(self.find_type(&aliasee), Some(ValTypeUnion::Struct(_))) {
            panic!("aliasee must be a struct type");
        }
        if let Some(vtyid) = self._struct_alias.get(&name).cloned() {
            return vtyid;
        }
        let vty = ValTypeUnion::StructAlias(StructAliasType{
            name:    name.clone(),
            aliasee: aliasee.clone(),
        });
        let idx = self._find_or_register_type(vty);
        let vtyid = ValTypeID(idx, weak_ref);
        self._struct_alias.insert(name, vtyid.clone());
        vtyid
    }
}

impl TypeContext {
    pub fn new() -> Rc<Self> {
        let mut type_storage = slab::Slab::with_capacity(1024);
        let void_tyid = ValTypeID(type_storage.insert(ValTypeUnion::Void), Weak::new());
        let ptr_tyid  = ValTypeID(type_storage.insert(ValTypeUnion::Ptr), Weak::new());
        let ret = Rc::new(Self {
            inner: RefCell::new(TypeContextInner {
                _type_storage: type_storage,
                _struct_alias: HashMap::new(),
                _int_types:    [const { None }; u8::MAX as usize],
                _float_types:  [const { None }; FloatTypeKind::NELEMS],
                _void_type:    void_tyid,
                _ptr_type:     ptr_tyid,
            }),
        });

        let weak_ret = Rc::downgrade(&ret);
        ret.borrow_mut()._void_type.1 = weak_ret.clone();
        ret.borrow_mut()._ptr_type.1  = weak_ret.clone();
        ret
    }

    pub fn borrow(&self) -> Ref<TypeContextInner> {
        self.inner.borrow()
    }
    pub fn borrow_mut(&self) -> std::cell::RefMut<TypeContextInner> {
        self.inner.borrow_mut()
    }

    pub fn get_int_type(self: &Rc<Self>, binary_bits: u8) -> ValTypeID {
        self.borrow_mut()._get_int_type(Rc::downgrade(self), binary_bits)
    }
    pub fn get_float_type(self: &Rc<Self>, kind: FloatTypeKind) -> ValTypeID {
        self.borrow_mut()._get_float_type(Rc::downgrade(self), kind)
    }
    pub fn get_void_type(self: &Rc<Self>) -> ValTypeID {
        self.borrow().get_void_type()
    }
    pub fn get_ptr_type(self: &Rc<Self>) -> ValTypeID {
        self.borrow().get_ptr_type()
    }
    pub fn reg_get_type(self: &Rc<Self>, vty: ValTypeUnion) -> ValTypeID {
        let idx = self.borrow_mut()._find_or_register_type(vty);
        ValTypeID(idx, Rc::downgrade(self))
    }
    pub fn reg_get_struct_alias(self: &Rc<Self>, name: String, aliasee: ValTypeID) -> ValTypeID {
        self.borrow_mut()._get_struct_alias(Rc::downgrade(self), name, aliasee)
    }
}

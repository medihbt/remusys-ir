use crate::{
    SymbolStr,
    base::INullableValue,
    ir::{
        FuncObj, IPtrValue, IRAllocs, ISubValueSSA, ITraceableValue, IUser, Module, OperandSet,
        UseID, UserList, ValueClass, ValueSSA,
        global::var::GlobalVar,
        module::allocs::{IPoolAllocated, PoolAllocatedDisposeRes},
    },
    typing::ValTypeID,
};
use mtb_entity_slab::{IEntityAllocID, IPoliciedID, IndexedID, PtrID, entity_id};
use std::cell::Cell;

pub mod func;
pub mod var;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Linkage {
    External,
    DSOLocal,
    Private,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum GlobalExportErr {
    #[error("Global symbol name {0:?} was already taken by {1:?}")]
    NameTaken(SymbolStr, GlobalID),

    #[error("Global symbol {0:?} was already exported with a different name")]
    AlreadyExported(GlobalID),

    #[error("The symbol to export is not pinned: {0:?}")]
    SymbolUnpinned(GlobalID),
}

pub struct GlobalCommon {
    pub name: SymbolStr,
    pub content_ty: ValTypeID,
    pub content_align_log: u8,
    pub users: Option<UserList>,
    pub back_linkage: Cell<Linkage>,
    pub(in crate::ir) dispose_mark: Cell<bool>,
}
impl Clone for GlobalCommon {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            content_ty: self.content_ty,
            content_align_log: self.content_align_log,
            users: None,
            back_linkage: Cell::new(self.back_linkage.get()),
            dispose_mark: Cell::new(self.dispose_mark.get()),
        }
    }
}
impl GlobalCommon {
    pub fn new(name: SymbolStr, content_ty: ValTypeID, align_log: u8, allocs: &IRAllocs) -> Self {
        Self {
            name,
            content_ty,
            content_align_log: align_log,
            users: Some(UserList::new(&allocs.uses)),
            back_linkage: Cell::new(Linkage::DSOLocal),
            dispose_mark: Cell::new(false),
        }
    }
}

pub trait ISubGlobal: IUser + Sized {
    fn get_common(&self) -> &GlobalCommon;
    fn common_mut(&mut self) -> &mut GlobalCommon;

    fn get_name(&self) -> &str {
        &self.get_common().name
    }
    fn clone_name(&self) -> SymbolStr {
        self.get_common().name.clone()
    }
    fn get_back_linkage(&self) -> Linkage {
        self.get_common().back_linkage.get()
    }
    fn set_back_linkage(&self, linkage: Linkage) {
        self.get_common().back_linkage.set(linkage);
    }
    fn is_extern(&self, allocs: &IRAllocs) -> bool;
    fn is_readonly(&self) -> bool;
    fn get_linkage(&self, allocs: &IRAllocs) -> Linkage {
        if self.is_extern(allocs) { Linkage::External } else { self.get_back_linkage() }
    }
    fn get_kind(&self, allocs: &IRAllocs) -> GlobalKind;
    fn get_linkage_prefix(&self, allocs: &IRAllocs) -> &'static str;

    fn try_from_ir_ref(g: &GlobalObj) -> Option<&Self>;
    fn try_from_ir_mut(g: &mut GlobalObj) -> Option<&mut Self>;
    fn try_from_ir(g: GlobalObj) -> Option<Self>;
    fn into_ir(self) -> GlobalObj;

    fn from_ir_ref(g: &GlobalObj) -> &Self {
        Self::try_from_ir_ref(g).expect("Invalid GlobalObj variant")
    }
    fn from_ir_mut(g: &mut GlobalObj) -> &mut Self {
        Self::try_from_ir_mut(g).expect("Invalid GlobalObj variant")
    }
    fn from_ir(g: GlobalObj) -> Self {
        Self::try_from_ir(g).expect("Invalid GlobalObj variant")
    }
}
impl<T: ISubGlobal> IPtrValue for T {
    fn get_ptr_pointee_type(&self) -> ValTypeID {
        self.get_common().content_ty
    }
    fn get_ptr_pointee_align(&self) -> u32 {
        1 << self.get_common().content_align_log
    }
}
pub trait ISubGlobalID: Copy + 'static {
    type GlobalT: ISubGlobal;

    fn from_raw_ptr(ptr: PtrID<GlobalObj, <GlobalID as IPoliciedID>::PolicyT>) -> Self;
    fn into_raw_ptr(self) -> PtrID<GlobalObj, <GlobalID as IPoliciedID>::PolicyT>;
    fn raw_from(id: GlobalID) -> Self {
        Self::from_raw_ptr(id.0)
    }
    fn raw_into(self) -> GlobalID {
        GlobalID(self.into_raw_ptr())
    }

    fn try_from_global(allocs: &IRAllocs, id: GlobalID) -> Option<Self> {
        let g = id.into_raw_ptr().deref(&allocs.globals);
        if Self::GlobalT::try_from_ir_ref(g).is_some() { Some(Self::raw_from(id)) } else { None }
    }
    fn from_global(allocs: &IRAllocs, id: GlobalID) -> Self {
        Self::try_from_global(allocs, id).expect("Invalid GlobalObj variant")
    }

    fn as_indexed(self, allocs: &IRAllocs) -> Option<GlobalIndex> {
        self.into_raw_ptr()
            .to_index(&allocs.globals)
            .map(GlobalIndex)
    }
    fn to_indexed(self, allocs: &IRAllocs) -> GlobalIndex {
        self.as_indexed(allocs).expect("UAF detected")
    }
    fn try_get_entity_index(self, allocs: &IRAllocs) -> Option<usize> {
        self.as_indexed(allocs).map(|x| x.0.get_order())
    }
    fn get_entity_index(self, allocs: &IRAllocs) -> usize {
        self.to_indexed(allocs).0.get_order()
    }

    fn try_deref_ir(self, allocs: &IRAllocs) -> Option<&Self::GlobalT> {
        let g = self.into_raw_ptr().deref(&allocs.globals);
        Self::GlobalT::try_from_ir_ref(g)
    }
    fn try_deref_ir_mut(self, allocs: &mut IRAllocs) -> Option<&mut Self::GlobalT> {
        let g = self.into_raw_ptr().deref_mut(&mut allocs.globals);
        Self::GlobalT::try_from_ir_mut(g)
    }
    fn deref_ir(self, allocs: &IRAllocs) -> &Self::GlobalT {
        self.try_deref_ir(allocs)
            .expect("Invalid GlobalObj variant")
    }
    fn deref_ir_mut(self, allocs: &mut IRAllocs) -> &mut Self::GlobalT {
        self.try_deref_ir_mut(allocs)
            .expect("Invalid GlobalObj variant")
    }

    fn get_name(self, allocs: &IRAllocs) -> &str {
        self.deref_ir(allocs).get_name()
    }
    fn get_linkage(self, allocs: &IRAllocs) -> Linkage {
        self.deref_ir(allocs).get_linkage(allocs)
    }
    fn get_back_linkage(self, allocs: &IRAllocs) -> Linkage {
        self.deref_ir(allocs).get_common().back_linkage.get()
    }
    fn set_back_linkage(self, allocs: &IRAllocs, linkage: Linkage) {
        self.deref_ir(allocs).get_common().back_linkage.set(linkage);
    }
    fn is_extern(self, allocs: &IRAllocs) -> bool {
        self.deref_ir(allocs).is_extern(allocs)
    }
    fn get_kind(self, allocs: &IRAllocs) -> GlobalKind {
        self.deref_ir(allocs).get_kind(allocs)
    }

    fn allocate_unpinned(allocs: &IRAllocs, obj: Self::GlobalT) -> Self {
        let id = GlobalObj::allocate(allocs, obj.into_ir());
        Self::raw_from(id)
    }
    fn allocate_pinned(module: &Module, obj: Self::GlobalT) -> Self {
        let id = Self::allocate_unpinned(&module.allocs, obj);
        module
            .symbols
            .borrow_mut()
            .pool_add(&module.allocs, id.raw_into());
        id
    }
    fn allocate_export(module: &Module, obj: Self::GlobalT) -> Result<Self, GlobalID> {
        let id = Self::allocate_pinned(module, obj);
        module
            .symbols
            .borrow_mut()
            .try_export_symbol(id.raw_into(), &module.allocs)?;
        Ok(id)
    }
    fn export(self, module: &Module) -> Result<Self, GlobalID> {
        module
            .symbols
            .borrow_mut()
            .try_export_symbol(self.raw_into(), &module.allocs)?;
        Ok(self)
    }
    /// 如果自己没有导出, 就重命名为 name 并导出全局符号.
    /// 如果 name 已被占用, 或者自己已经被导出, 则返回 Err (已有符号ID).
    fn rename_and_export(self, name: &str, module: &mut Module) -> Result<Self, GlobalExportErr> {
        let Module { allocs, symbols, .. } = module;
        let symbols = symbols.get_mut();

        if !symbols.symbol_pinned(self.raw_into(), allocs) {
            return Err(GlobalExportErr::SymbolUnpinned(self.raw_into()));
        }
        if let Some(id) = symbols.get_symbol_by_name(name) {
            return if id == self.raw_into() {
                Ok(self)
            } else {
                Err(GlobalExportErr::NameTaken(SymbolStr::new(name), id))
            };
        }
        if symbols.get_symbol_by_name(self.get_name(allocs)).is_some() {
            return Err(GlobalExportErr::AlreadyExported(self.raw_into()));
        }
        self.deref_ir_mut(allocs).common_mut().name = SymbolStr::new(name);
        symbols
            .try_export_symbol(self.raw_into(), allocs)
            .expect("Internal Error: should not fail exporting after name check");
        Ok(self)
    }

    fn dispose(self, module: &Module) -> PoolAllocatedDisposeRes {
        GlobalObj::dispose_id(self.raw_into(), module)
    }
}

#[entity_id(GlobalID, policy = 128, allocator_type = GlobalAlloc)]
#[entity_id(GlobalIndex, policy = 128, backend = index)]
pub enum GlobalObj {
    Var(GlobalVar),
    Func(FuncObj),
}

pub type GlobalRawIndex = IndexedID<GlobalObj, <GlobalID as IPoliciedID>::PolicyT>;

impl ITraceableValue for GlobalObj {
    fn users(&self) -> &UserList {
        self.try_get_users().expect("Users list missing")
    }
    fn try_get_users(&self) -> Option<&UserList> {
        self.get_common().users.as_ref()
    }
    fn get_valtype(&self) -> ValTypeID {
        ValTypeID::Ptr
    }
    fn has_unique_ref_semantics(&self) -> bool {
        true
    }
}
impl IUser for GlobalObj {
    fn get_operands(&self) -> OperandSet<'_> {
        match self {
            GlobalObj::Var(g) => g.get_operands(),
            GlobalObj::Func(f) => f.get_operands(),
        }
    }
    fn operands_mut(&mut self) -> &mut [UseID] {
        match self {
            GlobalObj::Var(g) => g.operands_mut(),
            GlobalObj::Func(f) => f.operands_mut(),
        }
    }
}
impl ISubGlobal for GlobalObj {
    fn get_common(&self) -> &GlobalCommon {
        match self {
            GlobalObj::Var(g) => &g.common,
            GlobalObj::Func(f) => &f.common,
        }
    }
    fn common_mut(&mut self) -> &mut GlobalCommon {
        match self {
            GlobalObj::Var(g) => &mut g.common,
            GlobalObj::Func(f) => &mut f.common,
        }
    }

    fn try_from_ir_ref(g: &GlobalObj) -> Option<&Self> {
        Some(g)
    }
    fn try_from_ir_mut(g: &mut GlobalObj) -> Option<&mut Self> {
        Some(g)
    }
    fn try_from_ir(g: GlobalObj) -> Option<Self> {
        Some(g)
    }
    fn into_ir(self) -> GlobalObj {
        self
    }

    fn is_extern(&self, allocs: &IRAllocs) -> bool {
        match self {
            GlobalObj::Var(g) => g.initval[0].get_operand(allocs).is_null(),
            GlobalObj::Func(f) => f.body.is_none(),
        }
    }
    fn is_readonly(&self) -> bool {
        match self {
            GlobalObj::Var(g) => g.is_readonly(),
            GlobalObj::Func(_) => false,
        }
    }
    fn get_linkage_prefix(&self, allocs: &IRAllocs) -> &'static str {
        match self {
            GlobalObj::Var(g) => g.get_linkage_prefix(allocs),
            GlobalObj::Func(f) => f.get_linkage_prefix(allocs),
        }
    }
    fn get_kind(&self, allocs: &IRAllocs) -> GlobalKind {
        match self {
            GlobalObj::Var(g) => g.get_kind(allocs),
            GlobalObj::Func(f) => f.get_kind(allocs),
        }
    }
}
impl std::fmt::Pointer for GlobalID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl ISubGlobalID for GlobalID {
    type GlobalT = GlobalObj;

    fn from_raw_ptr(ptr: PtrID<GlobalObj, <GlobalID as IPoliciedID>::PolicyT>) -> Self {
        GlobalID(ptr)
    }
    fn into_raw_ptr(self) -> PtrID<GlobalObj, <GlobalID as IPoliciedID>::PolicyT> {
        self.0
    }
}
impl ISubValueSSA for GlobalID {
    fn get_class(self) -> ValueClass {
        ValueClass::Global
    }
    fn try_from_ir(ir: ValueSSA) -> Option<Self> {
        match ir {
            ValueSSA::Global(gid) => Some(gid),
            _ => None,
        }
    }
    fn into_ir(self) -> ValueSSA {
        ValueSSA::Global(self)
    }

    fn get_valtype(self, _: &IRAllocs) -> ValTypeID {
        ValTypeID::Ptr
    }
    fn is_zero_const(self, _: &IRAllocs) -> bool {
        false
    }

    fn can_trace(self) -> bool {
        true
    }
    fn try_get_users(self, allocs: &IRAllocs) -> Option<&UserList> {
        self.deref_ir(allocs).try_get_users()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum GlobalKind {
    ExternVar,
    ExternConst,
    VarDef,
    ConstDef,
    ExternFunc,
    FuncDef,
}

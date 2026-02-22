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
use mtb_entity_slab::{IEntityAllocID, IPoliciedID, entity_id};
use std::cell::Cell;

pub mod func;
pub mod var;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Linkage {
    External,
    DSOLocal,
    Private,
}
#[cfg(feature = "serde")]
impl serde::Serialize for Linkage {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = match self {
            Linkage::External => "external",
            Linkage::DSOLocal => "dso_local",
            Linkage::Private => "private",
        };
        serializer.serialize_str(s)
    }
}
#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for Linkage {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = smol_str::SmolStr::deserialize(deserializer)?;
        match s.as_str() {
            "external" => Ok(Linkage::External),
            "dso_local" => Ok(Linkage::DSOLocal),
            "private" => Ok(Linkage::Private),
            _ => Err(serde::de::Error::custom("Invalid Linkage string")),
        }
    }
}

/// Thread-Local Storage (TLS) models for global variables.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum TLSModel {
    /// The general dynamic model, which supports dynamic allocation of thread-local
    /// variables at runtime. Accesses to thread-local variables using this model
    /// require function calls to the thread-local storage (TLS) runtime library.
    GeneralDynamic,

    /// The local dynamic model, which assumes that the thread-local variables
    /// are defined in the same shared object as the code accessing them. This model
    /// allows for more efficient access to thread-local variables compared to the
    /// general dynamic model.
    LocalDynamic,

    /// The initial exec model, which assumes that the thread-local variables
    /// are defined in the main executable or in shared objects that are loaded
    /// at program startup. This model allows for even more efficient access to
    /// thread-local variables compared to the local dynamic model.
    InitialExec,

    /// The local exec model, which assumes that the thread-local variables
    /// are defined in the same shared object as the code accessing them, and
    /// that they are not subject to dynamic loading. This model provides the
    /// most efficient access to thread-local variables.
    LocalExec,
}
#[cfg(feature = "serde")]
impl serde::Serialize for TLSModel {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: serde::Serializer,
    {
        let s = match self {
            TLSModel::GeneralDynamic => "generaldynamic",
            TLSModel::LocalDynamic => "localdynamic",
            TLSModel::InitialExec => "initialexec",
            TLSModel::LocalExec => "localexec",
        };
        serializer.serialize_str(s)
    }
}
#[cfg(feature = "serde")]
impl<'de> serde::Deserialize<'de> for TLSModel {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: serde::Deserializer<'de>,
    {
        let s = smol_str::SmolStr::deserialize(deserializer)?;
        match s.as_str() {
            "generaldynamic" => Ok(TLSModel::GeneralDynamic),
            "localdynamic" => Ok(TLSModel::LocalDynamic),
            "initialexec" => Ok(TLSModel::InitialExec),
            "localexec" => Ok(TLSModel::LocalExec),
            _ => Err(serde::de::Error::custom("Invalid TLSModel string")),
        }
    }
}
impl TLSModel {
    pub fn is_dynamic(self) -> bool {
        matches!(self, TLSModel::GeneralDynamic | TLSModel::LocalDynamic)
    }

    pub fn is_static(self) -> bool {
        matches!(self, TLSModel::InitialExec | TLSModel::LocalExec)
    }

    pub fn get_ir_text(self) -> &'static str {
        match self {
            TLSModel::GeneralDynamic => "generaldynamic",
            TLSModel::LocalDynamic => "localdynamic",
            TLSModel::InitialExec => "initialexec",
            TLSModel::LocalExec => "localexec",
        }
    }
    pub fn from_ir_text(text: &str) -> Option<Self> {
        match text {
            "generaldynamic" => Some(TLSModel::GeneralDynamic),
            "localdynamic" => Some(TLSModel::LocalDynamic),
            "initialexec" => Some(TLSModel::InitialExec),
            "localexec" => Some(TLSModel::LocalExec),
            _ => None,
        }
    }
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

    fn from_inner(ptr: GlobalInnerID) -> Self;
    fn into_inner(self) -> GlobalInnerID;
    fn raw_from(id: GlobalID) -> Self {
        Self::from_inner(id.0)
    }
    fn raw_into(self) -> GlobalID {
        GlobalID(self.into_inner())
    }

    fn try_from_global(allocs: &IRAllocs, id: GlobalID) -> Option<Self> {
        let g = id.into_inner().deref(&allocs.globals);
        if Self::GlobalT::try_from_ir_ref(g).is_some() { Some(Self::raw_from(id)) } else { None }
    }
    fn from_global(allocs: &IRAllocs, id: GlobalID) -> Self {
        Self::try_from_global(allocs, id).expect("Invalid GlobalObj variant")
    }

    fn try_get_entity_index(self, allocs: &IRAllocs) -> Option<usize> {
        if self.is_alive(allocs) { Some(self.into_inner().get_order()) } else { None }
    }
    fn get_entity_index(self, allocs: &IRAllocs) -> usize {
        self.try_get_entity_index(allocs)
            .expect("Invalid GlobalObj variant or the global has been disposed")
    }

    fn try_deref_ir(self, allocs: &IRAllocs) -> Option<&Self::GlobalT> {
        self.into_inner()
            .try_deref(&allocs.globals)
            .and_then(Self::GlobalT::try_from_ir_ref)
    }
    fn try_deref_ir_mut(self, allocs: &mut IRAllocs) -> Option<&mut Self::GlobalT> {
        let g = self.into_inner().deref_mut(&mut allocs.globals);
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
    fn is_alive(self, allocs: &IRAllocs) -> bool {
        match self.try_deref_ir(allocs) {
            Some(g) => !g.get_common().dispose_mark.get(),
            None => false,
        }
    }

    fn get_name(self, allocs: &IRAllocs) -> &str {
        self.deref_ir(allocs).get_name()
    }
    fn clone_name(self, allocs: &IRAllocs) -> SymbolStr {
        self.deref_ir(allocs).clone_name()
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

#[entity_id(GlobalID, policy = 128, allocator_type = GlobalAlloc, backend = index)]
pub enum GlobalObj {
    Var(GlobalVar),
    Func(FuncObj),
}
pub type GlobalInnerID = <GlobalID as IPoliciedID>::BackID;

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

    fn from_inner(ptr: GlobalInnerID) -> Self {
        GlobalID(ptr)
    }
    fn into_inner(self) -> GlobalInnerID {
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

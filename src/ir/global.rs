use crate::{
    base::INullableValue,
    impl_traceable_from_common,
    ir::{
        FuncObj, IPtrValue, IRAllocs, ISubValueSSA, ITraceableValue, IUser, OperandSet, UseID,
        UserList, ValueClass, ValueSSA, global::var::GlobalVar,
    },
    typing::ValTypeID,
};
use mtb_entity::{IEntityAllocID, PtrID};
use std::cell::Cell;

pub mod func;
pub mod var;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Linkage {
    External,
    DSOLocal,
    Private,
}

pub struct GlobalCommon {
    pub name: String,
    pub content_ty: ValTypeID,
    pub content_align_log: u8,
    pub users: Option<UserList>,
    pub back_linkage: Cell<Linkage>,
}
impl Clone for GlobalCommon {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            content_ty: self.content_ty,
            content_align_log: self.content_align_log,
            users: None,
            back_linkage: Cell::new(self.back_linkage.get()),
        }
    }
}

pub trait ISubGlobal: IUser {
    fn get_common(&self) -> &GlobalCommon;
    fn common_mut(&mut self) -> &mut GlobalCommon;

    fn get_name(&self) -> &str {
        &self.get_common().name
    }
    fn get_back_linkage(&self) -> Linkage {
        self.get_common().back_linkage.get()
    }
    fn set_back_linkage(&self, linkage: Linkage) {
        self.get_common().back_linkage.set(linkage);
    }
    fn is_extern(&self, allocs: &IRAllocs) -> bool;
    fn get_linkage(&self, allocs: &IRAllocs) -> Linkage {
        if self.is_extern(allocs) { Linkage::External } else { self.get_back_linkage() }
    }

    fn try_from_ir_ref(g: &GlobalObj) -> Option<&Self>;
    fn try_from_ir_mut(g: &mut GlobalObj) -> Option<&mut Self>;
    fn try_from_ir(g: GlobalObj) -> Option<Self>
    where
        Self: Sized;
    fn into_ir(self) -> GlobalObj;

    fn from_ir_ref(g: &GlobalObj) -> &Self {
        Self::try_from_ir_ref(g).expect("Invalid GlobalObj variant")
    }
    fn from_ir_mut(g: &mut GlobalObj) -> &mut Self {
        Self::try_from_ir_mut(g).expect("Invalid GlobalObj variant")
    }
    fn from_ir(g: GlobalObj) -> Self
    where
        Self: Sized,
    {
        Self::try_from_ir(g).expect("Invalid GlobalObj variant")
    }

    fn dispose(&self, allocs: &IRAllocs) {
        self.user_dispose(allocs);
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

    fn raw_from_ir(id: GlobalID) -> Self;
    fn into_ir(self) -> GlobalID;

    fn try_from_ir(allocs: &IRAllocs, id: GlobalID) -> Option<Self> {
        let g = id.deref(&allocs.globals);
        if Self::GlobalT::try_from_ir_ref(g).is_some() { Some(Self::raw_from_ir(id)) } else { None }
    }
    fn from_ir(allocs: &IRAllocs, id: GlobalID) -> Self {
        Self::try_from_ir(allocs, id).expect("Invalid GlobalObj variant")
    }

    fn try_deref_ir(self, allocs: &IRAllocs) -> Option<&Self::GlobalT> {
        let g = self.into_ir().deref(&allocs.globals);
        Self::GlobalT::try_from_ir_ref(g)
    }
    fn try_deref_ir_mut(self, allocs: &mut IRAllocs) -> Option<&mut Self::GlobalT> {
        let g = self.into_ir().deref_mut(&mut allocs.globals);
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

    fn new(allocs: &IRAllocs, obj: Self::GlobalT) -> Self {
        let mut g_obj = obj.into_ir();
        if g_obj.get_common().users.is_none() {
            g_obj.common_mut().users = Some(UserList::new(&allocs.uses));
        }
        let g_id = allocs.globals.allocate(g_obj);
        Self::raw_from_ir(g_id)
    }
    fn dispose(self, allocs: &IRAllocs) {
        self.deref_ir(allocs).dispose(allocs);
    }
}

pub enum GlobalObj {
    Var(GlobalVar),
    Func(FuncObj),
}
pub type GlobalID = PtrID<GlobalObj>;

impl_traceable_from_common!(GlobalObj, true);
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
}
impl ISubGlobalID for GlobalID {
    type GlobalT = GlobalObj;

    fn raw_from_ir(id: GlobalID) -> Self {
        id
    }
    fn into_ir(self) -> GlobalID {
        self
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

    fn can_trace(self) -> bool {
        true
    }
    fn try_get_users(self, allocs: &IRAllocs) -> Option<&UserList> {
        self.deref_ir(allocs).try_get_users()
    }
}

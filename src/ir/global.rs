use crate::{
    ir::{IRAllocs, IUser, OperandSet, UseID, UserList, global::var::GlobalVar},
    typing::ValTypeID,
};
use mtb_entity::{IEntityAllocID, PtrID};
use std::cell::Cell;

pub mod var;
pub mod func;

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
    pub linkage: Cell<Linkage>,
}
impl Clone for GlobalCommon {
    fn clone(&self) -> Self {
        Self {
            name: self.name.clone(),
            content_ty: self.content_ty,
            content_align_log: self.content_align_log,
            users: None,
            linkage: Cell::new(self.linkage.get()),
        }
    }
}

pub trait ISubGlobal: IUser {
    fn get_common(&self) -> &GlobalCommon;
    fn common_mut(&mut self) -> &mut GlobalCommon;

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
pub trait ISubGlobalID: Sized {
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

    fn new(allocs: &IRAllocs, obj: Self::GlobalT) -> Self {
        let mut g_obj = obj.into_ir();
        if g_obj.get_common().users.is_none() {
            g_obj.common_mut().users = Some(UserList::new(&allocs.uses));
        }
        let g_id = allocs.globals.allocate(g_obj);
        Self::raw_from_ir(g_id)
    }
}

#[derive(Clone)]
pub enum GlobalObj {
    Var(GlobalVar),
}
pub type GlobalID = PtrID<GlobalObj>;

impl IUser for GlobalObj {
    fn get_operands(&self) -> OperandSet<'_> {
        match self {
            GlobalObj::Var(g) => g.get_operands(),
        }
    }
    fn operands_mut(&mut self) -> &mut [UseID] {
        match self {
            GlobalObj::Var(g) => g.operands_mut(),
        }
    }
}
impl ISubGlobal for GlobalObj {
    fn get_common(&self) -> &GlobalCommon {
        match self {
            GlobalObj::Var(g) => &g.common,
        }
    }
    fn common_mut(&mut self) -> &mut GlobalCommon {
        match self {
            GlobalObj::Var(g) => &mut g.common,
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

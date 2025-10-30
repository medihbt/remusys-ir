use crate::ir::{
    IRAllocs, ITraceableValue, IUser, OperandSet, UseID, UserList,
    constant::{array::ArrayExpr, structure::StructExpr},
};
use mtb_entity::{IEntityAllocID, PtrID};

pub struct ExprCommon {
    pub users: Option<UserList>,
}
impl Clone for ExprCommon {
    fn clone(&self) -> Self {
        Self { users: None }
    }
}
impl ExprCommon {
    pub fn deep_cloned(&self, allocs: &IRAllocs) -> Self {
        Self { users: Some(UserList::new(&allocs.uses)) }
    }
    pub fn new(allocs: &IRAllocs) -> Self {
        Self { users: Some(UserList::new(&allocs.uses)) }
    }
    pub fn none() -> Self {
        Self { users: None }
    }
}

pub trait ISubExpr: IUser {
    fn get_common(&self) -> &ExprCommon;
    fn common_mut(&mut self) -> &mut ExprCommon;

    fn try_from_ir_ref(expr: &ExprObj) -> Option<&Self>;
    fn try_from_ir_mut(expr: &mut ExprObj) -> Option<&mut Self>;
    fn try_from_ir(expr: ExprObj) -> Option<Self>;
    fn into_ir(self) -> ExprObj;

    fn from_ir_ref(expr: &ExprObj) -> &Self {
        Self::try_from_ir_ref(expr).expect("Invalid ExprObj type for ISubExpr")
    }
    fn from_ir_mut(expr: &mut ExprObj) -> &mut Self {
        Self::try_from_ir_mut(expr).expect("Invalid ExprObj type for ISubExpr")
    }
    fn from_ir(expr: ExprObj) -> Self {
        Self::try_from_ir(expr).expect("Invalid ExprObj type for ISubExpr")
    }
}
impl<T: ISubExpr> ITraceableValue for T {
    fn users(&self) -> &UserList {
        self.get_common().users.as_ref().unwrap()
    }
    fn has_single_reference_semantics(&self) -> bool {
        false
    }
}

pub trait ISubExprID: Copy {
    type ExprObjT: ISubExpr + 'static;

    fn raw_from_ir(id: PtrID<ExprObj>) -> Self;
    fn into_ir(self) -> PtrID<ExprObj>;

    fn try_from_ir(id: PtrID<ExprObj>, allocs: &IRAllocs) -> Option<Self> {
        let expr = id.deref(&allocs.exprs);
        Self::ExprObjT::try_from_ir_ref(expr).map(|_| Self::raw_from_ir(id))
    }
    fn deref_ir(self, allocs: &IRAllocs) -> &Self::ExprObjT {
        let expr = self.into_ir().deref(&allocs.exprs);
        Self::ExprObjT::from_ir_ref(expr)
    }
    fn deref_ir_mut(self, allocs: &mut IRAllocs) -> &mut Self::ExprObjT {
        let expr = self.into_ir().deref_mut(&mut allocs.exprs);
        Self::ExprObjT::from_ir_mut(expr)
    }

    fn new(allocs: &IRAllocs, obj: Self::ExprObjT) -> Self {
        let mut obj = obj.into_ir();
        if obj.get_common().users.is_none() {
            obj.common_mut().users = Some(UserList::new(&allocs.uses));
        }
        let id = allocs.exprs.allocate(obj);
        Self::raw_from_ir(id)
    }
}

#[derive(Clone)]
pub enum ExprObj {
    Array(ArrayExpr),
    Struct(StructExpr),
}
pub type ExprID = PtrID<ExprObj>;

impl IUser for ExprObj {
    fn get_operands(&self) -> OperandSet<'_> {
        use ExprObj::*;
        match self {
            Array(arr) => arr.get_operands(),
            Struct(struc) => struc.get_operands(),
        }
    }
    fn operands_mut(&mut self) -> &mut [UseID] {
        use ExprObj::*;
        match self {
            Array(arr) => arr.operands_mut(),
            Struct(struc) => struc.operands_mut(),
        }
    }
}
impl ISubExpr for ExprObj {
    fn get_common(&self) -> &ExprCommon {
        use ExprObj::*;
        match self {
            Array(arr) => &arr.common,
            Struct(struc) => &struc.common,
        }
    }
    fn common_mut(&mut self) -> &mut ExprCommon {
        use ExprObj::*;
        match self {
            Array(arr) => &mut arr.common,
            Struct(struc) => &mut struc.common,
        }
    }
    fn try_from_ir_ref(expr: &ExprObj) -> Option<&Self> {
        Some(expr)
    }
    fn try_from_ir_mut(expr: &mut ExprObj) -> Option<&mut Self> {
        Some(expr)
    }
    fn try_from_ir(expr: ExprObj) -> Option<Self> {
        Some(expr)
    }
    fn into_ir(self) -> ExprObj {
        self
    }
}
impl ISubExprID for ExprID {
    type ExprObjT = ExprObj;

    fn raw_from_ir(id: PtrID<ExprObj>) -> Self {
        id
    }
    fn into_ir(self) -> PtrID<ExprObj> {
        self
    }
}

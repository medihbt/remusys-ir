use crate::{
    impl_traceable_from_common,
    ir::{
        FixVec, IRAllocs, ISubValueSSA, IUser, OperandSet, UseID, UserList, ValueClass, ValueSSA,
        constant::{array::ArrayExpr, structure::StructExpr},
        module::allocs::{IPoolAllocated, PoolAllocatedDisposeRes},
    },
    typing::ValTypeID,
};
use mtb_entity::{IEntityAllocID, PtrID};
use std::cell::Cell;

pub struct ExprCommon {
    pub users: Option<UserList>,
    pub(in crate::ir) dispose_mark: Cell<bool>,
}
impl Clone for ExprCommon {
    fn clone(&self) -> Self {
        Self {
            users: None,
            dispose_mark: Cell::new(self.dispose_mark.get()),
        }
    }
}
impl ExprCommon {
    pub fn new(allocs: &IRAllocs) -> Self {
        Self {
            users: Some(UserList::new(&allocs.uses)),
            dispose_mark: Cell::new(false),
        }
    }
    pub fn none() -> Self {
        Self { users: None, dispose_mark: Cell::new(false) }
    }
}

pub trait ISubExpr: IUser + Sized {
    fn get_common(&self) -> &ExprCommon;
    fn common_mut(&mut self) -> &mut ExprCommon;

    fn get_valtype(&self) -> ValTypeID;

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

    fn allocate(allocs: &IRAllocs, obj: Self::ExprObjT) -> Self {
        let id = ExprObj::allocate(allocs, obj.into_ir());
        Self::raw_from_ir(id)
    }

    fn dispose(self, allocs: &IRAllocs) -> PoolAllocatedDisposeRes {
        ExprObj::dispose_id(self.into_ir(), allocs)
    }
}

#[derive(Clone)]
pub enum ExprObj {
    Array(ArrayExpr),
    Struct(StructExpr),
    FixVec(FixVec),
}
pub type ExprID = PtrID<ExprObj>;

impl_traceable_from_common!(ExprObj, false);
impl IUser for ExprObj {
    fn get_operands(&self) -> OperandSet<'_> {
        use ExprObj::*;
        match self {
            Array(arr) => arr.get_operands(),
            Struct(struc) => struc.get_operands(),
            FixVec(vec) => vec.get_operands(),
        }
    }
    fn operands_mut(&mut self) -> &mut [UseID] {
        use ExprObj::*;
        match self {
            Array(arr) => arr.operands_mut(),
            Struct(struc) => struc.operands_mut(),
            FixVec(vec) => vec.operands_mut(),
        }
    }
}
impl ISubExpr for ExprObj {
    fn get_common(&self) -> &ExprCommon {
        use ExprObj::*;
        match self {
            Array(arr) => &arr.common,
            Struct(struc) => &struc.common,
            FixVec(vec) => &vec.common,
        }
    }
    fn common_mut(&mut self) -> &mut ExprCommon {
        use ExprObj::*;
        match self {
            Array(arr) => &mut arr.common,
            Struct(struc) => &mut struc.common,
            FixVec(vec) => &mut vec.common,
        }
    }
    fn get_valtype(&self) -> ValTypeID {
        use ExprObj::*;
        match self {
            Array(arr) => arr.get_valtype(),
            Struct(struc) => struc.get_valtype(),
            FixVec(vec) => vec.get_valtype(),
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
impl ISubValueSSA for ExprID {
    fn get_class(self) -> ValueClass {
        ValueClass::ConstExpr
    }
    fn try_from_ir(ir: ValueSSA) -> Option<Self> {
        match ir {
            ValueSSA::ConstExpr(id) => Some(id),
            _ => None,
        }
    }
    fn into_ir(self) -> ValueSSA {
        ValueSSA::ConstExpr(self)
    }
    fn is_zero_const(self, allocs: &IRAllocs) -> bool {
        let operands = match self.deref_ir(allocs) {
            ExprObj::Array(arr) => arr.elems.as_slice(),
            ExprObj::Struct(struc) => struc.fields.as_slice(),
            ExprObj::FixVec(vec) => vec.elems.as_slice(),
        };
        operands
            .iter()
            .all(|&use_id| use_id.get_operand(allocs).is_zero_const(allocs))
    }

    fn get_valtype(self, allocs: &IRAllocs) -> ValTypeID {
        self.deref_ir(allocs).get_valtype()
    }

    fn can_trace(self) -> bool {
        true
    }
    fn try_get_users(self, allocs: &IRAllocs) -> Option<&UserList> {
        Some(&self.deref_ir(allocs).get_common().users.as_ref().unwrap())
    }
}

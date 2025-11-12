use crate::{
    impl_traceable_from_common,
    ir::{
        FixVec, IRAllocs, ISubValueSSA, IUser, OperandSet, UseID, UserList, ValueClass, ValueSSA,
        constant::{array::ArrayExpr, structure::StructExpr},
        module::allocs::{IPoolAllocated, PoolAllocatedDisposeRes},
    },
    typing::ValTypeID,
};
use mtb_entity_slab::{IEntityAllocID, IPolicyPtrID, PtrID, entity_ptr_id};
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

    fn from_raw_ptr(id: ExprRawPtr) -> Self;
    fn into_raw_ptr(self) -> ExprRawPtr;

    fn raw_from(id: ExprID) -> Self {
        Self::from_raw_ptr(id.0)
    }
    fn raw_into(self) -> ExprID {
        ExprID(self.into_raw_ptr())
    }

    fn try_from_expr(
        id: PtrID<ExprObj, <ExprID as IPolicyPtrID>::PolicyT>,
        allocs: &IRAllocs,
    ) -> Option<Self> {
        let expr = id.deref(&allocs.exprs);
        Self::ExprObjT::try_from_ir_ref(expr).map(|_| Self::from_raw_ptr(id))
    }
    fn deref_ir(self, allocs: &IRAllocs) -> &Self::ExprObjT {
        let expr = self.into_raw_ptr().deref(&allocs.exprs);
        Self::ExprObjT::from_ir_ref(expr)
    }
    fn deref_ir_mut(self, allocs: &mut IRAllocs) -> &mut Self::ExprObjT {
        let expr = self.into_raw_ptr().deref_mut(&mut allocs.exprs);
        Self::ExprObjT::from_ir_mut(expr)
    }

    fn allocate(allocs: &IRAllocs, obj: Self::ExprObjT) -> Self {
        let id = ExprObj::allocate(allocs, obj.into_ir());
        Self::raw_from(id)
    }

    fn dispose(self, allocs: &IRAllocs) -> PoolAllocatedDisposeRes {
        ExprObj::dispose_id(self.raw_into(), allocs)
    }
}

#[derive(Clone)]
#[entity_ptr_id(ExprID, policy = 256, allocator_type = ExprAlloc)]
pub enum ExprObj {
    Array(ArrayExpr),
    Struct(StructExpr),
    FixVec(FixVec),
}
pub(in crate::ir) type ExprRawPtr = PtrID<ExprObj, <ExprID as IPolicyPtrID>::PolicyT>;

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
impl std::fmt::Pointer for ExprID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        self.0.fmt(f)
    }
}
impl ISubExprID for ExprID {
    type ExprObjT = ExprObj;

    fn from_raw_ptr(id: ExprRawPtr) -> Self {
        Self(id)
    }
    fn into_raw_ptr(self) -> ExprRawPtr {
        self.0
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

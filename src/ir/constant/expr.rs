use crate::{
    ir::{
        DataArrayExpr, FixVec, FixVecID, IRAllocs, ISubValueSSA, ITraceableValue, IUser,
        OperandSet, SplatArrayExpr, StructExprID, UseID, UserList, ValueClass, ValueSSA,
        constant::{
            array::{ArrayExpr, KVArrayExpr},
            structure::StructExpr,
        },
        module::allocs::{IPoolAllocated, PoolAllocatedDisposeRes},
    },
    typing::{AggrType, IValType, StructTypeID, TypeContext, ValTypeID},
};
use mtb_entity_slab::{IEntityAllocID, IPoliciedID, IndexedID, PtrID, entity_id};
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

    fn get_expr_type(&self) -> ValTypeID;

    fn try_from_ir_ref(expr: &ExprObj) -> Option<&Self>;
    fn try_from_ir_mut(expr: &mut ExprObj) -> Option<&mut Self>;
    fn try_from_ir(expr: ExprObj) -> Option<Self>;
    fn into_ir(self) -> ExprObj;

    fn is_zero_const(&self, allocs: &IRAllocs) -> bool;

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

    fn try_from_expr(id: ExprID, allocs: &IRAllocs) -> Option<Self> {
        let id = id.0;
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
#[macro_export]
macro_rules! _remusys_ir_subexpr {
    ($IDName:ident, $ObjType:ident) => {
        impl $crate::ir::ITraceableValue for $ObjType {
            fn users(&self) -> &$crate::ir::UserList {
                self.try_get_users()
                    .expect("Internal error: alocated Expression should have a valid UserList")
            }
            fn try_get_users(&self) -> Option<&$crate::ir::UserList> {
                self.get_common().users.as_ref()
            }
            fn get_valtype(&self) -> $crate::typing::ValTypeID {
                self.get_expr_type()
            }
            fn has_unique_ref_semantics(&self) -> bool {
                false
            }
        }

        #[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
        pub struct $IDName(pub $crate::ir::constant::expr::ExprRawPtr);
        impl $crate::ir::ISubExprID for $IDName {
            type ExprObjT = $ObjType;

            fn from_raw_ptr(id: $crate::ir::constant::expr::ExprRawPtr) -> Self {
                Self(id)
            }
            fn into_raw_ptr(self) -> $crate::ir::constant::expr::ExprRawPtr {
                self.0
            }
        }
        impl $crate::ir::IValueConvert for $IDName {
            fn try_from_value(
                value: $crate::ir::ValueSSA,
                module: &$crate::ir::Module,
            ) -> Option<Self> {
                let expr_id = match value {
                    $crate::ir::ValueSSA::ConstExpr(id) => id,
                    _ => return None,
                };
                Self::try_from_expr(expr_id, &module.allocs)
            }
            fn into_value(self) -> $crate::ir::ValueSSA {
                self.raw_into().into_ir()
            }
        }
    };
    ($IDName:ident, $ObjType:ident, ArrayExpr) => {
        $crate::_remusys_ir_subexpr!($IDName, $ObjType);
        impl $crate::ir::IArrayExprID for $IDName {}
    };
}

#[derive(Clone)]
#[entity_id(ExprID, policy = 256, allocator_type = ExprAlloc)]
#[entity_id(ExprIndex, policy = 256, backend = index)]
pub enum ExprObj {
    Array(ArrayExpr),
    DataArray(DataArrayExpr),
    SplatArray(SplatArrayExpr),
    KVArray(KVArrayExpr),
    Struct(StructExpr),
    FixVec(FixVec),
}
pub(in crate::ir) type ExprRawPtr = PtrID<ExprObj, <ExprID as IPoliciedID>::PolicyT>;
pub type ExprRawIndex = IndexedID<ExprObj, <ExprID as IPoliciedID>::PolicyT>;

impl ITraceableValue for ExprObj {
    fn users(&self) -> &UserList {
        self.try_get_users()
            .expect("Internal error: alocated ExprObj should have a valid UserList")
    }
    fn try_get_users(&self) -> Option<&UserList> {
        self.get_common().users.as_ref()
    }
    fn get_valtype(&self) -> ValTypeID {
        self.get_expr_type()
    }
    fn has_unique_ref_semantics(&self) -> bool {
        todo!()
    }
}
impl IUser for ExprObj {
    fn get_operands(&self) -> OperandSet<'_> {
        use ExprObj::*;
        match self {
            Array(arr) => arr.get_operands(),
            DataArray(arr) => arr.get_operands(),
            SplatArray(arr) => arr.get_operands(),
            KVArray(arr) => arr.get_operands(),
            Struct(struc) => struc.get_operands(),
            FixVec(vec) => vec.get_operands(),
        }
    }
    fn operands_mut(&mut self) -> &mut [UseID] {
        use ExprObj::*;
        match self {
            Array(arr) => arr.operands_mut(),
            DataArray(arr) => arr.operands_mut(),
            SplatArray(arr) => arr.operands_mut(),
            KVArray(arr) => arr.operands_mut(),
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
            DataArray(arr) => &arr.common,
            SplatArray(arr) => &arr.common,
            KVArray(arr) => &arr.common,
            Struct(struc) => &struc.common,
            FixVec(vec) => &vec.common,
        }
    }
    fn common_mut(&mut self) -> &mut ExprCommon {
        use ExprObj::*;
        match self {
            Array(arr) => &mut arr.common,
            DataArray(arr) => &mut arr.common,
            SplatArray(arr) => &mut arr.common,
            KVArray(arr) => &mut arr.common,
            Struct(struc) => &mut struc.common,
            FixVec(vec) => &mut vec.common,
        }
    }
    #[doc(hidden)]
    fn get_expr_type(&self) -> ValTypeID {
        use ExprObj::*;
        match self {
            Array(arr) => arr.get_expr_type(),
            DataArray(arr) => arr.get_expr_type(),
            SplatArray(arr) => arr.get_expr_type(),
            KVArray(arr) => arr.get_expr_type(),
            Struct(struc) => struc.get_expr_type(),
            FixVec(vec) => vec.get_expr_type(),
        }
    }
    fn is_zero_const(&self, allocs: &IRAllocs) -> bool {
        use ExprObj::*;
        match self {
            Array(arr) => arr.is_zero_const(allocs),
            DataArray(arr) => arr.is_zero_const(allocs),
            SplatArray(arr) => arr.is_zero_const(allocs),
            KVArray(arr) => arr.is_zero_const(allocs),
            Struct(struc) => struc.is_zero_const(allocs),
            FixVec(vec) => vec.is_zero_const(allocs),
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
        self.deref_ir(allocs).is_zero_const(allocs)
    }

    fn get_valtype(self, allocs: &IRAllocs) -> ValTypeID {
        self.deref_ir(allocs).get_expr_type()
    }

    fn can_trace(self) -> bool {
        true
    }
    fn try_get_users(self, allocs: &IRAllocs) -> Option<&UserList> {
        let common = self.deref_ir(allocs).get_common();
        let Some(users) = &common.users else {
            panic!("Internal error: alocated ExprObj should have a valid UserList");
        };
        Some(users)
    }
}
impl ExprID {
    pub fn as_indexed(self, allocs: &IRAllocs) -> Option<ExprIndex> {
        let indexed_id = self.0.to_index(&allocs.exprs)?;
        Some(ExprIndex(indexed_id))
    }
    pub fn to_indexed(self, allocs: &IRAllocs) -> ExprIndex {
        self.as_indexed(allocs)
            .expect("Error: Attempted to get indexed ID of freed ExprID")
    }
    pub fn try_from_indexed(indexed: ExprIndex, allocs: &IRAllocs) -> Option<Self> {
        let raw_ptr = indexed.0.to_ptr(&allocs.exprs)?;
        Some(ExprID(raw_ptr))
    }
    pub fn from_indexed(indexed: ExprIndex, allocs: &IRAllocs) -> Self {
        Self::try_from_indexed(indexed, allocs)
            .expect("Error: Attempted to get ExprID from freed ExprIndex")
    }

    pub fn try_get_entity_index(self, allocs: &IRAllocs) -> Option<usize> {
        let indexed_id = self.0.to_index(&allocs.exprs)?;
        Some(indexed_id.get_order())
    }
    pub fn get_entity_index(self, allocs: &IRAllocs) -> usize {
        self.try_get_entity_index(allocs)
            .expect("Error: Attempted to get indexed ID of freed ExprID")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct AggrZero(pub AggrType);

impl ISubValueSSA for AggrZero {
    fn get_class(self) -> ValueClass {
        ValueClass::AggrZero
    }
    fn try_from_ir(ir: ValueSSA) -> Option<Self> {
        match ir {
            ValueSSA::AggrZero(a) => Some(AggrZero(a)),
            _ => None,
        }
    }

    fn into_ir(self) -> ValueSSA {
        ValueSSA::AggrZero(self.0)
    }
    fn get_valtype(self, _: &IRAllocs) -> ValTypeID {
        self.0.into_ir()
    }

    fn can_trace(self) -> bool {
        false
    }
    fn try_get_users(self, _: &IRAllocs) -> Option<&UserList> {
        None
    }
    fn is_zero_const(self, _: &IRAllocs) -> bool {
        true
    }
}

impl AggrZero {
    pub fn expand(self, allocs: &IRAllocs, tctx: &TypeContext) -> ExprID {
        use crate::ir::SplatArrayExprID;
        match self.0 {
            AggrType::Array(arrty) => {
                let elemty = arrty.get_element_type(tctx);
                let elem = ValueSSA::new_zero(elemty)
                    .expect("Internal error: array element type cannot store");
                SplatArrayExprID::new(allocs, tctx, arrty, elem).raw_into()
            }
            AggrType::Struct(structty) => Self::make_zero_struct(allocs, tctx, structty),
            AggrType::Alias(sa) => Self::make_zero_struct(allocs, tctx, sa.get_aliasee(tctx)),
            AggrType::FixVec(vecty) => {
                let elemty = vecty.get_elem();
                let fvec = FixVecID::new_uninit(allocs, vecty);
                let zero = ValueSSA::new_zero(elemty.into_ir()).unwrap();
                for index in 0..vecty.get_len() {
                    fvec.set_elem(allocs, index, zero);
                }
                fvec.raw_into()
            }
        }
    }

    fn make_zero_struct(allocs: &IRAllocs, tctx: &TypeContext, structty: StructTypeID) -> ExprID {
        let nelems = structty.get_nfields(tctx);
        let struc_exp = StructExpr::new_uninit(allocs, tctx, structty);
        for i in 0..nelems {
            let ty = structty.get_fields(tctx)[i];
            let elem =
                ValueSSA::new_zero(ty).expect("Internal error: array element type cannot store");
            struc_exp.fields[i].set_operand(allocs, elem);
        }
        StructExprID::allocate(allocs, struc_exp).raw_into()
    }

    pub fn try_from_expr(expr: impl ISubExprID, allocs: &IRAllocs) -> Option<Self> {
        Self::do_try_from_expr(expr.raw_into(), allocs)
    }
    fn do_try_from_expr(expr: ExprID, allocs: &IRAllocs) -> Option<Self> {
        let (is_zconst, ty) = match expr.deref_ir(allocs) {
            ExprObj::Array(arr) => (arr.is_zero_const(allocs), AggrType::Array(arr.arrty)),
            ExprObj::DataArray(arr) => (arr.is_zero_const(allocs), AggrType::Array(arr.arrty)),
            ExprObj::SplatArray(arr) => (arr.is_zero_const(allocs), AggrType::Array(arr.arrty)),
            ExprObj::KVArray(arr) => (arr.is_zero_const(allocs), AggrType::Array(arr.arrty)),
            ExprObj::Struct(struc) => (
                struc.is_zero_const(allocs),
                AggrType::Struct(struc.structty),
            ),
            ExprObj::FixVec(fvec) => (fvec.is_zero_const(allocs), AggrType::FixVec(fvec.vecty)),
        };
        if is_zconst { Some(Self(ty)) } else { None }
    }
}

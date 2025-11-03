use crate::{
    impl_traceable_from_common,
    ir::{
        ExprID, ExprObj, IRAllocs, ISubExprID, IUser, OperandSet, UseID, UseKind,
        constant::expr::{ExprCommon, ISubExpr},
    },
    typing::{IValType, StructTypeID, TypeContext, ValTypeID},
};
use mtb_entity::PtrID;
use smallvec::SmallVec;

#[derive(Clone)]
pub struct StructExpr {
    pub common: ExprCommon,
    pub structty: StructTypeID,
    pub fields: SmallVec<[UseID; 4]>,
}
impl_traceable_from_common!(StructExpr, false);
impl IUser for StructExpr {
    fn get_operands(&self) -> OperandSet<'_> {
        OperandSet::Fixed(&self.fields)
    }
    fn operands_mut(&mut self) -> &mut [UseID] {
        &mut self.fields
    }
}
impl ISubExpr for StructExpr {
    fn get_common(&self) -> &ExprCommon {
        &self.common
    }
    fn common_mut(&mut self) -> &mut ExprCommon {
        &mut self.common
    }
    fn get_valtype(&self) -> ValTypeID {
        self.structty.into_ir()
    }
    fn try_from_ir_ref(expr: &ExprObj) -> Option<&Self> {
        if let ExprObj::Struct(struc) = expr { Some(struc) } else { None }
    }
    fn try_from_ir_mut(expr: &mut ExprObj) -> Option<&mut Self> {
        if let ExprObj::Struct(struc) = expr { Some(struc) } else { None }
    }
    fn try_from_ir(expr: ExprObj) -> Option<Self> {
        if let ExprObj::Struct(struc) = expr { Some(struc) } else { None }
    }
    fn into_ir(self) -> ExprObj {
        ExprObj::Struct(self)
    }
}
impl StructExpr {
    pub fn new_uninit(allocs: &IRAllocs, tctx: &TypeContext, structty: StructTypeID) -> Self {
        let nfields = structty.get_nfields(tctx);

        let fields = {
            let mut fields = SmallVec::with_capacity(nfields);
            for i in 0..nfields {
                let use_id = UseID::new(allocs, UseKind::StructField(i));
                fields.push(use_id);
            }
            fields
        };
        Self { common: ExprCommon::none(), structty, fields }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct StructExprID(pub ExprID);

impl ISubExprID for StructExprID {
    type ExprObjT = StructExpr;

    fn raw_from_ir(id: PtrID<ExprObj>) -> Self {
        StructExprID(id)
    }
    fn into_ir(self) -> PtrID<ExprObj> {
        self.0
    }
}
impl StructExprID {
    pub fn new_uninit(allocs: &IRAllocs, tctx: &TypeContext, structty: StructTypeID) -> Self {
        let expr = StructExpr::new_uninit(allocs, tctx, structty);
        Self::allocate(allocs, expr)
    }

    pub fn get_struct_type(self, allocs: &IRAllocs) -> StructTypeID {
        self.deref_ir(allocs).structty
    }
    pub fn get_fields(self, allocs: &IRAllocs) -> &[UseID] {
        &self.deref_ir(allocs).fields
    }
}

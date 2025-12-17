use crate::{
    _remusys_ir_subexpr,
    ir::{
        ExprObj, IRAllocs, ISubExprID, ISubValueSSA, IUser, OperandSet, UseID, UseKind, ValueSSA,
        constant::expr::{ExprCommon, ISubExpr},
    },
    typing::{IValType, StructTypeID, TypeContext, ValTypeID},
};
use smallvec::SmallVec;
use std::ops::RangeFull;

#[derive(Clone)]
pub struct StructExpr {
    pub common: ExprCommon,
    pub structty: StructTypeID,
    pub fields: SmallVec<[UseID; 4]>,
}
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
    fn get_expr_type(&self) -> ValTypeID {
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

    fn is_zero_const(&self, allocs: &IRAllocs) -> bool {
        if self.fields.is_empty() {
            return true;
        }
        self.fields
            .iter()
            .all(|f| f.get_operand(allocs).is_zero_const(allocs))
    }
}
impl StructExpr {
    pub const OP_FIELDS: RangeFull = RangeFull;

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

_remusys_ir_subexpr!(StructExprID, StructExpr);
impl StructExprID {
    pub fn new_uninit(allocs: &IRAllocs, tctx: &TypeContext, structty: StructTypeID) -> Self {
        let expr = StructExpr::new_uninit(allocs, tctx, structty);
        Self::allocate(allocs, expr)
    }

    pub fn get_struct_type(self, allocs: &IRAllocs) -> StructTypeID {
        self.deref_ir(allocs).structty
    }
    pub fn field_uses(self, allocs: &IRAllocs) -> &[UseID] {
        &self.deref_ir(allocs).fields
    }

    pub fn get_field_use(self, allocs: &IRAllocs, idx: usize) -> UseID {
        self.deref_ir(allocs).fields[idx]
    }
    pub fn get_field(self, allocs: &IRAllocs, idx: usize) -> ValueSSA {
        self.get_field_use(allocs, idx).get_operand(allocs)
    }
    pub fn set_field(self, allocs: &IRAllocs, idx: usize, val: ValueSSA) {
        self.get_field_use(allocs, idx).set_operand(allocs, val);
    }
}

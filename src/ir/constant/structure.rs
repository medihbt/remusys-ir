use crate::{
    ir::{
        ExprObj, IRAllocs, IUser, OperandSet, UseID, UseKind,
        constant::expr::{ExprCommon, ISubExpr},
    },
    typing::{StructTypeID, TypeContext},
};

#[derive(Clone)]
pub struct StructExpr {
    pub common: ExprCommon,
    pub structty: StructTypeID,
    pub fields: Box<[UseID]>,
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
            let mut fields = Vec::with_capacity(nfields);
            for i in 0..nfields {
                let use_id = UseID::new(UseKind::StructField(i), allocs);
                fields.push(use_id);
            }
            fields.into_boxed_slice()
        };
        Self { common: ExprCommon::none(), structty, fields }
    }
}

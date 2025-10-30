use crate::{
    ir::{
        ExprObj, IRAllocs, IUser, OperandSet, UseID, UseKind,
        constant::expr::{ExprCommon, ISubExpr},
    },
    typing::{ArrayTypeID, TypeContext, ValTypeID},
};

#[derive(Clone)]
pub struct ArrayExpr {
    pub common: ExprCommon,
    pub arrty: ArrayTypeID,
    pub elemty: ValTypeID,
    pub elems: Box<[UseID]>,
}
impl IUser for ArrayExpr {
    fn get_operands(&self) -> OperandSet<'_> {
        OperandSet::Fixed(&self.elems)
    }
    fn operands_mut(&mut self) -> &mut [UseID] {
        &mut self.elems
    }
}
impl ISubExpr for ArrayExpr {
    fn get_common(&self) -> &ExprCommon {
        &self.common
    }
    fn common_mut(&mut self) -> &mut ExprCommon {
        &mut self.common
    }
    fn try_from_ir_ref(expr: &ExprObj) -> Option<&Self> {
        if let ExprObj::Array(arr) = expr { Some(arr) } else { None }
    }
    fn try_from_ir_mut(expr: &mut ExprObj) -> Option<&mut Self> {
        if let ExprObj::Array(arr) = expr { Some(arr) } else { None }
    }
    fn try_from_ir(expr: ExprObj) -> Option<Self> {
        if let ExprObj::Array(arr) = expr { Some(arr) } else { None }
    }
    fn into_ir(self) -> ExprObj {
        ExprObj::Array(self)
    }
}
impl ArrayExpr {
    pub fn new_uninit(allocs: &IRAllocs, tctx: &TypeContext, arrty: ArrayTypeID) -> Self {
        let elemty = arrty.get_element_type(tctx);
        let nelems = arrty.get_num_elements(tctx);

        let elems = {
            let mut elems = Vec::with_capacity(nelems);
            for i in 0..nelems {
                let use_id = UseID::new(UseKind::ArrayElem(i), allocs);
                elems.push(use_id);
            }
            elems.into_boxed_slice()
        };
        Self { common: ExprCommon::none(), arrty, elemty, elems }
    }
}

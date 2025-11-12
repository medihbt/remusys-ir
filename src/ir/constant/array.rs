use crate::{
    impl_traceable_from_common,
    ir::{
        ExprObj, IRAllocs, ISubExprID, IUser, OperandSet, UseID, UseKind,
        constant::expr::{ExprCommon, ExprRawPtr, ISubExpr},
    },
    typing::{ArrayTypeID, IValType, TypeContext, ValTypeID},
};
use smallvec::SmallVec;

#[derive(Clone)]
pub struct ArrayExpr {
    pub common: ExprCommon,
    pub arrty: ArrayTypeID,
    pub elemty: ValTypeID,
    pub elems: SmallVec<[UseID; 4]>,
}
impl_traceable_from_common!(ArrayExpr, false);
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
    fn get_valtype(&self) -> ValTypeID {
        self.arrty.into_ir()
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
            let mut elems = SmallVec::with_capacity(nelems);
            for i in 0..nelems {
                let use_id = UseID::new(allocs, UseKind::ArrayElem(i));
                elems.push(use_id);
            }
            elems
        };
        Self { common: ExprCommon::none(), arrty, elemty, elems }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArrayExprID(pub ExprRawPtr);

impl ISubExprID for ArrayExprID {
    type ExprObjT = ArrayExpr;

    fn from_raw_ptr(id: ExprRawPtr) -> Self {
        ArrayExprID(id)
    }
    fn into_raw_ptr(self) -> ExprRawPtr {
        self.0
    }
}
impl ArrayExprID {
    pub fn new_uninit(allocs: &IRAllocs, tctx: &TypeContext, arrty: ArrayTypeID) -> Self {
        Self::allocate(allocs, ArrayExpr::new_uninit(allocs, tctx, arrty))
    }

    pub fn get_arrty(self, allocs: &IRAllocs) -> ArrayTypeID {
        self.deref_ir(allocs).arrty
    }
    pub fn get_elemty(self, allocs: &IRAllocs) -> ValTypeID {
        self.deref_ir(allocs).elemty
    }
    pub fn get_elems(self, allocs: &IRAllocs) -> &[UseID] {
        &self.deref_ir(allocs).elems
    }
}

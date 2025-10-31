use crate::{
    impl_traceable_from_common,
    ir::{ExprCommon, ExprID, ExprObj, IRAllocs, ISubExpr, ISubExprID, IUser, OperandSet, UseID},
    typing::{FixVecType, IValType, ValTypeID},
};
use mtb_entity::PtrID;

#[derive(Clone)]
pub struct FixVec {
    pub common: ExprCommon,
    pub elems: Box<[UseID]>,
    pub vecty: FixVecType,
}

impl_traceable_from_common!(FixVec, false);
impl IUser for FixVec {
    fn get_operands(&self) -> OperandSet<'_> {
        OperandSet::Fixed(&self.elems)
    }
    fn operands_mut(&mut self) -> &mut [UseID] {
        &mut self.elems
    }
}
impl ISubExpr for FixVec {
    fn get_common(&self) -> &ExprCommon {
        &self.common
    }
    fn common_mut(&mut self) -> &mut ExprCommon {
        &mut self.common
    }

    fn get_valtype(&self) -> ValTypeID {
        self.vecty.into_ir()
    }

    fn try_from_ir_ref(expr: &ExprObj) -> Option<&Self> {
        match expr {
            ExprObj::FixVec(vec) => Some(vec),
            _ => None,
        }
    }
    fn try_from_ir_mut(expr: &mut ExprObj) -> Option<&mut Self> {
        match expr {
            ExprObj::FixVec(vec) => Some(vec),
            _ => None,
        }
    }
    fn try_from_ir(expr: ExprObj) -> Option<Self>
    where
        Self: Sized,
    {
        match expr {
            ExprObj::FixVec(vec) => Some(vec),
            _ => None,
        }
    }
    fn into_ir(self) -> ExprObj {
        ExprObj::FixVec(self)
    }
}
impl FixVec {
    pub fn new_uninit(allocs: &IRAllocs, vecty: FixVecType) -> Self {
        let nelems = vecty.get_len();
        let elems = {
            let mut elems = Vec::with_capacity(nelems);
            for i in 0..nelems {
                let use_id = UseID::new(crate::ir::UseKind::VecElem(i), allocs);
                elems.push(use_id);
            }
            elems.into_boxed_slice()
        };
        Self { common: ExprCommon::none(), elems, vecty }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FixVecID(pub ExprID);

impl ISubExprID for FixVecID {
    type ExprObjT = FixVec;

    fn raw_from_ir(id: PtrID<ExprObj>) -> Self {
        FixVecID(id)
    }
    fn into_ir(self) -> PtrID<ExprObj> {
        self.0
    }
}

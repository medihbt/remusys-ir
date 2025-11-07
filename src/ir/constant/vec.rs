use crate::{
    impl_traceable_from_common,
    ir::{
        ExprCommon, ExprID, ExprObj, IRAllocs, ISubExpr, ISubExprID, IUser, OperandSet, UseID,
        UseKind,
    },
    typing::{FixVecType, IValType, ValTypeID},
};
use mtb_entity_slab::PtrID;
use smallvec::SmallVec;

#[derive(Clone)]
pub struct FixVec {
    pub common: ExprCommon,
    pub elems: SmallVec<[UseID; 4]>,
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
    fn try_from_ir(expr: ExprObj) -> Option<Self> {
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
            let mut elems = SmallVec::with_capacity(nelems);
            for i in 0..nelems {
                let use_id = UseID::new(allocs, UseKind::VecElem(i));
                elems.push(use_id);
            }
            elems
        };
        Self { common: ExprCommon::none(), elems, vecty }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FixVecID(pub ExprID);

impl ISubExprID for FixVecID {
    type ExprObjT = FixVec;

    fn raw_from_expr(id: PtrID<ExprObj>) -> Self {
        FixVecID(id)
    }
    fn into_expr(self) -> PtrID<ExprObj> {
        self.0
    }
}

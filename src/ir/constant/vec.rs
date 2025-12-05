use crate::{
    impl_traceable_from_common,
    ir::{
        ExprCommon, ExprObj, IRAllocs, ISubExpr, ISubExprID, ISubValueSSA, IUser, OperandSet,
        UseID, UseKind, ValueSSA, constant::expr::ExprRawPtr,
    },
    typing::{FixVecType, IValType, ScalarType, ValTypeID},
};
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
    fn is_zero_const(&self, allocs: &IRAllocs) -> bool {
        if self.elems.is_empty() {
            return true;
        }
        self.elems
            .iter()
            .all(|e| e.get_operand(allocs).is_zero_const(allocs))
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
pub struct FixVecID(pub ExprRawPtr);

impl ISubExprID for FixVecID {
    type ExprObjT = FixVec;

    fn from_raw_ptr(id: ExprRawPtr) -> Self {
        FixVecID(id)
    }
    fn into_raw_ptr(self) -> ExprRawPtr {
        self.0
    }
}
impl FixVecID {
    pub fn new_uninit(allocs: &IRAllocs, vecty: FixVecType) -> Self {
        let fixvec = FixVec::new_uninit(allocs, vecty);
        Self::allocate(allocs, fixvec)
    }

    pub fn get_vecty(self, allocs: &IRAllocs) -> FixVecType {
        self.deref_ir(allocs).vecty
    }
    pub fn get_elemty(self, allocs: &IRAllocs) -> ScalarType {
        self.get_vecty(allocs).0
    }
    pub fn get_nelems(self, allocs: &IRAllocs) -> usize {
        self.get_vecty(allocs).get_len()
    }

    pub fn elem_uses(self, allocs: &IRAllocs) -> &[UseID] {
        &self.deref_ir(allocs).elems
    }
    pub fn get_elem(self, allocs: &IRAllocs, index: usize) -> ValueSSA {
        self.elem_uses(allocs)[index].get_operand(allocs)
    }
    pub fn set_elem(self, allocs: &IRAllocs, index: usize, val: ValueSSA) -> bool {
        self.elem_uses(allocs)[index].set_operand(allocs, val)
    }
}

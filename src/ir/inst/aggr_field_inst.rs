use smallvec::SmallVec;
use thiserror::Error;

use crate::{
    ir::{IRAllocs, ISubInstID, ValueSSA, inst::IAggrFieldInst},
    typing::{AggrType, IValType, TypeContext, ValTypeID},
};

#[derive(Debug, Error)]
pub enum AggrFieldInstBuildErr {
    #[error("cannot extract field {0} from non-aggregate type {1:?}")]
    ConstructionFinished(u32, ValTypeID),

    #[error("field index {0} out of bounds for aggregate type {1:?}")]
    IndexOutofBounds(u32, AggrType),
}

pub type AggrFieldInstBuildRes<T = ()> = Result<T, AggrFieldInstBuildErr>;

pub trait IAggrFieldInstBuildable: Clone {
    type InstT: IAggrFieldInst + 'static;
    type InstID: ISubInstID<InstObjT = Self::InstT>;

    fn new(aggr_type: AggrType) -> Self;

    fn common(&self) -> &AggrFieldInstBuilderCommon;
    fn common_mut(&mut self) -> &mut AggrFieldInstBuilderCommon;

    fn try_add_step(&mut self, tctx: &TypeContext, index: u32) -> AggrFieldInstBuildRes<&mut Self> {
        let inner = self.common_mut();
        let currty = if inner.steps.is_empty() {
            inner.aggr_type.into_ir()
        } else {
            inner.steps.last().unwrap().1
        };
        let curr_aggr_ty = match AggrType::try_from_ir(currty) {
            Ok(aggr) => aggr,
            Err(_) => return Err(AggrFieldInstBuildErr::ConstructionFinished(index, currty)),
        };
        let Some(field_ty) = curr_aggr_ty.try_get_field(tctx, index as usize) else {
            return Err(AggrFieldInstBuildErr::IndexOutofBounds(index, curr_aggr_ty));
        };
        inner.steps.push((index, field_ty));
        Ok(self)
    }
    fn try_add_steps(
        &mut self,
        tctx: &TypeContext,
        indices: impl IntoIterator<Item = u32>,
    ) -> AggrFieldInstBuildRes<&mut Self> {
        for idx in indices.into_iter() {
            self.try_add_step(tctx, idx)?;
        }
        Ok(self)
    }
    fn add_step(&mut self, tctx: &TypeContext, index: u32) -> &mut Self {
        self.try_add_step(tctx, index)
            .expect("Failed to add step to FieldExtractBuilder")
    }
    fn add_steps(
        &mut self,
        tctx: &TypeContext,
        indices: impl IntoIterator<Item = u32>,
    ) -> &mut Self {
        self.try_add_steps(tctx, indices)
            .expect("Failed to add steps to FieldExtractBuilder")
    }

    fn build_obj(&mut self, allocs: &IRAllocs) -> Self::InstT;
    fn build_id(&mut self, allocs: &IRAllocs) -> Self::InstID {
        let inst = self.build_obj(allocs);
        Self::InstID::allocate(allocs, inst)
    }
}

#[derive(Clone)]
pub struct AggrFieldInstBuilderCommon {
    pub aggr: ValueSSA,
    pub aggr_type: AggrType,
    pub steps: SmallVec<[(u32, ValTypeID); 8]>,
}
impl AggrFieldInstBuilderCommon {
    pub fn new(aggr: ValueSSA, aggr_type: AggrType) -> Self {
        Self { aggr, aggr_type, steps: SmallVec::new() }
    }
}

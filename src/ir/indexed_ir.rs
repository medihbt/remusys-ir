use crate::{
    base::INullableValue,
    ir::{
        BlockIndex, ConstData, ExprIndex, FuncID, GlobalIndex, IRAllocs, ISubGlobalID, ISubInstID,
        ITraceableValue, IUser, InstID, InstIndex, InstObj, JumpTargetIndex, OperandSet, UseIndex,
        UseKind, UserList, ValueSSA,
    },
    typing::AggrType,
};
use mtb_entity_slab::IEntityAllocID;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(C, u8)]
pub enum IndexedValue {
    None,
    ConstData(ConstData),
    ConstExpr(ExprIndex),
    FuncArg(GlobalIndex, u32),
    AggrZero(AggrType),
    Block(BlockIndex),
    Inst(InstIndex),
    Global(GlobalIndex),
}
impl INullableValue for IndexedValue {
    fn is_null(&self) -> bool {
        matches!(self, IndexedValue::None)
    }
    fn new_null() -> Self {
        IndexedValue::None
    }
}
impl IndexedValue {
    pub fn try_from_value(value: ValueSSA, allocs: &IRAllocs) -> Option<Self> {
        match value {
            ValueSSA::None => Some(IndexedValue::None),
            ValueSSA::ConstData(data) => Some(IndexedValue::ConstData(data)),
            ValueSSA::ConstExpr(expr_id) => expr_id.as_indexed(allocs).map(Self::ConstExpr),
            ValueSSA::AggrZero(aggr) => Some(IndexedValue::AggrZero(aggr)),
            ValueSSA::FuncArg(func_id, index) => func_id
                .as_indexed(allocs)
                .map(|x| IndexedValue::FuncArg(x, index)),
            ValueSSA::Block(block_id) => block_id.as_indexed(allocs).map(IndexedValue::Block),
            ValueSSA::Inst(inst_id) => inst_id.as_indexed(allocs).map(IndexedValue::Inst),
            ValueSSA::Global(global_id) => global_id.as_indexed(allocs).map(IndexedValue::Global),
        }
    }

    pub fn try_into_value(self, allocs: &IRAllocs) -> Option<ValueSSA> {
        match self {
            IndexedValue::None => Some(ValueSSA::None),
            IndexedValue::ConstData(data) => Some(ValueSSA::ConstData(data)),
            IndexedValue::ConstExpr(expr_index) => {
                expr_index.as_primary(allocs).map(ValueSSA::ConstExpr)
            }
            IndexedValue::AggrZero(aggr) => Some(ValueSSA::AggrZero(aggr)),
            IndexedValue::FuncArg(func_index, index) => {
                func_index.as_primary(allocs).and_then(|x| {
                    let func_id = FuncID::try_from_global(allocs, x)?;
                    Some(ValueSSA::FuncArg(func_id, index))
                })
            }
            IndexedValue::Block(block_index) => block_index.as_primary(allocs).map(ValueSSA::Block),
            IndexedValue::Inst(inst_index) => inst_index.as_primary(allocs).map(ValueSSA::Inst),
            IndexedValue::Global(global_index) => {
                global_index.as_primary(allocs).map(ValueSSA::Global)
            }
        }
    }

    pub fn from_value(value: ValueSSA, allocs: &IRAllocs) -> Self {
        Self::try_from_value(value, allocs).expect("UAF detected")
    }
    pub fn into_value(self, allocs: &IRAllocs) -> ValueSSA {
        self.try_into_value(allocs).expect("UAF detected")
    }
}

pub trait PoolAllocatedIndex: Copy + Eq {
    type Object;
    type PrimaryID;

    fn as_primary(self, allocs: &IRAllocs) -> Option<Self::PrimaryID>;
    fn to_primary(self, allocs: &IRAllocs) -> Self::PrimaryID {
        self.as_primary(allocs).expect("UAF detected")
    }

    fn try_from_primary(primary: Self::PrimaryID, allocs: &IRAllocs) -> Option<Self>;
    fn from_primary(primary: Self::PrimaryID, allocs: &IRAllocs) -> Self {
        Self::try_from_primary(primary, allocs).expect("UAF detected")
    }

    fn try_deref_ir(self, allocs: &IRAllocs) -> Option<&Self::Object>;
    fn deref_ir(self, allocs: &IRAllocs) -> &Self::Object {
        self.try_deref_ir(allocs).expect("UAF detected")
    }
}
pub trait TraceableIndex: PoolAllocatedIndex {
    fn users_primary(self, allocs: &IRAllocs) -> &UserList;
    fn user_uses<T>(self, allocs: &IRAllocs) -> T
    where
        T: FromIterator<UseIndex>,
    {
        let iter = self
            .users_primary(allocs)
            .iter(&allocs.uses)
            .filter_map(|(user_use, _)| user_use.as_indexed(allocs));
        iter.collect()
    }
}
pub trait UserIndex: TraceableIndex {
    fn get_primary_uses(self, allocs: &IRAllocs) -> OperandSet<'_>;

    fn get_use_by_kind(self, allocs: &IRAllocs, kind: UseKind) -> Option<UseIndex> {
        let &primary_u = self
            .get_primary_uses(allocs)
            .iter()
            .find(|&uid| uid.get_kind(allocs) == kind)?;
        primary_u.as_indexed(allocs)
    }
    fn get_operand_use(self, allocs: &IRAllocs, index: usize) -> Option<UseIndex> {
        let &primary_u = self.get_primary_uses(allocs).get(index)?;
        primary_u.as_indexed(allocs)
    }
}
impl PoolAllocatedIndex for InstIndex {
    type Object = InstObj;
    type PrimaryID = InstID;

    fn as_primary(self, allocs: &IRAllocs) -> Option<InstID> {
        let ptr = self.0.to_ptr(&allocs.insts)?;
        Some(InstID(ptr))
    }
    fn try_from_primary(primary: InstID, allocs: &IRAllocs) -> Option<Self> {
        primary.as_indexed(allocs)
    }
    fn try_deref_ir(self, allocs: &IRAllocs) -> Option<&Self::Object> {
        self.0.try_deref(&allocs.insts)
    }
}
impl PoolAllocatedIndex for BlockIndex {
    type Object = crate::ir::BlockObj;
    type PrimaryID = crate::ir::BlockID;

    fn as_primary(self, allocs: &IRAllocs) -> Option<Self::PrimaryID> {
        let ptr = self.0.to_ptr(&allocs.blocks)?;
        Some(crate::ir::BlockID(ptr))
    }
    fn try_from_primary(primary: Self::PrimaryID, allocs: &IRAllocs) -> Option<Self> {
        primary.as_indexed(allocs)
    }
    fn try_deref_ir(self, allocs: &IRAllocs) -> Option<&Self::Object> {
        self.0.try_deref(&allocs.blocks)
    }
}
impl PoolAllocatedIndex for ExprIndex {
    type Object = crate::ir::constant::expr::ExprObj;
    type PrimaryID = crate::ir::constant::expr::ExprID;

    fn as_primary(self, allocs: &IRAllocs) -> Option<Self::PrimaryID> {
        let ptr = self.0.to_ptr(&allocs.exprs)?;
        Some(crate::ir::constant::expr::ExprID(ptr))
    }
    fn try_from_primary(primary: Self::PrimaryID, allocs: &IRAllocs) -> Option<Self> {
        primary.as_indexed(allocs)
    }
    fn try_deref_ir(self, allocs: &IRAllocs) -> Option<&Self::Object> {
        self.0.try_deref(&allocs.exprs)
    }
}
impl PoolAllocatedIndex for GlobalIndex {
    type Object = crate::ir::GlobalObj;
    type PrimaryID = crate::ir::GlobalID;

    fn as_primary(self, allocs: &IRAllocs) -> Option<Self::PrimaryID> {
        // GlobalIndex -> GlobalID (policed pointer)
        let ptr = self.0.to_ptr(&allocs.globals)?;
        Some(crate::ir::GlobalID(ptr))
    }
    fn try_from_primary(primary: Self::PrimaryID, allocs: &IRAllocs) -> Option<Self> {
        primary.as_indexed(allocs)
    }
    fn try_deref_ir(self, allocs: &IRAllocs) -> Option<&Self::Object> {
        self.0.try_deref(&allocs.globals)
    }
}
impl PoolAllocatedIndex for UseIndex {
    type Object = crate::ir::Use;
    type PrimaryID = crate::ir::UseID;

    fn as_primary(self, allocs: &IRAllocs) -> Option<Self::PrimaryID> {
        let ptr = self.0.to_ptr(&allocs.uses)?;
        Some(crate::ir::UseID(ptr))
    }
    fn try_from_primary(primary: Self::PrimaryID, allocs: &IRAllocs) -> Option<Self> {
        primary.as_indexed(allocs)
    }
    fn try_deref_ir(self, allocs: &IRAllocs) -> Option<&Self::Object> {
        self.0.try_deref(&allocs.uses)
    }
}
impl PoolAllocatedIndex for JumpTargetIndex {
    type Object = crate::ir::JumpTarget;
    type PrimaryID = crate::ir::JumpTargetID;

    fn as_primary(self, allocs: &IRAllocs) -> Option<Self::PrimaryID> {
        self.0.to_ptr(&allocs.jts).map(crate::ir::JumpTargetID)
    }
    fn try_from_primary(primary: Self::PrimaryID, allocs: &IRAllocs) -> Option<Self> {
        primary.as_indexed(allocs)
    }
    fn try_deref_ir(self, allocs: &IRAllocs) -> Option<&Self::Object> {
        self.0.try_deref(&allocs.jts)
    }
}

impl<T> TraceableIndex for T
where
    T: PoolAllocatedIndex,
    T::Object: ITraceableValue + 'static,
{
    fn users_primary(self, allocs: &IRAllocs) -> &UserList {
        self.deref_ir(allocs).users()
    }
}
impl<T> UserIndex for T
where
    T: TraceableIndex,
    T::Object: IUser + 'static,
{
    fn get_primary_uses(self, allocs: &IRAllocs) -> OperandSet<'_> {
        self.deref_ir(allocs).get_operands()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ir::ISubInst,
        opt::{CfgBlockStat, CfgDfsSeq},
        testing::cases::test_case_cfg_deep_while_br,
    };

    #[test]
    fn test_indexed_value_conversion() {
        let module = test_case_cfg_deep_while_br().module;
        let allocs = &module.allocs;
        for &func in module.symbols.borrow().func_pool() {
            let gidx = func.to_indexed(allocs);
            assert_eq!(gidx.to_primary(allocs), func.raw_into());

            let Ok(dfs) = CfgDfsSeq::new_pre(allocs, func) else {
                continue;
            };
            for node in &dfs.nodes {
                let CfgBlockStat::Block(block) = node.block else {
                    continue;
                };
                let bidx = block.to_indexed(allocs);
                assert_eq!(bidx.to_primary(allocs), block);

                for (inst_id, inst) in block.insts_iter(allocs) {
                    let iidx = inst_id.to_indexed(allocs);
                    assert_eq!(iidx.to_primary(allocs), inst_id);

                    for opuse in inst.operands_iter() {
                        let ouidx = opuse.to_indexed(allocs);
                        assert_eq!(ouidx.to_primary(allocs), opuse);
                        let validx = ouidx.get_operand(allocs);
                        let val = opuse.get_operand(allocs);
                        assert_eq!(validx.into_value(allocs), val);
                    }

                    let Some(jts) = inst.try_get_jts() else {
                        continue;
                    };
                    for &jt in jts.iter() {
                        let jidx = jt.to_indexed(allocs);
                        assert_eq!(jidx.to_primary(allocs), jt);

                        let jt_obj = jidx.deref_ir(allocs);
                        assert_eq!(jt_obj as *const _, jt.deref_ir(allocs) as *const _);
                    }
                }
            }
        }
    }
}

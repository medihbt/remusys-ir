use crate::{
    ir::{
        BlockIndex, ConstData, ExprIndex, FuncID, GlobalIndex, IRAllocs, ISubGlobalID, ISubInstID,
        InstIndex, OperandSet, UseIndex, UseKind, ValueSSA,
    },
    typing::AggrType,
};

#[derive(Debug, Clone, Copy, PartialEq, Hash)]
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

pub trait IndexedUserID: Copy {
    fn get_operand_uses_primary(self, allocs: &IRAllocs) -> OperandSet<'_>;

    fn get_operand_use_by_kind(self, allocs: &IRAllocs, kind: UseKind) -> Option<UseIndex> {
        let &primary_u = self
            .get_operand_uses_primary(allocs)
            .iter()
            .find(|&uid| uid.get_kind(allocs) == kind)?;
        primary_u.as_indexed(allocs)
    }
    fn get_operand_use(self, allocs: &IRAllocs, index: usize) -> Option<UseIndex> {
        let &primary_u = self.get_operand_uses_primary(allocs).get(index)?;
        primary_u.as_indexed(allocs)
    }
}

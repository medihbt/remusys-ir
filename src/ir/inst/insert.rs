use crate::{
    impl_debug_for_subinst_id, impl_traceable_from_common,
    ir::{
        IRAllocs, ISubInst, ISubInstID, ISubValueSSA, IUser, InstCommon, InstID, InstObj,
        JumpTargets, Opcode, OperandSet, UseID, UseKind, ValueSSA,
        inst::{
            AggrFieldInstBuilderCommon, IAggrFieldInst, IAggrFieldInstBuildable, IAggrIndexInst,
            IAggregateInst,
        },
    },
    typing::{AggrType, IValType, TypeContext, ValTypeID},
};
use smallvec::SmallVec;

/// 把数组 / 向量值 a 中的索引位 i 替换成元素 v 并返回新的数组 / 向量值。
///
/// ### LLVM IR 语法
///
/// ```llvm
/// %<result> = insertelement <aggr_type> %<aggr>, <elem_type> %<elem>, <index_type> %<index>
/// ```
///
/// ### 操作数布局
///
/// - `operand[0] = 'aggr'`: 被插入元素的数组 / 向量值。
/// - `operand[1] = 'elem'`: 要插入的元素值。
/// - `operand[2] = 'index'`: 要插入元素的位置索引
///
/// ### 返回类型
///
/// 返回类型与操作数 `aggr` 的类型相同。
pub struct IndexInsertInst {
    pub common: InstCommon,
    operands: [UseID; 3],
    pub elem_type: ValTypeID,
}
impl_traceable_from_common!(IndexInsertInst, true);
impl IUser for IndexInsertInst {
    fn get_operands(&self) -> OperandSet<'_> {
        OperandSet::Fixed(&self.operands)
    }
    fn operands_mut(&mut self) -> &mut [UseID] {
        &mut self.operands
    }
}
impl ISubInst for IndexInsertInst {
    fn get_common(&self) -> &InstCommon {
        &self.common
    }
    fn common_mut(&mut self) -> &mut InstCommon {
        &mut self.common
    }

    fn try_from_ir_ref(inst: &InstObj) -> Option<&Self> {
        match inst {
            InstObj::IndexInsert(i) => Some(i),
            _ => None,
        }
    }
    fn try_from_ir_mut(inst: &mut InstObj) -> Option<&mut Self> {
        match inst {
            InstObj::IndexInsert(i) => Some(i),
            _ => None,
        }
    }
    fn try_from_ir(inst: InstObj) -> Option<Self> {
        match inst {
            InstObj::IndexInsert(i) => Some(i),
            _ => None,
        }
    }
    fn into_ir(self) -> InstObj {
        InstObj::IndexInsert(self)
    }
    fn try_get_jts(&self) -> Option<JumpTargets<'_>> {
        None
    }
}
impl IAggregateInst for IndexInsertInst {
    fn get_aggr_operand_type(&self) -> AggrType {
        match self.get_valtype() {
            ValTypeID::Array(a) => AggrType::Array(a),
            ValTypeID::FixVec(v) => AggrType::FixVec(v),
            _ => panic!("IndexInsertInst's aggregate operand must be Array or Vector"),
        }
    }
    fn get_elem_type(&self) -> ValTypeID {
        self.elem_type
    }
    fn aggr_use(&self) -> UseID {
        self.operands[0]
    }
}
impl IAggrIndexInst for IndexInsertInst {
    fn index_use(&self) -> UseID {
        self.operands[2]
    }

    fn new_uninit(allocs: &IRAllocs, tctx: &TypeContext, aggr_type: AggrType) -> Self {
        let elem_type = match aggr_type {
            AggrType::Array(a) => a.get_element_type(tctx),
            AggrType::FixVec(v) => v.get_elem().into_ir(),
            _ => panic!("IndexInsertInst's aggregate operand must be Array or Vector"),
        };
        Self {
            common: InstCommon::new(Opcode::IndexInsert, aggr_type.into_ir()),
            operands: [
                UseID::new(allocs, UseKind::IndexExtractAggr),
                UseID::new(allocs, UseKind::IndexInsertElem),
                UseID::new(allocs, UseKind::IndexInsertIndex),
            ],
            elem_type,
        }
    }
}
impl IndexInsertInst {
    pub const OP_AGGR: usize = 0;
    pub const OP_ELEM: usize = 1;
    pub const OP_INDEX: usize = 2;

    pub fn elem_use(&self) -> UseID {
        self.operands[Self::OP_ELEM]
    }
    pub fn get_elem(&self, allocs: &IRAllocs) -> ValueSSA {
        self.elem_use().get_operand(allocs)
    }
    pub fn set_elem(&self, allocs: &IRAllocs, val: ValueSSA) {
        self.elem_use().set_operand(allocs, val);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct IndexInsertInstID(pub InstID);
impl_debug_for_subinst_id!(IndexInsertInstID);
impl ISubInstID for IndexInsertInstID {
    type InstObjT = IndexInsertInst;

    fn raw_from_instid(id: InstID) -> Self {
        IndexInsertInstID(id)
    }
    fn into_instid(self) -> InstID {
        self.0
    }
}
impl IndexInsertInstID {
    pub fn aggr_use(self, allocs: &IRAllocs) -> UseID {
        self.deref_ir(allocs).aggr_use()
    }
    pub fn get_aggr(self, allocs: &IRAllocs) -> ValueSSA {
        self.deref_ir(allocs).get_aggr(allocs)
    }
    pub fn set_aggr(self, allocs: &IRAllocs, val: ValueSSA) {
        self.deref_ir(allocs).set_aggr(allocs, val);
    }

    pub fn aggr_type(self, allocs: &IRAllocs) -> AggrType {
        self.deref_ir(allocs).get_aggr_operand_type()
    }
    pub fn get_elem_type(self, allocs: &IRAllocs) -> ValTypeID {
        self.deref_ir(allocs).get_elem_type()
    }
    pub fn get_index(self, allocs: &IRAllocs) -> ValueSSA {
        self.deref_ir(allocs).get_index(allocs)
    }
    pub fn set_index(self, allocs: &IRAllocs, val: ValueSSA) {
        self.deref_ir(allocs).set_index(allocs, val);
    }
}

/// 把数组 / 结构体 / 向量聚合值 a 中的指定字段替换成元素 v 并返回新的聚合值。
/// 字段位置通过常量索引链指定。
///
/// ### LLVM IR 语法
///
/// ```llvm
/// %<result> = insertvalue <aggr_type> %<aggr>, <elem_type> %<elem>, <index1>, <index2>, ...
/// ```
///
/// ### 操作数布局
///
/// - `operand[0] = 'aggr'`: 被插入元素的聚合值。
/// - `operand[1] = 'elem'`: 要插入的元素值。
///
/// ### 返回类型
///
/// 返回类型与操作数 `aggr` 的类型相同。
pub struct FieldInsertInst {
    pub common: InstCommon,
    operands: [UseID; 2],
    pub fields: SmallVec<[u32; 4]>,
    pub elem_type: ValTypeID,
}
impl_traceable_from_common!(FieldInsertInst, true);
impl IUser for FieldInsertInst {
    fn get_operands(&self) -> OperandSet<'_> {
        OperandSet::Fixed(&self.operands)
    }
    fn operands_mut(&mut self) -> &mut [UseID] {
        &mut self.operands
    }
}
impl ISubInst for FieldInsertInst {
    fn get_common(&self) -> &InstCommon {
        &self.common
    }
    fn common_mut(&mut self) -> &mut InstCommon {
        &mut self.common
    }

    fn try_from_ir_ref(inst: &InstObj) -> Option<&Self> {
        match inst {
            InstObj::FieldInsert(i) => Some(i),
            _ => None,
        }
    }
    fn try_from_ir_mut(inst: &mut InstObj) -> Option<&mut Self> {
        match inst {
            InstObj::FieldInsert(i) => Some(i),
            _ => None,
        }
    }
    fn try_from_ir(inst: InstObj) -> Option<Self> {
        match inst {
            InstObj::FieldInsert(i) => Some(i),
            _ => None,
        }
    }
    fn into_ir(self) -> InstObj {
        InstObj::FieldInsert(self)
    }
    fn try_get_jts(&self) -> Option<JumpTargets<'_>> {
        None
    }
}
impl IAggregateInst for FieldInsertInst {
    fn get_aggr_operand_type(&self) -> AggrType {
        match self.get_valtype() {
            ValTypeID::Array(a) => AggrType::Array(a),
            ValTypeID::FixVec(v) => AggrType::FixVec(v),
            ValTypeID::Struct(s) => AggrType::Struct(s),
            _ => panic!("FieldInsertInst's aggregate operand must be Array, Struct or Vector"),
        }
    }
    fn get_elem_type(&self) -> ValTypeID {
        self.elem_type
    }
    fn aggr_use(&self) -> UseID {
        self.operands[0]
    }
}
impl IAggrFieldInst for FieldInsertInst {
    type DefaultBuilderT = FieldInsertBuilder;

    fn get_field_indices(&self) -> &[u32] {
        &self.fields
    }
}
impl FieldInsertInst {
    pub const OP_AGGR: usize = 0;
    pub const OP_ELEM: usize = 1;

    pub fn builder(aggr_type: AggrType) -> FieldInsertBuilder {
        Self::default_builder(aggr_type)
    }

    pub fn elem_use(&self) -> UseID {
        self.operands[Self::OP_ELEM]
    }
    pub fn get_elem(&self, allocs: &IRAllocs) -> ValueSSA {
        self.elem_use().get_operand(allocs)
    }
    pub fn set_elem(&self, allocs: &IRAllocs, val: ValueSSA) {
        self.elem_use().set_operand(allocs, val);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FieldInsertInstID(pub InstID);
impl_debug_for_subinst_id!(FieldInsertInstID);
impl ISubInstID for FieldInsertInstID {
    type InstObjT = FieldInsertInst;

    fn raw_from_instid(id: InstID) -> Self {
        FieldInsertInstID(id)
    }
    fn into_instid(self) -> InstID {
        self.0
    }
}
impl FieldInsertInstID {
    pub fn builder(aggr_type: AggrType) -> FieldInsertBuilder {
        FieldInsertBuilder::new(aggr_type)
    }

    pub fn aggr_use(self, allocs: &IRAllocs) -> UseID {
        self.deref_ir(allocs).aggr_use()
    }
    pub fn get_aggr(self, allocs: &IRAllocs) -> ValueSSA {
        self.deref_ir(allocs).get_aggr(allocs)
    }
    pub fn set_aggr(self, allocs: &IRAllocs, val: ValueSSA) {
        self.deref_ir(allocs).set_aggr(allocs, val);
    }

    pub fn aggr_type(self, allocs: &IRAllocs) -> AggrType {
        self.deref_ir(allocs).get_aggr_operand_type()
    }
    pub fn get_field_indices(self, allocs: &IRAllocs) -> &[u32] {
        &self.deref_ir(allocs).fields
    }
}

#[derive(Clone)]
pub struct FieldInsertBuilder {
    common: AggrFieldInstBuilderCommon,
    elem: ValueSSA,
}
impl IAggrFieldInstBuildable for FieldInsertBuilder {
    type InstT = FieldInsertInst;
    type InstID = FieldInsertInstID;

    fn new(aggr_type: AggrType) -> Self {
        Self {
            common: AggrFieldInstBuilderCommon::new(ValueSSA::None, aggr_type),
            elem: ValueSSA::None,
        }
    }
    fn common(&self) -> &AggrFieldInstBuilderCommon {
        &self.common
    }
    fn common_mut(&mut self) -> &mut AggrFieldInstBuilderCommon {
        &mut self.common
    }
    fn build_inst(&mut self, allocs: &IRAllocs) -> Self::InstT {
        let elem = self.elem;
        let AggrFieldInstBuilderCommon { aggr, aggr_type, steps } = &self.common;
        let inst = FieldInsertInst {
            common: InstCommon::new(Opcode::FieldInsert, aggr_type.into_ir()),
            operands: [
                UseID::new(allocs, UseKind::FieldInsertAggr),
                UseID::new(allocs, UseKind::FieldInsertElem),
            ],
            fields: steps.iter().map(|(idx, _)| *idx).collect(),
            elem_type: elem.get_valtype(allocs),
        };
        if *aggr != ValueSSA::None {
            inst.set_aggr(allocs, *aggr);
        }
        if elem != ValueSSA::None {
            inst.set_elem(allocs, elem);
        }
        inst
    }
}
impl FieldInsertBuilder {
    pub fn elem(&mut self, val: ValueSSA) -> &mut Self {
        self.elem = val;
        self
    }
}

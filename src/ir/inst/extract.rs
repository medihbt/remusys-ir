use crate::{
    _remusys_ir_subinst,
    ir::{
        IRAllocs, ISubInst, ISubInstID, ISubValueSSA, ITraceableValue, IUser, InstCommon, InstObj,
        JumpTargets, Opcode, OperandSet, UseID, UseKind, ValueSSA,
        inst::{
            AggrFieldInstBuilderCommon, IAggrFieldInst, IAggrFieldInstBuildable, IAggrIndexInst,
            IAggregateInst,
        },
    },
    typing::{AggrType, IValType, TypeContext, ValTypeID},
};
use smallvec::SmallVec;

/// 从数组 / 向量聚合值 a 中提取变量索引 i 处的值.
///
/// ### LLVM IR 语法
///
/// ```llvm
/// %<id> = extractelement <aggr_type> %<aggr>, %<ty> %<index>
/// ```
///
/// ### 操作数布局
///
/// - `operands[0]` - 聚合值
/// - `operands[1]` - 索引值, 要求整数类型
pub struct IndexExtractInst {
    pub common: InstCommon,
    operands: [UseID; 2],
    pub aggr_type: AggrType,
}

impl IUser for IndexExtractInst {
    fn get_operands(&self) -> OperandSet<'_> {
        OperandSet::Fixed(&self.operands)
    }
    fn operands_mut(&mut self) -> &mut [UseID] {
        &mut self.operands
    }
}
impl ISubInst for IndexExtractInst {
    fn get_common(&self) -> &InstCommon {
        &self.common
    }
    fn common_mut(&mut self) -> &mut InstCommon {
        &mut self.common
    }
    fn try_from_ir_ref(inst: &InstObj) -> Option<&Self> {
        match inst {
            InstObj::IndexExtract(e) => Some(e),
            _ => None,
        }
    }
    fn try_from_ir_mut(inst: &mut InstObj) -> Option<&mut Self> {
        match inst {
            InstObj::IndexExtract(e) => Some(e),
            _ => None,
        }
    }
    fn try_from_ir(inst: InstObj) -> Option<Self> {
        match inst {
            InstObj::IndexExtract(e) => Some(e),
            _ => None,
        }
    }
    fn into_ir(self) -> InstObj {
        InstObj::IndexExtract(self)
    }
    fn try_get_jts(&self) -> Option<JumpTargets<'_>> {
        None
    }
}
impl IAggregateInst for IndexExtractInst {
    fn get_aggr_operand_type(&self) -> AggrType {
        self.aggr_type
    }
    fn get_elem_type(&self) -> ValTypeID {
        self.get_valtype()
    }
    fn aggr_use(&self) -> UseID {
        self.operands[Self::OP_AGGR]
    }
}
impl IAggrIndexInst for IndexExtractInst {
    fn index_use(&self) -> UseID {
        self.operands[Self::OP_INDEX]
    }

    fn new_uninit(allocs: &IRAllocs, tctx: &TypeContext, aggr_ty: AggrType) -> Self {
        let elemty = match aggr_ty {
            AggrType::Array(a) => a.get_element_type(tctx),
            AggrType::FixVec(v) => v.get_elem().into_ir(),
            _ => panic!("IndexExtractInst requires array or vector type but got {aggr_ty:?}"),
        };
        Self {
            common: InstCommon::new(Opcode::IndexExtract, elemty),
            operands: [
                UseID::new(allocs, UseKind::IndexExtractAggr),
                UseID::new(allocs, UseKind::IndexExtractIndex),
            ],
            aggr_type: aggr_ty,
        }
    }
}
impl IndexExtractInst {
    pub const OP_AGGR: usize = 0;
    pub const OP_INDEX: usize = 1;
}

_remusys_ir_subinst!(IndexExtractInstID, IndexExtractInst);
impl IndexExtractInstID {
    pub fn new_uninit(allocs: &IRAllocs, tctx: &TypeContext, aggr_ty: AggrType) -> Self {
        let inst = IndexExtractInst::new_uninit(allocs, tctx, aggr_ty);
        Self::allocate(allocs, inst)
    }

    pub fn new(allocs: &IRAllocs, tctx: &TypeContext, aggr: ValueSSA, index: ValueSSA) -> Self {
        let aggr_ty = aggr.get_valtype(allocs);
        let aggr_ty = AggrType::try_from_ir(aggr_ty).unwrap();
        let inst = Self::new_uninit(allocs, tctx, aggr_ty);
        inst.deref_ir(allocs).set_aggr(allocs, aggr);
        inst.deref_ir(allocs).set_index(allocs, index);
        inst
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

    pub fn index_use(self, allocs: &IRAllocs) -> UseID {
        self.deref_ir(allocs).index_use()
    }
    pub fn get_index(self, allocs: &IRAllocs) -> ValueSSA {
        self.deref_ir(allocs).get_index(allocs)
    }
    pub fn set_index(self, allocs: &IRAllocs, val: ValueSSA) {
        self.deref_ir(allocs).set_index(allocs, val);
    }

    pub fn aggr_type(self, allocs: &IRAllocs) -> AggrType {
        self.deref_ir(allocs).aggr_type
    }
    pub fn get_elem_type(self, allocs: &IRAllocs) -> ValTypeID {
        self.deref_ir(allocs).get_elem_type()
    }
}

/// 从数组 / 向量 / 结构体聚合值 a 中提取常量字段 field_idx 处的值.
///
/// ### LLVM IR 语法
///
/// ```llvm
/// %<id> = extractvalue <aggr_type> %a, <field_idx0>, <field_idx1>, ...
/// ```
///
/// ### 操作数布局
///
/// - `operands[0]` - 聚合值
pub struct FieldExtractInst {
    pub common: InstCommon,
    operands: [UseID; 1],
    pub fields: SmallVec<[u32; 4]>,
    pub aggr_type: AggrType,
}

impl IUser for FieldExtractInst {
    fn get_operands(&self) -> OperandSet<'_> {
        OperandSet::Fixed(&self.operands)
    }
    fn operands_mut(&mut self) -> &mut [UseID] {
        &mut self.operands
    }
}
impl ISubInst for FieldExtractInst {
    fn get_common(&self) -> &InstCommon {
        &self.common
    }
    fn common_mut(&mut self) -> &mut InstCommon {
        &mut self.common
    }
    fn try_from_ir_ref(inst: &InstObj) -> Option<&Self> {
        match inst {
            InstObj::FieldExtract(e) => Some(e),
            _ => None,
        }
    }
    fn try_from_ir_mut(inst: &mut InstObj) -> Option<&mut Self> {
        match inst {
            InstObj::FieldExtract(e) => Some(e),
            _ => None,
        }
    }
    fn try_from_ir(inst: InstObj) -> Option<Self> {
        match inst {
            InstObj::FieldExtract(e) => Some(e),
            _ => None,
        }
    }
    fn into_ir(self) -> InstObj {
        InstObj::FieldExtract(self)
    }
    fn try_get_jts(&self) -> Option<JumpTargets<'_>> {
        None
    }
}
impl IAggregateInst for FieldExtractInst {
    fn get_aggr_operand_type(&self) -> AggrType {
        self.aggr_type
    }
    fn get_elem_type(&self) -> ValTypeID {
        self.get_valtype()
    }
    fn aggr_use(&self) -> UseID {
        self.operands[Self::OP_AGGR]
    }
}
impl IAggrFieldInst for FieldExtractInst {
    type DefaultBuilderT = FieldExtractBuilder;

    fn get_field_indices(&self) -> &[u32] {
        &self.fields
    }
}
impl FieldExtractInst {
    pub const OP_AGGR: usize = 0;

    pub fn builder(aggr_type: AggrType) -> FieldExtractBuilder {
        Self::default_builder(aggr_type)
    }
}

_remusys_ir_subinst!(FieldExtractInstID, FieldExtractInst);
impl FieldExtractInstID {
    pub fn builder(aggr_type: AggrType) -> FieldExtractBuilder {
        FieldExtractInst::builder(aggr_type)
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
        self.deref_ir(allocs).aggr_type
    }
    pub fn get_field_indices(self, allocs: &IRAllocs) -> &[u32] {
        &self.deref_ir(allocs).fields
    }
    pub fn get_field_type(self, allocs: &IRAllocs) -> ValTypeID {
        self.deref_ir(allocs).get_valtype()
    }
}

#[derive(Clone)]
pub struct FieldExtractBuilder {
    pub common: AggrFieldInstBuilderCommon,
}
impl IAggrFieldInstBuildable for FieldExtractBuilder {
    type InstT = FieldExtractInst;
    type InstID = FieldExtractInstID;
    fn new(aggr_type: AggrType) -> Self {
        Self {
            common: AggrFieldInstBuilderCommon {
                aggr: ValueSSA::None,
                aggr_type,
                steps: SmallVec::new(),
            },
        }
    }
    fn common(&self) -> &AggrFieldInstBuilderCommon {
        &self.common
    }
    fn common_mut(&mut self) -> &mut AggrFieldInstBuilderCommon {
        &mut self.common
    }
    fn build_obj(&mut self, allocs: &IRAllocs) -> FieldExtractInst {
        let inner = self.common_mut();
        let aggr_type = inner.aggr_type;
        let (indices, ret_ty) = {
            let mut indices = SmallVec::with_capacity(inner.steps.len());
            for (idx, _) in &inner.steps {
                indices.push(*idx);
            }
            let end_ty =
                if let Some((_, ty)) = inner.steps.last() { *ty } else { aggr_type.into_ir() };
            (indices, end_ty)
        };
        let inst = FieldExtractInst {
            common: InstCommon::new(Opcode::FieldExtract, ret_ty),
            operands: [UseID::new(allocs, UseKind::FieldExtractAggr)],
            fields: indices,
            aggr_type,
        };
        if inner.aggr != ValueSSA::None {
            inst.set_aggr(allocs, inner.aggr);
        }
        inst
    }
}

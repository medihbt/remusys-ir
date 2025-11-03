use std::cell::{Ref, RefCell};

use smallvec::SmallVec;

use crate::{
    impl_debug_for_subinst_id, impl_traceable_from_common,
    ir::{
        BlockID, IRAllocs, ISubInst, ISubInstID, IUser, InstCommon, InstID, InstObj, Opcode,
        OperandSet, UseID, UseKind, ValueSSA,
    },
    typing::ValTypeID,
};

trait IPhiOperandSlot: Copy {
    fn pair(&self) -> &UseSlotPair;
    fn from_pair(pair: UseSlotPair) -> Self;

    fn get_block(&self, allocs: &IRAllocs) -> BlockID {
        let block_val = self.pair()[1].get_operand(allocs);
        match block_val {
            ValueSSA::Block(b) => b,
            _ => panic!("Expected BlockID in Phi operand slot, found {block_val:?}"),
        }
    }
    fn get_value(&self, allocs: &IRAllocs) -> ValueSSA {
        self.pair()[0].get_operand(allocs)
    }
    fn set_value(&self, allocs: &IRAllocs, val: ValueSSA) {
        self.pair()[0].set_operand(allocs, val);
    }

    fn new(allocs: &IRAllocs, index: u32, val: ValueSSA, block: BlockID) -> Self {
        let slots = [
            UseID::new(allocs, UseKind::PhiIncomingValue(index)),
            UseID::new(allocs, UseKind::PhiIncomingBlock(index)),
        ];
        slots[0].set_operand(allocs, val);
        slots[1].set_operand(allocs, ValueSSA::Block(block));
        Self::from_pair(slots)
    }

    fn set_index(&self, allocs: &IRAllocs, index: u32) {
        self.pair()[0].set_kind(allocs, UseKind::PhiIncomingValue(index));
        self.pair()[1].set_kind(allocs, UseKind::PhiIncomingBlock(index));
    }

    fn dispose(&self, allocs: &IRAllocs) {
        self.pair()[0].dispose(allocs);
        self.pair()[1].dispose(allocs);
    }
}
type UseSlotPair = [UseID; 2];
impl IPhiOperandSlot for UseSlotPair {
    fn pair(&self) -> &UseSlotPair {
        self
    }
    fn from_pair(pair: UseSlotPair) -> Self {
        pair
    }
}

/// Phi 节点：实现 SSA 形式中的 φ 函数
///
/// ## LLVM IR 语法
/// ```llvm
/// %<result> = phi <type> [ <value0>, %<label0> ], [ <value1>, %<label1> ], ...
/// ```
///
/// ## 操作数布局
/// Phi 节点的操作数以成对形式存储：
/// - `operands[i] = [Value, BlockID]`
///     - `operands[i][0]` - 来自前驱基本块 `label i` 的值 (ValueSSA)
///     - `operands[i][1]` - 对应的前驱基本块 `label i` 的标识符 (BlockID)
///
/// ## 内部数据结构
/// - **operands**: 动态数组存储所有操作数的 Use 引用
///
/// ## 语义
/// 根据控制流的来源选择相应的值：
/// 1. 在运行时，根据前一个执行的基本块
/// 2. 从对应的传入值中选择一个作为 Phi 节点的结果
/// 3. 所有传入值必须与 Phi 节点具有相同的类型
///
/// ## 约束
/// - 每个前驱基本块最多只能有一个对应的传入值
/// - 所有传入值的类型必须与 Phi 节点的返回类型相同
/// - Phi 节点必须出现在基本块的开始位置（在非 Phi 指令之前）. 在 Remusys-IR 中
///   有一个专门的结点变体 `PhiEnd`, 所有 Phi 指令都必须出现在 `PhiEnd` 结点之前。
pub struct PhiInst {
    pub common: InstCommon,
    operands: RefCell<SmallVec<[UseSlotPair; 2]>>,
}
impl_traceable_from_common!(PhiInst, true);
impl IUser for PhiInst {
    fn get_operands(&self) -> OperandSet<'_> {
        OperandSet::Phi(self.incoming_uses())
    }
    fn operands_mut(&mut self) -> &mut [UseID] {
        let operands = self.operands.get_mut();
        operands.as_mut_slice().as_flattened_mut()
    }
}
impl ISubInst for PhiInst {
    fn get_common(&self) -> &InstCommon {
        &self.common
    }
    fn common_mut(&mut self) -> &mut InstCommon {
        &mut self.common
    }
    fn try_from_ir_ref(inst: &InstObj) -> Option<&Self> {
        match inst {
            InstObj::Phi(p) => Some(p),
            _ => None,
        }
    }
    fn try_from_ir_mut(inst: &mut InstObj) -> Option<&mut Self> {
        match inst {
            InstObj::Phi(p) => Some(p),
            _ => None,
        }
    }
    fn try_from_ir(inst: InstObj) -> Option<Self> {
        match inst {
            InstObj::Phi(p) => Some(p),
            _ => None,
        }
    }
    fn into_ir(self) -> InstObj {
        InstObj::Phi(self)
    }
    fn try_get_jts(&self) -> Option<crate::ir::JumpTargets<'_>> {
        None
    }
}
impl PhiInst {
    pub fn new_empty(ty: ValTypeID) -> Self {
        Self {
            common: InstCommon::new(Opcode::Phi, ty),
            operands: RefCell::new(SmallVec::new()),
        }
    }
    pub fn with_capacity(ty: ValTypeID, capacity: usize) -> Self {
        Self {
            common: InstCommon::new(Opcode::Phi, ty),
            operands: RefCell::new(SmallVec::with_capacity(capacity)),
        }
    }

    pub fn incoming_uses(&self) -> Ref<'_, [UseSlotPair]> {
        Ref::map(self.operands.borrow(), |ops| ops.as_slice())
    }
    pub fn find_incoming_pos(&self, allocs: &IRAllocs, block: BlockID) -> Option<usize> {
        self.incoming_uses()
            .iter()
            .position(|slot_pair| slot_pair.get_block(allocs) == block)
    }
    pub fn find_incoming_uses(&self, allocs: &IRAllocs, block: BlockID) -> Option<[UseID; 2]> {
        self.incoming_uses()
            .iter()
            .find(|slot_pair| slot_pair.get_block(allocs) == block)
            .copied()
    }
    pub fn find_incoming_value(&self, allocs: &IRAllocs, block: BlockID) -> Option<ValueSSA> {
        self.find_incoming_uses(allocs, block)
            .map(|slot_pair| slot_pair.get_value(allocs))
    }
    pub fn set_incoming(
        &self,
        allocs: &IRAllocs,
        bb: BlockID,
        val: ValueSSA,
    ) -> Ref<'_, UseSlotPair> {
        let pos = self.find_incoming_pos(allocs, bb);
        match pos {
            Some(idx) => {
                let operands = self.operands.borrow();
                let slot_pair = &operands[idx];
                slot_pair.set_value(allocs, val);
                Ref::map(operands, |ops| &ops[idx])
            }
            None => {
                self.push_incoming(allocs, bb, val);
                let operands = self.operands.borrow();
                let idx = operands.len() - 1;
                Ref::map(operands, |ops| &ops[idx])
            }
        }
    }
    pub fn remove_incoming(&self, allocs: &IRAllocs, bb: BlockID) -> Option<ValueSSA> {
        let pos = self.find_incoming_pos(allocs, bb)?;
        let mut operands = self.operands.borrow_mut();
        let u = operands.swap_remove(pos);
        operands[pos].set_index(allocs, pos as u32);
        let value = u.get_value(allocs);
        u.dispose(allocs);
        Some(value)
    }

    fn push_incoming(&self, allocs: &IRAllocs, bb: BlockID, val: ValueSSA) {
        let mut operands = self.operands.borrow_mut();
        let index = operands.len() as u32;
        let slot_pair = UseSlotPair::new(allocs, index, val, bb);
        operands.push(slot_pair);
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PhiInstID(pub InstID);
impl_debug_for_subinst_id!(PhiInstID);
impl ISubInstID for PhiInstID {
    type InstObjT = PhiInst;

    fn raw_from_ir(id: InstID) -> Self {
        PhiInstID(id)
    }
    fn into_ir(self) -> InstID {
        self.0
    }
}
impl PhiInstID {
    pub fn new_empty(allocs: &IRAllocs, ty: ValTypeID) -> Self {
        let inst = PhiInst::new_empty(ty);
        Self::allocate(allocs, inst)
    }
    pub fn with_capacity(allocs: &IRAllocs, ty: ValTypeID, capacity: usize) -> Self {
        let inst = PhiInst::with_capacity(ty, capacity);
        Self::allocate(allocs, inst)
    }

    pub fn incoming_uses(self, allocs: &IRAllocs) -> Ref<'_, [UseSlotPair]> {
        self.deref_ir(allocs).incoming_uses()
    }
    pub fn find_incoming_pos(self, allocs: &IRAllocs, block: BlockID) -> Option<usize> {
        self.deref_ir(allocs).find_incoming_pos(allocs, block)
    }
    pub fn find_incoming_uses(self, allocs: &IRAllocs, block: BlockID) -> Option<[UseID; 2]> {
        self.deref_ir(allocs).find_incoming_uses(allocs, block)
    }
    pub fn find_incoming_value(self, allocs: &IRAllocs, block: BlockID) -> Option<ValueSSA> {
        self.deref_ir(allocs).find_incoming_value(allocs, block)
    }
    pub fn set_incoming(
        self,
        allocs: &IRAllocs,
        bb: BlockID,
        val: ValueSSA,
    ) -> Ref<'_, UseSlotPair> {
        self.deref_ir(allocs).set_incoming(allocs, bb, val)
    }
    pub fn remove_incoming(self, allocs: &IRAllocs, bb: BlockID) -> Option<ValueSSA> {
        self.deref_ir(allocs).remove_incoming(allocs, bb)
    }
}

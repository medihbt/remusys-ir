use crate::{
    _remusys_ir_subinst,
    ir::{
        BlockID, BlockSection, IRAllocs, ISubInst, ISubInstID, ISubValueSSA, IUser, InstCommon,
        InstID, InstObj, Opcode, OperandSet, PoolAllocatedDisposeRes, UseID, UseKind, UserID,
        ValueSSA,
    },
    typing::ValTypeID,
};
use smallvec::SmallVec;
use std::{
    cell::{Cell, Ref, RefCell},
    collections::{BTreeMap, HashMap},
    ops::DerefMut,
};
use thiserror::Error;

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

    fn new(
        allocs: &IRAllocs,
        id: Option<InstID>,
        index: u32,
        val: ValueSSA,
        block: BlockID,
    ) -> Self {
        let (uval, ublk) = (
            UseID::new(allocs, UseKind::PhiIncomingValue(index)),
            UseID::new(allocs, UseKind::PhiIncomingBlock(index)),
        );
        uval.set_operand(allocs, val);
        ublk.set_operand(allocs, ValueSSA::Block(block));
        uval.set_user(allocs, id.map(UserID::Inst));
        ublk.set_user(allocs, id.map(UserID::Inst));
        Self::from_pair([uval, ublk])
    }

    fn set_index(&self, allocs: &IRAllocs, index: u32) {
        let &[val, blk] = self.pair();
        val.set_kind(allocs, UseKind::PhiIncomingValue(index));
        blk.set_kind(allocs, UseKind::PhiIncomingBlock(index));
    }

    fn dispose(&self, allocs: &IRAllocs) -> PoolAllocatedDisposeRes {
        let &[val, blk] = self.pair();
        val.dispose(allocs)?;
        blk.dispose(allocs)
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
    pub(in crate::ir) self_id: Cell<Option<InstID>>,
}

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
    fn get_block_section(&self) -> BlockSection {
        BlockSection::Phi
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
            self_id: Cell::new(None),
        }
    }
    pub fn with_capacity(ty: ValTypeID, capacity: usize) -> Self {
        Self {
            common: InstCommon::new(Opcode::Phi, ty),
            operands: RefCell::new(SmallVec::with_capacity(capacity)),
            self_id: Cell::new(None),
        }
    }
    pub fn from_incomings(
        ty: ValTypeID,
        allocs: &IRAllocs,
        incomings: impl IntoIterator<Item = (BlockID, ValueSSA)>,
    ) -> Self {
        let incomings = BTreeMap::from_iter(incomings);
        let phi = Self::with_capacity(ty, incomings.len());
        for (block, val) in incomings {
            assert_eq!(
                ty,
                val.get_valtype(allocs),
                "Type mismatch in PhiInst incoming value"
            );
            phi.push_incoming(allocs, block, val);
        }
        phi
    }
    pub fn builder(allocs: &IRAllocs, ty: ValTypeID) -> PhiInstBuilder<'_> {
        PhiInstBuilder::new(allocs, ty)
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
        // 如果 swap_remove 将最后一个元素移到了 pos，则需要更新其索引；
        // 但当移除的是最后一个元素时，pos == new_len，直接访问会越界。
        if pos < operands.len() {
            operands[pos].set_index(allocs, pos as u32);
        }
        let value = u.get_value(allocs);
        u.dispose(allocs)
            .expect("Broken IR invariant in PhiInst::remove_incoming");
        Some(value)
    }

    fn push_incoming(&self, allocs: &IRAllocs, bb: BlockID, val: ValueSSA) {
        let mut operands = self.operands.borrow_mut();
        let index = operands.len() as u32;
        let slot_pair = UseSlotPair::new(allocs, self.self_id.get(), index, val, bb);
        operands.push(slot_pair);
    }

    pub fn begin_dedup<'ir>(&'ir self, allocs: &'ir IRAllocs) -> PhiInstDedup<'ir> {
        PhiInstDedup::new(self, allocs)
    }
}

_remusys_ir_subinst!(PhiInstID, PhiInst, section = Phi);
impl PhiInstID {
    pub fn new_empty(allocs: &IRAllocs, ty: ValTypeID) -> Self {
        let inst = PhiInst::new_empty(ty);
        Self::allocate(allocs, inst)
    }
    pub fn with_capacity(allocs: &IRAllocs, ty: ValTypeID, capacity: usize) -> Self {
        let inst = PhiInst::with_capacity(ty, capacity);
        Self::allocate(allocs, inst)
    }
    pub fn from_incomings(
        allocs: &IRAllocs,
        ty: ValTypeID,
        incomings: impl IntoIterator<Item = (BlockID, ValueSSA)>,
    ) -> Self {
        let inst = PhiInst::from_incomings(ty, allocs, incomings);
        Self::allocate(allocs, inst)
    }
    pub fn builder(allocs: &IRAllocs, ty: ValTypeID) -> PhiInstBuilder<'_> {
        PhiInstBuilder::new(allocs, ty)
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

    pub fn begin_dedup(self, allocs: &IRAllocs) -> PhiInstDedup<'_> {
        PhiInstDedup::new(self.deref_ir(allocs), allocs)
    }
}

#[derive(Debug, Clone, Copy, Error)]
pub enum PhiInstErr {
    #[error("PhiInst has duplicated uses with different operands")]
    DuplicatedUses,

    #[error("PhiInst incoming block {1:?} (related use: {0:?}) not found in parent preds")]
    IncomingNotInPreds(UseID, BlockID),

    #[error("PhiInst has duplicated incoming block {0:?}")]
    DuplicateIncoming(BlockID),

    #[error("PhiInst is missing incoming block {0:?}")]
    MissingIncoming(BlockID),
}
pub type PhiInstRes<T = ()> = Result<T, PhiInstErr>;

pub struct PhiInstDedup<'ir> {
    multimap: HashMap<BlockID, DedupUnits>,
    phi_inst: &'ir PhiInst,
    _lock: Ref<'ir, SmallVec<[UseSlotPair; 2]>>,
    allocs: &'ir IRAllocs,
    nodup: bool,
    initial_nodup: bool,
}

struct DedupUnit {
    index: u32,
    value: Option<ValueSSA>,
}
type DedupUnits = SmallVec<[DedupUnit; 4]>;

impl<'ir> PhiInstDedup<'ir> {
    pub fn new(phi_inst: &'ir PhiInst, allocs: &'ir IRAllocs) -> Self {
        let mut multimap = HashMap::new();
        let mut nodup = true;
        for (index, [uval, ublk]) in phi_inst.incoming_uses().iter().enumerate() {
            let block = ublk.get_operand(allocs);
            let value = uval.get_operand(allocs);
            let block_id = match block {
                ValueSSA::Block(b) => b,
                _ => panic!("Expected BlockID in Phi operand slot, found {block:?}"),
            };
            let entry = multimap.entry(block_id).or_insert_with(SmallVec::new);
            if !entry.is_empty() {
                nodup = false;
            }
            entry.push(DedupUnit { index: index as u32, value: Some(value) });
        }
        Self {
            multimap,
            phi_inst,
            _lock: phi_inst.operands.borrow(),
            allocs,
            nodup,
            initial_nodup: nodup,
        }
    }

    pub fn nodup(&self) -> bool {
        self.nodup
    }
    pub fn initial_nodup(&self) -> bool {
        self.initial_nodup
    }
    pub fn dedupped(&self) -> bool {
        self.nodup && !self.initial_nodup
    }

    /// 精简掉 “同一个 block 有多个 use 对应, 但操作数一模一样” 的情况
    /// 返回: 全部精简后回顾精简时是否存在一个 block 存在不同 value 的情况. 如果存在则返回 false
    pub fn dedup_same_operand(&mut self) -> bool {
        if self.nodup {
            return true;
        }
        let mut consistent = true;
        for (_, slot_vals) in self.multimap.iter_mut() {
            let first_val_pos = slot_vals.iter().position(|du| du.value.is_some());
            let Some(first_val_pos) = first_val_pos else {
                continue; // 全部都是 None，无需处理
            };
            let first_val = slot_vals[first_val_pos].value.unwrap();
            for &DedupUnit { value, .. } in slot_vals.iter().skip(first_val_pos + 1) {
                if let Some(val) = value
                    && val != first_val
                {
                    consistent = false;
                    break;
                }
            }
            if !consistent {
                break;
            }
            for slot_val in slot_vals.iter_mut().skip(first_val_pos + 1) {
                // 全部相同，保留第一个，其他标记为 None
                slot_val.value = None;
            }
        }
        self.nodup = consistent;
        consistent
    }

    pub fn keep_first(&mut self) {
        for (_, slot_vals) in self.multimap.iter_mut() {
            if slot_vals.len() <= 1 {
                continue;
            }
            // 保留第一个，其他标记为 None
            for slot_val in slot_vals.iter_mut().skip(1) {
                slot_val.value = None;
            }
        }
        self.nodup = true;
    }

    /// 应用精简结果到 PhiInst 上. 要求每个 slot 只有一个有效的 ValueSSA.
    pub fn apply(self) -> PhiInstRes {
        if !self.nodup {
            return Err(PhiInstErr::DuplicatedUses);
        }
        // 如果初始就是无重复的，则无需操作
        if self.initial_nodup {
            return Ok(());
        }
        // 释放掉 Phi 的操作数锁
        let Self { multimap, allocs, .. } = self;
        // 接下来重新组织操作数列表
        let mut operands = self.phi_inst.operands.borrow_mut();

        let old_operands = std::mem::take(operands.deref_mut());
        let mut new_opreands = SmallVec::with_capacity(multimap.len());

        for (_, dups) in multimap {
            for DedupUnit { index, value } in dups {
                let [uval, ublk] = old_operands[index as usize];
                if let Some(val) = value {
                    // 保留该 use
                    uval.set_operand(allocs, val);
                    uval.set_kind(allocs, UseKind::PhiIncomingValue(new_opreands.len() as u32));
                    ublk.set_kind(allocs, UseKind::PhiIncomingBlock(new_opreands.len() as u32));
                    new_opreands.push([uval, ublk]);
                } else {
                    // 释放掉该 use
                    uval.dispose(allocs).expect("Broken IR structure");
                    ublk.dispose(allocs).expect("Broken IR structure");
                }
            }
        }
        *operands = new_opreands;
        Ok(())
    }
}

pub struct PhiInstBuilder<'ir> {
    pub value_type: ValTypeID,
    pub incomings: BTreeMap<BlockID, ValueSSA>,
    pub allow_uninit: bool,
    allocs: &'ir IRAllocs,
}

impl<'ir> PhiInstBuilder<'ir> {
    pub fn new(allocs: &'ir IRAllocs, value_type: ValTypeID) -> Self {
        Self {
            value_type,
            incomings: BTreeMap::new(),
            allow_uninit: false,
            allocs,
        }
    }

    pub fn allow_uninit(&mut self, allow: bool) -> &mut Self {
        self.allow_uninit = allow;
        self
    }
    pub fn add_incoming(&mut self, block: BlockID, val: ValueSSA) -> &mut Self {
        if !self.allow_uninit {
            assert_eq!(
                self.value_type,
                val.get_valtype(self.allocs),
                "Type mismatch in PhiInstBuilder incoming value"
            );
        }
        self.incomings.insert(block, val);
        self
    }
    pub fn add_uninit_incoming(&mut self, block: BlockID) -> &mut Self {
        assert!(
            self.allow_uninit,
            "Cannot add uninitialized incoming value when allow_uninit is false"
        );
        self.incomings.insert(block, ValueSSA::None);
        self
    }
    pub fn incomings(
        &mut self,
        incomings: impl IntoIterator<Item = (BlockID, ValueSSA)>,
    ) -> &mut Self {
        for (block, val) in incomings {
            self.add_incoming(block, val);
        }
        self
    }
    pub fn uninit_incomings(&mut self, blocks: impl IntoIterator<Item = BlockID>) -> &mut Self {
        for block in blocks {
            self.add_uninit_incoming(block);
        }
        self
    }

    pub fn build_obj(&self) -> PhiInst {
        let mut operands = SmallVec::with_capacity(self.incomings.len());
        for (index, (&blk, &val)) in self.incomings.iter().enumerate() {
            let slots = UseSlotPair::new(self.allocs, None, index as u32, val, blk);
            operands.push(slots);
        }
        PhiInst {
            common: InstCommon::new(Opcode::Phi, self.value_type),
            operands: RefCell::new(operands),
            self_id: Cell::new(None),
        }
    }
    pub fn build_id(&self) -> PhiInstID {
        let inst = self.build_obj();
        PhiInstID::allocate(self.allocs, inst)
    }
}

use std::{
    cell::{Cell, Ref, RefCell},
    rc::{Rc, Weak},
};

use slab::Slab;

use crate::{
    base::{INullableValue, IWeakListNode, MixRef, SlabRef, WeakList},
    ir::{
        BlockData, BlockRef, FuncRef, IRAllocs, ISubInst, ISubValueSSA, InstData, InstRef, Use,
        UseKind, UserID,
        inst::{Br, BrRef, ISubInstRef, Jump, JumpRef, PhiRef, RetRef, Switch, SwitchRef},
    },
};

/// 跳转目标的类型，用于区分不同的控制流转移
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum JumpTargetKind {
    /// 无效/哨兵类型，用于链表哨兵节点
    None,
    /// 无条件跳转
    Jump,
    /// 条件分支的真分支
    BrTrue,
    /// 条件分支的假分支
    BrFalse,
    /// Switch 语句的默认分支
    SwitchDefault,
    /// Switch 语句的具体 case 分支，值为 case 常量
    SwitchCase(i128),
}

/// 跳转目标对象，连接终结指令和目标基本块
///
/// 每个跳转目标表示控制流图中的一条边，包含：
/// - 跳转类型（无条件跳转、条件分支等）
/// - 源终结指令的引用
/// - 目标基本块的引用
///
/// 跳转目标通过弱引用链表连接，避免循环引用问题。
#[derive(Debug)]
pub struct JumpTarget {
    /// 弱引用链表的节点头 (prev, next)
    node_head: RefCell<(Weak<Self>, Weak<Self>)>,
    /// 跳转目标的类型
    pub kind: JumpTargetKind,
    /// 产生此跳转的终结指令
    pub terminator: Cell<InstRef>,
    /// 跳转的目标基本块
    pub block: Cell<BlockRef>,
}

impl IWeakListNode for JumpTarget {
    fn load_head(&self) -> (Weak<Self>, Weak<Self>) {
        self.node_head.borrow().clone()
    }
    fn store_head(&self, head: (Weak<Self>, Weak<Self>)) {
        *self.node_head.borrow_mut() = head;
    }
    fn new_sentinel() -> Rc<Self> {
        Rc::new(Self {
            node_head: RefCell::new((Weak::new(), Weak::new())),
            kind: JumpTargetKind::None,
            terminator: Cell::new(InstRef::new_null()),
            block: Cell::new(BlockRef::new_null()),
        })
    }
    fn is_sentinel(&self) -> bool {
        self.kind == JumpTargetKind::None
    }

    /// 当目标基本块析构时通知到该 JumpTarget 边, 主动清理引用关系.
    fn on_list_finalize(&self) {
        self.block.set(BlockRef::new_null());
    }
}

impl Drop for JumpTarget {
    /// 析构时自动从前驱基本块的前驱列表中移除自己
    fn drop(&mut self) {
        self.detach();
    }
}

impl JumpTarget {
    /// 创建一个新的跳转目标
    ///
    /// # 参数
    /// - `kind`: 跳转目标的类型
    ///
    /// # 返回
    /// 返回新创建的跳转目标的强引用
    pub fn new(kind: JumpTargetKind) -> Rc<Self> {
        Rc::new(Self {
            node_head: RefCell::new((Weak::new(), Weak::new())),
            kind,
            terminator: Cell::new(InstRef::new_null()),
            block: Cell::new(BlockRef::new_null()),
        })
    }

    /// 获取产生此跳转的终结指令引用
    pub fn get_terminator_inst(&self) -> InstRef {
        self.terminator.get()
    }

    /// 获取产生此跳转的终结指令的强类型引用
    pub fn get_terminator(&self) -> TerminatorRef {
        let inst = self.terminator.get();
        use JumpTargetKind::*;
        match self.kind {
            Jump => TerminatorRef::Jump(JumpRef::from_raw_nocheck(inst)),
            BrTrue | BrFalse => TerminatorRef::Br(BrRef::from_raw_nocheck(inst)),
            SwitchDefault | SwitchCase(_) => {
                TerminatorRef::Switch(SwitchRef::from_raw_nocheck(inst))
            }
            None => unreachable!("Sentinel JumpTarget has no terminator"),
        }
    }

    /// 设置产生此跳转的终结指令
    ///
    /// ### 参数
    ///
    /// - `terminator`: 终结指令的引用
    pub fn set_terminator(&self, terminator: InstRef) {
        self.terminator.set(terminator);
    }

    /// 获取跳转的目标基本块引用
    pub fn get_block(&self) -> BlockRef {
        self.block.get()
    }

    /// 设置跳转的目标基本块
    ///
    /// 此操作会自动维护控制流图的前驱-后继关系：
    /// - 如果之前已经设置了目标，会从旧目标的前驱列表中移除
    /// - 设置新目标时，会将自己添加到新目标的前驱列表中
    ///
    /// # 参数
    /// - `alloc`: 基本块分配器，用于访问目标基本块数据
    /// - `block`: 新的目标基本块引用
    pub fn set_block(self: &Rc<Self>, alloc: &Slab<BlockData>, block: BlockRef) {
        if self.block.get() == block {
            return; // No change
        }
        self.detach();
        self.block.set(block);
        if block.is_nonnull() {
            block.to_data(alloc).preds.push_back(Rc::downgrade(self));
        }
    }

    /// 从前驱基本块的前驱列表中移除自己
    pub fn clean_block(&self) {
        if self.block.get().is_null() {
            return; // No block to clean
        }
        self.detach();
        self.block.set(BlockRef::new_null());
    }

    /// 自己是不是关键边
    pub fn is_critical_edge(&self, allocs: &IRAllocs) -> bool {
        let from_inst = self.get_terminator();
        let to_block = self.get_block();
        let has_multiple_succs = from_inst.has_multiple_blocks(&allocs.insts);
        let has_multiple_preds = to_block.to_data(&allocs.blocks).has_multiple_preds();
        has_multiple_succs && has_multiple_preds
    }
}

/// 基本块前驱列表的类型别名
///
/// 使用弱引用链表存储指向此基本块的所有跳转目标，
/// 避免控制流图中的循环引用问题。
pub type PredList = WeakList<JumpTarget>;
pub type JumpTargets<'a> = MixRef<'a, [Rc<JumpTarget>]>;

impl<'a> From<Ref<'a, Vec<Rc<JumpTarget>>>> for JumpTargets<'a> {
    fn from(value: Ref<'a, Vec<Rc<JumpTarget>>>) -> Self {
        let value = Ref::map(value, |value| value.as_slice());
        Self::Dyn(value)
    }
}

/// 终结指令的 trait，提供跳转目标管理功能
///
/// 终结指令是基本块的最后一条指令，负责控制流的转移。
/// 此 trait 提供了访问和操作跳转目标的统一接口。
pub trait ITerminatorInst: ISubInst {
    /// 读取跳转目标列表并执行回调函数
    ///
    /// # 参数
    /// - `reader`: 接受跳转目标数组的回调函数
    ///
    /// # 返回
    /// 回调函数的返回值
    fn read_jts<T>(&self, reader: impl FnOnce(&[Rc<JumpTarget>]) -> T) -> T;

    fn jts_mut(&mut self) -> &mut [Rc<JumpTarget>];

    fn get_jts(&self) -> JumpTargets;

    /// 获取跳转目标的数量
    fn n_jump_targets(&self) -> usize {
        self.read_jts(|jts| jts.len())
    }

    /// 导出所有跳转目标的基本块列表
    ///
    /// # 返回
    /// 包含所有目标基本块的 Vec，可能包含重复项
    fn dump_blocks(&self) -> Vec<BlockRef> {
        self.read_jts(|jts| jts.iter().map(|jt| jt.block.get()).collect())
    }

    /// 导出去重后的跳转目标基本块列表
    ///
    /// 根据跳转目标数量和是否需要排序，自动选择最优的去重算法：
    /// - 小于 32 个：使用 Vec + sort + dedup
    /// - 32-2048 个且需要排序：使用 BTreeSet
    /// - 大于 2048 个且不需要排序：使用 HashSet
    ///
    /// # 参数
    /// - `sorted`: 是否需要对结果进行排序
    ///
    /// # 返回
    /// 去重后的目标基本块列表
    fn dedep_dump_blocks(&self, sorted: bool) -> Vec<BlockRef> {
        const THRESHOLD0: usize = 32;
        const THRESHOLD1: usize = 2048;
        self.read_jts(|jts| {
            if jts.len() < THRESHOLD0 {
                collect_jump_blocks_dedup::through_vec(jts)
            } else if jts.len() < THRESHOLD1 || sorted {
                collect_jump_blocks_dedup::through_treeset(jts)
            } else {
                collect_jump_blocks_dedup::through_hashset(jts)
            }
        })
    }

    /// 检查当前基本块是否有多个后继基本块. 重边要合并以后再计算.
    fn has_multiple_blocks(&self) -> bool {
        let mut first = None;
        for jt in &self.get_jts() {
            let block = jt.block.get();
            match first {
                None => first = Some(block),
                Some(b) if b != block => return true,
                _ => continue,
            }
        }
        false
    }
}

pub trait ITerminatorRef: ISubInstRef<InstDataT: ITerminatorInst> {
    fn read_jts<T>(self, alloc: &Slab<InstData>, reader: impl FnOnce(&[Rc<JumpTarget>]) -> T) -> T {
        let inst = self.to_inst(alloc);
        inst.read_jts(reader)
    }

    fn jts_mut<'a>(self, alloc: &'a mut Slab<InstData>) -> &'a mut [Rc<JumpTarget>]
    where
        <Self as ISubInstRef>::InstDataT: 'a,
    {
        self.to_inst_mut(alloc).jts_mut()
    }

    fn has_multiple_blocks(self, allocs: &Slab<InstData>) -> bool {
        self.to_inst(allocs).has_multiple_blocks()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TerminatorRef {
    Unreachable(InstRef),
    Ret(RetRef),
    Jump(JumpRef),
    Br(BrRef),
    Switch(SwitchRef),
}

impl TerminatorRef {
    pub fn get_inst(&self) -> InstRef {
        match self {
            TerminatorRef::Unreachable(inst) => *inst,
            TerminatorRef::Ret(ret) => ret.into_raw(),
            TerminatorRef::Jump(jump) => jump.into_raw(),
            TerminatorRef::Br(br) => br.into_raw(),
            TerminatorRef::Switch(switch) => switch.into_raw(),
        }
    }

    pub fn try_from_instref(inst: InstRef, alloc: &Slab<InstData>) -> Option<Self> {
        match inst.to_inst(alloc) {
            InstData::Ret(_) => Some(TerminatorRef::Ret(RetRef::from_raw_nocheck(inst))),
            InstData::Jump(_) => Some(TerminatorRef::Jump(JumpRef::from_raw_nocheck(inst))),
            InstData::Br(_) => Some(TerminatorRef::Br(BrRef::from_raw_nocheck(inst))),
            InstData::Switch(_) => Some(TerminatorRef::Switch(SwitchRef::from_raw_nocheck(inst))),
            InstData::Unreachable(_) => Some(TerminatorRef::Unreachable(inst)),
            _ => None,
        }
    }

    pub fn from_instref(inst: InstRef, alloc: &Slab<InstData>) -> Self {
        Self::try_from_instref(inst, alloc).unwrap_or_else(|| {
            panic!("Expected a terminator instruction, got: {:?}", inst);
        })
    }

    pub fn get_jts(self, alloc: &Slab<InstData>) -> JumpTargets {
        match self {
            TerminatorRef::Unreachable(_) => JumpTargets::Fix(&[]),
            TerminatorRef::Ret(ret) => ret.to_inst(alloc).get_jts(),
            TerminatorRef::Jump(jump) => jump.to_inst(alloc).get_jts(),
            TerminatorRef::Br(br) => br.to_inst(alloc).get_jts(),
            TerminatorRef::Switch(switch) => switch.to_inst(alloc).get_jts(),
        }
    }
    pub fn jts_mut(self, alloc: &mut Slab<InstData>) -> &mut [Rc<JumpTarget>] {
        match self {
            TerminatorRef::Unreachable(_) => panic!("Unreachable does not have jump targets"),
            TerminatorRef::Ret(ret) => ret.to_inst_mut(alloc).jts_mut(),
            TerminatorRef::Jump(jump) => jump.to_inst_mut(alloc).jts_mut(),
            TerminatorRef::Br(br) => br.to_inst_mut(alloc).jts_mut(),
            TerminatorRef::Switch(switch) => switch.to_inst_mut(alloc).jts_mut(),
        }
    }

    pub fn read_jts<T>(
        self,
        alloc: &Slab<InstData>,
        reader: impl FnOnce(&[Rc<JumpTarget>]) -> T,
    ) -> T {
        match self {
            TerminatorRef::Unreachable(_) => reader(&[]),
            TerminatorRef::Ret(ret) => ret.to_inst(alloc).read_jts(reader),
            TerminatorRef::Jump(jump) => jump.to_inst(alloc).read_jts(reader),
            TerminatorRef::Br(br) => br.to_inst(alloc).read_jts(reader),
            TerminatorRef::Switch(switch) => switch.to_inst(alloc).read_jts(reader),
        }
    }

    pub fn dump_blocks(self, alloc: &Slab<InstData>) -> Vec<BlockRef> {
        self.read_jts(alloc, |jts| jts.iter().map(|jt| jt.block.get()).collect())
    }

    pub fn dedep_dump_blocks(self, alloc: &Slab<InstData>, sorted: bool) -> Vec<BlockRef> {
        match self {
            TerminatorRef::Unreachable(_) | TerminatorRef::Ret(_) => vec![],
            TerminatorRef::Jump(jump) => jump.to_inst(alloc).dedep_dump_blocks(sorted),
            TerminatorRef::Br(br) => br.to_inst(alloc).dedep_dump_blocks(sorted),
            TerminatorRef::Switch(switch) => switch.to_inst(alloc).dedep_dump_blocks(sorted),
        }
    }

    pub fn has_multiple_blocks(self, allocs: &Slab<InstData>) -> bool {
        match self {
            TerminatorRef::Unreachable(_) | TerminatorRef::Ret(_) => false,
            TerminatorRef::Jump(jump) => jump.to_inst(allocs).has_multiple_blocks(),
            TerminatorRef::Br(br) => br.to_inst(allocs).has_multiple_blocks(),
            TerminatorRef::Switch(switch) => switch.to_inst(allocs).has_multiple_blocks(),
        }
    }
}

pub enum TerminatorDataRef<'a> {
    Jump(&'a Jump),
    Br(&'a Br),
    Switch(&'a Switch),
}

impl<'a> TryFrom<&'a InstData> for TerminatorDataRef<'a> {
    type Error = &'static str;

    fn try_from(inst: &'a InstData) -> Result<Self, Self::Error> {
        match inst {
            InstData::Jump(jump) => Ok(TerminatorDataRef::Jump(jump)),
            InstData::Br(br) => Ok(TerminatorDataRef::Br(br)),
            InstData::Switch(switch) => Ok(TerminatorDataRef::Switch(switch)),
            _ => Err("Not a terminator instruction"),
        }
    }
}

impl<'a> TerminatorDataRef<'a> {
    pub fn get_jts(&self) -> JumpTargets {
        match self {
            TerminatorDataRef::Jump(jump) => jump.get_jts(),
            TerminatorDataRef::Br(br) => br.get_jts(),
            TerminatorDataRef::Switch(switch) => switch.get_jts(),
        }
    }

    pub fn read_jts<T>(&self, reader: impl FnOnce(&[Rc<JumpTarget>]) -> T) -> T {
        match self {
            TerminatorDataRef::Jump(jump) => jump.read_jts(reader),
            TerminatorDataRef::Br(br) => br.read_jts(reader),
            TerminatorDataRef::Switch(switch) => switch.read_jts(reader),
        }
    }
}

mod collect_jump_blocks_dedup {
    use crate::{
        base::INullableValue,
        ir::{BlockRef, JumpTarget},
    };
    use std::{
        collections::{BTreeSet, HashSet},
        rc::Rc,
    };

    pub(super) fn through_vec(targets: &[Rc<JumpTarget>]) -> Vec<BlockRef> {
        let mut blocks = Vec::with_capacity(targets.len());
        for jt in targets {
            let block = jt.block.get();
            if block.is_nonnull() {
                blocks.push(block);
            }
        }
        blocks.sort();
        blocks.dedup();
        blocks
    }

    pub(super) fn through_treeset(targets: &[Rc<JumpTarget>]) -> Vec<BlockRef> {
        let mut blocks = BTreeSet::new();
        for jt in targets {
            let block = jt.block.get();
            if block.is_nonnull() {
                blocks.insert(block);
            }
        }
        blocks.into_iter().collect()
    }

    pub(super) fn through_hashset(targets: &[Rc<JumpTarget>]) -> Vec<BlockRef> {
        let mut blocks = HashSet::new();
        for jt in targets {
            let block = jt.block.get();
            if block.is_nonnull() {
                blocks.insert(block);
            }
        }
        blocks.into_iter().collect()
    }
}

pub struct JumpTargetSplitter<'a> {
    allocs: &'a mut IRAllocs,
    old_jt: &'a Rc<JumpTarget>,
}

impl<'a> JumpTargetSplitter<'a> {
    pub fn new(allocs: &'a mut IRAllocs, old_jt: &'a Rc<JumpTarget>) -> Self {
        Self { allocs, old_jt }
    }

    /// 执行拆分操作, 返回新插入的基本块引用.
    ///
    /// 拆分完毕后这个结构体就没什么用了, 这里直接把它消费掉.
    pub fn split(mut self) -> BlockRef {
        let to_block = self.old_jt.get_block();
        assert!(
            to_block.is_nonnull(),
            "Cannot split a JumpTarget with null block"
        );
        let new_block = self.create_block_to_next(to_block);

        // 更新旧 JumpTarget 的目标为新基本块, 这样就完成了从旧基本块到新基本块的跳转.
        self.old_jt.set_block(&self.allocs.blocks, new_block);

        // 更新关联的 Phi 节点，让它们指向新基本块
        let pred_terminator = self.old_jt.get_terminator();
        let pred_bb = pred_terminator.get_inst().get_parent(&*self.allocs);
        let pred_users = pred_bb.users(self.allocs);

        // 检查是不是只有这一个 JumpTarget 指向这个基本块. 如果不是的话就需要拷贝而不是移动.
        let only_one_jt = {
            let mut count = 0;
            for jt in &pred_terminator.get_jts(&self.allocs.insts) {
                if jt.get_block() == to_block {
                    count += 1;
                }
                if count > 1 {
                    break;
                }
            }
            debug_assert!(
                count >= 1,
                "Terminator must have at least one JumpTarget to the block"
            );
            count == 1
        };

        self.redirect_phi_uses(new_block, pred_users, only_one_jt);

        new_block
    }

    /// 创建一个新的基本块, 插入函数体设置跳转到下一个基本块
    fn create_block_to_next(&mut self, to_block: BlockRef) -> BlockRef {
        let new_block = BlockData::empty_from_alloc(&mut self.allocs.insts);
        let jump_inst = {
            let jump = Jump::new(&self.allocs.blocks, to_block);
            InstRef::from_alloc(&mut self.allocs.insts, jump.into_ir())
        };
        // 设置新基本块的终结指令为 Jump
        new_block
            .set_terminator_with_allocs(&self.allocs, jump_inst)
            .expect("Failed to set terminator for new block");
        let new_block = BlockRef::from_allocs(self.allocs, new_block);

        // 取函数, 稍后要把新基本块插入到函数中.
        let parent_func = {
            let parent_func = to_block.to_data(&self.allocs.blocks).get_parent_func();
            assert!(
                parent_func.is_nonnull(),
                "Cannot split JumpTarget without a parent function"
            );
            FuncRef(parent_func)
        };

        // 具体的插入位置是: 该 JumpTarget 的前驱基本块的后面.
        let pred_block = self.old_jt.get_terminator_inst().get_parent(&*self.allocs);
        assert!(
            pred_block.is_nonnull(),
            "Cannot split JumpTarget without a predecessor block"
        );

        // 将新基本块插入到前驱基本块的后面
        let Some(body) = parent_func.try_get_body(&self.allocs.globals) else {
            unreachable!("Function {parent_func:?} without body cannot have critical edges");
        };
        body.node_add_next(&self.allocs.blocks, pred_block, new_block)
            .expect("Failed to insert new block after predecessor block");
        new_block
    }

    fn redirect_phi_uses(
        &self,
        new_block: BlockRef,
        pred_users: &WeakList<Use>,
        only_one_jt: bool,
    ) {
        if only_one_jt {
            // 只有这一个 JumpTarget 指向这个基本块, 需要直接移动相关的 Use.
            // 方式 1：先移动 Use 再调用 redirect_income（有些绕圈但正确）
            // 方式 2: 直接调用 redirect_income_operand_only (更直接，但有些危险)
            pred_users.move_to_if(
                new_block.users(self.allocs),
                |u| matches!(u.kind.get(), UseKind::PhiIncomingBlock(_)),
                |u| u.operand.set(new_block.into_ir()),
            );
        } else {
            // 有多个 JumpTarget 指向这个基本块, 需要拷贝相关的 Use.
            // 保持原有的 use-def 关系不变，为新基本块创建新的传入值
            for u in pred_users.iter() {
                let UseKind::PhiIncomingBlock(group_idx) = u.kind.get() else {
                    continue;
                };
                let phi = {
                    let UserID::Inst(inst) = u.user.get() else { unreachable!() };
                    PhiRef(inst).to_inst(&self.allocs.insts)
                };
                let income_val = phi.income_value_at(group_idx as usize);
                // 为新基本块创建新的传入值，不影响原有的 use-def 关系
                phi.set_income(self.allocs, new_block, income_val)
                    .expect("Failed to add new incoming to Phi");
            }
        }
    }
}

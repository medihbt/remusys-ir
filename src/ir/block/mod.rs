use crate::{
    base::{
        INullableValue, SlabListError, SlabListNode, SlabListNodeHead, SlabListNodeRef,
        SlabListRange, SlabRef, SlabRefList,
    },
    ir::{
        ConstData, IRAllocs, IRWriter, ISubInst, ISubValueSSA, ITraceableValue, InstCommon,
        InstData, InstRef, JumpTarget, Module, PredList, TerminatorRef, UserList, ValueSSA,
        block::jump_target::JumpTargets,
        global::GlobalRef,
        inst::{BrRef, ISubInstRef, JumpRef, PhiRef, Ret, RetRef, SwitchRef},
    },
    typing::id::ValTypeID,
};
use slab::Slab;
use std::{
    cell::{Cell, Ref},
    ops::ControlFlow,
    rc::Rc,
};

pub(super) mod jump_target;

/// 基本块数据结构
///
/// 基本块是控制流图的基本单元，包含一系列按顺序执行的指令。
/// 每个基本块有唯一的入口点（第一条指令）和出口点（终结指令）。
#[derive(Debug)]
pub struct BlockData {
    /// 基本块的内部数据（节点头、父函数、ID等）
    pub inner: Cell<BlockDataInner>,
    /// 基本块自身的引用
    pub self_ref: BlockRef,
    /// 基本块内的指令列表
    pub insts: SlabRefList<InstRef>,
    /// Phi 指令区域的结束标记
    pub phi_end: InstRef,
    /// 使用此基本块作为操作数的指令列表 (Use-Def 链)
    pub users: UserList,
    /// 此基本块的前驱基本块列表 (控制流图的入边)
    pub preds: PredList,
}

impl SlabListNode for BlockData {
    fn new_guide() -> Self {
        BlockData {
            inner: Cell::new(BlockDataInner {
                _node_head: SlabListNodeHead::new(),
                _id: 0,
                _parent_func: GlobalRef::new_null(),
            }),
            self_ref: BlockRef::new_null(),
            insts: SlabRefList::new_guide(),
            phi_end: InstRef::new_null(),
            users: UserList::new_empty(),
            preds: PredList::new_empty(),
        }
    }
    fn load_node_head(&self) -> SlabListNodeHead {
        self.inner.get()._node_head
    }
    fn store_node_head(&self, node_head: SlabListNodeHead) {
        let mut inner = self.inner.get();
        inner._node_head = node_head;
        self.inner.set(inner);
    }
}

impl ITraceableValue for BlockData {
    fn users(&self) -> &UserList {
        &self.users
    }
}

impl BlockData {
    /// 从指令分配器创建一个空的基本块
    ///
    /// 创建一个只包含 Phi 指令结束标记的空基本块。
    ///
    /// # 参数
    /// - `alloc`: 指令分配器的可变引用
    ///
    /// # 返回
    /// 返回新创建的空基本块
    pub fn empty_from_alloc(alloc: &mut Slab<InstData>) -> Self {
        let mut ret = Self {
            inner: Cell::new(BlockDataInner {
                _node_head: SlabListNodeHead::new(),
                _parent_func: GlobalRef::new_null(),
                _id: 0,
            }),
            self_ref: BlockRef::new_null(),
            insts: SlabRefList::from_slab(alloc),
            phi_end: InstRef::new_null(),
            users: UserList::new_empty(),
            preds: PredList::new_empty(),
        };
        let phi_end = {
            let phi_end = InstData::PhiInstEnd(InstCommon::new_empty());
            InstRef::from_alloc(alloc, phi_end)
        };
        ret.insts
            .push_back_ref(alloc, phi_end)
            .expect("Failed to push phi end");
        ret.phi_end = phi_end;
        ret
    }
    /// 从模块创建一个空的基本块
    ///
    /// # 参数
    /// - `module`: IR 模块引用
    ///
    /// # 返回
    /// 返回新创建的空基本块
    pub fn new_empty(module: &Module) -> Self {
        let mut alloc = module.allocs.borrow_mut();
        Self::empty_from_alloc(&mut alloc.insts)
    }

    /// 从可变模块创建一个空的基本块
    ///
    /// 与 `new_empty` 功能相同，但接受可变模块引用，避免借用检查开销。
    ///
    /// # 参数
    /// - `module`: IR 模块的可变引用
    ///
    /// # 返回
    /// 返回新创建的空基本块
    pub fn new_empty_from_mut_module(module: &mut Module) -> Self {
        let alloc = module.allocs.get_mut();
        Self::empty_from_alloc(&mut alloc.insts)
    }

    /// 从指令分配器创建一个包含 unreachable 指令的基本块
    ///
    /// 创建一个包含单个 unreachable 指令的基本块，表示不可达的代码路径。
    ///
    /// # 参数
    /// - `alloc`: 指令分配器的可变引用
    ///
    /// # 返回
    /// 返回包含 unreachable 指令的基本块
    pub fn new_unreachable_from_alloc(alloc: &mut Slab<InstData>) -> Self {
        let ret = Self::empty_from_alloc(alloc);
        let unreachable = {
            let data = InstData::Unreachable(InstCommon::new_empty());
            InstRef::from_alloc(alloc, data)
        };
        ret.insts
            .push_back_ref(alloc, unreachable)
            .expect("Failed to push unreachable inst");
        ret
    }

    /// 从模块创建一个包含 unreachable 指令的基本块
    ///
    /// # 参数
    /// - `module`: IR 模块引用
    ///
    /// # 返回
    /// 返回包含 unreachable 指令的基本块
    pub fn new_unreachable(module: &Module) -> Self {
        let mut alloc = module.allocs.borrow_mut();
        Self::new_unreachable_from_alloc(&mut alloc.insts)
    }

    /// 从可变模块创建一个包含 unreachable 指令的基本块
    ///
    /// # 参数
    /// - `module`: IR 模块的可变引用
    ///
    /// # 返回
    /// 返回包含 unreachable 指令的基本块
    pub fn new_unreachable_from_mut_module(module: &mut Module) -> Self {
        let alloc = module.allocs.get_mut();
        Self::new_unreachable_from_alloc(&mut alloc.insts)
    }

    /// 从指令分配器创建一个返回零值的基本块
    ///
    /// 创建一个包含返回零值指令的基本块，用于函数的默认返回路径。
    /// 零值根据返回类型自动确定（void 类型返回无值，其他类型返回对应的零值）。
    ///
    /// # 参数
    /// - `alloc`: 指令分配器的可变引用
    /// - `ret_ty`: 函数的返回值类型
    ///
    /// # 返回
    /// 返回包含返回零值指令的基本块
    ///
    /// # Panics
    /// 如果返回类型不支持零值构造则会 panic
    pub fn new_return_zero_from_alloc(alloc: &mut Slab<InstData>, ret_ty: ValTypeID) -> Self {
        let ret_bb = Self::empty_from_alloc(alloc);
        let ret_inst = {
            let zero_value = match ret_ty {
                ValTypeID::Void => ValueSSA::None,
                ValTypeID::Ptr
                | ValTypeID::Int(_)
                | ValTypeID::Float(_)
                | ValTypeID::Array(_)
                | ValTypeID::Struct(_) => ConstData::Zero(ret_ty).into_ir(),
                _ => panic!("Unsupported return type {ret_ty:?} for zero return"),
            };
            let retinst = Ret::new_raw(ret_ty);
            // 如上所示, 0 值都是不可追踪的常量, 因此这里直接绕开数据流反图追踪机制.
            retinst.retval().operand.set(zero_value);
            InstRef::from_alloc(alloc, retinst.into_ir())
        };
        ret_bb
            .insts
            .push_back_ref(alloc, ret_inst)
            .expect("Failed to push return inst");
        ret_bb
    }
    /// 从模块创建一个返回零值的基本块
    ///
    /// # 参数
    /// - `module`: IR 模块引用
    /// - `ret_ty`: 函数的返回值类型
    ///
    /// # 返回
    /// 返回包含返回零值指令的基本块
    pub fn new_return_zero(module: &Module, ret_ty: ValTypeID) -> Self {
        let mut alloc = module.allocs.borrow_mut();
        Self::new_return_zero_from_alloc(&mut alloc.insts, ret_ty)
    }

    /// 从可变模块创建一个返回零值的基本块
    ///
    /// # 参数
    /// - `module`: IR 模块的可变引用
    /// - `ret_ty`: 函数的返回值类型
    ///
    /// # 返回
    /// 返回包含返回零值指令的基本块
    pub fn new_return_zero_from_mut_module(module: &mut Module, ret_ty: ValTypeID) -> Self {
        let alloc = module.allocs.get_mut();
        Self::new_return_zero_from_alloc(&mut alloc.insts, ret_ty)
    }
}

impl BlockData {
    /// 获取基本块所属的父函数
    pub fn get_parent_func(&self) -> GlobalRef {
        self.inner.get()._parent_func
    }

    /// 设置基本块所属的父函数
    pub fn set_parent_func(&self, parent: GlobalRef) {
        let mut inner = self.inner.get();
        inner._parent_func = parent;
        self.inner.set(inner);
    }

    /// 加载基本块内所有指令的范围
    ///
    /// # 返回
    /// - `Some(SlabListRange)`: 如果基本块有指令，返回指令范围
    /// - `None`: 如果基本块为空
    pub fn load_inst_range(&self) -> Option<SlabListRange<InstRef>> {
        if self.insts.is_valid() { Some(self.insts.load_range()) } else { None }
    }

    /// 加载基本块内 Phi 指令的范围
    ///
    /// Phi 指令位于基本块的开始部分，用于 SSA 形式的控制流汇合。
    ///
    /// # 返回
    /// - `Some(SlabListRange)`: 如果基本块有 Phi 指令，返回 Phi 指令范围
    /// - `None`: 如果基本块为空
    pub fn try_load_phi_range(&self) -> Option<SlabListRange<InstRef>> {
        if self.insts.is_valid() {
            debug_assert!(
                self.phi_end.is_nonnull(),
                "Phi end should be set if insts is valid"
            );
            let node_head = self.insts._head;
            let node_tail = self.phi_end;
            Some(SlabListRange { node_head, node_tail })
        } else {
            None
        }
    }

    pub fn load_phi_range(&self) -> SlabListRange<InstRef> {
        self.try_load_phi_range()
            .expect("Block instructions are not valid")
    }

    /// 加载基本块内普通指令的范围
    ///
    /// 普通指令位于 Phi 指令之后，包含所有非 Phi 指令。
    ///
    /// # 返回
    /// - `Some(SlabListRange)`: 如果基本块有普通指令，返回普通指令范围
    /// - `None`: 如果基本块为空
    pub fn load_common_range(&self) -> Option<SlabListRange<InstRef>> {
        if self.insts.is_valid() {
            debug_assert!(
                self.phi_end.is_nonnull(),
                "Phi end should be set if insts is valid"
            );
            let node_head = self.phi_end;
            let node_tail = self.insts._tail;
            Some(SlabListRange { node_head, node_tail })
        } else {
            None
        }
    }

    pub fn try_get_terminator(
        &self,
        alloc: &Slab<InstData>,
    ) -> Result<TerminatorRef, &'static str> {
        let insts = &self.insts;
        let Some(back) = insts.get_back_ref(alloc) else {
            return Err("Block has no instructions");
        };
        let terminator = match back.to_data(alloc) {
            InstData::Unreachable(_) => TerminatorRef::Unreachable(back),
            InstData::Ret(_) => TerminatorRef::Ret(RetRef::from_raw_nocheck(back)),
            InstData::Jump(_) => TerminatorRef::Jump(JumpRef::from_raw_nocheck(back)),
            InstData::Br(_) => TerminatorRef::Br(BrRef::from_raw_nocheck(back)),
            InstData::Switch(_) => TerminatorRef::Switch(SwitchRef::from_raw_nocheck(back)),
            _ => return Err("Block does not have a terminator instruction"),
        };
        Ok(terminator)
    }

    pub fn get_terminator(&self, alloc: &Slab<InstData>) -> TerminatorRef {
        self.try_get_terminator(alloc)
            .expect("Block does not have a terminator instruction")
    }

    pub fn has_terminator(&self, alloc: &Slab<InstData>) -> bool {
        self.insts.is_valid() && self.try_get_terminator(alloc).is_ok()
    }

    pub fn get_successors<'a>(&self, alloc: &'a Slab<InstData>) -> JumpTargets<'a> {
        self.get_terminator(alloc).get_jts(alloc)
    }
    pub fn successors_mut<'a>(
        &mut self,
        alloc: &'a mut Slab<InstData>,
    ) -> &'a mut [Rc<JumpTarget>] {
        self.get_terminator(alloc).jts_mut(alloc)
    }

    pub fn has_phi(&self, alloc: &Slab<InstData>) -> bool {
        let range = self.load_phi_range();
        range.calc_length(alloc) > 0
    }

    pub fn is_empty(&self) -> bool {
        !self.insts.is_valid() || self.insts.is_empty()
    }

    pub fn build_add_phi(&self, alloc: &Slab<InstData>, phi: PhiRef) {
        self.insts
            .node_add_prev(alloc, self.phi_end, phi.into_raw())
            .expect("Failed to add phi instruction");
    }
    pub fn build_add_inst(&self, alloc: &Slab<InstData>, inst: impl ISubInstRef) {
        let inst = inst.into_raw();

        match inst.to_data(alloc) {
            InstData::Phi(_) => self.build_add_phi(alloc, PhiRef::from_raw_nocheck(inst)),
            x if x.is_terminator() => {
                if self.has_terminator(alloc) {
                    panic!("Tried to add a terminator instruction to a block that already has one");
                }
                self.insts
                    .push_back_ref(alloc, inst)
                    .expect("Failed to push terminator instruction")
            }
            _ => self
                .insts
                .push_back_ref(alloc, inst)
                .expect("Failed to push instruction"),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BlockDataInner {
    pub(super) _node_head: SlabListNodeHead,
    pub(super) _parent_func: GlobalRef,
    pub(super) _id: usize,
}

/// 基本块引用，用于在 IR 中标识和访问基本块
///
/// 实现了必要的比较和排序 trait，可以用作集合的键值。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct BlockRef(usize);

impl SlabRef for BlockRef {
    type RefObject = BlockData;
    fn from_handle(handle: usize) -> Self {
        BlockRef(handle)
    }
    fn get_handle(&self) -> usize {
        self.0
    }
}

impl SlabListNodeRef for BlockRef {
    fn on_node_push_next(
        curr: Self,
        next: Self,
        alloc: &Slab<BlockData>,
    ) -> Result<(), SlabListError> {
        if curr == next {
            return Err(SlabListError::RepeatedNode(next.get_handle()));
        }
        let self_parent = curr.to_data(alloc).get_parent_func();
        next.to_data(alloc).set_parent_func(self_parent);
        Ok(())
    }

    fn on_node_push_prev(
        curr: Self,
        prev: Self,
        alloc: &Slab<BlockData>,
    ) -> Result<(), SlabListError> {
        if curr == prev {
            Err(SlabListError::RepeatedNode(prev.get_handle()))
        } else {
            let self_parent = curr.to_data(alloc).get_parent_func();
            prev.to_data(alloc).set_parent_func(self_parent);
            Ok(())
        }
    }

    fn on_node_unplug(curr: Self, alloc: &Slab<BlockData>) -> Result<(), SlabListError> {
        let self_data = curr.to_data(alloc);
        if self_data.get_parent_func().is_null() {
            Err(SlabListError::UnpluggedItemAttached(curr.get_handle()))
        } else {
            self_data.set_parent_func(GlobalRef::new_null());
            Ok(())
        }
    }
}

impl ISubValueSSA for BlockRef {
    fn try_from_ir(value: &ValueSSA) -> Option<&Self> {
        match value {
            ValueSSA::Block(bb) => Some(bb),
            _ => None,
        }
    }
    fn into_ir(self) -> ValueSSA {
        ValueSSA::Block(self)
    }

    fn get_valtype(self, _: &IRAllocs) -> ValTypeID {
        ValTypeID::Void
    }

    fn try_gettype_noalloc(self) -> Option<ValTypeID> {
        Some(ValTypeID::Void)
    }

    fn is_zero(&self, _: &IRAllocs) -> bool {
        false
    }

    fn fmt_ir(&self, writer: &IRWriter) -> std::io::Result<()> {
        let number = writer.borrow_numbers().block_get_number(*self);
        if let Some(number) = number {
            writer.wrap_indent();
            write!(writer.output.borrow_mut(), "%{number}:")?;
        }
        writer.inc_indent();
        let alloc_block = &writer.allocs.blocks;
        let alloc_inst = &writer.allocs.insts;
        for (instref, inst) in self.insts_from_alloc(alloc_block).view(alloc_inst) {
            writer.wrap_indent();
            let number = writer.borrow_numbers().inst_get_number(instref);
            inst.fmt_ir(number, writer)?;
        }
        writer.dec_indent();
        writer.wrap_indent();
        Ok(())
    }
}

impl BlockRef {
    /// 从 IR 分配器创建基本块引用
    ///
    /// 将基本块数据插入分配器并返回对应的引用。
    /// 同时更新基本块内所有指令的父基本块信息和用户的操作数。
    ///
    /// # 参数
    /// - `allocs`: IR 分配器的可变引用
    /// - `data`: 要插入的基本块数据
    ///
    /// # 返回
    /// 返回新创建的基本块引用
    pub fn from_allocs(allocs: &mut IRAllocs, mut data: BlockData) -> Self {
        let ret = BlockRef(allocs.blocks.vacant_key());
        data.self_ref = ret;
        data.insts.forall_nodes(&allocs.insts, |_, inst| {
            inst.set_parent_bb(ret);
            ControlFlow::Continue(())
        });
        for user in &data.users {
            user.operand.set(ValueSSA::Block(ret));
        }
        allocs.blocks.insert(data);
        ret
    }

    pub fn new_unreachable(allocs: &mut IRAllocs) -> Self {
        let data = BlockData::new_unreachable_from_alloc(&mut allocs.insts);
        BlockRef::from_allocs(allocs, data)
    }

    pub fn new_return_zero(allocs: &mut IRAllocs, ret_ty: ValTypeID) -> Self {
        let data = BlockData::new_return_zero_from_alloc(&mut allocs.insts, ret_ty);
        BlockRef::from_allocs(allocs, data)
    }

    /// 从分配器获取基本块的指令列表
    ///
    /// # 参数
    /// - `alloc`: 基本块分配器的引用
    ///
    /// # 返回
    /// 返回基本块指令列表的引用
    pub fn insts_from_alloc<'a>(self, alloc: &'a Slab<BlockData>) -> &'a SlabRefList<InstRef> {
        &self.to_data(alloc).insts
    }

    /// 从模块获取基本块的指令列表
    ///
    /// # 参数
    /// - `module`: IR 模块引用
    ///
    /// # 返回
    /// 返回基本块指令列表的借用引用
    pub fn insts<'a>(self, module: &'a Module) -> Ref<'a, SlabRefList<InstRef>> {
        let allocs = module.allocs.borrow();
        Ref::map(allocs, |allocs| self.insts_from_alloc(&allocs.blocks))
    }

    /// 从可变模块获取基本块的指令列表
    ///
    /// # 参数
    /// - `module`: IR 模块的可变引用
    ///
    /// # 返回
    /// 返回基本块指令列表的引用
    pub fn insts_from_mut_module<'a>(self, module: &'a mut Module) -> &'a SlabRefList<InstRef> {
        let alloc = module.allocs.get_mut();
        self.insts_from_alloc(&alloc.blocks)
    }
}

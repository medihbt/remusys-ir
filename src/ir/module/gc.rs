use crate::{
    base::{FixBitSet, INullableValue, SlabListNode, SlabListNodeHead, SlabRef},
    ir::{
        AttrList, AttrListID, BlockData, BlockRef, ConstExprData, ExprRef, Func, GlobalData,
        GlobalRef, IRAllocs, ISubGlobal, ISubInst, ISubValueSSA, IUser, InstData, InstRef,
        JumpTarget, TerminatorRef, Use, UserID, ValueSSA,
    },
};
use std::{collections::VecDeque, ops::ControlFlow, rc::Rc, usize};

#[derive(Debug, Clone)]
pub struct IRLiveValueSet {
    pub insts: FixBitSet<4>,
    pub blocks: FixBitSet<1>,
    pub exprs: FixBitSet<2>,
    pub globals: FixBitSet<1>,
    pub attrs: FixBitSet<1>,
}

impl IRLiveValueSet {
    /// 创建一个新的 IRLiveValueSet，使用给定的 IRAllocs 的容量
    pub fn from_allocs(allocs: &IRAllocs) -> Self {
        Self {
            insts: FixBitSet::with_len(allocs.insts.capacity()),
            blocks: FixBitSet::with_len(allocs.blocks.capacity()),
            exprs: FixBitSet::with_len(allocs.exprs.capacity()),
            globals: FixBitSet::with_len(allocs.globals.capacity()),
            attrs: FixBitSet::with_len(allocs.attrs.capacity()),
        }
    }

    pub fn add(&mut self, value: impl ISubValueSSA) {
        let value = value.into_ir();
        match value {
            ValueSSA::ConstData(_) | ValueSSA::AggrZero(_) => {} // ConstData 是不可变的，不需要 GC
            ValueSSA::ConstExpr(expr) => self.exprs.enable(expr.get_handle()),
            ValueSSA::FuncArg(func, _) => self.globals.enable(func.get_handle()),
            ValueSSA::Block(block) => self.blocks.enable(block.get_handle()),
            ValueSSA::Inst(inst) => self.insts.enable(inst.get_handle()),
            ValueSSA::Global(global) => self.globals.enable(global.get_handle()),
            ValueSSA::None => {} // None 值不需要标记
        }
    }

    pub fn is_live(&self, value: impl ISubValueSSA) -> bool {
        let value = value.into_ir();
        match value {
            ValueSSA::ConstData(_) | ValueSSA::AggrZero(_) => true, // ConstData 总是存活的
            ValueSSA::ConstExpr(expr) => self.exprs.get(expr.get_handle()),
            ValueSSA::FuncArg(func, _) => self.globals.get(func.get_handle()),
            ValueSSA::Block(block) => self.blocks.get(block.get_handle()),
            ValueSSA::Inst(inst) => self.insts.get(inst.get_handle()),
            ValueSSA::Global(global) => self.globals.get(global.get_handle()),
            ValueSSA::None => true, // None 值总是存活的
        }
    }

    /// 从 allocs 中清理未标记存活的值.
    pub fn sweep(&self, allocs: &mut IRAllocs) {
        allocs.exprs.retain(|expr, _| self.exprs.get(expr));
        allocs.insts.retain(|inst, _| self.insts.get(inst));
        allocs.blocks.retain(|block, _| self.blocks.get(block));
        allocs.globals.retain(|global, _| self.globals.get(global));
        allocs.attrs.retain(|attr, _| self.attrs.get(attr));
    }
}

pub struct IRValueMarker<'a> {
    pub live_set: IRLiveValueSet,
    pub mark_queue: VecDeque<ValueSSA>,
    pub attrs_queue: VecDeque<AttrListID>,
    pub allocs: &'a mut IRAllocs,
}

impl<'a> IRValueMarker<'a> {
    pub fn from_allocs(allocs: &'a mut IRAllocs) -> Self {
        Self {
            live_set: IRLiveValueSet::from_allocs(allocs),
            mark_queue: VecDeque::new(),
            attrs_queue: VecDeque::new(),
            allocs,
        }
    }

    pub fn push_mark(&mut self, value: impl ISubValueSSA) {
        let Self { live_set, mark_queue, .. } = self;
        Self::do_push_mark(live_set, mark_queue, value);
    }
    pub fn push_mark_attr(&mut self, attr: AttrListID) {
        let Self { live_set, attrs_queue, .. } = self;
        if live_set.attrs.get(attr.get_handle()) {
            return;
        }
        live_set.attrs.enable(attr.get_handle());
        attrs_queue.push_back(attr);
    }
    pub fn mark_leaf(&mut self, value: impl ISubValueSSA) {
        let Self { live_set, .. } = self;
        let value = value.into_ir();
        if live_set.is_live(value) {
            return;
        }
        live_set.add(value);
        // 不将 value 放入 mark_queue，因此不会遍历其子对象
    }

    pub fn mark_all(&mut self) {
        while let Some(value) = self.mark_queue.pop_front() {
            self.consume_one(value);
        }
    }

    pub fn sweep(&mut self) {
        self.live_set.sweep(self.allocs);
    }

    /// 执行完整的 mark-and-sweep 垃圾回收
    ///
    /// ### 参数
    /// - `roots`: 根对象集合，从这些对象开始标记
    pub fn mark_and_sweep(&mut self, roots: impl IntoIterator<Item = ValueSSA>) {
        // 标记阶段：从根对象开始标记所有可达对象
        for root in roots {
            self.push_mark(root);
        }
        self.mark_all();

        // 清理阶段：删除未标记的对象
        self.sweep();
    }

    fn do_push_mark(
        live_set: &mut IRLiveValueSet,
        mark_queue: &mut VecDeque<ValueSSA>,
        value: impl ISubValueSSA,
    ) {
        let value = value.into_ir();
        if live_set.is_live(value) {
            return;
        }
        live_set.add(value);
        mark_queue.push_back(value);
    }

    fn consume_one(&mut self, value: ValueSSA) {
        match value {
            ValueSSA::ConstData(_) => {} // ConstData 不包含引用，无需遍历
            ValueSSA::AggrZero(_) => {}  // AggrZero 不包含引用，无需遍历
            ValueSSA::ConstExpr(expr) => self.consume_expr(expr),
            ValueSSA::Block(block) => self.consume_block(block),
            ValueSSA::Inst(inst) => self.consume_inst(inst),
            ValueSSA::Global(global) => self.consume_global(global),
            ValueSSA::FuncArg(_, _) => {} // FuncArg 只是对 Global 的索引，不需要额外处理
            ValueSSA::None => {}          // None 值无需处理
        }
    }

    fn consume_expr(&mut self, expr: ExprRef) {
        let Self { live_set, mark_queue, allocs, .. } = self;
        let expr_data = expr.to_data(&allocs.exprs);
        for elem in &expr_data.get_operands() {
            Self::do_push_mark(live_set, mark_queue, elem.get_operand());
        }
    }

    fn consume_block(&mut self, block: BlockRef) {
        let Self { live_set, mark_queue, allocs, .. } = self;
        let block_data = block.to_data(&allocs.blocks);
        block_data.insts.forall_nodes(&allocs.insts, |&iref, _| {
            Self::do_push_mark(live_set, mark_queue, iref);
            ControlFlow::Continue(())
        });
    }

    fn consume_inst(&mut self, inst: InstRef) {
        let Self { live_set, mark_queue, allocs, .. } = self;
        let inst_data = inst.to_data(&allocs.insts);
        for useref in &inst_data.get_operands() {
            Self::do_push_mark(live_set, mark_queue, useref.get_operand());
        }
        if let Some(terminator) = TerminatorRef::try_from_instref(inst, &allocs.insts) {
            for jt in &terminator.get_jts(&allocs.insts) {
                Self::do_push_mark(live_set, mark_queue, jt.get_block());
            }
        }
        if let InstData::Call(callop) = inst_data {
            for attr in callop.attrs.iter() {
                let attr = attr.borrow();
                Self::mark_all_attrs(live_set, &mut self.attrs_queue, &attr.includes, allocs);
            }
        };
    }

    fn consume_global(&mut self, global: GlobalRef) {
        let Self { live_set, mark_queue, attrs_queue, allocs } = self;
        match global.to_data(&allocs.globals) {
            GlobalData::Var(var) => Self::do_push_mark(live_set, mark_queue, var.get_init()),
            GlobalData::Func(func) => {
                Self::consume_func(live_set, mark_queue, attrs_queue, allocs, func)
            }
        }
    }

    fn consume_func(
        live_set: &mut IRLiveValueSet,
        mark_queue: &mut VecDeque<ValueSSA>,
        attrs_queue: &mut VecDeque<AttrListID>,
        allocs: &IRAllocs,
        func: &Func,
    ) {
        func.with_attrs(|attrs| {
            Self::mark_all_attrs(live_set, attrs_queue, &attrs.includes, allocs);
        });
        for arg in func.args.iter() {
            arg.with_attrs(|attrs| {
                Self::mark_all_attrs(live_set, attrs_queue, &attrs.includes, allocs);
            });
        }
        let Some(body) = func.get_body() else { return };
        body.forall_nodes(&allocs.blocks, |&bref, _| {
            Self::do_push_mark(live_set, mark_queue, bref);
            ControlFlow::Continue(())
        });
    }

    fn mark_attr_include(
        live_set: &mut IRLiveValueSet,
        attrs_queue: &mut VecDeque<AttrListID>,
        attrs: &[AttrListID],
    ) {
        for &include in attrs {
            live_set.attrs.enable(include.get_handle());
            attrs_queue.push_back(include);
        }
    }
    fn mark_all_attrs(
        live_set: &mut IRLiveValueSet,
        attrs_queue: &mut VecDeque<AttrListID>,
        attrs: &[AttrListID],
        allocs: &IRAllocs,
    ) {
        Self::mark_attr_include(live_set, attrs_queue, attrs);
        while let Some(attr) = attrs_queue.pop_front() {
            let attr = attr.to_data(&allocs.attrs);
            Self::mark_attr_include(live_set, attrs_queue, &attr.includes);
        }
    }
}

#[derive(Debug, Clone)]
pub struct IRValueCompactMap {
    pub insts: Vec<InstRef>,
    pub blocks: Vec<BlockRef>,
    pub exprs: Vec<ExprRef>,
    pub globals: Vec<GlobalRef>,
    pub attrs: Vec<AttrListID>,
}

impl IRValueCompactMap {
    fn from_liveset(liveset: &IRLiveValueSet) -> Self {
        Self {
            insts: Self::build_vecmap(&liveset.insts),
            blocks: Self::build_vecmap(&liveset.blocks),
            exprs: Self::build_vecmap(&liveset.exprs),
            globals: Self::build_vecmap(&liveset.globals),
            attrs: Self::build_vecmap(&liveset.attrs),
        }
    }

    fn build_vecmap<T: SlabRef, const N: usize>(bitset: &FixBitSet<N>) -> Vec<T> {
        let mut v = vec![T::new_null(); bitset.compact_len()];
        for live in bitset.iter() {
            v[live] = T::from_handle(live);
        }
        v
    }

    pub fn redirect_value(&self, value: impl ISubValueSSA) -> ValueSSA {
        let value = value.into_ir();
        match value {
            ValueSSA::ConstData(_) | ValueSSA::AggrZero(_) => value, // ConstData 保持不变
            ValueSSA::ConstExpr(expr) => {
                let new_expr = self.exprs[expr.get_handle()];
                ValueSSA::ConstExpr(new_expr)
            }
            ValueSSA::FuncArg(func, index) => {
                let new_func = self.globals[func.get_handle()];
                ValueSSA::FuncArg(new_func, index)
            }
            ValueSSA::Block(block) => {
                let new_block = self.blocks[block.get_handle()];
                ValueSSA::Block(new_block)
            }
            ValueSSA::Inst(inst) => {
                let new_inst = self.insts[inst.get_handle()];
                ValueSSA::Inst(new_inst)
            }
            ValueSSA::Global(global) => {
                let new_global = self.globals[global.get_handle()];
                ValueSSA::Global(new_global)
            }
            ValueSSA::None => ValueSSA::None, // None 保持不变
        }
    }

    pub fn redirect_inst(&self, inst: InstRef) -> InstRef {
        if inst.is_null() { inst } else { self.insts[inst.get_handle()] }
    }
    pub fn redirect_global(&self, global: GlobalRef) -> GlobalRef {
        if global.is_null() { global } else { self.globals[global.get_handle()] }
    }
    pub fn redirect_block(&self, block: BlockRef) -> BlockRef {
        if block.is_null() || block.is_vexit() { block } else { self.blocks[block.get_handle()] }
    }
    pub fn redirect_expr(&self, expr: ExprRef) -> ExprRef {
        if expr.is_null() { expr } else { self.exprs[expr.get_handle()] }
    }
    pub fn redirect_attr(&self, attr: AttrListID) -> AttrListID {
        if attr.is_null() { attr } else { self.attrs[attr.get_handle()] }
    }
}

impl<'a> IRValueMarker<'a> {
    fn compact(&mut self) -> IRValueCompactMap {
        let Self { live_set, allocs, .. } = self;
        let mut compact_map = IRValueCompactMap::from_liveset(live_set);

        allocs.globals.compact(|_, old, new| {
            compact_map.globals[old] = GlobalRef::from_handle(new);
            true
        });
        allocs.blocks.compact(|_, old, new| {
            compact_map.blocks[old] = BlockRef::from_handle(new);
            true
        });
        allocs.insts.compact(|_, old, new| {
            compact_map.insts[old] = InstRef::from_handle(new);
            true
        });
        allocs.exprs.compact(|_, old, new| {
            compact_map.exprs[old] = ExprRef::from_handle(new);
            true
        });
        allocs.attrs.compact(|_, old, new| {
            compact_map.attrs[old] = AttrListID::from_handle(new);
            true
        });

        for (id, inst) in allocs.insts.iter_mut() {
            Self::fix_inst(&compact_map, id, inst);
        }
        for (id, block) in allocs.blocks.iter_mut() {
            Self::fix_block(&compact_map, id, block);
        }
        for (id, global) in allocs.globals.iter_mut() {
            Self::fix_global(&compact_map, id, global);
        }
        for (id, expr) in allocs.exprs.iter_mut() {
            Self::fix_expr(&compact_map, id, expr);
        }
        for (id, attrs) in allocs.attrs.iter_mut() {
            Self::fix_attrs(&compact_map, id, attrs);
        }

        compact_map
    }

    fn redirect_jts(compact_map: &IRValueCompactMap, new: InstRef, jts: &[Rc<JumpTarget>]) {
        for jt in jts {
            jt.set_terminator(new);
            jt.block.set(compact_map.redirect_block(jt.get_block()));
        }
    }
    fn redirect_use(compact_map: &IRValueCompactMap, new: impl Into<UserID>, uses: &[Rc<Use>]) {
        let new = new.into();
        for u in uses {
            u.user.set(new);
            u.operand.set(compact_map.redirect_value(u.get_operand()));
        }
    }

    fn fix_inst(compact_map: &IRValueCompactMap, id: usize, inst: &mut InstData) {
        let new = InstRef::from_handle(id);
        inst.set_parent_bb(compact_map.redirect_block(inst.get_parent_bb()));
        inst.common_mut().self_ref = new;
        inst.store_node_head({
            let SlabListNodeHead { prev, next } = inst.load_node_head();
            SlabListNodeHead {
                prev: compact_map
                    .redirect_inst(InstRef::from_handle(prev))
                    .get_handle(),
                next: compact_map
                    .redirect_inst(InstRef::from_handle(next))
                    .get_handle(),
            }
        });
        Self::redirect_use(compact_map, new, &inst.get_operands());
        if let Some(jts) = inst.try_get_jts() {
            Self::redirect_jts(compact_map, new, &jts);
        }
        if let InstData::Call(callop) = inst {
            for attr in callop.attrs.iter_mut() {
                Self::fix_attrs(compact_map, usize::MAX, attr.get_mut());
            }
        }
    }

    fn fix_block(compact_map: &IRValueCompactMap, id: usize, block: &mut BlockData) {
        block.self_ref = BlockRef::from_handle(id);
        block.set_parent_func({
            let parent = block.get_parent_func();
            compact_map.redirect_global(parent)
        });
        block.store_node_head({
            let SlabListNodeHead { prev, next } = block.load_node_head();
            SlabListNodeHead {
                prev: compact_map
                    .redirect_block(BlockRef::from_handle(prev))
                    .get_handle(),
                next: compact_map
                    .redirect_block(BlockRef::from_handle(next))
                    .get_handle(),
            }
        });
        block.insts._head = compact_map.redirect_inst(block.insts._head);
        block.insts._tail = compact_map.redirect_inst(block.insts._tail);
        block.phi_end = compact_map.redirect_inst(block.phi_end);
    }

    fn fix_global(compact_map: &IRValueCompactMap, id: usize, global: &mut GlobalData) {
        let new = GlobalRef::from_handle(id);
        global.common_mut().self_ref = new;
        Self::redirect_use(compact_map, new, &global.get_operands());

        let GlobalData::Func(func) = global else { return };
        func.with_attrs_mut(|attrs| {
            Self::fix_attrs(compact_map, usize::MAX, attrs);
        });
        for arg in func.args.iter_mut() {
            arg.with_attrs_mut(|attrs| {
                Self::fix_attrs(compact_map, usize::MAX, attrs);
            });
        }

        func.entry.set(compact_map.redirect_block(func.entry.get()));
        func.body._head = compact_map.redirect_block(func.body._head);
        func.body._tail = compact_map.redirect_block(func.body._tail);
    }

    fn fix_expr(compact_map: &IRValueCompactMap, id: usize, expr: &mut ConstExprData) {
        Self::redirect_use(compact_map, ExprRef::from_handle(id), &expr.get_operands());
    }

    fn fix_attrs(compact_map: &IRValueCompactMap, id: usize, attrs: &mut AttrList) {
        if attrs.self_id.is_nonnull() {
            attrs.self_id = AttrListID::from_handle(id);
        }
        for attr in attrs.includes.iter_mut() {
            *attr = compact_map.redirect_attr(*attr);
        }
    }

    /// 标记-压缩法垃圾回收. 由于 Slab allocator 限制, 实际行为是: 先执行一次标记-清除,
    /// 然后就地压缩.
    ///
    /// ### Returns
    ///
    /// * `IRValueCompactMap` -- 从原始索引到压缩后索引的映射.
    ///
    /// ### Warning
    ///
    /// 注意: 绝大多数情况下不要调用这个压缩函数. 否则, 全局对象移动以后, IR Module 内部的引用可能会变得无效.
    pub(super) fn mark_and_compact(
        mut self,
        roots: impl IntoIterator<Item = ValueSSA>,
    ) -> IRValueCompactMap {
        self.mark_and_sweep(roots);
        self.compact()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ir::{ConstData, GlobalData, GlobalRef, Var};
    use crate::typing::{ScalarType, ValTypeID};

    #[test]
    fn test_gc_basic_marking() {
        let mut allocs = IRAllocs::new();

        // 创建一些测试对象
        let var = Var::new_extern("test_var".to_string(), ValTypeID::Int(32), 4);
        let global_handle = allocs.globals.insert(GlobalData::Var(var));
        let global_ref = GlobalRef::from_handle(global_handle);

        // 测试标记
        let mut marker = IRValueMarker::from_allocs(&mut allocs);
        marker.push_mark(ValueSSA::Global(global_ref));
        marker.mark_all();

        // 验证标记结果
        assert!(marker.live_set.is_live(ValueSSA::Global(global_ref)));
        assert!(marker.live_set.globals.get(global_handle));
    }

    #[test]
    fn test_gc_value_types() {
        let allocs = IRAllocs::new();
        let live_set = IRLiveValueSet::from_allocs(&allocs);

        // 测试不同类型的 ValueSSA 处理
        assert!(live_set.is_live(ValueSSA::None)); // None 总是存活
        assert!(live_set.is_live(ValueSSA::ConstData(ConstData::Zero(ScalarType::Int(32))))); // ConstData 总是存活
    }
}

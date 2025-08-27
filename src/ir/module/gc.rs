use std::{collections::VecDeque, ops::ControlFlow};

use crate::{
    base::{FixBitSet, SlabRef},
    ir::{
        BlockRef, ConstExprData, ExprRef, GlobalData, GlobalRef, IRAllocs, ISubValueSSA, IUser,
        InstRef, TerminatorRef, ValueSSA,
    },
};

#[derive(Debug, Clone)]
pub struct IRLiveValueSet {
    pub insts: FixBitSet<4>,
    pub blocks: FixBitSet<1>,
    pub exprs: FixBitSet<2>,
    pub globals: FixBitSet<1>,
}

impl IRLiveValueSet {
    /// 创建一个新的 IRLiveValueSet，使用给定的 IRAllocs 的容量
    pub fn from_allocs(allocs: &IRAllocs) -> Self {
        Self {
            insts: FixBitSet::with_len(allocs.insts.capacity()),
            blocks: FixBitSet::with_len(allocs.blocks.capacity()),
            exprs: FixBitSet::with_len(allocs.exprs.capacity()),
            globals: FixBitSet::with_len(allocs.globals.capacity()),
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
    }
}

pub struct IRValueMarker<'a> {
    pub live_set: IRLiveValueSet,
    pub mark_queue: VecDeque<ValueSSA>,
    pub allocs: &'a mut IRAllocs,
}

impl<'a> IRValueMarker<'a> {
    pub fn from_allocs(allocs: &'a mut IRAllocs) -> Self {
        Self {
            live_set: IRLiveValueSet::from_allocs(allocs),
            mark_queue: VecDeque::new(),
            allocs,
        }
    }

    pub fn push_mark(&mut self, value: impl ISubValueSSA) {
        let Self { live_set, mark_queue, .. } = self;
        Self::do_push_mark(live_set, mark_queue, value);
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
        let Self { live_set, mark_queue, allocs } = self;
        let expr_data = expr.to_data(&allocs.exprs);
        let elems = match expr_data {
            ConstExprData::Array(arr) => &arr.elems,
            ConstExprData::Struct(st) => &st.elems,
        };
        for elem in elems {
            Self::do_push_mark(live_set, mark_queue, elem.get_operand());
        }
    }

    fn consume_block(&mut self, block: BlockRef) {
        let Self { live_set, mark_queue, allocs } = self;
        let block_data = block.to_data(&allocs.blocks);
        block_data.insts.forall_nodes(&allocs.insts, |&iref, _| {
            Self::do_push_mark(live_set, mark_queue, iref);
            ControlFlow::Continue(())
        });
    }

    fn consume_inst(&mut self, inst: InstRef) {
        let Self { live_set, mark_queue, allocs } = self;
        let inst_data = inst.to_data(&allocs.insts);
        for useref in &inst_data.get_operands() {
            Self::do_push_mark(live_set, mark_queue, useref.get_operand());
        }
        if let Some(terminator) = TerminatorRef::try_from_instref(inst, &allocs.insts) {
            for jt in &terminator.get_jts(&allocs.insts) {
                Self::do_push_mark(live_set, mark_queue, jt.get_block());
            }
        }
    }

    fn consume_global(&mut self, global: GlobalRef) {
        let Self { live_set, mark_queue, allocs } = self;
        match global.to_data(&allocs.globals) {
            GlobalData::Var(var) => Self::do_push_mark(live_set, mark_queue, var.get_init()),
            GlobalData::Func(func) => {
                let Some(body) = func.get_body() else { return };
                body.forall_nodes(&allocs.blocks, |&bref, _| {
                    Self::do_push_mark(live_set, mark_queue, bref);
                    ControlFlow::Continue(())
                });
            }
        }
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

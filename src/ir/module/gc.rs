use crate::{
    base::FixBitSet,
    ir::{
        BlockID, ExprID, FuncID, GlobalID, GlobalObj, IRAllocs, ISubExprID, ISubGlobal,
        ISubGlobalID, ISubInst, ISubInstID, ISubValueSSA, ITraceableValue, IUser, InstID,
        PoolAllocatedClass, PoolAllocatedID, ValueSSA, module::allocs::IPoolAllocated,
    },
};
use mtb_entity_slab::IndexedID;
use std::collections::VecDeque;

#[derive(Debug, Clone)]
pub struct IRLiveSet {
    pub exprs: FixBitSet<4>,
    pub insts: FixBitSet<8>,
    pub globals: FixBitSet<2>,
    pub blocks: FixBitSet<4>,
    /// capacity 太大了, 内联存储不值得, 直接用 Large 变体
    pub uses: FixBitSet<1>,
    pub jts: FixBitSet<4>,
}

impl IRLiveSet {
    /// 创建一个新的 IRLiveValueSet，使用给定的 IRAllocs 的容量
    pub fn from_allocs(allocs: &IRAllocs) -> Self {
        Self {
            insts: FixBitSet::with_len(allocs.insts.capacity()),
            blocks: FixBitSet::with_len(allocs.blocks.capacity()),
            exprs: FixBitSet::with_len(allocs.exprs.capacity()),
            globals: FixBitSet::with_len(allocs.globals.capacity()),
            uses: FixBitSet::with_len(allocs.uses.capacity()),
            jts: FixBitSet::with_len(allocs.jts.capacity()),
        }
    }

    pub fn add(&mut self, allocs: &IRAllocs, id: impl Into<PoolAllocatedID>) {
        let id = id.into();
        let Some(index) = id.get_indexed(allocs) else {
            return;
        };
        match id.get_class() {
            PoolAllocatedClass::Inst => self.insts.enable(index),
            PoolAllocatedClass::Block => self.blocks.enable(index),
            PoolAllocatedClass::Expr => self.exprs.enable(index),
            PoolAllocatedClass::Global => self.globals.enable(index),
            PoolAllocatedClass::Use => self.uses.enable(index),
            PoolAllocatedClass::JumpTarget => self.jts.enable(index),
        }
    }
    pub fn is_alive(&self, allocs: &IRAllocs, id: impl Into<PoolAllocatedID>) -> bool {
        let id = id.into();
        let Some(index) = id.get_indexed(allocs) else {
            return false;
        };
        match id.get_class() {
            PoolAllocatedClass::Inst => self.insts.get(index),
            PoolAllocatedClass::Block => self.blocks.get(index),
            PoolAllocatedClass::Expr => self.exprs.get(index),
            PoolAllocatedClass::Global => self.globals.get(index),
            PoolAllocatedClass::Use => self.uses.get(index),
            PoolAllocatedClass::JumpTarget => self.jts.get(index),
        }
    }
    pub fn add_value(&mut self, allocs: &IRAllocs, value: impl ISubValueSSA) {
        match value.into_ir() {
            ValueSSA::Inst(id) => self.add(allocs, id),
            ValueSSA::Block(id) => self.add(allocs, id),
            ValueSSA::ConstExpr(id) => self.add(allocs, id),
            ValueSSA::Global(id) => self.add(allocs, id),
            ValueSSA::FuncArg(func, _) => self.add(allocs, func.into_global()),
            _ => {}
        }
    }
    pub fn value_is_alive(&self, allocs: &IRAllocs, value: impl ISubValueSSA) -> bool {
        match value.into_ir() {
            ValueSSA::Inst(id) => self.is_alive(allocs, id),
            ValueSSA::Block(id) => self.is_alive(allocs, id),
            ValueSSA::ConstExpr(id) => self.is_alive(allocs, id),
            ValueSSA::Global(id) => self.is_alive(allocs, id),
            ValueSSA::FuncArg(func, _) => self.is_alive(allocs, func.into_global()),
            _ => false,
        }
    }

    pub fn sweep(&self, allocs: &mut IRAllocs) -> usize {
        assert_eq!(
            allocs.num_pending_disposed(),
            0,
            "Cannot sweep while there are disposed objects pending cleanup."
        );
        // use 和 jt 在 dispose 时会维护环链表, 如果不 dispose 会破坏 use-def 关系
        for (id, up, u) in allocs.uses.iter() {
            if self.uses.get(id) {
                continue;
            }
            // 重复 dispose 不是一个错误, 忽略即可.
            let _ = u.dispose_obj(up, allocs);
            allocs.push_disposed(up);
        }
        for (id, jp, jt) in allocs.jts.iter() {
            if self.jts.get(id) {
                continue;
            }
            // 重复 dispose 不是一个错误, 忽略即可.
            let _ = jt.dispose_obj(jp, allocs);
            allocs.push_disposed(jp);
        }
        let mut num_freed = allocs.num_pending_disposed();
        // 清理掉已经 dispose 的对象.
        allocs.free_disposed();
        // 其他的对象就不需要 dispose 了. 直接 free 就行.
        allocs.insts.fully_free_if(
            |_, _, IndexedID(idx, _)| !self.insts.get(idx),
            |_| num_freed += 1,
        );
        allocs.blocks.fully_free_if(
            |_, _, IndexedID(idx, _)| !self.blocks.get(idx),
            |_| num_freed += 1,
        );
        allocs.exprs.fully_free_if(
            |_, _, IndexedID(idx, _)| !self.exprs.get(idx),
            |_| num_freed += 1,
        );
        allocs.globals.fully_free_if(
            |_, _, IndexedID(idx, _)| !self.globals.get(idx),
            |_| num_freed += 1,
        );
        num_freed
    }
}

pub struct IRMarker<'ir> {
    pub live_set: IRLiveSet,
    pub mark_queue: VecDeque<PoolAllocatedID>,
    pub ir_allocs: &'ir mut IRAllocs,
}
struct IRMarkerRef<'ir> {
    live: &'ir mut IRLiveSet,
    queue: &'ir mut VecDeque<PoolAllocatedID>,
    allocs: &'ir IRAllocs,
}

impl<'ir> IRMarker<'ir> {
    pub fn new(ir_allocs: &'ir mut IRAllocs) -> Self {
        Self {
            live_set: IRLiveSet::from_allocs(ir_allocs),
            mark_queue: VecDeque::new(),
            ir_allocs,
        }
    }
    pub fn finish(mut self) {
        self.mark_all();
        let Self { live_set, ir_allocs, .. } = self;
        let num_freed = live_set.sweep(ir_allocs);
        log::debug!("IR GC: freed {num_freed} allocations.");
    }

    pub fn push_mark(&mut self, id: impl Into<PoolAllocatedID>) -> &mut Self {
        let id = id.into();
        Self::do_push_mark(&mut self.borrow_proxy(), id);
        self
    }
    pub fn push_mark_value(&mut self, value: impl ISubValueSSA) -> &mut Self {
        let val = value.into_ir();
        match val {
            ValueSSA::Inst(id) => self.push_mark(id),
            ValueSSA::Block(id) => self.push_mark(id),
            ValueSSA::ConstExpr(id) => self.push_mark(id),
            ValueSSA::Global(id) => self.push_mark(id),
            ValueSSA::FuncArg(func, _) => self.push_mark(func.into_global()),
            _ => self,
        }
    }
    pub fn mark_leaf(&mut self, id: impl Into<PoolAllocatedID>) {
        self.live_set.add(self.ir_allocs, id);
        // 不将 value 放入 mark_queue，因此不会遍历其子对象
    }

    pub fn mark_all(&mut self) {
        while let Some(id) = self.mark_queue.pop_front() {
            use PoolAllocatedID::*;
            log::trace!("GC marking {:?}", id);
            match id {
                Block(b) => self.consume_block(b),
                Inst(i) => self.consume_inst(i),
                Expr(e) => self.consume_expr(e),
                Global(g) => self.consume_global(g),
                Use(u) => {
                    let val = u.get_operand(&self.ir_allocs);
                    self.push_mark_value(val);
                }
                JumpTarget(jt) => {
                    let Some(bb) = jt.get_block(&self.ir_allocs) else {
                        continue;
                    };
                    self.push_mark(bb);
                }
            }
        }
    }
    fn do_push_mark(proxy: &mut IRMarkerRef, id: impl Into<PoolAllocatedID>) {
        let IRMarkerRef { live, queue, allocs } = proxy;
        let id = id.into();
        if !live.is_alive(allocs, id) {
            live.add(allocs, id);
            queue.push_back(id);
        }
    }
    fn borrow_proxy(&mut self) -> IRMarkerRef<'_> {
        let Self { live_set, mark_queue, ir_allocs } = self;
        IRMarkerRef { live: live_set, queue: mark_queue, allocs: ir_allocs }
    }
    fn consume_block(&mut self, block: BlockID) {
        let mut proxy = self.borrow_proxy();
        let allocs = proxy.allocs;
        let Some(body) = block.deref_ir(allocs).try_get_body() else {
            return; // sentinel 也会被扫描到, 跳过
        };
        body.insts.forall_with_sentinel(&allocs.insts, |ip, inst| {
            assert_eq!(
                inst.get_parent(),
                Some(block),
                "IRMarker discovered broken Block -> Inst relationship."
            );
            Self::do_push_mark(&mut proxy, ip);
            true
        });
        Self::do_push_mark(&mut proxy, body.preds.sentinel);
        Self::do_push_mark(&mut proxy, body.users.sentinel);
    }
    fn consume_inst(&mut self, inst: InstID) {
        let mut proxy = self.borrow_proxy();
        let allocs = proxy.allocs;
        let inst = inst.deref_ir(allocs);
        for op in inst.get_operands() {
            Self::do_push_mark(&mut proxy, op);
        }
        if let Some(jts) = inst.try_get_jts() {
            for &jt in jts.iter() {
                Self::do_push_mark(&mut proxy, jt);
            }
        }
        if let Some(user) = inst.try_get_users() {
            Self::do_push_mark(&mut proxy, user.sentinel);
        }
    }
    fn consume_expr(&mut self, expr: ExprID) {
        let mut proxy = self.borrow_proxy();
        let allocs = proxy.allocs;
        let expr = expr.deref_ir(allocs);
        for op in expr.get_operands() {
            Self::do_push_mark(&mut proxy, op);
        }
        if let Some(user) = expr.try_get_users() {
            Self::do_push_mark(&mut proxy, user.sentinel);
        }
    }
    fn consume_global(&mut self, global_id: GlobalID) {
        let mut proxy = self.borrow_proxy();
        let allocs = proxy.allocs;
        let global = global_id.deref_ir(allocs);
        for op in global.get_operands() {
            Self::do_push_mark(&mut proxy, op);
        }
        if let Some(user) = global.try_get_users() {
            Self::do_push_mark(&mut proxy, user.sentinel);
        }

        let (args, body) = match global {
            GlobalObj::Var(_) => return,
            GlobalObj::Func(f) => (f.args.as_ref(), f.body.as_ref()),
        };

        let func_id = FuncID(global_id);
        for arg in args {
            Self::do_push_mark(&mut proxy, arg.users().sentinel);
        }
        if let Some(body) = body {
            assert_eq!(
                body.entry.get_parent_func(allocs),
                Some(func_id),
                "Function @{} entry NOT attached.",
                global.get_name()
            );
            body.blocks.forall_with_sentinel(&allocs.blocks, |bid, bb| {
                assert_eq!(
                    bb.get_parent_func(),
                    Some(func_id),
                    "IRMarker discovered broken GlobalFunc -> Block relationship."
                );
                Self::do_push_mark(&mut proxy, bid);
                true
            });
        }
    }
}

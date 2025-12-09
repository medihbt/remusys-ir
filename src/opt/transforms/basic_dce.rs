use crate::{
    ir::{
        BlockID, ExprID, FuncID, IRAllocs, IRBuilder, ISubExprID, ISubInstID, IUser, InstID,
        InstObj, InstOrdering, Module, PoolAllocatedDisposeRes, ValueSSA,
        checking::FuncDominanceCheck,
    },
    opt::{CfgBlockStat, CfgDfsSeq, transforms::IFuncTransformPass},
};
use std::{
    collections::{HashSet, VecDeque},
    sync::{Arc, OnceLock},
};

pub struct BasicFuncDCE<'ir> {
    pub builder: IRBuilder<&'ir Module>,
    pub dead_inst: Vec<(BlockID, InstID)>,
    pub dead_block: Vec<BlockID>,
}

static BASIC_DCE_NAME: OnceLock<Arc<str>> = OnceLock::new();

impl<'ir> Drop for BasicFuncDCE<'ir> {
    fn drop(&mut self) {
        self.dispose().expect("BasicFuncDCE: failed to dispose");
    }
}
impl<'ir> IFuncTransformPass for BasicFuncDCE<'ir> {
    fn get_name(&self) -> Arc<str> {
        BASIC_DCE_NAME
            .get_or_init(|| Arc::from("BasicFuncDCE"))
            .clone()
    }

    fn run_on_func(&mut self, order: &dyn InstOrdering, func: FuncID) {
        // Implementation of Basic Dead Code Elimination algorithm goes here
        let dfs = if cfg!(debug_assertions) {
            let allocs = self.builder.allocs();
            let func_check = FuncDominanceCheck::new(allocs, func);
            func_check.run().unwrap();
            func_check.dom_tree.dfs
        } else {
            let allocs = self.builder.allocs();
            CfgDfsSeq::new_pre(allocs, func).unwrap()
        };
        self.kill_blocks_and_analyze(&dfs, order, func);

        let allocs = self.builder.allocs();
        let mut live_marker = LiveInstMarker::new(allocs);
        for node in &dfs.nodes {
            let CfgBlockStat::Block(block) = node.block else {
                continue;
            };
            let insts = block.get_insts(allocs).iter(&allocs.insts);
            for (inst_id, inst) in insts {
                if self.inst_may_have_side_effects(inst) {
                    live_marker.push_mark(inst_id);
                }
                self.dead_inst.push((block, inst_id));
            }
        }
        live_marker.mark_all();
        self.dead_inst
            .retain(|(_, inst)| !live_marker.live_insts.contains(inst));
        self.dead_inst.shrink_to_fit();
        for &(block, inst) in &self.dead_inst {
            let insts = block.get_insts(allocs);
            // 卸接死指令，使得教具/工具可以在最终 dispose 之前观察到这些
            // 指令已从所属 block 的指令链中移除。真正的资源释放发生在
            // `cleanup()`：这里不做 dispose 以便上层工具接收删除信号。
            insts
                .node_unplug(inst, &allocs.insts)
                .expect("BasicFuncDCE: failed to unplug dead inst");
            // 通知指令排序器该指令已被移除
            order.on_inst_remove(block, inst);
        }
    }
}

impl<'ir> BasicFuncDCE<'ir> {
    pub fn new(module: &'ir Module) -> Self {
        Self {
            builder: IRBuilder::new(module),
            dead_inst: Vec::new(),
            dead_block: Vec::new(),
        }
    }

    pub fn dispose(&mut self) -> PoolAllocatedDisposeRes {
        let allocs = self.builder.allocs();
        for (_, inst) in self.dead_inst.drain(..) {
            inst.dispose(allocs)?;
        }
        for block in self.dead_block.drain(..) {
            block.dispose(allocs)?;
        }
        Ok(())
    }

    fn kill_blocks_and_analyze(&mut self, dfs: &CfgDfsSeq, order: &dyn InstOrdering, func: FuncID) {
        // Implementation for removing dead blocks
        let allocs = self.builder.allocs();
        let blocks = func.get_blocks(allocs).unwrap();
        let mut insts_cap = 0;
        for (block, _) in blocks.iter(&allocs.blocks) {
            if !dfs.block_reachable(block) {
                self.dead_block.push(block);
            } else {
                insts_cap += block.get_insts(allocs).len();
            }
        }
        for &block in &self.dead_block {
            // 从 blocks 中卸接不可达块；使用 `expect` 提供更明确的错误信息
            // 设计说明：这里卸接（unplug）不可达块是为了让后续的工具/教具
            // 能观察到块已从 CFG 中移除。最终的资源释放（dispose）由
            // `cleanup()` 执行；如果没有调用 `cleanup()`，GC 仍可回收这些对象。
            blocks
                .node_unplug(block, &allocs.blocks)
                .expect("BasicFuncDCE: failed to unplug dead block");
            order.invalidate_block(allocs, block);
        }
        self.dead_inst.reserve_exact(insts_cap);
    }

    /// 非常保守的副作用分析, 只要有一点点可能存在副作用, 那这个指令就算有副作用
    fn inst_may_have_side_effects(&self, inst: &InstObj) -> bool {
        use crate::ir::inst::InstObj::*;
        matches!(
            inst,
            GuideNode(_)
                | PhiInstEnd(_)
                | Unreachable(_)
                | Ret(_)
                | Jump(_)
                | Br(_)
                | Switch(_)
                | Store(_)
                | AmoRmw(_)
                | Call(_)
        )
    }
}

struct LiveInstMarker<'ir> {
    allocs: &'ir IRAllocs,
    live_insts: HashSet<InstID>,
    live_exprs: HashSet<ExprID>,
    mark_queue: VecDeque<MarkValue>,
}
enum MarkValue {
    Inst(InstID),
    Expr(ExprID),
}

impl<'ir> LiveInstMarker<'ir> {
    fn new(allocs: &'ir IRAllocs) -> Self {
        Self {
            allocs,
            live_insts: HashSet::new(),
            live_exprs: HashSet::new(),
            mark_queue: VecDeque::new(),
        }
    }

    fn push_mark(&mut self, inst: InstID) {
        if !self.live_insts.insert(inst) {
            return;
        }
        self.mark_queue.push_back(MarkValue::Inst(inst));
    }
    fn push_mark_expr(&mut self, expr: ExprID) {
        if !self.live_exprs.insert(expr) {
            return;
        }
        self.mark_queue.push_back(MarkValue::Expr(expr));
    }
    fn pop_mark(&mut self) -> Option<MarkValue> {
        self.mark_queue.pop_front()
    }

    fn mark_all(&mut self) {
        while let Some(mark) = self.pop_mark() {
            match mark {
                MarkValue::Inst(inst) => self.mark_inst(inst),
                MarkValue::Expr(expr) => self.mark_expr(expr),
            }
        }
    }
    fn mark_inst(&mut self, inst: InstID) {
        let allocs = self.allocs;
        for use_id in inst.get_operands(allocs) {
            match use_id.get_operand(allocs) {
                ValueSSA::ConstExpr(expr) => self.push_mark_expr(expr),
                ValueSSA::Inst(inst) => self.push_mark(inst),
                _ => {}
            }
        }
    }
    fn mark_expr(&mut self, expr: ExprID) {
        let allocs = self.allocs;
        for use_id in expr.deref_ir(allocs).get_operands() {
            match use_id.get_operand(allocs) {
                ValueSSA::ConstExpr(expr) => self.push_mark_expr(expr),
                ValueSSA::Inst(inst) => self.push_mark(inst),
                _ => {}
            }
        }
    }
}

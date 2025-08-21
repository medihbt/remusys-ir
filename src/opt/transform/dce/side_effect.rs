use crate::{
    base::SlabRef,
    ir::{
        FuncRef, GlobalRef, IRAllocs, IReferenceValue, ISubInst, IUser, InstData, InstRef, Module,
        UseKind, ValueSSA,
    },
};
use std::collections::{BTreeSet, HashMap, VecDeque};

/// 保守的副作用分析器.
#[derive(Debug)]
pub(super) struct SideEffectMarker {
    pub roots: BTreeSet<GlobalRef>,
    pub insts: BTreeSet<InstRef>,
    pub queue: VecDeque<InstRef>,
}

impl SideEffectMarker {
    /// 扫描所有带副作用的全局量.
    pub fn new_full(globals: &HashMap<String, GlobalRef>, allocs: &IRAllocs) -> Self {
        let mut ret = Self {
            roots: BTreeSet::new(),
            insts: BTreeSet::new(),
            queue: VecDeque::new(),
        };
        ret.init_roots(allocs, globals);
        log::debug!("before mark_all: {ret:#?}");
        ret.mark_all(allocs);
        log::debug!("after mark_all: {ret:#?}");
        ret
    }

    pub fn from_module(module: &Module) -> Self {
        let globals = module.globals.borrow();
        let allocs = module.borrow_allocs();
        Self::new_full(&globals, &allocs)
    }

    fn init_roots(&mut self, allocs: &IRAllocs, globals: &HashMap<String, GlobalRef>) {
        // 所有全局量都是根节点（保守策略）
        for (_, &g) in globals {
            self.roots.insert(g);
        }

        // 扫描所有函数中的副作用指令
        for (_, &g) in globals {
            let Some(func) = FuncRef::try_from_real(g, &allocs.globals) else {
                continue;
            };
            self.init_insts_root(allocs, func);
        }
    }

    fn init_insts_root(&mut self, allocs: &IRAllocs, func: FuncRef) {
        let Some(body) = func.try_get_body(&allocs.globals) else {
            return;
        };
        for (_, block) in body.view(&allocs.blocks) {
            for (iref, inst) in block.insts.view(&allocs.insts) {
                if self.inst_may_have_side_effect(inst, allocs) {
                    self.push_mark(iref);
                }
            }
        }
    }
    fn inst_may_have_side_effect(&self, inst: &InstData, allocs: &IRAllocs) -> bool {
        match inst {
            // 结构性指令：删除会破坏IR完整性
            InstData::ListGuideNode(_)
            | InstData::PhiInstEnd(_)
            | InstData::Unreachable(_)
            | InstData::Ret(_)
            | InstData::Jump(_)
            | InstData::Br(_)
            | InstData::Switch(_) => true,

            // 函数调用：保守策略，所有调用都认为有副作用
            InstData::Call(_) => true,

            // 存储指令：精确分析局部vs全局
            InstData::Store(str) => {
                let ValueSSA::Inst(target) = str.get_target() else { return true };
                let alloca_pure = {
                    let InstData::Alloca(alloca) = target.to_data(&allocs.insts) else {
                        return true;
                    };
                    alloca
                        .get_common()
                        .users
                        .iter()
                        .all(|u| u.kind.get() == UseKind::StoreTarget)
                };
                !alloca_pure
            }

            // 其他指令：纯计算，可以安全删除
            _ => false,
        }
    }

    fn push_mark(&mut self, inst: InstRef) -> bool {
        if self.insts.contains(&inst) {
            return false;
        }
        self.insts.insert(inst);
        self.queue.push_back(inst);
        true
    }

    fn mark_all(&mut self, allocs: &IRAllocs) {
        while let Some(inst) = self.queue.pop_front() {
            for operand in inst.to_value_data(allocs).operands_iter() {
                let ValueSSA::Inst(inst) = operand else {
                    continue;
                };
                self.push_mark(inst);
            }
        }
    }

    pub fn inst_has_side_effect(&self, inst: InstRef) -> bool {
        self.insts.contains(&inst)
    }
}

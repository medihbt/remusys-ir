use crate::{
    ir::{
        BlockID, ExprID, FuncID, IRAllocs, ISubExprID, ISubGlobalID, ISubInstID, ISubValueSSA,
        ITraceableValue, InstID, InstObj, InstOrderCache, Module, Use, UseKind, UserID, ValueSSA,
    },
    opt::{CfgBlockStat, DominatorTree},
};
use std::{collections::HashSet, fmt::Debug};

#[derive(thiserror::Error)]
pub enum DominanceCheckErr {
    #[error("Operand {operand:?} does not dominate its user {user:?}")]
    NotDominated { operand: ValueSSA, user: UserID },
}
pub type DominanceCheckRes<T = ()> = Result<T, DominanceCheckErr>;

impl Debug for DominanceCheckErr {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        std::fmt::Display::fmt(self, f)
    }
}

pub fn assert_module_dominance(module: &Module) {
    let allocs = &module.allocs;
    let symtab = module.symbols.borrow();
    for &func_id in symtab.func_pool() {
        if func_id.is_extern(allocs) {
            continue;
        }
        assert_func_dominance(allocs, func_id);
    }
}
pub fn assert_func_dominance(allocs: &IRAllocs, func: FuncID) {
    FuncDominanceCheck::new(allocs, func).run().unwrap()
}
pub fn module_dominance_check(module: &Module) -> DominanceCheckRes {
    let allocs = &module.allocs;
    let symtab = module.symbols.borrow();
    for &func_id in symtab.func_pool() {
        if func_id.is_extern(allocs) {
            continue;
        }
        FuncDominanceCheck::new(allocs, func_id).run()?;
    }
    Ok(())
}

pub struct FuncDominanceCheck<'ir> {
    pub func_id: FuncID,
    pub dom_tree: DominatorTree<InstOrderCache>,
    pub allocs: &'ir IRAllocs,
}

impl<'ir> FuncDominanceCheck<'ir> {
    pub fn new(allocs: &'ir IRAllocs, func_id: FuncID) -> Self {
        let inst_ord = InstOrderCache::new();
        let dom_tree = DominatorTree::builder(allocs, func_id)
            .build(allocs)
            .map_relation(inst_ord);
        Self { func_id, dom_tree, allocs }
    }

    pub fn run(&self) -> DominanceCheckRes {
        let dt = &self.dom_tree;
        let allocs = self.allocs;
        for dfs_node in &dt.dfs.nodes {
            let CfgBlockStat::Block(block) = dfs_node.block else {
                continue;
            };
            for (inst_id, iobj) in block.insts_iter(allocs) {
                match iobj {
                    InstObj::GuideNode(_) | InstObj::PhiInstEnd(_) => continue,
                    _ => CheckStat::new(self, inst_id).operand_dominates_all_uses()?,
                }
            }
        }

        Ok(())
    }
}

struct CheckStat<'ir> {
    processed: HashSet<ExprID>,
    expr_stack: Vec<ExprID>,
    allocs: &'ir IRAllocs,
    operand: InstID,
    dt: &'ir DominatorTree<InstOrderCache>,
}

impl<'ir> CheckStat<'ir> {
    fn new(fcheck: &'ir FuncDominanceCheck<'ir>, operand: InstID) -> Self {
        Self {
            processed: HashSet::new(),
            expr_stack: Vec::new(),
            allocs: fcheck.allocs,
            dt: &fcheck.dom_tree,
            operand,
        }
    }

    pub fn operand_dominates_all_uses(&mut self) -> DominanceCheckRes {
        let allocs = self.allocs;
        let operand = self.operand;
        for (_, uobj) in operand.deref_ir(allocs).user_iter(allocs) {
            self.dispatch_user(uobj)?;
            while let Some(expr_id) = self.expr_stack.pop() {
                if !self.processed.insert(expr_id) {
                    continue;
                }
                for (_, uobj) in expr_id.deref_ir(allocs).user_iter(allocs) {
                    self.dispatch_user(uobj)?;
                }
            }
        }
        Ok(())
    }
    fn dispatch_user(&mut self, user_use: &Use) -> DominanceCheckRes {
        let user = user_use
            .user
            .get()
            .expect("Internal error: Discovered a use without user");
        let inst_id = match user {
            // Remusys-IR 允许指令作为表达式的一部分, 用于简化一部分场景下的值传递.
            // 当然这部分在做 LLVM 兼容或者向下转换时需要被拆开.
            UserID::Expr(expr_id) => {
                self.expr_stack.push(expr_id);
                return Ok(());
            }
            UserID::Global(_) => return Ok(()),
            UserID::Inst(inst_id) => inst_id,
        };
        let allocs = self.allocs;
        let dt = self.dt;
        let operand = self.operand;

        let dominates = match user_use.get_kind() {
            UseKind::PhiIncomingValue(group_idx) => {
                let InstObj::Phi(phi) = inst_id.deref_ir(allocs) else {
                    panic!("Internal error: Phi incoming value use found on non-phi inst");
                };
                let [_, ublk] = phi.incoming_uses()[group_idx as usize];
                let income_bb = BlockID::from_ir(ublk.get_operand(allocs));
                dt.inst_dominates_block(allocs, operand, income_bb)
            }
            ukind if ukind.is_inst_operand() => dt.inst_dominates_inst(allocs, operand, inst_id),
            ukind => panic!("Internal error: Unsupported use kind in dominance check: {ukind:?}"),
        };
        if dominates {
            Ok(())
        } else {
            Err(DominanceCheckErr::NotDominated { operand: operand.into_ir(), user })
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::cases::test_case_cfg_deep_while_br;

    #[test]
    fn test_dominance_check() {
        let module = test_case_cfg_deep_while_br().module;
        assert_module_dominance(&module);
    }
}

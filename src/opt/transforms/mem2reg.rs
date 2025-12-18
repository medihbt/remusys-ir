use crate::{
    base::FixBitSet,
    ir::{
        BlockID, ConstData, FuncID, IRBuilder, IRFocus, ISubInstID, ITraceableValue, IValueConvert,
        InstID, InstObj, InstOrdering, Module, UserID, ValueSSA,
        inst::{AllocaInst, AllocaInstID, LoadInstID, PhiInstID, StoreInstID},
    },
    opt::{CfgBlockStat, DominanceFrontier, DominatorTree, IFuncTransformPass},
    typing::{IValType, ScalarType, ValTypeID},
};
use smallvec::SmallVec;
use std::{
    collections::{HashMap, HashSet},
    sync::Arc,
};

type DT<'a> = DominatorTree<&'a dyn InstOrdering>;
type DF<'a> = DominanceFrontier<'a, &'a dyn InstOrdering>;

pub struct Mem2Reg<'ir> {
    pub module: &'ir Module,
}

impl<'ir> IFuncTransformPass for Mem2Reg<'ir> {
    fn get_name(&self) -> Arc<str> {
        Arc::from("Mem2Reg")
    }

    fn run_on_func(&mut self, order: &dyn InstOrdering, func: FuncID) {
        let allocas = self.dump_promotable_allocas(func);
        let allocs = &self.module.allocs;
        let dt: DominatorTree<&dyn InstOrdering> = DominatorTree::builder(allocs, func)
            .expect("Dominance building error in Mem2Reg")
            .build()
            .map_relation(order);
        let df = DominanceFrontier::new(&dt, allocs).unwrap();
        for alloca in &allocas {
            self.promote_one_alloca(&df, alloca);
        }
    }
}

#[derive(Debug)]
struct PromoteInfo {
    alloca: AllocaInstID,
    valty: ValTypeID,
    loads: SmallVec<[LoadInstID; 4]>,
    stores: SmallVec<[StoreInstID; 4]>,
}

impl<'ir> Mem2Reg<'ir> {
    pub fn new(module: &'ir Module) -> Self {
        Self { module }
    }

    fn dump_promotable_allocas(&self, func: FuncID) -> Vec<PromoteInfo> {
        let allocs = &self.module.allocs;
        let entry = func.entry_unwrap(allocs);
        let mut ret = Vec::new();
        for (inst_id, inst) in entry.insts_iter(allocs) {
            let InstObj::Alloca(alloca) = inst else {
                continue;
            };
            if let Some(promote_info) =
                self.alloca_as_promotable(alloca, AllocaInstID::raw_from(inst_id))
            {
                ret.push(promote_info);
            }
        }
        ret
    }

    fn alloca_as_promotable(&self, alloca: &AllocaInst, id: AllocaInstID) -> Option<PromoteInfo> {
        let Ok(scalty) = ScalarType::try_from_ir(alloca.pointee_ty) else {
            // 暂时不支持数组等复杂类型的提升
            return None;
        };
        let allocs = &self.module.allocs;
        let mut loads = SmallVec::new();
        let mut stores = SmallVec::new();
        for (_, user_use) in alloca.user_iter(allocs) {
            use crate::ir::UseKind;
            let Some(UserID::Inst(user)) = user_use.user.get() else {
                return None;
            };
            let promotable = match user_use.get_kind() {
                UseKind::LoadSource => {
                    let load = LoadInstID::try_from_instid(user, allocs)
                        .expect("IR invariant violated: UseKind::LoadSource attached to non-Load instruction");
                    loads.push(load);
                    load.get_rettype(allocs) == scalty.into_ir()
                }
                UseKind::StoreTarget => {
                    let store = StoreInstID::try_from_instid(user, allocs)
                        .expect("IR invariant violated: UseKind::StoreTarget attached to non-Store instruction");
                    stores.push(store);
                    store.source_ty(allocs) == scalty.into_ir()
                }
                _ => false,
            };
            if !promotable {
                return None;
            }
        }
        Some(PromoteInfo { alloca: id, valty: scalty.into_ir(), loads, stores })
    }

    fn promote_one_alloca(&self, df: &DF, info: &PromoteInfo) {
        if info.stores.is_empty() {
            return self.promote_nostore(df, info);
        }
        if info.stores.len() == 1 {
            return self.promote_single_store(df, info);
        }
        if let Some(local_bb) = self.info_as_local(info) {
            // alloca 只在一个基本块内使用，可以直接进行局部提升
            return self.promote_local(df, info, local_bb);
        }

        let cfg_dfn_phi = self.insert_phis(df, info);
        self.rename(df, info, cfg_dfn_phi);
    }

    fn promote_nostore(&self, df: &DF, info: &PromoteInfo) {
        // 如果没有 store，则说明 alloca 没有被写入过，直接将 load 全部替换为 undef 即可
        let allocs = &self.module.allocs;
        let mut builder = IRBuilder::new(self.module);
        let undef = ValueSSA::ConstData(ConstData::Undef(info.valty));
        let order = &df.dom_tree.inst_order;

        for &load in &info.loads {
            load.deref_ir(allocs)
                .replace_self_with(allocs, undef)
                .expect("Internal error: failed to replace load with undef");
            builder
                .remove_inst_with_order(load, order)
                .expect("Internal error: failed to remove load instruction");
        }
        builder
            .remove_inst_with_order(info.alloca, order)
            .expect("Internal error: failed to remove alloca instruction");
    }
    fn promote_single_store(&self, df: &DF, info: &PromoteInfo) {
        let allocs = &self.module.allocs;
        let store = info.stores[0];
        let stored_val = store.get_source(allocs);
        let mut builder = IRBuilder::new(self.module);
        let order = &df.dom_tree.inst_order;

        let mut remove_defs = true;
        let dt = df.dom_tree;
        for &load in &info.loads {
            let dominator = store.raw_into();
            let dominatee = load.raw_into();
            if !dt.inst_dominates_inst(allocs, dominator, dominatee) {
                remove_defs = false;
                continue;
            }
            load.deref_ir(allocs)
                .replace_self_with(allocs, stored_val)
                .expect("Internal error: failed to replace load with stored value");
            builder
                .remove_inst_with_order(load, order)
                .expect("Internal error: failed to remove load instruction");
        }
        if remove_defs {
            builder
                .remove_inst_with_order(store, order)
                .expect("Internal error: failed to remove store instruction");
            builder
                .remove_inst_with_order(info.alloca, order)
                .expect("Internal error: failed to remove alloca instruction");
        }
    }

    fn info_as_local(&self, info: &PromoteInfo) -> Option<BlockID> {
        let mut ret = None;
        let allocs = &self.module.allocs;
        for &store in &info.stores {
            let parent = store.get_parent(allocs).expect("store has not preant");
            match ret {
                None => ret = Some(parent),
                Some(b) if b != parent => return None,
                _ => { /* continue */ }
            }
        }
        for &load in &info.loads {
            let parent = load.get_parent(allocs).expect("load has not preant");
            match ret {
                None => ret = Some(parent),
                Some(b) if b != parent => return None,
                _ => { /* continue */ }
            }
        }
        ret
    }
    fn promote_local(&self, df: &DF, info: &PromoteInfo, local_bb: BlockID) {
        let mut value = ValueSSA::ConstData(ConstData::Undef(info.valty));
        let allocs = &self.module.allocs;
        let stores = Self::dump_insts(&info.stores);
        let loads = Self::dump_insts(&info.loads);
        for (inst_id, inst) in local_bb.insts_iter(allocs) {
            match inst {
                InstObj::Store(store) => {
                    if stores.contains(&inst_id) {
                        value = store.get_source(allocs)
                    }
                }
                InstObj::Load(load) => {
                    if loads.contains(&inst_id) {
                        load.replace_self_with(allocs, value)
                            .expect("local info replacement failed")
                    }
                }
                _ => continue,
            }
        }
        let mut builder = IRBuilder::new(self.module);
        for &load in &info.loads {
            builder
                .remove_inst_with_order(load, df.dom_tree.inst_order)
                .expect("Internal error: failed to remove load instruction");
        }
    }
    fn dump_insts(insts: &[impl ISubInstID]) -> HashSet<InstID> {
        HashSet::from_iter(insts.iter().copied().map(ISubInstID::raw_into))
    }

    #[inline(never)]
    fn insert_phis(&self, df: &DF, info: &PromoteInfo) -> HashMap<usize, PhiInstID> {
        let def_dfns: FixBitSet = self.dump_def_cfgdfns(df, info);
        let mut phi_blocks = HashSet::new();
        let mut dfn_worklist: SmallVec<[usize; 16]> = def_dfns.iter().collect();

        while let Some(cfg_dfn) = dfn_worklist.pop() {
            for &df_dfn in &df.df[cfg_dfn] {
                if phi_blocks.insert(df_dfn) {
                    dfn_worklist.push(df_dfn);
                }
            }
        }
        let mut builder = IRBuilder::new(self.module);
        let mut ret = HashMap::new();
        let allocs = &self.module.allocs;
        for &dfn in &phi_blocks {
            let CfgBlockStat::Block(block) = df.dom_tree.dfs.nodes[dfn].block else {
                continue;
            };
            builder.set_focus(IRFocus::Block(block));
            let phi = PhiInstID::new_empty(allocs, info.valty);
            builder
                .insert_inst_with_order(phi, df.dom_tree.inst_order)
                .expect("Failed to insert phi");
            ret.insert(dfn, phi);
        }
        ret
    }
    fn dump_def_cfgdfns(&self, df: &DF<'_>, info: &PromoteInfo) -> FixBitSet {
        let allocs = &self.module.allocs;
        let dfs = &df.dom_tree.dfs;
        let mut set = FixBitSet::with_len(dfs.nodes.len());
        for &store in &info.stores {
            let parent = store
                .get_parent(allocs)
                .expect("IR invariant violated: store has no parent block");
            if let Some(&dfn) = dfs.unseq.get(&parent) {
                set.enable(dfn);
            }
        }
        set
    }

    #[inline(never)]
    fn rename(&self, df: &DF<'_>, info: &PromoteInfo, phis: HashMap<usize, PhiInstID>) {
        let mut renamer = Rename::new(self, phis, df, info);
        renamer.run();
        renamer.cleanup();
    }
}

struct Rename<'t> {
    builder: IRBuilder<&'t Module>,
    dfn_phi: HashMap<usize, PhiInstID>,
    defuse: HashSet<InstID>,
    stack: SmallVec<[ValueSSA; 16]>,
    df: &'t DF<'t>,
    dt: &'t DT<'t>,
    order: &'t dyn InstOrdering,
    info: &'t PromoteInfo,
}

impl<'t> Rename<'t> {
    fn new(
        mem2reg: &Mem2Reg<'t>,
        dfn_phi: HashMap<usize, PhiInstID>,
        df: &'t DF<'t>,
        info: &'t PromoteInfo,
    ) -> Self {
        let mut defuse = HashSet::new();
        if info.loads.len() >= 8 {
            defuse.extend(info.loads.iter().copied().map(ISubInstID::raw_into));
        }
        if info.stores.len() >= 8 {
            defuse.extend(info.stores.iter().copied().map(ISubInstID::raw_into));
        }
        Self {
            builder: IRBuilder::new(mem2reg.module),
            dfn_phi,
            defuse,
            stack: SmallVec::new(),
            df,
            dt: df.dom_tree,
            order: df.dom_tree.inst_order,
            info,
        }
    }
    fn push_value(&mut self, val: impl IValueConvert) {
        self.stack.push(val.into_value());
    }

    fn run(&mut self) {
        let undef = ValueSSA::ConstData(ConstData::Undef(self.info.valty));
        self.stack.push(undef);
        self.rename_one(DT::ROOT_INDEX);
    }

    fn rename_one(&mut self, cfg_dfn: usize) {
        let CfgBlockStat::Block(block) = self.dt.dfs.nodes[cfg_dfn].block else {
            return;
        };
        let stack_len = self.stack.len();
        if let Some(&phi) = self.dfn_phi.get(&cfg_dfn) {
            self.push_value(phi);
        }
        let allocs = {
            let module = self.builder.module;
            &module.allocs
        };

        for (inst_id, inst) in block.insts_iter(allocs) {
            let removed = match inst {
                InstObj::Store(store) if self.has_store(inst_id) => {
                    self.push_value(store.get_source(allocs));
                    true
                }
                InstObj::Load(load) if self.has_load(inst_id) => {
                    let Some(&val) = self.stack.last() else {
                        panic!("Internal error: stack underflow when renaming load");
                    };
                    load.replace_self_with(allocs, val)
                        .expect("Internal error: failed to replace load with renamed value");
                    true
                }
                _ => false,
            };
            if removed {
                self.builder
                    .remove_inst_with_order(inst_id, self.order)
                    .expect("Internal error: failed to remove load/store instruction");
            }
        }

        let succs = self.df.cfg.succ_of(block).unwrap_or(&[]);
        for &succ in succs {
            if let Some(phi) = self.block_get_phi(succ) {
                let Some(&val) = self.stack.last() else {
                    panic!("Internal error: stack underflow when renaming phi operand");
                };
                phi.set_incoming(allocs, block, val);
            }
        }

        for &child_dfn in &self.dt.nodes[cfg_dfn].children_dfn {
            self.rename_one(child_dfn);
        }
        self.stack.truncate(stack_len);
    }

    fn has_load(&mut self, load: InstID) -> bool {
        let loads = &self.info.loads;
        if loads.len() < 8 {
            loads.contains(&LoadInstID::raw_from(load))
        } else {
            self.defuse.contains(&load)
        }
    }
    fn has_store(&mut self, store: InstID) -> bool {
        let stores = &self.info.stores;
        if stores.len() < 8 {
            stores.contains(&StoreInstID::raw_from(store))
        } else {
            self.defuse.contains(&store)
        }
    }
    fn block_get_phi(&self, block: BlockID) -> Option<PhiInstID> {
        let dfn = self.dt.dfs.unseq.get(&block)?;
        self.dfn_phi.get(dfn).copied()
    }

    fn cleanup(&mut self) {
        let allocs = &self.builder.module.allocs;
        // dedup phi operands where possible
        for phi in self.dfn_phi.values() {
            let mut dedup = phi.begin_dedup(allocs);
            if !dedup.initial_nodup() {
                let ok = dedup.dedup_same_operand();
                if !ok {
                    dedup.keep_first();
                }
                dedup.apply().expect("Phi dedup failed");
            }
        }

        // finally remove the original alloca
        self.builder
            .remove_inst_with_order(self.info.alloca, self.order)
            .expect("Internal error: failed to remove alloca instruction");
        self.info.alloca.dispose(allocs).unwrap();
        for &def in &self.info.stores {
            def.dispose(allocs).unwrap();
        }
        for &load in &self.info.loads {
            load.dispose(allocs).unwrap();
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{ir::*, testing::cases::*};

    #[test]
    fn test_mem2reg() {
        let module = test_case_cfg_deep_while_br().module;
        write_ir_to_file(
            "../target/test-mem2reg-before.ll",
            &module,
            IRWriteOption::quiet(),
        );

        let main_func = module
            .get_global_by_name("main")
            .map(FuncID::raw_from)
            .expect("test case has no main function");

        let orders = InstOrderCache::new();
        Mem2Reg::new(&module).run_on_func(&orders, main_func);
        write_ir_to_file(
            "../target/test-mem2reg-after.ll",
            &module,
            IRWriteOption::quiet(),
        );
    }
}

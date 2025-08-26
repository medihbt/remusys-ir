use crate::{
    base::SlabRef,
    ir::{FuncRef, IRAllocs, IRValueMarker, ISubGlobal, Linkage, Module},
    opt::{
        analysis::cfg::{dfs::CfgDfsSeq, snapshot::CfgSnapshot},
        util::DfsOrder,
    },
};
use std::ops::ControlFlow;

pub(super) fn retain_globals(module: &mut Module) {
    let mut marker = IRValueMarker::from_allocs(&mut module.allocs);
    let mut to_erase = Vec::new();
    for (_, &global) in module.globals.borrow().iter() {
        let linkage = global.to_data(&marker.allocs.globals).get_linkage();
        if linkage == Linkage::DSOLocal {
            marker.push_mark(global);
        } else {
            to_erase.push(global);
        }
    }
    marker.mark_all();

    for g in to_erase {
        if marker.live_set.is_live(g) {
            continue;
        }
        let gdata = g.to_data(&marker.allocs.globals);
        module.globals.borrow_mut().remove(gdata.get_name());
    }

    marker.sweep();
}

fn reatin_cfg_for_func(allocs: &IRAllocs, func: FuncRef) {
    let cfg = CfgSnapshot::new(allocs, func.0);
    let dfs = CfgDfsSeq::new_from_snapshot(&cfg, DfsOrder::Pre);
    for node in &cfg.nodes {
        let bref = node.block;
        if dfs.block_is_reachable(bref) {
            continue;
        }
        dead_user::kill_block(allocs, bref, &dfs);
        func.get_body_from_alloc(&allocs.globals)
            .unplug_node(&allocs.blocks, bref)
            .unwrap_or_else(|e| {
                let name = func.to_data(&allocs.globals).get_name();
                panic!("Err {e:?}: failed to unplug dead block from func {name}")
            });
    }
}

pub(super) fn retain_cfg_for_module(module: &mut Module) {
    let allocs = &module.allocs;
    module.forall_funcs(false, |fref, _| {
        reatin_cfg_for_func(&allocs, fref);
        ControlFlow::Continue(())
    });
    module.gc_mark_sweep([]);
}

mod dead_user {
    use crate::{
        ir::{
            BlockRef, IRAllocs, IReferenceValue, ISubInstRef, ITraceableValue, InstRef, Use,
            UseKind, UserID, ValueSSA, inst::PhiRef,
        },
        opt::analysis::cfg::dfs::CfgDfsSeq,
    };
    use std::rc::Rc;

    pub(super) fn kill_block(allocs: &IRAllocs, bref: BlockRef, dfs: &CfgDfsSeq) {
        debug_assert!(!dfs.block_is_reachable(bref));
        for (iref, _) in bref.view_insts(allocs) {
            check_inst(allocs, iref, dfs);
        }
        let mut dead_users = Vec::new();
        for user in bref.users(allocs) {
            let res = check_dead_block_user(allocs, &user, dfs).unwrap();
            let UseKind::PhiIncomingBlock(idx) = res else {
                continue;
            };
            let UserID::Inst(inst) = user.user.get() else { unreachable!("Broken IR") };
            dead_users.push((PhiRef(inst), idx));
        }
        for (phi, idx) in dead_users {
            phi.to_inst(&allocs.insts).remove_income_index(idx as usize);
        }
    }

    type UseCheckRes = Result<UseKind, &'static str>;
    fn check_dead_block_user(allocs: &IRAllocs, u: &Use, dfs: &CfgDfsSeq) -> UseCheckRes {
        if matches!(u.kind.get(), UseKind::PhiIncomingBlock(_)) {
            return Ok(u.kind.get());
        }
        let UserID::Inst(inst) = u.user.get() else {
            return Err("Remusys block user can only be an instruction now");
        };
        if !dfs.block_is_reachable(inst.get_parent(allocs)) {
            return Ok(UseKind::GuideNode);
        }
        Err("Unreachable block used by reachable value, aborted")
    }

    fn check_inst(allocs: &IRAllocs, inst: InstRef, dfs: &CfgDfsSeq) {
        for u in inst.to_value_data(allocs).users() {
            check_dead_inst_user(allocs, &u, dfs);
        }
    }

    fn check_dead_inst_user(allocs: &IRAllocs, u: &Rc<Use>, dfs: &CfgDfsSeq) {
        let UserID::Inst(user) = u.user.get() else {
            panic!("Inst user can only be inst");
        };
        if !dfs.block_is_reachable(user.get_parent(allocs)) {
            return;
        }
        if let UseKind::PhiIncomingValue(idx) = u.kind.get() {
            let phi = PhiRef(user).to_inst(&allocs.insts);
            let incoming = phi.incoming_uses();
            let [_, blk] = &incoming[idx as usize];
            let ValueSSA::Block(income_bb) = blk.get_operand() else { unreachable!("Broken IR") };
            assert!(
                dfs.block_is_reachable(income_bb),
                "inst does not dominate its users"
            );
        } else {
            panic!("inst does not dominate its user");
        }
    }
}

//! Dead Code Elimination

use crate::{
    base::SlabRef,
    ir::{IRValueMarker, ISubGlobal, Linkage, Module},
};

mod dce_merge;

fn remove_unused_globals(module: &Module) {
    let mut allocs = module.borrow_allocs_mut();
    let mut marker = IRValueMarker::from_allocs(&mut allocs);
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

pub fn dce_pass(module: &Module) {
    remove_unused_globals(module);
    dce_merge::merge_exprs(module);
}

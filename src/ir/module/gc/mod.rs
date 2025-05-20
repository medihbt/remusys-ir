//! Garbage collection in Remusys-IR.

use liveset::IRRefLiveSet;

use crate::ir::ValueSSA;

use super::{Module, ModuleError};

pub mod compact;
pub mod liveset;
pub mod mark;
pub mod redirect;
pub mod sweep;

pub(super) fn module_gc_mark_sweep(
    module: &Module,
    extern_roots: impl Iterator<Item = ValueSSA>,
) -> Result<(), ModuleError> {
    let marker = mark::MarkVisitor::from_module(module, false);
    marker.mark_module().unwrap();
    let mut live_set = marker.release_live_set();
    for root in extern_roots {
        if live_set.value_is_live(root).unwrap() {
            continue;
        }
        live_set.mark_value_live(root).unwrap();
    }
    sweep::sweep_module(module, &live_set).unwrap();
    Ok(())
}

pub(super) fn module_gc_mark_compact(
    module: &Module,
    extern_roots: impl Iterator<Item = ValueSSA>,
) -> Result<IRRefLiveSet, ModuleError> {
    let (has_rcfg, has_rdfg) = {
        let has_rcfg = module.rcfg_enabled();
        let has_rdfg = module.rdfg_enabled();
        if has_rcfg {
            module.disable_rcfg();
        }
        if has_rdfg {
            module.disable_rdfg();
        }
        (has_rcfg, has_rdfg)
    };

    let marker = mark::MarkVisitor::from_module(module, true);
    marker.mark_module().unwrap();
    for root in extern_roots {
        if marker.value_is_live(root) {
            continue;
        }
        marker.mark_value(root).unwrap();
    }

    let redirector = redirect::Redirector::from_marker(marker);
    redirector.redirect_module().unwrap();

    let mut compactor = compact::CompactAlloc::from_redirector(&redirector);
    compactor.compact_generate_allocs();

    if has_rcfg {
        module.enable_rcfg().unwrap();
    }
    if has_rdfg {
        module.enable_rdfg().unwrap();
    }
    drop(compactor);

    Ok(redirector.live_set)
}

//! Garbage collection in Remusys-IR.

use crate::ir::ValueSSA;

use super::{Module, ModuleError};

pub mod redirect;
pub mod liveset;
pub mod mark;
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

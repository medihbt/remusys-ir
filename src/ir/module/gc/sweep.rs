use core::alloc;

use slab::Slab;

use crate::{
    base::slabref::SlabRef,
    ir::{
        block::{jump_target::{JumpTargetData, JumpTargetRef}, BlockData, BlockRef}, global::{GlobalData, GlobalRef}, inst::{
            usedef::{UseData, UseRef}, InstData, InstRef
        }, module::{Module, ModuleError}, ValueSSA
    },
};

use super::liveset::IRRefLiveSet;

pub(super) fn sweep_module(module: &Module, live_set: &IRRefLiveSet) -> Result<(), ModuleError> {
    let mut alloc_value = module.borrow_value_alloc_mut();
    let mut alloc_use = module.borrow_use_alloc_mut();
    let mut alloc_jt = module.borrow_jt_alloc_mut();
    sweep_global(&mut alloc_value.alloc_global, live_set);
    sweep_block(&mut alloc_value.alloc_block, live_set);
    sweep_inst(&mut alloc_value.alloc_inst, live_set);
    sweep_uses(&mut alloc_use, live_set);
    sweep_jt(&mut alloc_jt, live_set);
    Ok(())
}

fn sweep_global(alloc_global: &mut Slab<GlobalData>, live_set: &IRRefLiveSet) {
    alloc_global.retain(|handle, _| {
        let global_ref = GlobalRef::from_handle(handle);
        live_set
            .value_is_live(ValueSSA::Global(global_ref))
            .unwrap()
    });
}
fn sweep_block(alloc_block: &mut Slab<BlockData>, live_set: &IRRefLiveSet) {
    alloc_block.retain(|handle, _| {
        let block_ref = BlockRef::from_handle(handle);
        live_set.value_is_live(ValueSSA::Block(block_ref)).unwrap()
    });
}
fn sweep_inst(alloc_inst: &mut Slab<InstData>, live_set: &IRRefLiveSet) {
    alloc_inst.retain(|handle, _| {
        let inst_ref = InstRef::from_handle(handle);
        live_set.value_is_live(ValueSSA::Inst(inst_ref)).unwrap()
    });
}
fn sweep_uses(alloc_use: &mut Slab<UseData>, live_set: &IRRefLiveSet) {
    alloc_use.retain(|handle, _| {
        let use_ref = UseRef::from_handle(handle);
        live_set.use_is_live(use_ref).unwrap()
    });
}
fn sweep_jt(alloc_jt: &mut Slab<JumpTargetData>, live_set: &IRRefLiveSet) {
    alloc_jt.retain(|handle, _| {
        let jt_ref = JumpTargetRef::from_handle(handle);
        live_set.jt_is_live(jt_ref).unwrap()
    });
}

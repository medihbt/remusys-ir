use slab::Slab;

use crate::{
    base::SlabRef,
    ir::{
        ValueSSA,
        block::jump_target::{JumpTargetData, JumpTargetRef},
        inst::{UseData, UseRef},
        module::{Module, ModuleAllocatorInner},
    },
};

use super::redirect::Redirector;

pub(super) struct CompactAlloc<'a> {
    pub(super) redirector: &'a Redirector<'a>,
}

impl<'a> CompactAlloc<'a> {
    pub(super) fn from_redirector(redirector: &'a Redirector<'a>) -> Self {
        Self { redirector }
    }

    fn get_module(&self) -> &Module {
        self.redirector.module
    }

    pub(super) fn compact_generate_allocs(&mut self) {
        let module = self.get_module();
        let mut old_alloc_value = module.borrow_value_alloc_mut();
        let new_alloc_expr =
            self.build_value_alloc(&mut old_alloc_value.alloc_expr, ValueSSA::ConstExpr);
        let new_alloc_global =
            self.build_value_alloc(&mut old_alloc_value.alloc_global, ValueSSA::Global);
        let new_alloc_inst =
            self.build_value_alloc(&mut old_alloc_value.alloc_inst, ValueSSA::Inst);
        let new_alloc_block =
            self.build_value_alloc(&mut old_alloc_value.alloc_block, ValueSSA::Block);

        *old_alloc_value = ModuleAllocatorInner {
            alloc_expr: new_alloc_expr,
            alloc_global: new_alloc_global,
            alloc_inst: new_alloc_inst,
            alloc_block: new_alloc_block,
        };
        drop(old_alloc_value);

        let mut old_alloc_use = module.borrow_use_alloc_mut();
        *old_alloc_use = self.build_use_alloc(&mut old_alloc_use);
        drop(old_alloc_use);

        let mut old_alloc_jt = module.borrow_jt_alloc_mut();
        *old_alloc_jt = self.build_jt_alloc(&mut old_alloc_jt);
        drop(old_alloc_jt);
    }

    fn unplug_live_item<T>(
        alloc: &mut Slab<T>,
        mut judge_live: impl FnMut(usize, &T) -> bool,
    ) -> Vec<(usize, T)> {
        let mut result = Vec::with_capacity(alloc.len());
        let mut live_indices = Vec::with_capacity(alloc.len());
        for (index, item) in alloc.iter() {
            if judge_live(index, item) {
                live_indices.push(index);
            }
        }
        for index in live_indices {
            let item = alloc.remove(index);
            result.push((index, item));
        }
        result
    }
    fn build_value_alloc<T: SlabRef>(
        &self,
        alloc_value: &mut Slab<<T as SlabRef>::RefObject>,
        make_valuessa: impl Fn(T) -> ValueSSA,
    ) -> Slab<<T as SlabRef>::RefObject> {
        let live_set = &self.redirector.live_set;
        let mut values = Self::unplug_live_item(alloc_value, |index, _| {
            live_set
                .value_is_live(make_valuessa(T::from_handle(index)))
                .unwrap()
        });
        for (index, _) in values.iter_mut() {
            *index = live_set
                .get_value_new_pos(make_valuessa(T::from_handle(*index)))
                .unwrap();
        }
        Slab::from_iter(values.drain(..))
    }
    fn build_use_alloc(&self, alloc_use: &mut Slab<UseData>) -> Slab<UseData> {
        let live_set = &self.redirector.live_set;
        let mut uses = Self::unplug_live_item(alloc_use, |index, _| {
            live_set.use_is_live(UseRef::from_handle(index)).unwrap()
        });
        for (index, _) in uses.iter_mut() {
            *index = live_set
                .get_use_new_pos(UseRef::from_handle(*index))
                .unwrap();
        }
        Slab::from_iter(uses.drain(..))
    }
    fn build_jt_alloc(&self, alloc_jt: &mut Slab<JumpTargetData>) -> Slab<JumpTargetData> {
        let live_set = &self.redirector.live_set;
        let mut jts: Vec<(usize, JumpTargetData)> = Self::unplug_live_item(alloc_jt, |index, _| {
            live_set
                .jt_is_live(JumpTargetRef::from_handle(index))
                .unwrap()
        });
        for (index, _) in jts.iter_mut() {
            *index = live_set
                .get_jt_new_pos(JumpTargetRef::from_handle(*index))
                .unwrap();
        }
        Slab::from_iter(jts.drain(..))
    }
}

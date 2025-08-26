//! Dead Code Elimination

use std::collections::VecDeque;

use crate::{
    base::SlabRef,
    ir::{BlockData, BlockRef, FuncRef, IRAllocs, IUser, InstRef, Module, UserID},
    opt::transform::dce::side_effect::SideEffectMarker,
};

mod dce_merge;
mod dce_retain;
mod side_effect;

pub fn dce_pass(module: &mut Module) {
    dce_retain::retain_globals(module);
    dce_merge::merge_exprs(module);
    dce_retain::retain_cfg_for_module(module);
    DCEContext::new(module).exec(module);
}

struct DCEContext {
    side_effect: SideEffectMarker,
    inst_to_remove: VecDeque<InstRef>,
    inst_to_unplug: VecDeque<InstRef>,
    block_to_merge: VecDeque<(BlockMergeAction, BlockRef)>,
}

impl DCEContext {
    fn new(module: &Module) -> Self {
        Self {
            side_effect: SideEffectMarker::from_module(module),
            inst_to_remove: VecDeque::new(),
            inst_to_unplug: VecDeque::new(),
            block_to_merge: VecDeque::new(),
        }
    }

    fn exec(&mut self, module: &mut Module) {
        let funcs = module.dump_funcs(false);
        for func in funcs {
            self.perform_dce_for_func(&mut module.allocs, func);
        }
    }

    fn perform_dce_for_func(&mut self, allocs: &mut IRAllocs, func: FuncRef) {
        for (bref, block) in func.get_body_from_alloc(&allocs.globals).view(&allocs.blocks) {
            for (iref, inst) in block.insts.view(&allocs.insts) {
                if self.side_effect.inst_has_side_effect(iref) {
                    continue;
                }
                self.inst_to_unplug.push_back(iref);
                for operand in &inst.get_operands() {
                    operand.clean_operand();
                }
            }
            self.inst_to_remove.reserve(self.inst_to_unplug.len());
            while let Some(inst) = self.inst_to_unplug.pop_front() {
                block.insts.unplug_node(&allocs.insts, inst).unwrap();
                self.inst_to_remove.push_back(inst);
                self.side_effect.insts.remove(&inst);
            }

            if let Some(action) = Self::block_can_merge(bref, block, allocs) {
                self.block_to_merge.push_back((action, bref));
            }
        }

        while let Some(inst) = self.inst_to_remove.pop_front() {
            allocs.insts.remove(inst.get_handle());
        }

        while let Some((_action, _block)) = self.block_to_merge.pop_front() {
            // self.merge_block(allocs, action, block);
            // func.get_body(&allocs.globals)
            //     .unplug_node(&allocs.blocks, block)
            //     .unwrap();
            // allocs.blocks.remove(block.get_handle());
        }
    }
}

#[derive(Debug)]
enum BlockMergeAction {
    Del,
    Up,
    Down,
}

impl DCEContext {
    fn block_can_merge(
        bref: BlockRef,
        block: &BlockData,
        allocs: &IRAllocs,
    ) -> Option<BlockMergeAction> {
        let has_single_pred = !block.preds.is_empty() && !block.has_multiple_preds();
        let has_single_succ = !block.get_successors(&allocs.insts).is_empty()
            && !block.has_multiple_succs(&allocs.insts);
        if !has_single_pred || !has_single_succ {
            return None;
        }

        let pred = block.preds.front().unwrap().get_terminator();
        let succ = Self::_get_only_succ(bref, allocs);

        if succ == bref {
            // 检测到循环. 结合前面的判断条件可得, 这个基本块是个不可达的孤岛——这个应该早就被移除了
            // 这里返回 None 作为一个占位符.
            return None;
        }

        let only_used_by_succ = bref.users(allocs).iter().all(|u| {
            let UserID::Inst(user) = u.user.get() else {
                return false;
            };
            user.get_parent(allocs) == succ
        });
        if !only_used_by_succ {
            return None;
        }

        let block_empty = block.insts.len() == 2; /* PhiInstEnd + terminator */

        pred.read_jts(&allocs.insts, |jts| {
            use BlockMergeAction::*;
            if block_empty {
                // Case 1: 空块合并 - 确保前驱没有直接到后继的边
                let ok = jts.iter().all(|jt| jt.get_block() != succ);
                if ok { Some(Del) } else { None }
            } else if jts.iter().all(|jt| jt.get_block() == bref) {
                Some(Up)
            } else {
                // Case 3: 后继块只有 block 一个前驱
                let terminator = block.get_terminator_from_alloc(&allocs.insts);
                let ok = succ
                    .preds(allocs)
                    .iter()
                    .all(|jt| jt.get_terminator() == terminator);
                if ok { Some(Down) } else { None }
            }
        })
    }

    fn _merge_block(&mut self, _allocs: &IRAllocs, action: BlockMergeAction, block: BlockRef) {
        // match action {
        //     BlockMergeAction::Del => {
        //         let pred = Self::get_only_pred(block, allocs);
        //         let succ = Self::get_only_succ(block, allocs);
        //         // block.users(allocs).move_all_to(other, on_move)
        //         let curr_users = block.users(allocs);
        //         let pred_users = pred.users(allocs);
        //     }
        //     BlockMergeAction::Up => todo!(),
        //     BlockMergeAction::Down => todo!(),
        // }
        log::debug!("implement block merge: action {action:?} block {block:?}");
    }

    fn _get_only_succ(block: BlockRef, allocs: &IRAllocs) -> BlockRef {
        block.succs(allocs).first().unwrap().get_block()
    }
    fn _get_only_pred(block: BlockRef, allocs: &IRAllocs) -> BlockRef {
        block
            .preds(allocs)
            .front()
            .unwrap()
            .get_terminator_inst()
            .get_parent(allocs)
    }
}

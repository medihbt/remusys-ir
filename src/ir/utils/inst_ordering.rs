use crate::ir::{BlockID, IRAllocs, ISubInstID, InstID};
use mtb_entity_slab::IEntityListNodeID;
use std::{borrow::Borrow, cell::RefCell, collections::HashMap};

pub trait InstOrdering {
    /// Returns true if `f` comes before `b` in the instruction order.
    fn comes_before(&self, allocs: &IRAllocs, f: InstID, b: InstID) -> bool;

    /// Called when `inst` is inserted into its parent block.
    /// Since the inserted inst has a parent, we can get it from `inst`.
    fn on_inst_insert(&self, allocs: &IRAllocs, inst: InstID);

    /// `inst` is removed from `block`. since removed inst has no parent,
    /// we need to pass `block` explicitly
    fn on_inst_remove(&self, block: BlockID, inst: InstID);

    /// `old` is replaced with `new` at the same pos
    fn on_inst_replace(&self, allocs: &IRAllocs, old: InstID, new: InstID);

    fn invalidate_block(&self, allocs: &IRAllocs, block: BlockID);
    fn invalidate_all(&self, allocs: &IRAllocs);
}

pub struct ListWalkOrder;

impl InstOrdering for ListWalkOrder {
    fn comes_before(&self, allocs: &IRAllocs, f: InstID, b: InstID) -> bool {
        // traivial list walk
        let alloc = &allocs.insts;
        let mut current = f;
        while !current.is_sentinel(alloc) {
            if current == b {
                return true;
            }
            current = match current.get_next_id(alloc) {
                Some(inst) => inst,
                None => break,
            };
        }
        false
    }

    fn on_inst_insert(&self, _: &IRAllocs, _: InstID) {}
    fn on_inst_remove(&self, _: BlockID, _: InstID) {}
    fn on_inst_replace(&self, _: &IRAllocs, _: InstID, _: InstID) {}
    fn invalidate_block(&self, _: &IRAllocs, _: BlockID) {}
    fn invalidate_all(&self, _: &IRAllocs) {}
}
#[derive(Default)]
pub struct InstOrderCache {
    inner: RefCell<BCRInner>,
}

impl<C: Borrow<InstOrderCache>> InstOrdering for C {
    fn comes_before(&self, allocs: &IRAllocs, front: InstID, back: InstID) -> bool {
        let front_parent = front.get_parent(allocs).expect("inst has no parent block");
        let back_parent = back.get_parent(allocs).expect("inst has no parent block");
        if front_parent != back_parent {
            return false;
        }
        let mut inner = self.borrow().inner.borrow_mut();
        let front_pos = inner.ensure_known(allocs, front);
        let back_pos = inner.ensure_known(allocs, back);
        front_pos < back_pos
    }
    fn on_inst_insert(&self, allocs: &IRAllocs, inst: InstID) {
        let Some(prev) = inst.get_prev_id(&allocs.insts) else {
            panic!("Broken list: Cannot insert an inst at the pos of sentinel");
        };
        let parent = inst.get_parent(allocs).expect("inst has no parent");
        let mut inner = self.borrow().inner.borrow_mut();
        inner.truncate_block(allocs, parent, prev);
    }

    fn on_inst_remove(&self, parent: BlockID, inst: InstID) {
        let mut inner = self.borrow().inner.borrow_mut();
        inner.remove_inst(parent, inst);
    }

    fn on_inst_replace(&self, allocs: &IRAllocs, old: InstID, new: InstID) {
        let mut inner = self.borrow().inner.borrow_mut();
        inner.replace_inst(allocs, old, new);
    }

    fn invalidate_block(&self, allocs: &IRAllocs, block: BlockID) {
        self.borrow()
            .inner
            .borrow_mut()
            .invalidate_block(allocs, block);
    }

    fn invalidate_all(&self, _: &IRAllocs) {
        let mut inner = self.borrow().inner.borrow_mut();
        inner.block_valid_to.clear();
        inner.inst_pos.clear();
    }
}

impl InstOrderCache {
    pub fn new() -> Self {
        Self::default()
    }
}

#[derive(Default)]
struct BCRInner {
    /// 每条指令在其所属基本块中的位置索引。这个索引不一定是连续的，也不都有效.
    /// 有效性由 `block_valid_to` 决定.
    inst_pos: HashMap<InstID, usize>,
    /// 表示每个基本块中，`inst_pos` 中有效指令的最后一条指令的索引.
    block_valid_to: HashMap<BlockID, (usize, InstID)>,
}

impl BCRInner {
    fn invalidate_block(&mut self, allocs: &IRAllocs, block: BlockID) {
        self.block_valid_to.remove(&block);
        // 这个其实可以不用加。不过考虑到一个基本块移除掉以后所有的指令都大概不会再被访问了，清理掉也无妨。
        self.inst_pos
            .retain(|&inst, _| inst.get_parent(allocs) != Some(block));
    }

    fn inst_try_get_pos(&self, parent: BlockID, inst: InstID) -> Option<usize> {
        let &(valid_to, last_inst) = self.block_valid_to.get(&parent)?;
        if last_inst == inst {
            return Some(valid_to);
        }
        let &pos = self.inst_pos.get(&inst)?;
        if pos > valid_to { None } else { Some(pos) }
    }

    fn truncate_block(&mut self, allocs: &IRAllocs, block: BlockID, new_last: InstID) {
        if new_last.is_sentinel(&allocs.insts) {
            self.block_valid_to.remove(&block);
            return;
        }
        let Some(pos) = self.inst_try_get_pos(block, new_last) else {
            return;
        };
        // replace old_last old "to" position with new_last new "to" position
        self.block_valid_to.insert(block, (pos, new_last));

        // 继续往下走, 删除后续所有指令的缓存
        let mut next_inst = new_last;
        while let Some(following) = next_inst.get_next_id(&allocs.insts) {
            let Some(_) = self.inst_pos.get(&following) else {
                break;
            };
            self.inst_pos.remove(&following);
            next_inst = following;
        }
    }
    fn ensure_known(&mut self, allocs: &IRAllocs, inst: InstID) -> usize {
        let parent = inst.get_parent(allocs).expect("inst has no parent");
        if let Some(pos) = self.inst_try_get_pos(parent, inst) {
            return pos;
        }
        // walk from last known position
        let alloc = &allocs.insts;
        let (mut pos, mut current) = match self.block_valid_to.get(&parent) {
            Some(&(p, last_inst)) => (p, last_inst),
            None => {
                // start from the beginning of the block
                let first_inst = parent
                    .get_insts(allocs)
                    .get_front_id(alloc)
                    .expect("block has no insts");
                (0, first_inst)
            }
        };
        while current != inst {
            current = current
                .get_next_id(alloc)
                .expect("inst not found in its parent block");
            pos += 1;
            self.inst_pos.insert(current, pos);
        }
        // update valid_to
        self.block_valid_to.insert(parent, (pos, inst));
        pos
    }

    fn remove_inst(&mut self, parent: BlockID, inst: InstID) {
        self.inst_pos.remove(&inst);
        let Some((_, last_inst)) = self.block_valid_to.get_mut(&parent) else {
            return;
        };
        if *last_inst == inst {
            self.block_valid_to.remove(&parent);
        }
    }

    fn replace_inst(&mut self, allocs: &IRAllocs, old: InstID, new: InstID) {
        if let Some(&pos) = self.inst_pos.get(&old) {
            self.inst_pos.insert(new, pos);
            self.inst_pos.remove(&old);
        }
        let parent = new.get_parent(allocs).expect("unattached new inst");
        if let Some((_, last_inst)) = self.block_valid_to.get_mut(&parent)
            && *last_inst == old
        {
            *last_inst = new;
        }
    }
}

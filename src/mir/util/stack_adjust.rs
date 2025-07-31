use std::{
    cell::{Cell, RefCell},
    collections::{BTreeMap, VecDeque},
    ops::Range,
    rc::Rc,
};

use slab::Slab;

use crate::{
    base::{INullableValue, SlabListNodeRef, SlabListRange, SlabRef},
    mir::{
        inst::{
            IMirSubInst, MirInstRef,
            inst::MirInst,
            mirops::{MirRestoreRegs, MirSaveRegs},
        },
        module::{MirModule, block::MirBlockRef},
        operand::physreg_set::MirPhysRegSet,
    },
};

#[derive(Debug, Clone, Copy)]
pub enum MirSpAdjust {
    /// 减少栈指针, 需要指定减少的字节数
    SubSP { delta: u32, sub_sp: MirInstRef, add_sp: MirInstRef },
    /// 保存寄存器, 不到最后不知道要预留多少空间
    SaveRegs { regset: MirPhysRegSet, save_reg: MirInstRef, restore_reg: MirInstRef },
    /// 什么都不做, 仅仅是为了占位
    NOP,
}

impl MirSpAdjust {
    /// 获取栈指针的变化量
    pub fn get_sp_delta(&self) -> u32 {
        match self {
            MirSpAdjust::SubSP { delta, .. } => {
                if delta % 16 != 0 {
                    panic!("SubSP delta must be a multiple of 16, found: {delta}");
                }
                *delta
            }
            // 每个寄存器占用 8 字节
            MirSpAdjust::SaveRegs { regset, .. } => (regset.num_regs() * 8).next_multiple_of(16),
            MirSpAdjust::NOP => 0, // 什么都不做, 变化量为 0
        }
    }

    /// 获取当前的指令范围, 不包括头部的保存和尾部的恢复指令.
    pub fn get_inst_range(&self) -> Option<SlabListRange<MirInstRef>> {
        let (head, tail) = match self {
            MirSpAdjust::SubSP { sub_sp, add_sp, .. } => (*sub_sp, *add_sp),
            MirSpAdjust::SaveRegs { save_reg, restore_reg, .. } => (*save_reg, *restore_reg),
            MirSpAdjust::NOP => return None, // 什么都不做, 没有指令范围
        };
        Some(SlabListRange { node_head: head, node_tail: tail })
    }
}

#[derive(Debug, Clone)]
pub struct MirSpAdjustNode {
    pub block: MirBlockRef,
    pub adjust: Cell<MirSpAdjust>,
    pub children: Vec<MirSpAdjustNode>,
}

impl Default for MirSpAdjustNode {
    fn default() -> Self {
        Self {
            block: MirBlockRef::new_null(),
            adjust: Cell::new(MirSpAdjust::NOP),
            children: Vec::new(),
        }
    }
}

impl MirSpAdjustNode {
    pub fn replace_saved_regs(&self, saved_regs: MirPhysRegSet, alloc_inst: &Slab<MirInst>) {
        self.do_replace_saved_regs(saved_regs, alloc_inst);
        for children in &self.children {
            children.replace_saved_regs(saved_regs, alloc_inst);
        }
    }

    pub fn is_valid(&self) -> bool {
        self.block.is_nonnull() && !matches!(self.adjust.get(), MirSpAdjust::NOP)
    }
    pub fn is_invalid(&self) -> bool {
        !self.is_valid()
    }

    fn do_replace_saved_regs(&self, saved_regs: MirPhysRegSet, alloc_inst: &Slab<MirInst>) {
        let MirSpAdjust::SaveRegs { regset, save_reg, restore_reg } = self.adjust.get() else {
            return;
        };
        if regset == saved_regs {
            // 如果寄存器集相同, 则不需要替换
            return;
        }

        self.adjust
            .set(MirSpAdjust::SaveRegs { regset: saved_regs, save_reg, restore_reg });

        let MirInst::MirSaveRegs(save_reg) = save_reg.to_data(alloc_inst) else {
            panic!("Expected MirSaveRegs, found {:?}", save_reg);
        };
        let MirInst::MirRestoreRegs(restore_reg) = restore_reg.to_data(alloc_inst) else {
            panic!("Expected MirRestoreRegs, found {:?}", restore_reg);
        };
        save_reg.set_saved_regs(saved_regs);
        restore_reg.set_saved_regs(saved_regs);
    }

    fn make_offset_map(
        &self,
        level: u32,
        parent_offset: u32,
        alloc_inst: &Slab<MirInst>,
        out_map: &mut BTreeMap<MirInstRef, u32>,
    ) {
        let this_node_offset = self.adjust.get().get_sp_delta() + parent_offset;
        let Some(range) = self.adjust.get().get_inst_range() else {
            assert!(
                self.children.is_empty(),
                "Node with children must have a valid instruction range"
            );
            return; // 没有指令范围, 跳过
        };
        for node in self.children.iter() {
            node.make_offset_map(level + 1, this_node_offset, alloc_inst, out_map);
        }
        for (iref, _) in range.view(alloc_inst) {
            if out_map.contains_key(&iref) {
                // 这个指令引用已经在上面处理子结点的过程中被处理过了, 跳过
                continue;
            }
            eprintln!(
                "{}Adding offset for instruction {iref:?}: {this_node_offset}",
                "..".repeat(level as usize)
            );
            out_map.insert(iref, this_node_offset);
        }
    }
}

#[derive(Debug, Clone)]
pub struct MirSpAdjustTree {
    /// 根结点集合 -- 实际上是一个从 block 映射到 MirSpAdjustNode 的 multimap.
    /// 我似乎已经不止一次用数组排个序当 multimap 了...
    pub roots: Vec<MirSpAdjustNode>,
}

impl MirSpAdjustTree {
    /// 查找某个基本块 block 对应的 MirSpAdjustNode 区间.
    pub fn block_find_nodes(&self, block: MirBlockRef) -> &[MirSpAdjustNode] {
        let begin_index = self.roots.partition_point(|node| node.block < block);
        let end_index = self.roots.partition_point(|node| node.block <= block);
        &self.roots[begin_index..end_index]
    }

    pub fn block_find_nodes_mut(&mut self, block: MirBlockRef) -> &mut [MirSpAdjustNode] {
        let begin_index = self.roots.partition_point(|node| node.block < block);
        let end_index = self.roots.partition_point(|node| node.block <= block);
        &mut self.roots[begin_index..end_index]
    }

    pub fn block_set_adjusted_regs(
        &self,
        block: MirBlockRef,
        saved_regs: MirPhysRegSet,
        alloc_inst: &Slab<MirInst>,
    ) {
        for node in self.block_find_nodes(block) {
            node.replace_saved_regs(saved_regs, alloc_inst);
        }
    }

    pub fn roots_iter(&self) -> impl Iterator<Item = &MirSpAdjustNode> {
        self.roots.iter()
    }

    pub fn make_offset_map(&self, alloc_inst: &Slab<MirInst>) -> BTreeMap<MirInstRef, u32> {
        let mut offset_map = BTreeMap::new();
        for root in &self.roots {
            root.make_offset_map(0, 0, alloc_inst, &mut offset_map);
        }
        offset_map
    }
}

impl MirSpAdjustTree {
    /// 找到所有指令区间相邻的寄存器保存-恢复区间, 并将它们合并为一个区间.
    ///
    /// 相邻区间的判断条件是:
    ///
    /// * 两个区间处于同一个基本块内
    /// * 前一个区间的结束指令和下一个区间的开始指令相邻
    /// * 两个区间保存和恢复的寄存器集相同
    pub fn merge_regsave_intervals(
        &mut self,
        alloc_inst: &Slab<MirInst>,
        mut remove_inst: impl FnMut(MirInstRef, MirBlockRef),
    ) {
        Self::do_merge_regsave_intervals(&mut self.roots, alloc_inst, &mut remove_inst);
    }

    /// 合并模块中所有函数的寄存器保存区间.
    pub fn merge_regsave_intervals_for_module(&mut self, module: &mut MirModule) {
        let allocs = module.allocs.get_mut();
        let mut inst_queue = VecDeque::new();
        self.merge_regsave_intervals(&allocs.inst, |iref, bref| {
            inst_queue.push_back((iref, bref))
        });

        // 处理合并后的指令, 从模块中删除旧的指令
        while let Some((iref, bref)) = inst_queue.pop_front() {
            bref.get_insts(&allocs.block)
                .unplug_node(&allocs.inst, iref)
                .expect("Failed to unplug old inst");
            allocs.inst.remove(iref.get_handle());
        }
    }

    fn do_merge_regsave_intervals(
        nodes: &mut Vec<MirSpAdjustNode>,
        alloc_inst: &Slab<MirInst>,
        remove_inst: &mut impl FnMut(MirInstRef, MirBlockRef),
    ) {
        let to_merge = Self::make_merge_range(nodes, alloc_inst);
        let mut nodes_dup = std::mem::take(nodes);
        nodes.reserve({
            let n_nodes_to_remove: usize = to_merge.iter().map(|(_, r)| r.len() - 1).sum();
            nodes_dup.len() - n_nodes_to_remove
        });

        for (_, range) in to_merge {
            let interval = &mut nodes_dup[range];
            Self::merge_range_interval(interval, remove_inst);
        }

        for node in nodes_dup {
            if node.is_valid() {
                nodes.push(node);
            }
        }

        for node in nodes.iter_mut() {
            // 递归处理子结点
            Self::do_merge_regsave_intervals(&mut node.children, alloc_inst, remove_inst);
        }
    }

    fn make_merge_range(
        nodes: &mut [MirSpAdjustNode],
        alloc_inst: &Slab<MirInst>,
    ) -> Vec<(MirPhysRegSet, Range<usize>)> {
        let mut to_merge: Vec<(MirPhysRegSet, Range<usize>)> = Vec::new();
        let mut last_regsave = None;
        for index in 0..nodes.len() {
            let node = &nodes[index];
            let MirSpAdjust::SaveRegs { save_reg, restore_reg, regset } = node.adjust.get() else {
                continue; // 不是保存寄存器的调整, 跳过
            };

            let Some((back_set, back_range)) = to_merge.last_mut() else {
                // 没有待合并的区间, 直接添加当前区间
                to_merge.push((regset, index..index + 1));
                last_regsave = Some((node.block, restore_reg));
                continue;
            };
            let back_set = back_set.clone();
            let Some((last_parent_bb, last_reg_restore)) = last_regsave else {
                unreachable!();
            };
            if node.block == last_parent_bb
                && regset == back_set
                && last_reg_restore.get_next_ref(alloc_inst) == Some(save_reg)
            {
                // 当前区间和上一个区间相邻, 合并它们
                back_range.end += 1;
                last_regsave = Some((node.block, restore_reg));
            } else {
                // 当前区间和上一个区间不相邻, 结束上一个合并区间
                to_merge.push((back_set, index..index + 1));
                last_regsave = Some((node.block, restore_reg));
            };
        }

        to_merge.retain(|(_, range)| range.len() > 1);
        to_merge
    }

    fn merge_range_interval(
        interval: &mut [MirSpAdjustNode],
        remove_inst: &mut impl FnMut(MirInstRef, MirBlockRef),
    ) {
        let (bb, regset, save, mut restore, mut new_children) = {
            let node0 = std::mem::take(&mut interval[0]);
            let MirSpAdjust::SaveRegs { regset, save_reg, restore_reg } = node0.adjust.get() else {
                panic!("Expected SaveRegs adjustment, found {:?}", node0.adjust);
            };
            let MirSpAdjustNode { block, mut children, .. } = node0;
            children.reserve(
                interval
                    .iter()
                    .skip(1)
                    .map(|node| node.children.len())
                    .sum(),
            );
            (block, regset, save_reg, restore_reg, children)
        };

        for node in &mut interval[1..] {
            let MirSpAdjust::SaveRegs { save_reg, restore_reg, .. } = node.adjust.get() else {
                panic!("Expected SaveRegs adjustment, found {:?}", node.adjust);
            };
            let MirSpAdjustNode { children, .. } = std::mem::take(node);

            remove_inst(save_reg, bb);
            remove_inst(restore, bb);
            restore = restore_reg; // 更新恢复寄存器引用
            new_children.extend(children);
        }

        interval[0] = MirSpAdjustNode {
            block: bb,
            adjust: Cell::new(MirSpAdjust::SaveRegs {
                regset,
                save_reg: save,
                restore_reg: restore,
            }),
            children: new_children,
        };
    }
}

#[derive(Debug)]
struct AdjTreeBuilderNode {
    block: Cell<MirBlockRef>,
    adjust: Cell<MirSpAdjust>,
    children: RefCell<Vec<Rc<AdjTreeBuilderNode>>>,
}

impl AdjTreeBuilderNode {
    pub fn new(block: MirBlockRef, adjust: MirSpAdjust) -> Self {
        Self {
            block: Cell::new(block),
            adjust: Cell::new(adjust),
            children: RefCell::new(Vec::new()),
        }
    }

    pub fn add_child(&self, child: AdjTreeBuilderNode) -> Rc<AdjTreeBuilderNode> {
        let child = Rc::new(child);
        self.children.borrow_mut().push(child.clone());
        child
    }
}

pub enum LowerInstAction {
    /// 什么都不做
    NOP(MirInst),

    /// 开始操作: 预留 SP 空间
    BeginSubSP(u32, MirInst),
    /// 结束操作: 预留 SP 空间
    EndSubSP(MirInst),

    /// 开始操作: 保存寄存器
    BeginSaveRegs(MirPhysRegSet, MirSaveRegs),
    /// 结束操作: 保存寄存器
    EndSaveRegs(MirRestoreRegs),
}

pub struct AdjTreeBuilder {
    roots: Vec<Rc<AdjTreeBuilderNode>>,
    node_stack: Vec<Rc<AdjTreeBuilderNode>>,
    curr_block: MirBlockRef,
}

impl AdjTreeBuilder {
    pub fn new() -> Self {
        Self {
            roots: Vec::new(),
            node_stack: Vec::new(),
            curr_block: MirBlockRef::new_null(),
        }
    }

    pub fn focus_to_block(&mut self, block: MirBlockRef) {
        if self.curr_block == block {
            return; // 已经在这个块上了, 不需要切换
        }
        assert!(
            self.node_stack.is_empty(),
            "Cannot change block while node stack is not empty; stack size: {}",
            self.node_stack.len()
        );
        self.curr_block = block;
    }

    pub fn exec(&mut self, action: LowerInstAction, alloc_inst: &mut Slab<MirInst>) -> MirInstRef {
        use LowerInstAction::*;
        match action {
            NOP(inst) => {
                let iref = MirInstRef::from_alloc(alloc_inst, inst);
                let new_node = AdjTreeBuilderNode::new(self.curr_block, MirSpAdjust::NOP);
                if let Some(curr) = self.node_stack.last() {
                    curr.add_child(new_node);
                } else {
                    self.roots.push(Rc::new(new_node));
                }
                iref
            }
            BeginSubSP(offset, inst) => {
                let iref = MirInstRef::from_alloc(alloc_inst, inst);
                let new_node = AdjTreeBuilderNode::new(
                    self.curr_block,
                    MirSpAdjust::SubSP {
                        delta: offset,
                        sub_sp: iref,
                        // 结束 SP 调整的指令, 目前不知道是什么
                        add_sp: MirInstRef::new_null(),
                    },
                );

                let new_node = if let Some(curr) = self.node_stack.last() {
                    curr.add_child(new_node)
                } else {
                    // self.roots.push(Rc::new(new_node));
                    let new_node = Rc::new(new_node);
                    self.roots.push(new_node.clone());
                    new_node
                };
                self.node_stack.push(new_node);
                iref
            }
            EndSubSP(inst) => {
                let iref = MirInstRef::from_alloc(alloc_inst, inst);
                let Some(curr) = self.node_stack.pop() else {
                    panic!("EndSubSP called without matching BeginSubSP");
                };
                let mut adjust = curr.adjust.get();
                if let MirSpAdjust::SubSP { add_sp, .. } = &mut adjust {
                    *add_sp = iref; // 设置结束 SP 调整的指令
                } else {
                    panic!("EndSubSP called on a node that is not a BeginSubSP");
                }
                curr.adjust.set(adjust);
                drop(curr); // 释放当前节点, 以便后续操作
                iref
            }
            BeginSaveRegs(saved_regs, inst) => {
                let iref = MirInstRef::from_alloc(alloc_inst, inst.into_mir());
                let new_node = AdjTreeBuilderNode::new(
                    self.curr_block,
                    MirSpAdjust::SaveRegs {
                        regset: saved_regs,
                        save_reg: iref,
                        // 结束保存寄存器的指令, 目前不知道是什么
                        restore_reg: MirInstRef::new_null(),
                    },
                );
                let new_node = if let Some(curr) = self.node_stack.last() {
                    curr.add_child(new_node)
                } else {
                    // self.roots.push(Rc::new(new_node));
                    let new_node = Rc::new(new_node);
                    self.roots.push(new_node.clone());
                    new_node
                };
                self.node_stack.push(new_node);
                iref
            }
            EndSaveRegs(inst) => {
                let iref = MirInstRef::from_alloc(alloc_inst, inst.into_mir());
                let Some(curr) = self.node_stack.pop() else {
                    panic!("EndSaveRegs called without matching BeginSaveRegs");
                };
                let mut adjust = curr.adjust.get();
                if let MirSpAdjust::SaveRegs { restore_reg, .. } = &mut adjust {
                    *restore_reg = iref; // 设置结束保存寄存器的指令
                } else {
                    panic!("EndSaveRegs called on a node that is not a BeginSaveRegs");
                }
                curr.adjust.set(adjust);
                drop(curr); // 释放当前节点, 以便后续操作
                iref
            }
        }
    }

    /// 这个 builder 里所有的 Rc RefCell Cell 都是在 build 时才会被使用的, 实际的 MirSpAdjustTree
    /// 要是还有这些包装器的话就太多余且麻烦了. 因此这个 build 方法会打平所有的包装器, 给所有结点按照
    /// block 排序, 最后返回组装好的 MirSpAdjustTree.
    pub fn build(self) -> MirSpAdjustTree {
        assert!(
            self.node_stack.is_empty(),
            "Cannot build MirSpAdjustTree while node stack is not empty; stack size: {}",
            self.node_stack.len()
        );
        let Self { roots, .. } = self;
        let mut roots = Self::build_one_block_of_nodes(roots);
        roots.sort_by_key(|node| node.block);
        MirSpAdjustTree { roots }
    }

    fn build_one_block_of_nodes(nodes: Vec<Rc<AdjTreeBuilderNode>>) -> Vec<MirSpAdjustNode> {
        let mut result = Vec::with_capacity(nodes.len());
        for node in nodes {
            let node = Rc::try_unwrap(node)
                .expect("Failed to unwrap AdjTreeBuilderNode, it should not be shared");
            let block = node.block.get();
            let adjust = node.adjust.get();
            let children = Self::build_one_block_of_nodes(node.children.take());
            result.push(MirSpAdjustNode { block, adjust: Cell::new(adjust), children });
        }
        result
    }
}

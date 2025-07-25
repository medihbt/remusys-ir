mod lower_calls;
mod lower_copy;
mod lower_ldr_const;
mod lower_returns;
mod lower_stack;

use crate::{
    base::{NullableValue, slablist::SlabListRange, slabref::SlabRef},
    mir::{
        inst::{
            IMirSubInst, MirInstRef,
            inst::MirInst,
            mirops::{MirRestoreRegs, MirSaveRegs},
        },
        module::{MirGlobal, MirModule, block::MirBlockRef, func::MirFunc},
        operand::physreg_set::MirPhysRegSet,
    },
};
use slab::Slab;
use std::{
    collections::{BTreeMap, VecDeque},
    rc::Rc,
};

pub use lower_copy::*;
pub use lower_returns::lower_mir_ret;
pub use lower_ldr_const::*;

fn lower_an_inst(
    inst: &MirInst,
    parent_func: &MirFunc,
    out_actions: &mut VecDeque<LowerInstAction>,
) {
    match inst {
        MirInst::MirCopy64(copy64) => lower_copy64_inst(copy64, out_actions),
        MirInst::MirCopy32(copy32) => lower_copy32_inst(copy32, out_actions),
        MirInst::MirFCopy64(fcopy64) => {
            let mut inner = parent_func.borrow_inner_mut();
            lower_fcopy64_inst(fcopy64, &mut inner.vreg_alloc, out_actions)
        }
        MirInst::MirFCopy32(fcopy32) => {
            let mut inner = parent_func.borrow_inner_mut();
            lower_fcopy32_inst(fcopy32, &mut inner.vreg_alloc, out_actions)
        }
        MirInst::MirCall(call_inst) => {
            call_inst.dump_actions_template(out_actions);
        }
        MirInst::MirReturn(mir_ret) => {
            let mut inner = parent_func.borrow_inner_mut();
            lower_mir_ret(mir_ret, &mut inner.vreg_alloc, out_actions)
        }
        MirInst::MirPCopy(pcopy) => {
            todo!("Handle inst {pcopy:?}: Please implement MRS and MSR in RIG file first!")
        }
        MirInst::MirSwitch(mir_switch) => todo!("Handle inst {mir_switch:?}"),
        _ => {}
    }
}

pub fn lower_a_function(
    module: &MirModule,
    func: &MirFunc,
    sp_adjustments: &mut BTreeMap<MirInstRef, InstSPAdjustments>,
) {
    let mut allocs = module.allocs.borrow_mut();
    let mut insts_to_process = Vec::new();

    for (block_ref, block) in func.blocks.view(&allocs.block) {
        for (inst_ref, inst) in block.insts.view(&allocs.inst) {
            let is_mir_pseudo = matches!(
                inst,
                MirInst::MirCopy64(_)
                    | MirInst::MirCopy32(_)
                    | MirInst::MirFCopy64(_)
                    | MirInst::MirFCopy32(_)
                    | MirInst::MirPCopy(_)
                    | MirInst::MirCall(_)
                    | MirInst::MirReturn(_)
                    | MirInst::MirSwitch(_)
            );
            if is_mir_pseudo {
                insts_to_process.push((inst_ref, inst.clone(), block_ref));
            }
        }
    }

    let mut out_actions = VecDeque::new();
    for (iref, inst, parent_bb) in insts_to_process {
        lower_an_inst(&inst, func, &mut out_actions);
        let mut executor = LowerInstExecutor::with_capacity(out_actions.len(), parent_bb);
        while let Some(action) = out_actions.pop_front() {
            let new_inst = executor.exec(&mut allocs.inst, action);
            parent_bb
                .get_insts(&allocs.block)
                .node_add_prev(&allocs.inst, iref, new_inst)
                .expect("Failed to add new inst");
        }
        parent_bb
            .get_insts(&allocs.block)
            .unplug_node(&allocs.inst, iref)
            .expect("Failed to unplug old inst");
        allocs.inst.remove(iref.get_handle());

        assert_eq!(
            executor.height(),
            0,
            "Executor stack should be empty after execution"
        );

        for inst in executor.insts {
            sp_adjustments.insert(
                inst.inst,
                InstSPAdjustments {
                    inst: inst.inst,
                    parent: inst.parent,
                    adjustments: inst.adjustments,
                },
            );
        }
    }
}

/// LowerInst Action 的执行器.
struct LowerInstExecutor {
    insts: Vec<InstSPAdjustments>,
    begin_stack: Vec<usize>,
    adj_stack: Vec<InstSPAdjustment>,
    parent: MirBlockRef,
}

impl LowerInstExecutor {
    fn with_capacity(capacity: usize, parent: MirBlockRef) -> Self {
        Self {
            insts: Vec::with_capacity(capacity),
            begin_stack: Vec::new(),
            adj_stack: Vec::new(),
            parent,
        }
    }

    fn height(&self) -> usize {
        debug_assert_eq!(self.begin_stack.len(), self.adj_stack.len());
        self.begin_stack.len()
    }

    /// 执行一个 LowerInstAction, 返回一个 MirInstRef.
    ///
    /// #### 潜在的约定
    ///
    /// 这个 executor 连带整个 Stack adjustment 系统都暗含这么几个约定:
    ///
    /// * 栈指针的临时调整和恢复(包括参数溢出调整、寄存器保存和恢复调整)只会在同一个基本块内进行,
    ///   且进行完毕后栈指针立刻恢复到原本的位置.
    /// * 寄存器的保存在栈区内一定有一块属于自己的临时区间, 保存寄存器的操作一定是先开辟一块栈空间
    ///   再保存, 恢复寄存器的操作一定是先恢复寄存器再释放栈空间.
    ///
    /// 这些约定的产生主要是为了避免合并操作导致的 SP 计算麻烦.
    fn exec(&mut self, alloc_inst: &mut Slab<MirInst>, action: LowerInstAction) -> MirInstRef {
        match action {
            LowerInstAction::NOP(inst) => {
                let inst_ref = MirInstRef::from_alloc(alloc_inst, inst);
                self.insts.push(InstSPAdjustments {
                    inst: inst_ref,
                    parent: self.parent,
                    adjustments: self.adj_stack.clone(),
                });
                inst_ref
            }
            LowerInstAction::BeginSubSP(offset, inst) => {
                let inst_ref = MirInstRef::from_alloc(alloc_inst, inst);
                self.insts.push(InstSPAdjustments {
                    inst: inst_ref,
                    parent: self.parent,
                    adjustments: self.adj_stack.clone(),
                });
                self.adj_stack.push(InstSPAdjustment::SubSP {
                    delta: offset,
                    subsp: inst_ref,
                    addsp: MirInstRef::new_null(),
                });
                self.begin_stack.push(self.insts.len());
                inst_ref
            }
            LowerInstAction::EndSubSP(inst) => {
                let inst_ref = MirInstRef::from_alloc(alloc_inst, inst);
                let Some(InstSPAdjustment::SubSP { .. }) = self.adj_stack.pop() else {
                    panic!("EndSubSP without matching BeginSubSP");
                };
                let Some(begin_idx) = self.begin_stack.pop() else {
                    panic!("EndSubSP without matching BeginSubSP");
                };
                let adj_item_index = self.height();
                for inst in self.insts[begin_idx..].iter_mut() {
                    let InstSPAdjustment::SubSP { addsp, .. } =
                        &mut inst.adjustments[adj_item_index]
                    else {
                        panic!("EndSubSP without matching BeginSubSP");
                    };
                    *addsp = inst_ref;
                }
                inst_ref
            }
            LowerInstAction::BeginSaveRegs(saved_regs, inst) => {
                let inst_ref = MirInstRef::from_alloc(alloc_inst, inst.into_mir());
                self.insts.push(InstSPAdjustments {
                    inst: inst_ref,
                    parent: self.parent,
                    adjustments: self.adj_stack.clone(),
                });
                self.adj_stack.push(InstSPAdjustment::SaveRegs {
                    regset: saved_regs,
                    save_reg: inst_ref,
                    restore_reg: MirInstRef::new_null(),
                });
                self.begin_stack.push(self.insts.len());
                inst_ref
            }
            LowerInstAction::EndSaveRegs(inst) => {
                let inst_ref = MirInstRef::from_alloc(alloc_inst, inst.into_mir());
                let Some(InstSPAdjustment::SaveRegs { .. }) = self.adj_stack.pop() else {
                    panic!("EndSaveRegs without matching BeginSaveRegs");
                };
                let Some(begin_idx) = self.begin_stack.pop() else {
                    panic!("EndSaveRegs without matching BeginSaveRegs");
                };
                let adj_item_index = self.height();
                for inst in self.insts[begin_idx..].iter_mut() {
                    let InstSPAdjustment::SaveRegs { restore_reg, .. } =
                        &mut inst.adjustments[adj_item_index]
                    else {
                        panic!("EndSaveRegs without matching BeginSaveRegs");
                    };
                    *restore_reg = inst_ref;
                }
                inst_ref
            }
        }
    }
}

pub fn lower_a_module(module: &MirModule) -> BTreeMap<MirInstRef, InstSPAdjustments> {
    let mut funcs = Vec::new();
    for items in &module.items {
        match &*items.data_from_module(module) {
            MirGlobal::Function(f) => {
                if f.is_extern() {
                    continue;
                }
                funcs.push(Rc::clone(f));
            }
            _ => continue,
        }
    }
    let mut sp_adjustments = BTreeMap::new();
    for func in funcs {
        lower_a_function(module, &func, &mut sp_adjustments);
    }
    sp_adjustments
}

/// 表示在同一个 MIR 基本块中一小段 SP 会出现临时变动的操作.
/// 注意, Remusys MIR 目前只支持追踪在同一个基本块中进行的 SP 调整.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InstSPAdjustment {
    /// 表示 SP 经过一个直接的相减调整
    SubSP {
        delta: u32,
        subsp: MirInstRef,
        addsp: MirInstRef,
    },
    /// 表示保存寄存器, 不到最后不知道要预留多少空间
    SaveRegs {
        regset: MirPhysRegSet,
        save_reg: MirInstRef,
        restore_reg: MirInstRef,
    },
}

impl InstSPAdjustment {
    /// 获取当前 SP 调整的
    pub fn get_inst_range(&self) -> SlabListRange<MirInstRef> {
        let (begin, end) = match self {
            InstSPAdjustment::SubSP { subsp, addsp, .. } => (*subsp, *addsp),
            InstSPAdjustment::SaveRegs {
                save_reg,
                restore_reg,
                ..
            } => (*save_reg, *restore_reg),
        };
        SlabListRange {
            node_head: begin,
            node_tail: end,
        }
    }
}

#[derive(Debug, Clone)]
pub struct InstSPAdjustments {
    pub inst: MirInstRef,
    pub parent: MirBlockRef,
    pub adjustments: Vec<InstSPAdjustment>,
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

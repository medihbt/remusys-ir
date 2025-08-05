use crate::{
    base::SlabRef,
    mir::{
        inst::{IMirSubInst, MirInstRef, impls::*, inst::MirInst, opcode::MirOP},
        module::{
            MirModule,
            func::MirFunc,
            stack::{MirStackLayout, StackItemKind},
        },
        operand::{IMirSubOperand, MirOperand, imm::*, imm_traits, reg::*},
        translate::mir_pass::stack_lower::{TmpRegAlloc, lower_stackpos_ldst},
    },
};
use std::collections::{BTreeMap, VecDeque};

/// 处理函数内的每条指令, 把栈位置寄存器变为实际的栈位置.
pub fn lower_stackpos_operands_for_func(
    func: &MirFunc,
    module: &mut MirModule,
    sp_offset_map: &BTreeMap<MirInstRef, u32>,
    stack_layout: &MirStackLayout,
    stackpos_calc_queue: &mut VecDeque<MirInst>,
) {
    let tmpptr_reg = GPR64::new(RegID::Phys(29));
    let allocs = module.allocs.get_mut();
    let insts_with_stackpos = func.dump_insts_when(&allocs.block, &allocs.inst, |inst| {
        for operand in inst.in_operands() {
            if operand_maybe_stackpos(operand.get()) {
                return true;
            }
        }
        for operand in inst.out_operands() {
            let operand = operand.get();
            let opcode = inst.get_opcode();
            if operand_maybe_stackpos(operand) {
                panic!("Stackpos {operand:?} is read-only but used as output in inst {opcode:?}");
            }
        }
        false
    });
    for (bref, iref) in insts_with_stackpos {
        let extra_delta_sp = sp_offset_map.get(&iref).copied().unwrap_or(0) as u64;
        let action = lower_stackpos_operands_for_inst(
            stack_layout,
            stackpos_calc_queue,
            tmpptr_reg,
            extra_delta_sp,
            iref.to_data(&allocs.inst),
        );
        match action {
            LowerPosAction::Keep | LowerPosAction::InsertFront => {
                while let Some(inst) = stackpos_calc_queue.pop_front() {
                    let new_inst = MirInstRef::from_alloc(&mut allocs.inst, inst);
                    bref.get_insts(&allocs.block)
                        .node_add_prev(&allocs.inst, iref, new_inst)
                        .expect("Failed to add new inst");
                }
            }
            LowerPosAction::Replace => {
                while let Some(inst) = stackpos_calc_queue.pop_front() {
                    let new_inst = MirInstRef::from_alloc(&mut allocs.inst, inst);
                    bref.get_insts(&allocs.block)
                        .node_add_prev(&allocs.inst, iref, new_inst)
                        .expect("Failed to add new inst");
                }
                bref.get_insts(&allocs.block)
                    .unplug_node(&allocs.inst, iref)
                    .expect("Failed to remove old inst");
                allocs.inst.remove(iref.get_handle());
            }
            LowerPosAction::ReplaceOpcode(mir_op) => {
                while let Some(inst) = stackpos_calc_queue.pop_front() {
                    let new_inst = MirInstRef::from_alloc(&mut allocs.inst, inst);
                    bref.get_insts(&allocs.block)
                        .node_add_prev(&allocs.inst, iref, new_inst)
                        .expect("Failed to add new inst");
                }
                iref.to_data_mut(&mut allocs.inst).common_mut().opcode = mir_op;
            }
        }
    }

    fn operand_maybe_stackpos(operand: MirOperand) -> bool {
        let MirOperand::GPReg(reg) = operand else {
            return false;
        };
        reg.get_bits_log2() == 6 && reg.is_virtual()
    }
}

pub(super) enum LowerPosAction {
    /// 该改的都提前改完了, 清空指令队列, 不需要再做任何操作
    Keep,
    /// 把指令队列里的指令插入到当前指令前面
    InsertFront,
    /// 把当前指令替换成整个指令队列里的指令
    Replace,
    /// 把当前指令的操作码替换成新的操作码;
    /// 如果指令队列非空, 就把指令队列里的指令插入到当前指令前面
    ReplaceOpcode(MirOP),
}

fn lower_stackpos_operands_for_inst(
    stack: &MirStackLayout,
    insts: &mut VecDeque<MirInst>,
    tmpreg: GPR64,
    dsp: u64,
    inst: &MirInst,
) -> LowerPosAction {
    let sp = GPR64::sp();
    match inst {
        MirInst::Una64R(inst) if inst.opcode_is(MirOP::Mov64R) => {
            let dst = GPR64::from_real(inst.get_dst());
            let src = inst.get_src();
            let Some(vpos) = GPR64::try_from_real(src) else {
                return LowerPosAction::Keep;
            };
            let sp_offset = dsp + find_original_sp_offset(stack, vpos);
            if sp_offset == 0 {
                inst.set_src(GPR64::sp().into_real());
                return LowerPosAction::Keep;
            } else if let Some(sp_offset) = ImmCalc::try_new(sp_offset) {
                let add_inst = Bin64RC::new(MirOP::Add64I, dst, GPR64::sp(), sp_offset);
                insts.push_back(add_inst.into_mir());
            } else {
                let ldr_const64 =
                    LoadConst64::new(MirOP::LoadConst64, dst, Imm64(sp_offset, ImmKind::Full));
                insts.push_back(ldr_const64.into_mir());
                let add_inst = Bin64R::new(MirOP::Add64R, dst, GPR64::sp(), dst, None);
                insts.push_back(add_inst.into_mir());
            }
            LowerPosAction::Replace
        }
        MirInst::Bin64RC(inst) => {
            let opcode = inst.get_opcode();
            let lhs = GPR64::from_real(inst.get_rn());
            let sp_offset = dsp + find_original_sp_offset(stack, lhs);
            match opcode {
                MirOP::Add64I => {
                    lower_stackpos_operands_for_add64i(inst, sp_offset as u32, tmpreg, insts)
                }
                MirOP::Sub64I => {
                    lower_stackpos_operands_for_sub64i(inst, sp_offset as u32, tmpreg, insts)
                }
                _ => panic!(
                    "Unexpected opcode {opcode:?} for Bin64RC in lower_stackpos_operands_for_inst"
                ),
            }
        }
        MirInst::LoadGr32Base(inst) => {
            lower_stackpos_ldst::lower_stackpos_for_ldr32base(inst, insts, stack, tmpreg, dsp, sp)
        }
        MirInst::LoadGr64Base(inst) => {
            lower_stackpos_ldst::lower_stackpos_for_ldr64base(inst, insts, stack, tmpreg, dsp, sp)
        }
        MirInst::LdrSWBase(inst) => {
            lower_stackpos_ldst::lower_stackpos_for_ldrsw(inst, insts, stack, tmpreg, dsp, sp)
        }
        MirInst::LoadF32Base(inst) => {
            lower_stackpos_ldst::lower_stackpos_for_ldrf32base(inst, insts, stack, tmpreg, dsp, sp)
        }
        MirInst::LoadF64Base(inst) => {
            lower_stackpos_ldst::lower_stackpos_for_ldrf64base(inst, insts, stack, tmpreg, dsp, sp)
        }
        MirInst::StoreGr32Base(inst) => {
            lower_stackpos_ldst::lower_stackpos_for_str32base(inst, insts, stack, tmpreg, dsp, sp)
        }
        MirInst::StoreGr64Base(inst) => {
            // 这里要额外处理一种特殊情况: inst 的 `rd` 寄存器也是一个栈位置寄存器.
            lower_stackpos_ldst::lower_stackpos_for_str64base(inst, insts, stack, dsp, sp)
        }
        MirInst::StoreF32Base(inst) => {
            lower_stackpos_ldst::lower_stackpos_for_strf32base(inst, insts, stack, tmpreg, dsp, sp)
        }
        MirInst::StoreF64Base(inst) => {
            lower_stackpos_ldst::lower_stackpos_for_strf64base(inst, insts, stack, tmpreg, dsp, sp)
        }

        // 上面单独列出来的指令可以做一些特殊的优化, 这里统一处理其他情况下的操作数布局.
        _ => {
            lower_stackpos_operands_for_ordinary_inst(stack, insts, dsp, inst);
            LowerPosAction::InsertFront
        }
    }
}

/// 查找寄存器 `vreg` 在函数栈布局中的原始栈偏移量。
///
/// 这个 “原始栈偏移量” 是指函数在固定栈布局中为 `vreg` 分配的栈空间偏移量。当函数发生函数调用,
/// 做临时性栈空间调整时, 这个偏移量会被加上一个额外的偏移量, 以适应当前的栈布局。
pub fn find_original_sp_offset(stack_layout: &MirStackLayout, vreg: GPR64) -> u64 {
    let Some((kind, index)) = stack_layout.find_vreg_stackpos(vreg) else {
        panic!("Found non-stackpos vreg {vreg:?} which should be allocated in RegAlloc pass");
    };
    match kind {
        StackItemKind::Variable => stack_layout.vars[index].offset as u64,
        StackItemKind::SpilledArg => {
            let var_section_size: u64 = stack_layout.vars_size;
            let reg_section_size = stack_layout.saved_regs_section_size();
            var_section_size + reg_section_size + stack_layout.args[index].offset as u64
        }
        StackItemKind::SavedReg => panic!("Stackpos {vreg:?} should not be saved reg"),
    }
}

/// 处理一个 MIR 操作数, 将其转化为表示实际栈位置的寄存器. 同时增加必要的栈空间调整指令到 `insts_queue` 中。
pub(super) fn lower_stackpos_reg_for_operand(
    vpos: GPR64,
    tmpreg_alloc: &mut TmpRegAlloc,
    stack: &MirStackLayout,
    extra_delta_sp: u64,
    insts: &mut VecDeque<MirInst>,
) -> Option<GPR64> {
    match vpos.get_id() {
        RegID::StackPos(_) => {}
        RegID::Virt(_) => {
            panic!("Expected a stackpos or physical register, found virtual register {vpos:?}")
        }
        RegID::Invalid => panic!("Cannot lower stack position for Invalid register {vpos:?}"),
        _ => return None,
    }
    let vreg = vpos;
    let old_sp_offset = find_original_sp_offset(stack, vreg);
    let sp_offset = old_sp_offset + extra_delta_sp;
    let GPR64(id, _) = if sp_offset == 0 {
        GPR64::sp()
    } else if let Some(sp_offset) = ImmCalc::try_new(sp_offset) {
        let tmpreg = tmpreg_alloc.alloc();
        let add_inst = Bin64RC::new(MirOP::Add64I, tmpreg, GPR64::sp(), sp_offset);
        insts.push_back(add_inst.into_mir());
        tmpreg
    } else {
        let tmpreg = tmpreg_alloc.alloc();
        let ldr_const64 =
            LoadConst64::new(MirOP::LoadConst64, tmpreg, Imm64(sp_offset, ImmKind::Full));
        insts.push_back(ldr_const64.into_mir());
        let add_inst = Bin64R::new(MirOP::Add64R, tmpreg, GPR64::sp(), tmpreg, None);
        insts.push_back(add_inst.into_mir());
        tmpreg
    };
    let ret = GPR64(id, vreg.1);
    Some(ret)
}

fn lower_stackpos_operands_for_ordinary_inst(
    stack_layout: &MirStackLayout,
    stackpos_calc_queue: &mut VecDeque<MirInst>,
    extra_delta_sp: u64,
    inst: &MirInst,
) {
    let mut tmp_reg_alloc = TmpRegAlloc::new();
    for operand in inst.in_operands() {
        let MirOperand::GPReg(vpos) = operand.get() else {
            continue;
        };
        let Some(vpos) = GPR64::try_from_real(vpos) else {
            continue;
        };
        if let Some(new_vpos) = lower_stackpos_reg_for_operand(
            vpos,
            &mut tmp_reg_alloc,
            stack_layout,
            extra_delta_sp,
            stackpos_calc_queue,
        ) {
            operand.set(MirOperand::GPReg(new_vpos.into_real()));
        }
    }
}

#[allow(dead_code)]
fn collect_inst_gpregs(inst: &MirInst) -> Vec<GPR64> {
    let mut existing_gpregs =
        Vec::with_capacity(inst.in_operands().len() + inst.out_operands().len());
    for operand in inst.in_operands() {
        let MirOperand::GPReg(ppos) = operand.get() else {
            continue;
        };
        if !ppos.is_physical() {
            continue;
        }
        existing_gpregs.push(GPR64::new_raw(ppos.0));
    }
    for operand in inst.out_operands() {
        let MirOperand::GPReg(ppos) = operand.get() else {
            continue;
        };
        if !ppos.is_physical() || !ppos.get_use_flags().contains(RegUseFlags::USE) {
            continue;
        }
        existing_gpregs.push(GPR64::new_raw(ppos.0));
    }
    existing_gpregs
}

/// 处理指令 `add Xd, Xn, #imm`
///
/// #### 处理模型
///
/// 这里的 SP 指的都是已经向下调整过的栈指针. 在一般情况下, SP 上面只有“变量”“被调用者保存的寄存器”这两节;
/// 在函数调用时，SP 还会向下调整 —— 不过这个调整已经在之前的代码处理过了, 这里不用操心.
///
/// Remusys-MIR 没有专门表示栈空间某个位置指针的操作数, 考虑到优化过程中栈布局会随时变化, Remusys-MIR 复用了.
/// GPR64 虚拟寄存器作为指向栈空间某个位置的指针. 在 Remusys-MIR 的约定中, 表示栈空间位置的虚拟寄存器都隐含着
/// `SP + offset` 的语义.
///
/// * 原始: `add Xd, XPos, #rhs`
/// * 展开:
///     * `add Xd, (SP + sp_offset), #rhs`
///     * `add Xd, SP, #<rhs + sp_offset>`
///     * 直接 add 或者处理溢出后得到的一列指令
fn lower_stackpos_operands_for_add64i(
    inst: &Bin64RC,
    sp_offset: u32,
    tmpreg: GPR64,
    insts: &mut VecDeque<MirInst>,
) -> LowerPosAction {
    let rhs = inst.get_rm().0;
    let new_rhs = rhs + sp_offset as u32;
    if new_rhs == 0 {
        inst.set_rn(GPR64::sp().into_real());
        inst.set_rm(ImmCalc::new(0).into());
        LowerPosAction::Keep
    } else if let Some(new_rhs) = ImmCalc::try_new(new_rhs) {
        inst.set_rn(GPR64::sp().into_real());
        inst.set_rm(new_rhs.into());
        LowerPosAction::Keep
    } else {
        let ldr_const64 = LoadConst64::new(
            MirOP::LoadConst64,
            tmpreg,
            Imm64(new_rhs as u64, ImmKind::Full),
        );
        insts.push_back(ldr_const64.into_mir());
        let add_inst = Bin64R::new(
            MirOP::Add64R,
            GPR64::from_real(inst.get_rd()),
            GPR64::sp(),
            tmpreg,
            None,
        );
        insts.push_back(add_inst.into_mir());
        LowerPosAction::Replace
    }
}

/// 处理指令 `sub Xd, Xn, #imm`
///
/// #### 处理模型
///
/// 类似 `lower_stackpos_operands_for_add64i`. 但这里的 opcode 是 `sub`,
/// 因此需要重新推导偏移量模型.
///
/// * 原始: `sub Xd, XPos, #rhs`
/// * 展开:
///     * `sub Xd, (SP + sp_offset), #rhs`
///     * `sub Xd, SP, #<rhs - sp_offset>`
///     * 令 `new_offset = rhs - sp_offset`, 检查 `new_offset` 的符号:
///         * 为正: `sub Xd, SP, #new_offset`
///         * 为负: `add Xd, SP, #<-new_offset>`
///    * 直接改原指令或者处理溢出后得到的一列指令
fn lower_stackpos_operands_for_sub64i(
    inst: &Bin64RC,
    sp_offset: u32,
    tmpreg: GPR64,
    insts: &mut VecDeque<MirInst>,
) -> LowerPosAction {
    let rhs = inst.get_rm().0 as i64;
    let new_rhs = rhs - sp_offset as i64;

    if new_rhs == 0 {
        inst.set_rn(GPR64::sp().into_real());
        inst.set_rm(ImmCalc::new(0).into());
        LowerPosAction::Keep
    } else if new_rhs > 0 && imm_traits::is_calc_imm(new_rhs as u64) {
        let new_rhs = ImmCalc::new(new_rhs as u32);
        inst.set_rn(GPR64::sp().into_real());
        inst.set_rm(new_rhs.into());
        LowerPosAction::Keep
    } else if new_rhs < 0 && imm_traits::is_calc_imm(-new_rhs as u64) {
        let new_rhs = ImmCalc::new((-new_rhs) as u32);
        inst.set_rn(GPR64::sp().into_real());
        inst.set_rm(new_rhs.into());
        // 这里的操作数是负数, 需要把操作码改为加
        LowerPosAction::ReplaceOpcode(MirOP::Add64I)
    } else if new_rhs < 0 {
        // 处理溢出情况，需要根据 new_rhs 的符号选择指令
        // 负数：使用 add 指令
        let abs_new_rhs = (-new_rhs) as u64;
        let ldr_const64 = LoadConst64::new(
            MirOP::LoadConst64,
            tmpreg,
            Imm64(abs_new_rhs, ImmKind::Full),
        );
        insts.push_back(ldr_const64.into_mir());
        let add_inst = Bin64R::new(
            MirOP::Add64R,
            GPR64::from_real(inst.get_rd()),
            GPR64::sp(),
            tmpreg,
            None,
        );
        insts.push_back(add_inst.into_mir());
        LowerPosAction::Replace
    } else {
        // 处理溢出情况，需要根据 new_rhs 的符号选择指令
        // 正数：使用 sub 指令
        let ldr_const64 = LoadConst64::new(
            MirOP::LoadConst64,
            tmpreg,
            Imm64(new_rhs as u64, ImmKind::Full),
        );
        insts.push_back(ldr_const64.into_mir());
        let sub_inst = Bin64R::new(
            MirOP::Sub64R,
            GPR64::from_real(inst.get_rd()),
            GPR64::sp(),
            tmpreg,
            None,
        );
        insts.push_back(sub_inst.into_mir());
        LowerPosAction::Replace
    }
}

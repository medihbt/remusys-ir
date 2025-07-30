use crate::{
    base::slabref::SlabRef,
    mir::{
        inst::{IMirSubInst, MirInstRef, impls::*, inst::MirInst, opcode::MirOP},
        module::{MirModule, func::MirFunc, stack::MirStackLayout},
        operand::{MirOperand, imm::*, physreg_set::MirPhysRegSet, reg::*},
        translate::mirgen::operandgen::DispatchedReg,
        util::stack_adjust::MirSpAdjustTree,
    },
};
use std::collections::VecDeque;

/// Remusys 生成的汇编实在是太啰嗦了... 思来想去还是觉得, 应该对函数保存的寄存器剪剪枝了.
pub(crate) fn recalculate_func_saved_regs(
    func: &MirFunc,
    module: &mut MirModule,
    adj_tree: &MirSpAdjustTree,
) {
    let aapcs_callee_saved = MirPhysRegSet::new_aapcs_callee();
    let aapcs_caller_saved = MirPhysRegSet::new_aapcs_caller();
    let mut used_regs = MirPhysRegSet::new_empty();
    let mut defined_regs = MirPhysRegSet::new_empty();

    let allocs = module.allocs.get_mut();
    for (_, blocks) in func.blocks.view(&allocs.block) {
        for (_, inst) in blocks.insts.view(&allocs.inst) {
            if matches!(inst, MirInst::MirSaveRegs(_) | MirInst::MirRestoreRegs(_)) {
                // 如果是寄存器保存 / 恢复指令, 那么就不需要再处理了
                continue;
            }
            if let MirInst::BReg(breg) = inst {
                if breg.opcode_is(MirOP::Ret) && breg.get_target().get_id() == RegID::Phys(30) {
                    // 目前为了省事儿, X30 是当作如果是函数返回指令, 那么也不需要再处理了
                    continue;
                }
            }
            for operand in inst.in_operands() {
                match operand.get() {
                    MirOperand::GPReg(gpr) if gpr.is_physical() => {
                        used_regs.save_gpr(gpr.get_id());
                    }
                    MirOperand::VFReg(vfr) if vfr.is_physical() => {
                        used_regs.save_fpr(vfr.get_id());
                    }
                    _ => continue,
                }
            }
            for operand in inst.out_operands() {
                match operand.get() {
                    MirOperand::GPReg(gpr) if gpr.is_physical() => {
                        if gpr.get_use_flags().contains(RegUseFlags::USE) {
                            used_regs.save_gpr(gpr.get_id());
                        }
                        defined_regs.save_gpr(gpr.get_id());
                    }
                    MirOperand::VFReg(vfr) if vfr.is_physical() => {
                        if vfr.get_use_flags().contains(RegUseFlags::USE) {
                            used_regs.save_fpr(vfr.get_id());
                        }
                        defined_regs.save_fpr(vfr.get_id());
                    }
                    _ => continue,
                }
            }
        }
    }
    let func_saved = {
        let mut func_saved = defined_regs & aapcs_callee_saved;
        // X30 是函数返回地址寄存器, 需要被保存
        if func.has_call.get() {
            func_saved.save_gpr(RegID::Phys(30));
        }
        if func.get_name() == "main" {
            // main 函数需要保存 X29, 也就是帧指针寄存器
            func_saved.save_gpr(RegID::Phys(29));
        }

        func_saved
    };
    println!(
        "Rearranging saved regs for function {}: {aapcs_callee_saved:?} -> {func_saved:?}",
        func.get_name(),
    );
    func.reinit_saved_regs(func_saved);

    for (blockref, _) in func.blocks.view(&allocs.block) {
        let call_inst_saved = used_regs & aapcs_caller_saved;
        adj_tree.block_set_adjusted_regs(blockref.clone(), call_inst_saved, &allocs.inst);
    }
}

/// 为函数的入口生成栈空间预留和寄存器保存的指令模板。
///
/// #### 指令模板结构
///
/// Remusys 的栈空间布局约定已经在其他文件中说明了, 这里不再赘述。这些生成的指令
/// 按执行顺序分为下面这么几段:
///
/// * 保存被调用者该保存的寄存器
/// * 为局部变量开辟栈空间
///
/// #### 模板插入位置
///
/// 函数的开头, 在函数的第一条指令之前插入。
pub(crate) fn make_adjust_callee_sp_template(
    stack: &MirStackLayout,
    insts: &mut VecDeque<MirInst>,
) {
    let var_section_size = stack.vars_size;
    let reg_section_size = stack.saved_regs_section_size();
    let sp = GPR64::sp();

    if reg_section_size % 16 != 0 {
        panic!("Saved registers section size must be a multiple of 16, found: {reg_section_size}");
    }
    if var_section_size % 16 != 0 {
        panic!("Variable section size must be a multiple of 16, found: {var_section_size}");
    }

    // Step 1: 保存被调用者保存的寄存器段
    if reg_section_size > 0 {
        let delta_sp = ImmCalc::new(reg_section_size as u32);
        let subsp_inst = Bin64RC::new(MirOP::Sub64I, sp, sp, delta_sp);
        insts.push_back(subsp_inst.into_mir());
    }

    stack.foreach_saved_regs(|saved_reg, sp_offset| {
        let save_inst = match DispatchedReg::from_reg(saved_reg.preg) {
            DispatchedReg::F32(src) => {
                let offset = ImmLSP32::new(sp_offset as u32);
                StoreF32Base::new(MirOP::StrF32Base, src, sp, offset).into_mir()
            }
            DispatchedReg::F64(src) => {
                let offset = ImmLSP64::new(sp_offset);
                StoreF64Base::new(MirOP::StrF64Base, src, sp, offset).into_mir()
            }
            DispatchedReg::G32(src) => {
                let offset = ImmLSP32::new(sp_offset as u32);
                StoreGr32Base::new(MirOP::StrGr32Base, src, sp, offset).into_mir()
            }
            DispatchedReg::G64(src) => {
                let offset = ImmLSP64::new(sp_offset);
                StoreGr64Base::new(MirOP::StrGr64Base, src, sp, offset).into_mir()
            }
        };
        insts.push_back(save_inst);
    });

    // Step 2: 为局部变量开辟栈空间
    if var_section_size == 0 {
    } else if let Some(delta_sp) = ImmCalc::try_new(var_section_size as u64) {
        let subsp_inst = Bin64RC::new(MirOP::Sub64I, sp, sp, delta_sp);
        insts.push_back(subsp_inst.into_mir());
    } else {
        // 如果局部变量段太大了, 那么就需要使用临时寄存器来存储偏移量
        let tmpreg = GPR64::new(RegID::Phys(29));
        let ldr_const = LoadConst64::new(
            MirOP::LoadConst64,
            tmpreg,
            Imm64(var_section_size as u64, ImmKind::Full),
        );
        insts.push_back(ldr_const.into_mir());
        let subsp_inst = Bin64R::new(MirOP::Sub64R, sp, sp, tmpreg, None);
        insts.push_back(subsp_inst.into_mir());
    }
}

/// 生成函数入口的栈空间预留和寄存器保存指令模板, 并将其插入到函数的入口块中。
pub(crate) fn insert_entry_stack_adjustments(
    func: &MirFunc,
    module: &mut MirModule,
    stack_layout: &MirStackLayout,
    insts_queue: &mut VecDeque<MirInst>,
) {
    make_adjust_callee_sp_template(stack_layout, insts_queue);
    let allocs = module.allocs.get_mut();
    let entry_bb = func
        .blocks
        .get_front_ref(&allocs.block)
        .expect("Function must have at least one block");
    let focus_inst = {
        let insts = entry_bb.get_insts(&allocs.block);
        insts.get_front_ref(&allocs.inst).unwrap_or(insts._tail)
    };
    while let Some(inst) = insts_queue.pop_front() {
        let new_inst = MirInstRef::from_alloc(&mut allocs.inst, inst);
        entry_bb
            .get_insts(&allocs.block)
            .node_add_prev(&allocs.inst, focus_inst, new_inst)
            .expect("Failed to add new inst");
    }
}

/// 把函数 `func` 中的每条栈操作占位符指令——如寄存器保存 | 恢复、栈空间预留等, 转变成
/// 实际的栈操作指令并替换掉原来的占位符指令。
pub(crate) fn lower_stack_adjustment_insts(
    func: &MirFunc,
    module: &mut MirModule,
    insts_queue: &mut VecDeque<MirInst>,
    stack_layout: &MirStackLayout,
) {
    let allocs = module.allocs.get_mut();
    let to_lower = func.dump_insts_when(&allocs.block, &allocs.inst, |inst| {
        matches!(
            inst,
            MirInst::MirSaveRegs(_) | MirInst::MirRestoreRegs(_) | MirInst::MirRestoreHostRegs(..)
        )
    });
    for (block_ref, inst_ref) in to_lower {
        match inst_ref.to_data(&allocs.inst) {
            MirInst::MirSaveRegs(inst) => inst.dump_actions_template(insts_queue),
            MirInst::MirRestoreRegs(inst) => inst.dump_actions_template(insts_queue),
            MirInst::MirRestoreHostRegs(inst) => {
                inst.dump_template(insts_queue, stack_layout);
            }
            _ => continue,
        }
        while let Some(inst) = insts_queue.pop_front() {
            let new_inst_ref = MirInstRef::from_alloc(&mut allocs.inst, inst);
            block_ref
                .get_insts(&allocs.block)
                .node_add_prev(&allocs.inst, inst_ref, new_inst_ref)
                .expect("Failed to add new inst");
        }
        block_ref
            .get_insts(&allocs.block)
            .unplug_node(&allocs.inst, inst_ref)
            .expect("Failed to unplug old inst");
        allocs.inst.remove(inst_ref.get_handle());
    }
}

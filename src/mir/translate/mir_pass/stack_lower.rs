use crate::mir::{
    inst::{IMirSubInst, MirInstRef, impls::*, inst::MirInst, opcode::MirOP},
    module::{
        MirGlobal, MirModule,
        block::{MirBlock, MirBlockRef},
        func::MirFunc,
        stack::{MirStackLayout, SavedReg, StackItemKind},
    },
    operand::{
        IMirSubOperand,
        imm::*,
        imm_traits,
        reg::{GPR64, RegOperand, RegUseFlags},
    },
    translate::mirgen::operandgen::DispatchedReg,
};
use std::{cmp::Ordering, collections::VecDeque, rc::Rc};

pub fn lower_stack_for_module(module: &mut MirModule) {
    let mut all_funcs = Vec::new();
    for &globals in &module.items {
        let f = match &*globals.data_from_module(module) {
            MirGlobal::Function(f) if f.is_define() => Rc::clone(f),
            _ => continue,
        };
        all_funcs.push(f);
    }
    for func in &all_funcs {
        lower_function_stack(func, module);
    }
}

/// Remusys-MIR 栈布局指的是局部变量、函数参数、保存的寄存器在栈上的布局方式。
/// 在最终确定翻译到汇编之前，Remusys MIR 函数的栈布局会随时发生改变, 因此一开始
/// 不会马上生成栈空间预留 / 寄存器保存等相关的指令. 经过伪指令消除、寄存器分配等
/// 处理后, 栈布局最终会确定下来——也就是现在, 我们需要把维护在函数中的栈布局信息转化成
/// 对应的栈空间预留和寄存器保存指令。同时, 清空函数中的栈布局信息, 来表示“这个函数已经是
/// 一个汇编函数了, 接下来对它的所有改动都非法”。
pub fn lower_function_stack(func: &MirFunc, module: &MirModule) {
    if func.is_extern() {
        // Extern functions do not have a stack layout, so we can skip them.
        return;
    }
    let mut stack_layout = std::mem::take(&mut func.borrow_inner_mut().stack_layout);
    stack_layout.saved_regs.sort_by(saved_reg_cmp);

    // 首先确定栈空间预留和寄存器保存的指令模板.
    // 这两个模板后面在预留变量的时候还会滚雪球一样的加指令, 所以暂时不插进 MIR 中。
    let StackInfo {
        save_insts: asi,
        restore_insts: ari,
        section_size: asz,
    } = manage_callee_reg_stack_space(&mut stack_layout);

    // 然后预留局部变量的存储空间.
    let StackInfo {
        save_insts: vsi,
        restore_insts: vri,
        section_size: vsz,
    } = make_local_variable_layout(&mut stack_layout);

    let save_insts = {
        let mut save_insts = Vec::with_capacity(asi.len() + vsi.len());
        save_insts.extend(asi);
        save_insts.extend(vsi);
        save_insts
    };
    let restore_insts = {
        let mut restore_insts = Vec::with_capacity(ari.len() + vri.len());
        restore_insts.extend(vri);
        restore_insts.extend(ari);
        restore_insts
    };
    let this_frame_size = asz + vsz;

    // 模板已经准备好了, 现在把它们插入到函数的 MIR 中.
    apply_save_restore_templates(func, module, save_insts, restore_insts);

    // 接下来, 把所有表示栈空间位置的虚拟寄存器(因为这个 pass 在寄存器分配之后进行, 因此这也是最后留下的虚拟寄存器)
    // 替换成对应的 SP 偏移量.
    // 经过寄存器分配, 这种虚拟寄存器只有可能是指针了; 而且只读.
    let insts = find_maybe_stackpos_insts(func, module);

    // 在这些指令中, 遇到虚拟寄存器时, 尝试替换成 SP 偏移量.
    // 如果偏移量太大, 或者指令的操作数模式不允许使用偏移量, 就使用 X29 寄存器来保存偏移量。
    // X29 原来是帧指针, 但 SysY 不支持 alloca 这种动态分配栈空间的操作, 因此不需要帧指针。
    // X29 就当成一个专用的临时寄存器来使用, 在寄存器分配 pass 会刻意忽略它。
    for (block_ref, inst_ref) in insts {
        let mut added_insts: VecDeque<MirInst> = VecDeque::new();
        update_stack_instruction_refs(
            func,
            module,
            &stack_layout,
            this_frame_size,
            inst_ref,
            &mut added_insts,
        );
        while let Some(inst) = added_insts.pop_front() {
            let new_inst = MirInstRef::from_module(module, inst);
            let allocs = module.allocs.borrow();
            block_ref
                .get_insts(&allocs.block)
                .node_add_prev(&allocs.inst, inst_ref, new_inst)
                .expect("Failed to add new instruction");
        }
    }

    func.borrow_inner_mut().stack_layout = stack_layout;
}

fn update_stack_instruction_refs(
    func: &MirFunc,
    module: &MirModule,
    stack_layout: &MirStackLayout,
    this_frame_size: u64,
    inst_ref: MirInstRef,
    added_insts: &mut VecDeque<MirInst>,
) {
    match &*inst_ref.data_from_module(module) {
        MirInst::Bin64R(x) => {
            let stack_pos = GPR64::from_real(x.get_rn());
            let reg = make_regonly_target_reg(
                func,
                x.get_common().opcode,
                stack_layout,
                this_frame_size,
                added_insts,
                stack_pos,
            );
            x.set_rn(reg.into_real());
        }
        MirInst::Bin64RC(x) => {
            let stack_pos = GPR64::from_real(x.get_rn());
            let orig_offset = x.get_rm().0 as u64;
            let offset = get_stackpos_stack_offset(
                func,
                x.get_common().opcode,
                stack_layout,
                this_frame_size,
                stack_pos,
            );
            let offset = offset + orig_offset;
            let sp = GPR64::sp();
            let x29 = GPR64(29, RegUseFlags::empty());
            let (rn, imm) = if imm_traits::is_calc_imm(offset) {
                (sp, ImmCalc::new(offset as u32))
            } else {
                added_insts.push_back(
                    LoadConst64::new(MirOP::LoadConst64, x29, Imm64(offset, ImmKind::Full))
                        .into_mir(),
                );
                (x29, ImmCalc::new(0))
            };
            x.set_rn(rn.into_real());
            x.set_rm(imm);
        }
        MirInst::Una64R(x) => {
            let stack_pos = GPR64::from_real(x.get_src());
            let reg = make_regonly_target_reg(
                func,
                x.get_common().opcode,
                stack_layout,
                this_frame_size,
                added_insts,
                stack_pos,
            );
            x.set_src(reg.into_real());
        }
        MirInst::TenaryG64(x) => {
            let stack_pos = GPR64::from_real(x.get_rs());
            let reg = make_regonly_target_reg(
                func,
                x.get_common().opcode,
                stack_layout,
                this_frame_size,
                added_insts,
                stack_pos,
            );
            x.set_rs(reg.into_real());
        }

        MirInst::LoadGr64(ls) => {
            let stack_pos = GPR64::from_real(ls.get_rn());
            let reg = make_regonly_target_reg(
                func,
                ls.get_common().opcode,
                stack_layout,
                this_frame_size,
                added_insts,
                stack_pos,
            );
            ls.set_rn(reg.into_real());
        }
        MirInst::LoadGr32(ls) => {
            let stack_pos = GPR64::from_real(ls.get_rn());
            let reg = make_regonly_target_reg(
                func,
                ls.get_common().opcode,
                stack_layout,
                this_frame_size,
                added_insts,
                stack_pos,
            );
            ls.set_rn(reg.into_real());
        }
        MirInst::LoadF64(ls) => {
            let stack_pos = GPR64::from_real(ls.get_rn());
            let reg = make_regonly_target_reg(
                func,
                ls.get_common().opcode,
                stack_layout,
                this_frame_size,
                added_insts,
                stack_pos,
            );
            ls.set_rn(reg.into_real());
        }
        MirInst::LoadF32(ls) => {
            let stack_pos = GPR64::from_real(ls.get_rn());
            let reg = make_regonly_target_reg(
                func,
                ls.get_common().opcode,
                stack_layout,
                this_frame_size,
                added_insts,
                stack_pos,
            );
            ls.set_rn(reg.into_real());
        }
        MirInst::LoadGr64Base(ls) => {
            if ls.get_rn().is_virtual() {
                let stack_pos = GPR64::from_real(ls.get_rn());
                let orig_offset = ls.get_rm().0 as u64;
                let (rn, imm) = make_reg_imm_for_loadstore(
                    func,
                    ls.get_common().opcode,
                    stack_layout,
                    this_frame_size,
                    added_insts,
                    stack_pos,
                    orig_offset,
                    |x| imm_traits::is_load64_imm(x as i64),
                );
                ls.set_rn(rn.into_real());
                ls.set_rm(ImmLoad64(imm as i64));
            }
            if ls.get_rd().is_virtual() {
                let stack_pos = GPR64::from_real(ls.get_rd());
                let reg = make_regonly_target_reg(
                    func,
                    ls.get_common().opcode,
                    stack_layout,
                    this_frame_size,
                    added_insts,
                    stack_pos,
                );
                ls.set_rd(reg.into_real());
            }
        }
        MirInst::LoadGr32Base(ls) => {
            let stack_pos = GPR64::from_real(ls.get_rn());
            let orig_offset = ls.get_rm().0 as u64;
            let (rn, imm) = make_reg_imm_for_loadstore(
                func,
                ls.get_common().opcode,
                stack_layout,
                this_frame_size,
                added_insts,
                stack_pos,
                orig_offset,
                |x| imm_traits::is_load32_imm(x as i64),
            );
            ls.set_rn(rn.into_real());
            ls.set_rm(ImmLoad32(imm as i32));
        }
        MirInst::LoadF64Base(ls) => {
            let stack_pos = GPR64::from_real(ls.get_rn());
            let orig_offset = ls.get_rm().0 as u64;
            let (rn, imm) = make_reg_imm_for_loadstore(
                func,
                ls.get_common().opcode,
                stack_layout,
                this_frame_size,
                added_insts,
                stack_pos,
                orig_offset,
                |x| imm_traits::is_load64_imm(x as i64),
            );
            ls.set_rn(rn.into_real());
            ls.set_rm(ImmLoad64(imm as i64));
        }
        MirInst::LoadF32Base(ls) => {
            let stack_pos = GPR64::from_real(ls.get_rn());
            let orig_offset = ls.get_rm().0 as u64;
            let (rn, imm) = make_reg_imm_for_loadstore(
                func,
                ls.get_common().opcode,
                stack_layout,
                this_frame_size,
                added_insts,
                stack_pos,
                orig_offset,
                |x| imm_traits::is_load32_imm(x as i64),
            );
            ls.set_rn(rn.into_real());
            ls.set_rm(ImmLoad32(imm as i32));
        }
        MirInst::LoadGr64Indexed(ls) => {
            let stack_pos = GPR64::from_real(ls.get_rn());
            let orig_offset = ls.get_rm().0 as u64;
            let (rn, imm) = make_reg_imm_for_loadstore(
                func,
                ls.get_common().opcode,
                stack_layout,
                this_frame_size,
                added_insts,
                stack_pos,
                orig_offset,
                |x| imm_traits::is_load64_imm(x as i64),
            );
            ls.set_rn(rn.into_real());
            ls.set_rm(ImmLoad64(imm as i64));
        }
        MirInst::LoadGr32Indexed(ls) => {
            let stack_pos = GPR64::from_real(ls.get_rn());
            let orig_offset = ls.get_rm().0 as u64;
            let (rn, imm) = make_reg_imm_for_loadstore(
                func,
                ls.get_common().opcode,
                stack_layout,
                this_frame_size,
                added_insts,
                stack_pos,
                orig_offset,
                |x| imm_traits::is_load32_imm(x as i64),
            );
            ls.set_rn(rn.into_real());
            ls.set_rm(ImmLoad32(imm as i32));
        }
        MirInst::LoadF64Indexed(ls) => {
            let stack_pos = GPR64::from_real(ls.get_rn());
            let orig_offset = ls.get_rm().0 as u64;
            let (rn, imm) = make_reg_imm_for_loadstore(
                func,
                ls.get_common().opcode,
                stack_layout,
                this_frame_size,
                added_insts,
                stack_pos,
                orig_offset,
                |x| imm_traits::is_load64_imm(x as i64),
            );
            ls.set_rn(rn.into_real());
            ls.set_rm(ImmLoad64(imm as i64));
        }
        MirInst::LoadF32Indexed(ls) => {
            let stack_pos = GPR64::from_real(ls.get_rn());
            let orig_offset = ls.get_rm().0 as u64;
            let (rn, imm) = make_reg_imm_for_loadstore(
                func,
                ls.get_common().opcode,
                stack_layout,
                this_frame_size,
                added_insts,
                stack_pos,
                orig_offset,
                |x| imm_traits::is_load32_imm(x as i64),
            );
            ls.set_rn(rn.into_real());
            ls.set_rm(ImmLoad32(imm as i32));
        }
        MirInst::StoreGr64(ls) => {
            let stack_pos = GPR64::from_real(ls.get_rn());
            let reg = make_regonly_target_reg(
                func,
                ls.get_common().opcode,
                stack_layout,
                this_frame_size,
                added_insts,
                stack_pos,
            );
            ls.set_rn(reg.into_real());
        }
        MirInst::StoreGr32(ls) => {
            let stack_pos = GPR64::from_real(ls.get_rn());
            let reg = make_regonly_target_reg(
                func,
                ls.get_common().opcode,
                stack_layout,
                this_frame_size,
                added_insts,
                stack_pos,
            );
            ls.set_rn(reg.into_real());
        }
        MirInst::StoreF64(ls) => {
            let stack_pos = GPR64::from_real(ls.get_rn());
            let reg = make_regonly_target_reg(
                func,
                ls.get_common().opcode,
                stack_layout,
                this_frame_size,
                added_insts,
                stack_pos,
            );
            ls.set_rn(reg.into_real());
        }
        MirInst::StoreF32(ls) => {
            let stack_pos = GPR64::from_real(ls.get_rn());
            let reg = make_regonly_target_reg(
                func,
                ls.get_common().opcode,
                stack_layout,
                this_frame_size,
                added_insts,
                stack_pos,
            );
            ls.set_rn(reg.into_real());
        }
        MirInst::StoreGr64Base(ls) => {
            if ls.get_rn().is_virtual() {
                let stack_pos = GPR64::from_real(ls.get_rn());
                let orig_offset = ls.get_rm().0 as u64;
                let (rn, imm) = make_reg_imm_for_loadstore(
                    func,
                    ls.get_common().opcode,
                    stack_layout,
                    this_frame_size,
                    added_insts,
                    stack_pos,
                    orig_offset,
                    |x| imm_traits::is_load64_imm(x as i64),
                );
                ls.set_rn(rn.into_real());
                ls.set_rm(ImmLoad64(imm as i64));
            }
            if ls.get_rd().is_virtual() {
                let stack_pos = GPR64::from_real(ls.get_rd());
                let reg = make_regonly_target_reg(
                    func,
                    ls.get_common().opcode,
                    stack_layout,
                    this_frame_size,
                    added_insts,
                    stack_pos,
                );
                ls.set_rd(reg.into_real());
            }
        }
        MirInst::StoreGr32Base(ls) => {
            let stack_pos = GPR64::from_real(ls.get_rn());
            let orig_offset = ls.get_rm().0 as u64;
            let (rn, imm) = make_reg_imm_for_loadstore(
                func,
                ls.get_common().opcode,
                stack_layout,
                this_frame_size,
                added_insts,
                stack_pos,
                orig_offset,
                |x| imm_traits::is_load32_imm(x as i64),
            );
            ls.set_rn(rn.into_real());
            ls.set_rm(ImmLoad32(imm as i32));
        }
        MirInst::StoreF64Base(ls) => {
            let stack_pos = GPR64::from_real(ls.get_rn());
            let orig_offset = ls.get_rm().0 as u64;
            let (rn, imm) = make_reg_imm_for_loadstore(
                func,
                ls.get_common().opcode,
                stack_layout,
                this_frame_size,
                added_insts,
                stack_pos,
                orig_offset,
                |x| imm_traits::is_load64_imm(x as i64),
            );
            ls.set_rn(rn.into_real());
            ls.set_rm(ImmLoad64(imm as i64));
        }
        MirInst::StoreF32Base(ls) => {
            let stack_pos = GPR64::from_real(ls.get_rn());
            let orig_offset = ls.get_rm().0 as u64;
            let (rn, imm) = make_reg_imm_for_loadstore(
                func,
                ls.get_common().opcode,
                stack_layout,
                this_frame_size,
                added_insts,
                stack_pos,
                orig_offset,
                |x| imm_traits::is_load32_imm(x as i64),
            );
            ls.set_rn(rn.into_real());
            ls.set_rm(ImmLoad32(imm as i32));
        }
        MirInst::StoreGr64Indexed(ls) => {
            let stack_pos = GPR64::from_real(ls.get_rn());
            let orig_offset = ls.get_rm().0 as u64;
            let (rn, imm) = make_reg_imm_for_loadstore(
                func,
                ls.get_common().opcode,
                stack_layout,
                this_frame_size,
                added_insts,
                stack_pos,
                orig_offset,
                |x| imm_traits::is_load64_imm(x as i64),
            );
            ls.set_rn(rn.into_real());
            ls.set_rm(ImmLoad64(imm as i64));
        }
        MirInst::StoreGr32Indexed(ls) => {
            let stack_pos = GPR64::from_real(ls.get_rn());
            let orig_offset = ls.get_rm().0 as u64;
            let (rn, imm) = make_reg_imm_for_loadstore(
                func,
                ls.get_common().opcode,
                stack_layout,
                this_frame_size,
                added_insts,
                stack_pos,
                orig_offset,
                |x| imm_traits::is_load32_imm(x as i64),
            );
            ls.set_rn(rn.into_real());
            ls.set_rm(ImmLoad32(imm as i32));
        }
        MirInst::StoreF64Indexed(ls) => {
            let stack_pos = GPR64::from_real(ls.get_rn());
            let orig_offset = ls.get_rm().0 as u64;
            let (rn, imm) = make_reg_imm_for_loadstore(
                func,
                ls.get_common().opcode,
                stack_layout,
                this_frame_size,
                added_insts,
                stack_pos,
                orig_offset,
                |x| imm_traits::is_load64_imm(x as i64),
            );
            ls.set_rn(rn.into_real());
            ls.set_rm(ImmLoad64(imm as i64));
        }
        MirInst::StoreF32Indexed(ls) => {
            let stack_pos = GPR64::from_real(ls.get_rn());
            let orig_offset = ls.get_rm().0 as u64;
            let (rn, imm) = make_reg_imm_for_loadstore(
                func,
                ls.get_common().opcode,
                stack_layout,
                this_frame_size,
                added_insts,
                stack_pos,
                orig_offset,
                |x| imm_traits::is_load32_imm(x as i64),
            );
            ls.set_rn(rn.into_real());
            ls.set_rm(ImmLoad32(imm as i32));
        }
        _ => {}
    }
}

fn make_reg_imm_for_loadstore(
    func: &MirFunc,
    opcode: MirOP,
    stack_layout: &MirStackLayout,
    this_frame_size: u64,
    added_insts: &mut VecDeque<MirInst>,
    stack_pos: GPR64,
    orig_offset: u64,
    immload_judge: impl Fn(u64) -> bool,
) -> (GPR64, u64) {
    let offset = get_stackpos_stack_offset(func, opcode, stack_layout, this_frame_size, stack_pos);
    let offset = offset + orig_offset;
    let sp = GPR64::sp();
    let x29 = GPR64(29, RegUseFlags::empty());
    let (rn, imm) = if immload_judge(offset) {
        (sp, offset)
    } else if imm_traits::is_calc_imm(offset) {
        added_insts.push_back(
            Bin64RC::new(MirOP::Add64I, x29, sp, ImmCalc::new(offset as u32)).into_mir(),
        );
        (x29, 0)
    } else {
        added_insts.push_back(
            LoadConst64::new(MirOP::LoadConst64, x29, Imm64(offset, ImmKind::Full)).into_mir(),
        );
        added_insts.push_back(Bin64R::new(MirOP::Add64R, x29, sp, x29, None).into_mir());
        (x29, 0)
    };
    (rn, imm)
}

fn make_regonly_target_reg(
    func: &MirFunc,
    opcode: MirOP,
    stack_layout: &MirStackLayout,
    this_frame_size: u64,
    added_insts: &mut VecDeque<MirInst>,
    stack_pos: GPR64,
) -> GPR64 {
    let offset = get_stackpos_stack_offset(func, opcode, stack_layout, this_frame_size, stack_pos);
    let x29 = GPR64(29, RegUseFlags::empty());
    let sp = GPR64::sp();
    let reg = if offset == 0 {
        // 如果偏移量为 0, 则直接使用 SP 寄存器.
        sp
    } else if imm_traits::is_calc_imm(offset) {
        added_insts.push_back(
            Bin64RC::new(MirOP::Add64I, x29, sp, ImmCalc::new(offset as u32)).into_mir(),
        );
        x29
    } else {
        added_insts.push_back(
            LoadConst64::new(MirOP::LoadConst64, x29, Imm64(offset, ImmKind::Full)).into_mir(),
        );
        added_insts.push_back(Bin64R::new(MirOP::Add64R, x29, sp, x29, None).into_mir());
        x29
    };
    reg
}

fn get_stackpos_stack_offset(
    func: &MirFunc,
    opcode: MirOP,
    stack_layout: &MirStackLayout,
    this_frame_size: u64,
    stack_pos: GPR64,
) -> u64 {
    let (kind, idx) = match stack_layout.find_vreg_stackpos(stack_pos) {
        Some((kind, idx)) => (kind, idx as usize),
        _ => panic!(
            "Found non-stackpos vreg {stack_pos:?} in MirFunc {} MIR opcode {opcode:?}",
            func.get_name()
        ),
    };
    match kind {
        StackItemKind::Variable => stack_layout.vars[idx].offset as u64,
        StackItemKind::SpilledArg => stack_layout.args[idx].offset as u64 + this_frame_size,
        StackItemKind::SavedReg => panic!(
            "Found saved register vreg {stack_pos:?} in MirFunc {}",
            func.get_name()
        ),
    }
}

fn find_maybe_stackpos_insts(func: &MirFunc, module: &MirModule) -> Vec<(MirBlockRef, MirInstRef)> {
    func.dump_insts_with_module_when(module, |inst| match inst {
        MirInst::Bin64R(x) => x.get_rn().is_virtual() || x.get_rm().is_virtual(),
        MirInst::Bin64RC(x) => x.get_rn().is_virtual(),
        MirInst::Una64R(x) => x.get_src().is_virtual(),
        MirInst::TenaryG64(x) => x.get_rs().is_virtual(),
        MirInst::LoadGr64(ls) => ls.get_rn().is_virtual(),
        MirInst::StoreGr64(ls) => ls.get_rn().is_virtual(),
        MirInst::LoadGr32(ls) => ls.get_rn().is_virtual(),
        MirInst::StoreGr32(ls) => ls.get_rn().is_virtual(),
        MirInst::LoadF64(ls) => ls.get_rn().is_virtual(),
        MirInst::StoreF64(ls) => ls.get_rn().is_virtual(),
        MirInst::LoadF32(ls) => ls.get_rn().is_virtual(),
        MirInst::StoreF32(ls) => ls.get_rn().is_virtual(),
        // 因为栈地址都是指针, 只有 64 位 GPR 才能放下这个地址. 所以茫茫多的 LoadStore 变体中
        // 只有 64 位的 rd 才可能是栈位置.
        MirInst::LoadGr64Base(ls) => ls.get_rd().is_virtual() || ls.get_rn().is_virtual(),
        MirInst::StoreGr64Base(ls) => ls.get_rd().is_virtual() || ls.get_rn().is_virtual(),
        MirInst::LoadGr32Base(ls) => ls.get_rn().is_virtual(),
        MirInst::StoreGr32Base(ls) => ls.get_rn().is_virtual(),
        MirInst::LoadF64Base(ls) => ls.get_rn().is_virtual(),
        MirInst::StoreF64Base(ls) => ls.get_rn().is_virtual(),
        MirInst::LoadF32Base(ls) => ls.get_rn().is_virtual(),
        MirInst::StoreF32Base(ls) => ls.get_rn().is_virtual(),
        MirInst::LoadGr64Indexed(ls) => ls.get_rn().is_virtual(),
        MirInst::StoreGr64Indexed(ls) => ls.get_rn().is_virtual(),
        MirInst::LoadGr32Indexed(ls) => ls.get_rn().is_virtual(),
        MirInst::StoreGr32Indexed(ls) => ls.get_rn().is_virtual(),
        MirInst::LoadF64Indexed(ls) => ls.get_rn().is_virtual(),
        MirInst::StoreF64Indexed(ls) => ls.get_rn().is_virtual(),
        MirInst::LoadF32Indexed(ls) => ls.get_rn().is_virtual(),
        MirInst::StoreF32Indexed(ls) => ls.get_rn().is_virtual(),
        _ => false,
    })
}

fn apply_save_restore_templates(
    func: &MirFunc,
    module: &MirModule,
    save_insts: Vec<MirInst>,
    restore_insts: Vec<MirInst>,
) {
    let mut allocs = module.allocs.borrow_mut();
    let entry_save_block = match func.blocks.get_front_ref(&mut allocs.block) {
        Some(block_ref) => block_ref,
        None => {
            log::error!(
                "Error: Applied save & restore to an external function {}!",
                func.common.name
            );
            return;
        }
    };
    let (mut focus_inst, focus_block) = if entry_save_block.get_insts(&mut allocs.block).is_empty()
        || !entry_save_block.has_predecessors(&mut allocs.block)
    {
        // 如果入口块没有指令, 则直接在入口块前插入保存寄存器的指令。
        let head = entry_save_block.get_insts(&mut allocs.block)._head;
        (head, entry_save_block)
    } else {
        // 否则, 在入口块前插入一个新的块来保存寄存器。
        let new_bb = MirBlock::new(format!("{}.header", func.get_name()), &mut allocs.inst);
        let focus = new_bb.insts._head;
        let new_bb_ref = MirBlockRef::from_alloc(&mut allocs.block, new_bb);
        func.blocks
            .push_front_ref(&mut allocs.block, new_bb_ref)
            .expect("Failed to insert new block for register save in MirFunc");
        (focus, new_bb_ref)
    };

    // 在入口块前插入保存寄存器的指令。
    for inst in save_insts.iter() {
        let inst_ref = MirInstRef::from_alloc(&mut allocs.inst, inst.clone());
        match focus_block
            .get_insts(&allocs.block)
            .node_add_next(&allocs.inst, focus_inst, inst_ref)
        {
            Ok(()) => {}
            Err(err) => {
                panic!(
                    "Failed to add save instruction {:?} to block {} in MirFunc {}: {err:?}",
                    inst.get_opcode(),
                    focus_block.get_name(&allocs.block),
                    func.get_name(),
                )
            }
        }
        // 更新 focus_inst 为新插入的指令, 以便后续指令可以接在其后。
        focus_inst = inst_ref;
    }

    // 扫描函数的所有块, 找到所有 ret 指令的位置.
    let mut rets = Vec::new();
    for (block_ref, block) in func.blocks.view(&allocs.block) {
        for (inst_ref, inst) in block.insts.view(&allocs.inst) {
            if let MirInst::MirReturn(_) = inst {
                rets.push((block_ref, inst_ref));
            }
        }
    }

    let mut applied_restores = Vec::new();
    // 在每个 ret 指令前插入恢复寄存器的指令。
    for (ret_block, ret_inst) in rets {
        for inst in restore_insts.iter() {
            let inst_ref = MirInstRef::from_alloc(&mut allocs.inst, inst.clone());
            match ret_block
                .get_insts(&allocs.block)
                .node_add_prev(&allocs.inst, ret_inst, inst_ref)
            {
                Ok(()) => {}
                Err(err) => {
                    panic!(
                        "Failed to add restore instruction {:?} to block {} in MirFunc {}: {err:?}",
                        inst.get_opcode(),
                        ret_block.get_name(&allocs.block),
                        func.get_name(),
                    )
                }
            }
            applied_restores.push((ret_block, inst_ref));
        }
    }
}

struct StackInfo {
    save_insts: Vec<MirInst>,
    restore_insts: Vec<MirInst>,
    section_size: u64,
}

/// 生成预留 & 恢复栈空间的指令模板
fn manage_callee_reg_stack_space(stack_layout: &mut MirStackLayout) -> StackInfo {
    // 预留“保存寄存器”的栈空间
    let reserved_regs = &stack_layout.saved_regs;
    let (saved_regs, total_size) = make_regs_offset(reserved_regs);
    let mut reg_save_template = Vec::new();
    let mut reg_restore_template = Vec::new();

    // 保存模板: 添加一个 `sub sp, sp, #total_size` 指令
    let sp = GPR64::sp();
    reg_save_template.push(Bin64RC::new(MirOP::Sub64I, sp, sp, total_size).into_mir());
    for (reg, offset) in saved_regs {
        use DispatchedReg::*;
        use MirOP::{
            LdrF32Base, LdrF64Base, LdrGr32Base, LdrGr64Base, StrF32Base, StrF64Base, StrGr32Base,
            StrGr64Base,
        };
        let (save_inst, restore_inst) = match reg {
            F32(rd) => {
                let offset = ImmLoad32::new(offset as i32);
                (
                    StoreF32Base::new(StrF32Base, rd, sp, offset).into_mir(),
                    LoadF32Base::new(LdrF32Base, rd, sp, offset).into_mir(),
                )
            }
            F64(rd) => {
                let offset = ImmLoad64::new(offset as i64);
                (
                    StoreF64Base::new(StrF64Base, rd, sp, offset).into_mir(),
                    LoadF64Base::new(LdrF64Base, rd, sp, offset).into_mir(),
                )
            }
            G32(rd) => {
                let offset = ImmLoad32::new(offset as i32);
                (
                    StoreGr32Base::new(StrGr32Base, rd, sp, offset).into_mir(),
                    LoadGr32Base::new(LdrGr32Base, rd, sp, offset).into_mir(),
                )
            }
            G64(rd) => {
                let offset = ImmLoad64::new(offset as i64);
                (
                    StoreGr64Base::new(StrGr64Base, rd, sp, offset).into_mir(),
                    LoadGr64Base::new(LdrGr64Base, rd, sp, offset).into_mir(),
                )
            }
        };
        reg_save_template.push(save_inst);
        reg_restore_template.push(restore_inst);
    }
    // 恢复模板: 添加一个 `add sp, sp, #total_size` 指令
    reg_restore_template.push(Bin64RC::new(MirOP::Add64I, sp, sp, total_size).into_mir());

    // 返回预留和恢复栈空间的指令模板
    StackInfo {
        save_insts: reg_save_template,
        restore_insts: reg_restore_template,
        section_size: total_size.0 as u64,
    }
}

fn saved_reg_cmp(a: &SavedReg, b: &SavedReg) -> std::cmp::Ordering {
    let a_size = a.get_size_bytes();
    let b_size = b.get_size_bytes();
    match a_size.cmp(&b_size) {
        Ordering::Equal => {}
        ord => return ord,
    }
    let RegOperand(a_id, _, _, a_is_fp) = a.preg;
    let RegOperand(b_id, _, _, b_is_fp) = b.preg;
    match (a_is_fp, b_is_fp) {
        (true, false) => Ordering::Less,
        (false, true) => Ordering::Greater,
        _ => a_id.cmp(&b_id),
    }
}

/// 扫描所有保存的寄存器, 确定每个寄存器在栈“保存的寄存器”一节的偏移量，以及整个节的大小。
fn make_regs_offset(regs: &[SavedReg]) -> (Vec<(DispatchedReg, u64)>, ImmCalc) {
    let mut offset: u64 = 0;
    let mut result = Vec::with_capacity(regs.len());
    for reg in regs {
        let size = reg.get_size_bytes();
        offset = offset.next_multiple_of(size);
        result.push((DispatchedReg::from_reg(reg.preg), offset));
        offset += size;
    }
    let total_size = offset.next_multiple_of(8);
    (result, ImmCalc::new(total_size as u32))
}

fn make_local_variable_layout(stack_layout: &mut MirStackLayout) -> StackInfo {
    // 和前面一样, 先给栈布局排个序
    stack_layout.vars.sort_by_key(|i| i.size);
    // 然后计算每个变量的偏移量
    let mut offset: u64 = 0;
    for var in &mut stack_layout.vars {
        offset = offset.next_multiple_of(var.size);
        var.offset = offset as i64;
        offset += var.size;
    }
    let section_size = offset.next_multiple_of(8);
    stack_layout.vars_size = section_size;
    let mut reserve_insts = Vec::new();
    let mut restore_insts = Vec::new();

    if imm_traits::is_calc_imm(section_size) {
        // 如果栈空间大小是一个可以直接用立即数表示的数, 那么就直接生成预留和恢复栈空间的指令
        let sp = GPR64::sp();
        let rm = ImmCalc::new(section_size as u32);
        reserve_insts.push(Bin64RC::new(MirOP::Sub64I, sp, sp, rm).into_mir());
        restore_insts.push(Bin64RC::new(MirOP::Add64I, sp, sp, rm).into_mir());
    } else {
        // 否则, 需要先把栈空间大小存到一个寄存器中, 然后再生成预留和恢复栈空间的指令
        // 保存寄存器时的情况是: 作为被调用者已经保存了所有被调用者需要保存的寄存器. 找一个合适的寄存器来存放栈空间大小.
        // 这里找 X9 寄存器.
        let size_reg = GPR64(9, RegUseFlags::empty());
        let offset = Imm64(section_size, ImmKind::Full);
        let sp = GPR64::sp();
        let ldr_const = LoadConst64::new(MirOP::LoadConst64, size_reg, offset);
        let reserve_inst = Bin64R::new(MirOP::Sub64R, sp, sp, size_reg, None);
        reserve_insts.push(ldr_const.clone().into_mir());
        reserve_insts.push(reserve_inst.into_mir());

        // 恢复寄存器时的情况是: 函数结束了所有的控制流, 但调用者的寄存器尚未恢复.
        // 此时除了返回值寄存器以外所有寄存器都是空闲的. 找 X9 寄存器来恢复栈空间大小.
        let restore_inst = Bin64R::new(MirOP::Add64R, sp, sp, size_reg, None);
        restore_insts.push(ldr_const.into_mir());
        restore_insts.push(restore_inst.into_mir());
    }

    // 返回预留和恢复栈空间的指令模板
    StackInfo {
        save_insts: reserve_insts,
        restore_insts,
        section_size,
    }
}

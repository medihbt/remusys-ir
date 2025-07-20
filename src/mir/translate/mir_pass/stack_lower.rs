use std::cmp::Ordering;

use crate::mir::{
    inst::{IMirSubInst, MirInstRef, impls::*, inst::MirInst, opcode::MirOP},
    module::{
        MirModule,
        block::{MirBlock, MirBlockRef},
        func::MirFunc,
        stack::{MirStackLayout, SavedReg},
    },
    operand::{
        imm::*,
        imm_traits,
        reg::{GPR64, RegOperand, RegUseFlags},
    },
    translate::mirgen::operandgen::PureSourceReg,
};

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
    let (mut save, restore) = preserve_and_restore_stack_space(&stack_layout);

    // 然后预留局部变量的存储空间.
    let (reserve_vars, mut restore_vars) = make_local_variable_layout(&mut stack_layout);
    save.extend(reserve_vars); // 先保存寄存器再预留变量空间
    restore_vars.extend(restore); // 先恢复变量再恢复寄存器

    // 模板已经准备好了, 现在把它们插入到函数的 MIR 中.
    apply_save_restore_templates(func, module, save, restore_vars);

    // 接下来, 把所有表示栈空间位置的虚拟寄存器(因为这个 pass 在寄存器分配之后进行, 因此这也是最后留下的虚拟寄存器)
    // 替换成对应的 SP 偏移量.
    todo!("Replace all stack position virtual registers with SP offsets");
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

/// 生成预留 & 恢复栈空间的指令模板
fn preserve_and_restore_stack_space(stack_layout: &MirStackLayout) -> (Vec<MirInst>, Vec<MirInst>) {
    // 预留“保存寄存器”的栈空间
    let reserved_regs = &stack_layout.saved_regs;
    let (saved_regs, total_size) = make_regs_offset(reserved_regs);
    let mut reg_save_template = Vec::new();
    let mut reg_restore_template = Vec::new();

    // 保存模板: 添加一个 `sub sp, sp, #total_size` 指令
    let sp = GPR64::sp();
    reg_save_template.push(Bin64RC::new(MirOP::Sub64I, sp, sp, total_size).into_mir());
    for (reg, offset) in saved_regs {
        use MirOP::{
            LdrF32Base, LdrF64Base, LdrGr32Base, LdrGr64Base, StrF32Base, StrF64Base, StrGr32Base,
            StrGr64Base,
        };
        use PureSourceReg::*;
        let (save_inst, restore_inst) = match reg {
            F32(rd) => {
                let offset = ImmLoad32::new(offset as i32);
                (
                    LoadStoreF32Base::new(StrF32Base, rd, sp, offset).into_mir(),
                    LoadStoreF32Base::new(LdrF32Base, rd, sp, offset).into_mir(),
                )
            }
            F64(rd) => {
                let offset = ImmLoad64::new(offset as i64);
                (
                    LoadStoreF64Base::new(StrF64Base, rd, sp, offset).into_mir(),
                    LoadStoreF64Base::new(LdrF64Base, rd, sp, offset).into_mir(),
                )
            }
            G32(rd) => {
                let offset = ImmLoad32::new(offset as i32);
                (
                    LoadStoreGr32Base::new(StrGr32Base, rd, sp, offset).into_mir(),
                    LoadStoreGr32Base::new(LdrGr32Base, rd, sp, offset).into_mir(),
                )
            }
            G64(rd) => {
                let offset = ImmLoad64::new(offset as i64);
                (
                    LoadStoreGr64Base::new(StrGr64Base, rd, sp, offset).into_mir(),
                    LoadStoreGr64Base::new(LdrGr64Base, rd, sp, offset).into_mir(),
                )
            }
        };
        reg_save_template.push(save_inst);
        reg_restore_template.push(restore_inst);
    }
    // 恢复模板: 添加一个 `add sp, sp, #total_size` 指令
    reg_restore_template.push(Bin64RC::new(MirOP::Add64I, sp, sp, total_size).into_mir());

    // 返回预留和恢复栈空间的指令模板
    (reg_save_template, reg_restore_template)
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
fn make_regs_offset(regs: &[SavedReg]) -> (Vec<(PureSourceReg, u64)>, ImmCalc) {
    let mut offset: u64 = 0;
    let mut result = Vec::with_capacity(regs.len());
    for reg in regs {
        let size = reg.get_size_bytes();
        offset = offset.next_multiple_of(size);
        result.push((PureSourceReg::from_reg(reg.preg), offset));
        offset += size;
    }
    let total_size = offset.next_multiple_of(8);
    (result, ImmCalc::new(total_size as u32))
}

fn make_local_variable_layout(stack_layout: &mut MirStackLayout) -> (Vec<MirInst>, Vec<MirInst>) {
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

    (reserve_insts, restore_insts)
}

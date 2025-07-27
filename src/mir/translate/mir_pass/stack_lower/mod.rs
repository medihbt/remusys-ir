use crate::mir::{
    inst::MirInstRef,
    module::{MirGlobal, MirModule, func::MirFunc, stack::SavedReg},
    operand::reg::*,
    util::stack_adjust::MirSpAdjustTree,
};
use std::{
    cmp::Ordering,
    collections::{BTreeMap, VecDeque},
    rc::Rc,
};

mod lower_stackpos_ldst;
mod lower_stackpos_operand;
mod prepare_func_stack_layout;

pub fn lower_stack_for_module(module: &mut MirModule, mut adj_tree: MirSpAdjustTree) {
    let mut all_funcs = Vec::new();
    for &globals in &module.items {
        let f = match &*globals.data_from_module(module) {
            MirGlobal::Function(f) if f.is_define() => Rc::clone(f),
            _ => continue,
        };
        all_funcs.push(f);
    }

    for func in &all_funcs {
        prepare_func_stack_layout::recalculate_func_saved_regs(func, module, &mut adj_tree);
    }

    // 合并所有函数的寄存器保存区间
    adj_tree.merge_regsave_intervals_for_module(module);

    let sp_offset_map = adj_tree.make_offset_map(&module.allocs.get_mut().inst);
    for func in all_funcs {
        lower_function_stack(&func, module, &sp_offset_map);
    }
}

/// 为函数生成保存 / 恢复栈空间的指令, 然后替换函数内指令中所有与栈位置相关的操作数为实际的栈操作.
///
/// Remusys-MIR 栈布局指的是局部变量、函数参数、保存的寄存器在栈上的布局方式。
/// 在最终确定翻译到汇编之前，Remusys MIR 函数的栈布局会随时发生改变, 因此一开始
/// 不会马上生成栈空间预留 / 寄存器保存等相关的指令. 经过伪指令消除、寄存器分配等
/// 处理后, 栈布局最终会确定下来——也就是现在, 我们需要把维护在函数中的栈布局信息转化成
/// 对应的栈空间预留和寄存器保存指令。同时, 清空函数中的栈布局信息, 来表示“这个函数已经是
/// 一个汇编函数了, 接下来对它的所有改动都非法”。
fn lower_function_stack(
    func: &MirFunc,
    module: &mut MirModule,
    sp_offset_map: &BTreeMap<MirInstRef, u32>,
) {
    if func.is_extern() {
        // 外部函数连指令都没有, 直接跳过
        return;
    }

    let mut stack_layout = std::mem::take(&mut func.borrow_inner_mut().stack_layout);
    stack_layout.saved_regs.sort_by(saved_reg_cmp);

    let mut insts_queue = VecDeque::new();

    // 在函数入口为整个函数保存寄存器、开辟栈空间
    prepare_func_stack_layout::insert_entry_stack_adjustments(
        func,
        module,
        &stack_layout,
        &mut insts_queue,
    );

    // 处理函数内的指令, 将伪指令转换为实际的栈操作
    prepare_func_stack_layout::lower_stack_adjustment_insts(
        func,
        module,
        &mut insts_queue,
        &stack_layout,
    );

    // 处理函数内的每条指令, 把栈位置寄存器变为实际的栈位置. 这里会使用 X29 和 X28 作为统一的临时寄存器.
    let stackpos_calc_queue = &mut insts_queue;
    lower_stackpos_operand::lower_stackpos_operands_for_func(
        func,
        module,
        sp_offset_map,
        stack_layout,
        stackpos_calc_queue,
    );

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
}

#[derive(Debug, Clone)]
struct TmpRegAlloc(u8);

impl TmpRegAlloc {
    pub fn new() -> Self {
        TmpRegAlloc(29)
    }

    pub fn alloc(&mut self) -> GPR64 {
        let reg = GPR64::new_raw(self.0 as u32);
        self.0 -= 1;
        reg
    }
}

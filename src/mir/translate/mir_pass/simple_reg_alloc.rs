use crate::{
    base::slabref::SlabRef,
    mir::{
        inst::{IMirSubInst, MirInstRef, impls::*, inst::MirInst, opcode::MirOP},
        module::{
            MirGlobal, MirModule,
            block::{MirBlock, MirBlockRef},
            func::{MirFunc, MirFuncInner},
        },
        operand::{
            IMirSubOperand, MirOperand,
            imm::{ImmLoad32, ImmLoad64},
            reg::*,
        },
        translate::mirgen::operandgen::DispatchedReg,
    },
};
use slab::Slab;
use std::{cell::Cell, collections::VecDeque, rc::Rc};

/// 极其简单的寄存器分配算法 -- 每个虚拟寄存器都对应一个栈空间位置,
/// 所有带虚拟寄存器操作的指令都要配套一些 load and store 指令来实现。
///
/// ### 使用的寄存器
///
/// 下面的寄存器作为专用寄存器被占用.
///
/// * `X0-X7`, `D0-D7` 用于函数参数传递
/// * `X0` 用于函数返回值
///
/// 下面的寄存器作为临时寄存器被占用.
///
/// * `X8-X15`, `D8-D15`: 按照操作数分布分配
///
/// ### 如何表示栈空间引用
///
/// 在 aarch64 汇编中, 栈空间引用使用 `SP` 寄存器加上偏移量来表示. 但 Remusys-MIR 要
/// 做栈空间重布局优化, 所以栈空间引用仍然是一个虚拟寄存器——类型是 GPR64, 对应 IR 的指针类型.
/// 在 MIR 中遇到需要 spill 到栈上的寄存器时不会立即计算栈空间偏移量, 而是在局部变量栈布局表
/// 中注册一个栈空间位置并返回对应的虚拟寄存器. 这样可以在后续的栈布局优化中调整栈空间位置.
pub fn roughly_allocate_register(module: &mut MirModule) {
    let mut funcs = Vec::new();
    for &globals in &module.items {
        let f = match &*globals.data_from_module(module) {
            MirGlobal::Function(func) if func.is_define() => Rc::clone(func),
            _ => continue,
        };
        funcs.push(f);
    }
    for func in funcs {
        eprintln!(
            "..Roughly allocating registers for function {}...",
            func.get_name()
        );
        roughly_allocate_register_for_func(module, &func);
        eprintln!(
            "..Roughly allocated registers for function {}",
            func.get_name()
        );
    }
}

pub fn roughly_allocate_register_for_func(module: &mut MirModule, func: &MirFunc) {
    // 目前的实现是将所有虚拟寄存器都分配到栈上, 只保留函数参数寄存器和返回值寄存器.
    // 这只是一个占位符实现, 之后会单独开一个模块来实现更复杂的寄存器分配算法.
    let allocs = module.allocs.get_mut();
    let vreg_info = SpillVRegsResult::new(func, &allocs.block, &allocs.inst);

    eprintln!(
        "....Spilled {} virtual registers  to stack in function {}",
        vreg_info.stackpos_map.len(),
        func.get_name()
    );

    // 扫描所有关联指令, 替换所有虚拟寄存器为栈空间位置虚拟寄存器.
    let mut loads_before = VecDeque::new();
    let mut stores_after = VecDeque::new();
    for &(block_ref, inst_ref) in &vreg_info.relative_insts {
        let inst = inst_ref.to_slabref_unwrap(&allocs.inst);
        // 下面的寄存器作为操作数使用的临时寄存器, 需要在指令前后添加 load/store 指令.
        // * `X8-X15`, `D8-D15`: 按照操作数分布分配
        let mut curr_used_gpr = 8;
        let mut curr_used_fpr = 8;
        for operand in inst.in_operands() {
            let (ldr_inst, str_inst) =
                fetch_load_store_pair(&vreg_info, &mut curr_used_gpr, &mut curr_used_fpr, operand);
            if let Some(ldr) = ldr_inst {
                loads_before.push_back(ldr);
            }
            if let Some(str) = str_inst {
                stores_after.push_back(str);
            }
        }
        for operand in inst.out_operands() {
            let (ldr_inst, str_inst) =
                fetch_load_store_pair(&vreg_info, &mut curr_used_gpr, &mut curr_used_fpr, operand);
            if let Some(ldr) = ldr_inst {
                loads_before.push_back(ldr);
            }
            if let Some(str) = str_inst {
                stores_after.push_back(str);
            }
        }
        // 在原指令前添加 load 指令, 在原指令后添加 store 指令.
        while let Some(ldr) = loads_before.pop_front() {
            let new_inst = MirInstRef::from_alloc(&mut allocs.inst, ldr);
            block_ref
                .get_insts(&allocs.block)
                .node_add_prev(&allocs.inst, inst_ref, new_inst)
                .expect("Failed to add new load instruction");
        }
        while let Some(store) = stores_after.pop_back() {
            let new_inst = MirInstRef::from_alloc(&mut allocs.inst, store);
            block_ref
                .get_insts(&allocs.block)
                .node_add_next(&allocs.inst, inst_ref, new_inst)
                .expect("Failed to add new store instruction");
        }
    }
}

fn fetch_load_store_pair(
    vreg_info: &SpillVRegsResult,
    curr_used_gpr: &mut u32,
    curr_used_fpr: &mut u32,
    operand: &Cell<MirOperand>,
) -> (Option<MirInst>, Option<MirInst>) {
    let vreg = match operand.get() {
        MirOperand::GPReg(gpreg) if gpreg.is_virtual() => RegOperand::from(gpreg),
        MirOperand::VFReg(vfreg) if vfreg.is_virtual() => RegOperand::from(vfreg),
        _ => return (None, None),
    };
    let Some(stackpos) = vreg_info.find_stackpos(vreg) else {
        return (None, None);
    };
    let (ldr_inst, str_inst) =
        build_load_store_for_stackpos(curr_used_gpr, curr_used_fpr, operand, vreg, stackpos);
    let ldr_inst = if vreg.get_use_flags().contains(RegUseFlags::USE) {
        Some(ldr_inst)
    } else {
        None
    };
    let str_inst = if vreg.get_use_flags().contains(RegUseFlags::DEF) {
        Some(str_inst)
    } else {
        None
    };
    (ldr_inst, str_inst)
}

fn build_load_store_for_stackpos(
    curr_used_gpr: &mut u32,
    curr_used_fpr: &mut u32,
    operand: &Cell<MirOperand>,
    vreg: RegOperand,
    stackpos: GPR64,
) -> (MirInst, MirInst) {
    match DispatchedReg::from_reg(vreg) {
        DispatchedReg::F32(_) => {
            let mut fpr = FPR32(*curr_used_fpr, RegUseFlags::empty());
            *curr_used_fpr += 1;
            let imm0 = ImmLoad32::new(0);
            let ldr = LoadF32Base::new(MirOP::LdrF32Base, fpr, stackpos, imm0);
            let str = StoreF32Base::new(MirOP::StrF32Base, fpr, stackpos, imm0);
            fpr.1 = vreg.get_use_flags();
            operand.set(fpr.into_mir());
            (ldr.into_mir(), str.into_mir())
        }
        DispatchedReg::F64(_) => {
            let mut fpr = FPR64(*curr_used_fpr, RegUseFlags::empty());
            *curr_used_fpr += 1;
            let imm0 = ImmLoad64::new(0);
            let ldr = LoadF64Base::new(MirOP::LdrF64Base, fpr, stackpos, imm0);
            let str = StoreF64Base::new(MirOP::StrF64Base, fpr, stackpos, imm0);
            fpr.1 = vreg.get_use_flags();
            operand.set(fpr.into_mir());
            (ldr.into_mir(), str.into_mir())
        }
        DispatchedReg::G32(_) => {
            let mut gpr = GPR32(*curr_used_gpr, RegUseFlags::empty());
            *curr_used_gpr += 1;
            let imm0 = ImmLoad32::new(0);
            let ldr = LoadGr32Base::new(MirOP::LdrGr32Base, gpr, stackpos, imm0);
            let str = StoreGr32Base::new(MirOP::StrGr32Base, gpr, stackpos, imm0);
            gpr.1 = vreg.get_use_flags();
            operand.set(gpr.into_mir());
            (ldr.into_mir(), str.into_mir())
        }
        DispatchedReg::G64(_) => {
            let mut gpr = GPR64(*curr_used_gpr, RegUseFlags::empty());
            *curr_used_gpr += 1;
            let imm0 = ImmLoad64::new(0);
            let ldr = LoadGr64Base::new(MirOP::LdrGr64Base, gpr, stackpos, imm0);
            let str = StoreGr64Base::new(MirOP::StrGr64Base, gpr, stackpos, imm0);
            gpr.1 = vreg.get_use_flags();
            operand.set(gpr.into_mir());
            (ldr.into_mir(), str.into_mir())
        }
    }
}

struct SpillVRegsResult {
    stackpos_map: Vec<(RegOperand, GPR64)>,
    relative_insts: Vec<(MirBlockRef, MirInstRef)>,
}

impl SpillVRegsResult {
    fn new(func: &MirFunc, alloc_block: &Slab<MirBlock>, alloc_inst: &Slab<MirInst>) -> Self {
        let mut inner = func.borrow_inner_mut();
        // 大概需要映射这么多虚拟寄存器.
        // 实际上, 有一部分类型为 GPR64 的虚拟寄存器表示的是局部变量在栈帧上的位置,
        // 这部分虚拟寄存器不能做寄存器分配, 而需要在最后做栈空间分配时处理.
        let mut vregs: Vec<RegOperand> = Vec::new();
        let mut relative_insts = Vec::new();
        for (bref, block) in func.blocks.view(alloc_block) {
            for (iref, inst) in block.insts.view(alloc_inst) {
                // 只处理带虚拟寄存器的指令
                let mut has_vreg = false;
                for operand in inst.in_operands() {
                    has_vreg |= Self::try_add_vreg_operand(&inner, &mut vregs, operand.get());
                }
                for operand in inst.out_operands() {
                    has_vreg |= Self::try_add_vreg_operand(&inner, &mut vregs, operand.get());
                }
                if has_vreg {
                    relative_insts.push((bref, iref));
                }
            }
        }
        vregs.sort_by(|a, b| {
            let RegOperand(a_id, a_sub, _, a_is_fp) = a;
            let RegOperand(b_id, b_sub, _, b_is_fp) = b;
            a_is_fp
                .cmp(b_is_fp)
                .then(a_sub.get_bits_log2().cmp(&b_sub.get_bits_log2()))
                .then(a_id.cmp(&b_id))
        });
        let mut stackpos_map = Vec::with_capacity(vregs.len());
        for vreg in vregs {
            let MirFuncInner {
                stack_layout,
                vreg_alloc,
                ..
            } = &mut *inner;
            let stackpos_reg = {
                let stack_item = stack_layout.add_spilled_virtreg_variable(vreg, vreg_alloc);
                stack_item.stackpos_reg
            };
            stackpos_map.push((vreg, stackpos_reg));
        }
        Self {
            stackpos_map,
            relative_insts,
        }
    }

    fn find_stackpos(&self, vreg: RegOperand) -> Option<GPR64> {
        for &(key, stackpos) in self.stackpos_map.iter() {
            if key.same_pos_as(vreg) {
                return Some(stackpos);
            }
        }
        None
    }

    fn try_add_vreg_operand(
        inner: &MirFuncInner,
        vregs: &mut Vec<RegOperand>,
        operand: MirOperand,
    ) -> bool {
        let vreg = match operand {
            MirOperand::GPReg(gpreg) if gpreg.is_virtual() => {
                if gpreg.get_bits_log2() == 6
                    && inner.stack_layout.vreg_is_stackpos(GPR64::from_real(gpreg))
                {
                    // 如果是栈位置虚拟寄存器, 则不需要分配寄存器
                    return false;
                }
                RegOperand::from(gpreg)
            }
            MirOperand::VFReg(vfreg) if vfreg.is_virtual() => RegOperand::from(vfreg),
            _ => return false,
        };
        let bits_log2 = vreg.get_bits_log2();
        let mut found_duplicate = false;
        for pos in vregs.iter_mut() {
            if !pos.same_pos_as(vreg) {
                continue;
            }
            found_duplicate = true;
            if pos.get_bits_log2() < bits_log2 {
                pos.set_bits_log2(bits_log2);
            }
            break;
        }
        if !found_duplicate {
            vregs.push(vreg);
        }
        true
    }
}

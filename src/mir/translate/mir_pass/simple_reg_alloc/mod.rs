use crate::{
    base::slabref::SlabRef,
    mir::{
        inst::{IMirSubInst, MirInstRef, impls::*, inst::MirInst, opcode::MirOP},
        module::{
            MirGlobal, MirModule,
            block::{MirBlock, MirBlockRef},
            func::{MirFunc, MirFuncInner},
            stack::MirStackLayout,
        },
        operand::{
            IMirSubOperand, MirOperand,
            imm::{ImmLSP32, ImmLSP64},
            reg::*,
        },
        translate::mirgen::operandgen::DispatchedReg,
    },
};
use slab::Slab;
use std::{cell::Cell, collections::VecDeque, fmt::Debug, rc::Rc};

mod regalloc_lower_inst;
mod regalloc_lower_mir_constop;
mod regalloc_lower_mir_gep;
mod regalloc_lower_mir_ldrlit;
mod regalloc_lower_movs;

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
        let inst = inst_ref.to_data(&allocs.inst);
        let deletes_orig = regalloc_lower_inst::regalloc_lower_a_mir_inst(
            &vreg_info,
            &func.borrow_inner().stack_layout,
            &mut loads_before,
            &mut stores_after,
            inst,
        );
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
        if deletes_orig {
            // 处理“删除原指令”的情况.
            block_ref
                .get_insts(&allocs.block)
                .unplug_node(&allocs.inst, inst_ref)
                .expect("Failed to unplug original instruction");
            allocs.inst.remove(inst_ref.get_handle());
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
    let ldr_inst =
        if vreg.get_use_flags().contains(RegUseFlags::USE) { Some(ldr_inst) } else { None };
    let str_inst =
        if vreg.get_use_flags().contains(RegUseFlags::DEF) { Some(str_inst) } else { None };
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
            let imm0 = ImmLSP32::new(0);
            let ldr = LoadF32Base::new(MirOP::LdrF32Base, fpr, stackpos, imm0);
            let str = StoreF32Base::new(MirOP::StrF32Base, fpr, stackpos, imm0);
            fpr.1 = vreg.get_use_flags();
            operand.set(fpr.into_mir());
            (ldr.into_mir(), str.into_mir())
        }
        DispatchedReg::F64(_) => {
            let mut fpr = FPR64(*curr_used_fpr, RegUseFlags::empty());
            *curr_used_fpr += 1;
            let imm0 = ImmLSP64::new(0);
            let ldr = LoadF64Base::new(MirOP::LdrF64Base, fpr, stackpos, imm0);
            let str = StoreF64Base::new(MirOP::StrF64Base, fpr, stackpos, imm0);
            fpr.1 = vreg.get_use_flags();
            operand.set(fpr.into_mir());
            (ldr.into_mir(), str.into_mir())
        }
        DispatchedReg::G32(_) => {
            let mut gpr = GPR32(*curr_used_gpr, RegUseFlags::empty());
            *curr_used_gpr += 1;
            let imm0 = ImmLSP32::new(0);
            let ldr = LoadGr32Base::new(MirOP::LdrGr32Base, gpr, stackpos, imm0);
            let str = StoreGr32Base::new(MirOP::StrGr32Base, gpr, stackpos, imm0);
            gpr.1 = vreg.get_use_flags();
            operand.set(gpr.into_mir());
            (ldr.into_mir(), str.into_mir())
        }
        DispatchedReg::G64(_) => {
            let mut gpr = GPR64(*curr_used_gpr, RegUseFlags::empty());
            *curr_used_gpr += 1;
            let imm0 = ImmLSP64::new(0);
            let ldr = LoadGr64Base::new(MirOP::LdrGr64Base, gpr, stackpos, imm0);
            let str = StoreGr64Base::new(MirOP::StrGr64Base, gpr, stackpos, imm0);
            gpr.1 = vreg.get_use_flags();
            operand.set(gpr.into_mir());
            (ldr.into_mir(), str.into_mir())
        }
    }
}

pub struct SpillVRegsResult {
    stackpos_map: Vec<(RegOperand, GPR64)>,
    relative_insts: Vec<(MirBlockRef, MirInstRef)>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SpillVRegsError {
    NotFound,
    IsStackPos,
}

impl SpillVRegsResult {
    pub fn new(func: &MirFunc, alloc_block: &Slab<MirBlock>, alloc_inst: &Slab<MirInst>) -> Self {
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
            let MirFuncInner { stack_layout, vreg_alloc, .. } = &mut *inner;
            let stackpos_reg = {
                let stack_item = stack_layout.add_spilled_virtreg_variable(vreg, vreg_alloc);
                stack_item.stackpos_reg
            };
            stackpos_map.push((vreg, stackpos_reg));
        }
        Self { stackpos_map, relative_insts }
    }

    fn find_stackpos<T: Clone>(&self, vreg: T) -> Option<GPR64>
    where
        RegOperand: From<T>,
    {
        for &(key, stackpos) in self.stackpos_map.iter() {
            if key.same_pos_as(vreg.clone()) {
                return Some(stackpos);
            }
        }
        None
    }
    fn find_stackpos_full<T: Clone>(
        &self,
        stack: &MirStackLayout,
        vreg: T,
    ) -> Result<GPR64, SpillVRegsError>
    where
        RegOperand: From<T>,
    {
        for &(key, stackpos) in self.stackpos_map.iter() {
            if key.same_pos_as(vreg.clone()) {
                return Ok(stackpos);
            }
        }
        let vreg = RegOperand::from(vreg);
        let Some(vreg) = vreg.as_g64() else {
            return Err(SpillVRegsError::NotFound);
        };
        if stack.vreg_is_stackpos(vreg) {
            return Err(SpillVRegsError::IsStackPos);
        }
        Err(SpillVRegsError::NotFound)
    }
    fn findpos_full_unwrap<T: Clone>(&self, stack: &MirStackLayout, vreg: T) -> GPR64
    where
        RegOperand: From<T>,
    {
        let vregop = RegOperand::from(vreg.clone());
        self.find_stackpos_full(stack, vreg).unwrap_or_else(|e| {
            panic!("Virtual register {vregop:?} not found in stack layout: {e:?}")
        })
    }

    fn lower_gpr64(
        &self,
        stack: &MirStackLayout,
        reg: GPR64,
        loads_before: &mut VecDeque<MirInst>,
        tmpr_alloc: &mut SRATmpRegAlloc,
    ) -> GPR64 {
        if !reg.is_virtual() {
            return reg;
        }
        match self.find_stackpos_full(stack, reg) {
            Ok(stackpos) => {
                // 直接把基地址塞到返回的临时寄存器中, 接下来加减啥的方便.
                let tmpr = tmpr_alloc.alloc_gpr64();
                let ldr = LoadGr64Base::new(MirOP::LdrGr64Base, tmpr, stackpos, ImmLSP64(0));
                loads_before.push_back(ldr.into_mir());
                tmpr
            }
            Err(SpillVRegsError::IsStackPos) => reg,
            Err(SpillVRegsError::NotFound) => {
                panic!("Virtual register {reg:?} not found in stack layout");
            }
        }
    }

    fn lower_nonaddr_regs<T: Clone + Debug>(
        &self,
        stack: &MirStackLayout,
        reg: T,
        loads_before: &mut VecDeque<MirInst>,
        alloc_reg: impl FnOnce() -> T,
        make_inst: impl FnOnce(T, GPR64) -> MirInst,
    ) -> T
    where
        RegOperand: From<T>,
    {
        let oreg = RegOperand::from(reg.clone());
        if !oreg.is_virtual() {
            return reg;
        }
        match self.find_stackpos_full(stack, reg.clone()) {
            Ok(stackpos) => {
                // 直接把基地址塞到返回的临时寄存器中, 接下来加减啥的方便.
                let tmpr = alloc_reg();
                let ldr = make_inst(tmpr.clone(), stackpos);
                loads_before.push_back(ldr);
                tmpr
            }
            Err(SpillVRegsError::IsStackPos) => {
                panic!("Cannot lower a stack position register: {reg:?}");
            }
            Err(SpillVRegsError::NotFound) => {
                panic!("Virtual register {reg:?} not found in stack layout");
            }
        }
    }

    fn lower_gpr32(
        &self,
        stack: &MirStackLayout,
        reg: GPR32,
        loads_before: &mut VecDeque<MirInst>,
        tmpr_alloc: &mut SRATmpRegAlloc,
    ) -> GPR32 {
        self.lower_nonaddr_regs(
            stack,
            reg,
            loads_before,
            || tmpr_alloc.alloc_gpr32(),
            |tmpr, stackpos| {
                LoadGr32Base::new(MirOP::LdrGr32Base, tmpr, stackpos, ImmLSP32(0)).into_mir()
            },
        )
    }
    fn lower_fpr64(
        &self,
        stack: &MirStackLayout,
        reg: FPR64,
        loads_before: &mut VecDeque<MirInst>,
        tmpr_alloc: &mut SRATmpRegAlloc,
    ) -> FPR64 {
        self.lower_nonaddr_regs(
            stack,
            reg,
            loads_before,
            || tmpr_alloc.alloc_fpr64(),
            |tmpr, stackpos| {
                LoadF64Base::new(MirOP::LdrF64Base, tmpr, stackpos, ImmLSP64(0)).into_mir()
            },
        )
    }
    fn lower_fpr32(
        &self,
        stack: &MirStackLayout,
        reg: FPR32,
        loads_before: &mut VecDeque<MirInst>,
        tmpr_alloc: &mut SRATmpRegAlloc,
    ) -> FPR32 {
        self.lower_nonaddr_regs(
            stack,
            reg,
            loads_before,
            || tmpr_alloc.alloc_fpr32(),
            |tmpr, stackpos| {
                LoadF32Base::new(MirOP::LdrF32Base, tmpr, stackpos, ImmLSP32(0)).into_mir()
            },
        )
    }

    fn try_add_vreg_operand(
        inner: &MirFuncInner,
        vregs: &mut Vec<RegOperand>,
        operand: MirOperand,
    ) -> bool {
        let vreg = match operand {
            MirOperand::GPReg(gpreg) if Self::gpreg_can_alloc(gpreg, inner) => {
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

    /// 检查通用寄存器是否可以分配. 条件是:
    ///
    /// * 寄存器是虚拟寄存器
    /// * 寄存器不是栈空间位置寄存器
    fn gpreg_can_alloc(gpreg: GPReg, inner: &MirFuncInner) -> bool {
        if !gpreg.is_virtual() {
            return false;
        }
        let Some(gpreg) = GPR64::try_from_real(gpreg) else {
            // 只有 GPR64 才能作为栈空间位置寄存器.
            // 剩下的就是变量了, 可以分配到寄存器.
            return true;
        };
        if !inner.stack_layout.vreg_is_stackpos(gpreg) {
            return true;
        }
        let GPR64(_, uf) = gpreg;
        if uf.contains(RegUseFlags::DEF) {
            panic!("Cannot write to a stack position register: {gpreg:?}");
        }
        false
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct SRATmpRegAlloc(u8, u8);

impl SRATmpRegAlloc {
    pub fn new() -> Self {
        Self(8, 8) // 从 X9 + D9 开始分配临时寄存器
    }

    pub fn alloc_gpr64(&mut self) -> GPR64 {
        self.0 += 1;
        GPR64::new(RegID::Phys(self.0 as u32))
    }
    pub fn alloc_gpr32(&mut self) -> GPR32 {
        self.0 += 1;
        GPR32::new(RegID::Phys(self.0 as u32))
    }
    pub fn alloc_fpr64(&mut self) -> FPR64 {
        self.1 += 1;
        FPR64::new(RegID::Phys(self.0 as u32))
    }
    pub fn alloc_fpr32(&mut self) -> FPR32 {
        self.1 += 1;
        FPR32::new(RegID::Phys(self.0 as u32))
    }
}

//! clang 这样的汇编器似乎不支持 `ldr` 任意整数常量到寄存器的加载指令。
//! 这意味着我们需要将 `ldr` 指令转换为 `mov` 指令

use std::{collections::VecDeque, rc::Rc};

use slab::Slab;

use crate::{
    base::slabref::SlabRef,
    mir::{
        inst::{IMirSubInst, MirInstRef, impls::*, inst::MirInst, opcode::MirOP},
        module::{
            MirAllocs, MirGlobal, MirModule,
            block::{MirBlock, MirBlockRef},
        },
        operand::{IMirSubOperand, imm::*, imm_traits, reg::GPR64},
    },
};

/// 向下转换 `ldr` 指令，将常量加载到寄存器中。
///
/// #### 转换规则
///
/// * 如果立即数符合 `ImmCalc` 的要求，则转换为 `add <Rt>, ZR, #<imm>`
/// * 如果立即数符合 `ImmMov` 的要求，则转换为 `mov <Rt>, #<imm>`
/// * 否则, 尝试分段加载常量到寄存器中。
pub fn lower_ldr_const64(ldr_const64: &LoadConst64, out_insts: &mut VecDeque<MirInst>) {
    let target_reg = GPR64::from_real(ldr_const64.get_rd());
    let imm = ldr_const64.get_src().get_value();

    if imm == 0 {
        // 如果立即数是 0, 则直接使用 `mov <Rt>, ZR`
        let mov64 = Una64R::new(MirOP::Mov64R, target_reg, GPR64::zr(), None);
        out_insts.push_back(mov64.into_mir());
    } else if imm_traits::is_mov_imm(imm) {
        let mov64 = Mov64I::new(MirOP::Mov64I, target_reg, ImmMov::new(imm));
        out_insts.push_back(mov64.into_mir());
    } else if let Some(imm_movznk) = ImmMovZNK::try_from_u64(!imm) {
        let movznk = MovZNK64::new(MirOP::MovN64, target_reg, imm_movznk);
        out_insts.push_back(movznk.into_mir());
    } else {
        let imm_parts = [
            (imm & 0xFFFF) as u16,
            ((imm >> 16) & 0xFFFF) as u16,
            ((imm >> 32) & 0xFFFF) as u16,
            (imm >> 48) as u16,
        ];

        struct MovZKStat(MirOP);
        impl MovZKStat {
            fn new() -> Self {
                MovZKStat(MirOP::MovZ64)
            }
            fn get(&mut self) -> MirOP {
                let op = self.0;
                self.0 = MirOP::MovK64; // 切换到 MovK64
                op
            }
        }

        // 分段加载常量到寄存器中
        let mut stat = MovZKStat::new();
        if imm_parts[0] != 0 {
            let mov0 = MovZNK64::new(stat.get(), target_reg, ImmMovZNK(imm_parts[0], 0));
            out_insts.push_back(mov0.into_mir());
        }
        if imm_parts[1] != 0 {
            let mov1 = MovZNK64::new(stat.get(), target_reg, ImmMovZNK(imm_parts[1], 16));
            out_insts.push_back(mov1.into_mir());
        }
        if imm_parts[2] != 0 {
            let mov2 = MovZNK64::new(stat.get(), target_reg, ImmMovZNK(imm_parts[2], 32));
            out_insts.push_back(mov2.into_mir());
        }
        if imm_parts[3] != 0 {
            let mov3 = MovZNK64::new(stat.get(), target_reg, ImmMovZNK(imm_parts[3], 48));
            out_insts.push_back(mov3.into_mir());
        }
    }
}

/// 向下转换 `ldr` 指令，将符号所示的地址加载到寄存器中。
///
/// #### 转换规则
///
/// * 使用 `adrp` 指令加载符号的高位地址到寄存器中。
/// * 使用 `add` 指令将符号的低位地址加到寄存器中。
pub fn lower_ldr_symbol(ldr_symbol: &LoadConst64Symbol, out_insts: &mut VecDeque<MirInst>) {
    let target_reg = GPR64::from_real(ldr_symbol.get_rd());
    let symbol = ldr_symbol.get_src();

    let adrp = Adr::new(MirOP::AdrP, target_reg, symbol);
    let adds = Bin64RSym::new(MirOP::Add64Sym, target_reg, target_reg, symbol);
    out_insts.push_back(adrp.into_mir());
    out_insts.push_back(adds.into_mir());
}

fn preasm_pass_for_inst(inst: &MirInst, out_insts: &mut VecDeque<MirInst>) {
    match inst {
        MirInst::LoadConst64(ldr_const64) => lower_ldr_const64(ldr_const64, out_insts),
        MirInst::LoadConst64Symbol(ldr_symbol) => lower_ldr_symbol(ldr_symbol, out_insts),
        _ => {}
    }
}

/// 预汇编 pass.
///
/// 诸如 LLVM 这样的汇编器不支持某些伪指令, 因此在这里消除它们.
pub fn preasm_pass(
    insts: &[(MirBlockRef, MirInstRef)],
    alloc_bb: &Slab<MirBlock>,
    alloc_inst: &mut Slab<MirInst>,
) {
    let mut out_insts = VecDeque::new();
    for &(block_ref, inst_ref) in insts {
        preasm_pass_for_inst(inst_ref.to_slabref_unwrap(alloc_inst), &mut out_insts);
        while let Some(inst) = out_insts.pop_front() {
            let new_inst_ref = MirInstRef::from_alloc(alloc_inst, inst);
            block_ref
                .get_insts(alloc_bb)
                .node_add_prev(alloc_inst, inst_ref, new_inst_ref)
                .expect("Failed to add new inst");
        }
        block_ref
            .get_insts(alloc_bb)
            .unplug_node(alloc_inst, inst_ref)
            .expect("Failed to unplug old inst");
    }
}

pub fn preasm_pass_for_module(module: &mut MirModule) {
    let mut all_funcs = Vec::new();
    for &globals in &module.items {
        let f = match &*globals.data_from_module(module) {
            MirGlobal::Function(f) if f.is_define() => Rc::clone(f),
            _ => continue,
        };
        all_funcs.push(f);
    }
    for func in all_funcs {
        let insts_to_process = func.dump_insts_with_module_when(module, |inst| {
            matches!(
                inst,
                MirInst::LoadConst64(_) | MirInst::LoadConst64Symbol(_)
            )
        });
        let allocs = module.allocs.get_mut();
        let MirAllocs {
            block: alloc_block,
            inst: alloc_inst,
            ..
        } = &mut *allocs;
        preasm_pass(&insts_to_process, alloc_block, alloc_inst);
    }
}

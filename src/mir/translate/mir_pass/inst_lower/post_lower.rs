use std::collections::VecDeque;

use crate::{
    base::SlabRef,
    mir::{
        inst::{IMirSubInst, MirInstRef, impls::*, inst::MirInst, opcode::MirOP},
        module::{MirModule, func::MirFunc},
        operand::{
            IMirSubOperand,
            compound::MirSymbolOp,
            imm::{ImmFMov32, ImmFMov64, ImmLSP32, ImmLSP64},
            reg::*,
        },
    },
};

enum PostLowerAction {
    /// 把指令 MirInst 添加到当前指令的前面. 注意这点和 Rust 容器的 push_front 语义刚好
    /// 相反, 因为 Rust 双端队列是一个 `[a, b]` 的区间, 这里把指令列表可以当成 `(-∞, a], [b, +∞)` 的区间.
    ///
    /// 通过几次 PushFront 操作得到的指令序列仍然保持原有的顺序, 例如下面的基本块:
    ///
    /// ```
    /// ... -> I1 -> I2 -> *I3 -> I4 -> ...
    /// (*) 表示焦点.
    /// ```
    ///
    /// 以 I3 为焦点, 执行动作:
    ///
    /// ```
    /// PushFront(J1)
    /// PushFront(J2)
    /// PushFront(J3)
    /// ```
    ///
    /// 得到的指令序列为:
    ///
    /// ```
    /// ... -> I1 -> I2 -> J1 -> J2 -> J3 -> *I3 -> I4 -> ...
    /// ```
    PushFront(MirInst),
    DeleteThis,
}

fn lower_mir_ldrlit_g64(inst: &MirLdrLitG64, actions: &mut VecDeque<PostLowerAction>) {
    use PostLowerAction::*;

    let dst = GPR64::from_real(inst.get_dst());
    let src = inst.get_src();
    let addr = GPR64::from_real(inst.get_tmp_addr());

    assert!(addr.is_general_physical());
    assert!(dst.is_physical());

    let adr_inst = Adr::new(MirOP::AdrP, addr, src);
    actions.push_back(PushFront(adr_inst.into_mir()));

    let ldr_inst = LoadGr64BaseS::new(MirOP::LdrGr64BaseS, dst, addr, src);
    actions.push_back(PushFront(ldr_inst.into_mir()));
    actions.push_back(DeleteThis);
}

fn lower_mir_ldrlit_g32(inst: &MirLdrLitG32, actions: &mut VecDeque<PostLowerAction>) {
    use PostLowerAction::*;

    let dst = GPR32::from_real(inst.get_dst());
    let src = inst.get_src();
    let addr = GPR64::from_real(inst.get_tmp_addr());

    assert!(addr.is_general_physical());
    assert!(dst.is_physical());

    let adr_inst = Adr::new(MirOP::AdrP, addr, src);
    actions.push_back(PushFront(adr_inst.into_mir()));
    let ldr_inst = LoadGr32BaseS::new(MirOP::LdrGr32BaseS, dst, addr, src);
    actions.push_back(PushFront(ldr_inst.into_mir()));
    actions.push_back(DeleteThis);
}

fn lower_mir_ldrlit_f64(inst: &MirLdrLitF64, actions: &mut VecDeque<PostLowerAction>) {
    use PostLowerAction::*;

    let dst = FPR64::from_real(inst.get_dst());
    let src = inst.get_src();
    let addr = GPR64::from_real(inst.get_tmp_addr());

    assert!(addr.is_general_physical());
    assert!(dst.is_physical());

    let adr_inst = Adr::new(MirOP::AdrP, addr, src);
    actions.push_back(PushFront(adr_inst.into_mir()));
    let ldr_inst = LoadF64BaseS::new(MirOP::LdrF64BaseS, dst, addr, src);
    actions.push_back(PushFront(ldr_inst.into_mir()));
    actions.push_back(DeleteThis);
}

fn lower_mir_ldrlit_f32(inst: &MirLdrLitF32, actions: &mut VecDeque<PostLowerAction>) {
    use PostLowerAction::*;

    let dst = FPR32::from_real(inst.get_dst());
    let src = inst.get_src();
    let addr = GPR64::from_real(inst.get_tmp_addr());

    assert!(addr.is_general_physical());
    assert!(dst.is_physical());

    let adr_inst = Adr::new(MirOP::AdrP, addr, src);
    actions.push_back(PushFront(adr_inst.into_mir()));
    let ldr_inst = LoadF32BaseS::new(MirOP::LdrF32BaseS, dst, addr, src);
    actions.push_back(PushFront(ldr_inst.into_mir()));
    actions.push_back(DeleteThis);
}

fn lower_mir_strlit_g64(inst: &MirStrLitG64, actions: &mut VecDeque<PostLowerAction>) {
    use PostLowerAction::*;

    let src = GPR64::from_real(inst.get_rd());
    let dst = inst.get_to();
    let addr = GPR64::from_real(inst.get_tmp_addr());

    assert!(addr.is_general_physical());
    assert!(src.is_physical());

    let adr_inst = Adr::new(MirOP::AdrP, addr, dst);
    actions.push_back(PushFront(adr_inst.into_mir()));

    let str_inst = StoreGr64BaseS::new(MirOP::StrGr64BaseS, src, addr, dst);
    actions.push_back(PushFront(str_inst.into_mir()));
    actions.push_back(DeleteThis);
}

fn lower_mir_strlit_g32(inst: &MirStrLitG32, actions: &mut VecDeque<PostLowerAction>) {
    use PostLowerAction::*;

    let src = GPR32::from_real(inst.get_rd());
    let dst = inst.get_to();
    let addr = GPR64::from_real(inst.get_tmp_addr());

    assert!(addr.is_general_physical(), "inst {inst:#?}");
    assert!(!src.is_virtual(), "inst {inst:#?}");

    let adr_inst = Adr::new(MirOP::AdrP, addr, dst);
    actions.push_back(PushFront(adr_inst.into_mir()));
    let str_inst = StoreGr32BaseS::new(MirOP::StrGr32BaseS, src, addr, dst);
    actions.push_back(PushFront(str_inst.into_mir()));
    actions.push_back(DeleteThis);
}

fn lower_mir_strlit_f64(inst: &MirStrLitF64, actions: &mut VecDeque<PostLowerAction>) {
    use PostLowerAction::*;

    let src = FPR64::from_real(inst.get_rd());
    let dst = inst.get_to();
    let addr = GPR64::from_real(inst.get_tmp_addr());

    assert!(addr.is_general_physical());
    assert!(!src.is_virtual());

    let adr_inst = Adr::new(MirOP::AdrP, addr, dst);
    actions.push_back(PushFront(adr_inst.into_mir()));
    let str_inst = StoreF64BaseS::new(MirOP::StrF64BaseS, src, addr, dst);
    actions.push_back(PushFront(str_inst.into_mir()));
    actions.push_back(DeleteThis);
}

fn lower_mir_strlit_f32(inst: &MirStrLitF32, actions: &mut VecDeque<PostLowerAction>) {
    use PostLowerAction::*;

    let src = FPR32::from_real(inst.get_rd());
    let dst = inst.get_to();
    let addr = GPR64::from_real(inst.get_tmp_addr());

    assert!(addr.is_general_physical());
    assert!(!src.is_virtual());

    let adr_inst = Adr::new(MirOP::AdrP, addr, dst);
    actions.push_back(PushFront(adr_inst.into_mir()));
    let str_inst = StoreF32BaseS::new(MirOP::StrF32BaseS, src, addr, dst);
    actions.push_back(PushFront(str_inst.into_mir()));
    actions.push_back(DeleteThis);
}

fn lower_ldimm_f32(inst: &MirLdImmF32, actions: &mut VecDeque<PostLowerAction>) {
    use PostLowerAction::*;

    let dst = FPR32::from_real(inst.get_rd());
    let imm = inst.get_src();

    if let Some(src) = ImmFMov32::try_from_real(imm.zext_to_imm64()) {
        let fmov_inst = FMov32I::new(MirOP::FMov32I, dst, src);
        actions.push_back(PushFront(fmov_inst.into_mir()));
        actions.push_back(DeleteThis);
    } else {
        let tmpreg = GPR64::from_real(inst.get_tmpreg());
        assert!(tmpreg.is_general_physical());
        let ldrconst = LoadConst64::new(MirOP::LoadConst64, tmpreg, imm.zext_to_imm64());
        let fmovinst = UnaFG32::new(MirOP::FMovFG32, dst, tmpreg.trunc_to_gpr32());
        actions.push_back(PushFront(ldrconst.into_mir()));
        actions.push_back(PushFront(fmovinst.into_mir()));
        actions.push_back(DeleteThis);
    }
}

fn lower_ldimm_f64(inst: &MirLdImmF64, actions: &mut VecDeque<PostLowerAction>) {
    use PostLowerAction::*;

    let dst = FPR64::from_real(inst.get_rd());
    let imm = inst.get_src();

    if let Some(src) = ImmFMov64::try_from_real(imm) {
        let fmov_inst = FMov64I::new(MirOP::FMov64I, dst, src);
        actions.push_back(PushFront(fmov_inst.into_mir()));
        actions.push_back(DeleteThis);
    } else {
        let tmpreg = GPR64::from_real(inst.get_tmpreg());
        assert!(tmpreg.is_general_physical());
        let ldrconst = LoadConst64::new(MirOP::LoadConst64, tmpreg, imm);
        let fmovinst = UnaFG64::new(MirOP::FMovFG64, dst, tmpreg);
        actions.push_back(PushFront(ldrconst.into_mir()));
        actions.push_back(PushFront(fmovinst.into_mir()));
        actions.push_back(DeleteThis);
    }
}

fn lower_stimm32(inst: &MirStImm32, actions: &mut VecDeque<PostLowerAction>) {
    use PostLowerAction::*;

    let imm = inst.get_imm();
    let tmpreg = GPR64::from_real(inst.get_tmpreg());

    let base = GPR64::from_real(inst.get_base());
    let offset = inst.get_offset();

    if imm.0 == 0 {
        let store_wzr = StoreGr32Base::new(MirOP::StrGr32Base, GPR32::zr(), base, offset);
        actions.push_back(PushFront(store_wzr.into_mir()));
        actions.push_back(DeleteThis);
    } else {
        assert_ne!(
            tmpreg.get_id(),
            RegID::Invalid,
            "stimm32 {inst:#?} with invalid tmpreg"
        );
        let load_const = LoadConst64::new(MirOP::LoadConst64, tmpreg, imm.zext_to_imm64());
        let store_reg =
            StoreGr32Base::new(MirOP::StrGr32Base, tmpreg.trunc_to_gpr32(), base, offset);
        actions.push_back(PushFront(load_const.into_mir()));
        actions.push_back(PushFront(store_reg.into_mir()));
        actions.push_back(DeleteThis);
    }
}

fn lower_stimm64(inst: &MirStImm64, actions: &mut VecDeque<PostLowerAction>) {
    use PostLowerAction::*;

    let imm = inst.get_imm();
    let tmpreg = GPR64::from_real(inst.get_tmpreg());
    assert_ne!(
        tmpreg.get_id(),
        RegID::Invalid,
        "stimm64 {inst:#?} with invalid tmpreg"
    );

    let base = GPR64::from_real(inst.get_base());
    let offset = inst.get_offset();

    if imm.0 == 0 {
        let store_wzr = StoreGr64Base::new(MirOP::StrGr64Base, GPR64::zr(), base, offset);
        actions.push_back(PushFront(store_wzr.into_mir()));
        actions.push_back(DeleteThis);
    } else {
        let load_const = LoadConst64::new(MirOP::LoadConst64, tmpreg, imm);
        let store_reg = StoreGr64Base::new(MirOP::StrGr64Base, tmpreg, base, offset);
        actions.push_back(PushFront(load_const.into_mir()));
        actions.push_back(PushFront(store_reg.into_mir()));
        actions.push_back(DeleteThis);
    }
}

fn lower_stsym64(inst: &MirStSym64, actions: &mut VecDeque<PostLowerAction>) {
    use PostLowerAction::*;

    let sym = inst.get_imm();
    let tmpreg = GPR64::from_real(inst.get_tmpreg());
    assert_ne!(
        tmpreg.get_id(),
        RegID::Invalid,
        "stsym64 {inst:#?} with invalid tmpreg"
    );

    let base = GPR64::from_real(inst.get_base());
    let offset = inst.get_offset();

    let load_sym = LoadConst64Symbol::new(MirOP::LoadConst64Symbol, tmpreg, sym);
    let store_reg = StoreGr64Base::new(MirOP::StrGr64Base, tmpreg, base, offset);
    actions.push_back(PushFront(load_sym.into_mir()));
    actions.push_back(PushFront(store_reg.into_mir()));
    actions.push_back(DeleteThis);
}

fn lower_stimm64_sym(inst: &MirStImm64Sym, actions: &mut VecDeque<PostLowerAction>) {
    use PostLowerAction::*;

    let imm = inst.get_imm();
    let tmp_data = GPR64::from_real(inst.get_immreg());
    let tmp_addr = GPR64::from_real(inst.get_addr_reg());
    let sym = inst.get_target();

    // load imm to tmp_data
    let load_const = LoadConst64::new(MirOP::LoadConst64, tmp_data, imm);
    actions.push_back(PushFront(load_const.into_mir()));

    // load symbol address to tmp_addr
    let load_sym =
        LoadConst64Symbol::new(MirOP::LoadConst64Symbol, tmp_addr, MirSymbolOp::Global(sym));
    actions.push_back(PushFront(load_sym.into_mir()));

    // store imm to symbol address
    let store_inst = StoreGr64Base::new(MirOP::StrGr64Base, tmp_data, tmp_addr, ImmLSP64(0));
    actions.push_back(PushFront(store_inst.into_mir()));
    actions.push_back(DeleteThis);
}

fn lower_stimm32_sym(inst: &MirStImm32Sym, actions: &mut VecDeque<PostLowerAction>) {
    use PostLowerAction::*;

    let imm = inst.get_imm();
    let tmp_data = GPR64::from_real(inst.get_immreg());
    let tmp_addr = GPR64::from_real(inst.get_addr_reg());
    let sym = inst.get_target();

    // load imm to tmp_data
    let load_const = LoadConst64::new(MirOP::LoadConst64, tmp_data, imm.zext_to_imm64());
    actions.push_back(PushFront(load_const.into_mir()));

    // load symbol address to tmp_addr
    let load_sym =
        LoadConst64Symbol::new(MirOP::LoadConst64Symbol, tmp_addr, MirSymbolOp::Global(sym));
    actions.push_back(PushFront(load_sym.into_mir()));

    // store imm to symbol address
    let store_inst = StoreGr32Base::new(
        MirOP::StrGr32Base,
        tmp_data.trunc_to_gpr32(),
        tmp_addr,
        ImmLSP32(0),
    );
    actions.push_back(PushFront(store_inst.into_mir()));
    actions.push_back(DeleteThis);
}

/// 给 MirStSym64Sym 分配临时寄存器
///
/// #### 指令语法及语义
///
/// * `mir.stimm64.sym <imm> to <target> through data <tmpreg64> and addr <tmpreg64>`
///
/// 把符号 `<imm>` 的地址存储到 `<target>` 中, 在存储过程中可能需要使用 `data <tmpreg64>` 中转数据、
/// `addr <tmpreg64>` 中转地址.
///
/// 在 Remusys MIR 中, 全局符号自己就表示自己的地址.
fn lower_stsym64_sym(inst: &MirStSym64Sym, actions: &mut VecDeque<PostLowerAction>) {
    use PostLowerAction::*;

    let src = inst.get_imm();
    let tmp_data = GPR64::from_real(inst.get_immreg());
    let tmp_addr = GPR64::from_real(inst.get_addr_reg());
    let target = inst.get_target();

    // Copy src to tmp_data
    // Remusys uses LoadConstSymbol to represent "copy symbol address to register"
    let prepare_src = LoadConst64Symbol::new(MirOP::LoadConst64Symbol, tmp_data, src);
    actions.push_back(PushFront(prepare_src.into_mir()));

    // Copy target address to tmp_addr
    let prepare_target = LoadConst64Symbol::new(
        MirOP::LoadConst64Symbol,
        tmp_addr,
        MirSymbolOp::Global(target),
    );
    actions.push_back(PushFront(prepare_target.into_mir()));

    // Store tmp_data to `*tmp_addr`
    let store_inst = StoreGr64Base::new(MirOP::StrGr64Base, tmp_data, tmp_addr, ImmLSP64(0));
    actions.push_back(PushFront(store_inst.into_mir()));
    actions.push_back(DeleteThis);
}

fn post_lower_an_inst(inst: &MirInst, actions: &mut VecDeque<PostLowerAction>) {
    match inst {
        MirInst::MirLdrLitG64(inst) => lower_mir_ldrlit_g64(inst, actions),
        MirInst::MirLdrLitG32(inst) => lower_mir_ldrlit_g32(inst, actions),
        MirInst::MirLdrLitF64(inst) => lower_mir_ldrlit_f64(inst, actions),
        MirInst::MirLdrLitF32(inst) => lower_mir_ldrlit_f32(inst, actions),
        MirInst::MirStrLitG64(inst) => lower_mir_strlit_g64(inst, actions),
        MirInst::MirStrLitG32(inst) => lower_mir_strlit_g32(inst, actions),
        MirInst::MirStrLitF64(inst) => lower_mir_strlit_f64(inst, actions),
        MirInst::MirStrLitF32(inst) => lower_mir_strlit_f32(inst, actions),
        MirInst::MirLdImmF32(inst) => lower_ldimm_f32(inst, actions),
        MirInst::MirLdImmF64(inst) => lower_ldimm_f64(inst, actions),
        MirInst::MirStImm32(inst) => lower_stimm32(inst, actions),
        MirInst::MirStImm64(inst) => lower_stimm64(inst, actions),
        MirInst::MirStSym64(inst) => lower_stsym64(inst, actions),
        MirInst::MirStImm64Sym(inst) => lower_stimm64_sym(inst, actions),
        MirInst::MirStImm32Sym(inst) => lower_stimm32_sym(inst, actions),
        MirInst::MirStSym64Sym(inst) => lower_stsym64_sym(inst, actions),
        _ => {}
    }
}

pub(super) fn post_lower_a_function(func: &MirFunc, module: &mut MirModule) {
    let allocs = module.allocs.get_mut();
    let insts = func.dump_insts_when(&allocs.block, &allocs.inst, |inst| {
        matches!(
            inst,
            MirInst::MirLdrLitG64(_)
                | MirInst::MirLdrLitG32(_)
                | MirInst::MirLdrLitF64(_)
                | MirInst::MirLdrLitF32(_)
                | MirInst::MirStrLitG64(_)
                | MirInst::MirStrLitG32(_)
                | MirInst::MirStrLitF64(_)
                | MirInst::MirStrLitF32(_)
                | MirInst::MirLdImmF32(_)
                | MirInst::MirLdImmF64(_)
                | MirInst::MirStImm32(_)
                | MirInst::MirStImm64(_)
                | MirInst::MirStSym64(_)
                | MirInst::MirStImm64Sym(_)
                | MirInst::MirStImm32Sym(_)
                | MirInst::MirStSym64Sym(_)
        )
    });
    let mut actions = VecDeque::new();
    for (bref, iref) in insts {
        let mut self_deleted = false;
        post_lower_an_inst(iref.to_data(&allocs.inst), &mut actions);
        while let Some(action) = actions.pop_front() {
            match action {
                PostLowerAction::PushFront(new_inst) => {
                    if self_deleted {
                        panic!("Not implemented: PushFront after DeleteThis");
                    }
                    let new_inst = MirInstRef::from_alloc(&mut allocs.inst, new_inst);
                    bref.get_insts(&allocs.block)
                        .node_add_prev(&allocs.inst, iref, new_inst)
                        .expect("Failed to add new inst");
                }
                PostLowerAction::DeleteThis => {
                    bref.get_insts(&allocs.block)
                        .unplug_node(&allocs.inst, iref)
                        .expect("Failed to unplug old inst");
                    allocs.inst.remove(iref.get_handle());
                    self_deleted = true;
                }
            }
        }
    }
}

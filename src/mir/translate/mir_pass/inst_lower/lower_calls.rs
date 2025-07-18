use crate::mir::{
    inst::{
        IMirSubInst, impls::*, inst::MirInst, mir_call::MirCallerSavedRegs, mirops::MirCall,
        opcode::MirOP,
    },
    module::{
        MirGlobalRef, MirModule,
        stack::{MirStackLayout, VirtRegAlloc},
    },
    operand::{
        IMirSubOperand, MirOperand,
        imm::{Imm64, ImmCalc, ImmKind, ImmLoad32, ImmLoad64},
        imm_traits,
        reg::{FPR32, FPR64, GPR32, GPR64, GPReg, RegID, RegOperand, RegUseFlags, VFReg},
    },
    translate::mirgen::operandgen::PureSourceReg,
};
use std::collections::VecDeque;

pub fn lower_mir_call(
    call_inst: &MirCall,
    vreg_alloc: &mut VirtRegAlloc,
    out_insts: &mut VecDeque<MirInst>,
    mir_module: &MirModule,
) {
    // Save registers before the call.
    // make_save_regs_insts(call_inst.get_saved_regs(), out_insts);
    SavedRegStackPos::build_save_regs(call_inst.get_saved_regs(), out_insts);
    let callee_func = call_inst
        .get_callee_func()
        .expect("Callee function must be set");

    // Prepare the arguments for the call instruction.
    let mut arg_gpreg_id = 0;
    let mut arg_vfreg_id = 0;
    let mut spilled_arg_count = 0;
    let callee_stack_layout = callee_func.borrow_inner().stack_layout.clone();
    let spilled_args_size = callee_stack_layout.args_size;

    // 为参数预留 SP 栈空间.
    // 这里顺便把恢复栈空间的指令也做了.
    let mut restore_sp_insts = Vec::with_capacity(2);
    make_reserve_and_restore_stack_space_insts(
        vreg_alloc,
        out_insts,
        spilled_args_size,
        &mut restore_sp_insts,
    );

    for arg in call_inst.args() {
        let arg = arg.get();
        match arg {
            MirOperand::GPReg(source_gpr) => prepare_gpreg_arg(
                out_insts,
                &mut arg_gpreg_id,
                &mut spilled_arg_count,
                &callee_stack_layout,
                source_gpr,
            ),
            MirOperand::VFReg(vfreg) => prepare_fpreg_arg(
                out_insts,
                &mut arg_vfreg_id,
                &mut spilled_arg_count,
                &callee_stack_layout,
                vfreg,
            ),
            MirOperand::None => panic!("Unexpected None operand in call arguments"),
            MirOperand::PState(_) => panic!("Unexpected PState operand in call arguments"),
            _ => todo!(
                "MirCall converted all arguments into registers. But we encounted some errors. ({arg:?} handling)."
            ),
        }
    }

    // Then, we need to generate the call instruction.
    let callee_operand = MirGlobalRef::from_mir(call_inst.callee().get());
    let callee_entry = {
        let alloc_mbb = mir_module.borrow_alloc_block();
        callee_func.blocks.get_front_ref(&alloc_mbb)
    };

    let bl_inst = if let Some(callee_entry) = callee_entry {
        BLinkLabel::new(MirOP::BLink, GPR64::ra(), callee_entry).into_mir()
    } else {
        // External function or a function without a body.
        BLinkGlobal::new(MirOP::BLinkGlobal, GPR64::ra(), callee_operand).into_mir()
    };
    out_insts.push_back(bl_inst);

    // restore the stack space.
    for restore_inst in restore_sp_insts {
        out_insts.push_back(restore_inst);
    }

    // After the call, we need to prepare the return value.
    let mut saved_regs = call_inst.get_saved_regs();
    if let Some(ret_val) = call_inst.get_ret_arg() {
        saved_regs = prepare_return_value(out_insts, ret_val, saved_regs);
    }

    // After the call, we need to restore the registers.
    make_restore_regs_inst(saved_regs, out_insts);
}

fn prepare_return_value(
    out_insts: &mut VecDeque<MirInst>,
    ret_val: RegOperand,
    mut saved_regs: MirCallerSavedRegs,
) -> MirCallerSavedRegs {
    saved_regs.unsave_reg(ret_val);
    if ret_val.get_id() == RegID::Phys(0) {
        return saved_regs;
    }
    let ret_reg = PureSourceReg::from_reg(ret_val);
    let mov_inst = match ret_reg {
        PureSourceReg::F32(retr) => UnaF32::new(MirOP::FMov32R, retr, FPR32::retval()).into_mir(),
        PureSourceReg::F64(retr) => UnaF64::new(MirOP::FMov64R, retr, FPR64::retval()).into_mir(),
        PureSourceReg::G32(retr) => {
            Una32R::new(MirOP::Mov32R, retr, GPR32::retval(), None).into_mir()
        }
        PureSourceReg::G64(retr) => {
            Una64R::new(MirOP::Mov64R, retr, GPR64::retval(), None).into_mir()
        }
    };
    out_insts.push_back(mov_inst);
    saved_regs
}

fn prepare_fpreg_arg(
    out_insts: &mut VecDeque<MirInst>,
    arg_vfreg_id: &mut u32,
    spilled_arg_count: &mut usize,
    callee_stack_layout: &MirStackLayout,
    vfreg: VFReg,
) {
    if vfreg.get_id() == RegID::Phys(*arg_vfreg_id) {
        return;
    }
    // 现在我们知道, source vfreg 和参数要求的 vfreg id 不同, 需要移动过去.
    let VFReg(_, si, uf) = vfreg;
    if *arg_vfreg_id < 8 {
        let arg_vfreg = VFReg(*arg_vfreg_id, si, uf);
        *arg_vfreg_id += 1;
        let mov_inst = match arg_vfreg.get_bits_log2() {
            5 => {
                let source_fpr = FPR32::from_real(vfreg);
                let arg_fpr = FPR32::from_real(arg_vfreg);
                UnaF32::new(MirOP::Mov32R, arg_fpr, source_fpr).into_mir()
            }
            6 => {
                let source_fpr = FPR64::from_real(vfreg);
                let arg_fpr = FPR64::from_real(arg_vfreg);
                UnaF64::new(MirOP::Mov64R, arg_fpr, source_fpr).into_mir()
            }
            _ => panic!("Invalid VFReg bits: {}", arg_vfreg.get_bits_log2()),
        };
        out_insts.push_back(mov_inst);
    } else {
        // 参数溢出到栈上
        let spilled_arg_id = *spilled_arg_count;
        *spilled_arg_count += 1;
        let spilled_arg_pos = &callee_stack_layout.args[spilled_arg_id];
        let offset = spilled_arg_pos.offset;
        save_fpr_arg_to_stack(vfreg, offset as u64, out_insts);
    }
}

fn prepare_gpreg_arg(
    out_insts: &mut VecDeque<MirInst>,
    arg_gpreg_id: &mut u32,
    spilled_arg_count: &mut usize,
    callee_stack_layout: &MirStackLayout,
    source_gpr: GPReg,
) {
    if source_gpr.get_id() == RegID::Phys(*arg_gpreg_id) {
        // If the GPReg is already used, we can skip it.
        return;
    }
    let GPReg(_, si, uf) = source_gpr;

    // 现在我们知道, source gpr 和参数要求的 gpr id 不同, 需要移动过去.
    if *arg_gpreg_id < 8 {
        let arg_gpreg = GPReg(*arg_gpreg_id, si, uf);
        *arg_gpreg_id += 1;
        let mov_inst = match arg_gpreg.get_bits_log2() {
            5 => {
                let source_gpr = GPR32::from_real(source_gpr);
                let arg_gpr = GPR32::from_real(arg_gpreg);
                Una32R::new(MirOP::Mov32R, arg_gpr, source_gpr, None).into_mir()
            }
            6 => {
                let source_gpr = GPR64::from_real(source_gpr);
                let arg_gpr = GPR64::from_real(arg_gpreg);
                Una64R::new(MirOP::Mov64R, arg_gpr, source_gpr, None).into_mir()
            }
            _ => panic!("Invalid GPReg bits: {}", arg_gpreg.get_bits_log2()),
        };
        out_insts.push_back(mov_inst);
    } else {
        // 参数溢出到栈上
        let spilled_arg_id = *spilled_arg_count;
        *spilled_arg_count += 1;
        let spilled_arg_pos = &callee_stack_layout.args[spilled_arg_id];
        let offset = spilled_arg_pos.offset;
        save_gpr_arg_to_stack(source_gpr, offset as u64, out_insts);
    }
}

fn make_reserve_and_restore_stack_space_insts(
    vreg_alloc: &mut VirtRegAlloc,
    out_insts: &mut VecDeque<MirInst>,
    spilled_args_size: u64,
    restore_sp_insts: &mut Vec<MirInst>,
) {
    if spilled_args_size == 0 {
        return;
    }
    // Reserve stack space for spilled arguments.
    let sp = GPR64::sp();
    if imm_traits::is_calc_imm(spilled_args_size) {
        let offset = ImmCalc::new(spilled_args_size as u32);
        let reserve_sp = Bin64RC::new(MirOP::Sub64I, sp, sp, offset);
        out_insts.push_back(reserve_sp.into_mir());

        let restore_sp = Bin64RC::new(MirOP::Add64I, sp, sp, offset);
        restore_sp_insts.push(restore_sp.into_mir());
    } else {
        let temp_reg = vreg_alloc.insert_gp(GPR64::new_empty().into_real());
        let temp_reg = GPR64::from_real(temp_reg);
        let ldrconst = LoadConst64::new(
            MirOP::LoadConst64,
            temp_reg,
            Imm64(spilled_args_size, ImmKind::Full),
        );
        out_insts.push_back(ldrconst.into_mir());
        let reserve_sp = Bin64R::new(MirOP::Sub64R, sp, sp, temp_reg, None);
        out_insts.push_back(reserve_sp.into_mir());
        let restore_sp = Bin64R::new(MirOP::Add64R, sp, sp, temp_reg, None);
        restore_sp_insts.push(restore_sp.into_mir());
    }
}

#[derive(Debug, Clone)]
struct SavedRegStackPos {
    gpr_idx: [u8; 32],
    fpr_idx: [u8; 32],
}

impl SavedRegStackPos {
    const INVALID_ID: u8 = u8::MAX;

    pub fn _get_gpr_stack_offset(&self, id: u8) -> Option<usize> {
        if id >= 32 || self.gpr_idx[id as usize] == Self::INVALID_ID {
            None
        } else {
            let idx = self.gpr_idx[id as usize];
            Some(idx as usize * 8)
        }
    }
    pub fn _get_fpr_stack_offset(&self, id: u8) -> Option<usize> {
        if id >= 32 || self.fpr_idx[id as usize] == Self::INVALID_ID {
            None
        } else {
            let idx = self.fpr_idx[id as usize];
            Some(idx as usize * 8)
        }
    }
    pub fn _gpr_is_saved(&self, id: u8) -> bool {
        self._get_gpr_stack_offset(id).is_some()
    }
    pub fn _fpr_is_saved(&self, id: u8) -> bool {
        self._get_fpr_stack_offset(id).is_some()
    }

    pub fn build_save_regs(
        saved_regs: MirCallerSavedRegs,
        out_insts: &mut VecDeque<MirInst>,
    ) -> Self {
        let mut ret = Self {
            gpr_idx: [Self::INVALID_ID; 32],
            fpr_idx: [Self::INVALID_ID; 32],
        };
        if saved_regs.is_empty() {
            return ret;
        }

        // 目前该寄存器保存函数会直接保存寄存器的所有位到栈上—— MirCallerSavedRegs 还没有位域标识.
        // 尽管对于一些只有低 32 位的寄存器，这可能不是最优的方式，但它好写啊.
        let nregs = saved_regs.num_regs();
        let saved_stack_size = nregs as u32 * 8;
        // 这个保存栈大小小于 64 * 8 = 512, 在 sub 指令的立即数限制范围内.
        debug_assert!(imm_traits::is_calc_imm(saved_stack_size as u64));
        let sp = GPR64::sp();
        let offset = ImmCalc::new(saved_stack_size);

        // 预留一部分栈空间出来.
        out_insts.push_back(Bin64RC::new(MirOP::Sub64I, sp, sp, offset).into_mir());

        for (stack_pos, reg) in saved_regs.into_iter().enumerate() {
            let offset_imm = ImmLoad64::new(stack_pos as i64 * 8);
            let RegOperand(id, _, _, is_fp) = reg;
            if is_fp {
                let rd = FPR64(id, RegUseFlags::USE);
                let store = LoadStoreF64Base::new(MirOP::StrF64Base, rd, sp, offset_imm);
                out_insts.push_back(store.into_mir());
                ret.fpr_idx[id as usize] = stack_pos as u8;
            } else {
                let rd = GPR64(id, RegUseFlags::USE);
                let store = LoadStoreGr64Base::new(MirOP::StrGr64Base, rd, sp, offset_imm);
                out_insts.push_back(store.into_mir());
                ret.gpr_idx[id as usize] = stack_pos as u8;
            }
        }
        ret
    }
}

pub fn make_restore_regs_inst(saved_regs: MirCallerSavedRegs, out_insts: &mut VecDeque<MirInst>) {
    let nregs = saved_regs.num_regs();
    if nregs == 0 {
        return;
    }
    // 趁着栈空间还没有收回去, 先恢复寄存器.
    let sp = GPR64::sp();
    for (stack_pos, reg) in saved_regs.into_iter().enumerate() {
        let RegOperand(id, _, _, is_fp) = reg;
        let offset = ImmLoad64::new(stack_pos as i64 * 8);
        if is_fp {
            let rd = FPR64(id, RegUseFlags::KILL);
            let load = LoadStoreF64Base::new(MirOP::LdrF64Base, rd, sp, offset);
            out_insts.push_back(load.into_mir());
        } else {
            let rd = GPR64(id, RegUseFlags::KILL);
            let load = LoadStoreGr64Base::new(MirOP::LdrGr64Base, rd, sp, offset);
            out_insts.push_back(load.into_mir());
        }
    }

    // 收回之前预留的栈空间.
    let offset = ImmCalc::new(nregs as u32 * 8);
    let restore_sp = Bin64RC::new(MirOP::Add64I, sp, sp, offset);
    out_insts.push_back(restore_sp.into_mir());
}

fn save_gpr32_arg_to_stack(gpreg: GPReg, sp_offset: u64, out_insts: &mut VecDeque<MirInst>) {
    let sp = GPR64::sp();
    if imm_traits::is_load32_imm(sp_offset as i64) {
        let store_inst = LoadStoreGr32Base::new(
            MirOP::StrGr32Base,
            GPR32::from_real(gpreg),
            sp,
            ImmLoad32::new(sp_offset as i32),
        );
        out_insts.push_back(store_inst.into_mir());
    } else {
        // 偏移量太大，使用寄存器间接寻址
        let offset_reg = GPR64(0, RegUseFlags::DEF);
        let load_offset = LoadConst64::new(
            MirOP::LoadConst64,
            offset_reg,
            Imm64(sp_offset, ImmKind::Full),
        );
        out_insts.push_back(load_offset.into_mir());
        let store_inst = LoadStoreGr32::new(
            MirOP::StrGr32,
            GPR32::from_real(gpreg),
            sp,
            GPR64(0, RegUseFlags::USE),
            None,
        );
        out_insts.push_back(store_inst.into_mir());
    }
}

fn save_gpr64_arg_to_stack(gpreg: GPReg, sp_offset: u64, out_insts: &mut VecDeque<MirInst>) {
    let sp = GPR64::sp();
    if imm_traits::is_load64_imm(sp_offset as i64) {
        let store_inst = LoadStoreGr64Base::new(
            MirOP::StrGr64Base,
            GPR64::from_real(gpreg),
            sp,
            ImmLoad64::new(sp_offset as i64),
        );
        out_insts.push_back(store_inst.into_mir());
    } else {
        // 偏移量太大，使用寄存器间接寻址
        let offset_reg = GPR64(0, RegUseFlags::DEF);
        let load_offset = LoadConst64::new(
            MirOP::LoadConst64,
            offset_reg,
            Imm64(sp_offset, ImmKind::Full),
        );
        out_insts.push_back(load_offset.into_mir());
        let store_inst = LoadStoreGr64::new(
            MirOP::StrGr64,
            GPR64::from_real(gpreg),
            sp,
            GPR64(0, RegUseFlags::USE),
            None,
        );
        out_insts.push_back(store_inst.into_mir());
    }
}

/// 将 GPR 参数溢出存储到栈上
fn save_gpr_arg_to_stack(gpreg: GPReg, sp_offset: u64, out_insts: &mut VecDeque<MirInst>) {
    match gpreg.get_bits_log2() {
        5 => {
            // 32-bit register, store it to the stack.
            save_gpr32_arg_to_stack(gpreg, sp_offset, out_insts);
        }
        6 => {
            // 64-bit register, store it to the stack.
            save_gpr64_arg_to_stack(gpreg, sp_offset, out_insts);
        }
        _ => panic!("Invalid GPReg bits: {}", gpreg.get_bits_log2()),
    }
}

fn save_fpr64_arg_to_stack(fpr: FPR64, sp_offset: u64, out_insts: &mut VecDeque<MirInst>) {
    let sp = GPR64::sp();

    if imm_traits::is_load64_imm(sp_offset as i64) {
        let store_inst =
            LoadStoreF64Base::new(MirOP::StrF64Base, fpr, sp, ImmLoad64::new(sp_offset as i64));
        out_insts.push_back(store_inst.into_mir());
    } else {
        // 偏移量太大，使用寄存器间接寻址
        let offset_reg = GPR64(0, RegUseFlags::DEF);
        let load_offset = LoadConst64::new(
            MirOP::LoadConst64,
            offset_reg,
            Imm64(sp_offset, ImmKind::Full),
        );
        out_insts.push_back(load_offset.into_mir());
        let store_inst =
            LoadStoreF64::new(MirOP::StrF64, fpr, sp, GPR64(0, RegUseFlags::USE), None);
        out_insts.push_back(store_inst.into_mir());
    }
}

fn save_fpr32_arg_to_stack(fpr: FPR32, sp_offset: u64, out_insts: &mut VecDeque<MirInst>) {
    let sp = GPR64::sp();

    if imm_traits::is_load32_imm(sp_offset as i64) {
        let store_inst =
            LoadStoreF32Base::new(MirOP::StrF32Base, fpr, sp, ImmLoad32::new(sp_offset as i32));
        out_insts.push_back(store_inst.into_mir());
    } else {
        // 偏移量太大，使用寄存器间接寻址
        let offset_reg = GPR64(0, RegUseFlags::DEF);
        let load_offset = LoadConst64::new(
            MirOP::LoadConst64,
            offset_reg,
            Imm64(sp_offset, ImmKind::Full),
        );
        out_insts.push_back(load_offset.into_mir());
        let store_inst =
            LoadStoreF32::new(MirOP::StrF32, fpr, sp, GPR64(0, RegUseFlags::USE), None);
        out_insts.push_back(store_inst.into_mir());
    }
}

fn save_fpr_arg_to_stack(fpr: VFReg, sp_offset: u64, out_insts: &mut VecDeque<MirInst>) {
    match fpr.get_bits_log2() {
        5 => save_fpr32_arg_to_stack(FPR32::from_real(fpr), sp_offset, out_insts),
        6 => save_fpr64_arg_to_stack(FPR64::from_real(fpr), sp_offset, out_insts),
        _ => panic!("Invalid VFReg bits: {}", fpr.get_bits_log2()),
    }
}

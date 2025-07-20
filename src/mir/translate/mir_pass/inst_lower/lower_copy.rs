use crate::mir::{
    inst::{IMirSubInst, impls::*, inst::MirInst, opcode::MirOP},
    module::{MirGlobalRef, block::MirBlockRef, stack::VirtRegAlloc},
    operand::{IMirSubOperand, MirOperand, compound::MirSymbolOp, imm::*, imm_traits, reg::*},
    translate::mirgen::operandgen::PureSourceReg,
};
use std::collections::VecDeque;

pub fn lower_copy32_inst(copy32: &MirCopy32, out_insts: &mut VecDeque<MirInst>) {
    let src = copy32.get_src();
    let dst = GPR32::from_real(copy32.get_dst());

    match src {
        MirOperand::GPReg(gpreg) => {
            let pure = PureSourceReg::from_reg(gpreg.into());
            copy_from_reg(dst, pure, out_insts);
        }
        MirOperand::VFReg(vfreg) => {
            let pure = PureSourceReg::from_reg(vfreg.into());
            copy_from_reg(dst, pure, out_insts);
        }
        MirOperand::Imm64(imm64) => copy_from_u64(dst, imm64.get_value(), out_insts),
        MirOperand::Imm32(imm32) => copy_from_u32(dst, imm32.get_value(), out_insts),
        MirOperand::F32(f) => copy_from_u32(dst, f.to_bits(), out_insts),
        MirOperand::F64(f) => copy_from_u64(dst, f.to_bits(), out_insts),
        MirOperand::PState(_) => todo!("MSR and MSR instructions have not been implemented yet."),
        MirOperand::Label(_) | MirOperand::Global(_) | MirOperand::SwitchTab(_) => {
            panic!("64-bit operand {src:?} is not supported in 32-bit copy instruction")
        }
        MirOperand::None => { /* Nothing */ }
    }

    fn copy_from_reg(dst: GPR32, src: PureSourceReg, out_insts: &mut VecDeque<MirInst>) {
        use PureSourceReg::*;
        let inst = match src {
            F32(src) => UnaGF32::new(MirOP::FMovGF32, dst, src).into_mir(),
            F64(src) => UnaGF64::new(MirOP::FMovGF64, GPR64(dst.0, dst.1), src).into_mir(),
            G32(src) => Una32R::new(MirOP::Mov32R, dst, src, None).into_mir(),
            G64(GPR64(id, uf)) => Una32R::new(MirOP::Mov32R, dst, GPR32(id, uf), None).into_mir(),
        };
        out_insts.push_back(inst);
    }
    fn copy_from_u64(dst: GPR32, src: u64, out_insts: &mut VecDeque<MirInst>) {
        let inst = if imm_traits::is_mov_imm(src) {
            Mov32I::new(MirOP::Mov32I, dst, ImmMov::new(src as u32)).into_mir()
        } else {
            let GPR32(id, uf) = dst;
            let dst64 = GPR64(id, uf);
            LoadConst64::new(
                MirOP::LoadConst64,
                dst64,
                Imm64(src & 0xFFFF_FFFF, ImmKind::Full),
            )
            .into_mir()
        };
        out_insts.push_back(inst);
    }
    fn copy_from_u32(dst: GPR32, src: u32, out_insts: &mut VecDeque<MirInst>) {
        let inst = if imm_traits::is_mov_imm(src as u64) {
            Mov32I::new(MirOP::Mov32I, dst, ImmMov::new(src)).into_mir()
        } else {
            let GPR32(id, uf) = dst;
            let dst64 = GPR64(id, uf);
            LoadConst64::new(
                MirOP::LoadConst64,
                dst64,
                Imm64(src as u64 & 0xFFFF_FFFF, ImmKind::Full),
            )
            .into_mir()
        };
        out_insts.push_back(inst);
    }
}

pub fn lower_copy64_inst(copy64: &MirCopy64, out_insts: &mut VecDeque<MirInst>) {
    let src = copy64.get_src();
    let dst = GPR64::from_real(copy64.get_dst());

    match src {
        MirOperand::GPReg(gpreg) => {
            let pure = PureSourceReg::from_reg(gpreg.into());
            copy_from_reg(dst, pure, out_insts);
        }
        MirOperand::VFReg(vfreg) => {
            let pure = PureSourceReg::from_reg(vfreg.into());
            copy_from_reg(dst, pure, out_insts);
        }
        MirOperand::Imm64(imm64) => copy_from_u64(dst, imm64.get_value(), out_insts),
        MirOperand::Imm32(imm32) => copy_from_u32(dst, imm32.get_value(), out_insts),
        MirOperand::F32(f) => copy_from_u32(dst, f.to_bits(), out_insts),
        MirOperand::F64(f) => copy_from_u64(dst, f.to_bits(), out_insts),
        MirOperand::PState(_) => todo!("MSR and MSR instructions have not been implemented yet."),
        MirOperand::Label(label_ref) => copy_from_label(dst, label_ref, out_insts),
        MirOperand::Global(global_ref) => copy_from_global(dst, global_ref, out_insts),
        MirOperand::SwitchTab(switch_tab) => copy_from_switch_tab(dst, switch_tab, out_insts),
        MirOperand::None => { /* Nothing */ }
    }

    fn copy_from_reg(dst: GPR64, src: PureSourceReg, out_insts: &mut VecDeque<MirInst>) {
        use PureSourceReg::*;
        let inst = match src {
            F32(src) => UnaGF64::new(MirOP::FMovGF64, dst, FPR64(src.0, src.1)).into_mir(),
            F64(src) => UnaGF64::new(MirOP::FMovGF64, dst, src).into_mir(),
            G32(src) => Una64R::new(MirOP::Mov64R, dst, GPR64(src.0, src.1), None).into_mir(),
            G64(src) => Una64R::new(MirOP::Mov64R, dst, src, None).into_mir(),
        };
        out_insts.push_back(inst);
    }
    fn copy_from_u64(dst: GPR64, src: u64, out_insts: &mut VecDeque<MirInst>) {
        let inst = if imm_traits::is_mov_imm(src) {
            Mov64I::new(MirOP::Mov64I, dst, ImmMov::new(src as u32)).into_mir()
        } else {
            LoadConst64::new(MirOP::LoadConst64, dst, Imm64(src, ImmKind::Full)).into_mir()
        };
        out_insts.push_back(inst);
    }
    fn copy_from_u32(dst: GPR64, src: u32, out_insts: &mut VecDeque<MirInst>) {
        let inst = if imm_traits::is_mov_imm(src as u64) {
            Mov64I::new(MirOP::Mov64I, dst, ImmMov::new(src)).into_mir()
        } else {
            LoadConst64::new(MirOP::LoadConst64, dst, Imm64(src as u64, ImmKind::Full)).into_mir()
        };
        out_insts.push_back(inst);
    }
    fn copy_from_symbol(dst: GPR64, src: MirSymbolOp, out_insts: &mut VecDeque<MirInst>) {
        let inst = LoadConst64Symbol::new(MirOP::LoadConst64Symbol, dst, src);
        out_insts.push_back(inst.into_mir());
    }
    fn copy_from_label(dst: GPR64, src: MirBlockRef, out_insts: &mut VecDeque<MirInst>) {
        copy_from_symbol(dst, MirSymbolOp::Label(src), out_insts);
    }
    fn copy_from_global(dst: GPR64, src: MirGlobalRef, out_insts: &mut VecDeque<MirInst>) {
        copy_from_symbol(dst, MirSymbolOp::Global(src), out_insts);
    }
    fn copy_from_switch_tab(dst: GPR64, src_index: u32, out_insts: &mut VecDeque<MirInst>) {
        copy_from_symbol(dst, MirSymbolOp::SwitchTab(src_index), out_insts);
    }
}

pub fn lower_fcopy32_inst(
    fcopy32: &MirFCopy32,
    vreg_alloc: &mut VirtRegAlloc,
    out_insts: &mut VecDeque<MirInst>,
) {
    let src = fcopy32.get_src();
    let dst = FPR32::from_real(fcopy32.get_dst());

    match src {
        MirOperand::GPReg(src) => {
            let src = PureSourceReg::from_reg(src.into());
            copy_from_reg(dst, src, out_insts)
        }
        MirOperand::VFReg(src) => {
            let src = PureSourceReg::from_reg(src.into());
            copy_from_reg(dst, src, out_insts)
        }
        MirOperand::Imm64(src) => copy_from_u64full(dst, src.get_value(), vreg_alloc, out_insts),
        MirOperand::Imm32(src) => {
            copy_from_u64full(dst, src.get_value() as u64, vreg_alloc, out_insts)
        }
        MirOperand::F32(src) => copy_from_f32(dst, src, vreg_alloc, out_insts),
        MirOperand::F64(src) => copy_from_f32(dst, src as f32, vreg_alloc, out_insts),
        _ => panic!("Invalid source operand {src:?} for fcopy32 instruction"),
    }

    fn copy_from_f32(
        dst: FPR32,
        src: f32,
        vreg_alloc: &mut VirtRegAlloc,
        out_insts: &mut VecDeque<MirInst>,
    ) {
        if imm_traits::try_cast_f32_to_aarch8(src).is_some() {
            let inst = FMov32I::new(MirOP::FMov32I, dst, ImmFMov32::new(src)).into_mir();
            out_insts.push_back(inst);
        } else {
            copy_from_u64full(dst, src.to_bits() as u64, vreg_alloc, out_insts);
        }
    }
    fn copy_from_u64full(
        dst: FPR32,
        src: u64,
        vreg_alloc: &mut VirtRegAlloc,
        out_insts: &mut VecDeque<MirInst>,
    ) {
        let vreg = vreg_alloc.insert_gp(GPR64::new_empty().into_real());
        let vreg = GPR64::from_real(vreg);
        let src = Imm64(src, ImmKind::Full);
        let load_to_temp = LoadConst64::new(MirOP::LoadConst64, vreg, src);
        out_insts.push_back(load_to_temp.into_mir());
        let vreg32 = GPR32(vreg.0, vreg.1);
        let inst = UnaFG32::new(MirOP::FMovFG32, dst, vreg32);
        out_insts.push_back(inst.into_mir());
    }
    fn copy_from_reg(dst: FPR32, src: PureSourceReg, out_insts: &mut VecDeque<MirInst>) {
        use PureSourceReg::*;
        let inst = match src {
            F32(src) => UnaF32::new(MirOP::FMov32R, dst, src).into_mir(),
            F64(src) => UnaryF32F64::new(MirOP::FCvt32F64, dst, src).into_mir(),
            G32(src) => UnaFG32::new(MirOP::FMovFG32, dst, src).into_mir(),
            G64(GPR64(id, uf)) => UnaFG32::new(MirOP::FMovFG32, dst, GPR32(id, uf)).into_mir(),
        };
        out_insts.push_back(inst);
    }
}

pub fn lower_fcopy64_inst(
    fcopy64: &MirFCopy64,
    vreg_alloc: &mut VirtRegAlloc,
    out_insts: &mut VecDeque<MirInst>,
) {
    let src = fcopy64.get_src();
    let dst = FPR64::from_real(fcopy64.get_dst());

    match src {
        MirOperand::GPReg(src) => {
            let src = PureSourceReg::from_reg(src.into());
            copy_from_reg(dst, src, out_insts)
        }
        MirOperand::VFReg(src) => {
            let src = PureSourceReg::from_reg(src.into());
            copy_from_reg(dst, src, out_insts)
        }
        MirOperand::Imm64(src) => copy_from_u64full(dst, src.get_value(), vreg_alloc, out_insts),
        MirOperand::Imm32(src) => {
            copy_from_u64full(dst, src.get_value() as u64, vreg_alloc, out_insts)
        }
        MirOperand::F32(src) => copy_from_f64(dst, src as f64, vreg_alloc, out_insts),
        MirOperand::F64(src) => copy_from_f64(dst, src, vreg_alloc, out_insts),
        _ => panic!("Invalid source operand {src:?} for fcopy64 instruction"),
    }

    fn copy_from_f64(
        dst: FPR64,
        src: f64,
        vreg_alloc: &mut VirtRegAlloc,
        out_insts: &mut VecDeque<MirInst>,
    ) {
        if imm_traits::try_cast_f64_to_aarch8(src).is_some() {
            let inst = FMov64I::new(MirOP::FMov64I, dst, ImmFMov64::new(src)).into_mir();
            out_insts.push_back(inst);
        } else {
            copy_from_u64full(dst, src.to_bits(), vreg_alloc, out_insts);
        }
    }
    fn copy_from_u64full(
        dst: FPR64,
        src: u64,
        vreg_alloc: &mut VirtRegAlloc,
        out_insts: &mut VecDeque<MirInst>,
    ) {
        let vreg = vreg_alloc.insert_gp(GPR64::new_empty().into_real());
        let vreg = GPR64::from_real(vreg);
        let src = Imm64(src, ImmKind::Full);
        let load_to_temp = LoadConst64::new(MirOP::LoadConst64, vreg, src);
        out_insts.push_back(load_to_temp.into_mir());
        let inst = UnaFG64::new(MirOP::FMovFG64, dst, vreg);
        out_insts.push_back(inst.into_mir());
    }
    fn copy_from_reg(dst: FPR64, src: PureSourceReg, out_insts: &mut VecDeque<MirInst>) {
        use PureSourceReg::*;
        let inst = match src {
            F32(src) => UnaryF64F32::new(MirOP::FCvt64F32, dst, src).into_mir(),
            F64(src) => UnaF64::new(MirOP::FMov64R, dst, src).into_mir(),
            G32(src) => UnaFG64::new(MirOP::FMovFG64, dst, GPR64(src.0, src.1)).into_mir(),
            G64(src) => UnaFG64::new(MirOP::FMovFG64, dst, src).into_mir(),
        };
        out_insts.push_back(inst);
    }
}

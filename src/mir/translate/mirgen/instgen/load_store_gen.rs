use super::InstDispatchError;
use crate::{
    base::SlabRef,
    ir::{
        ValueSSA,
        constant::data::ConstData,
        inst::{InstData, InstRef, UseData},
    },
    mir::{
        inst::{IMirSubInst, impls::*, inst::MirInst, opcode::MirOP},
        module::{MirGlobalRef, vreg_alloc::VirtRegAlloc},
        operand::{
            MirOperand,
            compound::MirSymbolOp,
            imm::{Imm32, Imm64, ImmLSP32, ImmLSP64},
            reg::{FPR32, FPR64, GPR32, GPR64, GPReg, RegOperand, RegUseFlags, VFReg},
            subop::IMirSubOperand,
        },
        translate::mirgen::operandgen::{InstRetval, OperandMap, OperandMapError},
    },
    typing::{id::ValTypeID, types::FloatTypeKind},
};
use core::panic;
use log::debug;
use slab::Slab;
use std::{cell::Ref, collections::VecDeque};

#[derive(Debug, Clone, Copy)]
enum StrDest {
    G64(GPR64),
    Global(MirGlobalRef),
}

#[derive(Debug, Clone, Copy)]
enum StrSrc {
    F32(FPR32),
    F64(FPR64),
    G32(GPR32),
    G64(GPR64),
    Imm32(Imm32),
    Imm64(Imm64),
    Global(MirGlobalRef),
}

impl StrSrc {
    fn from_valuessa(operand_map: &OperandMap, src_ir: &ValueSSA) -> Self {
        match operand_map.find_operand_no_constdata(src_ir) {
            Ok(operand) => match operand {
                MirOperand::GPReg(GPReg(id, si, _)) => match si.get_bits_log2() {
                    5 => Self::G32(GPR32(id, RegUseFlags::USE)),
                    6 => Self::G64(GPR64(id, RegUseFlags::USE)),
                    _ => panic!("Unsupported GPR size for store: {si:?}"),
                },
                MirOperand::VFReg(VFReg(id, si, _)) => match si.get_bits_log2() {
                    5 => Self::F32(FPR32(id, RegUseFlags::USE)),
                    6 => Self::F64(FPR64(id, RegUseFlags::USE)),
                    _ => panic!("Unsupported VFR size for store: {si:?}"),
                },
                MirOperand::Global(gref) => Self::Global(gref),
                _ => panic!("Unsupported source operand for store: {operand:?}"),
            },
            Err(OperandMapError::IsConstData(data)) => Self::from_constdata(data),
            Err(e) => panic!("Failed to find source operand for store: {e:?}"),
        }
    }

    fn from_constdata(data: ConstData) -> Self {
        use FloatTypeKind::*;
        match data {
            ConstData::Zero(ty) => Self::zeroed(ty),
            ConstData::PtrNull(_) => Self::Imm64(Imm64::full(0)),
            ConstData::Int(64, val) => Self::Imm64(Imm64::full(val as u64)),
            ConstData::Int(32, val) => Self::Imm32(Imm32::full(val as u32)),
            ConstData::Float(Ieee32, f) => Self::Imm32(Imm32::from_fp_bits(f as f32)),
            ConstData::Float(Ieee64, f) => Self::Imm64(Imm64::from_fp_bits(f)),
            _ => panic!("Unsupported constant data for store: {data:?}"),
        }
    }

    /// Store 源操作数完全可以不经过浮点寄存器 —— 即使它要存一个浮点常量到内存中.
    fn zeroed(ty: ValTypeID) -> Self {
        match ty {
            ValTypeID::Int(32) => Self::Imm32(Imm32::full(0)),
            ValTypeID::Int(64) | ValTypeID::Ptr => Self::Imm64(Imm64::full(0)),
            ValTypeID::Float(FloatTypeKind::Ieee32) => Self::Imm32(Imm32::full(0)),
            ValTypeID::Float(FloatTypeKind::Ieee64) => Self::Imm64(Imm64::full(0)),
            _ => panic!("Unsupported zeroed type for store: {ty:?}"),
        }
    }
}

pub(super) fn generate_store_inst(
    operand_map: &OperandMap,
    vreg_alloc: &mut VirtRegAlloc,
    out_insts: &mut VecDeque<MirInst>,
    ir_ref: InstRef,
    alloc_inst: &Slab<InstData>,
    alloc_use: Ref<'_, Slab<UseData>>,
) -> Option<Result<(), InstDispatchError>> {
    let InstData::Store(_, store) = ir_ref.to_data(alloc_inst) else {
        panic!("Expected Store instruction");
    };

    let src_ir = store.source.get_operand(&alloc_use);
    let src_mir = StrSrc::from_valuessa(operand_map, &src_ir);

    let dst_ir = store.target.get_operand(&alloc_use);
    let dst_mir = match operand_map.find_operand_no_constdata(&dst_ir) {
        Ok(MirOperand::GPReg(gpreg)) => StrDest::G64(GPR64::from_real(gpreg)),
        Ok(MirOperand::Global(globl)) => StrDest::Global(globl),
        Ok(m) => panic!("Invalid dest for store: {m:?}"),
        Err(e) => panic!("Failed to find dest for store: {e:?}"),
    };
    match dst_mir {
        StrDest::G64(dst_ptr) => {
            let store_inst = generate_store_to_reg(src_mir, dst_ptr);
            out_insts.push_back(store_inst);
        }
        StrDest::Global(global) => {
            let hi20_addr = vreg_alloc.insert_gpr64(GPR64::new_empty());
            let store_inst = generate_store_to_global(src_mir, global, hi20_addr);
            out_insts.push_back(store_inst);
        }
    };
    None
}

fn generate_store_to_reg(src_mir: StrSrc, dst_ptr: GPR64) -> MirInst {
    let zoff32 = ImmLSP32::new(0);
    let zoff64 = ImmLSP64::new(0);
    let wasted = GPR64::new_empty();
    match src_mir {
        StrSrc::F32(fpr32) => {
            StoreF32Base::new(MirOP::StrF32Base, fpr32, dst_ptr, zoff32).into_mir()
        }
        StrSrc::F64(fpr64) => {
            StoreF64Base::new(MirOP::StrF64Base, fpr64, dst_ptr, zoff64).into_mir()
        }
        StrSrc::G32(gpr32) => {
            StoreGr32Base::new(MirOP::StrGr32Base, gpr32, dst_ptr, zoff32).into_mir()
        }
        StrSrc::G64(gpr64) => {
            StoreGr64Base::new(MirOP::StrGr64Base, gpr64, dst_ptr, zoff64).into_mir()
        }
        StrSrc::Imm32(imm32) => {
            MirStImm32::new(MirOP::MirStImm32, wasted, imm32, dst_ptr, zoff32).into_mir()
        }
        StrSrc::Imm64(imm64) => {
            MirStImm64::new(MirOP::MirStImm64, wasted, imm64, dst_ptr, zoff64).into_mir()
        }
        StrSrc::Global(gref) => {
            let imm = MirSymbolOp::Global(gref);
            MirStSym64::new(MirOP::MirStSym64, wasted, imm, dst_ptr, zoff64).into_mir()
        }
    }
}

fn generate_store_to_global(
    src_mir: StrSrc,
    dst_global: MirGlobalRef,
    hi20_addr: GPR64,
) -> MirInst {
    let dst_symop = MirSymbolOp::Global(dst_global);
    match src_mir {
        StrSrc::F32(src) => {
            MirStrLitF32::new(MirOP::MirStrLitF32, hi20_addr, src, dst_symop).into_mir()
        }
        StrSrc::F64(src) => {
            MirStrLitF64::new(MirOP::MirStrLitF64, hi20_addr, src, dst_symop).into_mir()
        }
        StrSrc::G32(src) => {
            MirStrLitG32::new(MirOP::MirStrLitG32, hi20_addr, src, dst_symop).into_mir()
        }
        StrSrc::G64(src) => {
            MirStrLitG64::new(MirOP::MirStrLitG64, hi20_addr, src, dst_symop).into_mir()
        }
        StrSrc::Imm32(imm32) => {
            let wasted = GPR64::new_empty();
            MirStImm32Sym::new(MirOP::MirStImm32Sym, wasted, wasted, imm32, dst_global).into_mir()
        }
        StrSrc::Imm64(imm64) => {
            let wasted = GPR64::new_empty();
            MirStImm64Sym::new(MirOP::MirStImm64Sym, wasted, wasted, imm64, dst_global).into_mir()
        }
        StrSrc::Global(src) => {
            let wasted = GPR64::new_empty();
            MirStSym64Sym::new(
                MirOP::MirStSymSym,
                wasted,
                wasted,
                MirSymbolOp::Global(src),
                dst_global,
            )
            .into_mir()
        }
    }
}

#[derive(Debug, Clone, Copy)]
enum LdrDest {
    F32(FPR32),
    F64(FPR64),
    G32(GPR32),
    G64(GPR64),
}

impl LdrDest {
    fn from_reg_operand(op: RegOperand) -> Self {
        let RegOperand(id, si, uf, is_fp) = op;
        let bits_log2 = si.get_bits_log2();
        match (is_fp, bits_log2) {
            (true, 5) => LdrDest::F32(FPR32(id, uf)),
            (true, 6) => LdrDest::F64(FPR64(id, uf)),
            (false, 5) => LdrDest::G32(GPR32(id, uf)),
            (false, 6) => LdrDest::G64(GPR64(id, uf)),
            _ => panic!("Unsupported size for load: {bits_log2}"),
        }
    }
}

pub(super) fn dispatch_load(
    operand_map: &OperandMap<'_>,
    ir_ref: InstRef,
    vreg_alloc: &mut VirtRegAlloc,
    alloc_inst: &Slab<InstData>,
    alloc_use: Ref<Slab<UseData>>,
    out_insts: &mut VecDeque<MirInst>,
) {
    let InstData::Load(_, load) = ir_ref.to_data(alloc_inst) else {
        panic!("Expected Load instruction");
    };
    let src_ir = load.source.get_operand(&alloc_use);
    let src_mir = operand_map
        .find_operand_no_constdata(&src_ir)
        .expect("Failed to find source operand for load");
    let dst_mir = operand_map.find_operand_for_inst(ir_ref).unwrap();
    let dst_mir = match dst_mir {
        InstRetval::Reg(reg) => reg,
        InstRetval::Wasted => {
            debug!("Load instruction {ir_ref:?} is wasted, skipping load generation");
            return;
        }
    };
    let dst_kind = LdrDest::from_reg_operand(dst_mir);
    let ldr_inst = match dst_kind {
        LdrDest::F32(dst) => match src_mir {
            MirOperand::GPReg(gpreg) => LoadF32::new(
                MirOP::LdrF32,
                dst,
                GPR64::from_real(gpreg),
                GPR64::zr(),
                None,
            )
            .into_mir(),
            MirOperand::Label(label) => {
                LoadF32Literal::new(MirOP::LdrF32Literal, dst, MirSymbolOp::Label(label)).into_mir()
            }
            MirOperand::Global(globl) => {
                let tmp_addr = vreg_alloc.insert_gpr64(GPR64::new_empty());
                let src: MirSymbolOp = MirSymbolOp::Global(globl);
                MirLdrLitF32::new(MirOP::MirLdrLitF32, dst, tmp_addr, src).into_mir()
            }
            _ => panic!("Invalid source operand load from {src_mir:?} to {dst_mir:?}"),
        },
        LdrDest::F64(fpr64) => match src_mir {
            MirOperand::GPReg(gpreg) => LoadF64Base::new(
                MirOP::LdrF64Base,
                fpr64,
                GPR64::from_real(gpreg),
                ImmLSP64::new(0),
            )
            .into_mir(),
            MirOperand::Label(label) => {
                LoadF64Literal::new(MirOP::LdrF64Literal, fpr64, MirSymbolOp::Label(label))
                    .into_mir()
            }
            MirOperand::Global(globl) => {
                let tmp_addr = vreg_alloc.insert_gpr64(GPR64::new_empty());
                let src = MirSymbolOp::Global(globl);
                MirLdrLitF64::new(MirOP::MirLdrLitF64, fpr64, tmp_addr, src).into_mir()
            }
            _ => panic!("Invalid source operand load from {src_mir:?} to {dst_mir:?}"),
        },
        LdrDest::G32(gpr32) => match src_mir {
            MirOperand::GPReg(gpreg) => LoadGr32Base::new(
                MirOP::LdrGr32Base,
                gpr32,
                GPR64::from_real(gpreg),
                ImmLSP32::new(0),
            )
            .into_mir(),
            MirOperand::Label(label) => {
                LoadGr32Literal::new(MirOP::LdrGr32Literal, gpr32, MirSymbolOp::Label(label))
                    .into_mir()
            }
            MirOperand::Global(globl) => {
                let tmp_addr = GPR64::new(gpr32.get_id());
                let src = MirSymbolOp::Global(globl);
                MirLdrLitG32::new(MirOP::MirLdrLitG32, gpr32, tmp_addr, src).into_mir()
            }
            _ => panic!("Invalid source operand load from {src_mir:?} to {dst_mir:?}"),
        },
        LdrDest::G64(dst) => match src_mir {
            MirOperand::GPReg(gpreg) => LoadGr64Base::new(
                MirOP::LdrGr64Base,
                dst,
                GPR64::from_real(gpreg),
                ImmLSP64::new(0),
            )
            .into_mir(),
            MirOperand::Label(label) => {
                LoadGr64Literal::new(MirOP::LdrGr64Literal, dst, MirSymbolOp::Label(label))
                    .into_mir()
            }
            MirOperand::Global(globl) => {
                let src = MirSymbolOp::Global(globl);
                MirLdrLitG64::new(MirOP::MirLdrLitG64, dst, dst, src).into_mir()
            }
            MirOperand::SwitchTab(index) => {
                let src = MirSymbolOp::SwitchTab(index);
                MirLdrLitG64::new(MirOP::MirLdrLitG64, dst, dst, src).into_mir()
            }
            _ => panic!("Invalid source operand load from {src_mir:?} to {dst_mir:?}"),
        },
    };
    out_insts.push_back(ldr_inst);
}

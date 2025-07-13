use std::rc::Rc;

use crate::mir::{
    inst::inst::MirInst,
    module::{MirModule, func::MirFunc},
};

mod format_inst;
pub mod format_opcode;

pub struct FuncFormatContext<'a> {
    pub writer: &'a mut dyn std::fmt::Write,
    pub operand_context: OperandContext,
    pub mir_module: &'a MirModule,
}

pub struct OperandContext {
    pub is_fp: bool,
    pub current_func: Rc<MirFunc>,
}

impl std::fmt::Write for FuncFormatContext<'_> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.writer.write_str(s)
    }
    fn write_char(&mut self, c: char) -> std::fmt::Result {
        self.writer.write_char(c)
    }
    fn write_fmt(&mut self, args: std::fmt::Arguments<'_>) -> std::fmt::Result {
        self.writer.write_fmt(args)
    }
}

impl<'a> FuncFormatContext<'a> {
    pub fn new(
        writer: &'a mut dyn std::fmt::Write,
        curr_func: Rc<MirFunc>,
        mir_module: &'a MirModule,
    ) -> Self {
        Self {
            writer,
            operand_context: OperandContext {
                is_fp: false,
                current_func: curr_func,
            },
            mir_module,
        }
    }

    pub fn get_current_func(&self) -> Rc<MirFunc> {
        self.operand_context.current_func.clone()
    }

    pub fn set_fp(&mut self, is_fp: bool) {
        self.operand_context.is_fp = is_fp;
    }

    pub fn format_inst(&mut self, inst: &MirInst) -> std::fmt::Result {
        match inst {
            MirInst::MirFCopy64(_)
            | MirInst::MirFCopy32(_)
            | MirInst::FMov64I(_)
            | MirInst::FMov32I(_)
            | MirInst::LoadConstF64(_)
            | MirInst::LoadConst64Symbol(_) => self.set_fp(true),
            _ => self.set_fp(false),
        };
        match inst {
            MirInst::GuideNode(_) => Ok(()),
            MirInst::CondBr(cond_br) => format_inst::fmt_cond_br(self, inst.get_opcode(), cond_br),
            MirInst::CBZs(cbzs) => format_inst::fmt_cbzs(self, inst.get_opcode(), cbzs),
            MirInst::TBZ64(tbz64) => format_inst::fmt_tbz64(self, inst.get_opcode(), tbz64),
            MirInst::TBZ32(tbz32) => format_inst::fmt_tbz32(self, inst.get_opcode(), tbz32),
            MirInst::UncondBr(uncond_br) => {
                format_inst::fmt_uncond_br(self, inst.get_opcode(), uncond_br)
            }
            MirInst::BReg(breg) => format_inst::fmt_breg(self, inst.get_opcode(), breg),
            MirInst::BLinkLabel(blink_label) => {
                format_inst::fmt_blink_label(self, inst.get_opcode(), blink_label)
            }
            MirInst::BLinkReg(blink_reg) => {
                format_inst::fmt_blink_reg(self, inst.get_opcode(), blink_reg)
            }
            MirInst::ICmp64R(icmp64_r) => {
                format_inst::fmt_icmp64r(self, inst.get_opcode(), icmp64_r)
            }
            MirInst::ICmp32R(icmp32_r) => {
                format_inst::fmt_icmp32r(self, inst.get_opcode(), icmp32_r)
            }
            MirInst::ICmp64I(icmp64_i) => {
                format_inst::fmt_icmp64i(self, inst.get_opcode(), icmp64_i)
            }
            MirInst::ICmp32I(icmp32_i) => {
                format_inst::fmt_icmp32i(self, inst.get_opcode(), icmp32_i)
            }
            MirInst::FCmp32(fcmp32) => format_inst::fmt_fcmp32(self, inst.get_opcode(), fcmp32),
            MirInst::FCmp64(fcmp64) => format_inst::fmt_fcmp64(self, inst.get_opcode(), fcmp64),
            MirInst::ICCmp64R(iccmp64_r) => {
                format_inst::fmt_iccmp64r(self, inst.get_opcode(), iccmp64_r)
            }
            MirInst::ICCmp32R(iccmp32_r) => {
                format_inst::fmt_iccmp32r(self, inst.get_opcode(), iccmp32_r)
            }
            MirInst::ICCmp64I(iccmp64_i) => {
                format_inst::fmt_iccmp64i(self, inst.get_opcode(), iccmp64_i)
            }
            MirInst::ICCmp32I(iccmp32_i) => {
                format_inst::fmt_iccmp32i(self, inst.get_opcode(), iccmp32_i)
            }
            MirInst::FCCmp32(fccmp32) => format_inst::fmt_fccmp32(self, inst.get_opcode(), fccmp32),
            MirInst::FCCmp64(fccmp64) => format_inst::fmt_fccmp64(self, inst.get_opcode(), fccmp64),
            MirInst::Bin64R(bin64_r) => format_inst::fmt_bin64r(self, inst.get_opcode(), bin64_r),
            MirInst::Bin32R(bin32_r) => format_inst::fmt_bin32r(self, inst.get_opcode(), bin32_r),
            MirInst::MulL(mul_l) => format_inst::fmt_mull(self, inst.get_opcode(), mul_l),
            MirInst::Bin64RC(bin64_rc) => {
                format_inst::fmt_bin64rc(self, inst.get_opcode(), bin64_rc)
            }
            MirInst::Bin32RC(bin32_rc) => {
                format_inst::fmt_bin32rc(self, inst.get_opcode(), bin32_rc)
            }
            MirInst::Bin64RL(bin64_rl) => {
                format_inst::fmt_bin64rl(self, inst.get_opcode(), bin64_rl)
            }
            MirInst::Bin32RL(bin32_rl) => {
                format_inst::fmt_bin32rl(self, inst.get_opcode(), bin32_rl)
            }
            MirInst::Bin64RS(bin64_rs) => {
                format_inst::fmt_bin64rs(self, inst.get_opcode(), bin64_rs)
            }
            MirInst::Bin64RU(bin64_ru) => {
                format_inst::fmt_bin64ru(self, inst.get_opcode(), bin64_ru)
            }
            MirInst::Bin32RS(bin32_rs) => {
                format_inst::fmt_bin32rs(self, inst.get_opcode(), bin32_rs)
            }
            MirInst::Bin32RU(bin32_ru) => {
                format_inst::fmt_bin32ru(self, inst.get_opcode(), bin32_ru)
            }
            MirInst::Bin64RShift(bin64_rshift) => {
                format_inst::fmt_bin64rshift(self, inst.get_opcode(), bin64_rshift)
            }
            MirInst::Bin32RShift(bin32_rshift) => {
                format_inst::fmt_bin32rshift(self, inst.get_opcode(), bin32_rshift)
            }
            MirInst::BinF64R(bin_f64_r) => {
                format_inst::fmt_binf64r(self, inst.get_opcode(), bin_f64_r)
            }
            MirInst::BinF32R(bin_f32_r) => {
                format_inst::fmt_binf32r(self, inst.get_opcode(), bin_f32_r)
            }
            MirInst::MirCopy64(mir_copy64) => {
                format_inst::fmt_mir_copy64(self, inst.get_opcode(), mir_copy64)
            }
            MirInst::MirCopy32(mir_copy32) => {
                format_inst::fmt_mir_copy32(self, inst.get_opcode(), mir_copy32)
            }
            MirInst::MirFCopy64(mir_fcopy64) => {
                format_inst::fmt_mir_fcopy64(self, inst.get_opcode(), mir_fcopy64)
            }
            MirInst::MirFCopy32(mir_fcopy32) => {
                format_inst::fmt_mir_fcopy32(self, inst.get_opcode(), mir_fcopy32)
            }
            MirInst::MirPCopy(mir_pcopy) => {
                format_inst::fmt_mir_pcopy(self, inst.get_opcode(), mir_pcopy)
            }
            MirInst::Una64R(una64_r) => format_inst::fmt_una64_r(self, inst.get_opcode(), una64_r),
            MirInst::Una32R(una32_r) => format_inst::fmt_una32_r(self, inst.get_opcode(), una32_r),
            MirInst::ExtR(ext_r) => format_inst::fmt_ext_r(self, inst.get_opcode(), ext_r),
            MirInst::Mov64I(mov64_i) => format_inst::fmt_mov64_i(self, inst.get_opcode(), mov64_i),
            MirInst::Mov32I(mov32_i) => format_inst::fmt_mov32_i(self, inst.get_opcode(), mov32_i),
            MirInst::Adr(adr) => format_inst::fmt_adr(self, inst.get_opcode(), adr),
            MirInst::UnaFG64(una_fg64) => {
                format_inst::fmt_una_fg64(self, inst.get_opcode(), una_fg64)
            }
            MirInst::UnaGF64(una_gf64) => {
                format_inst::fmt_una_gf64(self, inst.get_opcode(), una_gf64)
            }
            MirInst::UnaF64G32(una_f64_g32) => {
                format_inst::fmt_una_f64_g32(self, inst.get_opcode(), una_f64_g32)
            }
            MirInst::UnaFG32(una_fg32) => {
                format_inst::fmt_una_fg32(self, inst.get_opcode(), una_fg32)
            }
            MirInst::UnaGF32(una_gf32) => {
                format_inst::fmt_una_gf32(self, inst.get_opcode(), una_gf32)
            }
            MirInst::UnaG64F32(una_g64_f32) => {
                format_inst::fmt_una_g64_f32(self, inst.get_opcode(), una_g64_f32)
            }
            MirInst::UnaG32F64(una_g32_f64) => {
                format_inst::fmt_una_g32_f64(self, inst.get_opcode(), una_g32_f64)
            }
            MirInst::UnaF64(una_f64) => format_inst::fmt_una_f64(self, inst.get_opcode(), una_f64),
            MirInst::UnaF32(una_f32) => format_inst::fmt_una_f32(self, inst.get_opcode(), una_f32),
            MirInst::UnaryF32F64(unary_f32_f64) => {
                format_inst::fmt_unary_f32_f64(self, inst.get_opcode(), unary_f32_f64)
            }
            MirInst::UnaryF64F32(unary_f64_f32) => {
                format_inst::fmt_unary_f64_f32(self, inst.get_opcode(), unary_f64_f32)
            }
            MirInst::FMov64I(fmov64_i) => {
                format_inst::fmt_fmov64_i(self, inst.get_opcode(), fmov64_i)
            }
            MirInst::FMov32I(fmov32_i) => {
                format_inst::fmt_fmov32_i(self, inst.get_opcode(), fmov32_i)
            }
            MirInst::TenaryG64(tenary_g64) => {
                format_inst::fmt_tenary_g64(self, inst.get_opcode(), tenary_g64)
            }
            MirInst::TenaryG64G32(tenary_g64_g32) => {
                format_inst::fmt_tenary_g64_g32(self, inst.get_opcode(), tenary_g64_g32)
            }
            MirInst::TenaryG32(tenary_g32) => {
                format_inst::fmt_tenary_g32(self, inst.get_opcode(), tenary_g32)
            }
            MirInst::TenaryF64(tenary_f64) => {
                format_inst::fmt_tenary_f64(self, inst.get_opcode(), tenary_f64)
            }
            MirInst::TenaryF32(tenary_f32) => {
                format_inst::fmt_tenary_f32(self, inst.get_opcode(), tenary_f32)
            }
            MirInst::LoadStoreGr64(load_store_gr64) => {
                format_inst::fmt_load_store_gr64(self, inst.get_opcode(), load_store_gr64)
            }
            MirInst::LoadStoreGr32(load_store_gr32) => {
                format_inst::fmt_load_store_gr32(self, inst.get_opcode(), load_store_gr32)
            }
            MirInst::LoadStoreF64(load_store_f64) => {
                format_inst::fmt_load_store_f64(self, inst.get_opcode(), load_store_f64)
            }
            MirInst::LoadStoreF32(load_store_f32) => {
                format_inst::fmt_load_store_f32(self, inst.get_opcode(), load_store_f32)
            }
            MirInst::LoadStoreGr64Base(load_store_gr64_base) => {
                format_inst::fmt_load_store_gr64_base(self, inst.get_opcode(), load_store_gr64_base)
            }
            MirInst::LoadStoreGr32Base(load_store_gr32_base) => {
                format_inst::fmt_load_store_gr32_base(self, inst.get_opcode(), load_store_gr32_base)
            }
            MirInst::LoadStoreF64Base(load_store_f64_base) => {
                format_inst::fmt_load_store_f64_base(self, inst.get_opcode(), load_store_f64_base)
            }
            MirInst::LoadStoreF32Base(load_store_f32_base) => {
                format_inst::fmt_load_store_f32_base(self, inst.get_opcode(), load_store_f32_base)
            }
            MirInst::LoadStoreGr64Indexed(load_store_gr64_indexed) => {
                format_inst::fmt_load_store_gr64_indexed(
                    self,
                    inst.get_opcode(),
                    load_store_gr64_indexed,
                )
            }
            MirInst::LoadStoreGr32Indexed(load_store_gr32_indexed) => {
                format_inst::fmt_load_store_gr32_indexed(
                    self,
                    inst.get_opcode(),
                    load_store_gr32_indexed,
                )
            }
            MirInst::LoadStoreF64Indexed(load_store_f64_indexed) => {
                format_inst::fmt_load_store_f64_indexed(
                    self,
                    inst.get_opcode(),
                    load_store_f64_indexed,
                )
            }
            MirInst::LoadStoreF32Indexed(load_store_f32_indexed) => {
                format_inst::fmt_load_store_f32_indexed(
                    self,
                    inst.get_opcode(),
                    load_store_f32_indexed,
                )
            }
            MirInst::LoadStoreGr64Literal(load_store_gr64_literal) => {
                format_inst::fmt_load_store_gr64_literal(
                    self,
                    inst.get_opcode(),
                    load_store_gr64_literal,
                )
            }
            MirInst::LoadStoreGr32Literal(load_store_gr32_literal) => {
                format_inst::fmt_load_store_gr32_literal(
                    self,
                    inst.get_opcode(),
                    load_store_gr32_literal,
                )
            }
            MirInst::LoadStoreF64Literal(load_store_f64_literal) => {
                format_inst::fmt_load_store_f64_literal(
                    self,
                    inst.get_opcode(),
                    load_store_f64_literal,
                )
            }
            MirInst::LoadStoreF32Literal(load_store_f32_literal) => {
                format_inst::fmt_load_store_f32_literal(
                    self,
                    inst.get_opcode(),
                    load_store_f32_literal,
                )
            }
            MirInst::LoadConst64(load_const64) => {
                format_inst::fmt_load_const64(self, inst.get_opcode(), load_const64)
            }
            MirInst::LoadConstF64(load_const_f64) => {
                format_inst::fmt_load_const_f64(self, inst.get_opcode(), load_const_f64)
            }
            MirInst::LoadConst64Symbol(load_const64_symbol) => {
                format_inst::fmt_load_const64_symbol(self, inst.get_opcode(), load_const64_symbol)
            }
            MirInst::MirCall(mir_call) => mir_call.fmt_asm(self),
            MirInst::MirReturn(mir_return) => mir_return.fmt_asm(self),
            MirInst::MirSwitch(mir_switch) => mir_switch.fmt_asm(self),
        }
    }
}

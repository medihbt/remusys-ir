use crate::mir::{
    fmt::{FuncFormatContext, format_opcode::opcode_get_name_str},
    inst::{
        // generated from `data/mir.rig`, please do not visit
        impls::*,
        // generated from `data/mir.rig`, please do not visit
        opcode::MirOP,
    },
    operand::{IMirSubOperand, reg::RegID},
};
use std::fmt::Write;

pub fn fmt_cond_br(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    cond_br: &CondBr,
) -> std::fmt::Result {
    let name = match opcode {
        MirOP::BCond => "b.",
        MirOP::BCCond => "bc.",
        _ => panic!("Unexpected opcode for conditional branch: {:?}", opcode),
    };
    write!(formatter, "{name}{} ", cond_br.get_cond().get_name())?;
    cond_br.get_label().fmt_asm(formatter)
}

pub fn fmt_cbzs(formatter: &mut FuncFormatContext, opcode: MirOP, cbzs: &CBZs) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    cbzs.get_cond().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    cbzs.get_target().fmt_asm(formatter)
}

pub fn fmt_tbz64(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    tbz64: &TBZ64,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    tbz64.get_cond().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    tbz64.get_bits().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    tbz64.get_target().fmt_asm(formatter)
}

pub fn fmt_tbz32(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    tbz32: &TBZ32,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    tbz32.get_cond().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    tbz32.get_bits().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    tbz32.get_target().fmt_asm(formatter)
}

// 无条件分支指令格式化
pub fn fmt_uncond_br(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    uncond_br: &UncondBr,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    uncond_br.get_target().fmt_asm(formatter)
}

// 寄存器分支指令格式化
pub fn fmt_breg(formatter: &mut FuncFormatContext, opcode: MirOP, breg: &BReg) -> std::fmt::Result {
    if opcode == MirOP::Ret && breg.get_target().get_id() == RegID::Phys(30) {
        // 特殊处理返回指令
        write!(formatter, "ret")
    } else {
        let name = opcode_get_name_str(opcode);
        write!(formatter, "{name} ")?;
        breg.get_target().fmt_asm(formatter)
    }
}

// 链接分支指令格式化
pub fn fmt_blink_label(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    blink_label: &BLinkLabel,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    blink_label.get_target().fmt_asm(formatter)
}

pub fn fmt_blink_reg(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    blink_reg: &BLinkReg,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    blink_reg.get_target().fmt_asm(formatter)
}

// 64位整数比较指令（寄存器操作数）
pub fn fmt_icmp64r(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    icmp64r: &ICmp64R,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    icmp64r.get_rn().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    icmp64r.get_rhs().fmt_asm(formatter)?;

    // 如果有寄存器操作修饰符，添加移位操作
    if let Some(rm_op) = icmp64r.get_rm_op() {
        write!(formatter, ", {rm_op}")?; // 暂时使用Debug格式
    }
    Ok(())
}

// 32位整数比较指令（寄存器操作数）
pub fn fmt_icmp32r(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    icmp32r: &ICmp32R,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    icmp32r.get_rn().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    icmp32r.get_rhs().fmt_asm(formatter)?;

    if let Some(rm_op) = icmp32r.get_rm_op() {
        write!(formatter, ", {rm_op}")?; // 暂时使用Debug格式
    }
    Ok(())
}

// 64位整数比较指令（立即数操作数）
pub fn fmt_icmp64i(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    icmp64i: &ICmp64I,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    icmp64i.get_rn().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    icmp64i.get_rhs().fmt_asm(formatter)
}

// 32位整数比较指令（立即数操作数）
pub fn fmt_icmp32i(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    icmp32i: &ICmp32I,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    icmp32i.get_rn().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    icmp32i.get_rhs().fmt_asm(formatter)
}

// 浮点比较指令
pub fn fmt_fcmp32(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    fcmp32: &FCmp32,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    fcmp32.get_rn().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    fcmp32.get_rhs().fmt_asm(formatter)
}

pub fn fmt_fcmp64(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    fcmp64: &FCmp64,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    fcmp64.get_rn().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    fcmp64.get_rhs().fmt_asm(formatter)
}

// 条件比较指令格式化
pub fn fmt_iccmp64r(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    iccmp64r: &ICCmp64R,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    iccmp64r.get_rn().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    iccmp64r.get_rhs().fmt_asm(formatter)?;
    write!(formatter, ", #{}, ", iccmp64r.get_nzcv().bits())?;
    write!(formatter, "{}", iccmp64r.get_cond().get_name())
}

pub fn fmt_iccmp32r(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    iccmp32r: &ICCmp32R,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    iccmp32r.get_rn().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    iccmp32r.get_rhs().fmt_asm(formatter)?;
    write!(formatter, ", #{}, ", iccmp32r.get_nzcv().bits())?;
    write!(formatter, "{}", iccmp32r.get_cond().get_name())
}

pub fn fmt_iccmp64i(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    iccmp64i: &ICCmp64I,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    iccmp64i.get_rn().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    iccmp64i.get_rhs().fmt_asm(formatter)?;
    write!(formatter, ", #{}, ", iccmp64i.get_nzcv().bits())?;
    write!(formatter, "{}", iccmp64i.get_cond().get_name())
}

pub fn fmt_iccmp32i(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    iccmp32i: &ICCmp32I,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    iccmp32i.get_rn().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    iccmp32i.get_rhs().fmt_asm(formatter)?;
    write!(formatter, ", #{}, ", iccmp32i.get_nzcv().bits())?;
    write!(formatter, "{}", iccmp32i.get_cond().get_name())
}

pub fn fmt_fccmp32(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    fccmp32: &FCCmp32,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    fccmp32.get_rn().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    fccmp32.get_rhs().fmt_asm(formatter)?;
    write!(formatter, ", #{}, ", fccmp32.get_nzcv().bits())?;
    write!(formatter, "{}", fccmp32.get_cond().get_name())
}

pub fn fmt_fccmp64(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    fccmp64: &FCCmp64,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    fccmp64.get_rn().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    fccmp64.get_rhs().fmt_asm(formatter)?;
    write!(formatter, ", #{}, ", fccmp64.get_nzcv().bits())?;
    write!(formatter, "{}", fccmp64.get_cond().get_name())
}

// 二元运算指令格式化
pub fn fmt_bin64r(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    bin64r: &Bin64R,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    bin64r.get_rd().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    bin64r.get_rn().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    bin64r.get_rm().fmt_asm(formatter)?;

    if let Some(rm_op) = bin64r.get_rm_op() {
        write!(formatter, ", {rm_op}")?;
    }
    Ok(())
}

pub fn fmt_bin32r(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    bin32r: &Bin32R,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    bin32r.get_rd().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    bin32r.get_rn().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    bin32r.get_rm().fmt_asm(formatter)?;

    if let Some(rm_op) = bin32r.get_rm_op() {
        write!(formatter, ", {rm_op}")?;
    }
    Ok(())
}

pub fn fmt_mull(formatter: &mut FuncFormatContext, opcode: MirOP, mull: &MulL) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    mull.get_rd().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    mull.get_rn().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    mull.get_rm().fmt_asm(formatter)
}

// 带立即数的二元运算指令
pub fn fmt_bin64rc(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    bin64rc: &Bin64RC,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    bin64rc.get_rd().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    bin64rc.get_rn().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    bin64rc.get_rm().fmt_asm(formatter)
}

pub fn fmt_bin32rc(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    bin32rc: &Bin32RC,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    bin32rc.get_rd().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    bin32rc.get_rn().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    bin32rc.get_rm().fmt_asm(formatter)
}

// 其他立即数变体
pub fn fmt_bin64rl(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    bin64rl: &Bin64RL,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    bin64rl.get_rd().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    bin64rl.get_rn().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    bin64rl.get_rm().fmt_asm(formatter)
}

pub fn fmt_bin32rl(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    bin32rl: &Bin32RL,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    bin32rl.get_rd().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    bin32rl.get_rn().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    bin32rl.get_rm().fmt_asm(formatter)
}

// SMax/SMin/UMax/UMin 变体
pub fn fmt_bin64rs(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    bin64rs: &Bin64RS,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    bin64rs.get_rd().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    bin64rs.get_rn().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    bin64rs.get_rm().fmt_asm(formatter)
}

pub fn fmt_bin64ru(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    bin64ru: &Bin64RU,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    bin64ru.get_rd().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    bin64ru.get_rn().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    bin64ru.get_rm().fmt_asm(formatter)
}

pub fn fmt_bin32rs(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    bin32rs: &Bin32RS,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    bin32rs.get_rd().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    bin32rs.get_rn().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    bin32rs.get_rm().fmt_asm(formatter)
}

pub fn fmt_bin32ru(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    bin32ru: &Bin32RU,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    bin32ru.get_rd().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    bin32ru.get_rn().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    bin32ru.get_rm().fmt_asm(formatter)
}

// 移位指令
pub fn fmt_bin64rshift(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    bin64rshift: &Bin64RShift,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    bin64rshift.get_rd().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    bin64rshift.get_rn().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    bin64rshift.get_rm().fmt_asm(formatter)
}

pub fn fmt_bin32rshift(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    bin32rshift: &Bin32RShift,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    bin32rshift.get_rd().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    bin32rshift.get_rn().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    bin32rshift.get_rm().fmt_asm(formatter)
}

// 浮点二元运算
pub fn fmt_binf64r(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    binf64r: &BinF64R,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    binf64r.get_rd().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    binf64r.get_rn().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    binf64r.get_rm().fmt_asm(formatter)
}

pub fn fmt_binf32r(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    binf32r: &BinF32R,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    binf32r.get_rd().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    binf32r.get_rn().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    binf32r.get_rm().fmt_asm(formatter)
}

pub fn fmt_mir_copy64(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    copy64: &MirCopy64,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    copy64.get_dst().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    copy64.get_src().fmt_asm(formatter)
}

pub fn fmt_mir_copy32(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    copy32: &MirCopy32,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    copy32.get_dst().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    copy32.get_src().fmt_asm(formatter)
}

pub fn fmt_mir_fcopy64(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    fcopy64: &MirFCopy64,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    fcopy64.get_dst().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    fcopy64.get_src().fmt_asm(formatter)
}

pub fn fmt_mir_fcopy32(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    fcopy32: &MirFCopy32,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    fcopy32.get_dst().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    fcopy32.get_src().fmt_asm(formatter)
}

pub fn fmt_mir_pcopy(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    pcopy64: &MirPCopy,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    pcopy64.get_dst().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    pcopy64.get_src().fmt_asm(formatter)
}

pub fn fmt_una64_r(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    una64_r: &Una64R,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    una64_r.get_dst().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    una64_r.get_src().fmt_asm(formatter)?;
    let rm_op = if let Some(rm_op) = una64_r.get_dst_op() {
        rm_op
    } else {
        return Ok(());
    };
    write!(formatter, ", {rm_op}")
}

pub fn fmt_una32_r(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    una32_r: &Una32R,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    una32_r.get_dst().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    una32_r.get_src().fmt_asm(formatter)?;
    let rm_op = if let Some(rm_op) = una32_r.get_dst_op() {
        rm_op
    } else {
        return Ok(());
    };
    write!(formatter, ", {rm_op}")
}

// ===== Extension operations =====
pub fn fmt_ext_r(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    ext_r: &ExtR,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    ext_r.get_dst().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    ext_r.get_src().fmt_asm(formatter)
}

// ===== Move immediate instructions =====
pub fn fmt_mov64_i(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    mov64_i: &Mov64I,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    mov64_i.get_dst().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    mov64_i.get_src().fmt_asm(formatter)
}

pub fn fmt_mov32_i(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    mov32_i: &Mov32I,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    mov32_i.get_dst().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    mov32_i.get_src().fmt_asm(formatter)
}

pub fn fmt_adr(formatter: &mut FuncFormatContext, opcode: MirOP, adr: &Adr) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    adr.get_dst().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    adr.get_src().fmt_asm(formatter)
}

// ===== Floating point conversions =====
pub fn fmt_una_fg64(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    una_fg64: &UnaFG64,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    una_fg64.get_dst().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    una_fg64.get_src().fmt_asm(formatter)
}

pub fn fmt_una_gf64(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    una_gf64: &UnaGF64,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    una_gf64.get_dst().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    una_gf64.get_src().fmt_asm(formatter)
}

pub fn fmt_una_f64_g32(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    una_f64_g32: &UnaF64G32,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    una_f64_g32.get_dst().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    una_f64_g32.get_src().fmt_asm(formatter)
}

pub fn fmt_una_fg32(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    una_fg32: &UnaFG32,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    una_fg32.get_dst().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    una_fg32.get_src().fmt_asm(formatter)
}

pub fn fmt_una_f32_g64(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    una_f32_g64: &UnaF32G64,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    una_f32_g64.get_dst().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    una_f32_g64.get_src().fmt_asm(formatter)
}

pub fn fmt_una_gf32(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    una_gf32: &UnaGF32,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    una_gf32.get_dst().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    una_gf32.get_src().fmt_asm(formatter)
}

pub fn fmt_una_g64_f32(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    una_g64_f32: &UnaG64F32,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    una_g64_f32.get_dst().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    una_g64_f32.get_src().fmt_asm(formatter)
}

pub fn fmt_una_g32_f64(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    una_g32_f64: &UnaG32F64,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    una_g32_f64.get_dst().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    una_g32_f64.get_src().fmt_asm(formatter)
}

pub fn fmt_una_f64(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    una_f64: &UnaF64,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    una_f64.get_dst().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    una_f64.get_src().fmt_asm(formatter)
}

pub fn fmt_una_f32(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    una_f32: &UnaF32,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    una_f32.get_dst().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    una_f32.get_src().fmt_asm(formatter)
}

pub fn fmt_unary_f32_f64(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    unary_f32_f64: &UnaryF32F64,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    unary_f32_f64.get_dst().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    unary_f32_f64.get_src().fmt_asm(formatter)
}

pub fn fmt_unary_f64_f32(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    unary_f64_f32: &UnaryF64F32,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    unary_f64_f32.get_dst().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    unary_f64_f32.get_src().fmt_asm(formatter)
}

pub fn fmt_fmov64_i(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    fmov64_i: &FMov64I,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    fmov64_i.get_dst().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    fmov64_i.get_src().fmt_asm(formatter)
}

pub fn fmt_fmov32_i(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    fmov32_i: &FMov32I,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    fmov32_i.get_dst().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    fmov32_i.get_src().fmt_asm(formatter)
}

// ===== Ternary operations =====
pub fn fmt_tenary_g64(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    tenary_g64: &TenaryG64,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    tenary_g64.get_rd().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    tenary_g64.get_rn().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    tenary_g64.get_rm().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    tenary_g64.get_rs().fmt_asm(formatter)
}

pub fn fmt_tenary_g64_g32(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    tenary_g64_g32: &TenaryG64G32,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    tenary_g64_g32.get_rd().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    tenary_g64_g32.get_rn().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    tenary_g64_g32.get_rm().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    tenary_g64_g32.get_rs().fmt_asm(formatter)
}

pub fn fmt_tenary_g32(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    tenary_g32: &TenaryG32,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    tenary_g32.get_rd().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    tenary_g32.get_rn().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    tenary_g32.get_rm().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    tenary_g32.get_rs().fmt_asm(formatter)
}

pub fn fmt_tenary_f64(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    tenary_f64: &TenaryF64,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    tenary_f64.get_rd().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    tenary_f64.get_rn().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    tenary_f64.get_rm().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    tenary_f64.get_rs().fmt_asm(formatter)
}

pub fn fmt_tenary_f32(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    tenary_f32: &TenaryF32,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    tenary_f32.get_rd().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    tenary_f32.get_rn().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    tenary_f32.get_rm().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    tenary_f32.get_rs().fmt_asm(formatter)
}

// ===== Load/Store RRR instructions =====
pub fn fmt_load_store_gr64(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    load_store_gr64: &LoadStoreGr64,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    load_store_gr64.get_rd().fmt_asm(formatter)?;
    write!(formatter, ", [")?;
    load_store_gr64.get_rn().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    load_store_gr64.get_rm().fmt_asm(formatter)?;
    if let Some(rm_op) = load_store_gr64.get_rm_op() {
        write!(formatter, ", {rm_op}")?;
    }
    write!(formatter, "]")
}

pub fn fmt_load_store_gr32(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    load_store_gr32: &LoadStoreGr32,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    load_store_gr32.get_rd().fmt_asm(formatter)?;
    write!(formatter, ", [")?;
    load_store_gr32.get_rn().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    load_store_gr32.get_rm().fmt_asm(formatter)?;
    if let Some(rm_op) = load_store_gr32.get_rm_op() {
        write!(formatter, ", {rm_op}")?;
    }
    write!(formatter, "]")
}

pub fn fmt_load_store_f64(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    load_store_f64: &LoadStoreF64,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    load_store_f64.get_rd().fmt_asm(formatter)?;
    write!(formatter, ", [")?;
    load_store_f64.get_rn().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    load_store_f64.get_rm().fmt_asm(formatter)?;
    if let Some(rm_op) = load_store_f64.get_rm_op() {
        write!(formatter, ", {rm_op}")?;
    }
    write!(formatter, "]")
}

pub fn fmt_load_store_f32(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    load_store_f32: &LoadStoreF32,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    load_store_f32.get_rd().fmt_asm(formatter)?;
    write!(formatter, ", [")?;
    load_store_f32.get_rn().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    load_store_f32.get_rm().fmt_asm(formatter)?;
    if let Some(rm_op) = load_store_f32.get_rm_op() {
        write!(formatter, ", {rm_op}")?;
    }
    write!(formatter, "]")
}

// ===== Load/Store Base Offset instructions =====
pub fn fmt_load_store_gr64_base(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    load_store_gr64_base: &LoadStoreGr64Base,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    load_store_gr64_base.get_rd().fmt_asm(formatter)?;
    write!(formatter, ", [")?;
    load_store_gr64_base.get_rn().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    load_store_gr64_base.get_rm().fmt_asm(formatter)?;
    write!(formatter, "]")
}

pub fn fmt_load_store_gr32_base(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    load_store_gr32_base: &LoadStoreGr32Base,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    load_store_gr32_base.get_rd().fmt_asm(formatter)?;
    write!(formatter, ", [")?;
    load_store_gr32_base.get_rn().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    load_store_gr32_base.get_rm().fmt_asm(formatter)?;
    write!(formatter, "]")
}

pub fn fmt_load_store_f64_base(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    load_store_f64_base: &LoadStoreF64Base,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    load_store_f64_base.get_rd().fmt_asm(formatter)?;
    write!(formatter, ", [")?;
    load_store_f64_base.get_rn().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    load_store_f64_base.get_rm().fmt_asm(formatter)?;
    write!(formatter, "]")
}

pub fn fmt_load_store_f32_base(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    load_store_f32_base: &LoadStoreF32Base,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    load_store_f32_base.get_rd().fmt_asm(formatter)?;
    write!(formatter, ", [")?;
    load_store_f32_base.get_rn().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    load_store_f32_base.get_rm().fmt_asm(formatter)?;
    write!(formatter, "]")
}

// ===== Load/Store Indexed instructions =====
pub fn fmt_load_store_gr64_indexed(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    load_store_gr64_indexed: &LoadStoreGr64Indexed,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    load_store_gr64_indexed.get_rd().fmt_asm(formatter)?;
    write!(formatter, ", [")?;
    load_store_gr64_indexed.get_rn().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    load_store_gr64_indexed.get_rm().fmt_asm(formatter)?;
    // aarch64 地址模式：PostIndex="]!", PreIndex="]!", NoOffset="]"
    match load_store_gr64_indexed.get_addr_mode() {
        // 假设 AddrMode 有合适的方法获取模式信息
        _ => write!(formatter, "]!")?, // 简化处理，实际需要根据具体枚举值判断
    }
    Ok(())
}

pub fn fmt_load_store_gr32_indexed(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    load_store_gr32_indexed: &LoadStoreGr32Indexed,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    load_store_gr32_indexed.get_rd().fmt_asm(formatter)?;
    write!(formatter, ", [")?;
    load_store_gr32_indexed.get_rn().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    load_store_gr32_indexed.get_rm().fmt_asm(formatter)?;
    match load_store_gr32_indexed.get_addr_mode() {
        _ => write!(formatter, "]!")?,
    }
    Ok(())
}

pub fn fmt_load_store_f64_indexed(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    load_store_f64_indexed: &LoadStoreF64Indexed,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    load_store_f64_indexed.get_rd().fmt_asm(formatter)?;
    write!(formatter, ", [")?;
    load_store_f64_indexed.get_rn().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    load_store_f64_indexed.get_rm().fmt_asm(formatter)?;
    match load_store_f64_indexed.get_addr_mode() {
        _ => write!(formatter, "]!")?,
    }
    Ok(())
}

pub fn fmt_load_store_f32_indexed(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    load_store_f32_indexed: &LoadStoreF32Indexed,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    load_store_f32_indexed.get_rd().fmt_asm(formatter)?;
    write!(formatter, ", [")?;
    load_store_f32_indexed.get_rn().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    load_store_f32_indexed.get_rm().fmt_asm(formatter)?;
    match load_store_f32_indexed.get_addr_mode() {
        _ => write!(formatter, "]!")?,
    }
    Ok(())
}

// ===== Load/Store Literal instructions =====
pub fn fmt_load_store_gr64_literal(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    load_store_gr64_literal: &LoadStoreGr64Literal,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    load_store_gr64_literal.get_rd().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    load_store_gr64_literal.get_from().fmt_asm(formatter)
}

pub fn fmt_load_store_gr32_literal(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    load_store_gr32_literal: &LoadStoreGr32Literal,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    load_store_gr32_literal.get_rd().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    load_store_gr32_literal.get_from().fmt_asm(formatter)
}

pub fn fmt_load_store_f64_literal(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    load_store_f64_literal: &LoadStoreF64Literal,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    load_store_f64_literal.get_rd().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    load_store_f64_literal.get_from().fmt_asm(formatter)
}

pub fn fmt_load_store_f32_literal(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    load_store_f32_literal: &LoadStoreF32Literal,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    load_store_f32_literal.get_rd().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    load_store_f32_literal.get_from().fmt_asm(formatter)
}

// ===== Load Constant instructions =====
pub fn fmt_load_const64(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    load_const64: &LoadConst64,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    load_const64.get_rd().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    load_const64.get_src().fmt_asm(formatter)
}

pub fn fmt_load_const_f64(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    load_const_f64: &LoadConstF64,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    load_const_f64.get_rd().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    load_const_f64.get_src().fmt_asm(formatter)
}

pub fn fmt_load_const64_symbol(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    load_const64_symbol: &LoadConst64Symbol,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    load_const64_symbol.get_rd().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    load_const64_symbol.get_src().fmt_asm(formatter)
}

pub fn fmt_csel64(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    csel64: &CSel64,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    csel64.get_rd().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    csel64.get_rn().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    csel64.get_rm().fmt_asm(formatter)?;
    write!(formatter, ", {}", csel64.get_cond().get_name())?;
    Ok(())
}

pub fn fmt_csel32(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    csel32: &CSel32,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    csel32.get_rd().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    csel32.get_rn().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    csel32.get_rm().fmt_asm(formatter)?;
    write!(formatter, ", {}", csel32.get_cond().get_name())?;
    Ok(())
}

pub fn fmt_csel_f64(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    csel_f64: &CSelF64,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    csel_f64.get_rd().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    csel_f64.get_rn().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    csel_f64.get_rm().fmt_asm(formatter)?;
    write!(formatter, ", {}", csel_f64.get_cond().get_name())?;
    Ok(())
}

pub fn fmt_csel_f32(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    csel_f32: &CSelF32,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    csel_f32.get_rd().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    csel_f32.get_rn().fmt_asm(formatter)?;
    write!(formatter, ", ")?;
    csel_f32.get_rm().fmt_asm(formatter)?;
    write!(formatter, ", {}", csel_f32.get_cond().get_name())?;
    Ok(())
}

pub fn fmt_cset64(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    cset64: &CSet64,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    cset64.get_rd().fmt_asm(formatter)?;
    write!(formatter, ", {}", cset64.get_cond().get_name())?;
    Ok(())
}

pub fn fmt_cset32(
    formatter: &mut FuncFormatContext,
    opcode: MirOP,
    cset32: &CSet32,
) -> std::fmt::Result {
    let name = opcode_get_name_str(opcode);
    write!(formatter, "{name} ")?;
    cset32.get_rd().fmt_asm(formatter)?;
    write!(formatter, ", {}", cset32.get_cond().get_name())?;
    Ok(())
}

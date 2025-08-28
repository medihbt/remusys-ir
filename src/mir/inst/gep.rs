use std::{cell::Cell, fmt::Write, u32};

use crate::{
    base::INullableValue,
    mir::{
        fmt::FuncFormatContext,
        inst::{IMirSubInst, MirInstCommon, impls::*, inst::MirInst, opcode::MirOP},
        module::MirGlobalRef,
        operand::{
            IMirSubOperand, MirOperand,
            compound::MirSymbolOp,
            imm::{Imm32, Imm64, ImmCalc},
            imm_traits,
            reg::{GPR32, GPR64, GPReg, RegUseFlags},
        },
    },
};

#[derive(Clone, Copy)]
pub enum MirGEPBase {
    Reg(GPR64),
    Sym(MirGlobalRef),
}

impl MirGEPBase {
    fn reset_uf(&mut self) {
        match self {
            MirGEPBase::Reg(GPR64(_, uf)) => *uf = RegUseFlags::USE,
            MirGEPBase::Sym(_) => {}
        }
    }

    pub fn matches_reg(&self, reg: GPR64) -> bool {
        match self {
            MirGEPBase::Reg(gpr64) => gpr64.same_pos_as(reg),
            MirGEPBase::Sym(_) => false,
        }
    }
}

impl std::fmt::Debug for MirGEPBase {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MirGEPBase::Reg(reg) => reg.fmt(f),
            MirGEPBase::Sym(sym) => sym.fmt(f),
        }
    }
}

impl IMirSubOperand for MirGEPBase {
    type RealRepresents = Self;

    fn new_empty() -> Self {
        Self::Sym(MirGlobalRef::new_null())
    }

    fn from_mir(mir: MirOperand) -> Self {
        match mir {
            MirOperand::GPReg(gpreg) => match GPR64::try_from_real(gpreg) {
                Some(reg) => MirGEPBase::Reg(reg),
                None => panic!(
                    "GEPBase can only be constructed from GPR64, but got: {:?}",
                    gpreg
                ),
            },
            MirOperand::Global(globl) => MirGEPBase::Sym(globl),
            _ => panic!(
                "GEPBase can only be constructed from GPR64 or MirGlobalRef, but got: {:?}",
                mir
            ),
        }
    }

    fn into_mir(self) -> MirOperand {
        match self {
            MirGEPBase::Reg(reg) => reg.into_mir(),
            MirGEPBase::Sym(sym) => sym.into_mir(),
        }
    }

    fn try_from_real(real: Self) -> Option<Self> {
        Some(real)
    }
    fn into_real(self) -> Self {
        self
    }

    fn insert_to_real(self, real: Self) -> Self {
        use MirGEPBase::*;
        match self {
            Reg(GPR64(id, _)) => match real {
                Reg(GPR64(_, uf)) => Reg(GPR64(id, uf)),
                Sym(_) => Reg(GPR64(id, RegUseFlags::USE)),
            },
            Sym(_) => self,
        }
    }

    fn fmt_asm(&self, formatter: &mut FuncFormatContext) -> std::fmt::Result {
        match self {
            MirGEPBase::Reg(reg) => reg.fmt_asm(formatter),
            MirGEPBase::Sym(sym) => sym.fmt_asm(formatter),
        }
    }
}

#[derive(Clone, Copy)]
pub enum MirGEPOffset {
    /// 64 位整数立即数偏移量
    /// 这里的 i64 是为了兼容负数偏移量, 但实际使用时通常是正数.
    Imm(i64),
    /// 64 位整数寄存器变体
    G64(GPR64),
    /// 32 位有符号整数寄存器变体
    S32(GPR32),
    /// 32 位无符号整数寄存器变体
    U32(GPR32),
}

impl MirGEPOffset {
    fn reset_uf(&mut self) {
        match self {
            MirGEPOffset::G64(GPR64(_, uf)) => *uf = RegUseFlags::USE,
            MirGEPOffset::S32(GPR32(_, uf)) => *uf = RegUseFlags::USE,
            MirGEPOffset::U32(GPR32(_, uf)) => *uf = RegUseFlags::USE,
            MirGEPOffset::Imm(_) => {}
        }
    }
}

impl std::fmt::Debug for MirGEPOffset {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        match self {
            MirGEPOffset::Imm(imm) => write!(f, "Imm({})", imm),
            MirGEPOffset::G64(reg) => reg.fmt(f),
            MirGEPOffset::S32(reg) => reg.fmt(f),
            MirGEPOffset::U32(reg) => reg.fmt(f),
        }
    }
}

impl IMirSubOperand for MirGEPOffset {
    type RealRepresents = MirGEPOffset;

    fn new_empty() -> Self {
        Self::Imm(0)
    }

    fn from_mir(mir: MirOperand) -> Self {
        match mir {
            MirOperand::GPReg(GPReg(id, si, mut uf)) => {
                let has_sxtw = uf.contains(RegUseFlags::SXTW);
                uf.remove(RegUseFlags::SXTW);
                match (si.get_bits_log2(), has_sxtw) {
                    (5, true) => Self::S32(GPR32(id, uf)),
                    (5, false) => Self::U32(GPR32(id, uf)),
                    (6, _) => Self::G64(GPR64(id, uf)),
                    _ => panic!(
                        "GEPOffset can only be constructed from GPR64 or GPR32, but got: {mir:?}",
                    ),
                }
            }
            MirOperand::Imm64(Imm64(imm, _)) => Self::Imm(imm as i64),
            MirOperand::Imm32(Imm32(imm, _)) => Self::Imm(imm as i64),
            _ => panic!(
                "GEPOffset can only be constructed from GPR64 or immediate, but got: {:?}",
                mir
            ),
        }
    }

    fn into_mir(self) -> MirOperand {
        match self {
            MirGEPOffset::Imm(imm) => MirOperand::Imm64(Imm64::full(imm as u64)),
            MirGEPOffset::G64(reg) => reg.into_mir(),
            MirGEPOffset::S32(mut reg) => {
                reg.1.insert(RegUseFlags::SXTW);
                reg.into_mir()
            }
            MirGEPOffset::U32(reg) => reg.into_mir(),
        }
    }

    fn try_from_real(real: Self) -> Option<Self> {
        Some(real)
    }
    fn into_real(self) -> Self {
        self
    }
    fn insert_to_real(self, real: Self) -> Self {
        use MirGEPOffset::*;
        match self {
            Imm(_) => self,
            G64(GPR64(id, _)) => match real {
                G64(GPR64(_, uf)) => G64(GPR64(id, uf)),
                S32(GPR32(_, uf)) | U32(GPR32(_, uf)) => G64(GPR64(id, uf)),
                Imm(_) => G64(GPR64(id, RegUseFlags::USE)),
            },
            S32(GPR32(id, _)) => match real {
                G64(GPR64(_, uf)) => S32(GPR32(id, uf)),
                S32(r) | U32(r) => {
                    let GPR32(_, uf) = r;
                    S32(GPR32(id, uf))
                }
                Imm(_) => S32(GPR32(id, RegUseFlags::USE)),
            },
            U32(GPR32(id, _)) => match real {
                G64(GPR64(_, uf)) => U32(GPR32(id, uf)),
                S32(r) | U32(r) => {
                    let GPR32(_, uf) = r;
                    U32(GPR32(id, uf))
                }
                Imm(_) => U32(GPR32(id, RegUseFlags::USE)),
            },
        }
    }

    fn fmt_asm(&self, formatter: &mut FuncFormatContext) -> std::fmt::Result {
        match self {
            MirGEPOffset::Imm(imm) => Imm64::full(*imm as u64).fmt_asm(formatter),
            MirGEPOffset::G64(reg) => reg.fmt_asm(formatter),
            MirGEPOffset::S32(reg) => reg.fmt_asm(formatter),
            MirGEPOffset::U32(reg) => reg.fmt_asm(formatter),
        }
    }
}

/// ### MIR GetElementPtr 指令
///
/// MIR GEP 指令用于计算指针的偏移量, 主要用于处理结构体和数组的成员访问.
///
/// #### 语法
///
/// `mir.gep %<Xd> through %<tmpreg> from %<base> [<offset0> x <weight0>, <offset1> x <weight1>, ...]`
#[derive(Clone)]
pub struct MirGEP {
    _common: MirInstCommon,
    _operands: Vec<Cell<MirOperand>>,
    _weights: Vec<u64>,
}

impl MirGEP {
    pub fn dst(&self) -> &Cell<MirOperand> {
        &self._operands[0]
    }
    pub fn get_dst(&self) -> GPR64 {
        GPR64::from_mir(self.dst().get())
    }
    pub fn set_dst(&self, dst: GPR64) {
        let GPR64(id, _) = dst;
        let dst = GPR64(id, RegUseFlags::DEF);
        self.dst().set(dst.into_mir());
    }

    pub fn tmpreg(&self) -> &Cell<MirOperand> {
        &self._operands[1]
    }
    pub fn get_tmpreg(&self) -> GPR64 {
        GPR64::from_mir(self.tmpreg().get())
    }
    pub fn set_tmpreg(&self, tmpreg: GPR64) {
        let GPR64(id, _) = tmpreg;
        let tmpreg = GPR64(id, RegUseFlags::DEF);
        self.tmpreg().set(tmpreg.into_mir());
    }

    pub fn base(&self) -> &Cell<MirOperand> {
        &self._operands[2]
    }
    pub fn get_base(&self) -> MirGEPBase {
        MirGEPBase::from_mir(self.base().get())
    }
    pub fn set_base(&self, base: MirGEPBase) {
        self.base().set(base.into_mir());
    }

    pub fn offsets(&self) -> &[Cell<MirOperand>] {
        debug_assert_eq!(self._operands.len() - 3, self._weights.len());
        &self._operands[3..]
    }
    pub fn weights(&self) -> &[u64] {
        debug_assert_eq!(self._operands.len() - 3, self._weights.len());
        &self._weights
    }

    pub fn get_offset_weight(&self, index: usize) -> (MirGEPOffset, u64) {
        let offset = self.offsets()[index].get();
        let weight = self.weights()[index];
        (MirGEPOffset::from_mir(offset), weight)
    }
    pub fn get_offset(&self, index: usize) -> Option<MirGEPOffset> {
        self.offsets()
            .get(index)
            .map(|op| MirGEPOffset::from_mir(op.get()))
    }
    pub fn get_weight(&self, index: usize) -> Option<u64> {
        self.weights().get(index).copied()
    }

    pub fn iter_offsets(&self) -> MirGEPOffsetIter<'_> {
        MirGEPOffsetIter::new(self)
    }

    /// #### 语法
    ///
    /// `mir.gep %<Xd> through %<tmpreg> from %<base> [<offset0> x <weight0>, <offset1> x <weight1>, ...]`
    pub fn fmt_asm(&self, f: &mut FuncFormatContext) -> std::fmt::Result {
        write!(f, "mir.gep ")?;
        self.get_dst().fmt_asm(f)?;
        write!(f, " through ")?;
        self.get_tmpreg().fmt_asm(f)?;
        write!(f, " from ")?;
        self.get_base().fmt_asm(f)?;
        if !self.offsets().is_empty() {
            write!(f, " [")?;
            for i in 0..self.offsets().len() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                let (offset, weight) = self.get_offset_weight(i);
                offset.fmt_asm(f)?;
                write!(f, " x {weight}")?;
            }
            write!(f, "]")?;
        }
        Ok(())
    }

    pub fn new(
        mut dst: GPR64,
        mut tmp: GPR64,
        mut base: MirGEPBase,
        offset_weight: impl IntoIterator<Item = (MirGEPOffset, u64)>,
    ) -> Self {
        dst.1 = RegUseFlags::DEF;
        tmp.1 = RegUseFlags::DEF;
        base.reset_uf();
        let mut operands =
            vec![Cell::new(dst.into_mir()), Cell::new(tmp.into_mir()), Cell::new(base.into_mir())];
        let mut weights = Vec::new();
        for (mut offset, weight) in offset_weight {
            offset.reset_uf();
            operands.push(Cell::new(offset.into_mir()));
            weights.push(weight);
        }
        MirGEP {
            _common: MirInstCommon::new(MirOP::MirGEP),
            _operands: operands,
            _weights: weights,
        }
    }
}

impl std::fmt::Debug for MirGEP {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let mut f = f.debug_struct("MirGEP");
        let (prev, next) = {
            let head = self._common.node_head.get();
            (head.prev, head.next)
        };
        f.field("prev", &prev)
            .field("next", &next)
            .field("op[0]=dst", &self.get_dst())
            .field("op[1]=tmp", &self.get_tmpreg())
            .field("op[2]=base", &self.get_base());

        struct OffsetFormatter(&'static str, Vec<(String, String)>);

        impl std::fmt::Debug for OffsetFormatter {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                let mut debug = f.debug_struct(self.0);
                for (field, value) in &self.1 {
                    debug.field(field, value);
                }
                debug.finish()
            }
        }
        let offsets = {
            let mut offsets = Vec::with_capacity(self._operands.len() - 3);
            for i in 0..self._weights.len() {
                let (offset, weight) = self.get_offset_weight(i);
                offsets.push((format!("off[{i}]"), format!("{offset:?} x {weight}")));
            }
            OffsetFormatter("Offsets", offsets)
        };
        f.field("op[..]=offsets", &offsets);
        f.finish()
    }
}

impl IMirSubInst for MirGEP {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }

    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..2]
    }

    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[2..]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        opcode == MirOP::MirGEP
    }

    fn new_empty(_: MirOP) -> Self {
        MirGEP {
            _common: MirInstCommon::new(MirOP::MirGEP),
            _operands: vec![
                Cell::new(MirOperand::GPReg(
                    GPR64(u32::MAX, RegUseFlags::DEF).into_real(),
                )),
                Cell::new(MirOperand::GPReg(
                    GPR64(u32::MAX, RegUseFlags::DEF).into_real(),
                )),
                Cell::new(MirOperand::Global(MirGlobalRef::new_empty())),
            ],
            _weights: Vec::new(),
        }
    }

    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        if let MirInst::MirGEP(gep) = mir_inst { Some(gep) } else { None }
    }
    fn into_mir(self) -> MirInst {
        MirInst::MirGEP(self)
    }
}

impl MirGEP {
    /// 合并所有的常量偏移量到一个单独的偏移量中.
    pub fn merge_const_offsets(&mut self) {
        let mut merged_offset = 0;
        let mut reg_offsets = Vec::with_capacity(self.weights().len());
        for (offset, weight) in self.iter_offsets() {
            if let MirGEPOffset::Imm(imm) = offset {
                merged_offset += imm * weight as i64;
            } else {
                reg_offsets.push((offset, weight));
            }
        }

        // 在原有的操作数列表中改来改去太麻烦了, 直接换一个新的
        let mut new_operands = Vec::with_capacity(3 + 1 + reg_offsets.len());
        let mut new_weights = Vec::with_capacity(1 + reg_offsets.len());

        // 按定义顺序添加原有固定位置的操作数
        new_operands.push(self.dst().clone());
        new_operands.push(self.tmpreg().clone());
        new_operands.push(self.base().clone());

        // 如果有合并的偏移量，则添加到新的操作数中
        if merged_offset != 0 {
            new_operands.push(Cell::new(MirGEPOffset::Imm(merged_offset).into_mir()));
            new_weights.push(1);
        }

        // 添加剩余的寄存器偏移量
        for (offset, weight) in reg_offsets {
            new_operands.push(Cell::new(offset.into_mir()));
            new_weights.push(weight);
        }

        // 替换旧的操作数列表
        self._operands = new_operands;
        self._weights = new_weights;
    }

    /// 尝试简化 GEP 指令为简单的 MOV 或加法/减法指令.
    ///
    /// TODO: 这个函数有 bug, 调用该函数得到的简化序列有 SIGSEGV, 需要修复.
    pub fn try_simplify(&self, mut consume_inst: impl FnMut(MirInst)) -> bool {
        if self.weights().is_empty() && self.offsets().is_empty() {
            self.simplify_to_mov(consume_inst);
            return true;
        }
        if self.weights().len() != 1 || self.offsets().len() != 1 {
            return false;
        }
        let (offset, weight) = self.get_offset_weight(0);
        let MirGEPOffset::Imm(offset) = offset else {
            return false; // 只有常量偏移量才能简化
        };
        let offset = offset * weight as i64;

        if offset == 0 {
            self.simplify_to_mov(consume_inst);
            true // 简化为 mov 指令
        } else if offset > 0 && imm_traits::is_calc_imm(offset as u64) {
            let offset = ImmCalc::new(offset as u32);
            let inst = self.simplify_simple_ptradd(offset, MirOP::Add64I, &mut consume_inst);
            consume_inst(inst.into_mir());
            true // 简化为加法指令
        } else if offset < 0 && imm_traits::is_calc_imm((-offset) as u64) {
            let offset = ImmCalc::new((-offset) as u32);
            let inst = self.simplify_simple_ptradd(offset, MirOP::Sub64I, &mut consume_inst);
            consume_inst(inst.into_mir());
            true // 简化为减法指令
        } else {
            false // 无法简化
        }
    }

    fn simplify_simple_ptradd(
        &self,
        offset: ImmCalc,
        opcode: MirOP,
        consume_inst: &mut impl FnMut(MirInst),
    ) -> Bin64RC {
        let dst = self.get_dst();
        let base = self.get_base();
        let base_reg = match base {
            MirGEPBase::Reg(reg) => reg,
            MirGEPBase::Sym(sym) => {
                let inst =
                    LoadConst64Symbol::new(MirOP::LoadConst64Symbol, dst, MirSymbolOp::Global(sym));
                consume_inst(inst.into_mir());
                dst
            }
        };
        let inst = Bin64RC::new(opcode, dst, base_reg, offset);
        inst
    }

    fn simplify_to_mov(&self, mut consume_inst: impl FnMut(MirInst)) {
        // 如果没有偏移量和权重，那就相当于直接 mov 了.
        let inst = match self.get_base() {
            MirGEPBase::Reg(gpr64) => {
                Una64R::new(MirOP::Mov64R, self.get_dst(), gpr64, None).into_mir()
            }
            MirGEPBase::Sym(gref) => LoadConst64Symbol::new(
                MirOP::LoadConst64Symbol,
                self.get_dst(),
                MirSymbolOp::Global(gref),
            )
            .into_mir(),
        };
        consume_inst(inst);
    }
}

pub struct MirGEPOffsetIter<'a> {
    offset: &'a [Cell<MirOperand>],
    weights: &'a [u64],
    index: usize,
}

impl<'a> MirGEPOffsetIter<'a> {
    pub fn new(gep: &'a MirGEP) -> Self {
        MirGEPOffsetIter {
            offset: &gep._operands[3..],
            weights: &gep._weights,
            index: 0,
        }
    }

    pub fn reset(&mut self) {
        self.index = 0;
    }

    pub fn get(&self) -> Option<(MirGEPOffset, u64)> {
        if self.index < self.offset.len() {
            let offset = MirGEPOffset::from_mir(self.offset[self.index].get());
            let weight = self.weights[self.index];
            Some((offset, weight))
        } else {
            None
        }
    }

    pub fn len(&self) -> usize {
        self.offset.len() - self.index
    }
}

impl<'a> Iterator for MirGEPOffsetIter<'a> {
    type Item = (MirGEPOffset, u64);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index < self.offset.len() {
            let offset = MirGEPOffset::from_mir(self.offset[self.index].get());
            let weight = self.weights[self.index];
            self.index += 1;
            Some((offset, weight))
        } else {
            None
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.offset.len();
        (len - self.index, Some(len - self.index))
    }
}

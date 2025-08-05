use crate::mir::{
    fmt::FuncFormatContext,
    inst::{IMirSubInst, MirInstCommon, inst::MirInst, opcode::MirOP},
    operand::{IMirSubOperand, MirOperand, reg::*},
    translate::mirgen::operandgen::DispatchedReg,
};
use std::{cell::Cell, fmt::Write};

#[derive(Debug, Clone)]
pub struct MirFuncPrologue {
    common: MirInstCommon,
    pub args: Vec<MirMappedArg>,
}

#[derive(Clone, Copy, PartialEq, Eq)]
pub enum MirMappedArg {
    /// 32 位整数寄存器, 直接通过寄存器传递: `(vreg, preg)`
    ///
    /// * `vreg`: 参数虚拟寄存器
    /// * `preg`: 实际对应的传参物理寄存器
    RegG32(GPR32, GPR32),
    /// 64 位整数寄存器, 直接通过寄存器传递: `(vreg, preg)`
    ///
    /// * `vreg`: 参数虚拟寄存器
    /// * `preg`: 实际对应的传参物理寄存器
    RegG64(GPR64, GPR64),
    /// 32 位浮点寄存器, 直接通过寄存器传递: `(vreg, preg)`
    ///
    /// * `vreg`: 参数虚拟寄存器
    /// * `preg`: 实际对应的传参物理寄存器
    RegF32(FPR32, FPR32),
    /// 64 位浮点寄存器, 直接通过寄存器传递: `(vreg, preg)`
    ///
    /// * `vreg`: 参数虚拟寄存器
    /// * `preg`: 实际对应的传参物理寄存器
    RegF64(FPR64, FPR64),
    /// 32 位整数寄存器, 但参数溢出到栈上: `(vreg, stackpos)`
    ///
    /// * `vreg`: 参数虚拟寄存器
    /// * `stackpos`: 参数在栈上的位置
    SpilledG32(GPR32, GPR64),
    /// 64 位整数寄存器, 但参数溢出到栈上: `(vreg, stackpos)`
    ///
    /// * `vreg`: 参数虚拟寄存器
    /// * `stackpos`: 参数在栈上的位置
    SpilledG64(GPR64, GPR64),
    /// 32 位浮点寄存器, 但参数溢出到栈上: `(vreg, stackpos)`
    ///
    /// * `vreg`: 参数虚拟寄存器
    /// * `stackpos`: 参数在栈上的位置
    SpilledF32(FPR32, GPR64),
    /// 64 位浮点寄存器, 但参数溢出到栈上: `(vreg, stackpos)`
    ///
    /// * `vreg`: 参数虚拟寄存器
    /// * `stackpos`: 参数在栈上的位置
    SpilledF64(FPR64, GPR64),
}

impl MirMappedArg {
    pub fn get_vreg(self) -> DispatchedReg {
        use DispatchedReg::*;
        use MirMappedArg::*;
        match self {
            RegG32(vreg, _) => G32(vreg),
            RegG64(vreg, _) => G64(vreg),
            RegF32(vreg, _) => F32(vreg),
            RegF64(vreg, _) => F64(vreg),
            SpilledG32(vreg, _) => G32(vreg),
            SpilledG64(vreg, _) => G64(vreg),
            SpilledF32(vreg, _) => F32(vreg),
            SpilledF64(vreg, _) => F64(vreg),
        }
    }

    pub fn try_get_stackpos(self) -> Option<GPR64> {
        match self {
            MirMappedArg::SpilledG32(_, stackpos) => Some(stackpos),
            MirMappedArg::SpilledG64(_, stackpos) => Some(stackpos),
            MirMappedArg::SpilledF32(_, stackpos) => Some(stackpos),
            MirMappedArg::SpilledF64(_, stackpos) => Some(stackpos),
            _ => None,
        }
    }
    pub fn try_get_preg(self) -> Option<DispatchedReg> {
        use DispatchedReg::*;
        use MirMappedArg::*;
        match self {
            RegG32(_, preg) => Some(G32(preg)),
            RegG64(_, preg) => Some(G64(preg)),
            RegF32(_, preg) => Some(F32(preg)),
            RegF64(_, preg) => Some(F64(preg)),
            _ => None,
        }
    }

    pub fn is_spilled(self) -> bool {
        use MirMappedArg::*;
        matches!(
            self,
            SpilledG32(..) | SpilledG64(..) | SpilledF32(..) | SpilledF64(..)
        )
    }
}

impl std::fmt::Debug for MirMappedArg {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::RegG32(arg0, arg1) => write!(f, "Reg({arg0:?}=>{arg1:?})"),
            Self::RegG64(arg0, arg1) => write!(f, "Reg({arg0:?}=>{arg1:?})"),
            Self::RegF32(arg0, arg1) => write!(f, "Reg({arg0:?}=>{arg1:?})"),
            Self::RegF64(arg0, arg1) => write!(f, "Reg({arg0:?}=>{arg1:?})"),
            Self::SpilledG32(arg0, arg1) => write!(f, "Spilled({arg0:?}:>{arg1:?})"),
            Self::SpilledG64(arg0, arg1) => write!(f, "Spilled({arg0:?}:>{arg1:?})"),
            Self::SpilledF32(arg0, arg1) => write!(f, "Spilled({arg0:?}:>{arg1:?})"),
            Self::SpilledF64(arg0, arg1) => write!(f, "Spilled({arg0:?}:>{arg1:?})"),
        }
    }
}

impl IMirSubInst for MirFuncPrologue {
    fn get_common(&self) -> &MirInstCommon {
        &self.common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self.common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &[]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &[]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        opcode == MirOP::MirFuncPrologue
    }
    fn new_empty(_: MirOP) -> Self {
        Self {
            common: MirInstCommon::new(MirOP::MirFuncPrologue),
            args: Vec::new(),
        }
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        if let MirInst::MirFuncPrologue(prologue) = mir_inst { Some(prologue) } else { None }
    }
    fn into_mir(self) -> MirInst {
        MirInst::MirFuncPrologue(self)
    }
}

impl MirFuncPrologue {
    pub fn from_func() -> Self {
        todo!()
    }

    /// ### Syntax:
    ///
    /// ```mir
    /// mir.prologue args = [ %v0 => $r0, %v1 => $r1, ... ]
    /// ```
    pub fn fmt_asm(&self, formatter: &mut FuncFormatContext) -> std::fmt::Result {
        write!(formatter, "mir.prologue [")?;
        for &arg in &self.args {
            let vreg = arg.get_vreg();
            formatter.write_str("%")?;
            vreg.fmt_asm(formatter)?;
            use MirMappedArg::*;
            match arg {
                RegG32(_, preg) => {
                    formatter.write_str(" => $")?;
                    preg.fmt_asm(formatter)?;
                }
                RegG64(_, preg) => {
                    formatter.write_str(" => $")?;
                    preg.fmt_asm(formatter)?;
                }
                RegF32(_, preg) => {
                    formatter.write_str(" => $")?;
                    preg.fmt_asm(formatter)?;
                }
                RegF64(_, preg) => {
                    formatter.write_str(" => $")?;
                    preg.fmt_asm(formatter)?;
                }
                SpilledG32(_, stackpos) => {
                    formatter.write_str(" = s[")?;
                    stackpos.fmt_asm(formatter)?;
                    formatter.write_str("]")?;
                }
                SpilledG64(_, stackpos) => {
                    formatter.write_str(" = s[")?;
                    stackpos.fmt_asm(formatter)?;
                    formatter.write_str("]")?;
                }
                SpilledF32(_, stackpos) => {
                    formatter.write_str(" = s[")?;
                    stackpos.fmt_asm(formatter)?;
                    formatter.write_str("]")?;
                }
                SpilledF64(_, stackpos) => {
                    formatter.write_str(" = s[")?;
                    stackpos.fmt_asm(formatter)?;
                    formatter.write_str("]")?;
                }
            }
        }
        Ok(())
    }
}

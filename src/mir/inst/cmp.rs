use crate::mir::{
    inst::{MirInstCommon, cond::MirCondFlag, opcode::MirOP},
    operand::{
        MirOperand,
        reg::{PhysReg, RegOP, RegUseFlags, VirtReg},
    },
};
use bitflags::bitflags;
use std::cell::Cell;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct NZCV: u8 {
        const N = 0b0001; // Negative flag
        const Z = 0b0010; // Zero flag
        const C = 0b0100; // Carry flag
        const V = 0b1000; // Overflow flag
    }
}

/**
  Compare or test instruction

  AArch64 assembly syntax:

  * `cmp-op rn, rm`
  * `cmp-op rn, rm, <shift flag> #shift`
  * `cmp-op rn, rm, <SXTX|SXTW|UXTW>`
  * `cmp-op rn, #imm`

  These syntaxes are mapped to the following Remusys-MIR syntaxes:

  * `cmp-op %rn, %rm, implicit-def $PState`
  * `cmp-op %rn, %rm, <shift flag> #shift, implicit-def $PState`
  * `cmp-op %rn, %rm, <SXTX|SXTW|UXTW>, implicit-def $PState`
  * `cmp-op %rn, #imm, implicit-def $PState`

  Accepts the following opcodes:

  ```aarch64
  cmp cmn tst
  fcmp fcmpe
  ```
*/
#[derive(Debug, Clone)]
pub struct CmpOP {
    pub common: MirInstCommon,
    /// `[lhs, rhs, implicit-def $PState]`
    pub operands: [Cell<MirOperand>; 3],
    /// Optional RHS additional operation, such as a shift or sign extension.
    pub rhs_op: Option<RegOP>,
}

impl CmpOP {
    pub fn new(opcode: MirOP, rhs_op: Option<RegOP>) -> Self {
        let operands = [
            Cell::new(MirOperand::VirtReg(VirtReg::new_long(0))), // Rn
            Cell::new(MirOperand::None),                          // Rm or immediate
            Cell::new(MirOperand::PhysReg(PhysReg::PState(
                RegUseFlags::IMPLICIT_DEF,
            ))), // Implicit CSR
        ];
        Self {
            common: MirInstCommon::new(opcode),
            operands,
            rhs_op,
        }
    }

    pub fn operands(&self) -> &[Cell<MirOperand>] {
        &self.operands
    }
    pub fn get_opcode(&self) -> MirOP {
        self.common.opcode
    }

    pub fn rn(&self) -> &Cell<MirOperand> {
        &self.operands[0]
    }
    pub fn rhs(&self) -> &Cell<MirOperand> {
        &self.operands[1]
    }
    pub fn implicit_csr(&self) -> &Cell<MirOperand> {
        &self.operands[2]
    }
}

/// Conditional comparation
///
/// AArch64 assembly syntax:
///
/// * `condcmp-op rn, rhs, #<nzcv>, cond`
///
/// These syntaxes are mapped to the following Remusys-MIR syntaxes:
///
/// * `condcmp-op %rn, %rm, #<nzcv>, <cond>, implicit-def $PState`
///
/// Accepts the following opcodes:
///
/// ```aarch64
/// fccmp fccmpe ccmn ccmp
/// ```
#[derive(Debug, Clone)]
pub struct CondCmpOP {
    pub common: MirInstCommon,
    /// `[lhs, rhs, implicit-def $PState]`
    pub operands: [Cell<MirOperand>; 3],
    pub cond: MirCondFlag,
    pub nzcv: NZCV,
}

impl CondCmpOP {
    pub fn new(opcode: MirOP, cond: MirCondFlag, nzcv: NZCV) -> Self {
        let operands = [
            Cell::new(MirOperand::VirtReg(VirtReg::new_long(0))), // Rn
            Cell::new(MirOperand::None),                          // Rm or immediate
            Cell::new(MirOperand::PhysReg(PhysReg::PState(
                RegUseFlags::IMPLICIT_DEF,
            ))), // Implicit CSR
        ];
        Self {
            common: MirInstCommon::new(opcode),
            operands,
            cond,
            nzcv,
        }
    }

    pub fn operands(&self) -> &[Cell<MirOperand>] {
        &self.operands
    }
    pub fn get_opcode(&self) -> MirOP {
        self.common.opcode
    }

    pub fn rn(&self) -> &Cell<MirOperand> {
        &self.operands[0]
    }
    pub fn rhs(&self) -> &Cell<MirOperand> {
        &self.operands[1]
    }
    pub fn implicit_csr(&self) -> &Cell<MirOperand> {
        &self.operands[2]
    }
}

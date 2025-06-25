use std::cell::Cell;

use crate::mir::{
    inst::{MirInstCommon, cond::MirCondFlag, opcode::MirOP},
    operand::{
        MirOperand,
        reg::{PhysReg, RegUseFlags},
    },
};

/// Conditional branch instruction in the MIR.
/// 
/// AArch64 Syntax: `<b.><cond> <label>`
///
/// Operand layout:
///
/// - `[0]`: Label to branch to.
/// - `[1]`: Implicit PSTATE register, used to hold the condition flags.
///   This is typically set to `PSTATE(IMPLICIT_DEF)` to indicate that the
///   condition flags are not explicitly defined in the instruction.
///
/// Accepts opcode:
///
/// ```aarch64
/// b.<cond> bc.<cond>
/// ```
#[derive(Debug, Clone)]
pub struct CondBr {
    pub(super) common: MirInstCommon,
    pub operands: [Cell<MirOperand>; 2],
    pub cond: MirCondFlag,
}

impl CondBr {
    pub fn new(opcode: MirOP, cond: MirCondFlag) -> Self {
        Self {
            common: MirInstCommon::new(opcode),
            operands: [
                Cell::new(MirOperand::None),
                Cell::new(MirOperand::PhysReg(PhysReg::PState(
                    RegUseFlags::IMPLICIT_DEF,
                ))),
            ],
            cond,
        }
    }

    pub fn get_opcode(&self) -> MirOP {
        self.common.opcode
    }
    pub fn label(&self) -> &Cell<MirOperand> {
        &self.operands[0]
    }
    pub fn implicit_pstate(&self) -> &Cell<MirOperand> {
        &self.operands[1]
    }
}

/// Unconditional branch instruction in the MIR.
/// 
/// AArch64 Syntax: `<br-opcode> <label>`
///
/// Operand layout:
/// - `[0]`: Label to branch to.
///
/// Accepts opcode:
///
/// ```aarch64
/// b
/// br
/// ```
#[derive(Debug, Clone)]
pub struct UncondBr {
    pub(super) common: MirInstCommon,
    pub operands: [Cell<MirOperand>; 1],
}

impl UncondBr {
    pub fn new(opcode: MirOP) -> Self {
        Self {
            common: MirInstCommon::new(opcode),
            operands: [Cell::new(MirOperand::None)],
        }
    }

    pub fn get_opcode(&self) -> MirOP {
        self.common.opcode
    }
    pub fn label(&self) -> &Cell<MirOperand> {
        &self.operands[0]
    }
}

/// Branch link instruction in the MIR.
///
/// Operand layout: `<bl-opcode> <label>`
///
/// - `[0]`: Label to branch to.
/// - `[1]`: Implicit `%ra` register, used to hold the return address.
///
/// Accepts opcode:
///
/// ```aarch64
/// bl blr
/// ```
#[derive(Debug, Clone)]
pub struct BLink {
    pub(super) common: MirInstCommon,
    pub operands: [Cell<MirOperand>; 2],
}

impl BLink {
    pub fn new(opcode: MirOP) -> Self {
        let mut ret_addr = PhysReg::return_addr();
        ret_addr.add_use_flag(RegUseFlags::IMPLICIT_DEF);
        Self {
            common: MirInstCommon::new(opcode),
            operands: [
                Cell::new(MirOperand::None),
                Cell::new(MirOperand::PhysReg(ret_addr)),
            ],
        }
    }

    pub fn get_opcode(&self) -> MirOP {
        self.common.opcode
    }
    pub fn target(&self) -> &Cell<MirOperand> {
        &self.operands[0]
    }
    pub fn implicit_ra(&self) -> &Cell<MirOperand> {
        &self.operands[1]
    }
}

/// Compare / Test and branch instruction in the MIR.
///
/// Operand layout:
///
/// - `[0]`: Register condition
/// - `[1]`: Branch target label
///
/// Aarch64 + MIR Assembly:
///
/// - `<opcode> <reg>, <label>`
///
/// Accepts opcode:
///
/// ```aarch64
/// cbz cbnz tbz tbnz
/// ```
#[derive(Debug, Clone)]
pub struct RegCondBr {
    pub(super) common: MirInstCommon,
    pub operands: [Cell<MirOperand>; 2],
}

impl RegCondBr {
    pub fn new(opcode: MirOP) -> Self {
        Self {
            common: MirInstCommon::new(opcode),
            operands: [
                Cell::new(MirOperand::None), // Register condition
                Cell::new(MirOperand::None), // Branch target label
            ],
        }
    }

    pub fn get_opcode(&self) -> MirOP {
        self.common.opcode
    }
    pub fn reg_cond(&self) -> &Cell<MirOperand> {
        &self.operands[0]
    }
    pub fn label(&self) -> &Cell<MirOperand> {
        &self.operands[1]
    }

    pub fn is_cbnz(&self) -> bool {
        matches!(self.get_opcode(), MirOP::CBNZ)
    }
    pub fn is_cbz(&self) -> bool {
        matches!(self.get_opcode(), MirOP::CBZ)
    }
    pub fn is_tbnz(&self) -> bool {
        matches!(self.get_opcode(), MirOP::TBNZ)
    }
    pub fn is_tbz(&self) -> bool {
        matches!(self.get_opcode(), MirOP::TBZ)
    }
}

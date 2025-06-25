use std::cell::Cell;

use crate::mir::{
    inst::{
        MirInstCommon,
        cond::MirCondFlag,
        opcode::{MirOP, OperandLayout},
    },
    operand::{
        MirOperand,
        reg::{PhysReg, RegOP, RegUseFlags, VirtReg},
    },
};

/**
 Binary Operation Instruction

 AArch64 assembly syntax (Do not show CSR operand if contains a CSR):

 * `binop rd, rn, rm`
 * `binop rd, rn, rm, <shift flag> #shift`
 * `binop rd, rn, rm, <SXTX|SXTW|UXTW>`
 * `binop rd, rn, #imm`
 * `binop rd, rn, #imm, <shift flag> #shift`
 * `binop rd, rn, rm, <SXTX|SXTW|UXTW>`

 These syntaxes are mapped to the following Remusys-MIR syntaxes:

 * `binop %rd, %rn, %rm`
 * `binop %rd, %rn, #imm`
 * `binop %rd, %rn, %rm,  implicit-def $PState`
 * `binop %rd, %rn, #imm, implicit-def $PState`

 3-operand mode accepts opcode:

 ```aarch64
 add sub
 smax smin umax umin
 and bic eon eor orr orn
 asr lsl lsr ror
 asrv lslv lsrv rorv
 mul mneg smnegl umnegl smull smulh umull umulhn sdiv udiv

 fadd fsub fmul fdiv fnmul fmax fmin fmaxnm fminnm
 ```

 3-operand with CSR mode accepts opcode:

 ```aarch64
 adds subs
 adc adcs sbc sbcs
 ands bics
 ```
*/
#[derive(Debug, Clone)]
pub struct BinOp {
    pub(super) common: MirInstCommon,
    operand_pool: [Cell<MirOperand>; 4],
    num_operands: u8,
    pub rhs_modifier: Option<RegOP>,
}

impl BinOp {
    pub fn new(opcode: MirOP, rhs_op: Option<RegOP>) -> Self {
        let noperands = match opcode.get_operand_layout() {
            OperandLayout::NoImplicit(3) => 3,
            OperandLayout::ImplicitCSR(3) => 4,
            _ => panic!(
                "Unexpected operand layout {:?} for BinOp {opcode:?}",
                opcode.get_operand_layout()
            ),
        };
        let operand_pool = [
            Cell::new(MirOperand::VirtReg(VirtReg::new_long(0))), // Rd
            Cell::new(MirOperand::VirtReg(VirtReg::new_long(0))), // Rn
            Cell::new(MirOperand::None),                          // Rm or immediate
            Cell::new(MirOperand::PhysReg(PhysReg::PState(
                RegUseFlags::IMPLICIT_DEF,
            ))), // CSR (if applicable)
        ];
        Self {
            common: MirInstCommon::new(opcode),
            operand_pool,
            num_operands: noperands,
            rhs_modifier: rhs_op,
        }
    }

    pub fn operands(&self) -> &[Cell<MirOperand>] {
        &self.operand_pool[..self.num_operands as usize]
    }
    pub fn get_opcode(&self) -> MirOP {
        self.common.opcode
    }

    pub fn rd(&self) -> &Cell<MirOperand> {
        &self.operand_pool[0]
    }
    pub fn rn(&self) -> &Cell<MirOperand> {
        &self.operand_pool[1]
    }
    pub fn rhs(&self) -> &Cell<MirOperand> {
        &self.operand_pool[2]
    }

    pub fn csr(&self) -> Option<&Cell<MirOperand>> {
        if self.num_operands == 4 {
            Some(&self.operand_pool[3])
        } else {
            None
        }
    }
    pub fn has_csr(&self) -> bool {
        self.num_operands == 4
    }
}

/// Move and unary operations
///
/// AArch64 assembly syntax:
///
/// * `unary-op rd, rn`
/// * `unary-op rd, rn, <shift flag> #shift`
/// * `unary-op rd, rn, <SXTX|SXTW|UXTW>`
/// * `unary-op rd, #imm`
/// * `adr-op rd, label`
///
/// These syntaxes are mapped to the following Remusys-MIR syntaxes:
///
/// * `unary-op %rd, %rn`
/// * `unary-op %rd, %rn, (<shift flag> #shift | SXTX|SXTW|UXTW)`
/// * `unary-op %rd, %rn, {<shift flag> #shift | SXTX|SXTW|UXTW,} implicit-def $PState`
///
/// * `unary-op %rd, #imm`
/// * `unary-op %rd, #imm, implicit-def $PState`
///
/// Accepts the following opcodes:
///
/// ```aarch64
/// movz movn movk mov
/// adrp adr
/// sxtb sxth sxtw uxtb uxth
/// abs neg
/// cls clz cnt ctz rbit rev{16, 32, 64}
///
/// fmov fcvt
/// fcvtas fcvtau fcvtms fcvtmu
/// fcvtns fcvtnu fcvtps fcvtpu
/// fcvtzs fcvtzu fjcvt.zs
/// scvtf  ucvtf
///
/// frint(a,i,m,n,p,x,z,32x,32z,64x,64z)
/// fabs fneg fsqrt
/// ```
///
/// Accepts the following opcodes with CSR:
///
/// ```aarch64
/// negs ngc ngcs
/// ```
#[derive(Debug, Clone)]
pub struct UnaryOp {
    pub(super) common: MirInstCommon,
    operand_pool: [Cell<MirOperand>; 3],
    num_operands: u8,
    pub rhs_modifier: Option<RegOP>,
}

impl UnaryOp {
    pub fn new(opcode: MirOP, rhs_op: Option<RegOP>) -> Self {
        let noperands = match opcode.get_operand_layout() {
            OperandLayout::NoImplicit(2) => 2,
            OperandLayout::ImplicitCSR(2) => 3,
            _ => panic!(
                "Unexpected operand layout {:?} for UnaryOp {opcode:?}",
                opcode.get_operand_layout()
            ),
        };
        let operand_pool = [
            Cell::new(MirOperand::VirtReg(VirtReg::new_long(0))), // Rd
            Cell::new(MirOperand::VirtReg(VirtReg::new_long(0))), // Rn or immediate
            Cell::new(MirOperand::PhysReg(PhysReg::PState(
                RegUseFlags::IMPLICIT_DEF,
            ))), // CSR (if applicable)
        ];
        Self {
            common: MirInstCommon::new(opcode),
            operand_pool,
            num_operands: noperands,
            rhs_modifier: rhs_op,
        }
    }

    pub fn operands(&self) -> &[Cell<MirOperand>] {
        &self.operand_pool[..self.num_operands as usize]
    }
    pub fn get_opcode(&self) -> MirOP {
        self.common.opcode
    }

    pub fn rd(&self) -> &Cell<MirOperand> {
        &self.operand_pool[0]
    }
    pub fn rn(&self) -> &Cell<MirOperand> {
        &self.operand_pool[1]
    }
    pub fn implicit_csr(&self) -> Option<&Cell<MirOperand>> {
        if self.num_operands == 3 {
            Some(&self.operand_pool[2])
        } else {
            None
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BFMMode {
    ImmrImms,
    LsbWidth,
}

/// BFM (Bit Field Move) operation
///
/// AArch64 & Remusys-IR assembly syntax:
///
/// * `bfn-op rd, rn, #immr, #imms`
/// * `bfn-op rd, rn, #lsb, #width`
///
/// Accepts the following opcodes:
///
/// ```aarch64
/// bfm sbfm ubfm
/// bfi sbfiz ubfiz
/// bfxil sbfx ubfx
/// ```
#[derive(Debug, Clone)]
pub struct BFMOp {
    pub(super) common: MirInstCommon,
    pub operands: [Cell<MirOperand>; 4], // rd, rn, immr, imms
    pub mode: BFMMode,
}

impl BFMOp {
    pub fn new(opcode: MirOP, mode: BFMMode) -> Self {
        let operands = [
            Cell::new(MirOperand::VirtReg(VirtReg::new_long(0))), // Rd
            Cell::new(MirOperand::VirtReg(VirtReg::new_long(0))), // Rn
            Cell::new(MirOperand::None),                          // ImmR or LSB
            Cell::new(MirOperand::None),                          // ImmS or Width
        ];
        Self {
            common: MirInstCommon::new(opcode),
            operands,
            mode,
        }
    }

    pub fn get_opcode(&self) -> MirOP {
        self.common.opcode
    }

    pub fn rd(&self) -> &Cell<MirOperand> {
        &self.operands[0]
    }
    pub fn rn(&self) -> &Cell<MirOperand> {
        &self.operands[1]
    }
    pub fn immr(&self) -> &Cell<MirOperand> {
        &self.operands[2]
    }
    pub fn imms(&self) -> &Cell<MirOperand> {
        &self.operands[3]
    }
}

/// Extended Register Operation (ExtROp)
///
/// AArch64 assembly syntax:
///
/// * `extr rd, rn, rm, #imm`
///
/// These syntaxes are mapped to the following Remusys-MIR syntaxes:
///
/// * `extr %rd, %rn, %rm, #imm`
#[derive(Debug, Clone)]
pub struct ExtROp {
    pub(super) common: MirInstCommon,
    /// [rd, rn, rm, imm]
    pub operands: [Cell<MirOperand>; 4],
}

impl ExtROp {
    pub fn new(opcode: MirOP) -> Self {
        let operands = [
            Cell::new(MirOperand::VirtReg(VirtReg::new_long(0))), // Rd
            Cell::new(MirOperand::VirtReg(VirtReg::new_long(0))), // Rn
            Cell::new(MirOperand::VirtReg(VirtReg::new_long(0))), // Rm
            Cell::new(MirOperand::ImmConst(0)),                   // Imm
        ];
        Self {
            common: MirInstCommon::new(opcode),
            operands,
        }
    }

    pub fn get_opcode(&self) -> MirOP {
        self.common.opcode
    }

    pub fn rd(&self) -> &Cell<MirOperand> {
        &self.operands[0]
    }
    pub fn rn(&self) -> &Cell<MirOperand> {
        &self.operands[1]
    }
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self.operands[2]
    }
    pub fn imm(&self) -> &Cell<MirOperand> {
        &self.operands[3]
    }
}

/// Ternary Operation Instruction
///
/// used for some instructions like multiplication or division.
///
/// AArch64 (same as Remusys-MIR) assembly syntax:
///
/// * `triop rd, rn, rm, ra`
///
/// Accepts the following opcodes:
///
/// ```aarch64
/// madd msub smaddl smsubl umaddl umsubl
/// fmadd fmsub fnmadd fnmsub
/// ```
#[derive(Debug, Clone)]
pub struct TernaryOp {
    pub(super) common: MirInstCommon,
    /// [rd, rn, rm, ra]
    pub operands: [Cell<MirOperand>; 4],
}

impl TernaryOp {
    pub fn new(opcode: MirOP) -> Self {
        let operands = [
            Cell::new(MirOperand::VirtReg(VirtReg::new_long(0))), // Rd
            Cell::new(MirOperand::VirtReg(VirtReg::new_long(0))), // Rn
            Cell::new(MirOperand::VirtReg(VirtReg::new_long(0))), // Rm
            Cell::new(MirOperand::VirtReg(VirtReg::new_long(0))), // Ra
        ];
        Self {
            common: MirInstCommon::new(opcode),
            operands,
        }
    }

    pub fn get_opcode(&self) -> MirOP {
        self.common.opcode
    }

    pub fn rd(&self) -> &Cell<MirOperand> {
        &self.operands[0]
    }
    pub fn rn(&self) -> &Cell<MirOperand> {
        &self.operands[1]
    }
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self.operands[2]
    }
    pub fn ra(&self) -> &Cell<MirOperand> {
        &self.operands[3]
    }
}

/// Conditional Select Instruction
///
/// AArch64 assembly syntax:
///
/// * `csel rd, rn, rm, <cond>`
///
/// These syntaxes are mapped to the following Remusys-MIR syntaxes:
///
/// * `csel %rd, %rn, %rm, <cond>, implicit-def $PState`
///
/// Accepts the following opcodes:
///
/// ```aarch64
/// csel csinc csinv csneg
/// fcsel
/// ```
#[derive(Debug, Clone)]
pub struct CondSelect {
    pub(super) common: MirInstCommon,
    /// [rd, rn, rm, implicit-def $PState]
    pub operands: [Cell<MirOperand>; 4],
    pub cond: MirCondFlag,
}

impl CondSelect {
    pub fn new(opcode: MirOP, cond: MirCondFlag) -> Self {
        let operands = [
            Cell::new(MirOperand::VirtReg(VirtReg::new_long(0))), // Rd
            Cell::new(MirOperand::VirtReg(VirtReg::new_long(0))), // Rn
            Cell::new(MirOperand::VirtReg(VirtReg::new_long(0))), // Rm
            Cell::new(MirOperand::PhysReg(PhysReg::PState(
                RegUseFlags::IMPLICIT_DEF,
            ))), // Implicit CSR
        ];
        Self {
            common: MirInstCommon::new(opcode),
            operands,
            cond,
        }
    }

    pub fn get_opcode(&self) -> MirOP {
        self.common.opcode
    }

    pub fn rd(&self) -> &Cell<MirOperand> {
        &self.operands[0]
    }
    pub fn rn(&self) -> &Cell<MirOperand> {
        &self.operands[1]
    }
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self.operands[2]
    }
    pub fn implicit_csr(&self) -> &Cell<MirOperand> {
        &self.operands[3]
    }
}

/// Conditional Unary Operation Instruction
///
/// AArch64 assembly syntax:
///
/// * `cunary-op rd, rn, <cond>`
///
/// These syntaxes are mapped to the following Remusys-MIR syntaxes:
///
/// * `cunary-op %rd, %rn, <cond>, implicit-def $PState`
///
/// Accepts the following opcodes:
///
/// ```aarch64
/// cinc cinv cneg
/// ```
#[derive(Debug, Clone)]
pub struct CondUnaryOp {
    pub(super) common: MirInstCommon,
    /// [rd, rn, implicit-def $PState]
    pub operands: [Cell<MirOperand>; 3],
    pub cond: MirCondFlag,
}

impl CondUnaryOp {
    pub fn new(opcode: MirOP, cond: MirCondFlag) -> Self {
        let operands = [
            Cell::new(MirOperand::VirtReg(VirtReg::new_long(0))), // Rd
            Cell::new(MirOperand::VirtReg(VirtReg::new_long(0))), // Rn
            Cell::new(MirOperand::PhysReg(PhysReg::PState(
                RegUseFlags::IMPLICIT_DEF,
            ))), // Implicit CSR
        ];
        Self {
            common: MirInstCommon::new(opcode),
            operands,
            cond,
        }
    }

    pub fn get_opcode(&self) -> MirOP {
        self.common.opcode
    }

    pub fn rd(&self) -> &Cell<MirOperand> {
        &self.operands[0]
    }
    pub fn rn(&self) -> &Cell<MirOperand> {
        &self.operands[1]
    }
    pub fn implicit_csr(&self) -> &Cell<MirOperand> {
        &self.operands[2]
    }
}

/// Conditional Set Instruction
///
/// AArch64 assembly syntax:
///
/// * `cset rd, <cond>`
///
/// These syntaxes are mapped to the following Remusys-MIR syntaxes:
///
/// * `cset %rd, <cond>, implicit-def $PState`
///
/// Accepts the following opcodes:
///
/// ```aarch64
/// cset csetm
/// ```
#[derive(Debug, Clone)]
pub struct CondSet {
    pub(super) common: MirInstCommon,
    /// [rd, implicit-def $PState]
    pub operands: [Cell<MirOperand>; 2],
    pub cond: MirCondFlag,
}

impl CondSet {
    pub fn new(opcode: MirOP, cond: MirCondFlag) -> Self {
        let operands = [
            Cell::new(MirOperand::VirtReg(VirtReg::new_long(0))), // Rd
            Cell::new(MirOperand::PhysReg(PhysReg::PState(
                RegUseFlags::IMPLICIT_DEF,
            ))), // Implicit CSR
        ];
        Self {
            common: MirInstCommon::new(opcode),
            operands,
            cond,
        }
    }

    pub fn get_opcode(&self) -> MirOP {
        self.common.opcode
    }

    pub fn rd(&self) -> &Cell<MirOperand> {
        &self.operands[0]
    }
    pub fn implicit_csr(&self) -> &Cell<MirOperand> {
        &self.operands[1]
    }
}

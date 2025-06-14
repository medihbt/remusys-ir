use std::{cell::Cell, ops::Range};

use crate::mir::{
    inst::{
        MachineInstCommonBase,
        opcode::{AArch64OP, NumOperand},
    },
    operand::{
        MachineOperand, RegOP, RegOperand, RegUseFlags, constant::ImmConst, physreg::PhysReg,
        virtreg::VirtReg,
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
 and eor orr
 ```

 3-operand with CSR mode accepts opcode:

 ```aarch64
 adds subs
 adc adcs sbc sbcs
 ands
 ```
*/
#[derive(Debug, Clone)]
pub struct BinOP {
    pub common: MachineInstCommonBase,
    pub rhs_op: Option<RegOP>,
    operands: [Cell<MachineOperand>; 4], // rd, rn, rm, csr (optional)
    noperands: u8,
}

impl BinOP {
    pub fn new_raw(opcode: AArch64OP, has_csr: bool) -> Self {
        let noperands = if has_csr { 4 } else { 3 };
        let ret = Self {
            common: MachineInstCommonBase::new(opcode),
            rhs_op: None,
            operands: [const { Cell::new(MachineOperand::None) }; 4],
            noperands: noperands as u8,
        };
        if has_csr {
            ret.operands[3].set(MachineOperand::PhysReg(PhysReg::PState(
                RegUseFlags::IMPLICIT_DEF,
            )));
        }
        ret
    }
    pub fn new_reg(opcode: AArch64OP, rd: RegOperand, rn: RegOperand, rm: RegOperand) -> Self {
        let (noperands, has_csr) = Self::get_operand_config(opcode);
        assert_eq!(
            noperands, 3,
            "BinOP must have 3 operands (rd, rn, rm) or 4 if has_csr"
        );
        let inst = Self::new_raw(opcode, has_csr);
        inst.ref_rd().set(rd.into());
        inst.ref_rn().set(rn.into());
        inst.ref_rhs().set(rm.into());
        inst
    }
    pub fn new_imm(opcode: AArch64OP, rd: RegOperand, rn: RegOperand, imm: ImmConst) -> Self {
        let (noperands, has_csr) = Self::get_operand_config(opcode);
        assert_eq!(
            noperands, 3,
            "BinOP must have 3 operands (rd, rn, imm) or 4 if has_csr"
        );
        let inst = Self::new_raw(opcode, has_csr);
        inst.ref_rd().set(rd.into());
        inst.ref_rn().set(rn.into());
        inst.ref_rhs().set(MachineOperand::ImmConst(imm));
        inst
    }
    fn get_operand_config(opcode: AArch64OP) -> (u32, bool) {
        match opcode.get_n_operands() {
            NumOperand::MustCSR(n) => (n, true),
            NumOperand::Fix(n) => (n, false),
            NumOperand::Ldr | NumOperand::MayCSR(_) | NumOperand::Dyn => panic!(
                "operand count {:?} of opcode {} not supported for BinOP",
                opcode.get_n_operands(),
                opcode.get_asm_name()
            ),
        }
    }

    pub fn get_common(&self) -> &MachineInstCommonBase {
        &self.common
    }
    pub fn common_mut(&mut self) -> &mut MachineInstCommonBase {
        &mut self.common
    }
    pub fn get_opcode(&self) -> AArch64OP {
        self.common.opcode
    }
    pub fn get_noperands(&self) -> u8 {
        self.noperands
    }
    pub fn operands(&self) -> &[Cell<MachineOperand>] {
        &self.operands[..self.noperands as usize]
    }

    pub fn has_csr(&self) -> bool {
        self.noperands == 4
    }
    pub fn get_csr(&self) -> Option<MachineOperand> {
        if self.has_csr() {
            Some(self.operands[3].get())
        } else {
            None
        }
    }
    pub fn set_csr(&mut self, csr: MachineOperand) {
        if self.has_csr() {
            self.operands[3].set(csr);
        }
    }

    pub fn ref_rd(&self) -> &Cell<MachineOperand> {
        &self.operands[0]
    }
    pub fn get_rd(&self) -> RegOperand {
        RegOperand::from_machine_operand_unwrap(self.operands[0].get())
    }
    pub fn set_rd_vreg(&mut self, mut rd: VirtReg) {
        rd.add_use_flag(RegUseFlags::DEF);
        self.operands[0].set(MachineOperand::VirtReg(rd));
    }
    pub fn set_rd_preg(&mut self, mut rd: PhysReg) {
        rd.add_use_flag(RegUseFlags::DEF);
        self.operands[0].set(MachineOperand::PhysReg(rd));
    }

    pub fn ref_rn(&self) -> &Cell<MachineOperand> {
        &self.operands[1]
    }
    pub fn get_rn(&self) -> RegOperand {
        RegOperand::from_machine_operand_unwrap(self.operands[1].get())
    }
    pub fn set_rn_vreg(&mut self, rn: VirtReg) {
        self.operands[1].set(MachineOperand::VirtReg(rn));
    }
    pub fn set_rn_preg(&mut self, rn: PhysReg) {
        self.operands[1].set(MachineOperand::PhysReg(rn));
    }

    pub fn ref_rhs(&self) -> &Cell<MachineOperand> {
        &self.operands[2]
    }
    pub fn get_rm(&self) -> Option<RegOperand> {
        RegOperand::from_machine_operand(self.operands[2].get())
    }
    pub fn get_imm(&self) -> Option<ImmConst> {
        match self.operands[2].get() {
            MachineOperand::ImmConst(imm) => Some(imm),
            _ => None,
        }
    }
    pub fn set_rm_vreg(&mut self, rm: VirtReg) {
        self.operands[2].set(MachineOperand::VirtReg(rm));
    }
    pub fn set_rm_preg(&mut self, rm: PhysReg) {
        self.operands[2].set(MachineOperand::PhysReg(rm));
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
  ```
*/
#[derive(Debug, Clone)]
pub struct CmpOP {
    pub common: MachineInstCommonBase,
    /// `[lhs, rhs, implicit-def $PState]`
    pub operands: [Cell<MachineOperand>; 3],
    /// Optional RHS additional operation, such as a shift or sign extension.
    pub rhs_op: Option<RegOP>,
}

impl CmpOP {
    pub fn new_raw(opcode: AArch64OP) -> Self {
        let ret = Self {
            common: MachineInstCommonBase::new(opcode),
            rhs_op: None,
            operands: [const { Cell::new(MachineOperand::None) }; 3],
        };
        ret.operands[2].set(MachineOperand::PhysReg(PhysReg::PState(
            RegUseFlags::IMPLICIT_DEF,
        )));
        ret
    }

    pub fn ref_rn(&self) -> &Cell<MachineOperand> {
        &self.operands[0]
    }
    pub fn get_rn(&self) -> RegOperand {
        RegOperand::from_machine_operand_unwrap(self.operands[0].get())
    }

    pub fn ref_rhs(&self) -> &Cell<MachineOperand> {
        &self.operands[1]
    }
    pub fn get_rhs(&self) -> RegOperand {
        RegOperand::from_machine_operand_unwrap(self.operands[1].get())
    }

    pub fn ref_csr(&self) -> &Cell<MachineOperand> {
        &self.operands[2]
    }
    pub fn get_csr(&self) -> PhysReg {
        match self.operands[2].get() {
            MachineOperand::PhysReg(PhysReg::PState(f)) => PhysReg::PState(f),
            _ => panic!("Expected a physical register operand"),
        }
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
/// ```
#[derive(Debug, Clone)]
pub struct UnaryOP {
    pub common: MachineInstCommonBase,
    pub rhs_op: Option<RegOP>,
    operands: [Cell<MachineOperand>; 3], // rd, rn, csr (optional)
    noperands: u8,
}

impl UnaryOP {
    pub fn new_raw(opcode: AArch64OP, has_csr: bool) -> Self {
        let noperands = if has_csr { 3 } else { 2 };
        let ret = Self {
            common: MachineInstCommonBase::new(opcode),
            rhs_op: None,
            operands: [const { Cell::new(MachineOperand::None) }; 3],
            noperands: noperands as u8,
        };
        if has_csr {
            ret.operands[2].set(MachineOperand::PhysReg(PhysReg::PState(
                RegUseFlags::IMPLICIT_DEF,
            )));
        }
        ret
    }

    pub fn new_reg(opcode: AArch64OP, rd: RegOperand, rn: RegOperand) -> Self {
        let (noperands, has_csr) = Self::get_operand_config(opcode);
        assert_eq!(
            noperands, 2,
            "UnaryOP must have 2 operands (rd, rn) or 3 if has_csr"
        );
        let inst = Self::new_raw(opcode, has_csr);
        inst.ref_rd().set(rd.into());
        inst.ref_rhs().set(rn.into());
        inst
    }

    pub fn new_imm(opcode: AArch64OP, rd: RegOperand, imm: ImmConst) -> Self {
        let (noperands, has_csr) = Self::get_operand_config(opcode);
        assert_eq!(
            noperands, 2,
            "UnaryOP must have 2 operands (rd, imm) or 3 if has_csr"
        );
        let inst = Self::new_raw(opcode, has_csr);
        inst.ref_rd().set(rd.into());
        inst.ref_rhs().set(MachineOperand::ImmConst(imm));
        inst
    }

    fn get_operand_config(opcode: AArch64OP) -> (u32, bool) {
        match opcode.get_n_operands() {
            NumOperand::MustCSR(n) => (n, true),
            NumOperand::Fix(n) => (n, false),
            NumOperand::Ldr | NumOperand::MayCSR(_) | NumOperand::Dyn => panic!(
                "operand count {:?} of opcode {} not supported for UnaryOP",
                opcode.get_n_operands(),
                opcode.get_asm_name()
            ),
        }
    }

    pub fn ref_rd(&self) -> &Cell<MachineOperand> {
        &self.operands[0]
    }
    pub fn get_rd(&self) -> RegOperand {
        RegOperand::from_machine_operand_unwrap(self.operands[0].get())
    }

    pub fn ref_rhs(&self) -> &Cell<MachineOperand> {
        &self.operands[1]
    }
    pub fn get_rn(&self) -> Option<RegOperand> {
        RegOperand::from_machine_operand(self.operands[1].get())
    }
    pub fn get_rhs_imm(&self) -> Option<ImmConst> {
        match self.operands[1].get() {
            MachineOperand::ImmConst(imm) => Some(imm),
            _ => None,
        }
    }

    pub fn has_csr(&self) -> bool {
        self.noperands == 3
    }
    pub fn get_csr(&self) -> Option<MachineOperand> {
        if self.has_csr() {
            Some(self.operands[2].get())
        } else {
            None
        }
    }
    pub fn set_csr(&mut self, csr: MachineOperand) {
        if self.has_csr() {
            self.operands[2].set(csr);
        }
    }

    pub fn get_operands(&self) -> &[Cell<MachineOperand>] {
        &self.operands[..self.noperands as usize]
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
    pub common: MachineInstCommonBase,
    pub operands: [Cell<MachineOperand>; 4], // rd, rn, immr, imms
    pub mode: BFMMode,
}

impl BFMOp {
    pub fn new_raw(opcode: AArch64OP, mode: BFMMode) -> Self {
        Self {
            common: MachineInstCommonBase::new(opcode),
            operands: [const { Cell::new(MachineOperand::None) }; 4],
            mode,
        }
    }
    pub fn new(opcode: AArch64OP, rd: RegOperand, rn: RegOperand, imm2: i8, imm3: i8) -> Self {
        let inst = Self::new_raw(opcode, Self::find_bfm_mode(opcode));
        inst.ref_rd().set(rd.into());
        inst.ref_rn().set(rn.into());
        inst.ref_imm2()
            .set(MachineOperand::ImmConst(ImmConst::I32(imm2 as i32)));
        inst.ref_imm3()
            .set(MachineOperand::ImmConst(ImmConst::I32(imm3 as i32)));
        inst
    }
    pub fn new_bfc(rd: RegOperand, lsb: i8, width: u8) -> Self {
        Self::new(
            AArch64OP::BFM,
            rd,
            RegOperand::VirtReg(VirtReg::Zero(RegUseFlags::IMPLICIT_DEF)),
            (-lsb) % rd.get_bits() as i8, // immr
            width as i8 - 1,
        )
    }

    pub fn ref_rd(&self) -> &Cell<MachineOperand> {
        &self.operands[0]
    }
    pub fn get_rd(&self) -> RegOperand {
        RegOperand::from_machine_operand_unwrap(self.operands[0].get())
    }

    pub fn ref_rn(&self) -> &Cell<MachineOperand> {
        &self.operands[1]
    }
    pub fn get_rn(&self) -> RegOperand {
        RegOperand::from_machine_operand_unwrap(self.operands[1].get())
    }

    pub fn ref_imm2(&self) -> &Cell<MachineOperand> {
        &self.operands[2]
    }
    pub fn get_imm2(&self) -> i8 {
        match self.operands[2].get() {
            MachineOperand::ImmConst(ImmConst::I32(imm)) => imm as i8,
            _ => panic!("Expected an immediate operand for immr"),
        }
    }

    pub fn ref_imm3(&self) -> &Cell<MachineOperand> {
        &self.operands[3]
    }
    pub fn get_imm3(&self) -> i8 {
        match self.operands[3].get() {
            MachineOperand::ImmConst(ImmConst::I32(imm)) => imm as i8,
            _ => panic!("Expected an immediate operand for imms"),
        }
    }

    pub fn get_bit_range(&self) -> Range<i8> {
        type O = AArch64OP;
        let imm2 = self.get_imm2();
        let imm3 = self.get_imm3();
        match self.common.opcode {
            O::BFM | O::SBFM | O::UBFM => imm2..imm3 + 1,
            O::BFI | O::SBFIZ | O::UBFIZ => {
                let begin = (-imm2) % self.get_rd().get_bits() as i8;
                let end = imm3 - 1;
                begin..end
            }
            O::BFXIL | O::SBFX | O::UBFX => imm2..imm2 + imm3,
            _ => panic!("Invalid opcode for BFMOp: {:?}", self.common.opcode),
        }
    }

    fn find_bfm_mode(opcode: AArch64OP) -> BFMMode {
        type O = AArch64OP;
        match opcode {
            O::BFM | O::SBFM | O::UBFM => BFMMode::ImmrImms,
            O::BFI | O::SBFIZ | O::UBFIZ | O::SBFX | O::UBFX | O::BFXIL => BFMMode::LsbWidth,
            _ => panic!("Invalid opcode for BFMOp: {:?}", opcode),
        }
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
    pub common: MachineInstCommonBase,
    pub operands: [Cell<MachineOperand>; 4], // rd, rn, rm, imm
}

impl ExtROp {
    pub fn new(opcode: AArch64OP, rd: RegOperand, rn: RegOperand, rm: RegOperand, imm: i8) -> Self {
        assert_eq!(opcode.get_n_operands(), NumOperand::Fix(4));
        let inst = Self {
            common: MachineInstCommonBase::new(opcode),
            operands: [const { Cell::new(MachineOperand::None) }; 4],
        };
        inst.ref_rd().set(rd.into());
        inst.ref_rn().set(rn.into());
        inst.ref_rm().set(rm.into());
        inst.ref_imm().set(MachineOperand::ImmConst(ImmConst::I32(imm as i32)));
        inst
    }

    pub fn ref_rd(&self) -> &Cell<MachineOperand> {
        &self.operands[0]
    }
    pub fn get_rd(&self) -> RegOperand {
        RegOperand::from_machine_operand_unwrap(self.operands[0].get())
    }

    pub fn ref_rn(&self) -> &Cell<MachineOperand> {
        &self.operands[1]
    }
    pub fn get_rn(&self) -> RegOperand {
        RegOperand::from_machine_operand_unwrap(self.operands[1].get())
    }

    pub fn ref_rm(&self) -> &Cell<MachineOperand> {
        &self.operands[2]
    }
    pub fn get_rm(&self) -> RegOperand {
        RegOperand::from_machine_operand_unwrap(self.operands[2].get())
    }

    pub fn ref_imm(&self) -> &Cell<MachineOperand> {
        &self.operands[3]
    }
    pub fn get_imm(&self) -> i8 {
        match self.operands[3].get() {
            MachineOperand::ImmConst(ImmConst::I32(imm)) => imm as i8,
            _ => panic!("Expected an immediate operand for imm"),
        }
    }
}
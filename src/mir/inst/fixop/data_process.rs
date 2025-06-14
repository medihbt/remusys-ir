use std::cell::Cell;

use crate::mir::{
    inst::{
        opcode::{AArch64OP, NumOperand}, MachineInstCommonBase
    },
    operand::{constant::ImmConst, physreg::PhysReg, virtreg::VirtReg, MachineOperand, RegOperand, RegShiftOP},
};

/**
 Binary Operation Instruction

 AArch64 assembly syntax (Do not show CSR operand if contains a CSR):

 * `binop rd, rn, rm`
 * `binop rd, rn, rm, <shift flag> #shift`
 * `binop rd, rn, #imm`
 * `binop rd, rn, #imm, <shift flag> #shift` (evaluated to a constant value)

 These syntaxes are mapped to the following Remusys-MIR syntaxes:

 * `binop %rd, %rn, %rm`
 * `binop %rd, %rn, #imm`
 * `binop %rd, %rn, %rm,  implicit-def $PState`
 * `binop %rd, %rn, #imm, implicit-def $PState`
*/
#[derive(Debug, Clone)]
pub struct BinOP {
    pub common: MachineInstCommonBase,
    pub shift_op: Option<RegShiftOP>,
    pub shift_bits: u8,
    operands: [Cell<MachineOperand>; 4], // rd, rn, rm, csr (optional)
    noperands: u8,
}

impl BinOP {
    pub fn new_raw(opcode: AArch64OP, has_csr: bool) -> Self {
        let noperands = if has_csr { 4 } else { 3 };
        Self {
            common: MachineInstCommonBase::new(opcode),
            shift_op: None,
            shift_bits: 0,
            operands: [const { Cell::new(MachineOperand::None) }; 4],
            noperands: noperands as u8,
        }
    }
    pub fn new_reg(opcode: AArch64OP, rd: RegOperand, rn: RegOperand, rm: RegOperand) -> Self {
        let (noperands, has_csr) = Self::get_operand_config(opcode);
        assert_eq!(noperands, 3, "BinOP must have 3 operands (rd, rn, rm) or 4 if has_csr");
        let inst = Self::new_raw(opcode, has_csr);
        inst.ref_rd().set(rd.into());
        inst.ref_rn().set(rn.into());
        inst.ref_rhs().set(rm.into());
        inst
    }
    pub fn new_imm(opcode: AArch64OP, rd: RegOperand, rn: RegOperand, imm: ImmConst) -> Self {
        let (noperands, has_csr) = Self::get_operand_config(opcode);
        assert_eq!(noperands, 3, "BinOP must have 3 operands (rd, rn, imm) or 4 if has_csr");
        let inst = Self::new_raw(opcode, has_csr);
        inst.ref_rd().set(rd.into());
        inst.ref_rn().set(rn.into());
        inst.ref_rhs().set(MachineOperand::ImmConst(imm));
        inst
    }
    fn get_operand_config(opcode: AArch64OP) -> (u32, bool) {
        match opcode.get_n_operands() {
            NumOperand::MustCSR(n) => (n, true),
            NumOperand::Fix    (n) => (n, false),
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
    pub fn set_rd_vreg(&mut self, rd: VirtReg) {
        self.operands[0].set(MachineOperand::VirtReg(rd));
    }
    pub fn set_rd_preg(&mut self, rd: PhysReg) {
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

    pub fn rm_should_shift(&self) -> bool {
        self.shift_op.is_some() && self.shift_bits > 0
    }
}

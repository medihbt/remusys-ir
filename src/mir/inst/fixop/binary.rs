use crate::mir::{
    inst::{MachineInstCommonBase, fixop::FixOPInst, opcode::AArch64OP},
    operand::MachineOperand,
};

pub struct BinaryOP(pub FixOPInst);

impl BinaryOP {
    pub fn new(opcode: AArch64OP, has_csr: bool) -> Self {
        let noperands = if has_csr { 3 } else { 2 };
        Self(FixOPInst::new(opcode, noperands))
    }

    pub fn get_commmon(&self) -> &MachineInstCommonBase {
        &self.0.common
    }
    pub fn common_mut(&mut self) -> &mut MachineInstCommonBase {
        &mut self.0.common
    }
    pub fn get_operands(&self) -> &[MachineOperand] {
        self.0.get_operands()
    }
    pub fn operands_mut(&mut self) -> &mut [MachineOperand] {
        self.0.operands_mut()
    }

    pub fn get_csr(&self) -> Option<&MachineOperand> {
        if self.0.get_noperands() == 3 {
            Some(&self.0.operand_arr[2])
        } else {
            None
        }
    }
    pub fn set_csr(&mut self, csr: MachineOperand) {
        if self.0.get_noperands() == 3 {
            self.0.operand_arr[2] = csr;
        } else {
            panic!("This BinaryOP does not have a CSR operand");
        }
    }
}

use crate::mir::{
    inst::{fixop::FixOPInst, opcode::AArch64OP},
    operand::{virtreg::VirtReg, MachineOperand},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressMode {
    BaseOnly,   // [Rn]
    BaseOffset, // [Rn, #imm]
    PreIndex,   // [Rn, Rm]
    PostIndex,  // [Rn], Rm
    Literal,    // [#imm]
}

/// aarch64 syntax:
///
/// ```aarch64
/// <loadop|storeop> <Rt>, [<Rn>, <Rm>]
/// <loadop|storeop> <Rt>, [<Rn>, <Rm>, <UXTW|SXTW|SXTX>]
/// <loadop|storeop> <Rt>, [<Rn>, <Rm>, LSL #<shift>]
/// ```
pub struct LoadStoreRRR(pub FixOPInst, pub AddressMode);

impl LoadStoreRRR {
    pub fn new(opcode: AArch64OP, addr_mode: AddressMode) -> Self {
        assert!(matches!(
            addr_mode,
            AddressMode::BaseOnly | AddressMode::BaseOffset | AddressMode::PostIndex
        ));
        LoadStoreRRR(FixOPInst::new(opcode, 3), addr_mode)
    }

    pub fn get_rt(&self) -> &MachineOperand {
        &self.0.get_operands()[0]
    }
    pub fn get_rn(&self) -> &MachineOperand {
        &self.0.get_operands()[1]
    }
    pub fn get_rm(&self) -> &MachineOperand {
        &self.0.get_operands()[2]
    }
    pub fn get_addr_mode(&self) -> AddressMode {
        self.1
    }
}

pub struct LoadStoreRX(pub FixOPInst, pub AddressMode);

impl LoadStoreRX {
    pub fn new(opcode: AArch64OP, addr_mode: AddressMode) -> Self {
        LoadStoreRX(FixOPInst::new(opcode, 2), addr_mode)
    }
    pub fn new_base_only(opcode: AArch64OP, rd: VirtReg, rn: VirtReg) -> Self {
        let mut inst = LoadStoreRX::new(opcode, AddressMode::BaseOnly);
        inst.0.operands_mut()[0] = MachineOperand::VirtReg(rd);
        inst.0.operands_mut()[1] = MachineOperand::VirtReg(rn);
        inst
    }
    pub fn new_base_offset(opcode: AArch64OP, rd: VirtReg, rn: VirtReg, imm: i32) -> Self {
        let mut inst = LoadStoreRX::new(opcode, AddressMode::BaseOffset);
        inst.0.operands_mut()[0] = MachineOperand::VirtReg(rd);
        // inst.0.operands_mut()[1] = MachineOperand::RegExpr(RegExpr::Offset(rn, imm));
        inst
    }
    pub fn get_rt(&self) -> &MachineOperand {
        &self.0.get_operands()[0]
    }
    pub fn get_rn(&self) -> &MachineOperand {
        &self.0.get_operands()[1]
    }
    pub fn get_addr_mode(&self) -> AddressMode {
        self.1
    }
}

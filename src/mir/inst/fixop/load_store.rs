use core::panic;
use std::cell::Cell;

use crate::mir::{
    block::MachineBlockRef,
    inst::{MachineInstCommonBase, fixop::FixOPInst, opcode::AArch64OP},
    operand::{
        MachineOperand, RegOP, RegOperand, RegUseFlags, constant::ImmConst, physreg::PhysReg,
        virtreg::VirtReg,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressMode {
    BaseOnly,   // [Rn]
    BaseOffset, // [Rn, #imm]
    PreIndex,   // [Rn, #imm]!
    PostIndex,  // [Rn], Rm
    Literal,    // <label>
}

/// aarch64 syntax:
///
/// ```aarch64
/// <loadop|storeop> <Rt>, [<Rn>, <Rm>]
/// <loadop|storeop> <Rt>, [<Rn>, <Rm>, <UXTW|SXTW|SXTX>]
/// <loadop|storeop> <Rt>, [<Rn>, <Rm>, LSL #<shift>]
/// ```
#[derive(Debug, Clone)]
pub struct LoadStoreRRR {
    pub common: MachineInstCommonBase,
    pub operands: [Cell<MachineOperand>; 3],
    pub rm_op: Option<RegOP>,
}

impl LoadStoreRRR {
    pub fn new_raw(opcode: AArch64OP, rm_op: RegOP) -> Self {
        LoadStoreRRR {
            common: MachineInstCommonBase::new(opcode),
            operands: [const { Cell::new(MachineOperand::None) }; 3],
            rm_op: Some(rm_op),
        }
    }
    pub fn new(
        opcode: AArch64OP,
        mut rt: RegOperand,
        rn: RegOperand,
        rm: RegOperand,
        rm_op: RegOP,
    ) -> Self {
        let inst = LoadStoreRRR::new_raw(opcode, rm_op);
        *rt.use_flags_mut() = if opcode.is_load() {
            RegUseFlags::DEF
        } else {
            RegUseFlags::NONE
        };
        inst.ref_rt().set(rt.into());
        inst.ref_rn().set(rn.into());
        inst.ref_rm().set(rm.into());
        inst
    }

    pub fn ref_rt(&self) -> &Cell<MachineOperand> {
        &self.operands[0]
    }
    pub fn get_rt(&self) -> RegOperand {
        RegOperand::from_machine_operand_unwrap(self.ref_rt().get())
    }
    pub fn set_rt_vreg(&self, vreg: VirtReg) {
        self.ref_rt().set(MachineOperand::VirtReg(vreg));
    }
    pub fn set_rt_preg(&self, preg: PhysReg) {
        self.ref_rt().set(MachineOperand::PhysReg(preg));
    }

    pub fn ref_rn(&self) -> &Cell<MachineOperand> {
        &self.operands[1]
    }
    pub fn get_rn(&self) -> RegOperand {
        RegOperand::from_machine_operand_unwrap(self.ref_rn().get())
    }
    pub fn set_rn_vreg(&self, vreg: VirtReg) {
        self.ref_rn().set(MachineOperand::VirtReg(vreg));
    }
    pub fn set_rn_preg(&self, preg: PhysReg) {
        self.ref_rn().set(MachineOperand::PhysReg(preg));
    }

    pub fn ref_rm(&self) -> &Cell<MachineOperand> {
        &self.operands[2]
    }
    pub fn get_rm(&self) -> Option<RegOperand> {
        RegOperand::from_machine_operand(self.ref_rm().get())
    }
    pub fn set_rm_vreg(&self, vreg: VirtReg) {
        self.ref_rm().set(MachineOperand::VirtReg(vreg));
    }
    pub fn set_rm_preg(&self, preg: PhysReg) {
        self.ref_rm().set(MachineOperand::PhysReg(preg));
    }

    pub fn get_addr_mode(&self) -> AddressMode {
        if let Some(_) = &self.rm_op {
            AddressMode::BaseOffset // Rm present means BaseOffset
        } else {
            AddressMode::BaseOnly // No Rm means BaseOnly
        }
    }
}

/**
 Syntax:

 * `<loadop|storeop> <Rt>, [<Rn>]`             => AddressMode::BaseOnly
 * `<loadop|storeop> <Rt>, [<Rn>, #<simm>]`    => AddressMode::BaseOffset
 * `<loadop|storeop> <Rt>, [<Rn>, #<simm>]!`   => AddressMode::PreIndex
 * `<loadop|storeop> <Rt>, [<Rn>], #<simm>`    => AddressMode::PostIndex
 * `<loadop|storeop> <Rt>, <label>`            => AddressMode::Literal
*/
#[derive(Debug, Clone)]
pub struct LoadStoreRX(pub FixOPInst, pub AddressMode);

impl LoadStoreRX {
    pub fn new(opcode: AArch64OP, addr_mode: AddressMode) -> Self {
        let noperands = match addr_mode {
            AddressMode::BaseOnly | AddressMode::Literal => 2,
            AddressMode::BaseOffset | AddressMode::PreIndex | AddressMode::PostIndex => 3,
        };
        LoadStoreRX(FixOPInst::new(opcode, noperands), addr_mode)
    }
    pub fn new_base_only(opcode: AArch64OP, rd: RegOperand, rn: RegOperand) -> Self {
        let inst = LoadStoreRX::new(opcode, AddressMode::BaseOnly);
        inst.0.operands()[0].set(rd.into());
        inst.0.operands()[1].set(rn.into());
        inst
    }
    pub fn new_base_offset(opcode: AArch64OP, rd: RegOperand, rn: RegOperand, imm: i64) -> Self {
        let inst = LoadStoreRX::new(opcode, AddressMode::BaseOffset);
        let operands = inst.0.operands();
        operands[0].set(rd.into());
        operands[1].set(rn.into());
        operands[2].set(MachineOperand::ImmConst(ImmConst::I64(imm)));
        inst
    }
    pub fn new_pre_index(opcode: AArch64OP, rd: RegOperand, rn: RegOperand, imm: i64) -> Self {
        let inst = LoadStoreRX::new(opcode, AddressMode::PreIndex);
        let operands = inst.0.operands();
        operands[0].set(rd.into());
        operands[1].set(rn.into());
        operands[2].set(MachineOperand::ImmConst(ImmConst::I64(imm)));
        inst
    }
    pub fn new_post_index(
        opcode: AArch64OP,
        rd: RegOperand,
        rn: RegOperand,
        rm: RegOperand,
    ) -> Self {
        let inst = LoadStoreRX::new(opcode, AddressMode::PostIndex);
        let operands = inst.0.operands();
        operands[0].set(rd.into());
        operands[1].set(rn.into());
        operands[2].set(rm.into());
        inst
    }
    pub fn new_literal(opcode: AArch64OP, rd: RegOperand, label: MachineBlockRef) -> Self {
        let inst = LoadStoreRX::new(opcode, AddressMode::Literal);
        let operands = inst.0.operands();
        operands[0].set(rd.into());
        operands[1].set(MachineOperand::Label(label));
        inst
    }

    pub fn get_rt(&self) -> &Cell<MachineOperand> {
        &self.0.operands()[0]
    }
    pub fn get_rt_reg(&self) -> RegOperand {
        match self.get_rt().get() {
            MachineOperand::VirtReg(vreg) => RegOperand::VirtReg(vreg),
            MachineOperand::PhysReg(preg) => RegOperand::PhysReg(preg),
            _ => panic!(
                "Expected a register operand for RT, found: {:?}",
                self.get_rt().get()
            ),
        }
    }
    pub fn set_rt_vreg(&self, vreg: VirtReg) {
        self.get_rt().set(MachineOperand::VirtReg(vreg));
    }
    pub fn set_rt_preg(&self, preg: PhysReg) {
        self.get_rt().set(MachineOperand::PhysReg(preg));
    }

    pub fn get_rn_or_label(&self) -> &Cell<MachineOperand> {
        &self.0.operands()[1]
    }
    pub fn get_rn(&self) -> Option<RegOperand> {
        match self.get_rn_or_label().get() {
            MachineOperand::VirtReg(vreg) => Some(RegOperand::VirtReg(vreg)),
            MachineOperand::PhysReg(preg) => Some(RegOperand::PhysReg(preg)),
            MachineOperand::Label(_) => None, // RN is a label in literal mode
            _ => panic!(
                "Expected a register or label operand for RN, found: {:?}",
                self.get_rn_or_label().get()
            ),
        }
    }
    pub fn set_rn_vreg(&self, vreg: VirtReg) {
        self.get_rn_or_label().set(MachineOperand::VirtReg(vreg));
    }
    pub fn set_rn_preg(&self, preg: PhysReg) {
        self.get_rn_or_label().set(MachineOperand::PhysReg(preg));
    }
    pub fn get_label(&self) -> Option<MachineBlockRef> {
        if let MachineOperand::Label(label) = self.get_rn_or_label().get() {
            Some(label)
        } else {
            None
        }
    }
    pub fn set_label(&self, label: MachineBlockRef) {
        self.get_rn_or_label().set(MachineOperand::Label(label));
    }

    pub fn get_imm(&self) -> Option<&Cell<MachineOperand>> {
        match self.1 {
            AddressMode::BaseOffset | AddressMode::PreIndex | AddressMode::PostIndex => {
                Some(&self.0.operands()[2])
            }
            _ => None,
        }
    }
    pub fn get_addr_mode(&self) -> AddressMode {
        self.1
    }
}

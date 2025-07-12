#![doc = r" Remusys-MIR Instruction Definitions"]
#![doc = r" NOTE: This file is auto-generated from the RIG-DSL definitions."]
use crate::mir::{
    inst::{addr::*, cond::*, *},
    module::{block::*, func::*, global::*},
    operand::{compound::*, imm::*, reg::*, subop::*, MirOperand},
};
#[derive(Debug, Clone)]
pub struct UncondBr {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 1usize],
}
impl IMirSubInst for UncondBr {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..0usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[0usize..1usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::B)
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(MirBlockRef::new_empty().into_mir())],
        }
    }
}
impl UncondBr {
    pub fn new(opcode: MirOP, target: MirBlockRef) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(target.into_mir())],
        }
    }
    #[doc = "operand target at index 0 of type MirBlockRef"]
    pub fn target(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand target at index 0 of type MirBlockRef"]
    pub fn get_target(&self) -> MirBlockRef {
        MirBlockRef::from_mir(self.target().get())
    }
    #[doc = "set the value of operand target at 0 to a value of type MirBlockRef (checked by MirBlockRef)"]
    pub fn set_target(&self, value: MirBlockRef) {
        self.target().set(value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct BReg {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 1usize],
}
impl IMirSubInst for BReg {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..0usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[0usize..1usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::Br)
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(GPReg::new_empty().into_mir())],
        }
    }
}
impl BReg {
    pub fn new(opcode: MirOP, target: GPR64) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(target.into_mir())],
        }
    }
    #[doc = "operand target at index 0 of type GPReg"]
    pub fn target(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand target at index 0 of type GPReg"]
    pub fn get_target(&self) -> GPReg {
        GPReg::from_mir(self.target().get())
    }
    #[doc = "set the value of operand target at 0 to a value of type GPReg (checked by GPR64)"]
    pub fn set_target(&self, value: GPReg) {
        let prev_value = self.get_target();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.target().set(next_value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct BLinkLabel {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for BLinkLabel {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..2usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::BLink)
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(MirBlockRef::new_empty().into_mir()),
            ],
        }
    }
}
impl BLinkLabel {
    pub fn new(opcode: MirOP, ra: GPR64, target: MirBlockRef) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(ra.into_mir()), Cell::new(target.into_mir())],
        }
    }
    #[doc = "operand ra at index 0 of type GPReg"]
    pub fn ra(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand ra at index 0 of type GPReg"]
    pub fn get_ra(&self) -> GPReg {
        GPReg::from_mir(self.ra().get())
    }
    #[doc = "set the value of operand ra at 0 to a value of type GPReg (checked by GPR64)"]
    pub fn set_ra(&self, value: GPReg) {
        let prev_value = self.get_ra();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.ra().set(next_value.into_mir());
    }
    #[doc = "operand target at index 1 of type MirBlockRef"]
    pub fn target(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand target at index 1 of type MirBlockRef"]
    pub fn get_target(&self) -> MirBlockRef {
        MirBlockRef::from_mir(self.target().get())
    }
    #[doc = "set the value of operand target at 1 to a value of type MirBlockRef (checked by MirBlockRef)"]
    pub fn set_target(&self, value: MirBlockRef) {
        self.target().set(value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct BLinkReg {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for BLinkReg {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..2usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::BLinkReg)
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
            ],
        }
    }
}
impl BLinkReg {
    pub fn new(opcode: MirOP, ra: GPR64, target: GPR64) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(ra.into_mir()), Cell::new(target.into_mir())],
        }
    }
    #[doc = "operand ra at index 0 of type GPReg"]
    pub fn ra(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand ra at index 0 of type GPReg"]
    pub fn get_ra(&self) -> GPReg {
        GPReg::from_mir(self.ra().get())
    }
    #[doc = "set the value of operand ra at 0 to a value of type GPReg (checked by GPR64)"]
    pub fn set_ra(&self, value: GPReg) {
        let prev_value = self.get_ra();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.ra().set(next_value.into_mir());
    }
    #[doc = "operand target at index 1 of type GPReg"]
    pub fn target(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand target at index 1 of type GPReg"]
    pub fn get_target(&self) -> GPReg {
        GPReg::from_mir(self.target().get())
    }
    #[doc = "set the value of operand target at 1 to a value of type GPReg (checked by GPR64)"]
    pub fn set_target(&self, value: GPReg) {
        let prev_value = self.get_target();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.target().set(next_value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct ICmp64R {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
    rm_op: Option<RegOP>,
}
impl IMirSubInst for ICmp64R {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::ICmp64R | MirOP::ICmn64R)
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(PState::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
            ],
            rm_op: None,
        }
    }
}
impl ICmp64R {
    pub fn new(opcode: MirOP, csr: PState, rn: GPR64, rhs: GPR64, rm_op: Option<RegOP>) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(csr.into_mir()),
                Cell::new(rn.into_mir()),
                Cell::new(rhs.into_mir()),
            ],
            rm_op,
        }
    }
    #[doc = "operand csr at index 0 of type PState"]
    pub fn csr(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand csr at index 0 of type PState"]
    pub fn get_csr(&self) -> PState {
        PState::from_mir(self.csr().get())
    }
    #[doc = "set the value of operand csr at 0 to a value of type PState (checked by PState)"]
    pub fn set_csr(&self, value: PState) {
        let prev_value = self.get_csr();
        let checked_value = PState::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.csr().set(next_value.into_mir());
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn rn(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn get_rn(&self) -> GPReg {
        GPReg::from_mir(self.rn().get())
    }
    #[doc = "set the value of operand rn at 1 to a value of type GPReg (checked by GPR64)"]
    pub fn set_rn(&self, value: GPReg) {
        let prev_value = self.get_rn();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rn().set(next_value.into_mir());
    }
    #[doc = "operand rhs at index 2 of type GPReg"]
    pub fn rhs(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rhs at index 2 of type GPReg"]
    pub fn get_rhs(&self) -> GPReg {
        GPReg::from_mir(self.rhs().get())
    }
    #[doc = "set the value of operand rhs at 2 to a value of type GPReg (checked by GPR64)"]
    pub fn set_rhs(&self, value: GPReg) {
        let prev_value = self.get_rhs();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rhs().set(next_value.into_mir());
    }
    pub fn get_rm_op(&self) -> Option<RegOP> {
        self.rm_op.clone()
    }
    pub fn set_rm_op(&mut self, value: Option<RegOP>) {
        self.rm_op = value;
    }
}
#[derive(Debug, Clone)]
pub struct ICmp32R {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
    rm_op: Option<RegOP>,
}
impl IMirSubInst for ICmp32R {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::ICmp32R | MirOP::ICmn32R)
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(PState::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
            ],
            rm_op: None,
        }
    }
}
impl ICmp32R {
    pub fn new(opcode: MirOP, csr: PState, rn: GPR32, rhs: GPR32, rm_op: Option<RegOP>) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(csr.into_mir()),
                Cell::new(rn.into_mir()),
                Cell::new(rhs.into_mir()),
            ],
            rm_op,
        }
    }
    #[doc = "operand csr at index 0 of type PState"]
    pub fn csr(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand csr at index 0 of type PState"]
    pub fn get_csr(&self) -> PState {
        PState::from_mir(self.csr().get())
    }
    #[doc = "set the value of operand csr at 0 to a value of type PState (checked by PState)"]
    pub fn set_csr(&self, value: PState) {
        let prev_value = self.get_csr();
        let checked_value = PState::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.csr().set(next_value.into_mir());
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn rn(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn get_rn(&self) -> GPReg {
        GPReg::from_mir(self.rn().get())
    }
    #[doc = "set the value of operand rn at 1 to a value of type GPReg (checked by GPR32)"]
    pub fn set_rn(&self, value: GPReg) {
        let prev_value = self.get_rn();
        let checked_value = GPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rn().set(next_value.into_mir());
    }
    #[doc = "operand rhs at index 2 of type GPReg"]
    pub fn rhs(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rhs at index 2 of type GPReg"]
    pub fn get_rhs(&self) -> GPReg {
        GPReg::from_mir(self.rhs().get())
    }
    #[doc = "set the value of operand rhs at 2 to a value of type GPReg (checked by GPR32)"]
    pub fn set_rhs(&self, value: GPReg) {
        let prev_value = self.get_rhs();
        let checked_value = GPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rhs().set(next_value.into_mir());
    }
    pub fn get_rm_op(&self) -> Option<RegOP> {
        self.rm_op.clone()
    }
    pub fn set_rm_op(&mut self, value: Option<RegOP>) {
        self.rm_op = value;
    }
}
#[derive(Debug, Clone)]
pub struct ICmp64I {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for ICmp64I {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::ICmp64I | MirOP::ICmn64I)
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(PState::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmCalc::new_empty().into_mir()),
            ],
        }
    }
}
impl ICmp64I {
    pub fn new(opcode: MirOP, csr: PState, rn: GPR64, rhs: ImmCalc) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(csr.into_mir()),
                Cell::new(rn.into_mir()),
                Cell::new(rhs.into_mir()),
            ],
        }
    }
    #[doc = "operand csr at index 0 of type PState"]
    pub fn csr(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand csr at index 0 of type PState"]
    pub fn get_csr(&self) -> PState {
        PState::from_mir(self.csr().get())
    }
    #[doc = "set the value of operand csr at 0 to a value of type PState (checked by PState)"]
    pub fn set_csr(&self, value: PState) {
        let prev_value = self.get_csr();
        let checked_value = PState::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.csr().set(next_value.into_mir());
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn rn(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn get_rn(&self) -> GPReg {
        GPReg::from_mir(self.rn().get())
    }
    #[doc = "set the value of operand rn at 1 to a value of type GPReg (checked by GPR64)"]
    pub fn set_rn(&self, value: GPReg) {
        let prev_value = self.get_rn();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rn().set(next_value.into_mir());
    }
    #[doc = "operand rhs at index 2 of type ImmCalc"]
    pub fn rhs(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rhs at index 2 of type ImmCalc"]
    pub fn get_rhs(&self) -> ImmCalc {
        ImmCalc::from_mir(self.rhs().get())
    }
    #[doc = "set the value of operand rhs at 2 to a value of type ImmCalc (checked by ImmCalc)"]
    pub fn set_rhs(&self, value: ImmCalc) {
        self.rhs().set(value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct ICmp32I {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for ICmp32I {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::ICmp32I | MirOP::ICmn32I)
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(PState::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmCalc::new_empty().into_mir()),
            ],
        }
    }
}
impl ICmp32I {
    pub fn new(opcode: MirOP, csr: PState, rn: GPR32, rhs: ImmCalc) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(csr.into_mir()),
                Cell::new(rn.into_mir()),
                Cell::new(rhs.into_mir()),
            ],
        }
    }
    #[doc = "operand csr at index 0 of type PState"]
    pub fn csr(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand csr at index 0 of type PState"]
    pub fn get_csr(&self) -> PState {
        PState::from_mir(self.csr().get())
    }
    #[doc = "set the value of operand csr at 0 to a value of type PState (checked by PState)"]
    pub fn set_csr(&self, value: PState) {
        let prev_value = self.get_csr();
        let checked_value = PState::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.csr().set(next_value.into_mir());
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn rn(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn get_rn(&self) -> GPReg {
        GPReg::from_mir(self.rn().get())
    }
    #[doc = "set the value of operand rn at 1 to a value of type GPReg (checked by GPR32)"]
    pub fn set_rn(&self, value: GPReg) {
        let prev_value = self.get_rn();
        let checked_value = GPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rn().set(next_value.into_mir());
    }
    #[doc = "operand rhs at index 2 of type ImmCalc"]
    pub fn rhs(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rhs at index 2 of type ImmCalc"]
    pub fn get_rhs(&self) -> ImmCalc {
        ImmCalc::from_mir(self.rhs().get())
    }
    #[doc = "set the value of operand rhs at 2 to a value of type ImmCalc (checked by ImmCalc)"]
    pub fn set_rhs(&self, value: ImmCalc) {
        self.rhs().set(value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct FCmp32 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for FCmp32 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::FCmp32 | MirOP::FCmpE32)
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(PState::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
            ],
        }
    }
}
impl FCmp32 {
    pub fn new(opcode: MirOP, csr: PState, rn: FPR32, rhs: FPR32) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(csr.into_mir()),
                Cell::new(rn.into_mir()),
                Cell::new(rhs.into_mir()),
            ],
        }
    }
    #[doc = "operand csr at index 0 of type PState"]
    pub fn csr(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand csr at index 0 of type PState"]
    pub fn get_csr(&self) -> PState {
        PState::from_mir(self.csr().get())
    }
    #[doc = "set the value of operand csr at 0 to a value of type PState (checked by PState)"]
    pub fn set_csr(&self, value: PState) {
        let prev_value = self.get_csr();
        let checked_value = PState::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.csr().set(next_value.into_mir());
    }
    #[doc = "operand rn at index 1 of type VFReg"]
    pub fn rn(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand rn at index 1 of type VFReg"]
    pub fn get_rn(&self) -> VFReg {
        VFReg::from_mir(self.rn().get())
    }
    #[doc = "set the value of operand rn at 1 to a value of type VFReg (checked by FPR32)"]
    pub fn set_rn(&self, value: VFReg) {
        let prev_value = self.get_rn();
        let checked_value = FPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rn().set(next_value.into_mir());
    }
    #[doc = "operand rhs at index 2 of type VFReg"]
    pub fn rhs(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rhs at index 2 of type VFReg"]
    pub fn get_rhs(&self) -> VFReg {
        VFReg::from_mir(self.rhs().get())
    }
    #[doc = "set the value of operand rhs at 2 to a value of type VFReg (checked by FPR32)"]
    pub fn set_rhs(&self, value: VFReg) {
        let prev_value = self.get_rhs();
        let checked_value = FPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rhs().set(next_value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct FCmp64 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for FCmp64 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::FCmp64 | MirOP::FCmpE64)
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(PState::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
            ],
        }
    }
}
impl FCmp64 {
    pub fn new(opcode: MirOP, csr: PState, rn: FPR64, rhs: FPR64) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(csr.into_mir()),
                Cell::new(rn.into_mir()),
                Cell::new(rhs.into_mir()),
            ],
        }
    }
    #[doc = "operand csr at index 0 of type PState"]
    pub fn csr(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand csr at index 0 of type PState"]
    pub fn get_csr(&self) -> PState {
        PState::from_mir(self.csr().get())
    }
    #[doc = "set the value of operand csr at 0 to a value of type PState (checked by PState)"]
    pub fn set_csr(&self, value: PState) {
        let prev_value = self.get_csr();
        let checked_value = PState::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.csr().set(next_value.into_mir());
    }
    #[doc = "operand rn at index 1 of type VFReg"]
    pub fn rn(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand rn at index 1 of type VFReg"]
    pub fn get_rn(&self) -> VFReg {
        VFReg::from_mir(self.rn().get())
    }
    #[doc = "set the value of operand rn at 1 to a value of type VFReg (checked by FPR64)"]
    pub fn set_rn(&self, value: VFReg) {
        let prev_value = self.get_rn();
        let checked_value = FPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rn().set(next_value.into_mir());
    }
    #[doc = "operand rhs at index 2 of type VFReg"]
    pub fn rhs(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rhs at index 2 of type VFReg"]
    pub fn get_rhs(&self) -> VFReg {
        VFReg::from_mir(self.rhs().get())
    }
    #[doc = "set the value of operand rhs at 2 to a value of type VFReg (checked by FPR64)"]
    pub fn set_rhs(&self, value: VFReg) {
        let prev_value = self.get_rhs();
        let checked_value = FPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rhs().set(next_value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct ICCmp64R {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
    cond: MirCondFlag,
    nzcv: NZCV,
}
impl IMirSubInst for ICCmp64R {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::ICCmp64R | MirOP::ICCmn64R)
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(PState::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
            ],
            cond: MirCondFlag::AL,
            nzcv: NZCV::empty(),
        }
    }
}
impl ICCmp64R {
    pub fn new(
        opcode: MirOP,
        csr: PState,
        rn: GPR64,
        rhs: GPR64,
        cond: MirCondFlag,
        nzcv: NZCV,
    ) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(csr.into_mir()),
                Cell::new(rn.into_mir()),
                Cell::new(rhs.into_mir()),
            ],
            cond,
            nzcv,
        }
    }
    #[doc = "operand csr at index 0 of type PState"]
    pub fn csr(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand csr at index 0 of type PState"]
    pub fn get_csr(&self) -> PState {
        PState::from_mir(self.csr().get())
    }
    #[doc = "set the value of operand csr at 0 to a value of type PState (checked by PState)"]
    pub fn set_csr(&self, value: PState) {
        let prev_value = self.get_csr();
        let checked_value = PState::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.csr().set(next_value.into_mir());
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn rn(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn get_rn(&self) -> GPReg {
        GPReg::from_mir(self.rn().get())
    }
    #[doc = "set the value of operand rn at 1 to a value of type GPReg (checked by GPR64)"]
    pub fn set_rn(&self, value: GPReg) {
        let prev_value = self.get_rn();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rn().set(next_value.into_mir());
    }
    #[doc = "operand rhs at index 2 of type GPReg"]
    pub fn rhs(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rhs at index 2 of type GPReg"]
    pub fn get_rhs(&self) -> GPReg {
        GPReg::from_mir(self.rhs().get())
    }
    #[doc = "set the value of operand rhs at 2 to a value of type GPReg (checked by GPR64)"]
    pub fn set_rhs(&self, value: GPReg) {
        let prev_value = self.get_rhs();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rhs().set(next_value.into_mir());
    }
    pub fn get_cond(&self) -> MirCondFlag {
        self.cond.clone()
    }
    pub fn set_cond(&mut self, value: MirCondFlag) {
        self.cond = value;
    }
    pub fn get_nzcv(&self) -> NZCV {
        self.nzcv.clone()
    }
    pub fn set_nzcv(&mut self, value: NZCV) {
        self.nzcv = value;
    }
}
#[derive(Debug, Clone)]
pub struct ICCmp32R {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
    cond: MirCondFlag,
    nzcv: NZCV,
}
impl IMirSubInst for ICCmp32R {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::ICCmp32R | MirOP::ICCmn32R)
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(PState::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
            ],
            cond: MirCondFlag::AL,
            nzcv: NZCV::empty(),
        }
    }
}
impl ICCmp32R {
    pub fn new(
        opcode: MirOP,
        csr: PState,
        rn: GPR32,
        rhs: GPR32,
        cond: MirCondFlag,
        nzcv: NZCV,
    ) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(csr.into_mir()),
                Cell::new(rn.into_mir()),
                Cell::new(rhs.into_mir()),
            ],
            cond,
            nzcv,
        }
    }
    #[doc = "operand csr at index 0 of type PState"]
    pub fn csr(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand csr at index 0 of type PState"]
    pub fn get_csr(&self) -> PState {
        PState::from_mir(self.csr().get())
    }
    #[doc = "set the value of operand csr at 0 to a value of type PState (checked by PState)"]
    pub fn set_csr(&self, value: PState) {
        let prev_value = self.get_csr();
        let checked_value = PState::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.csr().set(next_value.into_mir());
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn rn(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn get_rn(&self) -> GPReg {
        GPReg::from_mir(self.rn().get())
    }
    #[doc = "set the value of operand rn at 1 to a value of type GPReg (checked by GPR32)"]
    pub fn set_rn(&self, value: GPReg) {
        let prev_value = self.get_rn();
        let checked_value = GPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rn().set(next_value.into_mir());
    }
    #[doc = "operand rhs at index 2 of type GPReg"]
    pub fn rhs(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rhs at index 2 of type GPReg"]
    pub fn get_rhs(&self) -> GPReg {
        GPReg::from_mir(self.rhs().get())
    }
    #[doc = "set the value of operand rhs at 2 to a value of type GPReg (checked by GPR32)"]
    pub fn set_rhs(&self, value: GPReg) {
        let prev_value = self.get_rhs();
        let checked_value = GPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rhs().set(next_value.into_mir());
    }
    pub fn get_cond(&self) -> MirCondFlag {
        self.cond.clone()
    }
    pub fn set_cond(&mut self, value: MirCondFlag) {
        self.cond = value;
    }
    pub fn get_nzcv(&self) -> NZCV {
        self.nzcv.clone()
    }
    pub fn set_nzcv(&mut self, value: NZCV) {
        self.nzcv = value;
    }
}
#[derive(Debug, Clone)]
pub struct ICCmp64I {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
    cond: MirCondFlag,
    nzcv: NZCV,
}
impl IMirSubInst for ICCmp64I {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::ICCmp64I | MirOP::ICCmn64I)
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(PState::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmCCmp::new_empty().into_mir()),
            ],
            cond: MirCondFlag::AL,
            nzcv: NZCV::empty(),
        }
    }
}
impl ICCmp64I {
    pub fn new(
        opcode: MirOP,
        csr: PState,
        rn: GPR64,
        rhs: ImmCCmp,
        cond: MirCondFlag,
        nzcv: NZCV,
    ) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(csr.into_mir()),
                Cell::new(rn.into_mir()),
                Cell::new(rhs.into_mir()),
            ],
            cond,
            nzcv,
        }
    }
    #[doc = "operand csr at index 0 of type PState"]
    pub fn csr(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand csr at index 0 of type PState"]
    pub fn get_csr(&self) -> PState {
        PState::from_mir(self.csr().get())
    }
    #[doc = "set the value of operand csr at 0 to a value of type PState (checked by PState)"]
    pub fn set_csr(&self, value: PState) {
        let prev_value = self.get_csr();
        let checked_value = PState::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.csr().set(next_value.into_mir());
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn rn(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn get_rn(&self) -> GPReg {
        GPReg::from_mir(self.rn().get())
    }
    #[doc = "set the value of operand rn at 1 to a value of type GPReg (checked by GPR64)"]
    pub fn set_rn(&self, value: GPReg) {
        let prev_value = self.get_rn();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rn().set(next_value.into_mir());
    }
    #[doc = "operand rhs at index 2 of type ImmCCmp"]
    pub fn rhs(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rhs at index 2 of type ImmCCmp"]
    pub fn get_rhs(&self) -> ImmCCmp {
        ImmCCmp::from_mir(self.rhs().get())
    }
    #[doc = "set the value of operand rhs at 2 to a value of type ImmCCmp (checked by ImmCCmp)"]
    pub fn set_rhs(&self, value: ImmCCmp) {
        self.rhs().set(value.into_mir());
    }
    pub fn get_cond(&self) -> MirCondFlag {
        self.cond.clone()
    }
    pub fn set_cond(&mut self, value: MirCondFlag) {
        self.cond = value;
    }
    pub fn get_nzcv(&self) -> NZCV {
        self.nzcv.clone()
    }
    pub fn set_nzcv(&mut self, value: NZCV) {
        self.nzcv = value;
    }
}
#[derive(Debug, Clone)]
pub struct ICCmp32I {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
    cond: MirCondFlag,
    nzcv: NZCV,
}
impl IMirSubInst for ICCmp32I {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::ICCmp32I | MirOP::ICCmn32I)
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(PState::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmCCmp::new_empty().into_mir()),
            ],
            cond: MirCondFlag::AL,
            nzcv: NZCV::empty(),
        }
    }
}
impl ICCmp32I {
    pub fn new(
        opcode: MirOP,
        csr: PState,
        rn: GPR32,
        rhs: ImmCCmp,
        cond: MirCondFlag,
        nzcv: NZCV,
    ) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(csr.into_mir()),
                Cell::new(rn.into_mir()),
                Cell::new(rhs.into_mir()),
            ],
            cond,
            nzcv,
        }
    }
    #[doc = "operand csr at index 0 of type PState"]
    pub fn csr(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand csr at index 0 of type PState"]
    pub fn get_csr(&self) -> PState {
        PState::from_mir(self.csr().get())
    }
    #[doc = "set the value of operand csr at 0 to a value of type PState (checked by PState)"]
    pub fn set_csr(&self, value: PState) {
        let prev_value = self.get_csr();
        let checked_value = PState::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.csr().set(next_value.into_mir());
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn rn(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn get_rn(&self) -> GPReg {
        GPReg::from_mir(self.rn().get())
    }
    #[doc = "set the value of operand rn at 1 to a value of type GPReg (checked by GPR32)"]
    pub fn set_rn(&self, value: GPReg) {
        let prev_value = self.get_rn();
        let checked_value = GPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rn().set(next_value.into_mir());
    }
    #[doc = "operand rhs at index 2 of type ImmCCmp"]
    pub fn rhs(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rhs at index 2 of type ImmCCmp"]
    pub fn get_rhs(&self) -> ImmCCmp {
        ImmCCmp::from_mir(self.rhs().get())
    }
    #[doc = "set the value of operand rhs at 2 to a value of type ImmCCmp (checked by ImmCCmp)"]
    pub fn set_rhs(&self, value: ImmCCmp) {
        self.rhs().set(value.into_mir());
    }
    pub fn get_cond(&self) -> MirCondFlag {
        self.cond.clone()
    }
    pub fn set_cond(&mut self, value: MirCondFlag) {
        self.cond = value;
    }
    pub fn get_nzcv(&self) -> NZCV {
        self.nzcv.clone()
    }
    pub fn set_nzcv(&mut self, value: NZCV) {
        self.nzcv = value;
    }
}
#[derive(Debug, Clone)]
pub struct FCCmp32 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
    cond: MirCondFlag,
    nzcv: NZCV,
}
impl IMirSubInst for FCCmp32 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::FCCmp32 | MirOP::FCCmpE32)
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(PState::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
            ],
            cond: MirCondFlag::AL,
            nzcv: NZCV::empty(),
        }
    }
}
impl FCCmp32 {
    pub fn new(
        opcode: MirOP,
        csr: PState,
        rn: FPR32,
        rhs: FPR32,
        cond: MirCondFlag,
        nzcv: NZCV,
    ) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(csr.into_mir()),
                Cell::new(rn.into_mir()),
                Cell::new(rhs.into_mir()),
            ],
            cond,
            nzcv,
        }
    }
    #[doc = "operand csr at index 0 of type PState"]
    pub fn csr(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand csr at index 0 of type PState"]
    pub fn get_csr(&self) -> PState {
        PState::from_mir(self.csr().get())
    }
    #[doc = "set the value of operand csr at 0 to a value of type PState (checked by PState)"]
    pub fn set_csr(&self, value: PState) {
        let prev_value = self.get_csr();
        let checked_value = PState::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.csr().set(next_value.into_mir());
    }
    #[doc = "operand rn at index 1 of type VFReg"]
    pub fn rn(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand rn at index 1 of type VFReg"]
    pub fn get_rn(&self) -> VFReg {
        VFReg::from_mir(self.rn().get())
    }
    #[doc = "set the value of operand rn at 1 to a value of type VFReg (checked by FPR32)"]
    pub fn set_rn(&self, value: VFReg) {
        let prev_value = self.get_rn();
        let checked_value = FPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rn().set(next_value.into_mir());
    }
    #[doc = "operand rhs at index 2 of type VFReg"]
    pub fn rhs(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rhs at index 2 of type VFReg"]
    pub fn get_rhs(&self) -> VFReg {
        VFReg::from_mir(self.rhs().get())
    }
    #[doc = "set the value of operand rhs at 2 to a value of type VFReg (checked by FPR32)"]
    pub fn set_rhs(&self, value: VFReg) {
        let prev_value = self.get_rhs();
        let checked_value = FPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rhs().set(next_value.into_mir());
    }
    pub fn get_cond(&self) -> MirCondFlag {
        self.cond.clone()
    }
    pub fn set_cond(&mut self, value: MirCondFlag) {
        self.cond = value;
    }
    pub fn get_nzcv(&self) -> NZCV {
        self.nzcv.clone()
    }
    pub fn set_nzcv(&mut self, value: NZCV) {
        self.nzcv = value;
    }
}
#[derive(Debug, Clone)]
pub struct FCCmp64 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
    cond: MirCondFlag,
    nzcv: NZCV,
}
impl IMirSubInst for FCCmp64 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::FCCmp64 | MirOP::FCCmpE64)
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(PState::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
            ],
            cond: MirCondFlag::AL,
            nzcv: NZCV::empty(),
        }
    }
}
impl FCCmp64 {
    pub fn new(
        opcode: MirOP,
        csr: PState,
        rn: FPR64,
        rhs: FPR64,
        cond: MirCondFlag,
        nzcv: NZCV,
    ) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(csr.into_mir()),
                Cell::new(rn.into_mir()),
                Cell::new(rhs.into_mir()),
            ],
            cond,
            nzcv,
        }
    }
    #[doc = "operand csr at index 0 of type PState"]
    pub fn csr(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand csr at index 0 of type PState"]
    pub fn get_csr(&self) -> PState {
        PState::from_mir(self.csr().get())
    }
    #[doc = "set the value of operand csr at 0 to a value of type PState (checked by PState)"]
    pub fn set_csr(&self, value: PState) {
        let prev_value = self.get_csr();
        let checked_value = PState::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.csr().set(next_value.into_mir());
    }
    #[doc = "operand rn at index 1 of type VFReg"]
    pub fn rn(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand rn at index 1 of type VFReg"]
    pub fn get_rn(&self) -> VFReg {
        VFReg::from_mir(self.rn().get())
    }
    #[doc = "set the value of operand rn at 1 to a value of type VFReg (checked by FPR64)"]
    pub fn set_rn(&self, value: VFReg) {
        let prev_value = self.get_rn();
        let checked_value = FPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rn().set(next_value.into_mir());
    }
    #[doc = "operand rhs at index 2 of type VFReg"]
    pub fn rhs(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rhs at index 2 of type VFReg"]
    pub fn get_rhs(&self) -> VFReg {
        VFReg::from_mir(self.rhs().get())
    }
    #[doc = "set the value of operand rhs at 2 to a value of type VFReg (checked by FPR64)"]
    pub fn set_rhs(&self, value: VFReg) {
        let prev_value = self.get_rhs();
        let checked_value = FPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rhs().set(next_value.into_mir());
    }
    pub fn get_cond(&self) -> MirCondFlag {
        self.cond.clone()
    }
    pub fn set_cond(&mut self, value: MirCondFlag) {
        self.cond = value;
    }
    pub fn get_nzcv(&self) -> NZCV {
        self.nzcv.clone()
    }
    pub fn set_nzcv(&mut self, value: NZCV) {
        self.nzcv = value;
    }
}
#[derive(Debug, Clone)]
pub struct Bin64R {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
    rm_op: Option<RegOP>,
}
impl IMirSubInst for Bin64R {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(
            opcode,
            MirOP::Add64R
                | MirOP::Sub64R
                | MirOP::SMax64R
                | MirOP::SMin64R
                | MirOP::UMax64R
                | MirOP::UMin64R
                | MirOP::And64R
                | MirOP::Bic64R
                | MirOP::EON64R
                | MirOP::EOR64R
                | MirOP::ORR64R
                | MirOP::ORN64R
                | MirOP::Asr64R
                | MirOP::Lsr64R
                | MirOP::Lsl64R
                | MirOP::Ror64R
                | MirOP::Mul64
                | MirOP::MNeg64
                | MirOP::SDiv64
                | MirOP::UDiv64
                | MirOP::SMulH
                | MirOP::UMulH
        )
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
            ],
            rm_op: None,
        }
    }
}
impl Bin64R {
    pub fn new(opcode: MirOP, rd: GPR64, rn: GPR64, rm: GPR64, rm_op: Option<RegOP>) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.into_mir()),
                Cell::new(rn.into_mir()),
                Cell::new(rm.into_mir()),
            ],
            rm_op,
        }
    }
    #[doc = "operand rd at index 0 of type GPReg"]
    pub fn rd(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand rd at index 0 of type GPReg"]
    pub fn get_rd(&self) -> GPReg {
        GPReg::from_mir(self.rd().get())
    }
    #[doc = "set the value of operand rd at 0 to a value of type GPReg (checked by GPR64)"]
    pub fn set_rd(&self, value: GPReg) {
        let prev_value = self.get_rd();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rd().set(next_value.into_mir());
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn rn(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn get_rn(&self) -> GPReg {
        GPReg::from_mir(self.rn().get())
    }
    #[doc = "set the value of operand rn at 1 to a value of type GPReg (checked by GPR64)"]
    pub fn set_rn(&self, value: GPReg) {
        let prev_value = self.get_rn();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rn().set(next_value.into_mir());
    }
    #[doc = "operand rm at index 2 of type GPReg"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type GPReg"]
    pub fn get_rm(&self) -> GPReg {
        GPReg::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type GPReg (checked by GPR64)"]
    pub fn set_rm(&self, value: GPReg) {
        let prev_value = self.get_rm();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rm().set(next_value.into_mir());
    }
    pub fn get_rm_op(&self) -> Option<RegOP> {
        self.rm_op.clone()
    }
    pub fn set_rm_op(&mut self, value: Option<RegOP>) {
        self.rm_op = value;
    }
}
#[derive(Debug, Clone)]
pub struct Bin32R {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
    rm_op: Option<RegOP>,
}
impl IMirSubInst for Bin32R {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(
            opcode,
            MirOP::Add32R
                | MirOP::Sub32R
                | MirOP::SMax32R
                | MirOP::SMin32R
                | MirOP::UMax32R
                | MirOP::UMin32R
                | MirOP::And32R
                | MirOP::Bic32R
                | MirOP::EON32R
                | MirOP::EOR32R
                | MirOP::ORR32R
                | MirOP::ORN32R
                | MirOP::Asr32R
                | MirOP::Lsr32R
                | MirOP::Lsl32R
                | MirOP::Ror32R
                | MirOP::Mul32
                | MirOP::MNeg32
                | MirOP::SDiv32
                | MirOP::UDiv32
        )
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
            ],
            rm_op: None,
        }
    }
}
impl Bin32R {
    pub fn new(opcode: MirOP, rd: GPR32, rn: GPR32, rm: GPR32, rm_op: Option<RegOP>) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.into_mir()),
                Cell::new(rn.into_mir()),
                Cell::new(rm.into_mir()),
            ],
            rm_op,
        }
    }
    #[doc = "operand rd at index 0 of type GPReg"]
    pub fn rd(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand rd at index 0 of type GPReg"]
    pub fn get_rd(&self) -> GPReg {
        GPReg::from_mir(self.rd().get())
    }
    #[doc = "set the value of operand rd at 0 to a value of type GPReg (checked by GPR32)"]
    pub fn set_rd(&self, value: GPReg) {
        let prev_value = self.get_rd();
        let checked_value = GPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rd().set(next_value.into_mir());
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn rn(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn get_rn(&self) -> GPReg {
        GPReg::from_mir(self.rn().get())
    }
    #[doc = "set the value of operand rn at 1 to a value of type GPReg (checked by GPR32)"]
    pub fn set_rn(&self, value: GPReg) {
        let prev_value = self.get_rn();
        let checked_value = GPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rn().set(next_value.into_mir());
    }
    #[doc = "operand rm at index 2 of type GPReg"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type GPReg"]
    pub fn get_rm(&self) -> GPReg {
        GPReg::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type GPReg (checked by GPR32)"]
    pub fn set_rm(&self, value: GPReg) {
        let prev_value = self.get_rm();
        let checked_value = GPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rm().set(next_value.into_mir());
    }
    pub fn get_rm_op(&self) -> Option<RegOP> {
        self.rm_op.clone()
    }
    pub fn set_rm_op(&mut self, value: Option<RegOP>) {
        self.rm_op = value;
    }
}
#[derive(Debug, Clone)]
pub struct MulL {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for MulL {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(
            opcode,
            MirOP::SMULL | MirOP::UMULL | MirOP::SMNegL | MirOP::UMNegL
        )
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
            ],
        }
    }
}
impl MulL {
    pub fn new(opcode: MirOP, rd: GPR64, rn: GPR32, rm: GPR32) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.into_mir()),
                Cell::new(rn.into_mir()),
                Cell::new(rm.into_mir()),
            ],
        }
    }
    #[doc = "operand rd at index 0 of type GPReg"]
    pub fn rd(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand rd at index 0 of type GPReg"]
    pub fn get_rd(&self) -> GPReg {
        GPReg::from_mir(self.rd().get())
    }
    #[doc = "set the value of operand rd at 0 to a value of type GPReg (checked by GPR64)"]
    pub fn set_rd(&self, value: GPReg) {
        let prev_value = self.get_rd();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rd().set(next_value.into_mir());
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn rn(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn get_rn(&self) -> GPReg {
        GPReg::from_mir(self.rn().get())
    }
    #[doc = "set the value of operand rn at 1 to a value of type GPReg (checked by GPR32)"]
    pub fn set_rn(&self, value: GPReg) {
        let prev_value = self.get_rn();
        let checked_value = GPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rn().set(next_value.into_mir());
    }
    #[doc = "operand rm at index 2 of type GPReg"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type GPReg"]
    pub fn get_rm(&self) -> GPReg {
        GPReg::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type GPReg (checked by GPR32)"]
    pub fn set_rm(&self, value: GPReg) {
        let prev_value = self.get_rm();
        let checked_value = GPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rm().set(next_value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct Bin64RC {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for Bin64RC {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::Add64I | MirOP::Sub64I)
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmCalc::new_empty().into_mir()),
            ],
        }
    }
}
impl Bin64RC {
    pub fn new(opcode: MirOP, rd: GPR64, rn: GPR64, rm: ImmCalc) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.into_mir()),
                Cell::new(rn.into_mir()),
                Cell::new(rm.into_mir()),
            ],
        }
    }
    #[doc = "operand rd at index 0 of type GPReg"]
    pub fn rd(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand rd at index 0 of type GPReg"]
    pub fn get_rd(&self) -> GPReg {
        GPReg::from_mir(self.rd().get())
    }
    #[doc = "set the value of operand rd at 0 to a value of type GPReg (checked by GPR64)"]
    pub fn set_rd(&self, value: GPReg) {
        let prev_value = self.get_rd();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rd().set(next_value.into_mir());
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn rn(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn get_rn(&self) -> GPReg {
        GPReg::from_mir(self.rn().get())
    }
    #[doc = "set the value of operand rn at 1 to a value of type GPReg (checked by GPR64)"]
    pub fn set_rn(&self, value: GPReg) {
        let prev_value = self.get_rn();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rn().set(next_value.into_mir());
    }
    #[doc = "operand rm at index 2 of type ImmCalc"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type ImmCalc"]
    pub fn get_rm(&self) -> ImmCalc {
        ImmCalc::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type ImmCalc (checked by ImmCalc)"]
    pub fn set_rm(&self, value: ImmCalc) {
        self.rm().set(value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct Bin32RC {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for Bin32RC {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::Add32I | MirOP::Sub32I)
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmCalc::new_empty().into_mir()),
            ],
        }
    }
}
impl Bin32RC {
    pub fn new(opcode: MirOP, rd: GPR32, rn: GPR32, rm: ImmCalc) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.into_mir()),
                Cell::new(rn.into_mir()),
                Cell::new(rm.into_mir()),
            ],
        }
    }
    #[doc = "operand rd at index 0 of type GPReg"]
    pub fn rd(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand rd at index 0 of type GPReg"]
    pub fn get_rd(&self) -> GPReg {
        GPReg::from_mir(self.rd().get())
    }
    #[doc = "set the value of operand rd at 0 to a value of type GPReg (checked by GPR32)"]
    pub fn set_rd(&self, value: GPReg) {
        let prev_value = self.get_rd();
        let checked_value = GPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rd().set(next_value.into_mir());
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn rn(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn get_rn(&self) -> GPReg {
        GPReg::from_mir(self.rn().get())
    }
    #[doc = "set the value of operand rn at 1 to a value of type GPReg (checked by GPR32)"]
    pub fn set_rn(&self, value: GPReg) {
        let prev_value = self.get_rn();
        let checked_value = GPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rn().set(next_value.into_mir());
    }
    #[doc = "operand rm at index 2 of type ImmCalc"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type ImmCalc"]
    pub fn get_rm(&self) -> ImmCalc {
        ImmCalc::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type ImmCalc (checked by ImmCalc)"]
    pub fn set_rm(&self, value: ImmCalc) {
        self.rm().set(value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct Bin64RL {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for Bin64RL {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(
            opcode,
            MirOP::And64I
                | MirOP::Bic64I
                | MirOP::EON64I
                | MirOP::EOR64I
                | MirOP::ORR64I
                | MirOP::ORN64I
        )
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmLogic::new_empty().into_mir()),
            ],
        }
    }
}
impl Bin64RL {
    pub fn new(opcode: MirOP, rd: GPR64, rn: GPR64, rm: ImmLogic) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.into_mir()),
                Cell::new(rn.into_mir()),
                Cell::new(rm.into_mir()),
            ],
        }
    }
    #[doc = "operand rd at index 0 of type GPReg"]
    pub fn rd(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand rd at index 0 of type GPReg"]
    pub fn get_rd(&self) -> GPReg {
        GPReg::from_mir(self.rd().get())
    }
    #[doc = "set the value of operand rd at 0 to a value of type GPReg (checked by GPR64)"]
    pub fn set_rd(&self, value: GPReg) {
        let prev_value = self.get_rd();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rd().set(next_value.into_mir());
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn rn(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn get_rn(&self) -> GPReg {
        GPReg::from_mir(self.rn().get())
    }
    #[doc = "set the value of operand rn at 1 to a value of type GPReg (checked by GPR64)"]
    pub fn set_rn(&self, value: GPReg) {
        let prev_value = self.get_rn();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rn().set(next_value.into_mir());
    }
    #[doc = "operand rm at index 2 of type ImmLogic"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type ImmLogic"]
    pub fn get_rm(&self) -> ImmLogic {
        ImmLogic::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type ImmLogic (checked by ImmLogic)"]
    pub fn set_rm(&self, value: ImmLogic) {
        self.rm().set(value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct Bin32RL {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for Bin32RL {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(
            opcode,
            MirOP::And32I
                | MirOP::Bic32I
                | MirOP::EON32I
                | MirOP::EOR32I
                | MirOP::ORR32I
                | MirOP::ORN32I
        )
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmLogic::new_empty().into_mir()),
            ],
        }
    }
}
impl Bin32RL {
    pub fn new(opcode: MirOP, rd: GPR32, rn: GPR32, rm: ImmLogic) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.into_mir()),
                Cell::new(rn.into_mir()),
                Cell::new(rm.into_mir()),
            ],
        }
    }
    #[doc = "operand rd at index 0 of type GPReg"]
    pub fn rd(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand rd at index 0 of type GPReg"]
    pub fn get_rd(&self) -> GPReg {
        GPReg::from_mir(self.rd().get())
    }
    #[doc = "set the value of operand rd at 0 to a value of type GPReg (checked by GPR32)"]
    pub fn set_rd(&self, value: GPReg) {
        let prev_value = self.get_rd();
        let checked_value = GPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rd().set(next_value.into_mir());
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn rn(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn get_rn(&self) -> GPReg {
        GPReg::from_mir(self.rn().get())
    }
    #[doc = "set the value of operand rn at 1 to a value of type GPReg (checked by GPR32)"]
    pub fn set_rn(&self, value: GPReg) {
        let prev_value = self.get_rn();
        let checked_value = GPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rn().set(next_value.into_mir());
    }
    #[doc = "operand rm at index 2 of type ImmLogic"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type ImmLogic"]
    pub fn get_rm(&self) -> ImmLogic {
        ImmLogic::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type ImmLogic (checked by ImmLogic)"]
    pub fn set_rm(&self, value: ImmLogic) {
        self.rm().set(value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct Bin64RS {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for Bin64RS {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::SMax64I | MirOP::SMin64I)
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmSMax::new_empty().into_mir()),
            ],
        }
    }
}
impl Bin64RS {
    pub fn new(opcode: MirOP, rd: GPR64, rn: GPR64, rm: ImmSMax) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.into_mir()),
                Cell::new(rn.into_mir()),
                Cell::new(rm.into_mir()),
            ],
        }
    }
    #[doc = "operand rd at index 0 of type GPReg"]
    pub fn rd(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand rd at index 0 of type GPReg"]
    pub fn get_rd(&self) -> GPReg {
        GPReg::from_mir(self.rd().get())
    }
    #[doc = "set the value of operand rd at 0 to a value of type GPReg (checked by GPR64)"]
    pub fn set_rd(&self, value: GPReg) {
        let prev_value = self.get_rd();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rd().set(next_value.into_mir());
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn rn(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn get_rn(&self) -> GPReg {
        GPReg::from_mir(self.rn().get())
    }
    #[doc = "set the value of operand rn at 1 to a value of type GPReg (checked by GPR64)"]
    pub fn set_rn(&self, value: GPReg) {
        let prev_value = self.get_rn();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rn().set(next_value.into_mir());
    }
    #[doc = "operand rm at index 2 of type ImmSMax"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type ImmSMax"]
    pub fn get_rm(&self) -> ImmSMax {
        ImmSMax::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type ImmSMax (checked by ImmSMax)"]
    pub fn set_rm(&self, value: ImmSMax) {
        self.rm().set(value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct Bin64RU {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for Bin64RU {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::UMax64I | MirOP::UMin64I)
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmUMax::new_empty().into_mir()),
            ],
        }
    }
}
impl Bin64RU {
    pub fn new(opcode: MirOP, rd: GPR64, rn: GPR64, rm: ImmUMax) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.into_mir()),
                Cell::new(rn.into_mir()),
                Cell::new(rm.into_mir()),
            ],
        }
    }
    #[doc = "operand rd at index 0 of type GPReg"]
    pub fn rd(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand rd at index 0 of type GPReg"]
    pub fn get_rd(&self) -> GPReg {
        GPReg::from_mir(self.rd().get())
    }
    #[doc = "set the value of operand rd at 0 to a value of type GPReg (checked by GPR64)"]
    pub fn set_rd(&self, value: GPReg) {
        let prev_value = self.get_rd();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rd().set(next_value.into_mir());
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn rn(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn get_rn(&self) -> GPReg {
        GPReg::from_mir(self.rn().get())
    }
    #[doc = "set the value of operand rn at 1 to a value of type GPReg (checked by GPR64)"]
    pub fn set_rn(&self, value: GPReg) {
        let prev_value = self.get_rn();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rn().set(next_value.into_mir());
    }
    #[doc = "operand rm at index 2 of type ImmUMax"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type ImmUMax"]
    pub fn get_rm(&self) -> ImmUMax {
        ImmUMax::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type ImmUMax (checked by ImmUMax)"]
    pub fn set_rm(&self, value: ImmUMax) {
        self.rm().set(value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct Bin32RS {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for Bin32RS {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::SMax32I | MirOP::SMin32I)
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmSMax::new_empty().into_mir()),
            ],
        }
    }
}
impl Bin32RS {
    pub fn new(opcode: MirOP, rd: GPR32, rn: GPR32, rm: ImmSMax) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.into_mir()),
                Cell::new(rn.into_mir()),
                Cell::new(rm.into_mir()),
            ],
        }
    }
    #[doc = "operand rd at index 0 of type GPReg"]
    pub fn rd(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand rd at index 0 of type GPReg"]
    pub fn get_rd(&self) -> GPReg {
        GPReg::from_mir(self.rd().get())
    }
    #[doc = "set the value of operand rd at 0 to a value of type GPReg (checked by GPR32)"]
    pub fn set_rd(&self, value: GPReg) {
        let prev_value = self.get_rd();
        let checked_value = GPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rd().set(next_value.into_mir());
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn rn(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn get_rn(&self) -> GPReg {
        GPReg::from_mir(self.rn().get())
    }
    #[doc = "set the value of operand rn at 1 to a value of type GPReg (checked by GPR32)"]
    pub fn set_rn(&self, value: GPReg) {
        let prev_value = self.get_rn();
        let checked_value = GPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rn().set(next_value.into_mir());
    }
    #[doc = "operand rm at index 2 of type ImmSMax"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type ImmSMax"]
    pub fn get_rm(&self) -> ImmSMax {
        ImmSMax::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type ImmSMax (checked by ImmSMax)"]
    pub fn set_rm(&self, value: ImmSMax) {
        self.rm().set(value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct Bin32RU {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for Bin32RU {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::UMax32I | MirOP::UMin32I)
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmUMax::new_empty().into_mir()),
            ],
        }
    }
}
impl Bin32RU {
    pub fn new(opcode: MirOP, rd: GPR32, rn: GPR32, rm: ImmUMax) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.into_mir()),
                Cell::new(rn.into_mir()),
                Cell::new(rm.into_mir()),
            ],
        }
    }
    #[doc = "operand rd at index 0 of type GPReg"]
    pub fn rd(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand rd at index 0 of type GPReg"]
    pub fn get_rd(&self) -> GPReg {
        GPReg::from_mir(self.rd().get())
    }
    #[doc = "set the value of operand rd at 0 to a value of type GPReg (checked by GPR32)"]
    pub fn set_rd(&self, value: GPReg) {
        let prev_value = self.get_rd();
        let checked_value = GPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rd().set(next_value.into_mir());
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn rn(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn get_rn(&self) -> GPReg {
        GPReg::from_mir(self.rn().get())
    }
    #[doc = "set the value of operand rn at 1 to a value of type GPReg (checked by GPR32)"]
    pub fn set_rn(&self, value: GPReg) {
        let prev_value = self.get_rn();
        let checked_value = GPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rn().set(next_value.into_mir());
    }
    #[doc = "operand rm at index 2 of type ImmUMax"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type ImmUMax"]
    pub fn get_rm(&self) -> ImmUMax {
        ImmUMax::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type ImmUMax (checked by ImmUMax)"]
    pub fn set_rm(&self, value: ImmUMax) {
        self.rm().set(value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct Bin64RShift {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for Bin64RShift {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(
            opcode,
            MirOP::Asr64I | MirOP::Lsr64I | MirOP::Lsl64I | MirOP::Ror64I
        )
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmShift::new_empty().into_mir()),
            ],
        }
    }
}
impl Bin64RShift {
    pub fn new(opcode: MirOP, rd: GPR64, rn: GPR64, rm: ImmShift) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.into_mir()),
                Cell::new(rn.into_mir()),
                Cell::new(rm.into_mir()),
            ],
        }
    }
    #[doc = "operand rd at index 0 of type GPReg"]
    pub fn rd(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand rd at index 0 of type GPReg"]
    pub fn get_rd(&self) -> GPReg {
        GPReg::from_mir(self.rd().get())
    }
    #[doc = "set the value of operand rd at 0 to a value of type GPReg (checked by GPR64)"]
    pub fn set_rd(&self, value: GPReg) {
        let prev_value = self.get_rd();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rd().set(next_value.into_mir());
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn rn(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn get_rn(&self) -> GPReg {
        GPReg::from_mir(self.rn().get())
    }
    #[doc = "set the value of operand rn at 1 to a value of type GPReg (checked by GPR64)"]
    pub fn set_rn(&self, value: GPReg) {
        let prev_value = self.get_rn();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rn().set(next_value.into_mir());
    }
    #[doc = "operand rm at index 2 of type ImmShift"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type ImmShift"]
    pub fn get_rm(&self) -> ImmShift {
        ImmShift::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type ImmShift (checked by ImmShift)"]
    pub fn set_rm(&self, value: ImmShift) {
        self.rm().set(value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct Bin32RShift {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for Bin32RShift {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(
            opcode,
            MirOP::Asr32I | MirOP::Lsr32I | MirOP::Lsl32I | MirOP::Ror32I
        )
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmShift::new_empty().into_mir()),
            ],
        }
    }
}
impl Bin32RShift {
    pub fn new(opcode: MirOP, rd: GPR32, rn: GPR32, rm: ImmShift) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.into_mir()),
                Cell::new(rn.into_mir()),
                Cell::new(rm.into_mir()),
            ],
        }
    }
    #[doc = "operand rd at index 0 of type GPReg"]
    pub fn rd(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand rd at index 0 of type GPReg"]
    pub fn get_rd(&self) -> GPReg {
        GPReg::from_mir(self.rd().get())
    }
    #[doc = "set the value of operand rd at 0 to a value of type GPReg (checked by GPR32)"]
    pub fn set_rd(&self, value: GPReg) {
        let prev_value = self.get_rd();
        let checked_value = GPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rd().set(next_value.into_mir());
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn rn(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn get_rn(&self) -> GPReg {
        GPReg::from_mir(self.rn().get())
    }
    #[doc = "set the value of operand rn at 1 to a value of type GPReg (checked by GPR32)"]
    pub fn set_rn(&self, value: GPReg) {
        let prev_value = self.get_rn();
        let checked_value = GPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rn().set(next_value.into_mir());
    }
    #[doc = "operand rm at index 2 of type ImmShift"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type ImmShift"]
    pub fn get_rm(&self) -> ImmShift {
        ImmShift::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type ImmShift (checked by ImmShift)"]
    pub fn set_rm(&self, value: ImmShift) {
        self.rm().set(value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct BinF64R {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for BinF64R {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(
            opcode,
            MirOP::FAdd64 | MirOP::FDiv64 | MirOP::FMul64 | MirOP::FNMul64 | MirOP::FSub64
        )
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
            ],
        }
    }
}
impl BinF64R {
    pub fn new(opcode: MirOP, rd: FPR64, rn: FPR64, rm: FPR64) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.into_mir()),
                Cell::new(rn.into_mir()),
                Cell::new(rm.into_mir()),
            ],
        }
    }
    #[doc = "operand rd at index 0 of type VFReg"]
    pub fn rd(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand rd at index 0 of type VFReg"]
    pub fn get_rd(&self) -> VFReg {
        VFReg::from_mir(self.rd().get())
    }
    #[doc = "set the value of operand rd at 0 to a value of type VFReg (checked by FPR64)"]
    pub fn set_rd(&self, value: VFReg) {
        let prev_value = self.get_rd();
        let checked_value = FPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rd().set(next_value.into_mir());
    }
    #[doc = "operand rn at index 1 of type VFReg"]
    pub fn rn(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand rn at index 1 of type VFReg"]
    pub fn get_rn(&self) -> VFReg {
        VFReg::from_mir(self.rn().get())
    }
    #[doc = "set the value of operand rn at 1 to a value of type VFReg (checked by FPR64)"]
    pub fn set_rn(&self, value: VFReg) {
        let prev_value = self.get_rn();
        let checked_value = FPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rn().set(next_value.into_mir());
    }
    #[doc = "operand rm at index 2 of type VFReg"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type VFReg"]
    pub fn get_rm(&self) -> VFReg {
        VFReg::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type VFReg (checked by FPR64)"]
    pub fn set_rm(&self, value: VFReg) {
        let prev_value = self.get_rm();
        let checked_value = FPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rm().set(next_value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct BinF32R {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for BinF32R {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(
            opcode,
            MirOP::FAdd32 | MirOP::FDiv32 | MirOP::FMul32 | MirOP::FNMul32 | MirOP::FSub32
        )
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
            ],
        }
    }
}
impl BinF32R {
    pub fn new(opcode: MirOP, rd: FPR32, rn: FPR32, rm: FPR32) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.into_mir()),
                Cell::new(rn.into_mir()),
                Cell::new(rm.into_mir()),
            ],
        }
    }
    #[doc = "operand rd at index 0 of type VFReg"]
    pub fn rd(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand rd at index 0 of type VFReg"]
    pub fn get_rd(&self) -> VFReg {
        VFReg::from_mir(self.rd().get())
    }
    #[doc = "set the value of operand rd at 0 to a value of type VFReg (checked by FPR32)"]
    pub fn set_rd(&self, value: VFReg) {
        let prev_value = self.get_rd();
        let checked_value = FPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rd().set(next_value.into_mir());
    }
    #[doc = "operand rn at index 1 of type VFReg"]
    pub fn rn(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand rn at index 1 of type VFReg"]
    pub fn get_rn(&self) -> VFReg {
        VFReg::from_mir(self.rn().get())
    }
    #[doc = "set the value of operand rn at 1 to a value of type VFReg (checked by FPR32)"]
    pub fn set_rn(&self, value: VFReg) {
        let prev_value = self.get_rn();
        let checked_value = FPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rn().set(next_value.into_mir());
    }
    #[doc = "operand rm at index 2 of type VFReg"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type VFReg"]
    pub fn get_rm(&self) -> VFReg {
        VFReg::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type VFReg (checked by FPR32)"]
    pub fn set_rm(&self, value: VFReg) {
        let prev_value = self.get_rm();
        let checked_value = FPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rm().set(next_value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct MirCopy64 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for MirCopy64 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..2usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::MirCopy64)
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(MirOperand::None.into_mir()),
            ],
        }
    }
}
impl MirCopy64 {
    pub fn new(opcode: MirOP, dst: GPR64, src: MirOperand) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(dst.into_mir()), Cell::new(src.into_mir())],
        }
    }
    #[doc = "operand dst at index 0 of type GPReg"]
    pub fn dst(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand dst at index 0 of type GPReg"]
    pub fn get_dst(&self) -> GPReg {
        GPReg::from_mir(self.dst().get())
    }
    #[doc = "set the value of operand dst at 0 to a value of type GPReg (checked by GPR64)"]
    pub fn set_dst(&self, value: GPReg) {
        let prev_value = self.get_dst();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.dst().set(next_value.into_mir());
    }
    #[doc = "operand src at index 1 of type MirOperand"]
    pub fn src(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand src at index 1 of type MirOperand"]
    pub fn get_src(&self) -> MirOperand {
        MirOperand::from_mir(self.src().get())
    }
    #[doc = "set the value of operand src at 1 to a value of type MirOperand (checked by MirOperand)"]
    pub fn set_src(&self, value: MirOperand) {
        self.src().set(value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct MirCopy32 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for MirCopy32 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..2usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::MirCopy32)
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(MirOperand::None.into_mir()),
            ],
        }
    }
}
impl MirCopy32 {
    pub fn new(opcode: MirOP, dst: GPR32, src: MirOperand) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(dst.into_mir()), Cell::new(src.into_mir())],
        }
    }
    #[doc = "operand dst at index 0 of type GPReg"]
    pub fn dst(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand dst at index 0 of type GPReg"]
    pub fn get_dst(&self) -> GPReg {
        GPReg::from_mir(self.dst().get())
    }
    #[doc = "set the value of operand dst at 0 to a value of type GPReg (checked by GPR32)"]
    pub fn set_dst(&self, value: GPReg) {
        let prev_value = self.get_dst();
        let checked_value = GPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.dst().set(next_value.into_mir());
    }
    #[doc = "operand src at index 1 of type MirOperand"]
    pub fn src(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand src at index 1 of type MirOperand"]
    pub fn get_src(&self) -> MirOperand {
        MirOperand::from_mir(self.src().get())
    }
    #[doc = "set the value of operand src at 1 to a value of type MirOperand (checked by MirOperand)"]
    pub fn set_src(&self, value: MirOperand) {
        self.src().set(value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct MirFCopy64 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for MirFCopy64 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..2usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::MirFCopy64)
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(MirOperand::None.into_mir()),
            ],
        }
    }
}
impl MirFCopy64 {
    pub fn new(opcode: MirOP, dst: FPR64, src: MirOperand) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(dst.into_mir()), Cell::new(src.into_mir())],
        }
    }
    #[doc = "operand dst at index 0 of type VFReg"]
    pub fn dst(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand dst at index 0 of type VFReg"]
    pub fn get_dst(&self) -> VFReg {
        VFReg::from_mir(self.dst().get())
    }
    #[doc = "set the value of operand dst at 0 to a value of type VFReg (checked by FPR64)"]
    pub fn set_dst(&self, value: VFReg) {
        let prev_value = self.get_dst();
        let checked_value = FPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.dst().set(next_value.into_mir());
    }
    #[doc = "operand src at index 1 of type MirOperand"]
    pub fn src(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand src at index 1 of type MirOperand"]
    pub fn get_src(&self) -> MirOperand {
        MirOperand::from_mir(self.src().get())
    }
    #[doc = "set the value of operand src at 1 to a value of type MirOperand (checked by MirOperand)"]
    pub fn set_src(&self, value: MirOperand) {
        self.src().set(value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct MirFCopy32 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for MirFCopy32 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..2usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::MirFCopy32)
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(MirOperand::None.into_mir()),
            ],
        }
    }
}
impl MirFCopy32 {
    pub fn new(opcode: MirOP, dst: FPR32, src: MirOperand) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(dst.into_mir()), Cell::new(src.into_mir())],
        }
    }
    #[doc = "operand dst at index 0 of type VFReg"]
    pub fn dst(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand dst at index 0 of type VFReg"]
    pub fn get_dst(&self) -> VFReg {
        VFReg::from_mir(self.dst().get())
    }
    #[doc = "set the value of operand dst at 0 to a value of type VFReg (checked by FPR32)"]
    pub fn set_dst(&self, value: VFReg) {
        let prev_value = self.get_dst();
        let checked_value = FPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.dst().set(next_value.into_mir());
    }
    #[doc = "operand src at index 1 of type MirOperand"]
    pub fn src(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand src at index 1 of type MirOperand"]
    pub fn get_src(&self) -> MirOperand {
        MirOperand::from_mir(self.src().get())
    }
    #[doc = "set the value of operand src at 1 to a value of type MirOperand (checked by MirOperand)"]
    pub fn set_src(&self, value: MirOperand) {
        self.src().set(value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct MirPCopy {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for MirPCopy {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..2usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::MirPCopy)
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(PState::new_empty().into_mir()),
                Cell::new(MirOperand::None.into_mir()),
            ],
        }
    }
}
impl MirPCopy {
    pub fn new(opcode: MirOP, dst: PState, src: MirOperand) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(dst.into_mir()), Cell::new(src.into_mir())],
        }
    }
    #[doc = "operand dst at index 0 of type PState"]
    pub fn dst(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand dst at index 0 of type PState"]
    pub fn get_dst(&self) -> PState {
        PState::from_mir(self.dst().get())
    }
    #[doc = "set the value of operand dst at 0 to a value of type PState (checked by PState)"]
    pub fn set_dst(&self, value: PState) {
        let prev_value = self.get_dst();
        let checked_value = PState::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.dst().set(next_value.into_mir());
    }
    #[doc = "operand src at index 1 of type MirOperand"]
    pub fn src(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand src at index 1 of type MirOperand"]
    pub fn get_src(&self) -> MirOperand {
        MirOperand::from_mir(self.src().get())
    }
    #[doc = "set the value of operand src at 1 to a value of type MirOperand (checked by MirOperand)"]
    pub fn set_src(&self, value: MirOperand) {
        self.src().set(value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct Una64R {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
    dst_op: Option<RegOP>,
}
impl IMirSubInst for Una64R {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..2usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(
            opcode,
            MirOP::Neg64R
                | MirOP::MVN64R
                | MirOP::Mov64R
                | MirOP::Abs64R
                | MirOP::CLS64
                | MirOP::CLZ64
                | MirOP::CNT64
                | MirOP::CTZ64
                | MirOP::RBit64
        )
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
            ],
            dst_op: None,
        }
    }
}
impl Una64R {
    pub fn new(opcode: MirOP, dst: GPR64, src: GPR64, dst_op: Option<RegOP>) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(dst.into_mir()), Cell::new(src.into_mir())],
            dst_op,
        }
    }
    #[doc = "operand dst at index 0 of type GPReg"]
    pub fn dst(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand dst at index 0 of type GPReg"]
    pub fn get_dst(&self) -> GPReg {
        GPReg::from_mir(self.dst().get())
    }
    #[doc = "set the value of operand dst at 0 to a value of type GPReg (checked by GPR64)"]
    pub fn set_dst(&self, value: GPReg) {
        let prev_value = self.get_dst();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.dst().set(next_value.into_mir());
    }
    #[doc = "operand src at index 1 of type GPReg"]
    pub fn src(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand src at index 1 of type GPReg"]
    pub fn get_src(&self) -> GPReg {
        GPReg::from_mir(self.src().get())
    }
    #[doc = "set the value of operand src at 1 to a value of type GPReg (checked by GPR64)"]
    pub fn set_src(&self, value: GPReg) {
        let prev_value = self.get_src();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.src().set(next_value.into_mir());
    }
    pub fn get_dst_op(&self) -> Option<RegOP> {
        self.dst_op.clone()
    }
    pub fn set_dst_op(&mut self, value: Option<RegOP>) {
        self.dst_op = value;
    }
}
#[derive(Debug, Clone)]
pub struct Una32R {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
    dst_op: Option<RegOP>,
}
impl IMirSubInst for Una32R {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..2usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(
            opcode,
            MirOP::Neg32R
                | MirOP::MVN32R
                | MirOP::Mov32R
                | MirOP::Abs32R
                | MirOP::CLS32
                | MirOP::CLZ32
                | MirOP::CNT32
                | MirOP::CTZ32
                | MirOP::RBit32
                | MirOP::SXTB32
                | MirOP::SXTH32
                | MirOP::SXTW32
                | MirOP::UXTB32
                | MirOP::UXTH32
        )
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
            ],
            dst_op: None,
        }
    }
}
impl Una32R {
    pub fn new(opcode: MirOP, dst: GPR32, src: GPR32, dst_op: Option<RegOP>) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(dst.into_mir()), Cell::new(src.into_mir())],
            dst_op,
        }
    }
    #[doc = "operand dst at index 0 of type GPReg"]
    pub fn dst(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand dst at index 0 of type GPReg"]
    pub fn get_dst(&self) -> GPReg {
        GPReg::from_mir(self.dst().get())
    }
    #[doc = "set the value of operand dst at 0 to a value of type GPReg (checked by GPR32)"]
    pub fn set_dst(&self, value: GPReg) {
        let prev_value = self.get_dst();
        let checked_value = GPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.dst().set(next_value.into_mir());
    }
    #[doc = "operand src at index 1 of type GPReg"]
    pub fn src(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand src at index 1 of type GPReg"]
    pub fn get_src(&self) -> GPReg {
        GPReg::from_mir(self.src().get())
    }
    #[doc = "set the value of operand src at 1 to a value of type GPReg (checked by GPR32)"]
    pub fn set_src(&self, value: GPReg) {
        let prev_value = self.get_src();
        let checked_value = GPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.src().set(next_value.into_mir());
    }
    pub fn get_dst_op(&self) -> Option<RegOP> {
        self.dst_op.clone()
    }
    pub fn set_dst_op(&mut self, value: Option<RegOP>) {
        self.dst_op = value;
    }
}
#[derive(Debug, Clone)]
pub struct ExtR {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for ExtR {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..2usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(
            opcode,
            MirOP::SXTB64 | MirOP::SXTH64 | MirOP::SXTW64 | MirOP::UXTB64 | MirOP::UXTH64
        )
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
            ],
        }
    }
}
impl ExtR {
    pub fn new(opcode: MirOP, dst: GPR64, src: GPR32) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(dst.into_mir()), Cell::new(src.into_mir())],
        }
    }
    #[doc = "operand dst at index 0 of type GPReg"]
    pub fn dst(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand dst at index 0 of type GPReg"]
    pub fn get_dst(&self) -> GPReg {
        GPReg::from_mir(self.dst().get())
    }
    #[doc = "set the value of operand dst at 0 to a value of type GPReg (checked by GPR64)"]
    pub fn set_dst(&self, value: GPReg) {
        let prev_value = self.get_dst();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.dst().set(next_value.into_mir());
    }
    #[doc = "operand src at index 1 of type GPReg"]
    pub fn src(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand src at index 1 of type GPReg"]
    pub fn get_src(&self) -> GPReg {
        GPReg::from_mir(self.src().get())
    }
    #[doc = "set the value of operand src at 1 to a value of type GPReg (checked by GPR32)"]
    pub fn set_src(&self, value: GPReg) {
        let prev_value = self.get_src();
        let checked_value = GPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.src().set(next_value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct Mov64I {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for Mov64I {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..2usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(
            opcode,
            MirOP::Mov64I | MirOP::MovZ64 | MirOP::MovN64 | MirOP::MovK64
        )
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmMov::new_empty().into_mir()),
            ],
        }
    }
}
impl Mov64I {
    pub fn new(opcode: MirOP, dst: GPR64, src: ImmMov) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(dst.into_mir()), Cell::new(src.into_mir())],
        }
    }
    #[doc = "operand dst at index 0 of type GPReg"]
    pub fn dst(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand dst at index 0 of type GPReg"]
    pub fn get_dst(&self) -> GPReg {
        GPReg::from_mir(self.dst().get())
    }
    #[doc = "set the value of operand dst at 0 to a value of type GPReg (checked by GPR64)"]
    pub fn set_dst(&self, value: GPReg) {
        let prev_value = self.get_dst();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.dst().set(next_value.into_mir());
    }
    #[doc = "operand src at index 1 of type ImmMov"]
    pub fn src(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand src at index 1 of type ImmMov"]
    pub fn get_src(&self) -> ImmMov {
        ImmMov::from_mir(self.src().get())
    }
    #[doc = "set the value of operand src at 1 to a value of type ImmMov (checked by ImmMov)"]
    pub fn set_src(&self, value: ImmMov) {
        self.src().set(value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct Mov32I {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for Mov32I {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..2usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(
            opcode,
            MirOP::Mov32I | MirOP::MovZ32 | MirOP::MovN32 | MirOP::MovK32
        )
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmMov::new_empty().into_mir()),
            ],
        }
    }
}
impl Mov32I {
    pub fn new(opcode: MirOP, dst: GPR32, src: ImmMov) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(dst.into_mir()), Cell::new(src.into_mir())],
        }
    }
    #[doc = "operand dst at index 0 of type GPReg"]
    pub fn dst(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand dst at index 0 of type GPReg"]
    pub fn get_dst(&self) -> GPReg {
        GPReg::from_mir(self.dst().get())
    }
    #[doc = "set the value of operand dst at 0 to a value of type GPReg (checked by GPR32)"]
    pub fn set_dst(&self, value: GPReg) {
        let prev_value = self.get_dst();
        let checked_value = GPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.dst().set(next_value.into_mir());
    }
    #[doc = "operand src at index 1 of type ImmMov"]
    pub fn src(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand src at index 1 of type ImmMov"]
    pub fn get_src(&self) -> ImmMov {
        ImmMov::from_mir(self.src().get())
    }
    #[doc = "set the value of operand src at 1 to a value of type ImmMov (checked by ImmMov)"]
    pub fn set_src(&self, value: ImmMov) {
        self.src().set(value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct Adr {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for Adr {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..2usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::AdrP | MirOP::Adr)
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(MirBlockRef::new_empty().into_mir()),
            ],
        }
    }
}
impl Adr {
    pub fn new(opcode: MirOP, dst: GPR64, src: MirBlockRef) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(dst.into_mir()), Cell::new(src.into_mir())],
        }
    }
    #[doc = "operand dst at index 0 of type GPReg"]
    pub fn dst(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand dst at index 0 of type GPReg"]
    pub fn get_dst(&self) -> GPReg {
        GPReg::from_mir(self.dst().get())
    }
    #[doc = "set the value of operand dst at 0 to a value of type GPReg (checked by GPR64)"]
    pub fn set_dst(&self, value: GPReg) {
        let prev_value = self.get_dst();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.dst().set(next_value.into_mir());
    }
    #[doc = "operand src at index 1 of type MirBlockRef"]
    pub fn src(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand src at index 1 of type MirBlockRef"]
    pub fn get_src(&self) -> MirBlockRef {
        MirBlockRef::from_mir(self.src().get())
    }
    #[doc = "set the value of operand src at 1 to a value of type MirBlockRef (checked by MirBlockRef)"]
    pub fn set_src(&self, value: MirBlockRef) {
        self.src().set(value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct UnaFG64 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for UnaFG64 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..2usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::FMovFG64 | MirOP::SCvtF64)
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
            ],
        }
    }
}
impl UnaFG64 {
    pub fn new(opcode: MirOP, dst: FPR64, src: GPR64) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(dst.into_mir()), Cell::new(src.into_mir())],
        }
    }
    #[doc = "operand dst at index 0 of type VFReg"]
    pub fn dst(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand dst at index 0 of type VFReg"]
    pub fn get_dst(&self) -> VFReg {
        VFReg::from_mir(self.dst().get())
    }
    #[doc = "set the value of operand dst at 0 to a value of type VFReg (checked by FPR64)"]
    pub fn set_dst(&self, value: VFReg) {
        let prev_value = self.get_dst();
        let checked_value = FPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.dst().set(next_value.into_mir());
    }
    #[doc = "operand src at index 1 of type GPReg"]
    pub fn src(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand src at index 1 of type GPReg"]
    pub fn get_src(&self) -> GPReg {
        GPReg::from_mir(self.src().get())
    }
    #[doc = "set the value of operand src at 1 to a value of type GPReg (checked by GPR64)"]
    pub fn set_src(&self, value: GPReg) {
        let prev_value = self.get_src();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.src().set(next_value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct UnaGF64 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for UnaGF64 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..2usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(
            opcode,
            MirOP::FMovGF64
                | MirOP::FCvtAS64
                | MirOP::FCvtAU64
                | MirOP::FCvtMS64
                | MirOP::FCvtMU64
                | MirOP::FCvtNS64
                | MirOP::FCvtNU64
                | MirOP::FCvtPS64
                | MirOP::FCvtPU64
                | MirOP::FCvtZS64
                | MirOP::FCvtZU64
        )
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
            ],
        }
    }
}
impl UnaGF64 {
    pub fn new(opcode: MirOP, dst: GPR64, src: FPR64) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(dst.into_mir()), Cell::new(src.into_mir())],
        }
    }
    #[doc = "operand dst at index 0 of type GPReg"]
    pub fn dst(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand dst at index 0 of type GPReg"]
    pub fn get_dst(&self) -> GPReg {
        GPReg::from_mir(self.dst().get())
    }
    #[doc = "set the value of operand dst at 0 to a value of type GPReg (checked by GPR64)"]
    pub fn set_dst(&self, value: GPReg) {
        let prev_value = self.get_dst();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.dst().set(next_value.into_mir());
    }
    #[doc = "operand src at index 1 of type VFReg"]
    pub fn src(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand src at index 1 of type VFReg"]
    pub fn get_src(&self) -> VFReg {
        VFReg::from_mir(self.src().get())
    }
    #[doc = "set the value of operand src at 1 to a value of type VFReg (checked by FPR64)"]
    pub fn set_src(&self, value: VFReg) {
        let prev_value = self.get_src();
        let checked_value = FPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.src().set(next_value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct UnaF64G32 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for UnaF64G32 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..2usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::SCvtF64G32)
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
            ],
        }
    }
}
impl UnaF64G32 {
    pub fn new(opcode: MirOP, dst: FPR64, src: GPR32) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(dst.into_mir()), Cell::new(src.into_mir())],
        }
    }
    #[doc = "operand dst at index 0 of type VFReg"]
    pub fn dst(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand dst at index 0 of type VFReg"]
    pub fn get_dst(&self) -> VFReg {
        VFReg::from_mir(self.dst().get())
    }
    #[doc = "set the value of operand dst at 0 to a value of type VFReg (checked by FPR64)"]
    pub fn set_dst(&self, value: VFReg) {
        let prev_value = self.get_dst();
        let checked_value = FPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.dst().set(next_value.into_mir());
    }
    #[doc = "operand src at index 1 of type GPReg"]
    pub fn src(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand src at index 1 of type GPReg"]
    pub fn get_src(&self) -> GPReg {
        GPReg::from_mir(self.src().get())
    }
    #[doc = "set the value of operand src at 1 to a value of type GPReg (checked by GPR32)"]
    pub fn set_src(&self, value: GPReg) {
        let prev_value = self.get_src();
        let checked_value = GPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.src().set(next_value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct UnaFG32 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for UnaFG32 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..2usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::FMovFG32 | MirOP::SCvtF32)
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
            ],
        }
    }
}
impl UnaFG32 {
    pub fn new(opcode: MirOP, dst: FPR32, src: GPR32) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(dst.into_mir()), Cell::new(src.into_mir())],
        }
    }
    #[doc = "operand dst at index 0 of type VFReg"]
    pub fn dst(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand dst at index 0 of type VFReg"]
    pub fn get_dst(&self) -> VFReg {
        VFReg::from_mir(self.dst().get())
    }
    #[doc = "set the value of operand dst at 0 to a value of type VFReg (checked by FPR32)"]
    pub fn set_dst(&self, value: VFReg) {
        let prev_value = self.get_dst();
        let checked_value = FPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.dst().set(next_value.into_mir());
    }
    #[doc = "operand src at index 1 of type GPReg"]
    pub fn src(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand src at index 1 of type GPReg"]
    pub fn get_src(&self) -> GPReg {
        GPReg::from_mir(self.src().get())
    }
    #[doc = "set the value of operand src at 1 to a value of type GPReg (checked by GPR32)"]
    pub fn set_src(&self, value: GPReg) {
        let prev_value = self.get_src();
        let checked_value = GPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.src().set(next_value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct UnaGF32 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for UnaGF32 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..2usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(
            opcode,
            MirOP::FMovGF32
                | MirOP::FCvtAS32
                | MirOP::FCvtAU32
                | MirOP::FCvtMS32
                | MirOP::FCvtMU32
                | MirOP::FCvtNS32
                | MirOP::FCvtNU32
                | MirOP::FCvtPS32
                | MirOP::FCvtPU32
                | MirOP::FCvtZS32
                | MirOP::FCvtZU32
        )
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
            ],
        }
    }
}
impl UnaGF32 {
    pub fn new(opcode: MirOP, dst: GPR32, src: FPR32) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(dst.into_mir()), Cell::new(src.into_mir())],
        }
    }
    #[doc = "operand dst at index 0 of type GPReg"]
    pub fn dst(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand dst at index 0 of type GPReg"]
    pub fn get_dst(&self) -> GPReg {
        GPReg::from_mir(self.dst().get())
    }
    #[doc = "set the value of operand dst at 0 to a value of type GPReg (checked by GPR32)"]
    pub fn set_dst(&self, value: GPReg) {
        let prev_value = self.get_dst();
        let checked_value = GPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.dst().set(next_value.into_mir());
    }
    #[doc = "operand src at index 1 of type VFReg"]
    pub fn src(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand src at index 1 of type VFReg"]
    pub fn get_src(&self) -> VFReg {
        VFReg::from_mir(self.src().get())
    }
    #[doc = "set the value of operand src at 1 to a value of type VFReg (checked by FPR32)"]
    pub fn set_src(&self, value: VFReg) {
        let prev_value = self.get_src();
        let checked_value = FPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.src().set(next_value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct UnaG64F32 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for UnaG64F32 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..2usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(
            opcode,
            MirOP::FCvtAS64F32
                | MirOP::FCvtAU64F32
                | MirOP::FCvtMS64F32
                | MirOP::FCvtMU64F32
                | MirOP::FCvtNS64F32
                | MirOP::FCvtNU64F32
                | MirOP::FCvtPS64F32
                | MirOP::FCvtPU64F32
                | MirOP::FCvtZS64F32
                | MirOP::FCvtZU64F32
        )
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
            ],
        }
    }
}
impl UnaG64F32 {
    pub fn new(opcode: MirOP, dst: GPR64, src: FPR32) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(dst.into_mir()), Cell::new(src.into_mir())],
        }
    }
    #[doc = "operand dst at index 0 of type GPReg"]
    pub fn dst(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand dst at index 0 of type GPReg"]
    pub fn get_dst(&self) -> GPReg {
        GPReg::from_mir(self.dst().get())
    }
    #[doc = "set the value of operand dst at 0 to a value of type GPReg (checked by GPR64)"]
    pub fn set_dst(&self, value: GPReg) {
        let prev_value = self.get_dst();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.dst().set(next_value.into_mir());
    }
    #[doc = "operand src at index 1 of type VFReg"]
    pub fn src(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand src at index 1 of type VFReg"]
    pub fn get_src(&self) -> VFReg {
        VFReg::from_mir(self.src().get())
    }
    #[doc = "set the value of operand src at 1 to a value of type VFReg (checked by FPR32)"]
    pub fn set_src(&self, value: VFReg) {
        let prev_value = self.get_src();
        let checked_value = FPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.src().set(next_value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct UnaG32F64 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for UnaG32F64 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..2usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::FCvtAS32F64 | MirOP::FCvtAU32F64)
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
            ],
        }
    }
}
impl UnaG32F64 {
    pub fn new(opcode: MirOP, dst: GPR32, src: FPR64) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(dst.into_mir()), Cell::new(src.into_mir())],
        }
    }
    #[doc = "operand dst at index 0 of type GPReg"]
    pub fn dst(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand dst at index 0 of type GPReg"]
    pub fn get_dst(&self) -> GPReg {
        GPReg::from_mir(self.dst().get())
    }
    #[doc = "set the value of operand dst at 0 to a value of type GPReg (checked by GPR32)"]
    pub fn set_dst(&self, value: GPReg) {
        let prev_value = self.get_dst();
        let checked_value = GPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.dst().set(next_value.into_mir());
    }
    #[doc = "operand src at index 1 of type VFReg"]
    pub fn src(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand src at index 1 of type VFReg"]
    pub fn get_src(&self) -> VFReg {
        VFReg::from_mir(self.src().get())
    }
    #[doc = "set the value of operand src at 1 to a value of type VFReg (checked by FPR64)"]
    pub fn set_src(&self, value: VFReg) {
        let prev_value = self.get_src();
        let checked_value = FPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.src().set(next_value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct UnaF64 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for UnaF64 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..2usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(
            opcode,
            MirOP::FMov64R
                | MirOP::FRIntA64
                | MirOP::FRIntI64
                | MirOP::FRIntM64
                | MirOP::FRIntN64
                | MirOP::FRIntP64
                | MirOP::FRIntX64
                | MirOP::FRIntZ64
                | MirOP::FRInt32X64
                | MirOP::FRIntZ32X64
                | MirOP::FRInt64X64
                | MirOP::FRIntZ64X64
                | MirOP::FAbs64
                | MirOP::FNeg64
                | MirOP::FSqrt64
        )
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
            ],
        }
    }
}
impl UnaF64 {
    pub fn new(opcode: MirOP, dst: FPR64, src: FPR64) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(dst.into_mir()), Cell::new(src.into_mir())],
        }
    }
    #[doc = "operand dst at index 0 of type VFReg"]
    pub fn dst(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand dst at index 0 of type VFReg"]
    pub fn get_dst(&self) -> VFReg {
        VFReg::from_mir(self.dst().get())
    }
    #[doc = "set the value of operand dst at 0 to a value of type VFReg (checked by FPR64)"]
    pub fn set_dst(&self, value: VFReg) {
        let prev_value = self.get_dst();
        let checked_value = FPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.dst().set(next_value.into_mir());
    }
    #[doc = "operand src at index 1 of type VFReg"]
    pub fn src(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand src at index 1 of type VFReg"]
    pub fn get_src(&self) -> VFReg {
        VFReg::from_mir(self.src().get())
    }
    #[doc = "set the value of operand src at 1 to a value of type VFReg (checked by FPR64)"]
    pub fn set_src(&self, value: VFReg) {
        let prev_value = self.get_src();
        let checked_value = FPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.src().set(next_value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct UnaF32 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for UnaF32 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..2usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(
            opcode,
            MirOP::FMov32R
                | MirOP::FRIntA32
                | MirOP::FRIntI32
                | MirOP::FRIntM32
                | MirOP::FRIntN32
                | MirOP::FRIntP32
                | MirOP::FRIntX32
                | MirOP::FRIntZ32
                | MirOP::FRInt32X32
                | MirOP::FRIntZ32X32
                | MirOP::FRInt64X32
                | MirOP::FRIntZ64X32
                | MirOP::FAbs32
                | MirOP::FNeg32
                | MirOP::FSqrt32
        )
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
            ],
        }
    }
}
impl UnaF32 {
    pub fn new(opcode: MirOP, dst: FPR32, src: FPR32) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(dst.into_mir()), Cell::new(src.into_mir())],
        }
    }
    #[doc = "operand dst at index 0 of type VFReg"]
    pub fn dst(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand dst at index 0 of type VFReg"]
    pub fn get_dst(&self) -> VFReg {
        VFReg::from_mir(self.dst().get())
    }
    #[doc = "set the value of operand dst at 0 to a value of type VFReg (checked by FPR32)"]
    pub fn set_dst(&self, value: VFReg) {
        let prev_value = self.get_dst();
        let checked_value = FPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.dst().set(next_value.into_mir());
    }
    #[doc = "operand src at index 1 of type VFReg"]
    pub fn src(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand src at index 1 of type VFReg"]
    pub fn get_src(&self) -> VFReg {
        VFReg::from_mir(self.src().get())
    }
    #[doc = "set the value of operand src at 1 to a value of type VFReg (checked by FPR32)"]
    pub fn set_src(&self, value: VFReg) {
        let prev_value = self.get_src();
        let checked_value = FPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.src().set(next_value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct UnaryF32F64 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for UnaryF32F64 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..2usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::FCvt32F64)
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
            ],
        }
    }
}
impl UnaryF32F64 {
    pub fn new(opcode: MirOP, dst: FPR32, src: FPR64) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(dst.into_mir()), Cell::new(src.into_mir())],
        }
    }
    #[doc = "operand dst at index 0 of type VFReg"]
    pub fn dst(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand dst at index 0 of type VFReg"]
    pub fn get_dst(&self) -> VFReg {
        VFReg::from_mir(self.dst().get())
    }
    #[doc = "set the value of operand dst at 0 to a value of type VFReg (checked by FPR32)"]
    pub fn set_dst(&self, value: VFReg) {
        let prev_value = self.get_dst();
        let checked_value = FPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.dst().set(next_value.into_mir());
    }
    #[doc = "operand src at index 1 of type VFReg"]
    pub fn src(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand src at index 1 of type VFReg"]
    pub fn get_src(&self) -> VFReg {
        VFReg::from_mir(self.src().get())
    }
    #[doc = "set the value of operand src at 1 to a value of type VFReg (checked by FPR64)"]
    pub fn set_src(&self, value: VFReg) {
        let prev_value = self.get_src();
        let checked_value = FPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.src().set(next_value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct UnaryF64F32 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for UnaryF64F32 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..2usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::FCvt64F32)
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
            ],
        }
    }
}
impl UnaryF64F32 {
    pub fn new(opcode: MirOP, dst: FPR64, src: FPR32) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(dst.into_mir()), Cell::new(src.into_mir())],
        }
    }
    #[doc = "operand dst at index 0 of type VFReg"]
    pub fn dst(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand dst at index 0 of type VFReg"]
    pub fn get_dst(&self) -> VFReg {
        VFReg::from_mir(self.dst().get())
    }
    #[doc = "set the value of operand dst at 0 to a value of type VFReg (checked by FPR64)"]
    pub fn set_dst(&self, value: VFReg) {
        let prev_value = self.get_dst();
        let checked_value = FPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.dst().set(next_value.into_mir());
    }
    #[doc = "operand src at index 1 of type VFReg"]
    pub fn src(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand src at index 1 of type VFReg"]
    pub fn get_src(&self) -> VFReg {
        VFReg::from_mir(self.src().get())
    }
    #[doc = "set the value of operand src at 1 to a value of type VFReg (checked by FPR32)"]
    pub fn set_src(&self, value: VFReg) {
        let prev_value = self.get_src();
        let checked_value = FPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.src().set(next_value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct FMov64I {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for FMov64I {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..2usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::FMov64I)
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(ImmFMov64::new_empty().into_mir()),
            ],
        }
    }
}
impl FMov64I {
    pub fn new(opcode: MirOP, dst: FPR64, src: ImmFMov64) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(dst.into_mir()), Cell::new(src.into_mir())],
        }
    }
    #[doc = "operand dst at index 0 of type VFReg"]
    pub fn dst(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand dst at index 0 of type VFReg"]
    pub fn get_dst(&self) -> VFReg {
        VFReg::from_mir(self.dst().get())
    }
    #[doc = "set the value of operand dst at 0 to a value of type VFReg (checked by FPR64)"]
    pub fn set_dst(&self, value: VFReg) {
        let prev_value = self.get_dst();
        let checked_value = FPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.dst().set(next_value.into_mir());
    }
    #[doc = "operand src at index 1 of type ImmFMov64"]
    pub fn src(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand src at index 1 of type ImmFMov64"]
    pub fn get_src(&self) -> ImmFMov64 {
        ImmFMov64::from_mir(self.src().get())
    }
    #[doc = "set the value of operand src at 1 to a value of type ImmFMov64 (checked by ImmFMov64)"]
    pub fn set_src(&self, value: ImmFMov64) {
        self.src().set(value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct FMov32I {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for FMov32I {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..2usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::FMov32I)
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(ImmFMov32::new_empty().into_mir()),
            ],
        }
    }
}
impl FMov32I {
    pub fn new(opcode: MirOP, dst: FPR32, src: ImmFMov32) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(dst.into_mir()), Cell::new(src.into_mir())],
        }
    }
    #[doc = "operand dst at index 0 of type VFReg"]
    pub fn dst(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand dst at index 0 of type VFReg"]
    pub fn get_dst(&self) -> VFReg {
        VFReg::from_mir(self.dst().get())
    }
    #[doc = "set the value of operand dst at 0 to a value of type VFReg (checked by FPR32)"]
    pub fn set_dst(&self, value: VFReg) {
        let prev_value = self.get_dst();
        let checked_value = FPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.dst().set(next_value.into_mir());
    }
    #[doc = "operand src at index 1 of type ImmFMov32"]
    pub fn src(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand src at index 1 of type ImmFMov32"]
    pub fn get_src(&self) -> ImmFMov32 {
        ImmFMov32::from_mir(self.src().get())
    }
    #[doc = "set the value of operand src at 1 to a value of type ImmFMov32 (checked by ImmFMov32)"]
    pub fn set_src(&self, value: ImmFMov32) {
        self.src().set(value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct TenaryG64 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 4usize],
}
impl IMirSubInst for TenaryG64 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..4usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::MAdd64 | MirOP::MSub64)
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
            ],
        }
    }
}
impl TenaryG64 {
    pub fn new(opcode: MirOP, rd: GPR64, rn: GPR64, rm: GPR64, rs: GPR64) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.into_mir()),
                Cell::new(rn.into_mir()),
                Cell::new(rm.into_mir()),
                Cell::new(rs.into_mir()),
            ],
        }
    }
    #[doc = "operand rd at index 0 of type GPReg"]
    pub fn rd(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand rd at index 0 of type GPReg"]
    pub fn get_rd(&self) -> GPReg {
        GPReg::from_mir(self.rd().get())
    }
    #[doc = "set the value of operand rd at 0 to a value of type GPReg (checked by GPR64)"]
    pub fn set_rd(&self, value: GPReg) {
        let prev_value = self.get_rd();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rd().set(next_value.into_mir());
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn rn(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn get_rn(&self) -> GPReg {
        GPReg::from_mir(self.rn().get())
    }
    #[doc = "set the value of operand rn at 1 to a value of type GPReg (checked by GPR64)"]
    pub fn set_rn(&self, value: GPReg) {
        let prev_value = self.get_rn();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rn().set(next_value.into_mir());
    }
    #[doc = "operand rm at index 2 of type GPReg"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type GPReg"]
    pub fn get_rm(&self) -> GPReg {
        GPReg::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type GPReg (checked by GPR64)"]
    pub fn set_rm(&self, value: GPReg) {
        let prev_value = self.get_rm();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rm().set(next_value.into_mir());
    }
    #[doc = "operand rs at index 3 of type GPReg"]
    pub fn rs(&self) -> &Cell<MirOperand> {
        &self._operands[3usize]
    }
    #[doc = "operand rs at index 3 of type GPReg"]
    pub fn get_rs(&self) -> GPReg {
        GPReg::from_mir(self.rs().get())
    }
    #[doc = "set the value of operand rs at 3 to a value of type GPReg (checked by GPR64)"]
    pub fn set_rs(&self, value: GPReg) {
        let prev_value = self.get_rs();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rs().set(next_value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct TenaryG64G32 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 4usize],
}
impl IMirSubInst for TenaryG64G32 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..4usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(
            opcode,
            MirOP::SMAddL | MirOP::SMSubL | MirOP::UMAddL | MirOP::UMSubL
        )
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
            ],
        }
    }
}
impl TenaryG64G32 {
    pub fn new(opcode: MirOP, rd: GPR64, rn: GPR32, rm: GPR32, rs: GPR64) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.into_mir()),
                Cell::new(rn.into_mir()),
                Cell::new(rm.into_mir()),
                Cell::new(rs.into_mir()),
            ],
        }
    }
    #[doc = "operand rd at index 0 of type GPReg"]
    pub fn rd(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand rd at index 0 of type GPReg"]
    pub fn get_rd(&self) -> GPReg {
        GPReg::from_mir(self.rd().get())
    }
    #[doc = "set the value of operand rd at 0 to a value of type GPReg (checked by GPR64)"]
    pub fn set_rd(&self, value: GPReg) {
        let prev_value = self.get_rd();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rd().set(next_value.into_mir());
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn rn(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn get_rn(&self) -> GPReg {
        GPReg::from_mir(self.rn().get())
    }
    #[doc = "set the value of operand rn at 1 to a value of type GPReg (checked by GPR32)"]
    pub fn set_rn(&self, value: GPReg) {
        let prev_value = self.get_rn();
        let checked_value = GPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rn().set(next_value.into_mir());
    }
    #[doc = "operand rm at index 2 of type GPReg"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type GPReg"]
    pub fn get_rm(&self) -> GPReg {
        GPReg::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type GPReg (checked by GPR32)"]
    pub fn set_rm(&self, value: GPReg) {
        let prev_value = self.get_rm();
        let checked_value = GPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rm().set(next_value.into_mir());
    }
    #[doc = "operand rs at index 3 of type GPReg"]
    pub fn rs(&self) -> &Cell<MirOperand> {
        &self._operands[3usize]
    }
    #[doc = "operand rs at index 3 of type GPReg"]
    pub fn get_rs(&self) -> GPReg {
        GPReg::from_mir(self.rs().get())
    }
    #[doc = "set the value of operand rs at 3 to a value of type GPReg (checked by GPR64)"]
    pub fn set_rs(&self, value: GPReg) {
        let prev_value = self.get_rs();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rs().set(next_value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct TenaryG32 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 4usize],
}
impl IMirSubInst for TenaryG32 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..4usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::MAdd32 | MirOP::MSub32)
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
            ],
        }
    }
}
impl TenaryG32 {
    pub fn new(opcode: MirOP, rd: GPR32, rn: GPR32, rm: GPR32, rs: GPR32) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.into_mir()),
                Cell::new(rn.into_mir()),
                Cell::new(rm.into_mir()),
                Cell::new(rs.into_mir()),
            ],
        }
    }
    #[doc = "operand rd at index 0 of type GPReg"]
    pub fn rd(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand rd at index 0 of type GPReg"]
    pub fn get_rd(&self) -> GPReg {
        GPReg::from_mir(self.rd().get())
    }
    #[doc = "set the value of operand rd at 0 to a value of type GPReg (checked by GPR32)"]
    pub fn set_rd(&self, value: GPReg) {
        let prev_value = self.get_rd();
        let checked_value = GPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rd().set(next_value.into_mir());
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn rn(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn get_rn(&self) -> GPReg {
        GPReg::from_mir(self.rn().get())
    }
    #[doc = "set the value of operand rn at 1 to a value of type GPReg (checked by GPR32)"]
    pub fn set_rn(&self, value: GPReg) {
        let prev_value = self.get_rn();
        let checked_value = GPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rn().set(next_value.into_mir());
    }
    #[doc = "operand rm at index 2 of type GPReg"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type GPReg"]
    pub fn get_rm(&self) -> GPReg {
        GPReg::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type GPReg (checked by GPR32)"]
    pub fn set_rm(&self, value: GPReg) {
        let prev_value = self.get_rm();
        let checked_value = GPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rm().set(next_value.into_mir());
    }
    #[doc = "operand rs at index 3 of type GPReg"]
    pub fn rs(&self) -> &Cell<MirOperand> {
        &self._operands[3usize]
    }
    #[doc = "operand rs at index 3 of type GPReg"]
    pub fn get_rs(&self) -> GPReg {
        GPReg::from_mir(self.rs().get())
    }
    #[doc = "set the value of operand rs at 3 to a value of type GPReg (checked by GPR32)"]
    pub fn set_rs(&self, value: GPReg) {
        let prev_value = self.get_rs();
        let checked_value = GPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rs().set(next_value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct TenaryF64 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 4usize],
}
impl IMirSubInst for TenaryF64 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..4usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(
            opcode,
            MirOP::FMAdd64 | MirOP::FMSub64 | MirOP::FNMAdd64 | MirOP::FNMSub64
        )
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
            ],
        }
    }
}
impl TenaryF64 {
    pub fn new(opcode: MirOP, rd: FPR64, rn: FPR64, rm: FPR64, rs: FPR64) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.into_mir()),
                Cell::new(rn.into_mir()),
                Cell::new(rm.into_mir()),
                Cell::new(rs.into_mir()),
            ],
        }
    }
    #[doc = "operand rd at index 0 of type VFReg"]
    pub fn rd(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand rd at index 0 of type VFReg"]
    pub fn get_rd(&self) -> VFReg {
        VFReg::from_mir(self.rd().get())
    }
    #[doc = "set the value of operand rd at 0 to a value of type VFReg (checked by FPR64)"]
    pub fn set_rd(&self, value: VFReg) {
        let prev_value = self.get_rd();
        let checked_value = FPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rd().set(next_value.into_mir());
    }
    #[doc = "operand rn at index 1 of type VFReg"]
    pub fn rn(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand rn at index 1 of type VFReg"]
    pub fn get_rn(&self) -> VFReg {
        VFReg::from_mir(self.rn().get())
    }
    #[doc = "set the value of operand rn at 1 to a value of type VFReg (checked by FPR64)"]
    pub fn set_rn(&self, value: VFReg) {
        let prev_value = self.get_rn();
        let checked_value = FPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rn().set(next_value.into_mir());
    }
    #[doc = "operand rm at index 2 of type VFReg"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type VFReg"]
    pub fn get_rm(&self) -> VFReg {
        VFReg::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type VFReg (checked by FPR64)"]
    pub fn set_rm(&self, value: VFReg) {
        let prev_value = self.get_rm();
        let checked_value = FPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rm().set(next_value.into_mir());
    }
    #[doc = "operand rs at index 3 of type VFReg"]
    pub fn rs(&self) -> &Cell<MirOperand> {
        &self._operands[3usize]
    }
    #[doc = "operand rs at index 3 of type VFReg"]
    pub fn get_rs(&self) -> VFReg {
        VFReg::from_mir(self.rs().get())
    }
    #[doc = "set the value of operand rs at 3 to a value of type VFReg (checked by FPR64)"]
    pub fn set_rs(&self, value: VFReg) {
        let prev_value = self.get_rs();
        let checked_value = FPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rs().set(next_value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct TenaryF32 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 4usize],
}
impl IMirSubInst for TenaryF32 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..4usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(
            opcode,
            MirOP::FMAdd32 | MirOP::FMSub32 | MirOP::FNMAdd32 | MirOP::FNMSub32
        )
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
            ],
        }
    }
}
impl TenaryF32 {
    pub fn new(opcode: MirOP, rd: FPR32, rn: FPR32, rm: FPR32, rs: FPR32) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.into_mir()),
                Cell::new(rn.into_mir()),
                Cell::new(rm.into_mir()),
                Cell::new(rs.into_mir()),
            ],
        }
    }
    #[doc = "operand rd at index 0 of type VFReg"]
    pub fn rd(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand rd at index 0 of type VFReg"]
    pub fn get_rd(&self) -> VFReg {
        VFReg::from_mir(self.rd().get())
    }
    #[doc = "set the value of operand rd at 0 to a value of type VFReg (checked by FPR32)"]
    pub fn set_rd(&self, value: VFReg) {
        let prev_value = self.get_rd();
        let checked_value = FPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rd().set(next_value.into_mir());
    }
    #[doc = "operand rn at index 1 of type VFReg"]
    pub fn rn(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand rn at index 1 of type VFReg"]
    pub fn get_rn(&self) -> VFReg {
        VFReg::from_mir(self.rn().get())
    }
    #[doc = "set the value of operand rn at 1 to a value of type VFReg (checked by FPR32)"]
    pub fn set_rn(&self, value: VFReg) {
        let prev_value = self.get_rn();
        let checked_value = FPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rn().set(next_value.into_mir());
    }
    #[doc = "operand rm at index 2 of type VFReg"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type VFReg"]
    pub fn get_rm(&self) -> VFReg {
        VFReg::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type VFReg (checked by FPR32)"]
    pub fn set_rm(&self, value: VFReg) {
        let prev_value = self.get_rm();
        let checked_value = FPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rm().set(next_value.into_mir());
    }
    #[doc = "operand rs at index 3 of type VFReg"]
    pub fn rs(&self) -> &Cell<MirOperand> {
        &self._operands[3usize]
    }
    #[doc = "operand rs at index 3 of type VFReg"]
    pub fn get_rs(&self) -> VFReg {
        VFReg::from_mir(self.rs().get())
    }
    #[doc = "set the value of operand rs at 3 to a value of type VFReg (checked by FPR32)"]
    pub fn set_rs(&self, value: VFReg) {
        let prev_value = self.get_rs();
        let checked_value = FPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rs().set(next_value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct LoadStoreGr64 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
    rm_op: Option<RegOP>,
}
impl IMirSubInst for LoadStoreGr64 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(
            opcode,
            MirOP::LdrGr64
                | MirOP::LdrBGr64
                | MirOP::LdrHGr64
                | MirOP::LdrSBGr64
                | MirOP::LdrSHGr64
                | MirOP::StrGr64
                | MirOP::StrBGr64
                | MirOP::StrHGr64
        )
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
            ],
            rm_op: None,
        }
    }
}
impl LoadStoreGr64 {
    pub fn new(opcode: MirOP, rd: GPR64, rn: GPR64, rm: GPR64, rm_op: Option<RegOP>) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.into_mir()),
                Cell::new(rn.into_mir()),
                Cell::new(rm.into_mir()),
            ],
            rm_op,
        }
    }
    #[doc = "operand rd at index 0 of type GPReg"]
    pub fn rd(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand rd at index 0 of type GPReg"]
    pub fn get_rd(&self) -> GPReg {
        GPReg::from_mir(self.rd().get())
    }
    #[doc = "set the value of operand rd at 0 to a value of type GPReg (checked by GPR64)"]
    pub fn set_rd(&self, value: GPReg) {
        let prev_value = self.get_rd();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rd().set(next_value.into_mir());
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn rn(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn get_rn(&self) -> GPReg {
        GPReg::from_mir(self.rn().get())
    }
    #[doc = "set the value of operand rn at 1 to a value of type GPReg (checked by GPR64)"]
    pub fn set_rn(&self, value: GPReg) {
        let prev_value = self.get_rn();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rn().set(next_value.into_mir());
    }
    #[doc = "operand rm at index 2 of type GPReg"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type GPReg"]
    pub fn get_rm(&self) -> GPReg {
        GPReg::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type GPReg (checked by GPR64)"]
    pub fn set_rm(&self, value: GPReg) {
        let prev_value = self.get_rm();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rm().set(next_value.into_mir());
    }
    pub fn get_rm_op(&self) -> Option<RegOP> {
        self.rm_op.clone()
    }
    pub fn set_rm_op(&mut self, value: Option<RegOP>) {
        self.rm_op = value;
    }
}
#[derive(Debug, Clone)]
pub struct LoadStoreGr32 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
    rm_op: Option<RegOP>,
}
impl IMirSubInst for LoadStoreGr32 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(
            opcode,
            MirOP::LdrGr32
                | MirOP::LdrBGr32
                | MirOP::LdrHGr32
                | MirOP::LdrSBGr32
                | MirOP::LdrSHGr32
                | MirOP::StrGr32
                | MirOP::StrBGr32
                | MirOP::StrHGr32
        )
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
            ],
            rm_op: None,
        }
    }
}
impl LoadStoreGr32 {
    pub fn new(opcode: MirOP, rd: GPR32, rn: GPR64, rm: GPR64, rm_op: Option<RegOP>) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.into_mir()),
                Cell::new(rn.into_mir()),
                Cell::new(rm.into_mir()),
            ],
            rm_op,
        }
    }
    #[doc = "operand rd at index 0 of type GPReg"]
    pub fn rd(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand rd at index 0 of type GPReg"]
    pub fn get_rd(&self) -> GPReg {
        GPReg::from_mir(self.rd().get())
    }
    #[doc = "set the value of operand rd at 0 to a value of type GPReg (checked by GPR32)"]
    pub fn set_rd(&self, value: GPReg) {
        let prev_value = self.get_rd();
        let checked_value = GPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rd().set(next_value.into_mir());
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn rn(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn get_rn(&self) -> GPReg {
        GPReg::from_mir(self.rn().get())
    }
    #[doc = "set the value of operand rn at 1 to a value of type GPReg (checked by GPR64)"]
    pub fn set_rn(&self, value: GPReg) {
        let prev_value = self.get_rn();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rn().set(next_value.into_mir());
    }
    #[doc = "operand rm at index 2 of type GPReg"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type GPReg"]
    pub fn get_rm(&self) -> GPReg {
        GPReg::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type GPReg (checked by GPR64)"]
    pub fn set_rm(&self, value: GPReg) {
        let prev_value = self.get_rm();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rm().set(next_value.into_mir());
    }
    pub fn get_rm_op(&self) -> Option<RegOP> {
        self.rm_op.clone()
    }
    pub fn set_rm_op(&mut self, value: Option<RegOP>) {
        self.rm_op = value;
    }
}
#[derive(Debug, Clone)]
pub struct LoadStoreF64 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
    rm_op: Option<RegOP>,
}
impl IMirSubInst for LoadStoreF64 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::LdrF64 | MirOP::StrF64)
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
            ],
            rm_op: None,
        }
    }
}
impl LoadStoreF64 {
    pub fn new(opcode: MirOP, rd: FPR64, rn: GPR64, rm: GPR64, rm_op: Option<RegOP>) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.into_mir()),
                Cell::new(rn.into_mir()),
                Cell::new(rm.into_mir()),
            ],
            rm_op,
        }
    }
    #[doc = "operand rd at index 0 of type VFReg"]
    pub fn rd(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand rd at index 0 of type VFReg"]
    pub fn get_rd(&self) -> VFReg {
        VFReg::from_mir(self.rd().get())
    }
    #[doc = "set the value of operand rd at 0 to a value of type VFReg (checked by FPR64)"]
    pub fn set_rd(&self, value: VFReg) {
        let prev_value = self.get_rd();
        let checked_value = FPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rd().set(next_value.into_mir());
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn rn(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn get_rn(&self) -> GPReg {
        GPReg::from_mir(self.rn().get())
    }
    #[doc = "set the value of operand rn at 1 to a value of type GPReg (checked by GPR64)"]
    pub fn set_rn(&self, value: GPReg) {
        let prev_value = self.get_rn();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rn().set(next_value.into_mir());
    }
    #[doc = "operand rm at index 2 of type GPReg"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type GPReg"]
    pub fn get_rm(&self) -> GPReg {
        GPReg::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type GPReg (checked by GPR64)"]
    pub fn set_rm(&self, value: GPReg) {
        let prev_value = self.get_rm();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rm().set(next_value.into_mir());
    }
    pub fn get_rm_op(&self) -> Option<RegOP> {
        self.rm_op.clone()
    }
    pub fn set_rm_op(&mut self, value: Option<RegOP>) {
        self.rm_op = value;
    }
}
#[derive(Debug, Clone)]
pub struct LoadStoreF32 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
    rm_op: Option<RegOP>,
}
impl IMirSubInst for LoadStoreF32 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::LdrF32 | MirOP::StrF32)
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
            ],
            rm_op: None,
        }
    }
}
impl LoadStoreF32 {
    pub fn new(opcode: MirOP, rd: FPR32, rn: GPR64, rm: GPR64, rm_op: Option<RegOP>) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.into_mir()),
                Cell::new(rn.into_mir()),
                Cell::new(rm.into_mir()),
            ],
            rm_op,
        }
    }
    #[doc = "operand rd at index 0 of type VFReg"]
    pub fn rd(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand rd at index 0 of type VFReg"]
    pub fn get_rd(&self) -> VFReg {
        VFReg::from_mir(self.rd().get())
    }
    #[doc = "set the value of operand rd at 0 to a value of type VFReg (checked by FPR32)"]
    pub fn set_rd(&self, value: VFReg) {
        let prev_value = self.get_rd();
        let checked_value = FPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rd().set(next_value.into_mir());
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn rn(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn get_rn(&self) -> GPReg {
        GPReg::from_mir(self.rn().get())
    }
    #[doc = "set the value of operand rn at 1 to a value of type GPReg (checked by GPR64)"]
    pub fn set_rn(&self, value: GPReg) {
        let prev_value = self.get_rn();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rn().set(next_value.into_mir());
    }
    #[doc = "operand rm at index 2 of type GPReg"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type GPReg"]
    pub fn get_rm(&self) -> GPReg {
        GPReg::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type GPReg (checked by GPR64)"]
    pub fn set_rm(&self, value: GPReg) {
        let prev_value = self.get_rm();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rm().set(next_value.into_mir());
    }
    pub fn get_rm_op(&self) -> Option<RegOP> {
        self.rm_op.clone()
    }
    pub fn set_rm_op(&mut self, value: Option<RegOP>) {
        self.rm_op = value;
    }
}
#[derive(Debug, Clone)]
pub struct LoadStoreGr64Base {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for LoadStoreGr64Base {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(
            opcode,
            MirOP::LdrGr64Base
                | MirOP::LdrBGr64Base
                | MirOP::LdrHGr64Base
                | MirOP::LdrSBGr64Base
                | MirOP::LdrSHGr64Base
                | MirOP::StrGr64Base
                | MirOP::StrBGr64Base
                | MirOP::StrHGr64Base
        )
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmLoad64::new_empty().into_mir()),
            ],
        }
    }
}
impl LoadStoreGr64Base {
    pub fn new(opcode: MirOP, rd: GPR64, rn: GPR64, rm: ImmLoad64) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.into_mir()),
                Cell::new(rn.into_mir()),
                Cell::new(rm.into_mir()),
            ],
        }
    }
    #[doc = "operand rd at index 0 of type GPReg"]
    pub fn rd(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand rd at index 0 of type GPReg"]
    pub fn get_rd(&self) -> GPReg {
        GPReg::from_mir(self.rd().get())
    }
    #[doc = "set the value of operand rd at 0 to a value of type GPReg (checked by GPR64)"]
    pub fn set_rd(&self, value: GPReg) {
        let prev_value = self.get_rd();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rd().set(next_value.into_mir());
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn rn(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn get_rn(&self) -> GPReg {
        GPReg::from_mir(self.rn().get())
    }
    #[doc = "set the value of operand rn at 1 to a value of type GPReg (checked by GPR64)"]
    pub fn set_rn(&self, value: GPReg) {
        let prev_value = self.get_rn();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rn().set(next_value.into_mir());
    }
    #[doc = "operand rm at index 2 of type ImmLoad64"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type ImmLoad64"]
    pub fn get_rm(&self) -> ImmLoad64 {
        ImmLoad64::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type ImmLoad64 (checked by ImmLoad64)"]
    pub fn set_rm(&self, value: ImmLoad64) {
        self.rm().set(value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct LoadStoreGr32Base {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for LoadStoreGr32Base {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(
            opcode,
            MirOP::LdrGr32Base
                | MirOP::LdrBGr32Base
                | MirOP::LdrHGr32Base
                | MirOP::LdrSBGr32Base
                | MirOP::LdrSHGr32Base
                | MirOP::StrGr32Base
                | MirOP::StrBGr32Base
                | MirOP::StrHGr32Base
        )
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmLoad32::new_empty().into_mir()),
            ],
        }
    }
}
impl LoadStoreGr32Base {
    pub fn new(opcode: MirOP, rd: GPR32, rn: GPR64, rm: ImmLoad32) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.into_mir()),
                Cell::new(rn.into_mir()),
                Cell::new(rm.into_mir()),
            ],
        }
    }
    #[doc = "operand rd at index 0 of type GPReg"]
    pub fn rd(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand rd at index 0 of type GPReg"]
    pub fn get_rd(&self) -> GPReg {
        GPReg::from_mir(self.rd().get())
    }
    #[doc = "set the value of operand rd at 0 to a value of type GPReg (checked by GPR32)"]
    pub fn set_rd(&self, value: GPReg) {
        let prev_value = self.get_rd();
        let checked_value = GPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rd().set(next_value.into_mir());
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn rn(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn get_rn(&self) -> GPReg {
        GPReg::from_mir(self.rn().get())
    }
    #[doc = "set the value of operand rn at 1 to a value of type GPReg (checked by GPR64)"]
    pub fn set_rn(&self, value: GPReg) {
        let prev_value = self.get_rn();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rn().set(next_value.into_mir());
    }
    #[doc = "operand rm at index 2 of type ImmLoad32"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type ImmLoad32"]
    pub fn get_rm(&self) -> ImmLoad32 {
        ImmLoad32::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type ImmLoad32 (checked by ImmLoad32)"]
    pub fn set_rm(&self, value: ImmLoad32) {
        self.rm().set(value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct LoadStoreF64Base {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for LoadStoreF64Base {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::LdrF64Base | MirOP::StrF64Base)
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmLoad64::new_empty().into_mir()),
            ],
        }
    }
}
impl LoadStoreF64Base {
    pub fn new(opcode: MirOP, rd: FPR64, rn: GPR64, rm: ImmLoad64) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.into_mir()),
                Cell::new(rn.into_mir()),
                Cell::new(rm.into_mir()),
            ],
        }
    }
    #[doc = "operand rd at index 0 of type VFReg"]
    pub fn rd(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand rd at index 0 of type VFReg"]
    pub fn get_rd(&self) -> VFReg {
        VFReg::from_mir(self.rd().get())
    }
    #[doc = "set the value of operand rd at 0 to a value of type VFReg (checked by FPR64)"]
    pub fn set_rd(&self, value: VFReg) {
        let prev_value = self.get_rd();
        let checked_value = FPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rd().set(next_value.into_mir());
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn rn(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn get_rn(&self) -> GPReg {
        GPReg::from_mir(self.rn().get())
    }
    #[doc = "set the value of operand rn at 1 to a value of type GPReg (checked by GPR64)"]
    pub fn set_rn(&self, value: GPReg) {
        let prev_value = self.get_rn();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rn().set(next_value.into_mir());
    }
    #[doc = "operand rm at index 2 of type ImmLoad64"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type ImmLoad64"]
    pub fn get_rm(&self) -> ImmLoad64 {
        ImmLoad64::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type ImmLoad64 (checked by ImmLoad64)"]
    pub fn set_rm(&self, value: ImmLoad64) {
        self.rm().set(value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct LoadStoreF32Base {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for LoadStoreF32Base {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::LdrF32Base | MirOP::StrF32Base)
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmLoad32::new_empty().into_mir()),
            ],
        }
    }
}
impl LoadStoreF32Base {
    pub fn new(opcode: MirOP, rd: FPR32, rn: GPR64, rm: ImmLoad32) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.into_mir()),
                Cell::new(rn.into_mir()),
                Cell::new(rm.into_mir()),
            ],
        }
    }
    #[doc = "operand rd at index 0 of type VFReg"]
    pub fn rd(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand rd at index 0 of type VFReg"]
    pub fn get_rd(&self) -> VFReg {
        VFReg::from_mir(self.rd().get())
    }
    #[doc = "set the value of operand rd at 0 to a value of type VFReg (checked by FPR32)"]
    pub fn set_rd(&self, value: VFReg) {
        let prev_value = self.get_rd();
        let checked_value = FPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rd().set(next_value.into_mir());
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn rn(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn get_rn(&self) -> GPReg {
        GPReg::from_mir(self.rn().get())
    }
    #[doc = "set the value of operand rn at 1 to a value of type GPReg (checked by GPR64)"]
    pub fn set_rn(&self, value: GPReg) {
        let prev_value = self.get_rn();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rn().set(next_value.into_mir());
    }
    #[doc = "operand rm at index 2 of type ImmLoad32"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type ImmLoad32"]
    pub fn get_rm(&self) -> ImmLoad32 {
        ImmLoad32::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type ImmLoad32 (checked by ImmLoad32)"]
    pub fn set_rm(&self, value: ImmLoad32) {
        self.rm().set(value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct LoadStoreGr64Indexed {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
    addr_mode: AddrMode,
}
impl IMirSubInst for LoadStoreGr64Indexed {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..2usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[2usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(
            opcode,
            MirOP::LdrGr64Indexed
                | MirOP::LdrBGr64Indexed
                | MirOP::LdrHGr64Indexed
                | MirOP::LdrSBGr64Indexed
                | MirOP::LdrSHGr64Indexed
                | MirOP::StrGr64Indexed
                | MirOP::StrBGr64Indexed
                | MirOP::StrHGr64Indexed
        )
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmLoad64::new_empty().into_mir()),
            ],
            addr_mode: AddrMode::PostIndex,
        }
    }
}
impl LoadStoreGr64Indexed {
    pub fn new(opcode: MirOP, rd: GPR64, rn: GPR64, rm: ImmLoad64, addr_mode: AddrMode) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.into_mir()),
                Cell::new(rn.into_mir()),
                Cell::new(rm.into_mir()),
            ],
            addr_mode,
        }
    }
    #[doc = "operand rd at index 0 of type GPReg"]
    pub fn rd(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand rd at index 0 of type GPReg"]
    pub fn get_rd(&self) -> GPReg {
        GPReg::from_mir(self.rd().get())
    }
    #[doc = "set the value of operand rd at 0 to a value of type GPReg (checked by GPR64)"]
    pub fn set_rd(&self, value: GPReg) {
        let prev_value = self.get_rd();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rd().set(next_value.into_mir());
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn rn(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn get_rn(&self) -> GPReg {
        GPReg::from_mir(self.rn().get())
    }
    #[doc = "set the value of operand rn at 1 to a value of type GPReg (checked by GPR64)"]
    pub fn set_rn(&self, value: GPReg) {
        let prev_value = self.get_rn();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rn().set(next_value.into_mir());
    }
    #[doc = "operand rm at index 2 of type ImmLoad64"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type ImmLoad64"]
    pub fn get_rm(&self) -> ImmLoad64 {
        ImmLoad64::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type ImmLoad64 (checked by ImmLoad64)"]
    pub fn set_rm(&self, value: ImmLoad64) {
        self.rm().set(value.into_mir());
    }
    pub fn get_addr_mode(&self) -> AddrMode {
        self.addr_mode.clone()
    }
    pub fn set_addr_mode(&mut self, value: AddrMode) {
        self.addr_mode = value;
    }
}
#[derive(Debug, Clone)]
pub struct LoadStoreGr32Indexed {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
    addr_mode: AddrMode,
}
impl IMirSubInst for LoadStoreGr32Indexed {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..2usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[2usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(
            opcode,
            MirOP::LdrGr32Indexed
                | MirOP::LdrBGr32Indexed
                | MirOP::LdrHGr32Indexed
                | MirOP::LdrSBGr32Indexed
                | MirOP::LdrSHGr32Indexed
                | MirOP::StrGr32Indexed
                | MirOP::StrBGr32Indexed
                | MirOP::StrHGr32Indexed
        )
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmLoad32::new_empty().into_mir()),
            ],
            addr_mode: AddrMode::PostIndex,
        }
    }
}
impl LoadStoreGr32Indexed {
    pub fn new(opcode: MirOP, rd: GPR32, rn: GPR64, rm: ImmLoad32, addr_mode: AddrMode) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.into_mir()),
                Cell::new(rn.into_mir()),
                Cell::new(rm.into_mir()),
            ],
            addr_mode,
        }
    }
    #[doc = "operand rd at index 0 of type GPReg"]
    pub fn rd(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand rd at index 0 of type GPReg"]
    pub fn get_rd(&self) -> GPReg {
        GPReg::from_mir(self.rd().get())
    }
    #[doc = "set the value of operand rd at 0 to a value of type GPReg (checked by GPR32)"]
    pub fn set_rd(&self, value: GPReg) {
        let prev_value = self.get_rd();
        let checked_value = GPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rd().set(next_value.into_mir());
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn rn(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn get_rn(&self) -> GPReg {
        GPReg::from_mir(self.rn().get())
    }
    #[doc = "set the value of operand rn at 1 to a value of type GPReg (checked by GPR64)"]
    pub fn set_rn(&self, value: GPReg) {
        let prev_value = self.get_rn();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rn().set(next_value.into_mir());
    }
    #[doc = "operand rm at index 2 of type ImmLoad32"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type ImmLoad32"]
    pub fn get_rm(&self) -> ImmLoad32 {
        ImmLoad32::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type ImmLoad32 (checked by ImmLoad32)"]
    pub fn set_rm(&self, value: ImmLoad32) {
        self.rm().set(value.into_mir());
    }
    pub fn get_addr_mode(&self) -> AddrMode {
        self.addr_mode.clone()
    }
    pub fn set_addr_mode(&mut self, value: AddrMode) {
        self.addr_mode = value;
    }
}
#[derive(Debug, Clone)]
pub struct LoadStoreF64Indexed {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
    addr_mode: AddrMode,
}
impl IMirSubInst for LoadStoreF64Indexed {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..2usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[2usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::LdrF64Indexed | MirOP::StrF64Indexed)
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmLoad64::new_empty().into_mir()),
            ],
            addr_mode: AddrMode::PostIndex,
        }
    }
}
impl LoadStoreF64Indexed {
    pub fn new(opcode: MirOP, rd: FPR64, rn: GPR64, rm: ImmLoad64, addr_mode: AddrMode) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.into_mir()),
                Cell::new(rn.into_mir()),
                Cell::new(rm.into_mir()),
            ],
            addr_mode,
        }
    }
    #[doc = "operand rd at index 0 of type VFReg"]
    pub fn rd(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand rd at index 0 of type VFReg"]
    pub fn get_rd(&self) -> VFReg {
        VFReg::from_mir(self.rd().get())
    }
    #[doc = "set the value of operand rd at 0 to a value of type VFReg (checked by FPR64)"]
    pub fn set_rd(&self, value: VFReg) {
        let prev_value = self.get_rd();
        let checked_value = FPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rd().set(next_value.into_mir());
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn rn(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn get_rn(&self) -> GPReg {
        GPReg::from_mir(self.rn().get())
    }
    #[doc = "set the value of operand rn at 1 to a value of type GPReg (checked by GPR64)"]
    pub fn set_rn(&self, value: GPReg) {
        let prev_value = self.get_rn();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rn().set(next_value.into_mir());
    }
    #[doc = "operand rm at index 2 of type ImmLoad64"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type ImmLoad64"]
    pub fn get_rm(&self) -> ImmLoad64 {
        ImmLoad64::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type ImmLoad64 (checked by ImmLoad64)"]
    pub fn set_rm(&self, value: ImmLoad64) {
        self.rm().set(value.into_mir());
    }
    pub fn get_addr_mode(&self) -> AddrMode {
        self.addr_mode.clone()
    }
    pub fn set_addr_mode(&mut self, value: AddrMode) {
        self.addr_mode = value;
    }
}
#[derive(Debug, Clone)]
pub struct LoadStoreF32Indexed {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
    addr_mode: AddrMode,
}
impl IMirSubInst for LoadStoreF32Indexed {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..2usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[2usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::LdrF32Indexed | MirOP::StrF32Indexed)
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmLoad32::new_empty().into_mir()),
            ],
            addr_mode: AddrMode::PostIndex,
        }
    }
}
impl LoadStoreF32Indexed {
    pub fn new(opcode: MirOP, rd: FPR32, rn: GPR64, rm: ImmLoad32, addr_mode: AddrMode) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.into_mir()),
                Cell::new(rn.into_mir()),
                Cell::new(rm.into_mir()),
            ],
            addr_mode,
        }
    }
    #[doc = "operand rd at index 0 of type VFReg"]
    pub fn rd(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand rd at index 0 of type VFReg"]
    pub fn get_rd(&self) -> VFReg {
        VFReg::from_mir(self.rd().get())
    }
    #[doc = "set the value of operand rd at 0 to a value of type VFReg (checked by FPR32)"]
    pub fn set_rd(&self, value: VFReg) {
        let prev_value = self.get_rd();
        let checked_value = FPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rd().set(next_value.into_mir());
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn rn(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand rn at index 1 of type GPReg"]
    pub fn get_rn(&self) -> GPReg {
        GPReg::from_mir(self.rn().get())
    }
    #[doc = "set the value of operand rn at 1 to a value of type GPReg (checked by GPR64)"]
    pub fn set_rn(&self, value: GPReg) {
        let prev_value = self.get_rn();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rn().set(next_value.into_mir());
    }
    #[doc = "operand rm at index 2 of type ImmLoad32"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type ImmLoad32"]
    pub fn get_rm(&self) -> ImmLoad32 {
        ImmLoad32::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type ImmLoad32 (checked by ImmLoad32)"]
    pub fn set_rm(&self, value: ImmLoad32) {
        self.rm().set(value.into_mir());
    }
    pub fn get_addr_mode(&self) -> AddrMode {
        self.addr_mode.clone()
    }
    pub fn set_addr_mode(&mut self, value: AddrMode) {
        self.addr_mode = value;
    }
}
#[derive(Debug, Clone)]
pub struct LoadStoreGr64Literal {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for LoadStoreGr64Literal {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..2usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(
            opcode,
            MirOP::LdrGr64Literal
                | MirOP::LdrBGr64Literal
                | MirOP::LdrHGr64Literal
                | MirOP::LdrSBGr64Literal
                | MirOP::LdrSHGr64Literal
                | MirOP::StrGr64Literal
                | MirOP::StrBGr64Literal
                | MirOP::StrHGr64Literal
        )
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(MirSymbolOp::new_empty().into_mir()),
            ],
        }
    }
}
impl LoadStoreGr64Literal {
    pub fn new(opcode: MirOP, rd: GPR64, from: MirSymbolOp) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(rd.into_mir()), Cell::new(from.into_mir())],
        }
    }
    #[doc = "operand rd at index 0 of type GPReg"]
    pub fn rd(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand rd at index 0 of type GPReg"]
    pub fn get_rd(&self) -> GPReg {
        GPReg::from_mir(self.rd().get())
    }
    #[doc = "set the value of operand rd at 0 to a value of type GPReg (checked by GPR64)"]
    pub fn set_rd(&self, value: GPReg) {
        let prev_value = self.get_rd();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rd().set(next_value.into_mir());
    }
    #[doc = "operand from at index 1 of type MirSymbolOp"]
    pub fn from(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand from at index 1 of type MirSymbolOp"]
    pub fn get_from(&self) -> MirSymbolOp {
        MirSymbolOp::from_mir(self.from().get())
    }
    #[doc = "set the value of operand from at 1 to a value of type MirSymbolOp (checked by MirSymbolOp)"]
    pub fn set_from(&self, value: MirSymbolOp) {
        self.from().set(value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct LoadStoreGr32Literal {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for LoadStoreGr32Literal {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..2usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(
            opcode,
            MirOP::LdrGr32Literal
                | MirOP::LdrBGr32Literal
                | MirOP::LdrHGr32Literal
                | MirOP::LdrSBGr32Literal
                | MirOP::LdrSHGr32Literal
                | MirOP::StrGr32Literal
                | MirOP::StrBGr32Literal
                | MirOP::StrHGr32Literal
        )
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(MirSymbolOp::new_empty().into_mir()),
            ],
        }
    }
}
impl LoadStoreGr32Literal {
    pub fn new(opcode: MirOP, rd: GPR32, from: MirSymbolOp) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(rd.into_mir()), Cell::new(from.into_mir())],
        }
    }
    #[doc = "operand rd at index 0 of type GPReg"]
    pub fn rd(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand rd at index 0 of type GPReg"]
    pub fn get_rd(&self) -> GPReg {
        GPReg::from_mir(self.rd().get())
    }
    #[doc = "set the value of operand rd at 0 to a value of type GPReg (checked by GPR32)"]
    pub fn set_rd(&self, value: GPReg) {
        let prev_value = self.get_rd();
        let checked_value = GPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rd().set(next_value.into_mir());
    }
    #[doc = "operand from at index 1 of type MirSymbolOp"]
    pub fn from(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand from at index 1 of type MirSymbolOp"]
    pub fn get_from(&self) -> MirSymbolOp {
        MirSymbolOp::from_mir(self.from().get())
    }
    #[doc = "set the value of operand from at 1 to a value of type MirSymbolOp (checked by MirSymbolOp)"]
    pub fn set_from(&self, value: MirSymbolOp) {
        self.from().set(value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct LoadStoreF64Literal {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for LoadStoreF64Literal {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..2usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::LdrF64Literal | MirOP::StrF64Literal)
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(MirSymbolOp::new_empty().into_mir()),
            ],
        }
    }
}
impl LoadStoreF64Literal {
    pub fn new(opcode: MirOP, rd: FPR64, from: MirSymbolOp) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(rd.into_mir()), Cell::new(from.into_mir())],
        }
    }
    #[doc = "operand rd at index 0 of type VFReg"]
    pub fn rd(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand rd at index 0 of type VFReg"]
    pub fn get_rd(&self) -> VFReg {
        VFReg::from_mir(self.rd().get())
    }
    #[doc = "set the value of operand rd at 0 to a value of type VFReg (checked by FPR64)"]
    pub fn set_rd(&self, value: VFReg) {
        let prev_value = self.get_rd();
        let checked_value = FPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rd().set(next_value.into_mir());
    }
    #[doc = "operand from at index 1 of type MirSymbolOp"]
    pub fn from(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand from at index 1 of type MirSymbolOp"]
    pub fn get_from(&self) -> MirSymbolOp {
        MirSymbolOp::from_mir(self.from().get())
    }
    #[doc = "set the value of operand from at 1 to a value of type MirSymbolOp (checked by MirSymbolOp)"]
    pub fn set_from(&self, value: MirSymbolOp) {
        self.from().set(value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct LoadStoreF32Literal {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for LoadStoreF32Literal {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..2usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::LdrF32Literal | MirOP::StrF32Literal)
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(MirSymbolOp::new_empty().into_mir()),
            ],
        }
    }
}
impl LoadStoreF32Literal {
    pub fn new(opcode: MirOP, rd: FPR32, from: MirSymbolOp) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(rd.into_mir()), Cell::new(from.into_mir())],
        }
    }
    #[doc = "operand rd at index 0 of type VFReg"]
    pub fn rd(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand rd at index 0 of type VFReg"]
    pub fn get_rd(&self) -> VFReg {
        VFReg::from_mir(self.rd().get())
    }
    #[doc = "set the value of operand rd at 0 to a value of type VFReg (checked by FPR32)"]
    pub fn set_rd(&self, value: VFReg) {
        let prev_value = self.get_rd();
        let checked_value = FPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rd().set(next_value.into_mir());
    }
    #[doc = "operand from at index 1 of type MirSymbolOp"]
    pub fn from(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand from at index 1 of type MirSymbolOp"]
    pub fn get_from(&self) -> MirSymbolOp {
        MirSymbolOp::from_mir(self.from().get())
    }
    #[doc = "set the value of operand from at 1 to a value of type MirSymbolOp (checked by MirSymbolOp)"]
    pub fn set_from(&self, value: MirSymbolOp) {
        self.from().set(value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct LoadConst64 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for LoadConst64 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..2usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::LoadConst64)
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(Imm64::new_empty().into_mir()),
            ],
        }
    }
}
impl LoadConst64 {
    pub fn new(opcode: MirOP, rd: GPR64, src: Imm64) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(rd.into_mir()), Cell::new(src.into_mir())],
        }
    }
    #[doc = "operand rd at index 0 of type GPReg"]
    pub fn rd(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand rd at index 0 of type GPReg"]
    pub fn get_rd(&self) -> GPReg {
        GPReg::from_mir(self.rd().get())
    }
    #[doc = "set the value of operand rd at 0 to a value of type GPReg (checked by GPR64)"]
    pub fn set_rd(&self, value: GPReg) {
        let prev_value = self.get_rd();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rd().set(next_value.into_mir());
    }
    #[doc = "operand src at index 1 of type Imm64"]
    pub fn src(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand src at index 1 of type Imm64"]
    pub fn get_src(&self) -> Imm64 {
        Imm64::from_mir(self.src().get())
    }
    #[doc = "set the value of operand src at 1 to a value of type Imm64 (checked by Imm64)"]
    pub fn set_src(&self, value: Imm64) {
        self.src().set(value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct LoadConstF64 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for LoadConstF64 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..2usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::LoadConstF64)
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(Imm64::new_empty().into_mir()),
            ],
        }
    }
}
impl LoadConstF64 {
    pub fn new(opcode: MirOP, rd: FPR64, src: Imm64) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(rd.into_mir()), Cell::new(src.into_mir())],
        }
    }
    #[doc = "operand rd at index 0 of type VFReg"]
    pub fn rd(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand rd at index 0 of type VFReg"]
    pub fn get_rd(&self) -> VFReg {
        VFReg::from_mir(self.rd().get())
    }
    #[doc = "set the value of operand rd at 0 to a value of type VFReg (checked by FPR64)"]
    pub fn set_rd(&self, value: VFReg) {
        let prev_value = self.get_rd();
        let checked_value = FPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rd().set(next_value.into_mir());
    }
    #[doc = "operand src at index 1 of type Imm64"]
    pub fn src(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand src at index 1 of type Imm64"]
    pub fn get_src(&self) -> Imm64 {
        Imm64::from_mir(self.src().get())
    }
    #[doc = "set the value of operand src at 1 to a value of type Imm64 (checked by Imm64)"]
    pub fn set_src(&self, value: Imm64) {
        self.src().set(value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct LoadConst64Symbol {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for LoadConst64Symbol {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..2usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::LoadConst64Symbol)
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(MirSymbolOp::new_empty().into_mir()),
            ],
        }
    }
}
impl LoadConst64Symbol {
    pub fn new(opcode: MirOP, rd: GPR64, src: MirSymbolOp) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(rd.into_mir()), Cell::new(src.into_mir())],
        }
    }
    #[doc = "operand rd at index 0 of type GPReg"]
    pub fn rd(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand rd at index 0 of type GPReg"]
    pub fn get_rd(&self) -> GPReg {
        GPReg::from_mir(self.rd().get())
    }
    #[doc = "set the value of operand rd at 0 to a value of type GPReg (checked by GPR64)"]
    pub fn set_rd(&self, value: GPReg) {
        let prev_value = self.get_rd();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rd().set(next_value.into_mir());
    }
    #[doc = "operand src at index 1 of type MirSymbolOp"]
    pub fn src(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand src at index 1 of type MirSymbolOp"]
    pub fn get_src(&self) -> MirSymbolOp {
        MirSymbolOp::from_mir(self.src().get())
    }
    #[doc = "set the value of operand src at 1 to a value of type MirSymbolOp (checked by MirSymbolOp)"]
    pub fn set_src(&self, value: MirSymbolOp) {
        self.src().set(value.into_mir());
    }
}
#[derive(Debug, Clone)]
pub struct CondBr {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
    cond: MirCondFlag,
}
impl IMirSubInst for CondBr {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..0usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[0usize..2usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::BCond | MirOP::BCCond)
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(MirBlockRef::new_empty().into_mir()),
                Cell::new(PState::new_empty().into_mir()),
            ],
            cond: MirCondFlag::AL,
        }
    }
}
impl CondBr {
    pub fn new(opcode: MirOP, label: MirBlockRef, csr: PState, cond: MirCondFlag) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(label.into_mir()), Cell::new(csr.into_mir())],
            cond,
        }
    }
    #[doc = "operand label at index 0 of type MirBlockRef"]
    pub fn label(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand label at index 0 of type MirBlockRef"]
    pub fn get_label(&self) -> MirBlockRef {
        MirBlockRef::from_mir(self.label().get())
    }
    #[doc = "set the value of operand label at 0 to a value of type MirBlockRef (checked by MirBlockRef)"]
    pub fn set_label(&self, value: MirBlockRef) {
        self.label().set(value.into_mir());
    }
    #[doc = "operand csr at index 1 of type PState"]
    pub fn csr(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand csr at index 1 of type PState"]
    pub fn get_csr(&self) -> PState {
        PState::from_mir(self.csr().get())
    }
    #[doc = "set the value of operand csr at 1 to a value of type PState (checked by PState)"]
    pub fn set_csr(&self, value: PState) {
        let prev_value = self.get_csr();
        let checked_value = PState::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.csr().set(next_value.into_mir());
    }
    pub fn get_cond(&self) -> MirCondFlag {
        self.cond.clone()
    }
    pub fn set_cond(&mut self, value: MirCondFlag) {
        self.cond = value;
    }
}
#[derive(Debug, Clone)]
pub struct RegCondBr {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for RegCondBr {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..0usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[0usize..2usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::CBZ | MirOP::CBNZ | MirOP::TBZ | MirOP::TBNZ)
    }
    fn new_empty(opcode: MirOP) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(MirBlockRef::new_empty().into_mir()),
            ],
        }
    }
}
impl RegCondBr {
    pub fn new(opcode: MirOP, cond: GPR64, target: MirBlockRef) -> Self {
        Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(cond.into_mir()), Cell::new(target.into_mir())],
        }
    }
    #[doc = "operand cond at index 0 of type GPReg"]
    pub fn cond(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand cond at index 0 of type GPReg"]
    pub fn get_cond(&self) -> GPReg {
        GPReg::from_mir(self.cond().get())
    }
    #[doc = "set the value of operand cond at 0 to a value of type GPReg (checked by GPR64)"]
    pub fn set_cond(&self, value: GPReg) {
        let prev_value = self.get_cond();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.cond().set(next_value.into_mir());
    }
    #[doc = "operand target at index 1 of type MirBlockRef"]
    pub fn target(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand target at index 1 of type MirBlockRef"]
    pub fn get_target(&self) -> MirBlockRef {
        MirBlockRef::from_mir(self.target().get())
    }
    #[doc = "set the value of operand target at 1 to a value of type MirBlockRef (checked by MirBlockRef)"]
    pub fn set_target(&self, value: MirBlockRef) {
        self.target().set(value.into_mir());
    }
}

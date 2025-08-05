#![doc = r" Remusys-MIR Instruction Definitions"]
#![doc = r" NOTE: This file is auto-generated from the RIG-DSL definitions."]
#[allow(unused_imports)]
use crate::mir::{
    inst::{addr::*, cond::*, *},
    module::{block::*, func::*, global::*, *},
    operand::{compound::*, imm::*, reg::*, subop::*, MirOperand},
};
#[derive(Clone)]
pub struct UncondBr {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 1usize],
}
impl IMirSubInst for UncondBr {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(UncondBr),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(MirBlockRef::new_empty().into_mir())],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::UncondBr(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::UncondBr(self)
    }
}
impl UncondBr {
    pub fn new(opcode: MirOP, target: MirBlockRef) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(UncondBr),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(target.into_mir())],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for UncondBr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(UncondBr))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("target=op[0]", &self.get_target())
            .finish()
    }
}
#[derive(Clone)]
pub struct BReg {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 1usize],
}
impl IMirSubInst for BReg {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..0usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[0usize..1usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::Br | MirOP::Ret)
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(BReg),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(GPReg::new_empty().into_mir())],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::BReg(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::BReg(self)
    }
}
impl BReg {
    pub fn new(opcode: MirOP, target: GPR64) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(BReg),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(target.clean_use_flags().into_mir())],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for BReg {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(BReg))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("target=op[0]", &self.get_target())
            .finish()
    }
}
#[derive(Clone)]
pub struct BLinkLabel {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for BLinkLabel {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(BLinkLabel),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(MirBlockRef::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::BLinkLabel(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::BLinkLabel(self)
    }
}
impl BLinkLabel {
    pub fn new(opcode: MirOP, ra: GPR64, target: MirBlockRef) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(BLinkLabel),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(ra.clean_use_flags().into_mir()), Cell::new(target.into_mir())],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for BLinkLabel {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(BLinkLabel))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("ra=op[0]", &self.get_ra())
            .field("target=op[1]", &self.get_target())
            .finish()
    }
}
#[derive(Clone)]
pub struct BLinkGlobal {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for BLinkGlobal {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..2usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::BLinkGlobal)
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(BLinkGlobal),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(MirGlobalRef::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::BLinkGlobal(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::BLinkGlobal(self)
    }
}
impl BLinkGlobal {
    pub fn new(opcode: MirOP, ra: GPR64, target: MirGlobalRef) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(BLinkGlobal),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(ra.clean_use_flags().into_mir()), Cell::new(target.into_mir())],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
    #[doc = "operand target at index 1 of type MirGlobalRef"]
    pub fn target(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand target at index 1 of type MirGlobalRef"]
    pub fn get_target(&self) -> MirGlobalRef {
        MirGlobalRef::from_mir(self.target().get())
    }
    #[doc = "set the value of operand target at 1 to a value of type MirGlobalRef (checked by MirGlobalRef)"]
    pub fn set_target(&self, value: MirGlobalRef) {
        self.target().set(value.into_mir());
    }
}
impl std::fmt::Debug for BLinkGlobal {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(BLinkGlobal))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("ra=op[0]", &self.get_ra())
            .field("target=op[1]", &self.get_target())
            .finish()
    }
}
#[derive(Clone)]
pub struct BLinkReg {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for BLinkReg {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(BLinkReg),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::BLinkReg(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::BLinkReg(self)
    }
}
impl BLinkReg {
    pub fn new(opcode: MirOP, ra: GPR64, target: GPR64) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(BLinkReg),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(ra.clean_use_flags().into_mir()),
                Cell::new(target.clean_use_flags().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for BLinkReg {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(BLinkReg))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("ra=op[0]", &self.get_ra())
            .field("target=op[1]", &self.get_target())
            .finish()
    }
}
#[derive(Clone)]
pub struct TBZ64 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for TBZ64 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..0usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[0usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::TBZ64 | MirOP::TBNZ64)
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(TBZ64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(Imm32::new_empty().into_mir()),
                Cell::new(MirBlockRef::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::TBZ64(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::TBZ64(self)
    }
}
impl TBZ64 {
    pub fn new(opcode: MirOP, cond: GPR64, bits: Imm32, target: MirBlockRef) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(TBZ64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(cond.clean_use_flags().into_mir()),
                Cell::new(bits.into_mir()),
                Cell::new(target.into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
    #[doc = "operand bits at index 1 of type Imm32"]
    pub fn bits(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand bits at index 1 of type Imm32"]
    pub fn get_bits(&self) -> Imm32 {
        Imm32::from_mir(self.bits().get())
    }
    #[doc = "set the value of operand bits at 1 to a value of type Imm32 (checked by Imm32)"]
    pub fn set_bits(&self, value: Imm32) {
        self.bits().set(value.into_mir());
    }
    #[doc = "operand target at index 2 of type MirBlockRef"]
    pub fn target(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand target at index 2 of type MirBlockRef"]
    pub fn get_target(&self) -> MirBlockRef {
        MirBlockRef::from_mir(self.target().get())
    }
    #[doc = "set the value of operand target at 2 to a value of type MirBlockRef (checked by MirBlockRef)"]
    pub fn set_target(&self, value: MirBlockRef) {
        self.target().set(value.into_mir());
    }
}
impl std::fmt::Debug for TBZ64 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(TBZ64))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("cond=op[0]", &self.get_cond())
            .field("bits=op[1]", &self.get_bits())
            .field("target=op[2]", &self.get_target())
            .finish()
    }
}
#[derive(Clone)]
pub struct TBZ32 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for TBZ32 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..0usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[0usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::TBZ32 | MirOP::TBNZ32)
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(TBZ32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(Imm32::new_empty().into_mir()),
                Cell::new(MirBlockRef::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::TBZ32(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::TBZ32(self)
    }
}
impl TBZ32 {
    pub fn new(opcode: MirOP, cond: GPR32, bits: Imm32, target: MirBlockRef) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(TBZ32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(cond.clean_use_flags().into_mir()),
                Cell::new(bits.into_mir()),
                Cell::new(target.into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    #[doc = "operand cond at index 0 of type GPReg"]
    pub fn cond(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand cond at index 0 of type GPReg"]
    pub fn get_cond(&self) -> GPReg {
        GPReg::from_mir(self.cond().get())
    }
    #[doc = "set the value of operand cond at 0 to a value of type GPReg (checked by GPR32)"]
    pub fn set_cond(&self, value: GPReg) {
        let prev_value = self.get_cond();
        let checked_value = GPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.cond().set(next_value.into_mir());
    }
    #[doc = "operand bits at index 1 of type Imm32"]
    pub fn bits(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand bits at index 1 of type Imm32"]
    pub fn get_bits(&self) -> Imm32 {
        Imm32::from_mir(self.bits().get())
    }
    #[doc = "set the value of operand bits at 1 to a value of type Imm32 (checked by Imm32)"]
    pub fn set_bits(&self, value: Imm32) {
        self.bits().set(value.into_mir());
    }
    #[doc = "operand target at index 2 of type MirBlockRef"]
    pub fn target(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand target at index 2 of type MirBlockRef"]
    pub fn get_target(&self) -> MirBlockRef {
        MirBlockRef::from_mir(self.target().get())
    }
    #[doc = "set the value of operand target at 2 to a value of type MirBlockRef (checked by MirBlockRef)"]
    pub fn set_target(&self, value: MirBlockRef) {
        self.target().set(value.into_mir());
    }
}
impl std::fmt::Debug for TBZ32 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(TBZ32))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("cond=op[0]", &self.get_cond())
            .field("bits=op[1]", &self.get_bits())
            .field("target=op[2]", &self.get_target())
            .finish()
    }
}
#[derive(Clone)]
pub struct ICmp64R {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
    rm_op: Option<RegOP>,
}
impl IMirSubInst for ICmp64R {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(ICmp64R),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(PState::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
            ],
            rm_op: None,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::ICmp64R(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::ICmp64R(self)
    }
}
impl ICmp64R {
    pub fn new(opcode: MirOP, csr: PState, rn: GPR64, rhs: GPR64, rm_op: Option<RegOP>) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(ICmp64R),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(csr.into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rhs.clean_use_flags().into_mir()),
            ],
            rm_op,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for ICmp64R {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(ICmp64R))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("csr=op[0]", &self.get_csr())
            .field("rn=op[1]", &self.get_rn())
            .field("rhs=op[2]", &self.get_rhs())
            .field("rm_op", &self.rm_op)
            .finish()
    }
}
#[derive(Clone)]
pub struct ICmp32R {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
    rm_op: Option<RegOP>,
}
impl IMirSubInst for ICmp32R {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(ICmp32R),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(PState::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
            ],
            rm_op: None,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::ICmp32R(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::ICmp32R(self)
    }
}
impl ICmp32R {
    pub fn new(opcode: MirOP, csr: PState, rn: GPR32, rhs: GPR32, rm_op: Option<RegOP>) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(ICmp32R),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(csr.into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rhs.clean_use_flags().into_mir()),
            ],
            rm_op,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for ICmp32R {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(ICmp32R))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("csr=op[0]", &self.get_csr())
            .field("rn=op[1]", &self.get_rn())
            .field("rhs=op[2]", &self.get_rhs())
            .field("rm_op", &self.rm_op)
            .finish()
    }
}
#[derive(Clone)]
pub struct ICmp64I {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for ICmp64I {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(ICmp64I),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(PState::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmCalc::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::ICmp64I(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::ICmp64I(self)
    }
}
impl ICmp64I {
    pub fn new(opcode: MirOP, csr: PState, rn: GPR64, rhs: ImmCalc) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(ICmp64I),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(csr.into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rhs.into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for ICmp64I {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(ICmp64I))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("csr=op[0]", &self.get_csr())
            .field("rn=op[1]", &self.get_rn())
            .field("rhs=op[2]", &self.get_rhs())
            .finish()
    }
}
#[derive(Clone)]
pub struct ICmp32I {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for ICmp32I {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(ICmp32I),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(PState::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmCalc::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::ICmp32I(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::ICmp32I(self)
    }
}
impl ICmp32I {
    pub fn new(opcode: MirOP, csr: PState, rn: GPR32, rhs: ImmCalc) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(ICmp32I),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(csr.into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rhs.into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for ICmp32I {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(ICmp32I))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("csr=op[0]", &self.get_csr())
            .field("rn=op[1]", &self.get_rn())
            .field("rhs=op[2]", &self.get_rhs())
            .finish()
    }
}
#[derive(Clone)]
pub struct FCmp32 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for FCmp32 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(FCmp32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(PState::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::FCmp32(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::FCmp32(self)
    }
}
impl FCmp32 {
    pub fn new(opcode: MirOP, csr: PState, rn: FPR32, rhs: FPR32) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(FCmp32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(csr.into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rhs.clean_use_flags().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for FCmp32 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(FCmp32))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("csr=op[0]", &self.get_csr())
            .field("rn=op[1]", &self.get_rn())
            .field("rhs=op[2]", &self.get_rhs())
            .finish()
    }
}
#[derive(Clone)]
pub struct FCmp64 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for FCmp64 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(FCmp64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(PState::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::FCmp64(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::FCmp64(self)
    }
}
impl FCmp64 {
    pub fn new(opcode: MirOP, csr: PState, rn: FPR64, rhs: FPR64) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(FCmp64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(csr.into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rhs.clean_use_flags().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for FCmp64 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(FCmp64))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("csr=op[0]", &self.get_csr())
            .field("rn=op[1]", &self.get_rn())
            .field("rhs=op[2]", &self.get_rhs())
            .finish()
    }
}
#[derive(Clone)]
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
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(ICCmp64R),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(PState::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
            ],
            cond: MirCondFlag::AL,
            nzcv: NZCV::empty(),
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::ICCmp64R(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::ICCmp64R(self)
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(ICCmp64R),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(csr.into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rhs.clean_use_flags().into_mir()),
            ],
            cond,
            nzcv,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for ICCmp64R {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(ICCmp64R))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("csr=op[0]", &self.get_csr())
            .field("rn=op[1]", &self.get_rn())
            .field("rhs=op[2]", &self.get_rhs())
            .field("cond", &self.cond)
            .field("nzcv", &self.nzcv)
            .finish()
    }
}
#[derive(Clone)]
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
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(ICCmp32R),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(PState::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
            ],
            cond: MirCondFlag::AL,
            nzcv: NZCV::empty(),
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::ICCmp32R(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::ICCmp32R(self)
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(ICCmp32R),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(csr.into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rhs.clean_use_flags().into_mir()),
            ],
            cond,
            nzcv,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for ICCmp32R {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(ICCmp32R))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("csr=op[0]", &self.get_csr())
            .field("rn=op[1]", &self.get_rn())
            .field("rhs=op[2]", &self.get_rhs())
            .field("cond", &self.cond)
            .field("nzcv", &self.nzcv)
            .finish()
    }
}
#[derive(Clone)]
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
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(ICCmp64I),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(PState::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmCCmp::new_empty().into_mir()),
            ],
            cond: MirCondFlag::AL,
            nzcv: NZCV::empty(),
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::ICCmp64I(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::ICCmp64I(self)
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(ICCmp64I),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(csr.into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rhs.into_mir()),
            ],
            cond,
            nzcv,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for ICCmp64I {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(ICCmp64I))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("csr=op[0]", &self.get_csr())
            .field("rn=op[1]", &self.get_rn())
            .field("rhs=op[2]", &self.get_rhs())
            .field("cond", &self.cond)
            .field("nzcv", &self.nzcv)
            .finish()
    }
}
#[derive(Clone)]
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
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(ICCmp32I),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(PState::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmCCmp::new_empty().into_mir()),
            ],
            cond: MirCondFlag::AL,
            nzcv: NZCV::empty(),
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::ICCmp32I(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::ICCmp32I(self)
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(ICCmp32I),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(csr.into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rhs.into_mir()),
            ],
            cond,
            nzcv,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for ICCmp32I {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(ICCmp32I))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("csr=op[0]", &self.get_csr())
            .field("rn=op[1]", &self.get_rn())
            .field("rhs=op[2]", &self.get_rhs())
            .field("cond", &self.cond)
            .field("nzcv", &self.nzcv)
            .finish()
    }
}
#[derive(Clone)]
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
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(FCCmp32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(PState::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
            ],
            cond: MirCondFlag::AL,
            nzcv: NZCV::empty(),
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::FCCmp32(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::FCCmp32(self)
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(FCCmp32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(csr.into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rhs.clean_use_flags().into_mir()),
            ],
            cond,
            nzcv,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for FCCmp32 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(FCCmp32))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("csr=op[0]", &self.get_csr())
            .field("rn=op[1]", &self.get_rn())
            .field("rhs=op[2]", &self.get_rhs())
            .field("cond", &self.cond)
            .field("nzcv", &self.nzcv)
            .finish()
    }
}
#[derive(Clone)]
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
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(FCCmp64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(PState::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
            ],
            cond: MirCondFlag::AL,
            nzcv: NZCV::empty(),
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::FCCmp64(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::FCCmp64(self)
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(FCCmp64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(csr.into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rhs.clean_use_flags().into_mir()),
            ],
            cond,
            nzcv,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for FCCmp64 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(FCCmp64))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("csr=op[0]", &self.get_csr())
            .field("rn=op[1]", &self.get_rn())
            .field("rhs=op[2]", &self.get_rhs())
            .field("cond", &self.cond)
            .field("nzcv", &self.nzcv)
            .finish()
    }
}
#[derive(Clone)]
pub struct Bin64R {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
    rm_op: Option<RegOP>,
}
impl IMirSubInst for Bin64R {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(Bin64R),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
            ],
            rm_op: None,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::Bin64R(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::Bin64R(self)
    }
}
impl Bin64R {
    pub fn new(opcode: MirOP, rd: GPR64, rn: GPR64, rm: GPR64, rm_op: Option<RegOP>) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(Bin64R),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rm.clean_use_flags().into_mir()),
            ],
            rm_op,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for Bin64R {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(Bin64R))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("rn=op[1]", &self.get_rn())
            .field("rm=op[2]", &self.get_rm())
            .field("rm_op", &self.rm_op)
            .finish()
    }
}
#[derive(Clone)]
pub struct Bin32R {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
    rm_op: Option<RegOP>,
}
impl IMirSubInst for Bin32R {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(Bin32R),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
            ],
            rm_op: None,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::Bin32R(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::Bin32R(self)
    }
}
impl Bin32R {
    pub fn new(opcode: MirOP, rd: GPR32, rn: GPR32, rm: GPR32, rm_op: Option<RegOP>) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(Bin32R),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rm.clean_use_flags().into_mir()),
            ],
            rm_op,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for Bin32R {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(Bin32R))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("rn=op[1]", &self.get_rn())
            .field("rm=op[2]", &self.get_rm())
            .field("rm_op", &self.rm_op)
            .finish()
    }
}
#[derive(Clone)]
pub struct MulL {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for MulL {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(MulL),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::MulL(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::MulL(self)
    }
}
impl MulL {
    pub fn new(opcode: MirOP, rd: GPR64, rn: GPR32, rm: GPR32) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(MulL),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rm.clean_use_flags().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for MulL {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(MulL))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("rn=op[1]", &self.get_rn())
            .field("rm=op[2]", &self.get_rm())
            .finish()
    }
}
#[derive(Clone)]
pub struct Bin64RC {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for Bin64RC {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(Bin64RC),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmCalc::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::Bin64RC(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::Bin64RC(self)
    }
}
impl Bin64RC {
    pub fn new(opcode: MirOP, rd: GPR64, rn: GPR64, rm: ImmCalc) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(Bin64RC),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rm.into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for Bin64RC {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(Bin64RC))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("rn=op[1]", &self.get_rn())
            .field("rm=op[2]", &self.get_rm())
            .finish()
    }
}
#[derive(Clone)]
pub struct Bin32RC {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for Bin32RC {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(Bin32RC),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmCalc::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::Bin32RC(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::Bin32RC(self)
    }
}
impl Bin32RC {
    pub fn new(opcode: MirOP, rd: GPR32, rn: GPR32, rm: ImmCalc) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(Bin32RC),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rm.into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for Bin32RC {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(Bin32RC))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("rn=op[1]", &self.get_rn())
            .field("rm=op[2]", &self.get_rm())
            .finish()
    }
}
#[derive(Clone)]
pub struct Bin64RSym {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for Bin64RSym {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::Add64Sym | MirOP::Sub64Sym)
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(Bin64RSym),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(MirSymbolOp::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::Bin64RSym(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::Bin64RSym(self)
    }
}
impl Bin64RSym {
    pub fn new(opcode: MirOP, rd: GPR64, rn: GPR64, rm: MirSymbolOp) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(Bin64RSym),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rm.into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
    #[doc = "operand rm at index 2 of type MirSymbolOp"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type MirSymbolOp"]
    pub fn get_rm(&self) -> MirSymbolOp {
        MirSymbolOp::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type MirSymbolOp (checked by MirSymbolOp)"]
    pub fn set_rm(&self, value: MirSymbolOp) {
        self.rm().set(value.into_mir());
    }
}
impl std::fmt::Debug for Bin64RSym {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(Bin64RSym))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("rn=op[1]", &self.get_rn())
            .field("rm=op[2]", &self.get_rm())
            .finish()
    }
}
#[derive(Clone)]
pub struct Bin64RL {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for Bin64RL {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(Bin64RL),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmLogic::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::Bin64RL(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::Bin64RL(self)
    }
}
impl Bin64RL {
    pub fn new(opcode: MirOP, rd: GPR64, rn: GPR64, rm: ImmLogic) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(Bin64RL),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rm.into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for Bin64RL {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(Bin64RL))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("rn=op[1]", &self.get_rn())
            .field("rm=op[2]", &self.get_rm())
            .finish()
    }
}
#[derive(Clone)]
pub struct Bin32RL {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for Bin32RL {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(Bin32RL),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmLogic::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::Bin32RL(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::Bin32RL(self)
    }
}
impl Bin32RL {
    pub fn new(opcode: MirOP, rd: GPR32, rn: GPR32, rm: ImmLogic) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(Bin32RL),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rm.into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for Bin32RL {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(Bin32RL))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("rn=op[1]", &self.get_rn())
            .field("rm=op[2]", &self.get_rm())
            .finish()
    }
}
#[derive(Clone)]
pub struct Bin64RS {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for Bin64RS {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(Bin64RS),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmSMax::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::Bin64RS(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::Bin64RS(self)
    }
}
impl Bin64RS {
    pub fn new(opcode: MirOP, rd: GPR64, rn: GPR64, rm: ImmSMax) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(Bin64RS),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rm.into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for Bin64RS {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(Bin64RS))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("rn=op[1]", &self.get_rn())
            .field("rm=op[2]", &self.get_rm())
            .finish()
    }
}
#[derive(Clone)]
pub struct Bin64RU {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for Bin64RU {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(Bin64RU),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmUMax::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::Bin64RU(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::Bin64RU(self)
    }
}
impl Bin64RU {
    pub fn new(opcode: MirOP, rd: GPR64, rn: GPR64, rm: ImmUMax) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(Bin64RU),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rm.into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for Bin64RU {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(Bin64RU))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("rn=op[1]", &self.get_rn())
            .field("rm=op[2]", &self.get_rm())
            .finish()
    }
}
#[derive(Clone)]
pub struct Bin32RS {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for Bin32RS {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(Bin32RS),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmSMax::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::Bin32RS(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::Bin32RS(self)
    }
}
impl Bin32RS {
    pub fn new(opcode: MirOP, rd: GPR32, rn: GPR32, rm: ImmSMax) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(Bin32RS),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rm.into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for Bin32RS {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(Bin32RS))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("rn=op[1]", &self.get_rn())
            .field("rm=op[2]", &self.get_rm())
            .finish()
    }
}
#[derive(Clone)]
pub struct Bin32RU {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for Bin32RU {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(Bin32RU),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmUMax::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::Bin32RU(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::Bin32RU(self)
    }
}
impl Bin32RU {
    pub fn new(opcode: MirOP, rd: GPR32, rn: GPR32, rm: ImmUMax) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(Bin32RU),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rm.into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for Bin32RU {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(Bin32RU))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("rn=op[1]", &self.get_rn())
            .field("rm=op[2]", &self.get_rm())
            .finish()
    }
}
#[derive(Clone)]
pub struct Bin64RShift {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for Bin64RShift {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(Bin64RShift),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmShift::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::Bin64RShift(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::Bin64RShift(self)
    }
}
impl Bin64RShift {
    pub fn new(opcode: MirOP, rd: GPR64, rn: GPR64, rm: ImmShift) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(Bin64RShift),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rm.into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for Bin64RShift {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(Bin64RShift))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("rn=op[1]", &self.get_rn())
            .field("rm=op[2]", &self.get_rm())
            .finish()
    }
}
#[derive(Clone)]
pub struct Bin32RShift {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for Bin32RShift {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(Bin32RShift),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmShift::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::Bin32RShift(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::Bin32RShift(self)
    }
}
impl Bin32RShift {
    pub fn new(opcode: MirOP, rd: GPR32, rn: GPR32, rm: ImmShift) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(Bin32RShift),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rm.into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for Bin32RShift {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(Bin32RShift))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("rn=op[1]", &self.get_rn())
            .field("rm=op[2]", &self.get_rm())
            .finish()
    }
}
#[derive(Clone)]
pub struct BinF64R {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for BinF64R {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(BinF64R),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::BinF64R(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::BinF64R(self)
    }
}
impl BinF64R {
    pub fn new(opcode: MirOP, rd: FPR64, rn: FPR64, rm: FPR64) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(BinF64R),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rm.clean_use_flags().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for BinF64R {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(BinF64R))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("rn=op[1]", &self.get_rn())
            .field("rm=op[2]", &self.get_rm())
            .finish()
    }
}
#[derive(Clone)]
pub struct BinF32R {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for BinF32R {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(BinF32R),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::BinF32R(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::BinF32R(self)
    }
}
impl BinF32R {
    pub fn new(opcode: MirOP, rd: FPR32, rn: FPR32, rm: FPR32) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(BinF32R),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rm.clean_use_flags().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for BinF32R {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(BinF32R))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("rn=op[1]", &self.get_rn())
            .field("rm=op[2]", &self.get_rm())
            .finish()
    }
}
#[derive(Clone)]
pub struct MirCopy64 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for MirCopy64 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(MirCopy64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(MirOperand::None.into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::MirCopy64(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::MirCopy64(self)
    }
}
impl MirCopy64 {
    pub fn new(opcode: MirOP, dst: GPR64, src: MirOperand) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(MirCopy64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(dst.clean_use_flags().into_mir()), Cell::new(src.into_mir())],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for MirCopy64 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(MirCopy64))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("dst=op[0]", &self.get_dst())
            .field("src=op[1]", &self.get_src())
            .finish()
    }
}
#[derive(Clone)]
pub struct MirCopy32 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for MirCopy32 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(MirCopy32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(MirOperand::None.into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::MirCopy32(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::MirCopy32(self)
    }
}
impl MirCopy32 {
    pub fn new(opcode: MirOP, dst: GPR32, src: MirOperand) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(MirCopy32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(dst.clean_use_flags().into_mir()), Cell::new(src.into_mir())],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for MirCopy32 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(MirCopy32))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("dst=op[0]", &self.get_dst())
            .field("src=op[1]", &self.get_src())
            .finish()
    }
}
#[derive(Clone)]
pub struct MirFCopy64 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for MirFCopy64 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(MirFCopy64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(MirOperand::None.into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::MirFCopy64(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::MirFCopy64(self)
    }
}
impl MirFCopy64 {
    pub fn new(opcode: MirOP, dst: FPR64, src: MirOperand) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(MirFCopy64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(dst.clean_use_flags().into_mir()), Cell::new(src.into_mir())],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for MirFCopy64 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(MirFCopy64))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("dst=op[0]", &self.get_dst())
            .field("src=op[1]", &self.get_src())
            .finish()
    }
}
#[derive(Clone)]
pub struct MirFCopy32 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for MirFCopy32 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(MirFCopy32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(MirOperand::None.into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::MirFCopy32(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::MirFCopy32(self)
    }
}
impl MirFCopy32 {
    pub fn new(opcode: MirOP, dst: FPR32, src: MirOperand) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(MirFCopy32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(dst.clean_use_flags().into_mir()), Cell::new(src.into_mir())],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for MirFCopy32 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(MirFCopy32))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("dst=op[0]", &self.get_dst())
            .field("src=op[1]", &self.get_src())
            .finish()
    }
}
#[derive(Clone)]
pub struct MirPCopy {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for MirPCopy {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(MirPCopy),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(PState::new_empty().into_mir()),
                Cell::new(MirOperand::None.into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::MirPCopy(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::MirPCopy(self)
    }
}
impl MirPCopy {
    pub fn new(opcode: MirOP, dst: PState, src: MirOperand) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(MirPCopy),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(dst.into_mir()), Cell::new(src.into_mir())],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for MirPCopy {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(MirPCopy))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("dst=op[0]", &self.get_dst())
            .field("src=op[1]", &self.get_src())
            .finish()
    }
}
#[derive(Clone)]
pub struct Una64R {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
    dst_op: Option<RegOP>,
}
impl IMirSubInst for Una64R {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
                | MirOP::LoadStackPosGr64
                | MirOP::StoreStackPosGr64
        )
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(Una64R),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
            ],
            dst_op: None,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::Una64R(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::Una64R(self)
    }
}
impl Una64R {
    pub fn new(opcode: MirOP, dst: GPR64, src: GPR64, dst_op: Option<RegOP>) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(Una64R),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(dst.clean_use_flags().into_mir()),
                Cell::new(src.clean_use_flags().into_mir()),
            ],
            dst_op,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for Una64R {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(Una64R))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("dst=op[0]", &self.get_dst())
            .field("src=op[1]", &self.get_src())
            .field("dst_op", &self.dst_op)
            .finish()
    }
}
#[derive(Clone)]
pub struct Una32R {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
    dst_op: Option<RegOP>,
}
impl IMirSubInst for Una32R {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
                | MirOP::LoadStackPosGr32
                | MirOP::StoreStackPosGr32
        )
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(Una32R),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
            ],
            dst_op: None,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::Una32R(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::Una32R(self)
    }
}
impl Una32R {
    pub fn new(opcode: MirOP, dst: GPR32, src: GPR32, dst_op: Option<RegOP>) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(Una32R),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(dst.clean_use_flags().into_mir()),
                Cell::new(src.clean_use_flags().into_mir()),
            ],
            dst_op,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for Una32R {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(Una32R))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("dst=op[0]", &self.get_dst())
            .field("src=op[1]", &self.get_src())
            .field("dst_op", &self.dst_op)
            .finish()
    }
}
#[derive(Clone)]
pub struct ExtR {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for ExtR {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(ExtR),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::ExtR(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::ExtR(self)
    }
}
impl ExtR {
    pub fn new(opcode: MirOP, dst: GPR64, src: GPR32) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(ExtR),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(dst.clean_use_flags().into_mir()),
                Cell::new(src.clean_use_flags().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for ExtR {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(ExtR))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("dst=op[0]", &self.get_dst())
            .field("src=op[1]", &self.get_src())
            .finish()
    }
}
#[derive(Clone)]
pub struct Mov64I {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for Mov64I {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..2usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::Mov64I)
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(Mov64I),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmMov::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::Mov64I(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::Mov64I(self)
    }
}
impl Mov64I {
    pub fn new(opcode: MirOP, dst: GPR64, src: ImmMov) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(Mov64I),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(dst.clean_use_flags().into_mir()), Cell::new(src.into_mir())],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for Mov64I {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(Mov64I))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("dst=op[0]", &self.get_dst())
            .field("src=op[1]", &self.get_src())
            .finish()
    }
}
#[derive(Clone)]
pub struct MovZNK64 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for MovZNK64 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..2usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::MovZ64 | MirOP::MovN64 | MirOP::MovK64)
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(MovZNK64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmMovZNK::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::MovZNK64(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::MovZNK64(self)
    }
}
impl MovZNK64 {
    pub fn new(opcode: MirOP, dst: GPR64, src: ImmMovZNK) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(MovZNK64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(dst.clean_use_flags().into_mir()), Cell::new(src.into_mir())],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
    #[doc = "operand src at index 1 of type ImmMovZNK"]
    pub fn src(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand src at index 1 of type ImmMovZNK"]
    pub fn get_src(&self) -> ImmMovZNK {
        ImmMovZNK::from_mir(self.src().get())
    }
    #[doc = "set the value of operand src at 1 to a value of type ImmMovZNK (checked by ImmMovZNK)"]
    pub fn set_src(&self, value: ImmMovZNK) {
        self.src().set(value.into_mir());
    }
}
impl std::fmt::Debug for MovZNK64 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(MovZNK64))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("dst=op[0]", &self.get_dst())
            .field("src=op[1]", &self.get_src())
            .finish()
    }
}
#[derive(Clone)]
pub struct Mov32I {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for Mov32I {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..2usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::Mov32I)
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(Mov32I),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmMov::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::Mov32I(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::Mov32I(self)
    }
}
impl Mov32I {
    pub fn new(opcode: MirOP, dst: GPR32, src: ImmMov) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(Mov32I),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(dst.clean_use_flags().into_mir()), Cell::new(src.into_mir())],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for Mov32I {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(Mov32I))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("dst=op[0]", &self.get_dst())
            .field("src=op[1]", &self.get_src())
            .finish()
    }
}
#[derive(Clone)]
pub struct MovZNK32 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for MovZNK32 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..2usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::MovZ32 | MirOP::MovN32 | MirOP::MovK32)
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(MovZNK32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmMovZNK::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::MovZNK32(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::MovZNK32(self)
    }
}
impl MovZNK32 {
    pub fn new(opcode: MirOP, dst: GPR32, src: ImmMovZNK) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(MovZNK32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(dst.clean_use_flags().into_mir()), Cell::new(src.into_mir())],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
    #[doc = "operand src at index 1 of type ImmMovZNK"]
    pub fn src(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand src at index 1 of type ImmMovZNK"]
    pub fn get_src(&self) -> ImmMovZNK {
        ImmMovZNK::from_mir(self.src().get())
    }
    #[doc = "set the value of operand src at 1 to a value of type ImmMovZNK (checked by ImmMovZNK)"]
    pub fn set_src(&self, value: ImmMovZNK) {
        self.src().set(value.into_mir());
    }
}
impl std::fmt::Debug for MovZNK32 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(MovZNK32))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("dst=op[0]", &self.get_dst())
            .field("src=op[1]", &self.get_src())
            .finish()
    }
}
#[derive(Clone)]
pub struct Adr {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for Adr {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(Adr),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(MirSymbolOp::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::Adr(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::Adr(self)
    }
}
impl Adr {
    pub fn new(opcode: MirOP, dst: GPR64, src: MirSymbolOp) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(Adr),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(dst.clean_use_flags().into_mir()), Cell::new(src.into_mir())],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for Adr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(Adr))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("dst=op[0]", &self.get_dst())
            .field("src=op[1]", &self.get_src())
            .finish()
    }
}
#[derive(Clone)]
pub struct UnaFG64 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for UnaFG64 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..2usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::FMovFG64 | MirOP::SCvtF64 | MirOP::UCvtF64)
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(UnaFG64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::UnaFG64(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::UnaFG64(self)
    }
}
impl UnaFG64 {
    pub fn new(opcode: MirOP, dst: FPR64, src: GPR64) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(UnaFG64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(dst.clean_use_flags().into_mir()),
                Cell::new(src.clean_use_flags().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for UnaFG64 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(UnaFG64))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("dst=op[0]", &self.get_dst())
            .field("src=op[1]", &self.get_src())
            .finish()
    }
}
#[derive(Clone)]
pub struct UnaGF64 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for UnaGF64 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(UnaGF64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::UnaGF64(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::UnaGF64(self)
    }
}
impl UnaGF64 {
    pub fn new(opcode: MirOP, dst: GPR64, src: FPR64) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(UnaGF64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(dst.clean_use_flags().into_mir()),
                Cell::new(src.clean_use_flags().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for UnaGF64 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(UnaGF64))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("dst=op[0]", &self.get_dst())
            .field("src=op[1]", &self.get_src())
            .finish()
    }
}
#[derive(Clone)]
pub struct UnaF64G32 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for UnaF64G32 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..2usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::SCvtF64G32 | MirOP::UCvtF64G32)
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(UnaF64G32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::UnaF64G32(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::UnaF64G32(self)
    }
}
impl UnaF64G32 {
    pub fn new(opcode: MirOP, dst: FPR64, src: GPR32) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(UnaF64G32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(dst.clean_use_flags().into_mir()),
                Cell::new(src.clean_use_flags().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for UnaF64G32 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(UnaF64G32))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("dst=op[0]", &self.get_dst())
            .field("src=op[1]", &self.get_src())
            .finish()
    }
}
#[derive(Clone)]
pub struct UnaFG32 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for UnaFG32 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..2usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::FMovFG32 | MirOP::SCvtF32 | MirOP::UCvtF32)
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(UnaFG32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::UnaFG32(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::UnaFG32(self)
    }
}
impl UnaFG32 {
    pub fn new(opcode: MirOP, dst: FPR32, src: GPR32) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(UnaFG32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(dst.clean_use_flags().into_mir()),
                Cell::new(src.clean_use_flags().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for UnaFG32 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(UnaFG32))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("dst=op[0]", &self.get_dst())
            .field("src=op[1]", &self.get_src())
            .finish()
    }
}
#[derive(Clone)]
pub struct UnaF32G64 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for UnaF32G64 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..2usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::SCvtF32G64 | MirOP::UCvtF32G64)
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(UnaF32G64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::UnaF32G64(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::UnaF32G64(self)
    }
}
impl UnaF32G64 {
    pub fn new(opcode: MirOP, dst: FPR32, src: GPR64) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(UnaF32G64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(dst.clean_use_flags().into_mir()),
                Cell::new(src.clean_use_flags().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
    #[doc = "set the value of operand src at 1 to a value of type GPReg (checked by GPR64)"]
    pub fn set_src(&self, value: GPReg) {
        let prev_value = self.get_src();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.src().set(next_value.into_mir());
    }
}
impl std::fmt::Debug for UnaF32G64 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(UnaF32G64))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("dst=op[0]", &self.get_dst())
            .field("src=op[1]", &self.get_src())
            .finish()
    }
}
#[derive(Clone)]
pub struct UnaGF32 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for UnaGF32 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(UnaGF32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::UnaGF32(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::UnaGF32(self)
    }
}
impl UnaGF32 {
    pub fn new(opcode: MirOP, dst: GPR32, src: FPR32) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(UnaGF32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(dst.clean_use_flags().into_mir()),
                Cell::new(src.clean_use_flags().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for UnaGF32 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(UnaGF32))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("dst=op[0]", &self.get_dst())
            .field("src=op[1]", &self.get_src())
            .finish()
    }
}
#[derive(Clone)]
pub struct UnaG64F32 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for UnaG64F32 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(UnaG64F32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::UnaG64F32(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::UnaG64F32(self)
    }
}
impl UnaG64F32 {
    pub fn new(opcode: MirOP, dst: GPR64, src: FPR32) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(UnaG64F32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(dst.clean_use_flags().into_mir()),
                Cell::new(src.clean_use_flags().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for UnaG64F32 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(UnaG64F32))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("dst=op[0]", &self.get_dst())
            .field("src=op[1]", &self.get_src())
            .finish()
    }
}
#[derive(Clone)]
pub struct UnaG32F64 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for UnaG32F64 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(UnaG32F64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::UnaG32F64(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::UnaG32F64(self)
    }
}
impl UnaG32F64 {
    pub fn new(opcode: MirOP, dst: GPR32, src: FPR64) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(UnaG32F64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(dst.clean_use_flags().into_mir()),
                Cell::new(src.clean_use_flags().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for UnaG32F64 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(UnaG32F64))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("dst=op[0]", &self.get_dst())
            .field("src=op[1]", &self.get_src())
            .finish()
    }
}
#[derive(Clone)]
pub struct UnaF64 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for UnaF64 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
                | MirOP::LoadStackPosF64
                | MirOP::StoreStackPosF64
        )
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(UnaF64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::UnaF64(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::UnaF64(self)
    }
}
impl UnaF64 {
    pub fn new(opcode: MirOP, dst: FPR64, src: FPR64) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(UnaF64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(dst.clean_use_flags().into_mir()),
                Cell::new(src.clean_use_flags().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for UnaF64 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(UnaF64))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("dst=op[0]", &self.get_dst())
            .field("src=op[1]", &self.get_src())
            .finish()
    }
}
#[derive(Clone)]
pub struct UnaF32 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for UnaF32 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
                | MirOP::LoadStackPosF32
                | MirOP::StoreStackPosF32
        )
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(UnaF32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::UnaF32(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::UnaF32(self)
    }
}
impl UnaF32 {
    pub fn new(opcode: MirOP, dst: FPR32, src: FPR32) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(UnaF32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(dst.clean_use_flags().into_mir()),
                Cell::new(src.clean_use_flags().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for UnaF32 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(UnaF32))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("dst=op[0]", &self.get_dst())
            .field("src=op[1]", &self.get_src())
            .finish()
    }
}
#[derive(Clone)]
pub struct UnaryF32F64 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for UnaryF32F64 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(UnaryF32F64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::UnaryF32F64(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::UnaryF32F64(self)
    }
}
impl UnaryF32F64 {
    pub fn new(opcode: MirOP, dst: FPR32, src: FPR64) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(UnaryF32F64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(dst.clean_use_flags().into_mir()),
                Cell::new(src.clean_use_flags().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for UnaryF32F64 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(UnaryF32F64))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("dst=op[0]", &self.get_dst())
            .field("src=op[1]", &self.get_src())
            .finish()
    }
}
#[derive(Clone)]
pub struct UnaryF64F32 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for UnaryF64F32 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(UnaryF64F32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::UnaryF64F32(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::UnaryF64F32(self)
    }
}
impl UnaryF64F32 {
    pub fn new(opcode: MirOP, dst: FPR64, src: FPR32) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(UnaryF64F32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(dst.clean_use_flags().into_mir()),
                Cell::new(src.clean_use_flags().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for UnaryF64F32 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(UnaryF64F32))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("dst=op[0]", &self.get_dst())
            .field("src=op[1]", &self.get_src())
            .finish()
    }
}
#[derive(Clone)]
pub struct FMov64I {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for FMov64I {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(FMov64I),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(ImmFMov64::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::FMov64I(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::FMov64I(self)
    }
}
impl FMov64I {
    pub fn new(opcode: MirOP, dst: FPR64, src: ImmFMov64) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(FMov64I),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(dst.clean_use_flags().into_mir()), Cell::new(src.into_mir())],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for FMov64I {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(FMov64I))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("dst=op[0]", &self.get_dst())
            .field("src=op[1]", &self.get_src())
            .finish()
    }
}
#[derive(Clone)]
pub struct FMov32I {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for FMov32I {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(FMov32I),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(ImmFMov32::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::FMov32I(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::FMov32I(self)
    }
}
impl FMov32I {
    pub fn new(opcode: MirOP, dst: FPR32, src: ImmFMov32) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(FMov32I),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(dst.clean_use_flags().into_mir()), Cell::new(src.into_mir())],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for FMov32I {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(FMov32I))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("dst=op[0]", &self.get_dst())
            .field("src=op[1]", &self.get_src())
            .finish()
    }
}
#[derive(Clone)]
pub struct TenaryG64 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 4usize],
}
impl IMirSubInst for TenaryG64 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(TenaryG64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::TenaryG64(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::TenaryG64(self)
    }
}
impl TenaryG64 {
    pub fn new(opcode: MirOP, rd: GPR64, rn: GPR64, rm: GPR64, rs: GPR64) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(TenaryG64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rm.clean_use_flags().into_mir()),
                Cell::new(rs.clean_use_flags().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for TenaryG64 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(TenaryG64))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("rn=op[1]", &self.get_rn())
            .field("rm=op[2]", &self.get_rm())
            .field("rs=op[3]", &self.get_rs())
            .finish()
    }
}
#[derive(Clone)]
pub struct TenaryG64G32 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 4usize],
}
impl IMirSubInst for TenaryG64G32 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(TenaryG64G32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::TenaryG64G32(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::TenaryG64G32(self)
    }
}
impl TenaryG64G32 {
    pub fn new(opcode: MirOP, rd: GPR64, rn: GPR32, rm: GPR32, rs: GPR64) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(TenaryG64G32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rm.clean_use_flags().into_mir()),
                Cell::new(rs.clean_use_flags().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for TenaryG64G32 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(TenaryG64G32))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("rn=op[1]", &self.get_rn())
            .field("rm=op[2]", &self.get_rm())
            .field("rs=op[3]", &self.get_rs())
            .finish()
    }
}
#[derive(Clone)]
pub struct TenaryG32 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 4usize],
}
impl IMirSubInst for TenaryG32 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(TenaryG32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::TenaryG32(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::TenaryG32(self)
    }
}
impl TenaryG32 {
    pub fn new(opcode: MirOP, rd: GPR32, rn: GPR32, rm: GPR32, rs: GPR32) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(TenaryG32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rm.clean_use_flags().into_mir()),
                Cell::new(rs.clean_use_flags().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for TenaryG32 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(TenaryG32))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("rn=op[1]", &self.get_rn())
            .field("rm=op[2]", &self.get_rm())
            .field("rs=op[3]", &self.get_rs())
            .finish()
    }
}
#[derive(Clone)]
pub struct TenaryF64 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 4usize],
}
impl IMirSubInst for TenaryF64 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(TenaryF64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::TenaryF64(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::TenaryF64(self)
    }
}
impl TenaryF64 {
    pub fn new(opcode: MirOP, rd: FPR64, rn: FPR64, rm: FPR64, rs: FPR64) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(TenaryF64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rm.clean_use_flags().into_mir()),
                Cell::new(rs.clean_use_flags().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for TenaryF64 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(TenaryF64))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("rn=op[1]", &self.get_rn())
            .field("rm=op[2]", &self.get_rm())
            .field("rs=op[3]", &self.get_rs())
            .finish()
    }
}
#[derive(Clone)]
pub struct TenaryF32 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 4usize],
}
impl IMirSubInst for TenaryF32 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(TenaryF32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::TenaryF32(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::TenaryF32(self)
    }
}
impl TenaryF32 {
    pub fn new(opcode: MirOP, rd: FPR32, rn: FPR32, rm: FPR32, rs: FPR32) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(TenaryF32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rm.clean_use_flags().into_mir()),
                Cell::new(rs.clean_use_flags().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for TenaryF32 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(TenaryF32))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("rn=op[1]", &self.get_rn())
            .field("rm=op[2]", &self.get_rm())
            .field("rs=op[3]", &self.get_rs())
            .finish()
    }
}
#[derive(Clone)]
pub struct LoadGr64 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
    rm_op: Option<RegOP>,
}
impl IMirSubInst for LoadGr64 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
                | MirOP::LdrSW
        )
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(LoadGr64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
            ],
            rm_op: None,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::LoadGr64(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::LoadGr64(self)
    }
}
impl LoadGr64 {
    pub fn new(opcode: MirOP, rd: GPR64, rn: GPR64, rm: GPR64, rm_op: Option<RegOP>) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(LoadGr64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rm.clean_use_flags().into_mir()),
            ],
            rm_op,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for LoadGr64 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(LoadGr64))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("rn=op[1]", &self.get_rn())
            .field("rm=op[2]", &self.get_rm())
            .field("rm_op", &self.rm_op)
            .finish()
    }
}
#[derive(Clone)]
pub struct LoadG64G32 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
    rm_op: Option<RegOP>,
}
impl IMirSubInst for LoadG64G32 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::LdrSWOG32)
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(LoadG64G32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
            ],
            rm_op: None,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::LoadG64G32(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::LoadG64G32(self)
    }
}
impl LoadG64G32 {
    pub fn new(opcode: MirOP, rd: GPR64, rn: GPR64, rm: GPR32, rm_op: Option<RegOP>) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(LoadG64G32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rm.clean_use_flags().into_mir()),
            ],
            rm_op,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for LoadG64G32 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(LoadG64G32))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("rn=op[1]", &self.get_rn())
            .field("rm=op[2]", &self.get_rm())
            .field("rm_op", &self.rm_op)
            .finish()
    }
}
#[derive(Clone)]
pub struct LoadGr32 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
    rm_op: Option<RegOP>,
}
impl IMirSubInst for LoadGr32 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        )
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(LoadGr32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
            ],
            rm_op: None,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::LoadGr32(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::LoadGr32(self)
    }
}
impl LoadGr32 {
    pub fn new(opcode: MirOP, rd: GPR32, rn: GPR64, rm: GPR64, rm_op: Option<RegOP>) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(LoadGr32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rm.clean_use_flags().into_mir()),
            ],
            rm_op,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for LoadGr32 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(LoadGr32))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("rn=op[1]", &self.get_rn())
            .field("rm=op[2]", &self.get_rm())
            .field("rm_op", &self.rm_op)
            .finish()
    }
}
#[derive(Clone)]
pub struct LoadF64 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
    rm_op: Option<RegOP>,
}
impl IMirSubInst for LoadF64 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::LdrF64)
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(LoadF64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
            ],
            rm_op: None,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::LoadF64(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::LoadF64(self)
    }
}
impl LoadF64 {
    pub fn new(opcode: MirOP, rd: FPR64, rn: GPR64, rm: GPR64, rm_op: Option<RegOP>) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(LoadF64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rm.clean_use_flags().into_mir()),
            ],
            rm_op,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for LoadF64 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(LoadF64))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("rn=op[1]", &self.get_rn())
            .field("rm=op[2]", &self.get_rm())
            .field("rm_op", &self.rm_op)
            .finish()
    }
}
#[derive(Clone)]
pub struct LoadF32 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
    rm_op: Option<RegOP>,
}
impl IMirSubInst for LoadF32 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::LdrF32)
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(LoadF32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
            ],
            rm_op: None,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::LoadF32(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::LoadF32(self)
    }
}
impl LoadF32 {
    pub fn new(opcode: MirOP, rd: FPR32, rn: GPR64, rm: GPR64, rm_op: Option<RegOP>) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(LoadF32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rm.clean_use_flags().into_mir()),
            ],
            rm_op,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for LoadF32 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(LoadF32))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("rn=op[1]", &self.get_rn())
            .field("rm=op[2]", &self.get_rm())
            .field("rm_op", &self.rm_op)
            .finish()
    }
}
#[derive(Clone)]
pub struct StoreGr64 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
    rm_op: Option<RegOP>,
}
impl IMirSubInst for StoreGr64 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..0usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[0usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::StrGr64 | MirOP::StrBGr64 | MirOP::StrHGr64)
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(StoreGr64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
            ],
            rm_op: None,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::StoreGr64(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::StoreGr64(self)
    }
}
impl StoreGr64 {
    pub fn new(opcode: MirOP, rd: GPR64, rn: GPR64, rm: GPR64, rm_op: Option<RegOP>) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(StoreGr64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rm.clean_use_flags().into_mir()),
            ],
            rm_op,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for StoreGr64 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(StoreGr64))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("rn=op[1]", &self.get_rn())
            .field("rm=op[2]", &self.get_rm())
            .field("rm_op", &self.rm_op)
            .finish()
    }
}
#[derive(Clone)]
pub struct StoreGr32 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
    rm_op: Option<RegOP>,
}
impl IMirSubInst for StoreGr32 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..0usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[0usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::StrGr32 | MirOP::StrBGr32 | MirOP::StrHGr32)
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(StoreGr32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
            ],
            rm_op: None,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::StoreGr32(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::StoreGr32(self)
    }
}
impl StoreGr32 {
    pub fn new(opcode: MirOP, rd: GPR32, rn: GPR64, rm: GPR64, rm_op: Option<RegOP>) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(StoreGr32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rm.clean_use_flags().into_mir()),
            ],
            rm_op,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for StoreGr32 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(StoreGr32))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("rn=op[1]", &self.get_rn())
            .field("rm=op[2]", &self.get_rm())
            .field("rm_op", &self.rm_op)
            .finish()
    }
}
#[derive(Clone)]
pub struct StoreF64 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
    rm_op: Option<RegOP>,
}
impl IMirSubInst for StoreF64 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..0usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[0usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::StrF64)
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(StoreF64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
            ],
            rm_op: None,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::StoreF64(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::StoreF64(self)
    }
}
impl StoreF64 {
    pub fn new(opcode: MirOP, rd: FPR64, rn: GPR64, rm: GPR64, rm_op: Option<RegOP>) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(StoreF64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rm.clean_use_flags().into_mir()),
            ],
            rm_op,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for StoreF64 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(StoreF64))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("rn=op[1]", &self.get_rn())
            .field("rm=op[2]", &self.get_rm())
            .field("rm_op", &self.rm_op)
            .finish()
    }
}
#[derive(Clone)]
pub struct StoreF32 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
    rm_op: Option<RegOP>,
}
impl IMirSubInst for StoreF32 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..0usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[0usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::StrF32)
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(StoreF32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
            ],
            rm_op: None,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::StoreF32(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::StoreF32(self)
    }
}
impl StoreF32 {
    pub fn new(opcode: MirOP, rd: FPR32, rn: GPR64, rm: GPR64, rm_op: Option<RegOP>) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(StoreF32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rm.clean_use_flags().into_mir()),
            ],
            rm_op,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for StoreF32 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(StoreF32))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("rn=op[1]", &self.get_rn())
            .field("rm=op[2]", &self.get_rm())
            .field("rm_op", &self.rm_op)
            .finish()
    }
}
#[derive(Clone)]
pub struct LoadGr64Base {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for LoadGr64Base {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        )
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(LoadGr64Base),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmLSP64::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::LoadGr64Base(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::LoadGr64Base(self)
    }
}
impl LoadGr64Base {
    pub fn new(opcode: MirOP, rd: GPR64, rn: GPR64, rm: ImmLSP64) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(LoadGr64Base),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rm.into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
    #[doc = "operand rm at index 2 of type ImmLSP64"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type ImmLSP64"]
    pub fn get_rm(&self) -> ImmLSP64 {
        ImmLSP64::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type ImmLSP64 (checked by ImmLSP64)"]
    pub fn set_rm(&self, value: ImmLSP64) {
        self.rm().set(value.into_mir());
    }
}
impl std::fmt::Debug for LoadGr64Base {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(LoadGr64Base))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("rn=op[1]", &self.get_rn())
            .field("rm=op[2]", &self.get_rm())
            .finish()
    }
}
#[derive(Clone)]
pub struct LdrSWBase {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for LdrSWBase {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::LdrSWBase)
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(LdrSWBase),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmLSP32::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::LdrSWBase(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::LdrSWBase(self)
    }
}
impl LdrSWBase {
    pub fn new(opcode: MirOP, rd: GPR64, rn: GPR64, rm: ImmLSP32) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(LdrSWBase),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rm.into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
    #[doc = "operand rm at index 2 of type ImmLSP32"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type ImmLSP32"]
    pub fn get_rm(&self) -> ImmLSP32 {
        ImmLSP32::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type ImmLSP32 (checked by ImmLSP32)"]
    pub fn set_rm(&self, value: ImmLSP32) {
        self.rm().set(value.into_mir());
    }
}
impl std::fmt::Debug for LdrSWBase {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(LdrSWBase))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("rn=op[1]", &self.get_rn())
            .field("rm=op[2]", &self.get_rm())
            .finish()
    }
}
#[derive(Clone)]
pub struct LoadGr32Base {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for LoadGr32Base {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        )
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(LoadGr32Base),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmLSP32::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::LoadGr32Base(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::LoadGr32Base(self)
    }
}
impl LoadGr32Base {
    pub fn new(opcode: MirOP, rd: GPR32, rn: GPR64, rm: ImmLSP32) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(LoadGr32Base),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rm.into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
    #[doc = "operand rm at index 2 of type ImmLSP32"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type ImmLSP32"]
    pub fn get_rm(&self) -> ImmLSP32 {
        ImmLSP32::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type ImmLSP32 (checked by ImmLSP32)"]
    pub fn set_rm(&self, value: ImmLSP32) {
        self.rm().set(value.into_mir());
    }
}
impl std::fmt::Debug for LoadGr32Base {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(LoadGr32Base))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("rn=op[1]", &self.get_rn())
            .field("rm=op[2]", &self.get_rm())
            .finish()
    }
}
#[derive(Clone)]
pub struct LoadF64Base {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for LoadF64Base {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::LdrF64Base)
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(LoadF64Base),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmLSP64::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::LoadF64Base(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::LoadF64Base(self)
    }
}
impl LoadF64Base {
    pub fn new(opcode: MirOP, rd: FPR64, rn: GPR64, rm: ImmLSP64) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(LoadF64Base),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rm.into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
    #[doc = "operand rm at index 2 of type ImmLSP64"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type ImmLSP64"]
    pub fn get_rm(&self) -> ImmLSP64 {
        ImmLSP64::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type ImmLSP64 (checked by ImmLSP64)"]
    pub fn set_rm(&self, value: ImmLSP64) {
        self.rm().set(value.into_mir());
    }
}
impl std::fmt::Debug for LoadF64Base {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(LoadF64Base))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("rn=op[1]", &self.get_rn())
            .field("rm=op[2]", &self.get_rm())
            .finish()
    }
}
#[derive(Clone)]
pub struct LoadF32Base {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for LoadF32Base {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::LdrF32Base)
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(LoadF32Base),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmLSP32::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::LoadF32Base(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::LoadF32Base(self)
    }
}
impl LoadF32Base {
    pub fn new(opcode: MirOP, rd: FPR32, rn: GPR64, rm: ImmLSP32) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(LoadF32Base),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rm.into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
    #[doc = "operand rm at index 2 of type ImmLSP32"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type ImmLSP32"]
    pub fn get_rm(&self) -> ImmLSP32 {
        ImmLSP32::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type ImmLSP32 (checked by ImmLSP32)"]
    pub fn set_rm(&self, value: ImmLSP32) {
        self.rm().set(value.into_mir());
    }
}
impl std::fmt::Debug for LoadF32Base {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(LoadF32Base))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("rn=op[1]", &self.get_rn())
            .field("rm=op[2]", &self.get_rm())
            .finish()
    }
}
#[derive(Clone)]
pub struct LoadGr64BaseS {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for LoadGr64BaseS {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
            MirOP::LdrGr64BaseS
                | MirOP::LdrBGr64BaseS
                | MirOP::LdrHGr64BaseS
                | MirOP::LdrSBGr64BaseS
                | MirOP::LdrSHGr64BaseS
                | MirOP::LdrSWBaseS
        )
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(LoadGr64BaseS),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(MirSymbolOp::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::LoadGr64BaseS(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::LoadGr64BaseS(self)
    }
}
impl LoadGr64BaseS {
    pub fn new(opcode: MirOP, rd: GPR64, rn: GPR64, rm: MirSymbolOp) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(LoadGr64BaseS),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rm.into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
    #[doc = "operand rm at index 2 of type MirSymbolOp"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type MirSymbolOp"]
    pub fn get_rm(&self) -> MirSymbolOp {
        MirSymbolOp::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type MirSymbolOp (checked by MirSymbolOp)"]
    pub fn set_rm(&self, value: MirSymbolOp) {
        self.rm().set(value.into_mir());
    }
}
impl std::fmt::Debug for LoadGr64BaseS {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(LoadGr64BaseS))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("rn=op[1]", &self.get_rn())
            .field("rm=op[2]", &self.get_rm())
            .finish()
    }
}
#[derive(Clone)]
pub struct LoadGr32BaseS {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for LoadGr32BaseS {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
            MirOP::LdrGr32BaseS
                | MirOP::LdrBGr32BaseS
                | MirOP::LdrHGr32BaseS
                | MirOP::LdrSBGr32BaseS
                | MirOP::LdrSHGr32BaseS
        )
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(LoadGr32BaseS),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(MirSymbolOp::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::LoadGr32BaseS(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::LoadGr32BaseS(self)
    }
}
impl LoadGr32BaseS {
    pub fn new(opcode: MirOP, rd: GPR32, rn: GPR64, rm: MirSymbolOp) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(LoadGr32BaseS),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rm.into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
    #[doc = "operand rm at index 2 of type MirSymbolOp"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type MirSymbolOp"]
    pub fn get_rm(&self) -> MirSymbolOp {
        MirSymbolOp::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type MirSymbolOp (checked by MirSymbolOp)"]
    pub fn set_rm(&self, value: MirSymbolOp) {
        self.rm().set(value.into_mir());
    }
}
impl std::fmt::Debug for LoadGr32BaseS {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(LoadGr32BaseS))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("rn=op[1]", &self.get_rn())
            .field("rm=op[2]", &self.get_rm())
            .finish()
    }
}
#[derive(Clone)]
pub struct LoadF64BaseS {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for LoadF64BaseS {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::LdrF64BaseS)
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(LoadF64BaseS),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(MirSymbolOp::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::LoadF64BaseS(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::LoadF64BaseS(self)
    }
}
impl LoadF64BaseS {
    pub fn new(opcode: MirOP, rd: FPR64, rn: GPR64, rm: MirSymbolOp) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(LoadF64BaseS),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rm.into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
    #[doc = "operand rm at index 2 of type MirSymbolOp"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type MirSymbolOp"]
    pub fn get_rm(&self) -> MirSymbolOp {
        MirSymbolOp::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type MirSymbolOp (checked by MirSymbolOp)"]
    pub fn set_rm(&self, value: MirSymbolOp) {
        self.rm().set(value.into_mir());
    }
}
impl std::fmt::Debug for LoadF64BaseS {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(LoadF64BaseS))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("rn=op[1]", &self.get_rn())
            .field("rm=op[2]", &self.get_rm())
            .finish()
    }
}
#[derive(Clone)]
pub struct LoadF32BaseS {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for LoadF32BaseS {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::LdrF32BaseS)
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(LoadF32BaseS),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(MirSymbolOp::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::LoadF32BaseS(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::LoadF32BaseS(self)
    }
}
impl LoadF32BaseS {
    pub fn new(opcode: MirOP, rd: FPR32, rn: GPR64, rm: MirSymbolOp) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(LoadF32BaseS),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rm.into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
    #[doc = "operand rm at index 2 of type MirSymbolOp"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type MirSymbolOp"]
    pub fn get_rm(&self) -> MirSymbolOp {
        MirSymbolOp::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type MirSymbolOp (checked by MirSymbolOp)"]
    pub fn set_rm(&self, value: MirSymbolOp) {
        self.rm().set(value.into_mir());
    }
}
impl std::fmt::Debug for LoadF32BaseS {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(LoadF32BaseS))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("rn=op[1]", &self.get_rn())
            .field("rm=op[2]", &self.get_rm())
            .finish()
    }
}
#[derive(Clone)]
pub struct StoreGr64Base {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for StoreGr64Base {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..0usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[0usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(
            opcode,
            MirOP::StrGr64Base | MirOP::StrBGr64Base | MirOP::StrHGr64Base
        )
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(StoreGr64Base),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmLSP64::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::StoreGr64Base(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::StoreGr64Base(self)
    }
}
impl StoreGr64Base {
    pub fn new(opcode: MirOP, rd: GPR64, rn: GPR64, rm: ImmLSP64) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(StoreGr64Base),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rm.into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
    #[doc = "operand rm at index 2 of type ImmLSP64"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type ImmLSP64"]
    pub fn get_rm(&self) -> ImmLSP64 {
        ImmLSP64::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type ImmLSP64 (checked by ImmLSP64)"]
    pub fn set_rm(&self, value: ImmLSP64) {
        self.rm().set(value.into_mir());
    }
}
impl std::fmt::Debug for StoreGr64Base {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(StoreGr64Base))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("rn=op[1]", &self.get_rn())
            .field("rm=op[2]", &self.get_rm())
            .finish()
    }
}
#[derive(Clone)]
pub struct StoreGr32Base {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for StoreGr32Base {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..0usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[0usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(
            opcode,
            MirOP::StrGr32Base | MirOP::StrBGr32Base | MirOP::StrHGr32Base
        )
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(StoreGr32Base),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmLSP32::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::StoreGr32Base(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::StoreGr32Base(self)
    }
}
impl StoreGr32Base {
    pub fn new(opcode: MirOP, rd: GPR32, rn: GPR64, rm: ImmLSP32) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(StoreGr32Base),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rm.into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
    #[doc = "operand rm at index 2 of type ImmLSP32"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type ImmLSP32"]
    pub fn get_rm(&self) -> ImmLSP32 {
        ImmLSP32::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type ImmLSP32 (checked by ImmLSP32)"]
    pub fn set_rm(&self, value: ImmLSP32) {
        self.rm().set(value.into_mir());
    }
}
impl std::fmt::Debug for StoreGr32Base {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(StoreGr32Base))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("rn=op[1]", &self.get_rn())
            .field("rm=op[2]", &self.get_rm())
            .finish()
    }
}
#[derive(Clone)]
pub struct StoreF64Base {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for StoreF64Base {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..0usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[0usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::StrF64Base)
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(StoreF64Base),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmLSP64::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::StoreF64Base(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::StoreF64Base(self)
    }
}
impl StoreF64Base {
    pub fn new(opcode: MirOP, rd: FPR64, rn: GPR64, rm: ImmLSP64) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(StoreF64Base),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rm.into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
    #[doc = "operand rm at index 2 of type ImmLSP64"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type ImmLSP64"]
    pub fn get_rm(&self) -> ImmLSP64 {
        ImmLSP64::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type ImmLSP64 (checked by ImmLSP64)"]
    pub fn set_rm(&self, value: ImmLSP64) {
        self.rm().set(value.into_mir());
    }
}
impl std::fmt::Debug for StoreF64Base {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(StoreF64Base))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("rn=op[1]", &self.get_rn())
            .field("rm=op[2]", &self.get_rm())
            .finish()
    }
}
#[derive(Clone)]
pub struct StoreF32Base {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for StoreF32Base {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..0usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[0usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::StrF32Base)
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(StoreF32Base),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmLSP32::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::StoreF32Base(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::StoreF32Base(self)
    }
}
impl StoreF32Base {
    pub fn new(opcode: MirOP, rd: FPR32, rn: GPR64, rm: ImmLSP32) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(StoreF32Base),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rm.into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
    #[doc = "operand rm at index 2 of type ImmLSP32"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type ImmLSP32"]
    pub fn get_rm(&self) -> ImmLSP32 {
        ImmLSP32::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type ImmLSP32 (checked by ImmLSP32)"]
    pub fn set_rm(&self, value: ImmLSP32) {
        self.rm().set(value.into_mir());
    }
}
impl std::fmt::Debug for StoreF32Base {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(StoreF32Base))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("rn=op[1]", &self.get_rn())
            .field("rm=op[2]", &self.get_rm())
            .finish()
    }
}
#[derive(Clone)]
pub struct StoreGr64BaseS {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for StoreGr64BaseS {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..0usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[0usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(
            opcode,
            MirOP::StrGr64BaseS | MirOP::StrBGr64BaseS | MirOP::StrHGr64BaseS
        )
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(StoreGr64BaseS),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(MirSymbolOp::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::StoreGr64BaseS(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::StoreGr64BaseS(self)
    }
}
impl StoreGr64BaseS {
    pub fn new(opcode: MirOP, rd: GPR64, rn: GPR64, rm: MirSymbolOp) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(StoreGr64BaseS),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rm.into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
    #[doc = "operand rm at index 2 of type MirSymbolOp"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type MirSymbolOp"]
    pub fn get_rm(&self) -> MirSymbolOp {
        MirSymbolOp::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type MirSymbolOp (checked by MirSymbolOp)"]
    pub fn set_rm(&self, value: MirSymbolOp) {
        self.rm().set(value.into_mir());
    }
}
impl std::fmt::Debug for StoreGr64BaseS {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(StoreGr64BaseS))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("rn=op[1]", &self.get_rn())
            .field("rm=op[2]", &self.get_rm())
            .finish()
    }
}
#[derive(Clone)]
pub struct StoreGr32BaseS {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for StoreGr32BaseS {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..0usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[0usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(
            opcode,
            MirOP::StrGr32BaseS | MirOP::StrBGr32BaseS | MirOP::StrHGr32BaseS
        )
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(StoreGr32BaseS),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(MirSymbolOp::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::StoreGr32BaseS(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::StoreGr32BaseS(self)
    }
}
impl StoreGr32BaseS {
    pub fn new(opcode: MirOP, rd: GPR32, rn: GPR64, rm: MirSymbolOp) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(StoreGr32BaseS),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rm.into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
    #[doc = "operand rm at index 2 of type MirSymbolOp"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type MirSymbolOp"]
    pub fn get_rm(&self) -> MirSymbolOp {
        MirSymbolOp::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type MirSymbolOp (checked by MirSymbolOp)"]
    pub fn set_rm(&self, value: MirSymbolOp) {
        self.rm().set(value.into_mir());
    }
}
impl std::fmt::Debug for StoreGr32BaseS {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(StoreGr32BaseS))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("rn=op[1]", &self.get_rn())
            .field("rm=op[2]", &self.get_rm())
            .finish()
    }
}
#[derive(Clone)]
pub struct StoreF64BaseS {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for StoreF64BaseS {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..0usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[0usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::StrF64BaseS)
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(StoreF64BaseS),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(MirSymbolOp::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::StoreF64BaseS(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::StoreF64BaseS(self)
    }
}
impl StoreF64BaseS {
    pub fn new(opcode: MirOP, rd: FPR64, rn: GPR64, rm: MirSymbolOp) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(StoreF64BaseS),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rm.into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
    #[doc = "operand rm at index 2 of type MirSymbolOp"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type MirSymbolOp"]
    pub fn get_rm(&self) -> MirSymbolOp {
        MirSymbolOp::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type MirSymbolOp (checked by MirSymbolOp)"]
    pub fn set_rm(&self, value: MirSymbolOp) {
        self.rm().set(value.into_mir());
    }
}
impl std::fmt::Debug for StoreF64BaseS {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(StoreF64BaseS))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("rn=op[1]", &self.get_rn())
            .field("rm=op[2]", &self.get_rm())
            .finish()
    }
}
#[derive(Clone)]
pub struct StoreF32BaseS {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for StoreF32BaseS {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..0usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[0usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::StrF32BaseS)
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(StoreF32BaseS),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(MirSymbolOp::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::StoreF32BaseS(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::StoreF32BaseS(self)
    }
}
impl StoreF32BaseS {
    pub fn new(opcode: MirOP, rd: FPR32, rn: GPR64, rm: MirSymbolOp) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(StoreF32BaseS),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rm.into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
    #[doc = "operand rm at index 2 of type MirSymbolOp"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type MirSymbolOp"]
    pub fn get_rm(&self) -> MirSymbolOp {
        MirSymbolOp::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type MirSymbolOp (checked by MirSymbolOp)"]
    pub fn set_rm(&self, value: MirSymbolOp) {
        self.rm().set(value.into_mir());
    }
}
impl std::fmt::Debug for StoreF32BaseS {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(StoreF32BaseS))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("rn=op[1]", &self.get_rn())
            .field("rm=op[2]", &self.get_rm())
            .finish()
    }
}
#[derive(Clone)]
pub struct LoadGr64Indexed {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
    addr_mode: AddrMode,
}
impl IMirSubInst for LoadGr64Indexed {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        )
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(LoadGr64Indexed),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmLSP64::new_empty().into_mir()),
            ],
            addr_mode: AddrMode::PostIndex,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        {
            super::utils::mark_operand_used(ret.rd());
        }
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::LoadGr64Indexed(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::LoadGr64Indexed(self)
    }
}
impl LoadGr64Indexed {
    pub fn new(opcode: MirOP, rd: GPR64, rn: GPR64, rm: ImmLSP64, addr_mode: AddrMode) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(LoadGr64Indexed),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rm.into_mir()),
            ],
            addr_mode,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        {
            super::utils::mark_operand_used(ret.rd());
        }
        ret
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
    #[doc = "operand rm at index 2 of type ImmLSP64"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type ImmLSP64"]
    pub fn get_rm(&self) -> ImmLSP64 {
        ImmLSP64::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type ImmLSP64 (checked by ImmLSP64)"]
    pub fn set_rm(&self, value: ImmLSP64) {
        self.rm().set(value.into_mir());
    }
    pub fn get_addr_mode(&self) -> AddrMode {
        self.addr_mode.clone()
    }
    pub fn set_addr_mode(&mut self, value: AddrMode) {
        self.addr_mode = value;
    }
}
impl std::fmt::Debug for LoadGr64Indexed {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(LoadGr64Indexed))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("rn=op[1]", &self.get_rn())
            .field("rm=op[2]", &self.get_rm())
            .field("addr_mode", &self.addr_mode)
            .finish()
    }
}
#[derive(Clone)]
pub struct LoadGr32Indexed {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
    addr_mode: AddrMode,
}
impl IMirSubInst for LoadGr32Indexed {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        )
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(LoadGr32Indexed),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmLSP32::new_empty().into_mir()),
            ],
            addr_mode: AddrMode::PostIndex,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        {
            super::utils::mark_operand_used(ret.rd());
        }
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::LoadGr32Indexed(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::LoadGr32Indexed(self)
    }
}
impl LoadGr32Indexed {
    pub fn new(opcode: MirOP, rd: GPR32, rn: GPR64, rm: ImmLSP32, addr_mode: AddrMode) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(LoadGr32Indexed),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rm.into_mir()),
            ],
            addr_mode,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        {
            super::utils::mark_operand_used(ret.rd());
        }
        ret
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
    #[doc = "operand rm at index 2 of type ImmLSP32"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type ImmLSP32"]
    pub fn get_rm(&self) -> ImmLSP32 {
        ImmLSP32::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type ImmLSP32 (checked by ImmLSP32)"]
    pub fn set_rm(&self, value: ImmLSP32) {
        self.rm().set(value.into_mir());
    }
    pub fn get_addr_mode(&self) -> AddrMode {
        self.addr_mode.clone()
    }
    pub fn set_addr_mode(&mut self, value: AddrMode) {
        self.addr_mode = value;
    }
}
impl std::fmt::Debug for LoadGr32Indexed {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(LoadGr32Indexed))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("rn=op[1]", &self.get_rn())
            .field("rm=op[2]", &self.get_rm())
            .field("addr_mode", &self.addr_mode)
            .finish()
    }
}
#[derive(Clone)]
pub struct LoadF64Indexed {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
    addr_mode: AddrMode,
}
impl IMirSubInst for LoadF64Indexed {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..2usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[2usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::LdrF64Indexed)
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(LoadF64Indexed),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmLSP64::new_empty().into_mir()),
            ],
            addr_mode: AddrMode::PostIndex,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        {
            super::utils::mark_operand_used(ret.rd());
        }
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::LoadF64Indexed(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::LoadF64Indexed(self)
    }
}
impl LoadF64Indexed {
    pub fn new(opcode: MirOP, rd: FPR64, rn: GPR64, rm: ImmLSP64, addr_mode: AddrMode) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(LoadF64Indexed),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rm.into_mir()),
            ],
            addr_mode,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        {
            super::utils::mark_operand_used(ret.rd());
        }
        ret
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
    #[doc = "operand rm at index 2 of type ImmLSP64"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type ImmLSP64"]
    pub fn get_rm(&self) -> ImmLSP64 {
        ImmLSP64::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type ImmLSP64 (checked by ImmLSP64)"]
    pub fn set_rm(&self, value: ImmLSP64) {
        self.rm().set(value.into_mir());
    }
    pub fn get_addr_mode(&self) -> AddrMode {
        self.addr_mode.clone()
    }
    pub fn set_addr_mode(&mut self, value: AddrMode) {
        self.addr_mode = value;
    }
}
impl std::fmt::Debug for LoadF64Indexed {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(LoadF64Indexed))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("rn=op[1]", &self.get_rn())
            .field("rm=op[2]", &self.get_rm())
            .field("addr_mode", &self.addr_mode)
            .finish()
    }
}
#[derive(Clone)]
pub struct LoadF32Indexed {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
    addr_mode: AddrMode,
}
impl IMirSubInst for LoadF32Indexed {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..2usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[2usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::LdrF32Indexed)
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(LoadF32Indexed),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmLSP32::new_empty().into_mir()),
            ],
            addr_mode: AddrMode::PostIndex,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        {
            super::utils::mark_operand_used(ret.rd());
        }
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::LoadF32Indexed(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::LoadF32Indexed(self)
    }
}
impl LoadF32Indexed {
    pub fn new(opcode: MirOP, rd: FPR32, rn: GPR64, rm: ImmLSP32, addr_mode: AddrMode) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(LoadF32Indexed),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rm.into_mir()),
            ],
            addr_mode,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        {
            super::utils::mark_operand_used(ret.rd());
        }
        ret
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
    #[doc = "operand rm at index 2 of type ImmLSP32"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type ImmLSP32"]
    pub fn get_rm(&self) -> ImmLSP32 {
        ImmLSP32::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type ImmLSP32 (checked by ImmLSP32)"]
    pub fn set_rm(&self, value: ImmLSP32) {
        self.rm().set(value.into_mir());
    }
    pub fn get_addr_mode(&self) -> AddrMode {
        self.addr_mode.clone()
    }
    pub fn set_addr_mode(&mut self, value: AddrMode) {
        self.addr_mode = value;
    }
}
impl std::fmt::Debug for LoadF32Indexed {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(LoadF32Indexed))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("rn=op[1]", &self.get_rn())
            .field("rm=op[2]", &self.get_rm())
            .field("addr_mode", &self.addr_mode)
            .finish()
    }
}
#[derive(Clone)]
pub struct StoreGr64Indexed {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
    addr_mode: AddrMode,
}
impl IMirSubInst for StoreGr64Indexed {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
            MirOP::StrGr64Indexed | MirOP::StrBGr64Indexed | MirOP::StrHGr64Indexed
        )
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(StoreGr64Indexed),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmLSP64::new_empty().into_mir()),
            ],
            addr_mode: AddrMode::PostIndex,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::StoreGr64Indexed(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::StoreGr64Indexed(self)
    }
}
impl StoreGr64Indexed {
    pub fn new(opcode: MirOP, rn: GPR64, rd: GPR64, rm: ImmLSP64, addr_mode: AddrMode) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(StoreGr64Indexed),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rm.into_mir()),
            ],
            addr_mode,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    #[doc = "operand rn at index 0 of type GPReg"]
    pub fn rn(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand rn at index 0 of type GPReg"]
    pub fn get_rn(&self) -> GPReg {
        GPReg::from_mir(self.rn().get())
    }
    #[doc = "set the value of operand rn at 0 to a value of type GPReg (checked by GPR64)"]
    pub fn set_rn(&self, value: GPReg) {
        let prev_value = self.get_rn();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rn().set(next_value.into_mir());
    }
    #[doc = "operand rd at index 1 of type GPReg"]
    pub fn rd(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand rd at index 1 of type GPReg"]
    pub fn get_rd(&self) -> GPReg {
        GPReg::from_mir(self.rd().get())
    }
    #[doc = "set the value of operand rd at 1 to a value of type GPReg (checked by GPR64)"]
    pub fn set_rd(&self, value: GPReg) {
        let prev_value = self.get_rd();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rd().set(next_value.into_mir());
    }
    #[doc = "operand rm at index 2 of type ImmLSP64"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type ImmLSP64"]
    pub fn get_rm(&self) -> ImmLSP64 {
        ImmLSP64::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type ImmLSP64 (checked by ImmLSP64)"]
    pub fn set_rm(&self, value: ImmLSP64) {
        self.rm().set(value.into_mir());
    }
    pub fn get_addr_mode(&self) -> AddrMode {
        self.addr_mode.clone()
    }
    pub fn set_addr_mode(&mut self, value: AddrMode) {
        self.addr_mode = value;
    }
}
impl std::fmt::Debug for StoreGr64Indexed {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(StoreGr64Indexed))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rn=op[0]", &self.get_rn())
            .field("rd=op[1]", &self.get_rd())
            .field("rm=op[2]", &self.get_rm())
            .field("addr_mode", &self.addr_mode)
            .finish()
    }
}
#[derive(Clone)]
pub struct StoreGr32Indexed {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
    addr_mode: AddrMode,
}
impl IMirSubInst for StoreGr32Indexed {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
            MirOP::StrGr32Indexed | MirOP::StrBGr32Indexed | MirOP::StrHGr32Indexed
        )
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(StoreGr32Indexed),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmLSP32::new_empty().into_mir()),
            ],
            addr_mode: AddrMode::PostIndex,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::StoreGr32Indexed(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::StoreGr32Indexed(self)
    }
}
impl StoreGr32Indexed {
    pub fn new(opcode: MirOP, rn: GPR64, rd: GPR32, rm: ImmLSP32, addr_mode: AddrMode) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(StoreGr32Indexed),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rm.into_mir()),
            ],
            addr_mode,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    #[doc = "operand rn at index 0 of type GPReg"]
    pub fn rn(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand rn at index 0 of type GPReg"]
    pub fn get_rn(&self) -> GPReg {
        GPReg::from_mir(self.rn().get())
    }
    #[doc = "set the value of operand rn at 0 to a value of type GPReg (checked by GPR64)"]
    pub fn set_rn(&self, value: GPReg) {
        let prev_value = self.get_rn();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rn().set(next_value.into_mir());
    }
    #[doc = "operand rd at index 1 of type GPReg"]
    pub fn rd(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand rd at index 1 of type GPReg"]
    pub fn get_rd(&self) -> GPReg {
        GPReg::from_mir(self.rd().get())
    }
    #[doc = "set the value of operand rd at 1 to a value of type GPReg (checked by GPR32)"]
    pub fn set_rd(&self, value: GPReg) {
        let prev_value = self.get_rd();
        let checked_value = GPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rd().set(next_value.into_mir());
    }
    #[doc = "operand rm at index 2 of type ImmLSP32"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type ImmLSP32"]
    pub fn get_rm(&self) -> ImmLSP32 {
        ImmLSP32::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type ImmLSP32 (checked by ImmLSP32)"]
    pub fn set_rm(&self, value: ImmLSP32) {
        self.rm().set(value.into_mir());
    }
    pub fn get_addr_mode(&self) -> AddrMode {
        self.addr_mode.clone()
    }
    pub fn set_addr_mode(&mut self, value: AddrMode) {
        self.addr_mode = value;
    }
}
impl std::fmt::Debug for StoreGr32Indexed {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(StoreGr32Indexed))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rn=op[0]", &self.get_rn())
            .field("rd=op[1]", &self.get_rd())
            .field("rm=op[2]", &self.get_rm())
            .field("addr_mode", &self.addr_mode)
            .finish()
    }
}
#[derive(Clone)]
pub struct StoreF64Indexed {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
    addr_mode: AddrMode,
}
impl IMirSubInst for StoreF64Indexed {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::StrF64Indexed)
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(StoreF64Indexed),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(ImmLSP64::new_empty().into_mir()),
            ],
            addr_mode: AddrMode::PostIndex,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::StoreF64Indexed(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::StoreF64Indexed(self)
    }
}
impl StoreF64Indexed {
    pub fn new(opcode: MirOP, rn: GPR64, rd: FPR64, rm: ImmLSP64, addr_mode: AddrMode) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(StoreF64Indexed),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rm.into_mir()),
            ],
            addr_mode,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    #[doc = "operand rn at index 0 of type GPReg"]
    pub fn rn(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand rn at index 0 of type GPReg"]
    pub fn get_rn(&self) -> GPReg {
        GPReg::from_mir(self.rn().get())
    }
    #[doc = "set the value of operand rn at 0 to a value of type GPReg (checked by GPR64)"]
    pub fn set_rn(&self, value: GPReg) {
        let prev_value = self.get_rn();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rn().set(next_value.into_mir());
    }
    #[doc = "operand rd at index 1 of type VFReg"]
    pub fn rd(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand rd at index 1 of type VFReg"]
    pub fn get_rd(&self) -> VFReg {
        VFReg::from_mir(self.rd().get())
    }
    #[doc = "set the value of operand rd at 1 to a value of type VFReg (checked by FPR64)"]
    pub fn set_rd(&self, value: VFReg) {
        let prev_value = self.get_rd();
        let checked_value = FPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rd().set(next_value.into_mir());
    }
    #[doc = "operand rm at index 2 of type ImmLSP64"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type ImmLSP64"]
    pub fn get_rm(&self) -> ImmLSP64 {
        ImmLSP64::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type ImmLSP64 (checked by ImmLSP64)"]
    pub fn set_rm(&self, value: ImmLSP64) {
        self.rm().set(value.into_mir());
    }
    pub fn get_addr_mode(&self) -> AddrMode {
        self.addr_mode.clone()
    }
    pub fn set_addr_mode(&mut self, value: AddrMode) {
        self.addr_mode = value;
    }
}
impl std::fmt::Debug for StoreF64Indexed {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(StoreF64Indexed))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rn=op[0]", &self.get_rn())
            .field("rd=op[1]", &self.get_rd())
            .field("rm=op[2]", &self.get_rm())
            .field("addr_mode", &self.addr_mode)
            .finish()
    }
}
#[derive(Clone)]
pub struct StoreF32Indexed {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
    addr_mode: AddrMode,
}
impl IMirSubInst for StoreF32Indexed {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::StrF32Indexed)
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(StoreF32Indexed),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(ImmLSP32::new_empty().into_mir()),
            ],
            addr_mode: AddrMode::PostIndex,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::StoreF32Indexed(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::StoreF32Indexed(self)
    }
}
impl StoreF32Indexed {
    pub fn new(opcode: MirOP, rn: GPR64, rd: FPR32, rm: ImmLSP32, addr_mode: AddrMode) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(StoreF32Indexed),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rm.into_mir()),
            ],
            addr_mode,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    #[doc = "operand rn at index 0 of type GPReg"]
    pub fn rn(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand rn at index 0 of type GPReg"]
    pub fn get_rn(&self) -> GPReg {
        GPReg::from_mir(self.rn().get())
    }
    #[doc = "set the value of operand rn at 0 to a value of type GPReg (checked by GPR64)"]
    pub fn set_rn(&self, value: GPReg) {
        let prev_value = self.get_rn();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rn().set(next_value.into_mir());
    }
    #[doc = "operand rd at index 1 of type VFReg"]
    pub fn rd(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand rd at index 1 of type VFReg"]
    pub fn get_rd(&self) -> VFReg {
        VFReg::from_mir(self.rd().get())
    }
    #[doc = "set the value of operand rd at 1 to a value of type VFReg (checked by FPR32)"]
    pub fn set_rd(&self, value: VFReg) {
        let prev_value = self.get_rd();
        let checked_value = FPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rd().set(next_value.into_mir());
    }
    #[doc = "operand rm at index 2 of type ImmLSP32"]
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand rm at index 2 of type ImmLSP32"]
    pub fn get_rm(&self) -> ImmLSP32 {
        ImmLSP32::from_mir(self.rm().get())
    }
    #[doc = "set the value of operand rm at 2 to a value of type ImmLSP32 (checked by ImmLSP32)"]
    pub fn set_rm(&self, value: ImmLSP32) {
        self.rm().set(value.into_mir());
    }
    pub fn get_addr_mode(&self) -> AddrMode {
        self.addr_mode.clone()
    }
    pub fn set_addr_mode(&mut self, value: AddrMode) {
        self.addr_mode = value;
    }
}
impl std::fmt::Debug for StoreF32Indexed {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(StoreF32Indexed))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rn=op[0]", &self.get_rn())
            .field("rd=op[1]", &self.get_rd())
            .field("rm=op[2]", &self.get_rm())
            .field("addr_mode", &self.addr_mode)
            .finish()
    }
}
#[derive(Clone)]
pub struct LoadGr64Literal {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for LoadGr64Literal {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
                | MirOP::LdrSWLiteral
        )
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(LoadGr64Literal),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(MirSymbolOp::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::LoadGr64Literal(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::LoadGr64Literal(self)
    }
}
impl LoadGr64Literal {
    pub fn new(opcode: MirOP, rd: GPR64, from: MirSymbolOp) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(LoadGr64Literal),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(rd.clean_use_flags().into_mir()), Cell::new(from.into_mir())],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for LoadGr64Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(LoadGr64Literal))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("from=op[1]", &self.get_from())
            .finish()
    }
}
#[derive(Clone)]
pub struct LoadGr32Literal {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for LoadGr32Literal {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        )
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(LoadGr32Literal),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(MirSymbolOp::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::LoadGr32Literal(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::LoadGr32Literal(self)
    }
}
impl LoadGr32Literal {
    pub fn new(opcode: MirOP, rd: GPR32, from: MirSymbolOp) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(LoadGr32Literal),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(rd.clean_use_flags().into_mir()), Cell::new(from.into_mir())],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for LoadGr32Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(LoadGr32Literal))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("from=op[1]", &self.get_from())
            .finish()
    }
}
#[derive(Clone)]
pub struct LoadF64Literal {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for LoadF64Literal {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..2usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::LdrF64Literal)
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(LoadF64Literal),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(MirSymbolOp::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::LoadF64Literal(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::LoadF64Literal(self)
    }
}
impl LoadF64Literal {
    pub fn new(opcode: MirOP, rd: FPR64, from: MirSymbolOp) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(LoadF64Literal),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(rd.clean_use_flags().into_mir()), Cell::new(from.into_mir())],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for LoadF64Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(LoadF64Literal))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("from=op[1]", &self.get_from())
            .finish()
    }
}
#[derive(Clone)]
pub struct LoadF32Literal {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for LoadF32Literal {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..2usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::LdrF32Literal)
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(LoadF32Literal),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(MirSymbolOp::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::LoadF32Literal(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::LoadF32Literal(self)
    }
}
impl LoadF32Literal {
    pub fn new(opcode: MirOP, rd: FPR32, from: MirSymbolOp) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(LoadF32Literal),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(rd.clean_use_flags().into_mir()), Cell::new(from.into_mir())],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for LoadF32Literal {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(LoadF32Literal))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("from=op[1]", &self.get_from())
            .finish()
    }
}
#[derive(Clone)]
pub struct MirLdrLitG64 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for MirLdrLitG64 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..2usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[2usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::MirLdrLitG64)
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(MirLdrLitG64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(MirSymbolOp::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::MirLdrLitG64(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::MirLdrLitG64(self)
    }
}
impl MirLdrLitG64 {
    pub fn new(opcode: MirOP, dst: GPR64, tmp_addr: GPR64, src: MirSymbolOp) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(MirLdrLitG64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(dst.clean_use_flags().into_mir()),
                Cell::new(tmp_addr.clean_use_flags().into_mir()),
                Cell::new(src.into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
    #[doc = "operand tmp_addr at index 1 of type GPReg"]
    pub fn tmp_addr(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand tmp_addr at index 1 of type GPReg"]
    pub fn get_tmp_addr(&self) -> GPReg {
        GPReg::from_mir(self.tmp_addr().get())
    }
    #[doc = "set the value of operand tmp_addr at 1 to a value of type GPReg (checked by GPR64)"]
    pub fn set_tmp_addr(&self, value: GPReg) {
        let prev_value = self.get_tmp_addr();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.tmp_addr().set(next_value.into_mir());
    }
    #[doc = "operand src at index 2 of type MirSymbolOp"]
    pub fn src(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand src at index 2 of type MirSymbolOp"]
    pub fn get_src(&self) -> MirSymbolOp {
        MirSymbolOp::from_mir(self.src().get())
    }
    #[doc = "set the value of operand src at 2 to a value of type MirSymbolOp (checked by MirSymbolOp)"]
    pub fn set_src(&self, value: MirSymbolOp) {
        self.src().set(value.into_mir());
    }
}
impl std::fmt::Debug for MirLdrLitG64 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(MirLdrLitG64))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("dst=op[0]", &self.get_dst())
            .field("tmp_addr=op[1]", &self.get_tmp_addr())
            .field("src=op[2]", &self.get_src())
            .finish()
    }
}
#[derive(Clone)]
pub struct MirLdrLitG32 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for MirLdrLitG32 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..2usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[2usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::MirLdrLitG32)
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(MirLdrLitG32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(MirSymbolOp::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::MirLdrLitG32(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::MirLdrLitG32(self)
    }
}
impl MirLdrLitG32 {
    pub fn new(opcode: MirOP, dst: GPR32, tmp_addr: GPR64, src: MirSymbolOp) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(MirLdrLitG32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(dst.clean_use_flags().into_mir()),
                Cell::new(tmp_addr.clean_use_flags().into_mir()),
                Cell::new(src.into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
    #[doc = "operand tmp_addr at index 1 of type GPReg"]
    pub fn tmp_addr(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand tmp_addr at index 1 of type GPReg"]
    pub fn get_tmp_addr(&self) -> GPReg {
        GPReg::from_mir(self.tmp_addr().get())
    }
    #[doc = "set the value of operand tmp_addr at 1 to a value of type GPReg (checked by GPR64)"]
    pub fn set_tmp_addr(&self, value: GPReg) {
        let prev_value = self.get_tmp_addr();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.tmp_addr().set(next_value.into_mir());
    }
    #[doc = "operand src at index 2 of type MirSymbolOp"]
    pub fn src(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand src at index 2 of type MirSymbolOp"]
    pub fn get_src(&self) -> MirSymbolOp {
        MirSymbolOp::from_mir(self.src().get())
    }
    #[doc = "set the value of operand src at 2 to a value of type MirSymbolOp (checked by MirSymbolOp)"]
    pub fn set_src(&self, value: MirSymbolOp) {
        self.src().set(value.into_mir());
    }
}
impl std::fmt::Debug for MirLdrLitG32 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(MirLdrLitG32))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("dst=op[0]", &self.get_dst())
            .field("tmp_addr=op[1]", &self.get_tmp_addr())
            .field("src=op[2]", &self.get_src())
            .finish()
    }
}
#[derive(Clone)]
pub struct MirLdrLitF64 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for MirLdrLitF64 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..2usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[2usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::MirLdrLitF64)
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(MirLdrLitF64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(MirSymbolOp::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::MirLdrLitF64(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::MirLdrLitF64(self)
    }
}
impl MirLdrLitF64 {
    pub fn new(opcode: MirOP, dst: FPR64, tmp_addr: GPR64, src: MirSymbolOp) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(MirLdrLitF64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(dst.clean_use_flags().into_mir()),
                Cell::new(tmp_addr.clean_use_flags().into_mir()),
                Cell::new(src.into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
    #[doc = "operand tmp_addr at index 1 of type GPReg"]
    pub fn tmp_addr(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand tmp_addr at index 1 of type GPReg"]
    pub fn get_tmp_addr(&self) -> GPReg {
        GPReg::from_mir(self.tmp_addr().get())
    }
    #[doc = "set the value of operand tmp_addr at 1 to a value of type GPReg (checked by GPR64)"]
    pub fn set_tmp_addr(&self, value: GPReg) {
        let prev_value = self.get_tmp_addr();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.tmp_addr().set(next_value.into_mir());
    }
    #[doc = "operand src at index 2 of type MirSymbolOp"]
    pub fn src(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand src at index 2 of type MirSymbolOp"]
    pub fn get_src(&self) -> MirSymbolOp {
        MirSymbolOp::from_mir(self.src().get())
    }
    #[doc = "set the value of operand src at 2 to a value of type MirSymbolOp (checked by MirSymbolOp)"]
    pub fn set_src(&self, value: MirSymbolOp) {
        self.src().set(value.into_mir());
    }
}
impl std::fmt::Debug for MirLdrLitF64 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(MirLdrLitF64))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("dst=op[0]", &self.get_dst())
            .field("tmp_addr=op[1]", &self.get_tmp_addr())
            .field("src=op[2]", &self.get_src())
            .finish()
    }
}
#[derive(Clone)]
pub struct MirLdrLitF32 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for MirLdrLitF32 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..2usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[2usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::MirLdrLitF32)
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(MirLdrLitF32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(MirSymbolOp::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::MirLdrLitF32(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::MirLdrLitF32(self)
    }
}
impl MirLdrLitF32 {
    pub fn new(opcode: MirOP, dst: FPR32, tmp_addr: GPR64, src: MirSymbolOp) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(MirLdrLitF32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(dst.clean_use_flags().into_mir()),
                Cell::new(tmp_addr.clean_use_flags().into_mir()),
                Cell::new(src.into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
    #[doc = "operand tmp_addr at index 1 of type GPReg"]
    pub fn tmp_addr(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand tmp_addr at index 1 of type GPReg"]
    pub fn get_tmp_addr(&self) -> GPReg {
        GPReg::from_mir(self.tmp_addr().get())
    }
    #[doc = "set the value of operand tmp_addr at 1 to a value of type GPReg (checked by GPR64)"]
    pub fn set_tmp_addr(&self, value: GPReg) {
        let prev_value = self.get_tmp_addr();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.tmp_addr().set(next_value.into_mir());
    }
    #[doc = "operand src at index 2 of type MirSymbolOp"]
    pub fn src(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand src at index 2 of type MirSymbolOp"]
    pub fn get_src(&self) -> MirSymbolOp {
        MirSymbolOp::from_mir(self.src().get())
    }
    #[doc = "set the value of operand src at 2 to a value of type MirSymbolOp (checked by MirSymbolOp)"]
    pub fn set_src(&self, value: MirSymbolOp) {
        self.src().set(value.into_mir());
    }
}
impl std::fmt::Debug for MirLdrLitF32 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(MirLdrLitF32))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("dst=op[0]", &self.get_dst())
            .field("tmp_addr=op[1]", &self.get_tmp_addr())
            .field("src=op[2]", &self.get_src())
            .finish()
    }
}
#[derive(Clone)]
pub struct MirStrLitG64 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for MirStrLitG64 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::MirStrLitG64)
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(MirStrLitG64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(MirSymbolOp::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::MirStrLitG64(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::MirStrLitG64(self)
    }
}
impl MirStrLitG64 {
    pub fn new(opcode: MirOP, tmp_addr: GPR64, rd: GPR64, to: MirSymbolOp) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(MirStrLitG64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(tmp_addr.clean_use_flags().into_mir()),
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(to.into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    #[doc = "operand tmp_addr at index 0 of type GPReg"]
    pub fn tmp_addr(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand tmp_addr at index 0 of type GPReg"]
    pub fn get_tmp_addr(&self) -> GPReg {
        GPReg::from_mir(self.tmp_addr().get())
    }
    #[doc = "set the value of operand tmp_addr at 0 to a value of type GPReg (checked by GPR64)"]
    pub fn set_tmp_addr(&self, value: GPReg) {
        let prev_value = self.get_tmp_addr();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.tmp_addr().set(next_value.into_mir());
    }
    #[doc = "operand rd at index 1 of type GPReg"]
    pub fn rd(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand rd at index 1 of type GPReg"]
    pub fn get_rd(&self) -> GPReg {
        GPReg::from_mir(self.rd().get())
    }
    #[doc = "set the value of operand rd at 1 to a value of type GPReg (checked by GPR64)"]
    pub fn set_rd(&self, value: GPReg) {
        let prev_value = self.get_rd();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rd().set(next_value.into_mir());
    }
    #[doc = "operand to at index 2 of type MirSymbolOp"]
    pub fn to(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand to at index 2 of type MirSymbolOp"]
    pub fn get_to(&self) -> MirSymbolOp {
        MirSymbolOp::from_mir(self.to().get())
    }
    #[doc = "set the value of operand to at 2 to a value of type MirSymbolOp (checked by MirSymbolOp)"]
    pub fn set_to(&self, value: MirSymbolOp) {
        self.to().set(value.into_mir());
    }
}
impl std::fmt::Debug for MirStrLitG64 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(MirStrLitG64))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("tmp_addr=op[0]", &self.get_tmp_addr())
            .field("rd=op[1]", &self.get_rd())
            .field("to=op[2]", &self.get_to())
            .finish()
    }
}
#[derive(Clone)]
pub struct MirStrLitG32 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for MirStrLitG32 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::MirStrLitG32)
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(MirStrLitG32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(MirSymbolOp::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::MirStrLitG32(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::MirStrLitG32(self)
    }
}
impl MirStrLitG32 {
    pub fn new(opcode: MirOP, tmp_addr: GPR64, rd: GPR32, to: MirSymbolOp) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(MirStrLitG32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(tmp_addr.clean_use_flags().into_mir()),
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(to.into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    #[doc = "operand tmp_addr at index 0 of type GPReg"]
    pub fn tmp_addr(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand tmp_addr at index 0 of type GPReg"]
    pub fn get_tmp_addr(&self) -> GPReg {
        GPReg::from_mir(self.tmp_addr().get())
    }
    #[doc = "set the value of operand tmp_addr at 0 to a value of type GPReg (checked by GPR64)"]
    pub fn set_tmp_addr(&self, value: GPReg) {
        let prev_value = self.get_tmp_addr();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.tmp_addr().set(next_value.into_mir());
    }
    #[doc = "operand rd at index 1 of type GPReg"]
    pub fn rd(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand rd at index 1 of type GPReg"]
    pub fn get_rd(&self) -> GPReg {
        GPReg::from_mir(self.rd().get())
    }
    #[doc = "set the value of operand rd at 1 to a value of type GPReg (checked by GPR32)"]
    pub fn set_rd(&self, value: GPReg) {
        let prev_value = self.get_rd();
        let checked_value = GPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rd().set(next_value.into_mir());
    }
    #[doc = "operand to at index 2 of type MirSymbolOp"]
    pub fn to(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand to at index 2 of type MirSymbolOp"]
    pub fn get_to(&self) -> MirSymbolOp {
        MirSymbolOp::from_mir(self.to().get())
    }
    #[doc = "set the value of operand to at 2 to a value of type MirSymbolOp (checked by MirSymbolOp)"]
    pub fn set_to(&self, value: MirSymbolOp) {
        self.to().set(value.into_mir());
    }
}
impl std::fmt::Debug for MirStrLitG32 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(MirStrLitG32))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("tmp_addr=op[0]", &self.get_tmp_addr())
            .field("rd=op[1]", &self.get_rd())
            .field("to=op[2]", &self.get_to())
            .finish()
    }
}
#[derive(Clone)]
pub struct MirStrLitF64 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for MirStrLitF64 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::MirStrLitF64)
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(MirStrLitF64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(MirSymbolOp::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::MirStrLitF64(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::MirStrLitF64(self)
    }
}
impl MirStrLitF64 {
    pub fn new(opcode: MirOP, tmp_addr: GPR64, rd: FPR64, to: MirSymbolOp) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(MirStrLitF64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(tmp_addr.clean_use_flags().into_mir()),
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(to.into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    #[doc = "operand tmp_addr at index 0 of type GPReg"]
    pub fn tmp_addr(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand tmp_addr at index 0 of type GPReg"]
    pub fn get_tmp_addr(&self) -> GPReg {
        GPReg::from_mir(self.tmp_addr().get())
    }
    #[doc = "set the value of operand tmp_addr at 0 to a value of type GPReg (checked by GPR64)"]
    pub fn set_tmp_addr(&self, value: GPReg) {
        let prev_value = self.get_tmp_addr();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.tmp_addr().set(next_value.into_mir());
    }
    #[doc = "operand rd at index 1 of type VFReg"]
    pub fn rd(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand rd at index 1 of type VFReg"]
    pub fn get_rd(&self) -> VFReg {
        VFReg::from_mir(self.rd().get())
    }
    #[doc = "set the value of operand rd at 1 to a value of type VFReg (checked by FPR64)"]
    pub fn set_rd(&self, value: VFReg) {
        let prev_value = self.get_rd();
        let checked_value = FPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rd().set(next_value.into_mir());
    }
    #[doc = "operand to at index 2 of type MirSymbolOp"]
    pub fn to(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand to at index 2 of type MirSymbolOp"]
    pub fn get_to(&self) -> MirSymbolOp {
        MirSymbolOp::from_mir(self.to().get())
    }
    #[doc = "set the value of operand to at 2 to a value of type MirSymbolOp (checked by MirSymbolOp)"]
    pub fn set_to(&self, value: MirSymbolOp) {
        self.to().set(value.into_mir());
    }
}
impl std::fmt::Debug for MirStrLitF64 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(MirStrLitF64))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("tmp_addr=op[0]", &self.get_tmp_addr())
            .field("rd=op[1]", &self.get_rd())
            .field("to=op[2]", &self.get_to())
            .finish()
    }
}
#[derive(Clone)]
pub struct MirStrLitF32 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for MirStrLitF32 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::MirStrLitF32)
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(MirStrLitF32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(MirSymbolOp::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::MirStrLitF32(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::MirStrLitF32(self)
    }
}
impl MirStrLitF32 {
    pub fn new(opcode: MirOP, tmp_addr: GPR64, rd: FPR32, to: MirSymbolOp) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(MirStrLitF32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(tmp_addr.clean_use_flags().into_mir()),
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(to.into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    #[doc = "operand tmp_addr at index 0 of type GPReg"]
    pub fn tmp_addr(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand tmp_addr at index 0 of type GPReg"]
    pub fn get_tmp_addr(&self) -> GPReg {
        GPReg::from_mir(self.tmp_addr().get())
    }
    #[doc = "set the value of operand tmp_addr at 0 to a value of type GPReg (checked by GPR64)"]
    pub fn set_tmp_addr(&self, value: GPReg) {
        let prev_value = self.get_tmp_addr();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.tmp_addr().set(next_value.into_mir());
    }
    #[doc = "operand rd at index 1 of type VFReg"]
    pub fn rd(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand rd at index 1 of type VFReg"]
    pub fn get_rd(&self) -> VFReg {
        VFReg::from_mir(self.rd().get())
    }
    #[doc = "set the value of operand rd at 1 to a value of type VFReg (checked by FPR32)"]
    pub fn set_rd(&self, value: VFReg) {
        let prev_value = self.get_rd();
        let checked_value = FPR32::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.rd().set(next_value.into_mir());
    }
    #[doc = "operand to at index 2 of type MirSymbolOp"]
    pub fn to(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand to at index 2 of type MirSymbolOp"]
    pub fn get_to(&self) -> MirSymbolOp {
        MirSymbolOp::from_mir(self.to().get())
    }
    #[doc = "set the value of operand to at 2 to a value of type MirSymbolOp (checked by MirSymbolOp)"]
    pub fn set_to(&self, value: MirSymbolOp) {
        self.to().set(value.into_mir());
    }
}
impl std::fmt::Debug for MirStrLitF32 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(MirStrLitF32))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("tmp_addr=op[0]", &self.get_tmp_addr())
            .field("rd=op[1]", &self.get_rd())
            .field("to=op[2]", &self.get_to())
            .finish()
    }
}
#[derive(Clone)]
pub struct MirStImm64 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 4usize],
}
impl IMirSubInst for MirStImm64 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..4usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::MirStImm64)
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(MirStImm64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(Imm64::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmLSP64::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::MirStImm64(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::MirStImm64(self)
    }
}
impl MirStImm64 {
    pub fn new(opcode: MirOP, tmpreg: GPR64, imm: Imm64, base: GPR64, offset: ImmLSP64) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(MirStImm64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(tmpreg.clean_use_flags().into_mir()),
                Cell::new(imm.into_mir()),
                Cell::new(base.clean_use_flags().into_mir()),
                Cell::new(offset.into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    #[doc = "operand tmpreg at index 0 of type GPReg"]
    pub fn tmpreg(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand tmpreg at index 0 of type GPReg"]
    pub fn get_tmpreg(&self) -> GPReg {
        GPReg::from_mir(self.tmpreg().get())
    }
    #[doc = "set the value of operand tmpreg at 0 to a value of type GPReg (checked by GPR64)"]
    pub fn set_tmpreg(&self, value: GPReg) {
        let prev_value = self.get_tmpreg();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.tmpreg().set(next_value.into_mir());
    }
    #[doc = "operand imm at index 1 of type Imm64"]
    pub fn imm(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand imm at index 1 of type Imm64"]
    pub fn get_imm(&self) -> Imm64 {
        Imm64::from_mir(self.imm().get())
    }
    #[doc = "set the value of operand imm at 1 to a value of type Imm64 (checked by Imm64)"]
    pub fn set_imm(&self, value: Imm64) {
        self.imm().set(value.into_mir());
    }
    #[doc = "operand base at index 2 of type GPReg"]
    pub fn base(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand base at index 2 of type GPReg"]
    pub fn get_base(&self) -> GPReg {
        GPReg::from_mir(self.base().get())
    }
    #[doc = "set the value of operand base at 2 to a value of type GPReg (checked by GPR64)"]
    pub fn set_base(&self, value: GPReg) {
        let prev_value = self.get_base();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.base().set(next_value.into_mir());
    }
    #[doc = "operand offset at index 3 of type ImmLSP64"]
    pub fn offset(&self) -> &Cell<MirOperand> {
        &self._operands[3usize]
    }
    #[doc = "operand offset at index 3 of type ImmLSP64"]
    pub fn get_offset(&self) -> ImmLSP64 {
        ImmLSP64::from_mir(self.offset().get())
    }
    #[doc = "set the value of operand offset at 3 to a value of type ImmLSP64 (checked by ImmLSP64)"]
    pub fn set_offset(&self, value: ImmLSP64) {
        self.offset().set(value.into_mir());
    }
}
impl std::fmt::Debug for MirStImm64 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(MirStImm64))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("tmpreg=op[0]", &self.get_tmpreg())
            .field("imm=op[1]", &self.get_imm())
            .field("base=op[2]", &self.get_base())
            .field("offset=op[3]", &self.get_offset())
            .finish()
    }
}
#[derive(Clone)]
pub struct MirStSym64 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 4usize],
}
impl IMirSubInst for MirStSym64 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..4usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::MirStSym64)
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(MirStSym64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(MirSymbolOp::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmLSP64::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::MirStSym64(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::MirStSym64(self)
    }
}
impl MirStSym64 {
    pub fn new(
        opcode: MirOP,
        tmpreg: GPR64,
        imm: MirSymbolOp,
        base: GPR64,
        offset: ImmLSP64,
    ) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(MirStSym64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(tmpreg.clean_use_flags().into_mir()),
                Cell::new(imm.into_mir()),
                Cell::new(base.clean_use_flags().into_mir()),
                Cell::new(offset.into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    #[doc = "operand tmpreg at index 0 of type GPReg"]
    pub fn tmpreg(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand tmpreg at index 0 of type GPReg"]
    pub fn get_tmpreg(&self) -> GPReg {
        GPReg::from_mir(self.tmpreg().get())
    }
    #[doc = "set the value of operand tmpreg at 0 to a value of type GPReg (checked by GPR64)"]
    pub fn set_tmpreg(&self, value: GPReg) {
        let prev_value = self.get_tmpreg();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.tmpreg().set(next_value.into_mir());
    }
    #[doc = "operand imm at index 1 of type MirSymbolOp"]
    pub fn imm(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand imm at index 1 of type MirSymbolOp"]
    pub fn get_imm(&self) -> MirSymbolOp {
        MirSymbolOp::from_mir(self.imm().get())
    }
    #[doc = "set the value of operand imm at 1 to a value of type MirSymbolOp (checked by MirSymbolOp)"]
    pub fn set_imm(&self, value: MirSymbolOp) {
        self.imm().set(value.into_mir());
    }
    #[doc = "operand base at index 2 of type GPReg"]
    pub fn base(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand base at index 2 of type GPReg"]
    pub fn get_base(&self) -> GPReg {
        GPReg::from_mir(self.base().get())
    }
    #[doc = "set the value of operand base at 2 to a value of type GPReg (checked by GPR64)"]
    pub fn set_base(&self, value: GPReg) {
        let prev_value = self.get_base();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.base().set(next_value.into_mir());
    }
    #[doc = "operand offset at index 3 of type ImmLSP64"]
    pub fn offset(&self) -> &Cell<MirOperand> {
        &self._operands[3usize]
    }
    #[doc = "operand offset at index 3 of type ImmLSP64"]
    pub fn get_offset(&self) -> ImmLSP64 {
        ImmLSP64::from_mir(self.offset().get())
    }
    #[doc = "set the value of operand offset at 3 to a value of type ImmLSP64 (checked by ImmLSP64)"]
    pub fn set_offset(&self, value: ImmLSP64) {
        self.offset().set(value.into_mir());
    }
}
impl std::fmt::Debug for MirStSym64 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(MirStSym64))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("tmpreg=op[0]", &self.get_tmpreg())
            .field("imm=op[1]", &self.get_imm())
            .field("base=op[2]", &self.get_base())
            .field("offset=op[3]", &self.get_offset())
            .finish()
    }
}
#[derive(Clone)]
pub struct MirStImm32 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 4usize],
}
impl IMirSubInst for MirStImm32 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..4usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::MirStImm32)
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(MirStImm32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(Imm32::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(ImmLSP32::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::MirStImm32(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::MirStImm32(self)
    }
}
impl MirStImm32 {
    pub fn new(opcode: MirOP, tmpreg: GPR64, imm: Imm32, base: GPR64, offset: ImmLSP32) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(MirStImm32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(tmpreg.clean_use_flags().into_mir()),
                Cell::new(imm.into_mir()),
                Cell::new(base.clean_use_flags().into_mir()),
                Cell::new(offset.into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    #[doc = "operand tmpreg at index 0 of type GPReg"]
    pub fn tmpreg(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand tmpreg at index 0 of type GPReg"]
    pub fn get_tmpreg(&self) -> GPReg {
        GPReg::from_mir(self.tmpreg().get())
    }
    #[doc = "set the value of operand tmpreg at 0 to a value of type GPReg (checked by GPR64)"]
    pub fn set_tmpreg(&self, value: GPReg) {
        let prev_value = self.get_tmpreg();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.tmpreg().set(next_value.into_mir());
    }
    #[doc = "operand imm at index 1 of type Imm32"]
    pub fn imm(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand imm at index 1 of type Imm32"]
    pub fn get_imm(&self) -> Imm32 {
        Imm32::from_mir(self.imm().get())
    }
    #[doc = "set the value of operand imm at 1 to a value of type Imm32 (checked by Imm32)"]
    pub fn set_imm(&self, value: Imm32) {
        self.imm().set(value.into_mir());
    }
    #[doc = "operand base at index 2 of type GPReg"]
    pub fn base(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand base at index 2 of type GPReg"]
    pub fn get_base(&self) -> GPReg {
        GPReg::from_mir(self.base().get())
    }
    #[doc = "set the value of operand base at 2 to a value of type GPReg (checked by GPR64)"]
    pub fn set_base(&self, value: GPReg) {
        let prev_value = self.get_base();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.base().set(next_value.into_mir());
    }
    #[doc = "operand offset at index 3 of type ImmLSP32"]
    pub fn offset(&self) -> &Cell<MirOperand> {
        &self._operands[3usize]
    }
    #[doc = "operand offset at index 3 of type ImmLSP32"]
    pub fn get_offset(&self) -> ImmLSP32 {
        ImmLSP32::from_mir(self.offset().get())
    }
    #[doc = "set the value of operand offset at 3 to a value of type ImmLSP32 (checked by ImmLSP32)"]
    pub fn set_offset(&self, value: ImmLSP32) {
        self.offset().set(value.into_mir());
    }
}
impl std::fmt::Debug for MirStImm32 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(MirStImm32))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("tmpreg=op[0]", &self.get_tmpreg())
            .field("imm=op[1]", &self.get_imm())
            .field("base=op[2]", &self.get_base())
            .field("offset=op[3]", &self.get_offset())
            .finish()
    }
}
#[derive(Clone)]
pub struct MirStImm64Sym {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 4usize],
}
impl IMirSubInst for MirStImm64Sym {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..2usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[2usize..4usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::MirStImm64Sym)
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(MirStImm64Sym),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(Imm64::new_empty().into_mir()),
                Cell::new(MirGlobalRef::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::MirStImm64Sym(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::MirStImm64Sym(self)
    }
}
impl MirStImm64Sym {
    pub fn new(
        opcode: MirOP,
        immreg: GPR64,
        addr_reg: GPR64,
        imm: Imm64,
        target: MirGlobalRef,
    ) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(MirStImm64Sym),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(immreg.clean_use_flags().into_mir()),
                Cell::new(addr_reg.clean_use_flags().into_mir()),
                Cell::new(imm.into_mir()),
                Cell::new(target.into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    #[doc = "operand immreg at index 0 of type GPReg"]
    pub fn immreg(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand immreg at index 0 of type GPReg"]
    pub fn get_immreg(&self) -> GPReg {
        GPReg::from_mir(self.immreg().get())
    }
    #[doc = "set the value of operand immreg at 0 to a value of type GPReg (checked by GPR64)"]
    pub fn set_immreg(&self, value: GPReg) {
        let prev_value = self.get_immreg();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.immreg().set(next_value.into_mir());
    }
    #[doc = "operand addr_reg at index 1 of type GPReg"]
    pub fn addr_reg(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand addr_reg at index 1 of type GPReg"]
    pub fn get_addr_reg(&self) -> GPReg {
        GPReg::from_mir(self.addr_reg().get())
    }
    #[doc = "set the value of operand addr_reg at 1 to a value of type GPReg (checked by GPR64)"]
    pub fn set_addr_reg(&self, value: GPReg) {
        let prev_value = self.get_addr_reg();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.addr_reg().set(next_value.into_mir());
    }
    #[doc = "operand imm at index 2 of type Imm64"]
    pub fn imm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand imm at index 2 of type Imm64"]
    pub fn get_imm(&self) -> Imm64 {
        Imm64::from_mir(self.imm().get())
    }
    #[doc = "set the value of operand imm at 2 to a value of type Imm64 (checked by Imm64)"]
    pub fn set_imm(&self, value: Imm64) {
        self.imm().set(value.into_mir());
    }
    #[doc = "operand target at index 3 of type MirGlobalRef"]
    pub fn target(&self) -> &Cell<MirOperand> {
        &self._operands[3usize]
    }
    #[doc = "operand target at index 3 of type MirGlobalRef"]
    pub fn get_target(&self) -> MirGlobalRef {
        MirGlobalRef::from_mir(self.target().get())
    }
    #[doc = "set the value of operand target at 3 to a value of type MirGlobalRef (checked by MirGlobalRef)"]
    pub fn set_target(&self, value: MirGlobalRef) {
        self.target().set(value.into_mir());
    }
}
impl std::fmt::Debug for MirStImm64Sym {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(MirStImm64Sym))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("immreg=op[0]", &self.get_immreg())
            .field("addr_reg=op[1]", &self.get_addr_reg())
            .field("imm=op[2]", &self.get_imm())
            .field("target=op[3]", &self.get_target())
            .finish()
    }
}
#[derive(Clone)]
pub struct MirStImm32Sym {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 4usize],
}
impl IMirSubInst for MirStImm32Sym {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..2usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[2usize..4usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::MirStImm32Sym)
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(MirStImm32Sym),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(Imm32::new_empty().into_mir()),
                Cell::new(MirGlobalRef::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::MirStImm32Sym(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::MirStImm32Sym(self)
    }
}
impl MirStImm32Sym {
    pub fn new(
        opcode: MirOP,
        immreg: GPR64,
        addr_reg: GPR64,
        imm: Imm32,
        target: MirGlobalRef,
    ) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(MirStImm32Sym),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(immreg.clean_use_flags().into_mir()),
                Cell::new(addr_reg.clean_use_flags().into_mir()),
                Cell::new(imm.into_mir()),
                Cell::new(target.into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    #[doc = "operand immreg at index 0 of type GPReg"]
    pub fn immreg(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand immreg at index 0 of type GPReg"]
    pub fn get_immreg(&self) -> GPReg {
        GPReg::from_mir(self.immreg().get())
    }
    #[doc = "set the value of operand immreg at 0 to a value of type GPReg (checked by GPR64)"]
    pub fn set_immreg(&self, value: GPReg) {
        let prev_value = self.get_immreg();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.immreg().set(next_value.into_mir());
    }
    #[doc = "operand addr_reg at index 1 of type GPReg"]
    pub fn addr_reg(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand addr_reg at index 1 of type GPReg"]
    pub fn get_addr_reg(&self) -> GPReg {
        GPReg::from_mir(self.addr_reg().get())
    }
    #[doc = "set the value of operand addr_reg at 1 to a value of type GPReg (checked by GPR64)"]
    pub fn set_addr_reg(&self, value: GPReg) {
        let prev_value = self.get_addr_reg();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.addr_reg().set(next_value.into_mir());
    }
    #[doc = "operand imm at index 2 of type Imm32"]
    pub fn imm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand imm at index 2 of type Imm32"]
    pub fn get_imm(&self) -> Imm32 {
        Imm32::from_mir(self.imm().get())
    }
    #[doc = "set the value of operand imm at 2 to a value of type Imm32 (checked by Imm32)"]
    pub fn set_imm(&self, value: Imm32) {
        self.imm().set(value.into_mir());
    }
    #[doc = "operand target at index 3 of type MirGlobalRef"]
    pub fn target(&self) -> &Cell<MirOperand> {
        &self._operands[3usize]
    }
    #[doc = "operand target at index 3 of type MirGlobalRef"]
    pub fn get_target(&self) -> MirGlobalRef {
        MirGlobalRef::from_mir(self.target().get())
    }
    #[doc = "set the value of operand target at 3 to a value of type MirGlobalRef (checked by MirGlobalRef)"]
    pub fn set_target(&self, value: MirGlobalRef) {
        self.target().set(value.into_mir());
    }
}
impl std::fmt::Debug for MirStImm32Sym {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(MirStImm32Sym))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("immreg=op[0]", &self.get_immreg())
            .field("addr_reg=op[1]", &self.get_addr_reg())
            .field("imm=op[2]", &self.get_imm())
            .field("target=op[3]", &self.get_target())
            .finish()
    }
}
#[derive(Clone)]
pub struct MirStSym64Sym {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 4usize],
}
impl IMirSubInst for MirStSym64Sym {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..2usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[2usize..4usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::MirStSymSym)
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(MirStSym64Sym),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(MirSymbolOp::new_empty().into_mir()),
                Cell::new(MirGlobalRef::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::MirStSym64Sym(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::MirStSym64Sym(self)
    }
}
impl MirStSym64Sym {
    pub fn new(
        opcode: MirOP,
        immreg: GPR64,
        addr_reg: GPR64,
        imm: MirSymbolOp,
        target: MirGlobalRef,
    ) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(MirStSym64Sym),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(immreg.clean_use_flags().into_mir()),
                Cell::new(addr_reg.clean_use_flags().into_mir()),
                Cell::new(imm.into_mir()),
                Cell::new(target.into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    #[doc = "operand immreg at index 0 of type GPReg"]
    pub fn immreg(&self) -> &Cell<MirOperand> {
        &self._operands[0usize]
    }
    #[doc = "operand immreg at index 0 of type GPReg"]
    pub fn get_immreg(&self) -> GPReg {
        GPReg::from_mir(self.immreg().get())
    }
    #[doc = "set the value of operand immreg at 0 to a value of type GPReg (checked by GPR64)"]
    pub fn set_immreg(&self, value: GPReg) {
        let prev_value = self.get_immreg();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.immreg().set(next_value.into_mir());
    }
    #[doc = "operand addr_reg at index 1 of type GPReg"]
    pub fn addr_reg(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand addr_reg at index 1 of type GPReg"]
    pub fn get_addr_reg(&self) -> GPReg {
        GPReg::from_mir(self.addr_reg().get())
    }
    #[doc = "set the value of operand addr_reg at 1 to a value of type GPReg (checked by GPR64)"]
    pub fn set_addr_reg(&self, value: GPReg) {
        let prev_value = self.get_addr_reg();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.addr_reg().set(next_value.into_mir());
    }
    #[doc = "operand imm at index 2 of type MirSymbolOp"]
    pub fn imm(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand imm at index 2 of type MirSymbolOp"]
    pub fn get_imm(&self) -> MirSymbolOp {
        MirSymbolOp::from_mir(self.imm().get())
    }
    #[doc = "set the value of operand imm at 2 to a value of type MirSymbolOp (checked by MirSymbolOp)"]
    pub fn set_imm(&self, value: MirSymbolOp) {
        self.imm().set(value.into_mir());
    }
    #[doc = "operand target at index 3 of type MirGlobalRef"]
    pub fn target(&self) -> &Cell<MirOperand> {
        &self._operands[3usize]
    }
    #[doc = "operand target at index 3 of type MirGlobalRef"]
    pub fn get_target(&self) -> MirGlobalRef {
        MirGlobalRef::from_mir(self.target().get())
    }
    #[doc = "set the value of operand target at 3 to a value of type MirGlobalRef (checked by MirGlobalRef)"]
    pub fn set_target(&self, value: MirGlobalRef) {
        self.target().set(value.into_mir());
    }
}
impl std::fmt::Debug for MirStSym64Sym {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(MirStSym64Sym))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("immreg=op[0]", &self.get_immreg())
            .field("addr_reg=op[1]", &self.get_addr_reg())
            .field("imm=op[2]", &self.get_imm())
            .field("target=op[3]", &self.get_target())
            .finish()
    }
}
#[derive(Clone)]
pub struct LoadConst64 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for LoadConst64 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(LoadConst64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(Imm64::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::LoadConst64(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::LoadConst64(self)
    }
}
impl LoadConst64 {
    pub fn new(opcode: MirOP, rd: GPR64, src: Imm64) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(LoadConst64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(rd.clean_use_flags().into_mir()), Cell::new(src.into_mir())],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for LoadConst64 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(LoadConst64))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("src=op[1]", &self.get_src())
            .finish()
    }
}
#[derive(Clone)]
pub struct LoadConst64Symbol {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for LoadConst64Symbol {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(LoadConst64Symbol),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(MirSymbolOp::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::LoadConst64Symbol(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::LoadConst64Symbol(self)
    }
}
impl LoadConst64Symbol {
    pub fn new(opcode: MirOP, rd: GPR64, src: MirSymbolOp) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(LoadConst64Symbol),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(rd.clean_use_flags().into_mir()), Cell::new(src.into_mir())],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for LoadConst64Symbol {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(LoadConst64Symbol))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("src=op[1]", &self.get_src())
            .finish()
    }
}
#[derive(Clone)]
pub struct MirLdImmF64 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for MirLdImmF64 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..2usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[2usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::MirLdImmF64)
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(MirLdImmF64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(Imm64::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::MirLdImmF64(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::MirLdImmF64(self)
    }
}
impl MirLdImmF64 {
    pub fn new(opcode: MirOP, rd: FPR64, tmpreg: GPR64, src: Imm64) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(MirLdImmF64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(tmpreg.clean_use_flags().into_mir()),
                Cell::new(src.into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
    #[doc = "operand tmpreg at index 1 of type GPReg"]
    pub fn tmpreg(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand tmpreg at index 1 of type GPReg"]
    pub fn get_tmpreg(&self) -> GPReg {
        GPReg::from_mir(self.tmpreg().get())
    }
    #[doc = "set the value of operand tmpreg at 1 to a value of type GPReg (checked by GPR64)"]
    pub fn set_tmpreg(&self, value: GPReg) {
        let prev_value = self.get_tmpreg();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.tmpreg().set(next_value.into_mir());
    }
    #[doc = "operand src at index 2 of type Imm64"]
    pub fn src(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand src at index 2 of type Imm64"]
    pub fn get_src(&self) -> Imm64 {
        Imm64::from_mir(self.src().get())
    }
    #[doc = "set the value of operand src at 2 to a value of type Imm64 (checked by Imm64)"]
    pub fn set_src(&self, value: Imm64) {
        self.src().set(value.into_mir());
    }
}
impl std::fmt::Debug for MirLdImmF64 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(MirLdImmF64))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("tmpreg=op[1]", &self.get_tmpreg())
            .field("src=op[2]", &self.get_src())
            .finish()
    }
}
#[derive(Clone)]
pub struct MirLdImmF32 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3usize],
}
impl IMirSubInst for MirLdImmF32 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..2usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[2usize..3usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::MirLdImmF32)
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(MirLdImmF32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(Imm32::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::MirLdImmF32(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::MirLdImmF32(self)
    }
}
impl MirLdImmF32 {
    pub fn new(opcode: MirOP, rd: FPR32, tmpreg: GPR64, src: Imm32) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(MirLdImmF32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(tmpreg.clean_use_flags().into_mir()),
                Cell::new(src.into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
    #[doc = "operand tmpreg at index 1 of type GPReg"]
    pub fn tmpreg(&self) -> &Cell<MirOperand> {
        &self._operands[1usize]
    }
    #[doc = "operand tmpreg at index 1 of type GPReg"]
    pub fn get_tmpreg(&self) -> GPReg {
        GPReg::from_mir(self.tmpreg().get())
    }
    #[doc = "set the value of operand tmpreg at 1 to a value of type GPReg (checked by GPR64)"]
    pub fn set_tmpreg(&self, value: GPReg) {
        let prev_value = self.get_tmpreg();
        let checked_value = GPR64::from_real(value);
        let next_value = checked_value.insert_to_real(prev_value);
        self.tmpreg().set(next_value.into_mir());
    }
    #[doc = "operand src at index 2 of type Imm32"]
    pub fn src(&self) -> &Cell<MirOperand> {
        &self._operands[2usize]
    }
    #[doc = "operand src at index 2 of type Imm32"]
    pub fn get_src(&self) -> Imm32 {
        Imm32::from_mir(self.src().get())
    }
    #[doc = "set the value of operand src at 2 to a value of type Imm32 (checked by Imm32)"]
    pub fn set_src(&self, value: Imm32) {
        self.src().set(value.into_mir());
    }
}
impl std::fmt::Debug for MirLdImmF32 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(MirLdImmF32))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("tmpreg=op[1]", &self.get_tmpreg())
            .field("src=op[2]", &self.get_src())
            .finish()
    }
}
#[derive(Clone)]
pub struct CSel64 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 4usize],
    cond: MirCondFlag,
}
impl IMirSubInst for CSel64 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
            MirOP::CSel64 | MirOP::CSInc64 | MirOP::CSInv64 | MirOP::CSNeg64
        )
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(CSel64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(PState::new_empty().into_mir()),
            ],
            cond: MirCondFlag::AL,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::CSel64(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::CSel64(self)
    }
}
impl CSel64 {
    pub fn new(
        opcode: MirOP,
        rd: GPR64,
        rn: GPR64,
        rm: GPR64,
        csr: PState,
        cond: MirCondFlag,
    ) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(CSel64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rm.clean_use_flags().into_mir()),
                Cell::new(csr.into_mir()),
            ],
            cond,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
    #[doc = "operand csr at index 3 of type PState"]
    pub fn csr(&self) -> &Cell<MirOperand> {
        &self._operands[3usize]
    }
    #[doc = "operand csr at index 3 of type PState"]
    pub fn get_csr(&self) -> PState {
        PState::from_mir(self.csr().get())
    }
    #[doc = "set the value of operand csr at 3 to a value of type PState (checked by PState)"]
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
impl std::fmt::Debug for CSel64 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(CSel64))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("rn=op[1]", &self.get_rn())
            .field("rm=op[2]", &self.get_rm())
            .field("csr=op[3]", &self.get_csr())
            .field("cond", &self.cond)
            .finish()
    }
}
#[derive(Clone)]
pub struct CSel32 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 4usize],
    cond: MirCondFlag,
}
impl IMirSubInst for CSel32 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
            MirOP::CSel32 | MirOP::CSInc32 | MirOP::CSInv32 | MirOP::CSNeg32
        )
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(CSel32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(PState::new_empty().into_mir()),
            ],
            cond: MirCondFlag::AL,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::CSel32(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::CSel32(self)
    }
}
impl CSel32 {
    pub fn new(
        opcode: MirOP,
        rd: GPR32,
        rn: GPR32,
        rm: GPR32,
        csr: PState,
        cond: MirCondFlag,
    ) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(CSel32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rm.clean_use_flags().into_mir()),
                Cell::new(csr.into_mir()),
            ],
            cond,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
    #[doc = "operand csr at index 3 of type PState"]
    pub fn csr(&self) -> &Cell<MirOperand> {
        &self._operands[3usize]
    }
    #[doc = "operand csr at index 3 of type PState"]
    pub fn get_csr(&self) -> PState {
        PState::from_mir(self.csr().get())
    }
    #[doc = "set the value of operand csr at 3 to a value of type PState (checked by PState)"]
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
impl std::fmt::Debug for CSel32 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(CSel32))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("rn=op[1]", &self.get_rn())
            .field("rm=op[2]", &self.get_rm())
            .field("csr=op[3]", &self.get_csr())
            .field("cond", &self.cond)
            .finish()
    }
}
#[derive(Clone)]
pub struct CSelF64 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 4usize],
    cond: MirCondFlag,
}
impl IMirSubInst for CSelF64 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..4usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::CSelF64)
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(CSelF64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(PState::new_empty().into_mir()),
            ],
            cond: MirCondFlag::AL,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::CSelF64(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::CSelF64(self)
    }
}
impl CSelF64 {
    pub fn new(
        opcode: MirOP,
        rd: FPR64,
        rn: FPR64,
        rm: FPR64,
        csr: PState,
        cond: MirCondFlag,
    ) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(CSelF64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rm.clean_use_flags().into_mir()),
                Cell::new(csr.into_mir()),
            ],
            cond,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
    #[doc = "operand csr at index 3 of type PState"]
    pub fn csr(&self) -> &Cell<MirOperand> {
        &self._operands[3usize]
    }
    #[doc = "operand csr at index 3 of type PState"]
    pub fn get_csr(&self) -> PState {
        PState::from_mir(self.csr().get())
    }
    #[doc = "set the value of operand csr at 3 to a value of type PState (checked by PState)"]
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
impl std::fmt::Debug for CSelF64 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(CSelF64))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("rn=op[1]", &self.get_rn())
            .field("rm=op[2]", &self.get_rm())
            .field("csr=op[3]", &self.get_csr())
            .field("cond", &self.cond)
            .finish()
    }
}
#[derive(Clone)]
pub struct CSelF32 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 4usize],
    cond: MirCondFlag,
}
impl IMirSubInst for CSelF32 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..4usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::CSelF32)
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(CSelF32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(VFReg::new_empty().into_mir()),
                Cell::new(PState::new_empty().into_mir()),
            ],
            cond: MirCondFlag::AL,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::CSelF32(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::CSelF32(self)
    }
}
impl CSelF32 {
    pub fn new(
        opcode: MirOP,
        rd: FPR32,
        rn: FPR32,
        rm: FPR32,
        csr: PState,
        cond: MirCondFlag,
    ) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(CSelF32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(rd.clean_use_flags().into_mir()),
                Cell::new(rn.clean_use_flags().into_mir()),
                Cell::new(rm.clean_use_flags().into_mir()),
                Cell::new(csr.into_mir()),
            ],
            cond,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
    #[doc = "operand csr at index 3 of type PState"]
    pub fn csr(&self) -> &Cell<MirOperand> {
        &self._operands[3usize]
    }
    #[doc = "operand csr at index 3 of type PState"]
    pub fn get_csr(&self) -> PState {
        PState::from_mir(self.csr().get())
    }
    #[doc = "set the value of operand csr at 3 to a value of type PState (checked by PState)"]
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
impl std::fmt::Debug for CSelF32 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(CSelF32))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("rn=op[1]", &self.get_rn())
            .field("rm=op[2]", &self.get_rm())
            .field("csr=op[3]", &self.get_csr())
            .field("cond", &self.cond)
            .finish()
    }
}
#[derive(Clone)]
pub struct CSet64 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
    cond: MirCondFlag,
}
impl IMirSubInst for CSet64 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..2usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::CSet64)
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(CSet64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(PState::new_empty().into_mir()),
            ],
            cond: MirCondFlag::AL,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::CSet64(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::CSet64(self)
    }
}
impl CSet64 {
    pub fn new(opcode: MirOP, rd: GPR64, csr: PState, cond: MirCondFlag) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(CSet64),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(rd.clean_use_flags().into_mir()), Cell::new(csr.into_mir())],
            cond,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for CSet64 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(CSet64))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("csr=op[1]", &self.get_csr())
            .field("cond", &self.cond)
            .finish()
    }
}
#[derive(Clone)]
pub struct CSet32 {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
    cond: MirCondFlag,
}
impl IMirSubInst for CSet32 {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..1usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[1usize..2usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::CSet32)
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(CSet32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(PState::new_empty().into_mir()),
            ],
            cond: MirCondFlag::AL,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::CSet32(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::CSet32(self)
    }
}
impl CSet32 {
    pub fn new(opcode: MirOP, rd: GPR32, csr: PState, cond: MirCondFlag) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(CSet32),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(rd.clean_use_flags().into_mir()), Cell::new(csr.into_mir())],
            cond,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for CSet32 {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(CSet32))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("rd=op[0]", &self.get_rd())
            .field("csr=op[1]", &self.get_csr())
            .field("cond", &self.cond)
            .finish()
    }
}
#[derive(Clone)]
pub struct CondBr {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
    cond: MirCondFlag,
}
impl IMirSubInst for CondBr {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
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
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(CondBr),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(MirBlockRef::new_empty().into_mir()),
                Cell::new(PState::new_empty().into_mir()),
            ],
            cond: MirCondFlag::AL,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::CondBr(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::CondBr(self)
    }
}
impl CondBr {
    pub fn new(opcode: MirOP, label: MirBlockRef, csr: PState, cond: MirCondFlag) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(CondBr),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(label.into_mir()), Cell::new(csr.into_mir())],
            cond,
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for CondBr {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(CondBr))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("label=op[0]", &self.get_label())
            .field("csr=op[1]", &self.get_csr())
            .field("cond", &self.cond)
            .finish()
    }
}
#[derive(Clone)]
pub struct CBZs {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2usize],
}
impl IMirSubInst for CBZs {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..0usize]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[0usize..2usize]
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::CBZ | MirOP::CBNZ)
    }
    fn new_empty(opcode: MirOP) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(CBZs),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(GPReg::new_empty().into_mir()),
                Cell::new(MirBlockRef::new_empty().into_mir()),
            ],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::CBZs(inst) => Some(inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::CBZs(self)
    }
}
impl CBZs {
    pub fn new(opcode: MirOP, cond: GPR64, target: MirBlockRef) -> Self {
        if !Self::accepts_opcode(opcode) {
            panic!(
                "Cannot create an instance of {} with opcode {:?}",
                stringify!(CBZs),
                opcode
            );
        }
        let ret = Self {
            _common: MirInstCommon::new(opcode),
            _operands: [Cell::new(cond.clean_use_flags().into_mir()), Cell::new(target.into_mir())],
        };
        super::utils::mark_out_operands_defined(ret.out_operands());
        super::utils::mark_in_operands_used(ret.in_operands());
        ret
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
impl std::fmt::Debug for CBZs {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        let node_head = self._common.node_head.get();
        let prev = node_head.prev;
        let next = node_head.next;
        let opcode = self._common.opcode;
        f.debug_struct(stringify!(CBZs))
            .field("opcode", &opcode)
            .field("prev", &prev)
            .field("next", &next)
            .field("cond=op[0]", &self.get_cond())
            .field("target=op[1]", &self.get_target())
            .finish()
    }
}

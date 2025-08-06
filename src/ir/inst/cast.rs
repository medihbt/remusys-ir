use crate::{
    ir::{
        IRAllocs, ISubInst, ISubValueSSA, InstCommon, InstData, InstKind, InstRef, Opcode, Use,
        UseKind, ValueSSA,
        inst::{ISubInstRef, InstOperands},
    },
    typing::id::ValTypeID,
};
use std::rc::Rc;

#[derive(Debug)]
pub struct CastOp {
    common: InstCommon,
    fromop: [Rc<Use>; 1],
    pub fromty: ValTypeID, // 源类型
}

impl ISubInst for CastOp {
    fn new_empty(opcode: Opcode) -> Self {
        Self {
            common: InstCommon::new(opcode, ValTypeID::Void),
            fromop: [Use::new(UseKind::CastOpFrom)],
            fromty: ValTypeID::Void, // 初始类型为 Void
        }
    }
    fn try_from_ir(inst: &InstData) -> Option<&Self> {
        if let InstData::Cast(cast) = inst { Some(cast) } else { None }
    }
    fn try_from_ir_mut(inst: &mut InstData) -> Option<&mut Self> {
        match inst {
            InstData::Cast(cast) => Some(cast),
            _ => None,
        }
    }
    fn into_ir(self) -> InstData {
        InstData::Cast(self)
    }
    fn get_common(&self) -> &InstCommon {
        &self.common
    }
    fn common_mut(&mut self) -> &mut InstCommon {
        &mut self.common
    }
    fn is_terminator(&self) -> bool {
        false
    }
    fn get_operands(&self) -> InstOperands {
        InstOperands::Fixed(&self.fromop)
    }
    fn operands_mut(&mut self) -> &mut [Rc<Use>] {
        &mut self.fromop
    }
}

impl CastOp {
    pub fn new_raw(opcode: Opcode, fromty: ValTypeID, to_ty: ValTypeID) -> Self {
        assert_eq!(opcode.get_kind(), InstKind::Cast);
        Self {
            common: InstCommon::new(opcode, to_ty),
            fromop: [Use::new(UseKind::CastOpFrom)],
            fromty,
        }
    }

    pub fn new(allocs: &IRAllocs, opcode: Opcode, to_ty: ValTypeID, from: ValueSSA) -> Self {
        let cast_op = Self::new_raw(opcode, from.get_valtype(allocs), to_ty);
        cast_op.fromop[0].set_operand(allocs, from);
        cast_op
    }

    pub fn get_to_type(&self) -> ValTypeID {
        self.get_common().ret_type
    }

    pub fn get_from(&self) -> ValueSSA {
        self.fromop[0].get_operand()
    }
    pub fn set_from(&mut self, allocs: &IRAllocs, from: ValueSSA) {
        if self.fromty != from.get_valtype(allocs) {
            let fromty = self.fromty;
            let new_fromty = from.get_valtype(allocs);
            panic!("Type mismatch: expected {fromty:?}, got {new_fromty:?}");
        }
        self.fromop[0].set_operand(allocs, from);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CastOpRef(InstRef);

impl ISubInstRef for CastOpRef {
    type InstDataT = CastOp;

    fn from_raw_nocheck(inst_ref: InstRef) -> Self {
        CastOpRef(inst_ref)
    }
    fn into_raw(self) -> InstRef {
        self.0
    }
}

use crate::{
    ir::{
        IRAllocs, IRWriter, ISubInst, ISubValueSSA, IUser, InstCommon, InstData, InstRef, Opcode,
        OperandSet, Use, UseKind, ValueSSA, inst::ISubInstRef,
    },
    typing::ValTypeID,
};
use std::rc::Rc;

/// 选择指令
///
/// ### LLVM IR 语法
///
/// ```llvm
/// %<name> = select <type>, i1 <cond>, <true value>, <false value>
/// ```
#[derive(Debug)]
pub struct SelectOp {
    common: InstCommon,
    operands: [Rc<Use>; 3],
}

impl IUser for SelectOp {
    fn get_operands(&self) -> OperandSet<'_> {
        OperandSet::Fixed(&self.operands)
    }
    fn operands_mut(&mut self) -> &mut [Rc<Use>] {
        &mut self.operands
    }
}

impl ISubInst for SelectOp {
    fn new_empty(_: Opcode) -> Self {
        Self {
            common: InstCommon::new(Opcode::Select, ValTypeID::Void),
            operands: [
                Use::new(UseKind::SelectCond),
                Use::new(UseKind::SelectTrue),
                Use::new(UseKind::SelectFalse),
            ],
        }
    }
    fn try_from_ir(inst: &InstData) -> Option<&Self> {
        if let InstData::Select(select_op) = inst { Some(select_op) } else { None }
    }
    fn try_from_ir_mut(inst: &mut InstData) -> Option<&mut Self> {
        if let InstData::Select(select_op) = inst { Some(select_op) } else { None }
    }
    fn into_ir(self) -> InstData {
        InstData::Select(self)
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

    fn fmt_ir(&self, id: Option<usize>, writer: &IRWriter) -> std::io::Result<()> {
        let Some(id) = id else {
            use std::io::{Error, ErrorKind::InvalidInput};
            return Err(Error::new(InvalidInput, "ID must be provided for CastOp"));
        };
        write!(writer, "%{id} = select ")?;
        writer.write_type(self.get_valtype())?;

        writer.write_str(", i1 ")?;
        writer.write_operand(self.get_cond())?;

        writer.write_str(", ")?;
        writer.write_operand(self.get_true_val())?;

        writer.write_str(", ")?;
        writer.write_operand(self.get_false_val())
    }
}

impl SelectOp {
    pub fn new_raw(ret_type: ValTypeID) -> Self {
        Self {
            common: InstCommon::new(Opcode::Select, ret_type),
            operands: [
                Use::new(UseKind::SelectCond),
                Use::new(UseKind::SelectTrue),
                Use::new(UseKind::SelectFalse),
            ],
        }
    }

    pub fn new(allocs: &IRAllocs, cond: ValueSSA, true_val: ValueSSA, false_val: ValueSSA) -> Self {
        let ret_type = Self::do_check_operands(allocs, &cond, &true_val, &false_val).unwrap();
        let select_op = Self::new_raw(ret_type);
        select_op.operands[0].set_operand(allocs, cond);
        select_op.operands[1].set_operand(allocs, true_val);
        select_op.operands[2].set_operand(allocs, false_val);
        select_op
    }

    fn do_check_operands(
        allocs: &IRAllocs,
        cond: &ValueSSA,
        true_val: &ValueSSA,
        false_val: &ValueSSA,
    ) -> Result<ValTypeID, String> {
        let ValTypeID::Int(1) = cond.get_valtype(allocs) else {
            return Err("SelectOp condition must be a boolean type".into());
        };
        let true_ty = true_val.get_valtype(allocs);
        let false_ty = false_val.get_valtype(allocs);
        if true_ty != false_ty {
            return Err(format!(
                "SelectOp true and false values must have the same type: {true_ty:?} != {false_ty:?}"
            ));
        }
        let ret_ty = match true_ty {
            ValTypeID::Int(_) | ValTypeID::Float(_) | ValTypeID::Ptr => true_ty,
            ValTypeID::Struct(_) | ValTypeID::Array(_) => true_ty,
            _ => {
                return Err(format!("SelectOp does not support this type: {true_ty:?}",));
            }
        };
        Ok(ret_ty)
    }

    pub fn check(&self, allocs: &IRAllocs) -> Result<(), String> {
        let ret_ty = Self::do_check_operands(
            allocs, // 使用默认的 IRAllocs 进行检查
            &self.operands[0].get_operand(),
            &self.operands[1].get_operand(),
            &self.operands[2].get_operand(),
        )?;
        if ret_ty == self.common.ret_type {
            Ok(())
        } else {
            Err(format!(
                "SelectOp return type mismatch: expected {:?}, got {:?}",
                self.common.ret_type, ret_ty
            ))
        }
    }

    pub fn cond_use(&self) -> &Rc<Use> {
        &self.operands[0]
    }
    pub fn true_use(&self) -> &Rc<Use> {
        &self.operands[1]
    }
    pub fn false_use(&self) -> &Rc<Use> {
        &self.operands[2]
    }

    pub fn get_cond(&self) -> ValueSSA {
        self.operands[0].get_operand()
    }
    pub fn get_true_val(&self) -> ValueSSA {
        self.operands[1].get_operand()
    }
    pub fn get_false_val(&self) -> ValueSSA {
        self.operands[2].get_operand()
    }

    pub fn set_cond(&self, allocs: &IRAllocs, cond: ValueSSA) {
        self.operands[0].set_operand(allocs, cond);
    }
    pub fn set_true_val(&self, allocs: &IRAllocs, true_val: ValueSSA) {
        self.operands[1].set_operand(allocs, true_val);
    }
    pub fn set_false_val(&self, allocs: &IRAllocs, false_val: ValueSSA) {
        self.operands[2].set_operand(allocs, false_val);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct SelectOpRef(InstRef);

impl ISubInstRef for SelectOpRef {
    type InstDataT = SelectOp;

    fn from_raw_nocheck(inst_ref: InstRef) -> Self {
        SelectOpRef(inst_ref)
    }
    fn into_raw(self) -> InstRef {
        self.0
    }
}

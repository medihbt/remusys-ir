use std::rc::Rc;

use crate::{
    ir::{
        IRAllocs, IRWriter, ISubInst, ISubValueSSA, IUser, InstCommon, InstData, InstKind, InstRef,
        Opcode, OperandSet, Use, UseKind, ValueSSA,
        checking::{self, ValueCheckError},
        inst::ISubInstRef,
    },
    typing::{ValTypeClass, ValTypeID},
};

/// 二元操作指令: 执行两个操作数的二元运算（算术运算、逻辑运算、移位运算），并返回结果。
///
/// ### LLVM 语法
///
/// ```llvm
/// %<result> = <opcode> <ty> <op1>, <op2>
/// ```
#[derive(Debug)]
pub struct BinOp {
    common: InstCommon,
    operands: [Rc<Use>; 2],
}

impl IUser for BinOp {
    fn get_operands(&self) -> OperandSet {
        OperandSet::Fixed(&self.operands)
    }
    fn operands_mut(&mut self) -> &mut [Rc<Use>] {
        &mut self.operands
    }
}

impl ISubInst for BinOp {
    fn new_empty(opcode: Opcode) -> Self {
        Self {
            common: InstCommon::new(opcode, ValTypeID::Void),
            operands: [Use::new(UseKind::BinOpLhs), Use::new(UseKind::BinOpRhs)],
        }
    }
    fn try_from_ir(inst: &InstData) -> Option<&Self> {
        if let InstData::BinOp(binop) = inst { Some(binop) } else { None }
    }
    fn try_from_ir_mut(inst: &mut InstData) -> Option<&mut Self> {
        if let InstData::BinOp(binop) = inst { Some(binop) } else { None }
    }
    fn into_ir(self) -> InstData {
        InstData::BinOp(self)
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
        let Some(id) = id else { panic!("Tried to format BinOp without an ID") };
        write!(writer, "%{} = {} ", id, self.get_opcode().to_string())?;
        writer.write_type(self.common.ret_type)?;
        writer.write_str(" ")?;
        writer.write_operand(self.get_lhs())?;
        writer.write_str(", ")?;
        writer.write_operand(self.get_rhs())
    }
}

impl BinOp {
    pub fn new_raw(opcode: Opcode, ret_type: ValTypeID) -> Self {
        Self {
            common: InstCommon::new(opcode, ret_type),
            operands: [Use::new(UseKind::BinOpLhs), Use::new(UseKind::BinOpRhs)],
        }
    }
    pub fn new(allocs: &IRAllocs, opcode: Opcode, lhs: ValueSSA, rhs: ValueSSA) -> Self {
        let binop = Self::new_raw(opcode, lhs.get_valtype(allocs));
        binop.operands[0].set_operand(allocs, lhs);
        binop.operands[1].set_operand(allocs, rhs);
        binop
    }

    pub fn accepts_opcode(opcode: Opcode) -> bool {
        matches!(opcode.get_kind(), InstKind::BinOp)
    }

    pub fn lhs(&self) -> &Rc<Use> {
        &self.operands[0]
    }
    pub fn get_lhs(&self) -> ValueSSA {
        self.lhs().get_operand()
    }
    pub fn set_lhs(&self, allocs: &IRAllocs, lhs: ValueSSA) {
        self.lhs().set_operand(allocs, lhs);
    }

    pub fn rhs(&self) -> &Rc<Use> {
        &self.operands[1]
    }
    pub fn get_rhs(&self) -> ValueSSA {
        self.rhs().get_operand()
    }
    pub fn set_rhs(&self, allocs: &IRAllocs, rhs: ValueSSA) {
        self.rhs().set_operand(allocs, rhs);
    }

    pub fn check_operands(&self, allocs: &IRAllocs) {
        self.validate(allocs).unwrap()
    }
    pub fn validate(&self, allocs: &IRAllocs) -> Result<(), ValueCheckError> {
        Self::do_validate_operands(
            self.common.opcode,
            self.common.ret_type,
            self.get_lhs(),
            self.get_rhs(),
            allocs,
        )
    }

    fn do_validate_operands(
        opcode: Opcode,
        retty: ValTypeID,
        lhs: ValueSSA,
        rhs: ValueSSA,
        allocs: &IRAllocs,
    ) -> Result<(), ValueCheckError> {
        match opcode {
            Opcode::Add | Opcode::Sub | Opcode::Mul => {
                if !retty.isclass_or_vec(ValTypeClass::Int) {
                    return Err(ValueCheckError::TypeNotClass(retty, ValTypeClass::Int));
                }
                checking::type_matches(retty, lhs, allocs)?;
                checking::type_matches(retty, rhs, allocs)
            }
            Opcode::Fadd | Opcode::Fsub | Opcode::Fmul | Opcode::Fdiv | Opcode::Frem => {
                if !retty.isclass_or_vec(ValTypeClass::Float) {
                    return Err(ValueCheckError::TypeNotClass(retty, ValTypeClass::Float));
                }
                checking::type_matches(retty, lhs, allocs)?;
                checking::type_matches(retty, rhs, allocs)
            }
            Opcode::Sdiv | Opcode::Udiv | Opcode::Srem | Opcode::Urem => {
                if !retty.isclass_or_vec(ValTypeClass::Int) {
                    return Err(ValueCheckError::TypeNotClass(retty, ValTypeClass::Int));
                }
                checking::type_matches(retty, lhs, allocs)?;
                checking::type_matches(retty, rhs, allocs)?;
                if let ValueSSA::ConstData(x) = rhs {
                    if x.is_zero() {
                        return Err(ValueCheckError::InvalidZeroOP(
                            x.into_ir(),
                            opcode,
                            UseKind::BinOpRhs,
                        ));
                    }
                }
                Ok(())
            }
            Opcode::BitAnd | Opcode::BitOr | Opcode::BitXor => {
                if !retty.isclass_or_vec(ValTypeClass::Int) {
                    return Err(ValueCheckError::TypeNotClass(retty, ValTypeClass::Int));
                }
                checking::type_matches(retty, lhs, allocs)?;
                checking::type_matches(retty, rhs, allocs)?;
                Ok(())
            }
            Opcode::Shl | Opcode::Lshr | Opcode::Ashr => {
                if !retty.isclass_or_vec(ValTypeClass::Int) {
                    return Err(ValueCheckError::TypeNotClass(retty, ValTypeClass::Int));
                }
                checking::type_matches(retty, lhs, allocs)?;
                checking::type_matches(retty, rhs, allocs)?;
                Ok(())
            }
            _ => {
                if opcode.is_binary_op() {
                    panic!("Binary operation {:?} is not implemented", opcode);
                } else {
                    panic!("Opcode {:?} is not a binary operation", opcode);
                }
            }
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct BinOpRef(InstRef);

impl ISubInstRef for BinOpRef {
    type InstDataT = BinOp;
    fn from_raw_nocheck(inst_ref: InstRef) -> Self {
        Self(inst_ref)
    }
    fn into_raw(self) -> InstRef {
        self.0
    }
}

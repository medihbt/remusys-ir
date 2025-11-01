use crate::{
    impl_traceable_from_common,
    ir::{
        IFuncUniqueUser, IPtrUniqueUser, IPtrValue, IRAllocs, ISubInst, ISubInstID, IUser,
        InstCommon, InstID, InstObj, JumpTargets, Opcode, OperandSet, UseID, UseKind,
    },
    typing::{FuncTypeID, IValType, ValTypeID},
};
use std::{cell::Cell, ops::RangeFrom};

/// 函数调用指令
///
/// CallOp 表示对函数的调用操作，支持固定参数和可变参数函数。
///
/// ## LLVM 语法
///
/// ```llvm
/// ; has retval:
/// %result = (tail?) call <ret_type> @function_name(<arg_types>, ...)
/// ; returns void:
/// (tail?) call void @function_name(<arg_types>, ...)
/// ```
///
/// ## 操作数布局
///
/// ```text
/// [callee, arg0, arg1, ..., argN]
/// ```
/// - `callee`: 被调用的函数（全局函数引用）
/// - `arg0..argN`: 传递给函数的参数
///
/// - `callee`: 被调用函数在此次调用时附加的额外属性
/// - `arg0..argN`: 本次调用每个实参的额外属性
pub struct CallInst {
    pub common: InstCommon,
    pub operands: Box<[UseID]>,
    pub callee_ty: FuncTypeID,
    pub fixed_nargs: u32,
    pub is_vararg: bool,
    pub is_tail_call: Cell<bool>,
}
impl_traceable_from_common!(CallInst, true);
impl IUser for CallInst {
    fn get_operands(&self) -> OperandSet<'_> {
        OperandSet::Fixed(&self.operands)
    }
    fn operands_mut(&mut self) -> &mut [UseID] {
        &mut self.operands
    }
}
impl IPtrUniqueUser for CallInst {
    fn get_operand_pointee_type(&self) -> ValTypeID {
        self.callee_ty.into_ir()
    }
    fn get_operand_pointee_align(&self) -> u32 {
        0
    }
}
impl IFuncUniqueUser for CallInst {
    fn get_operand_func_type(&self) -> FuncTypeID {
        self.callee_ty
    }
}
impl ISubInst for CallInst {
    fn get_common(&self) -> &InstCommon {
        &self.common
    }
    fn common_mut(&mut self) -> &mut InstCommon {
        &mut self.common
    }
    fn try_from_ir_ref(inst: &InstObj) -> Option<&Self> {
        match inst {
            InstObj::Call(c) => Some(c),
            _ => None,
        }
    }
    fn try_from_ir_mut(inst: &mut InstObj) -> Option<&mut Self> {
        match inst {
            InstObj::Call(c) => Some(c),
            _ => None,
        }
    }
    fn try_from_ir(inst: InstObj) -> Option<Self> {
        match inst {
            InstObj::Call(c) => Some(c),
            _ => None,
        }
    }
    fn into_ir(self) -> InstObj {
        InstObj::Call(self)
    }
    fn try_get_jts(&self) -> Option<JumpTargets<'_>> {
        None
    }
}
impl CallInst {
    pub const OP_CALLEE: usize = 0;
    pub const OP_ARGS: RangeFrom<usize> = 1..;
    pub const OP_ARGS_BEGIN: usize = 1;
}

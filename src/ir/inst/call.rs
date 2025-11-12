use crate::{
    impl_traceable_from_common,
    ir::{
        IFuncUniqueUser, IPtrUniqueUser, IRAllocs, ISubInst, ISubInstID, IUser, InstCommon,
        InstObj, JumpTargets, Opcode, OperandSet, UseID, UseKind, ValueSSA,
    },
    subinst_id,
    typing::{FuncTypeID, IValType, TypeContext, ValTypeID},
};
use smallvec::{SmallVec, smallvec};
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
    pub operands: SmallVec<[UseID; 4]>,
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

    pub fn builder(tctx: &TypeContext, callee_ty: FuncTypeID) -> CallInstBuilder {
        CallInstBuilder::new(tctx, callee_ty)
    }

    pub fn callee_use(&self) -> UseID {
        self.operands[Self::OP_CALLEE]
    }
    pub fn get_callee(&self, allocs: &IRAllocs) -> ValueSSA {
        self.callee_use().get_operand(allocs)
    }
    pub fn set_callee(&self, allocs: &IRAllocs, callee: ValueSSA) {
        self.callee_use().set_operand(allocs, callee);
    }

    pub fn arg_uses(&self) -> &[UseID] {
        &self.operands[Self::OP_ARGS]
    }
    pub fn get_arg(&self, allocs: &IRAllocs, index: usize) -> ValueSSA {
        self.arg_uses()[index].get_operand(allocs)
    }
    pub fn set_arg(&self, allocs: &IRAllocs, index: usize, arg: ValueSSA) {
        self.arg_uses()[index].set_operand(allocs, arg);
    }
}

subinst_id!(CallInstID, CallInst);
impl CallInstID {
    pub fn callee_use(self, allocs: &IRAllocs) -> UseID {
        self.deref_ir(allocs).callee_use()
    }
    pub fn get_callee(self, allocs: &IRAllocs) -> ValueSSA {
        self.deref_ir(allocs).get_callee(allocs)
    }
    pub fn set_callee(self, allocs: &IRAllocs, callee: ValueSSA) {
        self.deref_ir(allocs).set_callee(allocs, callee);
    }

    #[inline]
    pub fn arg_uses(self, allocs: &IRAllocs) -> &[UseID] {
        self.deref_ir(allocs).arg_uses()
    }
    pub fn nargs(self, allocs: &IRAllocs) -> usize {
        self.arg_uses(allocs).len()
    }
    #[inline]
    pub fn get_arg(self, allocs: &IRAllocs, index: usize) -> ValueSSA {
        self.deref_ir(allocs).get_arg(allocs, index)
    }
    #[inline]
    pub fn set_arg(self, allocs: &IRAllocs, index: usize, arg: ValueSSA) {
        self.deref_ir(allocs).set_arg(allocs, index, arg);
    }

    pub fn is_tail_call(self, allocs: &IRAllocs) -> bool {
        self.deref_ir(allocs).is_tail_call.get()
    }
    pub fn set_tail_call(self, allocs: &IRAllocs, is_tail: bool) {
        self.deref_ir(allocs).is_tail_call.set(is_tail);
    }

    pub fn callee_ty(self, allocs: &IRAllocs) -> FuncTypeID {
        self.deref_ir(allocs).callee_ty
    }
    pub fn is_vararg(self, allocs: &IRAllocs) -> bool {
        self.deref_ir(allocs).is_vararg
    }
    pub fn fixed_nargs(self, allocs: &IRAllocs) -> u32 {
        self.deref_ir(allocs).fixed_nargs
    }
}

#[derive(Clone)]
pub struct CallInstBuilder {
    callee_ty: FuncTypeID,
    ret_ty: ValTypeID,
    callee: ValueSSA,
    is_vararg: bool,
    fixed_nargs: u32,
    args: SmallVec<[ValueSSA; 4]>,
    is_tail_call: bool,
}
impl CallInstBuilder {
    pub fn new(tctx: &TypeContext, callee_ty: FuncTypeID) -> Self {
        let (ret_ty, nargs, is_vararg) = {
            let fty = callee_ty.deref_ir(tctx);
            (fty.ret_type, fty.args.len() as u32, fty.is_vararg)
        };
        Self {
            callee_ty,
            ret_ty,
            callee: ValueSSA::None,
            is_vararg,
            fixed_nargs: nargs,
            args: SmallVec::new(),
            is_tail_call: false,
        }
    }
    pub fn resize_nargs(&mut self, new_nargs: u32) -> Option<&mut Self> {
        if new_nargs < self.fixed_nargs {
            return None;
        }
        if !self.is_vararg && new_nargs != self.fixed_nargs {
            return None;
        }
        self.fixed_nargs = new_nargs;
        self.args.resize(new_nargs as usize, ValueSSA::None);
        Some(self)
    }
    pub fn set_arg(&mut self, index: usize, arg: ValueSSA) -> &mut Self {
        if self.args.is_empty() && self.fixed_nargs > 0 {
            self.args = smallvec![ValueSSA::None; self.fixed_nargs as usize];
        }
        self.args[index] = arg;
        self
    }
    pub fn with_args(&mut self, args: &[ValueSSA]) -> &mut Self {
        for (i, arg) in args.iter().enumerate() {
            self.set_arg(i, *arg);
        }
        self
    }
    pub fn callee(&mut self, callee: ValueSSA) -> &mut Self {
        self.callee = callee;
        self
    }
    pub fn is_tail_call(&mut self, is_tail: bool) -> &mut Self {
        self.is_tail_call = is_tail;
        self
    }
    pub fn build_obj(&mut self, allocs: &IRAllocs) -> CallInst {
        let nargs = if self.args.is_empty() { self.fixed_nargs as usize } else { self.args.len() };
        let operands = {
            let mut ops = SmallVec::with_capacity(1 + nargs);
            ops.push(UseID::new(allocs, UseKind::CallOpCallee));
            for i in 0..nargs {
                let arg_use = UseID::new(allocs, UseKind::CallOpArg(i as u32));
                ops.push(arg_use);
            }
            ops
        };
        let ret = CallInst {
            common: InstCommon::new(Opcode::Call, self.ret_ty),
            operands,
            callee_ty: self.callee_ty,
            fixed_nargs: self.fixed_nargs,
            is_vararg: self.is_vararg,
            is_tail_call: Cell::new(self.is_tail_call),
        };
        if self.callee != ValueSSA::None {
            ret.set_callee(allocs, self.callee);
        }
        for (i, &arg) in self.args.iter().enumerate() {
            let use_id = ret.operands[CallInst::OP_ARGS_BEGIN + i];
            use_id.set_operand(allocs, arg);
        }
        ret
    }
    pub fn build_id(&mut self, allocs: &IRAllocs) -> CallInstID {
        let inst = self.build_obj(allocs);
        CallInstID::allocate(allocs, inst)
    }
}

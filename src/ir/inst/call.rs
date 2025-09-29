use crate::{
    base::INullableValue,
    ir::{
        AttrList, FuncUser, GlobalRef, IAttrHolderValue, IRAllocs, IRAllocsEditable,
        IRAllocsReadable, IRWriter, ISubInst, ISubValueSSA, IUser, InstCommon, InstData, InstRef,
        Module, Opcode, OperandSet, PtrUser, Use, UseKind, ValueSSA, inst::ISubInstRef,
    },
    typing::{FuncTypeRef, IValType, TypeContext, ValTypeID},
};
use std::{cell::RefCell, num::NonZero, rc::Rc};

/// 函数调用指令
///
/// CallOp 表示对函数的调用操作，支持固定参数和可变参数函数。
///
/// ## LLVM 语法
///
/// ```llvm
/// ; has retval:
/// %result = call <ret_type> @function_name(<arg_types>, ...)
/// ; returns void:
/// call void @function_name(<arg_types>, ...)
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
/// ## 属性布局
///
/// ```text
/// [attr(Callee), attr(arg0), attr(arg1), ..., attr(argN)]
/// ```
///
/// - `callee`: 被调用函数在此次调用时附加的额外属性
/// - `arg0..argN`: 本次调用每个实参的额外属性
///
/// ## 设计特点
/// - **类型安全**: 通过 `FuncTypeRef` 确保参数类型和数量正确
/// - **可变参数**: 支持 C 风格的可变参数函数
/// - **Use-Def 链**: 每个操作数都支持数据流分析
#[derive(Debug)]
pub struct CallOp {
    /// 指令的通用数据（父基本块、操作码、返回类型等）
    common: InstCommon,
    /// 操作数数组：`[callee, arg0, arg1, ..., argN]`
    operands: Box<[Rc<Use>]>,
    /// 属性表: `[attr(Callee), attr(arg0), attr(arg1), ..., attr(argN)]`
    pub attrs: Box<[RefCell<AttrList>]>,
    /// 被调用函数的类型签名
    pub callee_ty: FuncTypeRef,
    /// 固定参数的数量（不包括可变参数）
    pub fixed_nargs: usize,
    /// 是否为可变参数函数
    pub is_vararg: bool,
}

impl PtrUser for CallOp {
    fn get_operand_pointee_type(&self) -> ValTypeID {
        ValTypeID::Func(self.callee_ty)
    }
    fn get_operand_align(&self) -> Option<NonZero<usize>> {
        None
    }
}

impl FuncUser for CallOp {}

impl IUser for CallOp {
    fn get_operands(&self) -> OperandSet<'_> {
        OperandSet::Fixed(&self.operands)
    }
    fn operands_mut(&mut self) -> &mut [Rc<Use>] {
        &mut self.operands
    }
}

impl ISubInst for CallOp {
    fn new_empty(opcode: Opcode) -> Self {
        CallOp {
            common: InstCommon::new(opcode, ValTypeID::Void),
            operands: Box::new([]),
            attrs: Box::new([]),
            callee_ty: FuncTypeRef::new_null(),
            fixed_nargs: 0,
            is_vararg: false,
        }
    }
    fn try_from_ir(inst: &InstData) -> Option<&Self> {
        match inst {
            InstData::Call(call) => Some(call),
            _ => None,
        }
    }
    fn try_from_ir_mut(inst: &mut InstData) -> Option<&mut Self> {
        match inst {
            InstData::Call(call) => Some(call),
            _ => None,
        }
    }
    fn into_ir(self) -> InstData {
        InstData::Call(self)
    }
    fn get_common(&self) -> &InstCommon {
        &self.common
    }
    fn common_mut(&mut self) -> &mut InstCommon {
        &mut self.common
    }
    fn is_terminator(&self) -> bool {
        false // CallOp is not a terminator
    }

    fn fmt_ir(&self, id: Option<usize>, writer: &IRWriter) -> std::io::Result<()> {
        if let Some(id) = id {
            write!(writer, "%{} = ", id)?;
        }
        writer.write_str("call ")?;
        writer.write_type(self.common.ret_type)?;
        writer.write_str(" ")?;
        writer.write_operand(self.get_callee())?;
        writer.write_str("(")?;
        for (index, arg) in self.args().iter().enumerate() {
            if index > 0 {
                writer.write_str(", ")?;
            }
            let type_ctx = writer.type_ctx;
            let arg_ty = self
                .callee_ty
                .try_get_arg(type_ctx, index)
                .unwrap_or(arg.get_operand().get_valtype(writer.allocs));
            writer.write_type(arg_ty)?;
            writer.write_str(" ")?;
            writer.write_operand(arg.get_operand())?;
        }
        writer.write_str(")")
    }
}

impl CallOp {
    pub const OP_CALLEE: usize = 0;
    pub const OP_ARG_BEGIN: usize = 1;

    /// 创建固定参数函数的调用指令
    ///
    /// 用于调用参数数量固定的函数，自动根据函数类型确定参数数量。
    ///
    /// # 参数
    /// - `type_ctx`: 类型上下文，用于解析函数类型信息
    /// - `func_ty`: 被调用函数的类型引用
    ///
    /// # 返回
    /// 返回新创建的函数调用指令
    ///
    /// # Panics
    /// 如果 `func_ty` 是可变参数函数类型则会 panic
    pub fn new_raw_fixed(type_ctx: &TypeContext, func_ty: FuncTypeRef) -> Self {
        debug_assert!(
            !func_ty.is_vararg(type_ctx),
            "Function type must not be vararg for fixed call"
        );
        Self::new_raw_from_allocs(type_ctx, func_ty, func_ty.nargs(type_ctx))
    }

    /// 从模块创建固定参数函数的调用指令
    ///
    /// # 参数
    /// - `module`: IR 模块引用，提供类型上下文
    /// - `func_ty`: 被调用函数的类型引用
    ///
    /// # 返回
    /// 返回新创建的函数调用指令
    pub fn new_fixed_from_module(module: &Module, func_ty: FuncTypeRef) -> Self {
        let type_ctx = module.type_ctx.as_ref();
        Self::new_raw_fixed(type_ctx, func_ty)
    }

    /// 创建指定参数数量的函数调用指令
    ///
    /// 这是最基础的构造函数，支持固定参数和可变参数函数。
    /// 对于可变参数函数，`nargs` 可以大于函数类型定义的固定参数数量。
    ///
    /// # 参数
    /// - `type_ctx`: 类型上下文
    /// - `func_ty`: 被调用函数的类型引用
    /// - `nargs`: 实际传递的参数数量
    ///
    /// # 返回
    /// 返回新创建的函数调用指令（操作数未初始化）
    ///
    /// # Panics
    /// - 对于固定参数函数：如果 `nargs` 不等于函数定义的参数数量
    /// - 对于可变参数函数：如果 `nargs` 小于函数定义的固定参数数量
    pub fn new_raw_from_allocs(type_ctx: &TypeContext, func_ty: FuncTypeRef, nargs: usize) -> Self {
        Self::check_operand_count(type_ctx, func_ty, nargs);
        Self {
            common: InstCommon::new(Opcode::Call, func_ty.ret_type(type_ctx)),
            operands: Self::alloc_operands(nargs as u32),
            attrs: vec![RefCell::new(AttrList::default()); nargs + 1].into_boxed_slice(),
            callee_ty: func_ty,
            fixed_nargs: func_ty.nargs(type_ctx),
            is_vararg: func_ty.is_vararg(type_ctx),
        }
    }

    fn alloc_operands(nargs: u32) -> Box<[Rc<Use>]> {
        let mut operands = Vec::with_capacity(nargs as usize + 1); // +1 for the callee
        operands.push(Use::new(UseKind::CallOpCallee)); // Placeholder for the callee
        for id in 0..nargs {
            operands.push(Use::new(UseKind::CallOpArg(id)));
        }
        operands.into_boxed_slice()
    }

    fn check_operand_count(type_ctx: &TypeContext, func_ty: FuncTypeRef, nargs: usize) {
        let is_vararg = func_ty.is_vararg(type_ctx);
        let fixed_nargs = func_ty.nargs(type_ctx);

        if is_vararg && nargs < fixed_nargs {
            let fname = func_ty.get_display_name(type_ctx);
            panic!("Vararg {fname:?} expects at least {fixed_nargs} arguments, but got {nargs}");
        } else if !is_vararg && nargs != fixed_nargs {
            let fname = func_ty.get_display_name(type_ctx);
            panic!("FuncType {fname:?} expects exactly {fixed_nargs} arguments, but got {nargs}",);
        }
    }

    /// 从分配器创建完整的函数调用指令
    ///
    /// 创建函数调用指令并初始化所有操作数（被调用函数和参数）。
    ///
    /// # 参数
    /// - `allocs`: IR 分配器，用于设置 Use-Def 关系
    /// - `type_ctx`: 类型上下文
    /// - `func_ty`: 被调用函数的类型引用
    /// - `callee`: 被调用的函数（必须是全局引用）
    /// - `args`: 传递给函数的参数迭代器
    ///
    /// # 返回
    /// 返回完全初始化的函数调用指令
    pub fn from_allocs(
        allocs: &IRAllocs,
        type_ctx: &TypeContext,
        callee: GlobalRef,
        args: impl Iterator<Item = ValueSSA> + Clone,
    ) -> Self {
        let nargs = args.clone().count();
        let func_ty = callee.get_content_type(allocs);
        let func_ty = match func_ty {
            ValTypeID::Func(func_ty) => func_ty,
            _ => panic!("Callee must be a function type for CallOp"),
        };
        let ret = Self::new_raw_from_allocs(type_ctx, func_ty, nargs);
        ret.operands[0].set_operand(allocs, ValueSSA::Global(callee));
        for (i, arg) in args.enumerate() {
            ret.operands[i + 1].set_operand(allocs, arg.clone());
        }
        ret
    }
}

impl CallOp {
    /// 获取被调用函数的 Use 对象引用
    ///
    /// 返回操作数数组中第一个元素，即被调用的函数。
    ///
    /// # 返回
    /// 被调用函数的 Use 对象引用
    pub fn callee(&self) -> &Rc<Use> {
        &self.operands[0]
    }

    /// 获取被调用函数的值
    ///
    /// # 返回
    /// 被调用函数的 ValueSSA，通常是 `ValueSSA::Global`
    pub fn get_callee(&self) -> ValueSSA {
        self.callee().get_operand()
    }

    /// 设置被调用的函数
    ///
    /// # 参数
    /// - `allocs`: IR 分配器，用于更新 Use-Def 关系
    /// - `callee`: 新的被调用函数
    pub fn set_callee(&self, allocs: &IRAllocs, callee: ValueSSA) {
        self.callee().set_operand(allocs, callee);
    }

    /// 获取被调用函数的属性列表
    pub fn callee_attrs(&self) -> &RefCell<AttrList> {
        &self.attrs[0]
    }
    pub fn callee_attrs_mut(&mut self) -> &mut AttrList {
        self.attrs[0].get_mut()
    }
    pub fn read_callee_attrs<R>(&self, read: impl FnOnce(&AttrList) -> R) -> R {
        read(&self.attrs[0].borrow())
    }
    pub fn edit_callee_attrs<R>(&self, edit: impl FnOnce(&mut AttrList) -> R) -> R {
        edit(&mut self.attrs[0].borrow_mut())
    }

    /// 获取所有参数的 Use 对象数组
    ///
    /// 返回操作数数组中除了第一个元素（被调用函数）之外的所有元素。
    ///
    /// # 返回
    /// 参数的 Use 对象数组切片
    pub fn args(&self) -> &[Rc<Use>] {
        &self.operands[1..]
    }

    /// 获取参数的属性列表
    pub fn args_attrs(&self) -> &[RefCell<AttrList>] {
        &self.attrs[1..]
    }
    pub fn args_attrs_mut(&mut self) -> &mut [RefCell<AttrList>] {
        &mut self.attrs[1..]
    }

    pub fn arg_attr(&self, id: usize) -> &RefCell<AttrList> {
        assert!(id < self.fixed_nargs, "Index out of bounds for CallOp args");
        &self.attrs[id + 1]
    }
    pub fn arg_attr_mut(&mut self, id: usize) -> &mut AttrList {
        assert!(id < self.fixed_nargs, "Index out of bounds for CallOp args");
        self.attrs[id + 1].get_mut()
    }
    pub fn read_arg_attr<R>(&self, id: usize, read: impl FnOnce(&AttrList) -> R) -> R {
        assert!(id < self.fixed_nargs, "Index out of bounds for CallOp args");
        read(&self.attrs[id + 1].borrow())
    }
    pub fn edit_arg_attr<R>(&self, id: usize, edit: impl FnOnce(&mut AttrList) -> R) -> R {
        assert!(id < self.fixed_nargs, "Index out of bounds for CallOp args");
        edit(&mut self.attrs[id + 1].borrow_mut())
    }

    /// 获取指定索引参数的 Use 对象引用
    ///
    /// # 参数
    /// - `index`: 参数索引（从 0 开始）
    ///
    /// # 返回
    /// 指定参数的 Use 对象引用
    ///
    /// # Panics
    /// 如果索引超出固定参数范围则会 panic
    pub fn ref_arg(&self, index: usize) -> &Rc<Use> {
        assert!(
            index < self.fixed_nargs,
            "Index out of bounds for CallOp args"
        );
        &self.operands[index + 1]
    }

    /// 获取指定索引参数的值
    ///
    /// # 参数
    /// - `index`: 参数索引（从 0 开始）
    ///
    /// # 返回
    /// 指定参数的 ValueSSA
    pub fn get_arg(&self, index: usize) -> ValueSSA {
        self.ref_arg(index).get_operand()
    }

    /// 设置指定索引参数的值
    ///
    /// # 参数
    /// - `allocs`: IR 分配器，用于更新 Use-Def 关系
    /// - `index`: 参数索引（从 0 开始）
    /// - `value`: 新的参数值
    ///
    /// # Panics
    /// 如果索引超出固定参数范围则会 panic
    pub fn set_arg(&self, allocs: &IRAllocs, index: usize, value: ValueSSA) {
        assert!(
            index < self.fixed_nargs,
            "Index out of bounds for CallOp args"
        );
        self.ref_arg(index).set_operand(allocs, value);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CallOpRef(InstRef);

impl IAttrHolderValue for CallOpRef {
    fn attrs(self, allocs: &impl IRAllocsReadable) -> &RefCell<AttrList> {
        let call_inst = self.to_inst(&allocs.get_allocs_ref().insts);
        call_inst.callee_attrs()
    }
    fn attrs_mut(self, allocs: &mut impl IRAllocsEditable) -> &mut AttrList {
        let call_inst = self.to_inst_mut(&mut allocs.get_allocs_mutref().insts);
        call_inst.callee_attrs_mut()
    }
}

impl ISubInstRef for CallOpRef {
    type InstDataT = CallOp;

    fn from_raw_nocheck(inst_ref: InstRef) -> Self {
        CallOpRef(inst_ref)
    }
    fn into_raw(self) -> InstRef {
        self.0
    }
}

impl CallOpRef {
    pub fn get_callee(self, allocs: &impl IRAllocsReadable) -> ValueSSA {
        self.to_inst(&allocs.get_allocs_ref().insts).get_callee()
    }
    pub fn set_callee(self, allocs: &impl IRAllocsReadable, callee: ValueSSA) {
        let allocs = allocs.get_allocs_ref();
        self.to_inst(&allocs.insts).set_callee(allocs, callee);
    }

    pub fn arg_uses(self, allocs: &impl IRAllocsReadable) -> &[Rc<Use>] {
        self.to_inst(&allocs.get_allocs_ref().insts).args()
    }
    pub fn arg_count(self, allocs: &impl IRAllocsReadable) -> usize {
        self.to_inst(&allocs.get_allocs_ref().insts).args().len()
    }
    pub fn get_arg(self, allocs: &impl IRAllocsReadable, index: usize) -> ValueSSA {
        self.to_inst(&allocs.get_allocs_ref().insts).get_arg(index)
    }
    pub fn set_arg(self, allocs: &impl IRAllocsReadable, index: usize, value: ValueSSA) {
        let allocs = allocs.get_allocs_ref();
        self.to_inst(&allocs.insts).set_arg(allocs, index, value);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct CallArgID(pub CallOpRef, pub usize);

impl IAttrHolderValue for CallArgID {
    fn attrs(self, allocs: &impl IRAllocsReadable) -> &RefCell<AttrList> {
        let call_inst = self.0.to_inst(&allocs.get_allocs_ref().insts);
        call_inst.arg_attr(self.1)
    }
    fn attrs_mut(self, allocs: &mut impl IRAllocsEditable) -> &mut AttrList {
        let call_inst = self.0.to_inst_mut(&mut allocs.get_allocs_mutref().insts);
        call_inst.arg_attr_mut(self.1)
    }
}

impl CallArgID {
    pub fn get_use(self, allocs: &impl IRAllocsReadable) -> &Rc<Use> {
        let call_inst = self.0.to_inst(&allocs.get_allocs_ref().insts);
        call_inst.ref_arg(self.1)
    }

    pub fn get_value(self, allocs: &impl IRAllocsReadable) -> ValueSSA {
        self.get_use(allocs).get_operand()
    }

    pub fn set_value(self, allocs: &impl IRAllocsReadable, value: ValueSSA) {
        let allocs = allocs.get_allocs_ref();
        let call_inst = self.0.to_inst(&allocs.insts);
        call_inst.set_arg(allocs, self.1, value);
    }
}

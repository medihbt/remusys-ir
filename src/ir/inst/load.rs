use crate::{
    ir::{
        IRAllocs, IRWriter, ISubInst, IUser, InstCommon, InstData, InstRef, Opcode, OperandSet,
        PtrUser, Use, UseKind, ValueSSA, inst::ISubInstRef,
    },
    typing::{IValType, TypeContext, ValTypeID},
};
use std::{num::NonZero, rc::Rc};

/// Load 指令：从内存中加载数据到寄存器或变量
///
/// LoadOp 实现了 LLVM 的 load 指令语义，用于从指定的内存地址加载数据。
/// 该指令是编译器 IR 中最基本的内存读取操作之一。
///
/// ### LLVM 语法
///
/// ```llvm
/// <result> = load <ty>, ptr <pointer>, align <alignment>
/// ```
///
/// ### 指令语义
///
/// 1. **内存读取**：从 `pointer` 指向的内存地址读取 `ty` 类型的数据
/// 2. **类型安全**：加载的数据类型由 `common.ret_type` 指定
/// 3. **内存对齐**：支持指定内存对齐方式，提高访问效率
/// 4. **SSA 形式**：加载的结果作为新的 SSA 值返回
///
/// ### 操作数布局
///
/// * `operands[0]`: 源地址 (LoadSource) - 指向要加载数据的内存地址
///
/// ### 内存对齐
///
/// * `pointee_align_log2`: 对齐方式的以2为底的对数，实际对齐大小为 `1 << pointee_align_log2`
/// * 对齐值必须是2的幂次，且不能超过被加载类型的自然对齐
///
/// ### 使用示例
///
/// ```ignore
/// // 从指针 %ptr 加载一个 i32 值，使用4字节对齐
/// let load_inst = LoadOp::new(allocs, ValTypeID::Int(32), ptr_value, 2); // 2^2 = 4 bytes
///
/// // 使用类型的自然对齐方式
/// let auto_load = LoadOp::new_autoalign(allocs, type_ctx, ValTypeID::Int(64), ptr_value);
/// ```
#[derive(Debug)]
pub struct LoadOp {
    /// 指令的公共字段（操作码、返回类型等）
    common: InstCommon,

    /// 操作数数组，包含源地址指针
    operands: [Rc<Use>; 1],

    /// 被加载数据的内存对齐方式（以2为底的对数）
    /// 实际对齐大小为 `1 << pointee_align_log2` 字节
    pub pointee_align_log2: u8,
}

impl IUser for LoadOp {
    fn get_operands(&self) -> OperandSet {
        OperandSet::Fixed(&self.operands)
    }
    fn operands_mut(&mut self) -> &mut [Rc<Use>] {
        &mut self.operands
    }
}

impl ISubInst for LoadOp {
    fn new_empty(_: Opcode) -> Self {
        Self {
            common: InstCommon::new(Opcode::Load, ValTypeID::Void),
            operands: [Use::new(UseKind::LoadSource)],
            pointee_align_log2: 0,
        }
    }
    fn try_from_ir(inst: &InstData) -> Option<&Self> {
        if let InstData::Load(load) = inst { Some(load) } else { None }
    }
    fn try_from_ir_mut(inst: &mut InstData) -> Option<&mut Self> {
        if let InstData::Load(load) = inst { Some(load) } else { None }
    }
    fn into_ir(self) -> InstData {
        InstData::Load(self)
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
            return Err(Error::new(InvalidInput, "ID must be provided for LoadOp"));
        };
        write!(writer, "%{id} = load ")?;
        writer.write_type(self.get_valtype())?;
        writer.write_str(", ptr ")?;
        writer.write_operand(self.get_source())?;
        write!(writer, ", align {}", 1 << self.pointee_align_log2)
    }
}

impl PtrUser for LoadOp {
    fn get_operand_pointee_type(&self) -> ValTypeID {
        self.common.ret_type
    }

    fn get_operand_align(&self) -> Option<NonZero<usize>> {
        NonZero::new(1 << self.pointee_align_log2)
    }
}

impl LoadOp {
    /// 创建一个未初始化操作数的 LoadOp 指令
    ///
    /// 此方法创建一个"原始"的 Load 指令，操作数需要后续手动设置。
    /// 主要用于指令构建的中间步骤或需要延迟设置操作数的场景。
    ///
    /// #### 参数
    ///
    /// * `ret_type` - 加载操作的返回类型，即被加载数据的类型
    /// * `align_log2` - 内存对齐方式的以2为底的对数
    ///
    /// #### 返回值
    ///
    /// 返回一个操作数未初始化的 LoadOp 实例
    ///
    /// #### 示例
    ///
    /// ```ignore
    /// let load = LoadOp::new_raw(ValTypeID::Int(32), 2); // 4字节对齐的i32加载
    /// // 后续需要调用 set_source 设置源地址
    /// ```
    pub fn new_raw(ret_type: ValTypeID, align_log2: u8) -> Self {
        Self {
            common: InstCommon::new(Opcode::Load, ret_type),
            operands: [Use::new(UseKind::LoadSource)],
            pointee_align_log2: align_log2,
        }
    }

    /// 创建一个完整初始化的 LoadOp 指令
    ///
    /// 此方法创建一个完全初始化的 Load 指令，包括源地址和对齐信息。
    /// 这是创建 Load 指令的标准方法。
    ///
    /// #### 参数
    ///
    /// * `allocs` - IR 分配器，用于管理 Use-Def 链
    /// * `ret_ty` - 加载操作的返回类型
    /// * `source` - 源地址，必须是指针类型的 SSA 值
    /// * `align_log2` - 内存对齐方式的以2为底的对数
    ///
    /// #### 返回值
    ///
    /// 返回一个完全初始化的 LoadOp 实例
    ///
    /// #### 示例
    ///
    /// ```ignore
    /// let load = LoadOp::new(allocs, ValTypeID::Int(64), ptr_ssa, 3); // 8字节对齐的i64加载
    /// ```
    pub fn new(allocs: &IRAllocs, ret_ty: ValTypeID, source: ValueSSA, align_log2: u8) -> Self {
        let load_op = Self::new_raw(ret_ty, align_log2);
        load_op.operands[0].set_operand(allocs, source);
        load_op
    }

    /// 创建一个使用类型自然对齐的 LoadOp 指令
    ///
    /// 此方法根据被加载类型的自然对齐方式自动计算对齐参数，
    /// 简化了常见场景下的指令创建过程。
    ///
    /// #### 参数
    ///
    /// * `allocs` - IR 分配器，用于管理 Use-Def 链
    /// * `type_ctx` - 类型上下文，用于查询类型的对齐信息
    /// * `ret_ty` - 加载操作的返回类型
    /// * `source` - 源地址，必须是指针类型的 SSA 值
    ///
    /// #### 返回值
    ///
    /// 返回一个使用自然对齐的 LoadOp 实例
    ///
    /// #### Panics
    ///
    /// * 如果类型的自然对齐不是2的幂次，则会 panic
    /// * 如果无法获取类型的对齐信息，则会 panic
    ///
    /// #### 示例
    ///
    /// ```ignore
    /// let load = LoadOp::new_autoalign(allocs, type_ctx, ValTypeID::Float(64), ptr_ssa);
    /// // 自动使用 f64 的自然对齐（通常是8字节）
    /// ```
    pub fn new_autoalign(
        allocs: &IRAllocs,
        type_ctx: &TypeContext,
        ret_ty: ValTypeID,
        source: ValueSSA,
    ) -> Self {
        let align_log2 = {
            let align = ret_ty.get_align(type_ctx);
            if align.is_power_of_two() {
                align.trailing_zeros() as u8
            } else {
                panic!("Type alignment is not a power of two: {}", align);
            }
        };
        Self::new(allocs, ret_ty, source, align_log2)
    }

    /// 获取源地址操作数的 Use 对象引用
    ///
    /// Use 对象管理着 SSA 值之间的 Use-Def 关系，
    /// 可以用于查询或修改操作数的连接关系。
    ///
    /// #### 返回值
    ///
    /// 返回源地址操作数的 Use 对象引用
    pub fn source_use(&self) -> &Use {
        &self.operands[0]
    }

    /// 获取源地址的 SSA 值
    ///
    /// #### 返回值
    ///
    /// 返回当前设置的源地址 SSA 值
    pub fn get_source(&self) -> ValueSSA {
        self.source_use().get_operand()
    }

    /// 设置源地址的 SSA 值
    ///
    /// 此方法会更新 Use-Def 链，确保 SSA 图的一致性。
    ///
    /// #### 参数
    ///
    /// * `allocs` - IR 分配器，用于管理 Use-Def 链
    /// * `source` - 新的源地址 SSA 值
    pub fn set_source(&mut self, allocs: &IRAllocs, source: ValueSSA) {
        self.operands[0].set_operand(allocs, source);
    }
}

/// LoadOp 指令的强类型引用
///
/// LoadInstRef 提供了一个类型安全的方式来引用 LoadOp 指令，
/// 确保引用的指令确实是 Load 指令类型。这种设计避免了运行时的类型检查开销，
/// 同时提供了编译时的类型安全保证。
///
/// # 用途
///
/// * **类型安全**：确保引用的指令是 LoadOp 类型
/// * **性能优化**：避免运行时类型转换开销
/// * **API 一致性**：与其他指令引用类型保持一致的接口
///
/// # 示例
///
/// ```ignore
/// let inst_ref: InstRef = /* 获取指令引用 */;
/// let load_ref = LoadInstRef::from_raw_nocheck(inst_ref);
/// // 现在可以安全地访问 LoadOp 的特定方法
/// ```
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct LoadInstRef(InstRef);

impl ISubInstRef for LoadInstRef {
    type InstDataT = LoadOp;

    fn from_raw_nocheck(inst_ref: InstRef) -> Self {
        LoadInstRef(inst_ref)
    }
    fn into_raw(self) -> InstRef {
        self.0
    }
}

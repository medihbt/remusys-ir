use crate::{
    impl_debug_for_subinst_id, impl_traceable_from_common,
    ir::{
        IPtrUniqueUser, IRAllocs, ISubInst, ISubInstID, IUser, InstCommon, InstID, InstObj, Opcode,
        OperandSet, UseID, UseKind, ValueSSA,
    },
    typing::ValTypeID,
};

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
/// * `align_log2`: 对齐方式的以2为底的对数，实际对齐大小为 `1 << align_log2`
/// * 对齐值必须是2的幂次，且不能超过被加载类型的自然对齐
pub struct LoadInst {
    pub common: InstCommon,
    operands: [UseID; 1],
    pub align_log2: u8,
}
impl_traceable_from_common!(LoadInst, true);
impl IUser for LoadInst {
    fn get_operands(&self) -> OperandSet<'_> {
        OperandSet::Fixed(&self.operands)
    }
    fn operands_mut(&mut self) -> &mut [UseID] {
        &mut self.operands
    }
}
impl IPtrUniqueUser for LoadInst {
    fn get_operand_pointee_type(&self) -> ValTypeID {
        self.common.ret_type
    }
    fn get_operand_pointee_align(&self) -> u32 {
        1 << self.align_log2
    }
}
impl ISubInst for LoadInst {
    fn get_common(&self) -> &InstCommon {
        &self.common
    }
    fn common_mut(&mut self) -> &mut InstCommon {
        &mut self.common
    }
    fn try_from_ir_ref(inst: &InstObj) -> Option<&Self> {
        match inst {
            InstObj::Load(load) => Some(load),
            _ => None,
        }
    }
    fn try_from_ir_mut(inst: &mut InstObj) -> Option<&mut Self> {
        match inst {
            InstObj::Load(load) => Some(load),
            _ => None,
        }
    }
    fn try_from_ir(inst: InstObj) -> Option<Self> {
        match inst {
            InstObj::Load(load) => Some(load),
            _ => None,
        }
    }
    fn into_ir(self) -> InstObj {
        InstObj::Load(self)
    }
    fn try_get_jts(&self) -> Option<crate::ir::JumpTargets<'_>> {
        None
    }
}
impl LoadInst {
    pub const OP_SOURCE: usize = 0;

    pub fn new_uninit(allocs: &IRAllocs, pointee_ty: ValTypeID, align_log2: u8) -> Self {
        Self {
            common: InstCommon::new(Opcode::Load, pointee_ty),
            operands: [UseID::new(allocs, UseKind::LoadSource)],
            align_log2,
        }
    }

    pub fn source_use(&self) -> UseID {
        self.operands[Self::OP_SOURCE]
    }
    pub fn get_source(&self, allocs: &IRAllocs) -> ValueSSA {
        self.source_use().get_operand(allocs)
    }
    pub fn set_source(&self, allocs: &IRAllocs, value: ValueSSA) {
        self.source_use().set_operand(allocs, value);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct LoadInstID(pub InstID);
impl_debug_for_subinst_id!(LoadInstID);
impl ISubInstID for LoadInstID {
    type InstObjT = LoadInst;

    fn raw_from_ir(id: InstID) -> Self {
        LoadInstID(id)
    }
    fn into_ir(self) -> InstID {
        self.0
    }
}
impl LoadInstID {
    pub fn new_uninit(allocs: &IRAllocs, pointee_ty: ValTypeID, align_log2: u8) -> Self {
        Self::allocate(allocs, LoadInst::new_uninit(allocs, pointee_ty, align_log2))
    }

    pub fn source_use(self, allocs: &IRAllocs) -> UseID {
        self.deref_ir(allocs).source_use()
    }
    pub fn get_source(self, allocs: &IRAllocs) -> ValueSSA {
        self.deref_ir(allocs).get_source(allocs)
    }
    pub fn set_source(self, allocs: &IRAllocs, value: ValueSSA) {
        self.deref_ir(allocs).set_source(allocs, value);
    }

    pub fn align_log2(self, allocs: &IRAllocs) -> u8 {
        self.deref_ir(allocs).align_log2
    }
    pub fn get_align(self, allocs: &IRAllocs) -> u32 {
        1 << self.align_log2(allocs)
    }
}

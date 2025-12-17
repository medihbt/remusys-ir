use crate::{
    _remusys_ir_subinst,
    ir::{
        IPtrUniqueUser, IRAllocs, ISubInst, ISubInstID, ISubValueSSA, IUser, InstCommon, InstObj,
        Opcode, OperandSet, UseID, UseKind, ValueSSA,
    },
    typing::ValTypeID,
};

/// Store 指令: 存储一个 SSA 值到指针所示的存储区域
///
/// ### 语法
///
/// ```llvm
/// store <ty> <value>, ptr <pointer>, align <alignment>
/// ```
///
/// * `alignment`: 内存对齐方式，大小是 `1 << source_align_log2` 字节。
///
/// ### 操作数布局
///
/// * `operands[0]`: 要存储的值 (StoreValue) - 指向要存储的数据
/// * `operands[1]`: 目标地址 (StoreTarget) - 指向要存储数据的内存地址
pub struct StoreInst {
    pub common: InstCommon,
    operands: [UseID; 2],
    pub source_ty: ValTypeID,
    pub align_log2: u8,
}

impl IUser for StoreInst {
    fn get_operands(&self) -> OperandSet<'_> {
        OperandSet::Fixed(&self.operands)
    }
    fn operands_mut(&mut self) -> &mut [UseID] {
        &mut self.operands
    }
}
impl IPtrUniqueUser for StoreInst {
    fn get_operand_pointee_type(&self) -> ValTypeID {
        self.source_ty
    }
    fn get_operand_pointee_align(&self) -> u32 {
        1 << self.align_log2
    }
}
impl ISubInst for StoreInst {
    fn get_common(&self) -> &InstCommon {
        &self.common
    }
    fn common_mut(&mut self) -> &mut InstCommon {
        &mut self.common
    }
    fn try_from_ir_ref(inst: &InstObj) -> Option<&Self> {
        match inst {
            InstObj::Store(store) => Some(store),
            _ => None,
        }
    }
    fn try_from_ir_mut(inst: &mut InstObj) -> Option<&mut Self> {
        match inst {
            InstObj::Store(store) => Some(store),
            _ => None,
        }
    }
    fn try_from_ir(inst: InstObj) -> Option<Self> {
        match inst {
            InstObj::Store(store) => Some(store),
            _ => None,
        }
    }
    fn into_ir(self) -> InstObj {
        InstObj::Store(self)
    }
    fn try_get_jts(&self) -> Option<crate::ir::JumpTargets<'_>> {
        None
    }
}
impl StoreInst {
    pub const OP_SOURCE: usize = 0;
    pub const OP_TARGET: usize = 1;

    pub fn new_uninit(allocs: &IRAllocs, source_ty: ValTypeID, align_log2: u8) -> Self {
        Self {
            common: InstCommon::new(Opcode::Store, ValTypeID::Void),
            operands: [
                UseID::new(allocs, UseKind::StoreSource),
                UseID::new(allocs, UseKind::StoreTarget),
            ],
            source_ty,
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

    pub fn target_use(&self) -> UseID {
        self.operands[Self::OP_TARGET]
    }
    pub fn get_target(&self, allocs: &IRAllocs) -> ValueSSA {
        self.target_use().get_operand(allocs)
    }
    pub fn set_target(&self, allocs: &IRAllocs, value: ValueSSA) {
        self.target_use().set_operand(allocs, value);
    }
}

_remusys_ir_subinst!(StoreInstID, StoreInst);
impl StoreInstID {
    pub fn new_uninit(allocs: &IRAllocs, source_ty: ValTypeID, align_log2: u8) -> Self {
        let inst = StoreInst::new_uninit(allocs, source_ty, align_log2);
        Self::allocate(allocs, inst)
    }
    pub fn new(allocs: &IRAllocs, source: ValueSSA, target: ValueSSA, align_log2: u8) -> Self {
        let source_ty = source.get_valtype(allocs);
        let ret = Self::new_uninit(allocs, source_ty, align_log2);
        ret.set_source(allocs, source);
        ret.set_target(allocs, target);
        ret
    }

    pub fn source_use(&self, allocs: &IRAllocs) -> UseID {
        self.deref_ir(allocs).source_use()
    }
    pub fn get_source(&self, allocs: &IRAllocs) -> ValueSSA {
        self.deref_ir(allocs).get_source(allocs)
    }
    pub fn set_source(&self, allocs: &IRAllocs, value: ValueSSA) {
        self.deref_ir(allocs).set_source(allocs, value);
    }

    pub fn target_use(&self, allocs: &IRAllocs) -> UseID {
        self.deref_ir(allocs).target_use()
    }
    pub fn get_target(&self, allocs: &IRAllocs) -> ValueSSA {
        self.deref_ir(allocs).get_target(allocs)
    }
    pub fn set_target(&self, allocs: &IRAllocs, value: ValueSSA) {
        self.deref_ir(allocs).set_target(allocs, value);
    }

    pub fn source_ty(&self, allocs: &IRAllocs) -> ValTypeID {
        self.deref_ir(allocs).source_ty
    }
    pub fn align_log2(&self, allocs: &IRAllocs) -> u8 {
        self.deref_ir(allocs).align_log2
    }
}

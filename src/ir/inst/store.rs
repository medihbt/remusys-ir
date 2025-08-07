use crate::{
    ir::{
        IRAllocs, IRWriter, ISubInst, ISubValueSSA, InstCommon, InstData, InstRef, Opcode, PtrUser,
        Use, UseKind, ValueSSA,
        inst::{ISubInstRef, InstOperands},
    },
    typing::{context::TypeContext, id::ValTypeID},
};
use std::{num::NonZero, rc::Rc};

/// Store 指令: 将数据存储到内存地址。
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
#[derive(Debug)]
pub struct StoreOp {
    common: InstCommon,
    operands: [Rc<Use>; 2],

    /// 被存储数据的类型
    pub source_ty: ValTypeID,

    /// 被存储数据的内存对齐方式（以2为底的对数）
    /// 实际对齐大小为 `1 << pointee_align_log2` 字节
    pub source_align_log2: u8,
}

impl ISubInst for StoreOp {
    fn new_empty(_: Opcode) -> Self {
        Self {
            common: InstCommon::new(Opcode::Store, ValTypeID::Void),
            operands: [Use::new(UseKind::StoreSource), Use::new(UseKind::StoreTarget)],
            source_ty: ValTypeID::Void,
            source_align_log2: 0,
        }
    }
    fn try_from_ir(inst: &InstData) -> Option<&Self> {
        if let InstData::Store(store) = inst { Some(store) } else { None }
    }
    fn try_from_ir_mut(inst: &mut InstData) -> Option<&mut Self> {
        if let InstData::Store(store) = inst { Some(store) } else { None }
    }
    fn into_ir(self) -> InstData {
        InstData::Store(self)
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
        InstOperands::Fixed(&self.operands)
    }
    fn operands_mut(&mut self) -> &mut [Rc<Use>] {
        &mut self.operands
    }

    fn fmt_ir(&self, _: Option<usize>, writer: &IRWriter) -> std::io::Result<()> {
        writer.write_str("store ")?;
        writer.write_type(self.source_ty)?;
        writer.write_str(" ")?;
        writer.write_operand(self.get_source())?;

        writer.write_str(", ptr ")?;
        writer.write_operand(self.get_target())?;

        write!(writer, ", align {}", 1 << self.source_align_log2)
    }
}

impl PtrUser for StoreOp {
    fn get_operand_pointee_type(&self) -> ValTypeID {
        self.source_ty
    }

    fn get_operand_align(&self) -> Option<NonZero<usize>> {
        NonZero::new(1 << self.source_align_log2)
    }
}

impl StoreOp {
    pub fn new_raw(source_ty: ValTypeID, align_log2: u8) -> Self {
        Self {
            common: InstCommon::new(Opcode::Store, ValTypeID::Void),
            operands: [Use::new(UseKind::StoreSource), Use::new(UseKind::StoreTarget)],
            source_ty,
            source_align_log2: align_log2,
        }
    }
    pub fn new(
        allocs: &IRAllocs,
        type_ctx: &TypeContext,
        source: ValueSSA,
        target: ValueSSA,
    ) -> Self {
        let source_ty = source.get_valtype(allocs);
        let source_align_log2 = source_ty.get_align_log2(type_ctx);
        assert_eq!(
            target.get_valtype(allocs),
            ValTypeID::Ptr,
            "Store target must be a pointer type"
        );
        let store = Self::new_raw(source_ty, source_align_log2);
        store.operands[0].set_operand(allocs, source);
        store.operands[1].set_operand(allocs, target);
        store
    }

    pub fn source_use(&self) -> &Rc<Use> {
        &self.operands[0]
    }
    pub fn target_use(&self) -> &Rc<Use> {
        &self.operands[1]
    }

    pub fn get_source(&self) -> ValueSSA {
        self.source_use().get_operand()
    }
    pub fn get_target(&self) -> ValueSSA {
        self.target_use().get_operand()
    }

    pub fn set_source(&mut self, allocs: &IRAllocs, source: ValueSSA) {
        self.source_use().set_operand(allocs, source);
    }
    pub fn set_target(&mut self, allocs: &IRAllocs, target: ValueSSA) {
        self.target_use().set_operand(allocs, target);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct StoreOpRef(InstRef);

impl ISubInstRef for StoreOpRef {
    type InstDataT = StoreOp;

    fn from_raw_nocheck(inst_ref: InstRef) -> Self {
        StoreOpRef(inst_ref)
    }
    fn into_raw(self) -> InstRef {
        self.0
    }
}

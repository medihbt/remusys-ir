use crate::{
    _remusys_ir_subinst_id, impl_traceable_from_common,
    ir::{
        IPtrValue, IRAllocs, ISubInst, ISubInstID, IUser, InstCommon, InstObj, JumpTargets, Opcode,
        OperandSet, UseID,
    },
    typing::ValTypeID,
};

/// 在栈上分配一段固定大小的内存. 这个指令的特殊之处在于, 该指令分配得到的内存
/// 在函数内全局有效、全局存活, 直到函数返回或被销毁.
///
/// 想要分配动态大小的栈内存或者控制分配内存的生命周期, 请使用 `DynAlloca` 指令.
/// 只不过时间实在来不及了, `DynAlloca` 在此版本中不实现.
///
/// * 操作数布局: 没有操作数.
///
/// ### 语法
///
/// ```llvm
/// %<result> = alloca <pointee_ty>, align <alignment>
/// ```
pub struct AllocaInst {
    pub common: InstCommon,
    /// 指向分配内存的类型. 如果要一次分配多个同类型的元素, 请使用对应的数组类型
    /// 填充 `pointee_ty`.
    pub pointee_ty: ValTypeID,
    pub align_log2: u8,
}
impl_traceable_from_common!(AllocaInst, true);
impl IUser for AllocaInst {
    fn get_operands(&self) -> OperandSet<'_> {
        OperandSet::Fixed(&[])
    }
    fn operands_mut(&mut self) -> &mut [UseID] {
        &mut []
    }
}
impl IPtrValue for AllocaInst {
    fn get_ptr_pointee_type(&self) -> ValTypeID {
        self.pointee_ty
    }
    fn get_ptr_pointee_align(&self) -> u32 {
        1 << self.align_log2
    }
}
impl ISubInst for AllocaInst {
    fn get_common(&self) -> &InstCommon {
        &self.common
    }
    fn common_mut(&mut self) -> &mut InstCommon {
        &mut self.common
    }

    fn try_from_ir_ref(inst: &InstObj) -> Option<&Self> {
        match inst {
            InstObj::Alloca(a) => Some(a),
            _ => None,
        }
    }
    fn try_from_ir_mut(inst: &mut InstObj) -> Option<&mut Self> {
        match inst {
            InstObj::Alloca(a) => Some(a),
            _ => None,
        }
    }
    fn try_from_ir(inst: InstObj) -> Option<Self> {
        match inst {
            InstObj::Alloca(a) => Some(a),
            _ => None,
        }
    }
    fn into_ir(self) -> InstObj {
        InstObj::Alloca(self)
    }

    fn try_get_jts(&self) -> Option<JumpTargets<'_>> {
        None
    }
}
impl AllocaInst {
    /// 创建一个新的 Alloca 指令, 分配指定类型的内存.
    pub fn new(pointee_ty: ValTypeID, align_log2: u8) -> Self {
        Self {
            common: InstCommon::new(Opcode::Alloca, ValTypeID::Ptr),
            pointee_ty,
            align_log2,
        }
    }
}

_remusys_ir_subinst_id!(AllocaInstID, AllocaInst);
impl AllocaInstID {
    pub fn new(allocs: &IRAllocs, pointee_ty: ValTypeID, align_log2: u8) -> Self {
        Self::allocate(allocs, AllocaInst::new(pointee_ty, align_log2))
    }

    pub fn get_pointee_ty(self, allocs: &IRAllocs) -> ValTypeID {
        self.deref_ir(allocs).pointee_ty
    }
    pub fn get_align_log2(self, allocs: &IRAllocs) -> u8 {
        self.deref_ir(allocs).align_log2
    }
    pub fn get_align(self, allocs: &IRAllocs) -> u32 {
        1 << self.get_align_log2(allocs)
    }
}

use std::{num::NonZero, rc::Rc};

use crate::{
    ir::{
        IRWriter, ISubInst, InstCommon, InstData, InstRef, Opcode, PtrStorage, Use,
        inst::{ISubInstRef, InstOperands},
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
#[derive(Debug)]
pub struct Alloca {
    common: InstCommon,
    /// 指向分配内存的类型. 如果要一次分配多个同类型的元素, 请使用
    /// 对应的数组类型填充 `pointee_ty`.
    pub pointee_ty: ValTypeID,
    /// 对齐方式的对数
    pub align_log2: u8,
}

impl ISubInst for Alloca {
    fn new_empty(_: Opcode) -> Self {
        Self {
            common: InstCommon::new(Opcode::Alloca, ValTypeID::Ptr),
            pointee_ty: ValTypeID::Void,
            align_log2: 0,
        }
    }
    fn try_from_ir(inst: &InstData) -> Option<&Self> {
        match inst {
            InstData::Alloca(alloca) => Some(alloca),
            _ => None,
        }
    }
    fn try_from_ir_mut(inst: &mut InstData) -> Option<&mut Self> {
        match inst {
            InstData::Alloca(alloca) => Some(alloca),
            _ => None,
        }
    }
    fn into_ir(self) -> InstData {
        InstData::Alloca(self)
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
        InstOperands::Fixed(&[])
    }
    fn operands_mut(&mut self) -> &mut [Rc<Use>] {
        &mut []
    }

    fn fmt_ir(&self, id: Option<usize>, writer: &IRWriter) -> std::io::Result<()> {
        let Some(id) = id else { panic!("Tried to format an Alloca without an ID") };
        write!(writer, "%{} = alloca ", id)?;
        writer.write_type(self.pointee_ty)?;
        write!(writer, ", align {}", 1usize << self.align_log2)
    }
}

impl PtrStorage for Alloca {
    fn get_stored_pointee_type(&self) -> ValTypeID {
        self.pointee_ty
    }
    fn get_stored_pointee_align(&self) -> Option<NonZero<usize>> {
        NonZero::new(1usize << self.align_log2)
    }
}

impl Alloca {
    /// 创建一个新的 Alloca 指令, 分配指定类型的内存.
    pub fn new(pointee_ty: ValTypeID, align_log2: u8) -> Self {
        Self {
            common: InstCommon::new(Opcode::Alloca, ValTypeID::Ptr),
            pointee_ty,
            align_log2,
        }
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct AllocaRef(InstRef);

impl ISubInstRef for AllocaRef {
    type InstDataT = Alloca;
    fn from_raw_nocheck(inst_ref: InstRef) -> Self {
        Self(inst_ref)
    }
    fn into_raw(self) -> InstRef {
        self.0
    }
}

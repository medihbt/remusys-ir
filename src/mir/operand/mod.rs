//! 重构的 MIR 操作数模块

use crate::mir::{
    module::{block::MirBlockRef, MirGlobalRef},
    operand::{
        imm::ImmConst, reg::{PReg, VReg}, suboperand::IMirSubOperand
    },
};

pub mod imm;
pub mod reg;
pub mod suboperand;

/// MIR 操作数.
#[derive(Debug, Clone, Copy, PartialEq, Hash)]
pub enum MirOperand {
    /// 没有操作数, 通常用于占位或无操作数指令.
    None,
    /// 物理寄存器操作数.
    PReg(PReg),
    /// 虚拟寄存器操作数.
    VReg(VReg),
    /// 立即数操作数, 表示其二进制布局.
    Imm(i64),
    /// 有限制的立即数
    ImmLimit(ImmConst),
    /// 全局变量引用.
    Global(MirGlobalRef),
    /// 标签引用, 用于控制流跳转.
    Label(MirBlockRef),
    /// 连续 case 跳转表位置. 这里 `Vec` 表示这个跳转表在内存布局上是个 MirBlockRef 数组.
    VecSwitchTab(u32),
    /// 不连续 case 跳转表位置. 这里 `Bin` 表示这个跳转表使用二分查找.
    BinSwitchTab(u32),
}

impl MirOperand {
    pub fn into_sub<SubOpT: IMirSubOperand>(self) -> SubOpT {
        SubOpT::from_mirop(self)
    }
    pub fn from_sub(sub_op: impl IMirSubOperand) -> Self {
        sub_op.into_mirop()
    }
}

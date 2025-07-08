use crate::{
    base::NullableValue,
    mir::{
        module::{MirGlobalRef, block::MirBlockRef},
        operand::{
            MirOperand,
            reg::{PReg, RegUseFlags, SubRegIndex, VReg},
        },
    },
};

/// MIR 操作数限定子操作数接口.
pub trait IMirSubOperand {
    fn new_empty_mirsubop() -> Self;

    /// 将 MIR 操作数转换为目标操作数.
    fn from_mirop(operand: MirOperand) -> Self;

    /// 将目标操作数转换为 MIR 操作数.
    fn into_mirop(self) -> MirOperand;

    /// 在保证原操作数额外属性的情况下将目标操作数插入到 MIR 操作数中.
    /// 例如, 原操作数是寄存器操作数时, 它的使用标志和子寄存器索引会被保留.
    fn insert_to_mirop(self, op: MirOperand) -> MirOperand;
}

impl IMirSubOperand for MirBlockRef {
    fn new_empty_mirsubop() -> Self {
        MirBlockRef::new_null()
    }

    fn from_mirop(operand: MirOperand) -> Self {
        match operand {
            MirOperand::Label(block_ref) => block_ref,
            _ => panic!("Expected a MirBlockRef, found: {:?}", operand),
        }
    }
    fn into_mirop(self) -> MirOperand {
        MirOperand::Label(self)
    }
    fn insert_to_mirop(self, _: MirOperand) -> MirOperand {
        MirOperand::Label(self)
    }
}

impl IMirSubOperand for MirGlobalRef {
    fn from_mirop(operand: MirOperand) -> Self {
        match operand {
            MirOperand::Global(item_ref) => item_ref,
            _ => panic!("Expected a ModuleItemRef, found: {:?}", operand),
        }
    }
    fn into_mirop(self) -> MirOperand {
        MirOperand::Global(self)
    }
    fn insert_to_mirop(self, _: MirOperand) -> MirOperand {
        MirOperand::Global(self)
    }

    fn new_empty_mirsubop() -> Self {
        MirGlobalRef::new_null()
    }
}

impl IMirSubOperand for i64 {
    fn from_mirop(operand: MirOperand) -> Self {
        match operand {
            MirOperand::Imm(value) => value,
            _ => panic!("Expected an i64, found: {:?}", operand),
        }
    }
    fn into_mirop(self) -> MirOperand {
        MirOperand::Imm(self)
    }
    fn insert_to_mirop(self, _: MirOperand) -> MirOperand {
        MirOperand::Imm(self)
    }

    fn new_empty_mirsubop() -> Self {
        0
    }
}

/// 限制操作数集合1: 寄存器操作数
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum RegOperand {
    V(VReg),
    P(PReg),
}

impl RegOperand {
    pub fn is_phys(&self) -> bool {
        matches!(self, Self::P(_))
    }
    pub fn is_virt(&self) -> bool {
        matches!(self, Self::V(_))
    }
    pub fn is_float(&self) -> bool {
        matches!(
            self,
            Self::V(VReg::Float(..)) | Self::P(PReg::V(..))
        )
    }

    pub fn get_bits(&self) -> u8 {
        match self {
            Self::V(vr) => vr.get_bits(),
            Self::P(pr) => pr.get_bits(),
        }
    }
    pub fn get_use_flags(&self) -> RegUseFlags {
        match self {
            Self::V(vr) => vr.get_use_flags(),
            Self::P(pr) => pr.get_use_flags(),
        }
    }
    pub fn use_flags_mut(&mut self) -> &mut RegUseFlags {
        match self {
            Self::V(vr) => vr.use_flags_mut(),
            Self::P(pr) => pr.use_flags_mut(),
        }
    }
    pub fn insert_use_flags(mut self, flags: RegUseFlags) -> Self {
        *self.use_flags_mut() = flags;
        self
    }

    fn try_set_subreg_index(&mut self, si: SubRegIndex) -> bool {
        let self_si = match self {
            Self::V(vreg) => vreg.subreg_index_mut(),
            Self::P(preg) => match preg {
                PReg::X(_, si, _)
                | PReg::V(_, si, _)
                | PReg::SP(si, _)
                | PReg::ZR(si, _)
                | PReg::PC(si, _) => si,
                PReg::PState(_) => return false,
            },
        };
        *self_si = si;
        true
    }
}

impl IMirSubOperand for RegOperand {
    fn from_mirop(operand: MirOperand) -> Self {
        match operand {
            MirOperand::PReg(r) => Self::P(r),
            MirOperand::VReg(r) => Self::V(r),
            _ => panic!("Expected a register operand, found: {operand:?}"),
        }
    }
    fn into_mirop(self) -> MirOperand {
        match self {
            Self::P(r) => MirOperand::PReg(r),
            Self::V(r) => MirOperand::VReg(r),
        }
    }
    fn insert_to_mirop(mut self, op: MirOperand) -> MirOperand {
        let uf = match op {
            MirOperand::PReg(preg) => match preg {
                PReg::X(_, si, uf)
                | PReg::V(_, si, uf)
                | PReg::SP(si, uf)
                | PReg::ZR(si, uf)
                | PReg::PC(si, uf) => {
                    self.try_set_subreg_index(si);
                    uf
                }
                PReg::PState(uf) => uf,
            },
            MirOperand::VReg(vreg) => {
                self.try_set_subreg_index(vreg.get_subreg_index());
                vreg.get_use_flags()
            }
            _ => {
                return self.into_mirop();
            }
        };
        self.insert_use_flags(uf).into_mirop()
    }

    fn new_empty_mirsubop() -> Self {
        RegOperand::V(VReg::new_empty_mirsubop())
    }
}

/// 限制操作数集合2: case 连续时 switch 跳转表在函数 switch 集合中的位置
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VecSwitchTabPos(u32);

impl IMirSubOperand for VecSwitchTabPos {
    fn from_mirop(operand: MirOperand) -> Self {
        match operand {
            MirOperand::VecSwitchTab(pos) => VecSwitchTabPos(pos),
            _ => panic!("Expected a VecSwitchTabPos, found: {:?}", operand),
        }
    }
    fn into_mirop(self) -> MirOperand {
        MirOperand::VecSwitchTab(self.0)
    }
    fn insert_to_mirop(self, _: MirOperand) -> MirOperand {
        MirOperand::VecSwitchTab(self.0)
    }

    fn new_empty_mirsubop() -> Self {
        VecSwitchTabPos(0)
    }
}

/// 限制操作数集合3: case 不连续时 switch 跳转表在函数二分查找 switch 集合中的位置
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BinSwitchTabPos(u32);

impl IMirSubOperand for BinSwitchTabPos {
    fn from_mirop(operand: MirOperand) -> Self {
        match operand {
            MirOperand::BinSwitchTab(pos) => BinSwitchTabPos(pos),
            _ => panic!("Expected a BinSwitchTabPos, found: {:?}", operand),
        }
    }
    fn into_mirop(self) -> MirOperand {
        MirOperand::BinSwitchTab(self.0)
    }
    fn insert_to_mirop(self, _: MirOperand) -> MirOperand {
        MirOperand::BinSwitchTab(self.0)
    }

    fn new_empty_mirsubop() -> Self {
        BinSwitchTabPos(0)
    }
}

/// 立即数位限制枚举.
/// 用于限制立即数的位数和范围.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImmBitLimit {
    Full,
    Hi20,
    Lo12,
}

/// 限制操作数集合4: 立即数或符号操作数
/// 包含立即数、符号引用和标签引用.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ImmSymOperand {
    Imm(i64),
    Sym(MirGlobalRef),
    Label(MirBlockRef),
    VecSwitchTabPos(u32),
    BinSwitchTabPos(u32),
}

impl IMirSubOperand for ImmSymOperand {
    fn from_mirop(operand: MirOperand) -> Self {
        match operand {
            MirOperand::Imm(value) => Self::Imm(value),
            MirOperand::Global(item_ref) => Self::Sym(item_ref),
            MirOperand::Label(block_ref) => Self::Label(block_ref),
            MirOperand::VecSwitchTab(pos) => Self::VecSwitchTabPos(pos),
            MirOperand::BinSwitchTab(pos) => Self::BinSwitchTabPos(pos),
            // 其他 MIR 操作数类型不符合要求
            _ => panic!(
                "Expected an immediate or symbol operand, found: {:?}",
                operand
            ),
        }
    }
    fn into_mirop(self) -> MirOperand {
        match self {
            Self::Imm(value) => MirOperand::Imm(value),
            Self::Sym(item_ref) => MirOperand::Global(item_ref),
            Self::Label(block_ref) => MirOperand::Label(block_ref),
            Self::VecSwitchTabPos(pos) => MirOperand::VecSwitchTab(pos),
            Self::BinSwitchTabPos(pos) => MirOperand::BinSwitchTab(pos),
        }
    }
    fn insert_to_mirop(self, _: MirOperand) -> MirOperand {
        self.into_mirop()
    }

    fn new_empty_mirsubop() -> Self {
        ImmSymOperand::Imm(0)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct PStateSubOperand;

impl IMirSubOperand for PStateSubOperand {
    fn from_mirop(operand: MirOperand) -> Self {
        match operand {
            MirOperand::PReg(PReg::PState(_)) => PStateSubOperand,
            _ => panic!("Expected a PState sub operand, found: {:?}", operand),
        }
    }
    fn into_mirop(self) -> MirOperand {
        MirOperand::PReg(PReg::PState(RegUseFlags::IMPLICIT_DEF))
    }
    fn insert_to_mirop(self, _: MirOperand) -> MirOperand {
        self.into_mirop()
    }

    fn new_empty_mirsubop() -> Self {
        PStateSubOperand
    }
}

impl IMirSubOperand for MirOperand {
    fn new_empty_mirsubop() -> Self {
        MirOperand::None
    }
    fn from_mirop(operand: MirOperand) -> Self {
        operand
    }
    fn into_mirop(self) -> MirOperand {
        self
    }
    fn insert_to_mirop(self, op: MirOperand) -> MirOperand {
        op
    }
}

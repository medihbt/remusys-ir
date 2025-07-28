use std::{
    cell::Cell,
    fmt::Debug,
    ops::{BitAnd, BitAndAssign, Sub, SubAssign},
};

use crate::mir::operand::{IMirSubOperand, MirOperand, reg::*};
#[derive(Clone, Copy, PartialEq, Eq)]
pub struct MirPhysRegSet {
    pub gpr_bitset: u32,
    pub fpr_bitset: u32,
}

impl Debug for MirPhysRegSet {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let &Self { gpr_bitset, fpr_bitset } = self;
        write!(
            f,
            "MirPhysRegSet {{ gpr: {:#032b}, fpr: {:#032b} }}",
            gpr_bitset, fpr_bitset
        )
    }
}

impl MirPhysRegSet {
    pub fn new_empty() -> Self {
        MirPhysRegSet { gpr_bitset: 0, fpr_bitset: 0 }
    }

    /// 根据 AAPCS64 ABI, 调用者需要保存的寄存器包括:
    ///
    /// #### GPRs
    ///
    /// - x0-x7: 可以用于传参/传递返回值
    /// - x8: 间接结果位置寄存器
    /// - x9-x15: 调用者保存的临时变量寄存器
    /// - x16,x17,x18: 平台特定寄存器
    /// - x30: 返回地址寄存器 -- 一般来说, 当函数中有 call 时, x30 在函数入口点就会保存。
    ///
    /// #### FPRs
    ///
    /// - v0-v7: 可以用于传参/传递返回值
    /// - v16-v31: 调用者保存的临时变量寄存器
    pub const fn new_aapcs_caller() -> Self {
        MirPhysRegSet {
            gpr_bitset: 0b01000000_00000111_11111111_11111111,
            fpr_bitset: 0b11111111_11111111_00000000_11111111,
        }
    }

    /// 根据 AAPCS64 ABI, 被调用者需要保存的寄存器包括:
    ///
    /// #### GPRs
    /// - x19-x28: 被调用者保存的通用寄存器
    /// - x29: 帧指针寄存器 (FP)
    /// - sp: 栈指针寄存器 (但通常不在此处管理)
    ///
    /// #### FPRs
    /// - v8-v15: 被调用者保存的浮点/向量寄存器
    pub const fn new_aapcs_callee() -> Self {
        Self {
            // x19-x28 (位19-28) + x29 (位29)
            gpr_bitset: 0b00111111_11111000_00000000_00000000,
            // v8-v15 (位8-15)
            fpr_bitset: 0b00000000_00000000_11111111_00000000,
        }
    }

    /// 尝试保存一个通用寄存器.
    /// 如果寄存器已经被保存或者寄存器 ID 是虚的, 则返回 `false`。否则保存寄存器并返回 `true` .
    pub const fn save_gpr(&mut self, pos: RegID) -> bool {
        let RegID::Phys(id) = pos else {
            return false; // 不能保存虚寄存器
        };
        if self.gpr_bitset & (1 << id) != 0 {
            return false; // 寄存器已经被保存
        }
        self.gpr_bitset |= 1 << id;
        true
    }

    /// 尝试保存一个浮点寄存器.
    /// 如果寄存器已经被保存或者寄存器 ID 是虚的,
    /// 则返回 `false`。否则保存寄存器并返回 `true`
    pub const fn save_fpr(&mut self, pos: RegID) -> bool {
        let RegID::Phys(id) = pos else {
            return false; // 不能保存虚寄存器
        };
        if self.fpr_bitset & (1 << id) != 0 {
            return false; // 寄存器已经被保存
        }
        self.fpr_bitset |= 1 << id;
        true
    }

    pub const fn save_reg(&mut self, reg_operand: RegOperand) -> bool {
        let RegOperand(id, _, _, is_fp) = reg_operand;
        if is_fp { self.save_fpr(RegID::Phys(id)) } else { self.save_gpr(RegID::Phys(id)) }
    }

    pub const fn unsave_gpr(&mut self, pos: RegID) -> bool {
        let RegID::Phys(id) = pos else {
            return false; // 不能取消保存虚寄存器
        };
        if self.gpr_bitset & (1 << id) == 0 {
            return false; // 寄存器没有被保存
        }
        self.gpr_bitset &= !(1 << id);
        true
    }
    pub const fn unsave_fpr(&mut self, pos: RegID) -> bool {
        let RegID::Phys(id) = pos else {
            return false; // 不能取消保存虚寄存器
        };
        if self.fpr_bitset & (1 << id) == 0 {
            return false; // 寄存器没有被保存
        }
        self.fpr_bitset &= !(1 << id);
        true
    }
    pub const fn unsave_reg(&mut self, reg_operand: RegOperand) -> bool {
        let RegOperand(id, _, _, is_fp) = reg_operand;
        if is_fp {
            self.unsave_fpr(RegID::from_real(id))
        } else {
            self.unsave_gpr(RegID::from_real(id))
        }
    }

    pub const fn insert_saved_gpr(mut self, pos: RegID) -> Self {
        self.save_gpr(pos);
        self
    }
    pub const fn insert_saved_fpr(mut self, pos: RegID) -> Self {
        self.save_fpr(pos);
        self
    }
    pub const fn insert_saved_reg(mut self, reg_operand: RegOperand) -> Self {
        self.save_reg(reg_operand);
        self
    }

    pub const fn has_saved_gpr(&self, pos: RegID) -> bool {
        let RegID::Phys(id) = pos else {
            return false; // 不能检查虚寄存器
        };
        self.gpr_bitset & (1 << id) != 0
    }
    pub const fn has_saved_fpr(&self, pos: RegID) -> bool {
        let RegID::Phys(id) = pos else {
            return false; // 不能检查虚寄存器
        };
        self.fpr_bitset & (1 << id) != 0
    }
    pub const fn has_saved_reg(&self, reg_operand: RegOperand) -> bool {
        let RegOperand(id, _, _, is_fp) = reg_operand;
        if is_fp {
            self.has_saved_fpr(RegID::Phys(id))
        } else {
            self.has_saved_gpr(RegID::Phys(id))
        }
    }

    pub const fn is_empty(&self) -> bool {
        self.gpr_bitset == 0 && self.fpr_bitset == 0
    }
    pub const fn num_gprs(&self) -> u32 {
        self.gpr_bitset.count_ones()
    }
    pub const fn num_fprs(&self) -> u32 {
        self.fpr_bitset.count_ones()
    }
    pub const fn num_regs(&self) -> u32 {
        self.num_gprs() + self.num_fprs()
    }

    pub const fn iter(self) -> MirPhysRegSetIter {
        MirPhysRegSetIter {
            gpr_bitset: self.gpr_bitset,
            fpr_bitset: self.fpr_bitset,
            index: 0,
        }
    }
    pub fn dump_regs(&self) -> Vec<RegOperand> {
        self.iter().collect()
    }
}

impl Sub for MirPhysRegSet {
    type Output = Self;

    fn sub(self, other: Self) -> Self {
        MirPhysRegSet {
            gpr_bitset: self.gpr_bitset & !other.gpr_bitset,
            fpr_bitset: self.fpr_bitset & !other.fpr_bitset,
        }
    }
}

impl SubAssign for MirPhysRegSet {
    fn sub_assign(&mut self, other: Self) {
        self.gpr_bitset &= !other.gpr_bitset;
        self.fpr_bitset &= !other.fpr_bitset;
    }
}

impl BitAnd for MirPhysRegSet {
    type Output = Self;

    fn bitand(self, other: Self) -> Self {
        MirPhysRegSet {
            gpr_bitset: self.gpr_bitset & other.gpr_bitset,
            fpr_bitset: self.fpr_bitset & other.fpr_bitset,
        }
    }
}

impl BitAndAssign for MirPhysRegSet {
    fn bitand_assign(&mut self, other: Self) {
        self.gpr_bitset &= other.gpr_bitset;
        self.fpr_bitset &= other.fpr_bitset;
    }
}

impl IntoIterator for MirPhysRegSet {
    type Item = RegOperand;
    type IntoIter = MirPhysRegSetIter;

    fn into_iter(self) -> MirPhysRegSetIter {
        MirPhysRegSetIter {
            gpr_bitset: self.gpr_bitset,
            fpr_bitset: self.fpr_bitset,
            index: 0,
        }
    }
}

impl IntoIterator for &MirPhysRegSet {
    type Item = RegOperand;
    type IntoIter = MirPhysRegSetIter;

    fn into_iter(self) -> MirPhysRegSetIter {
        MirPhysRegSetIter {
            gpr_bitset: self.gpr_bitset,
            fpr_bitset: self.fpr_bitset,
            index: 0,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct MirPhysRegSetIter {
    pub gpr_bitset: u32,
    pub fpr_bitset: u32,
    pub index: u8,
}

impl Iterator for MirPhysRegSetIter {
    type Item = RegOperand;

    fn next(&mut self) -> Option<RegOperand> {
        while self.index < 64 {
            let (bitset, is_fp, id) = if self.index < 32 {
                (self.gpr_bitset, false, self.index)
            } else {
                (self.fpr_bitset, true, self.index - 32)
            };
            self.index += 1;

            if bitset & (1 << id) != 0 {
                let sub_index = SubRegIndex::new(6, 0);
                let reg_operand = RegOperand(id as u32, sub_index, RegUseFlags::KILL, is_fp);
                return Some(reg_operand);
            }
        }
        None // 超出寄存器范围或没有更多寄存器
    }
}

impl FromIterator<RegOperand> for MirPhysRegSet {
    fn from_iter<I: IntoIterator<Item = RegOperand>>(iter: I) -> Self {
        let mut set = MirPhysRegSet::new_empty();
        for reg in iter {
            set.save_reg(reg);
        }
        set
    }
}

impl<const N: usize> From<&[RegOperand; N]> for MirPhysRegSet {
    fn from(array: &[RegOperand; N]) -> Self {
        let mut set = MirPhysRegSet::new_empty();
        for &reg in array {
            set.save_reg(reg);
        }
        set
    }
}
impl<const N: usize> From<&[GPReg; N]> for MirPhysRegSet {
    fn from(array: &[GPReg; N]) -> Self {
        let mut set = MirPhysRegSet::new_empty();
        for reg in array {
            set.save_gpr(RegID::from_real(reg.get_id_raw()));
        }
        set
    }
}
impl<const N: usize> From<&[VFReg; N]> for MirPhysRegSet {
    fn from(array: &[VFReg; N]) -> Self {
        let mut set = MirPhysRegSet::new_empty();
        for reg in array {
            set.save_fpr(RegID::from_real(reg.get_id_raw()));
        }
        set
    }
}

#[derive(Debug, Clone)]
pub struct RegOperandSet {
    _operands: Box<[Cell<MirOperand>; 64]>,
    _noperands: Cell<usize>,
    _physreg_set: Cell<MirPhysRegSet>,
    _use_flags: RegUseFlags,
}

impl RegOperandSet {
    pub fn new(use_flags: RegUseFlags) -> Self {
        RegOperandSet {
            _operands: Box::new([const { Cell::new(MirOperand::None) }; 64]),
            _noperands: Cell::new(0),
            _physreg_set: Cell::new(MirPhysRegSet::new_empty()),
            _use_flags: use_flags,
        }
    }

    pub fn update(&self, phys_set: MirPhysRegSet) {
        let old = self._physreg_set.get();
        if old == phys_set {
            return; // 没有变化
        }
        self._physreg_set.set(phys_set);
        // 更新寄存器操作数
        for (i, reg) in phys_set.iter().enumerate() {
            debug_assert!(i < 64, "Too many saved registers: {i}");
            let RegOperand(id, _, _, is_fp) = reg;
            let operand = if is_fp {
                VFReg::new_double(RegID::from_real(id))
                    .insert_use_flags(self._use_flags)
                    .into_mir()
            } else {
                GPReg::new_long(RegID::from_real(id))
                    .insert_use_flags(self._use_flags)
                    .into_mir()
            };
            self._operands[i].set(operand);
        }
        self._noperands.set(phys_set.num_regs() as usize);
    }

    pub fn operands(&self) -> &[Cell<MirOperand>] {
        &self._operands[..self._noperands.get()]
    }
}

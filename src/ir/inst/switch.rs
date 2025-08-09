use slab::Slab;

use crate::{
    base::INullableValue,
    ir::{
        BlockData, BlockRef, IRAllocs, IRWriter, ISubInst, ISubValueSSA, ITerminatorInst,
        InstCommon, InstData, InstRef, JumpTarget, JumpTargetKind, Opcode, Use, UseKind, ValueSSA,
        block::jump_target::JumpTargets,
        inst::{ISubInstRef, InstOperands},
    },
    typing::id::ValTypeID,
};
use std::{
    cell::{Ref, RefCell},
    rc::Rc,
};

/// Switch 指令：实现 LLVM IR 中的 switch 语句，根据条件值跳转到不同的基本块
///
/// ### LLVM IR 语法
///
/// ```llvm
/// switch <intty> <value>, label <defaultdest> [
///     <intty> <val0>, label <dest0>
///     <intty> <val1>, label <dest1>
///     <intty> <val2>, label <dest2>
///     ...
/// ]
/// ```
///
/// ### 操作数布局
///
/// - **条件操作数**: 一个整数类型的值，用于匹配各个 case
///
/// ### 跳转目标布局
///
/// - **targets[0]**: 默认跳转目标 (`JumpTargetKind::SwitchDefault`)
/// - **targets[1..]**: 各个 case 跳转目标 (`JumpTargetKind::SwitchCase(value)`)
///
/// ### 语义
///
/// 1. 计算条件操作数的值
/// 2. 按顺序查找匹配的 case 值
/// 3. 如果找到匹配的 case，跳转到对应的基本块
/// 4. 如果没有匹配的 case，跳转到默认基本块
///
/// ## 约束
///
/// - 条件操作数必须是整数类型
/// - 每个 case 值必须唯一
/// - 必须有且仅有一个默认跳转目标
#[derive(Debug)]
pub struct Switch {
    /// 指令的公共数据（操作码、类型、用户列表等）
    common: InstCommon,
    /// 条件操作数：要匹配的整数值
    cond: [Rc<Use>; 1],
    /// 跳转目标列表：[0] 是默认目标，[1..] 是各个 case 目标
    targets: RefCell<Vec<Rc<JumpTarget>>>,
}

impl ISubInst for Switch {
    fn new_empty(_: Opcode) -> Self {
        Self {
            common: InstCommon::new(Opcode::Switch, ValTypeID::Void),
            cond: [Use::new(UseKind::BranchCond)],
            targets: RefCell::new(vec![JumpTarget::new(JumpTargetKind::SwitchDefault)]),
        }
    }
    fn try_from_ir(inst: &InstData) -> Option<&Self> {
        match inst {
            InstData::Switch(switch) => Some(switch),
            _ => None,
        }
    }
    fn try_from_ir_mut(inst: &mut InstData) -> Option<&mut Self> {
        match inst {
            InstData::Switch(switch) => Some(switch),
            _ => None,
        }
    }
    fn into_ir(self) -> InstData {
        InstData::Switch(self)
    }
    fn get_common(&self) -> &InstCommon {
        &self.common
    }
    fn common_mut(&mut self) -> &mut InstCommon {
        &mut self.common
    }

    fn is_terminator(&self) -> bool {
        true
    }
    fn get_operands(&self) -> InstOperands {
        InstOperands::Fixed(&self.cond)
    }
    fn operands_mut(&mut self) -> &mut [Rc<Use>] {
        &mut self.cond
    }

    fn init_self_reference(&mut self, self_ref: InstRef) {
        InstData::basic_init_self_reference(self_ref, self);
        // 设置所有跳转目标的终结指令引用
        for jt in &self.get_jts() {
            jt.terminator.set(self_ref);
        }
    }

    fn fmt_ir(&self, _: Option<usize>, writer: &IRWriter) -> std::io::Result<()> {
        write!(writer, "switch ")?;

        // 写入条件操作数的类型和值
        let cond = self.get_cond();
        let cond_type = cond.get_valtype(&writer.allocs);
        writer.write_type(cond_type)?;
        write!(writer, " ")?;
        writer.write_operand(cond)?;

        // 写入默认跳转目标
        write!(writer, ", label ")?;
        writer.write_operand(self.get_default())?;

        // 写入各个 case
        write!(writer, " [")?;
        writer.inc_indent();
        for case in self.cases().iter() {
            let JumpTargetKind::SwitchCase(case_value) = case.kind else {
                continue;
            };
            writer.wrap_indent();
            writer.write_type(cond_type)?;
            write!(writer, " {}, label ", case_value)?;
            writer.write_operand(case.get_block())?;
        }
        writer.dec_indent();
        writer.wrap_indent();
        // 结束 case 列表
        write!(writer, "]")
    }

    fn cleanup(&self) {
        InstData::basic_cleanup(self);
        // 清理跳转目标
        for jt in &*self.targets.borrow() {
            jt.clean_block();
        }
    }
}

impl ITerminatorInst for Switch {
    fn read_jts<T>(&self, reader: impl FnOnce(&[Rc<JumpTarget>]) -> T) -> T {
        reader(&self.targets.borrow())
    }

    fn jts_mut(&mut self) -> &mut [Rc<JumpTarget>] {
        self.targets.get_mut()
    }

    fn get_jts(&self) -> JumpTargets {
        JumpTargets::AsRef(self.targets.borrow())
    }
}

impl Switch {
    /// 创建一个新的 Switch 指令
    ///
    /// # 参数
    /// - `allocs`: IR 分配器，用于类型检查
    /// - `cond`: 条件操作数，必须是整数类型
    ///
    /// # Panics
    /// 如果条件操作数不是整数类型则会 panic
    pub fn new(allocs: &IRAllocs, cond: ValueSSA, default: BlockRef) -> Self {
        let mut switch = Self::new_empty(Opcode::Switch);
        switch.set_cond(allocs, cond);
        switch.set_default(&allocs.blocks, default);
        switch
    }

    /// 获取条件操作数的 Use 引用
    pub fn cond(&self) -> &Rc<Use> {
        &self.cond[0]
    }

    /// 获取条件操作数的值
    pub fn get_cond(&self) -> ValueSSA {
        self.cond[0].get_operand()
    }

    /// 设置条件操作数
    ///
    /// # 参数
    /// - `allocs`: IR 分配器，用于类型检查
    /// - `cond`: 新的条件操作数
    ///
    /// # Panics
    /// 如果条件操作数不是整数类型则会 panic
    pub fn set_cond(&mut self, allocs: &IRAllocs, cond: ValueSSA) {
        if cond != ValueSSA::None && !matches!(cond.get_valtype(allocs), ValTypeID::Int(_)) {
            panic!(
                "Switch condition must be an integer type, got: {:?}",
                cond.get_valtype(allocs)
            );
        }
        self.cond[0].set_operand(allocs, cond);
    }

    /// 获取默认跳转目标的引用
    pub fn default(&self) -> Ref<Rc<JumpTarget>> {
        Ref::map(self.targets.borrow(), |targets| &targets[0])
    }

    /// 克隆默认跳转目标
    pub fn clone_default(&self) -> Rc<JumpTarget> {
        self.default().clone()
    }

    /// 获取默认跳转目标的基本块
    pub fn get_default(&self) -> BlockRef {
        self.default().get_block()
    }

    /// 设置默认跳转目标的基本块
    pub fn set_default(&self, alloc: &Slab<BlockData>, block: BlockRef) {
        self.default().set_block(alloc, block);
    }

    /// 获取所有 case 跳转目标的引用（不包括默认目标）
    pub fn cases(&self) -> Ref<[Rc<JumpTarget>]> {
        Ref::map(self.targets.borrow(), |targets| &targets[1..])
    }

    /// 查找指定 case 值的跳转目标引用
    ///
    /// # 修复说明
    /// 原实现有嵌套借用问题，现在直接从 targets 中查找
    pub fn ref_case<T: Into<i128>>(&self, case: T) -> Option<Ref<Rc<JumpTarget>>> {
        let case_value = case.into();
        let targets = self.targets.borrow();

        // 从索引1开始查找（索引0是默认目标）
        for (idx, jt) in targets.iter().enumerate().skip(1) {
            if jt.kind == JumpTargetKind::SwitchCase(case_value) {
                return Some(Ref::map(targets, |targets| &targets[idx]));
            }
        }
        None
    }

    /// 获取指定 case 值的目标基本块
    pub fn get_case<T: Into<i128>>(&self, case: T) -> Option<BlockRef> {
        self.ref_case(case).map(|jt| jt.get_block())
    }

    /// 设置已存在的 case 的目标基本块
    ///
    /// # 参数
    /// - `case`: case 值
    /// - `block`: 新的目标基本块
    ///
    /// # 返回值
    /// 如果 case 存在则返回 `true`，否则返回 `false`
    pub fn set_existing_case<T: Into<i128>>(
        &self,
        alloc: &Slab<BlockData>,
        case: T,
        block: BlockRef,
    ) -> bool {
        let case_value = case.into();
        if let Some(case_ref) = self.ref_case(case_value) {
            case_ref.set_block(alloc, block);
            true
        } else {
            false
        }
    }

    /// 设置 case 的目标基本块（如果不存在则创建新的 case）
    ///
    /// # 参数
    /// - `case`: case 值
    /// - `block`: 目标基本块
    ///
    /// # 返回值
    /// 返回对应的跳转目标引用
    ///
    /// # 注意
    /// 这个方法确保不会有重复的 case 值
    pub fn set_case<T: Into<i128>>(
        &self,
        alloc: &Slab<BlockData>,
        case: T,
        block: BlockRef,
    ) -> Rc<JumpTarget> {
        let case_value = case.into();
        if let Some(existing_case) = self.ref_case(case_value) {
            // 更新已存在的 case
            existing_case.set_block(alloc, block);
            existing_case.clone()
        } else {
            // 创建新的 case
            let new_case = JumpTarget::new(JumpTargetKind::SwitchCase(case_value));
            new_case.set_block(alloc, block);
            new_case.terminator.set(self.get_common().self_ref);
            self.targets.borrow_mut().push(new_case.clone());
            new_case
        }
    }

    /// 移除指定的 case
    ///
    /// # 参数
    /// - `case`: 要移除的 case 值
    ///
    /// # 返回值
    /// 如果成功移除则返回 `true`，否则返回 `false`
    ///
    /// # 修复说明
    /// 使用 `rposition` 从后往前查找，避免重复 case 时的问题
    pub fn remove_case<T: Into<i128>>(&self, case: T) -> bool {
        let case_value = case.into();
        let mut targets = self.targets.borrow_mut();

        // 从后往前查找，避免索引变化的问题
        if let Some(index) = targets
            .iter()
            .rposition(|jt| jt.kind == JumpTargetKind::SwitchCase(case_value))
        {
            // 在移除前清理跳转目标的基本块引用
            targets[index].set_block(&Slab::new(), BlockRef::new_null());
            targets.remove(index);
            true
        } else {
            false
        }
    }

    /// 根据条件移除多个 case
    ///
    /// # 参数
    /// - `alloc`: 基本块分配器，用于清理引用
    /// - `condition`: 判断函数，返回 `true` 的 case 将被移除
    ///
    /// # 注意
    /// 默认跳转目标永远不会被移除
    pub fn remove_cases_when(
        &self,
        alloc: &Slab<BlockData>,
        condition: impl Fn(&Rc<JumpTarget>) -> bool,
    ) {
        let mut targets = self.targets.borrow_mut();
        targets.retain(|jt| {
            if jt.kind == JumpTargetKind::SwitchDefault || !condition(jt) {
                true
            } else {
                // 在移除前清理跳转目标的基本块引用
                jt.set_block(alloc, BlockRef::new_null());
                false
            }
        });
    }
}

/// Switch 指令的强类型引用
///
/// 这是对 `InstRef` 的类型安全封装，确保引用的指令确实是 Switch 类型。
/// 提供了类型安全的方法来访问和操作 Switch 指令。
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SwitchRef(InstRef);

impl ISubInstRef for SwitchRef {
    type InstDataT = Switch;
    fn from_raw_nocheck(inst_ref: InstRef) -> Self {
        Self(inst_ref)
    }
    fn into_raw(self) -> InstRef {
        self.0
    }
}

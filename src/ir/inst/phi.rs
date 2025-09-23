use crate::{
    base::INullableValue,
    ir::{
        BlockRef, IRAllocs, IRWriter, ISubInst, ISubValueSSA, IUser, InstCommon, InstData, InstRef,
        Opcode, OperandSet, Use, UseKind, UserID, ValueSSA, inst::ISubInstRef,
    },
    typing::ValTypeID,
};
use std::{
    cell::{Ref, RefCell},
    rc::Rc,
};

#[derive(Debug, Clone)]
pub enum PhiError {
    IncomeBBNotFound(BlockRef),
    DuplicatedIncomeBB(BlockRef),
    InvalidUseType,
}

pub type PhiRes<T = ()> = Result<T, PhiError>;

/// Phi 节点：实现 SSA 形式中的 φ 函数
///
/// ## LLVM IR 语法
/// ```llvm
/// %<result> = phi <type> [ <value0>, %<label0> ], [ <value1>, %<label1> ], ...
/// ```
///
/// ## 操作数布局
/// Phi 节点的操作数以成对形式存储：
/// - `operands[2i]`: 第 i 个传入值 (value)
/// - `operands[2i + 1]`: 第 i 个来源基本块 (block)
///
/// ## 内部数据结构
/// - **incoming_map**: 将基本块映射到操作数索引对 `(value_idx, block_idx)`
/// - **operands**: 动态数组存储所有操作数的 Use 引用
///
/// ## 语义
/// 根据控制流的来源选择相应的值：
/// 1. 在运行时，根据前一个执行的基本块
/// 2. 从对应的传入值中选择一个作为 Phi 节点的结果
/// 3. 所有传入值必须与 Phi 节点具有相同的类型
///
/// ## 约束
/// - 每个前驱基本块最多只能有一个对应的传入值
/// - 所有传入值的类型必须与 Phi 节点的返回类型相同
/// - Phi 节点必须出现在基本块的开始位置（在非 Phi 指令之前）
#[derive(Debug)]
pub struct PhiNode {
    /// 指令的公共数据（操作码、类型、用户列表等）
    common: InstCommon,
    /// 操作数列表：包含值和基本块的 Use 引用，按 [value, block, value, block, ...] 的模式排列
    incomes: RefCell<Vec<[Rc<Use>; 2]>>,
}

impl IUser for PhiNode {
    fn get_operands(&self) -> OperandSet<'_> {
        let operands = self.incomes.borrow();
        let operands = Ref::map(operands, |ops| ops.as_flattened());
        OperandSet::InRef(operands)
    }

    fn operands_mut(&mut self) -> &mut [Rc<Use>] {
        self.incomes.get_mut().as_flattened_mut()
    }
}

impl ISubInst for PhiNode {
    fn new_empty(opcode: Opcode) -> Self {
        if opcode != Opcode::Phi {
            panic!("Tried to create a PhiNode with non-Phi opcode");
        }
        Self {
            common: InstCommon::new(opcode, ValTypeID::Void),
            incomes: RefCell::new(Vec::new()),
        }
    }

    fn try_from_ir(inst: &InstData) -> Option<&Self> {
        if let InstData::Phi(phi) = inst { Some(phi) } else { None }
    }
    fn try_from_ir_mut(inst: &mut InstData) -> Option<&mut Self> {
        if let InstData::Phi(phi) = inst { Some(phi) } else { None }
    }

    fn into_ir(self) -> InstData {
        InstData::Phi(self)
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
            return Err(Error::new(
                InvalidInput,
                "ID must be provided for Phi instruction",
            ));
        };

        // 写入指令格式: %id = phi <type>
        let opcode = self.get_opcode().get_name();
        write!(writer, "%{} = {} ", id, opcode)?;
        writer.write_type(self.common.ret_type)?;
        writer.write_str(" ")?;

        // 写入所有 incoming values: [ <value>, %<label> ], ...
        let incomes = self.incoming_uses();
        for (i, [uval, ublk]) in incomes.iter().enumerate() {
            if i > 0 {
                writer.write_str(", ")?;
            }
            writer.write_str("[ ")?;

            // 写入值
            let incoming_val = uval.get_operand();
            writer.write_operand(incoming_val)?;

            // 写入来源基本块
            writer.write_str(", ")?;
            writer.write_operand(ublk.get_operand())?;

            writer.write_str(" ]")?;
        }
        Ok(())
    }
}

impl PhiNode {
    pub fn new(ret_type: ValTypeID) -> Self {
        Self {
            common: InstCommon::new(Opcode::Phi, ret_type),
            incomes: RefCell::new(Vec::new()),
        }
    }

    pub fn incoming_uses(&self) -> Ref<'_, [[Rc<Use>; 2]]> {
        Ref::map(self.incomes.borrow(), |ops| ops.as_slice())
    }
    pub fn locate_income_group(&self, block: BlockRef) -> Option<usize> {
        for (index, [_, blk]) in self.incoming_uses().iter().enumerate() {
            if blk.get_operand() == ValueSSA::Block(block) {
                return Some(index);
            }
        }
        None
    }
    pub fn locate_incomes(&self, block: BlockRef) -> Option<(usize, usize)> {
        self.locate_income_group(block)
            .map(|index| (index * 2, index * 2 + 1))
    }
    pub fn find_income(&self, block: BlockRef) -> Option<Ref<'_, [Rc<Use>; 2]>> {
        self.locate_income_group(block)
            .map(|index| Ref::map(self.incomes.borrow(), |ops| &ops[index]))
    }
    pub fn has_income(&self, block: BlockRef) -> bool {
        self.locate_income_group(block).is_some()
    }
    pub fn get_income_block_use(&self, block: BlockRef) -> Option<Rc<Use>> {
        self.find_income(block).and_then(|income| {
            let [_, block_use] = &*income;
            Some(block_use.clone())
        })
    }
    pub fn get_income_value_use(&self, block: BlockRef) -> Option<Rc<Use>> {
        self.find_income(block).and_then(|income| {
            let [value_use, _] = &*income;
            Some(value_use.clone())
        })
    }
    pub fn get_income_value(&self, block: BlockRef) -> Option<ValueSSA> {
        self.locate_income_group(block).map(|index| {
            let ops = self.incomes.borrow();
            ops[index][0].get_operand()
        })
    }

    pub fn income_block_at(&self, index: usize) -> BlockRef {
        let income = self.incoming_uses();
        let [_, ublk] = &income[index];
        BlockRef::from_ir(ublk.get_operand())
    }
    pub fn income_value_at(&self, index: usize) -> ValueSSA {
        let income = self.incoming_uses();
        let [uval, _] = &income[index];
        uval.get_operand()
    }

    pub fn set_existing_income(
        &self,
        allocs: &IRAllocs,
        block: BlockRef,
        value: ValueSSA,
    ) -> PhiRes {
        let Some(group) = self.find_income(block) else {
            return Err(PhiError::IncomeBBNotFound(block));
        };
        let [value_use, block_use] = &*group;
        if block_use.get_operand() != ValueSSA::Block(block) {
            return Err(PhiError::InvalidUseType);
        }
        value_use.set_operand(allocs, value);
        Ok(())
    }

    /// 为指定前驱基本块设置一个传入值, 如果该基本块已经存在传入值则覆盖.
    /// 如果该基本块不存在传入值则新增一对传入值和传入基本块操作数.
    pub fn set_income(&self, allocs: &IRAllocs, block: BlockRef, value: ValueSSA) -> PhiRes {
        match self.set_existing_income(allocs, block, value) {
            Ok(()) => return Ok(()),
            Err(PhiError::IncomeBBNotFound(_)) => {}
            Err(e) => return Err(e),
        };

        let mut operands = self.incomes.borrow_mut();
        let group_index = operands.len();

        // 构建新的操作数对：使用组索引来标识配对关系
        // group_index 用于在删除时快速定位配对的操作数
        let value_use = Use::new(UseKind::PhiIncomingValue(group_index as u32));
        let block_use = Use::new(UseKind::PhiIncomingBlock(group_index as u32));

        // 维护 use-def 的反向引用关系
        let self_id = self.get_self_ref();
        let self_id = if self_id.is_null() { UserID::None } else { UserID::Inst(self_id) };
        value_use.user.set(self_id);
        block_use.user.set(self_id);

        value_use.set_operand(allocs, value);
        block_use.set_operand(allocs, ValueSSA::Block(block));

        // 将新的操作数对添加到操作数列表中
        operands.push([value_use, block_use]);

        Ok(())
    }

    /// 移除指定前驱基本块的传入值, 如果该基本块不存在传入值则返回错误.
    /// 移除时会保持操作数列表的紧凑性, 通过与末尾元素交换并弹出末尾元素来实现.
    /// 这种方式会改变被交换元素的索引, 因此需要更新它们的 UseKind 以反映新的索引.
    pub fn remove_income(&self, block: BlockRef) -> PhiRes {
        let Some(group_index) = self.locate_income_group(block) else {
            return Err(PhiError::IncomeBBNotFound(block));
        };
        self.remove_income_index(group_index);
        Ok(())
    }

    /// 移除指定组索引下的基本块传入值.
    /// 移除时会保持操作数列表的紧凑性, 通过与末尾元素交换并弹出末尾元素来实现.
    /// 这种方式会改变被交换元素的索引, 因此需要更新它们的 UseKind 以反映新的索引.
    pub fn remove_income_index(&self, group_index: usize) {
        let mut incomes = self.incomes.borrow_mut();
        // 清理操作数, 同时解除掉相关的 Use 关系.
        // 由于 UserList 良好的性质, 这里不需要借助 IRAllocs.
        let [val, blk] = &incomes[group_index];
        val.clean_operand();
        blk.clean_operand();

        if group_index != incomes.len() - 1 {
            let back_index = incomes.len() - 1;
            incomes.swap(group_index, back_index);
            // 更新被移动到 group_index 位置的操作数的 UseKind
            let [uval, ublk] = &incomes[group_index];
            uval.kind.set(UseKind::PhiIncomingValue(group_index as u32));
            ublk.kind.set(UseKind::PhiIncomingBlock(group_index as u32));
        }
        incomes.pop();
    }

    pub fn retain_valid_income(&self) {
        let mut incomes = self.incomes.borrow_mut();
        let mut index = 0;
        while index < incomes.len() {
            if incomes[index][1].get_operand().is_nonnull() {
                index += 1;
                continue;
            }
            let [val, blk] = if index == incomes.len() - 1 {
                let Some(pair) = incomes.pop() else {
                    return;
                };
                pair
            } else {
                let pair = incomes.swap_remove(index);
                let [val, blk] = &incomes[index];
                val.kind.set(UseKind::PhiIncomingValue(index as u32));
                blk.kind.set(UseKind::PhiIncomingBlock(index as u32));
                pair
            };
            val.clean_operand();
            blk.clean_operand();
        }
    }

    /// 重定向 Phi 节点中某个前驱基本块的引用到新的基本块
    ///
    /// 此方法用于基本块拆分、边拆分等编译器优化场景，将 Phi 节点中原本指向
    /// 某个前驱基本块的传入值重定向到新的基本块。
    ///
    /// ### 参数
    /// - `allocs`: IR 分配器，用于更新操作数引用
    /// - `income_bb_use`: 指向要重定向的基本块的 Use 引用，必须是 `PhiIncomingBlock` 类型
    /// - `new_block`: 新的目标基本块引用
    ///
    /// ### 返回值
    /// - `Ok(())`: 重定向成功
    /// - `Err(PhiError::DuplicatedIncomeBB)`: 新基本块已经存在于 Phi 节点中
    /// - `Err(PhiError::InvalidUseType)`: 传入的 Use 引用不是 `PhiIncomingBlock` 类型
    /// - `Err(PhiError::IncomeBBNotFound)`: 旧基本块不在 Phi 节点的前驱列表中
    ///
    /// ### 操作步骤
    /// 1. 检查新基本块是否已存在，避免重复添加
    /// 2. 验证传入的 Use 引用类型，确保是基本块引用而非值引用
    /// 3. 从 `incoming_map` 中移除旧基本块的映射关系
    /// 4. 更新对应的 `PhiIncomingValue` 的 UseKind，将其组索引更新
    /// 5. 更新基本块操作数的实际值，指向新基本块
    /// 6. 在 `incoming_map` 中插入新的映射关系
    ///
    /// ### 原子性保证
    /// 此操作具有原子性：要么完全成功，要么在任何步骤失败时保持原始状态不变。
    /// 在检查阶段发现问题会提前返回错误，不会修改任何内部状态。
    ///
    /// ### 内部一致性
    /// 函数确保 `operands` 数组、`incoming_map` 映射和 `UseKind` 枚举值之间的
    /// 一致性，维护 Phi 节点内部数据结构的完整性。
    pub fn redirect_income(
        &self,
        allocs: &IRAllocs,
        income_bb_use: &Rc<Use>,
        new_block: BlockRef,
    ) -> Result<(), PhiError> {
        self.do_redirect_income(RedirectAction::Full(allocs), income_bb_use, new_block)
    }

    /// 重定向 Phi 节点中某个前驱基本块的引用到新的基本块，但不更新 UserList
    ///
    /// ### ⚠️ 危险函数：调用者责任
    ///
    /// 此函数直接设置操作数字段而不进行 use-def 关系维护，
    /// 调用者必须确保在调用此函数之前已经正确维护了 UserList 关系。
    ///
    /// ### 参数
    /// - `income_bb_use`: 指向要重定向的基本块的 Use 引用，必须是 `PhiIncomingBlock` 类型
    /// - `new_block`: 新的目标基本块引用
    ///
    /// ### 前置条件（调用者保证）
    /// 1. `income_bb_use` 已经从旧基本块的 UserList 中移除
    /// 2. `income_bb_use` 已经添加到新基本块的 UserList 中
    /// 3. 调用者确保不会有悬空的 use-def 引用
    ///
    /// ### 返回值
    /// - `Ok(())`: 重定向成功
    /// - `Err(PhiError::*)`: 各种验证失败
    #[allow(dead_code)]
    pub(crate) unsafe fn redirect_income_operand_only(
        &self,
        income_bb_use: &Rc<Use>,
        new_block: BlockRef,
    ) -> PhiRes {
        self.do_redirect_income(RedirectAction::SetOnly, income_bb_use, new_block)
    }

    fn do_redirect_income(
        &self,
        action: RedirectAction,
        income_bb_use: &Rc<Use>,
        new_block: BlockRef,
    ) -> PhiRes {
        // 检查新基本块是否已经存在于 Phi 节点中. 每个前驱基本块在 Phi 节点中只能有一个对应的传入值
        // 就算 redirect_income 是为了合并两个分支也不行, 因为这时两个 income value 有冲突, PhiNode
        // 自身无法解决.
        if self.has_income(new_block) {
            return Err(PhiError::DuplicatedIncomeBB(new_block));
        }

        // 验证传入的 Use 引用类型
        let UseKind::PhiIncomingBlock(group_idx) = income_bb_use.kind.get() else {
            return Err(PhiError::InvalidUseType);
        };

        // 更新对应的 UseKind, 将其关联的基本块从 old_block 改为 new_block
        let incomes = self.incoming_uses();
        let [_, ublk] = &incomes[group_idx as usize];

        // 验证传入的 Use 引用确实是我们找到的那个
        debug_assert!(
            Rc::ptr_eq(ublk, income_bb_use),
            "income_bb_use should match the block use at group_idx"
        );

        // 更新基本块操作数的实际值，将基本块引用从 old_block 改为 new_block
        match action {
            RedirectAction::Full(allocs) => {
                // 注意：即使 income_bb_use 已经被预先移动到 new_block 的 UserList 中，
                // set_operand 也能正确处理：detach() 会从当前实际所在的列表中移除，
                // 然后 add_user() 会重新添加到目标列表中，确保 use-def 关系正确
                income_bb_use.set_operand(allocs, ValueSSA::Block(new_block));
            }
            RedirectAction::SetOnly => {
                // 仅更新操作数, 不进行 UserList 的移动
                // 这种方式适用于已经在新基本块的 UserList 中的情况
                // 但需要确保 income_bb_use 已经被移动到 new_block 的 UserList
                income_bb_use.operand.set(ValueSSA::Block(new_block));
            }
        }

        Ok(())
    }
}

enum RedirectAction<'a> {
    Full(&'a IRAllocs),
    SetOnly,
}

pub struct PhiIncomeIter<'a> {
    operands: Ref<'a, [[Rc<Use>; 2]]>,
    index: usize,
}

impl<'a> Iterator for PhiIncomeIter<'a> {
    type Item = (ValueSSA, BlockRef);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.operands.len() {
            return None; // No more operands
        }
        let [value_use, block_use] = &self.operands[self.index];
        self.index += 1; // Move to the next pair
        Some((
            value_use.get_operand(),
            BlockRef::from_ir(block_use.get_operand()),
        ))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.operands.len() - self.index;
        (remaining, Some(remaining))
    }
}

impl<'a> IntoIterator for &'a PhiNode {
    type Item = (ValueSSA, BlockRef);
    type IntoIter = PhiIncomeIter<'a>;

    fn into_iter(self) -> Self::IntoIter {
        let operands = self.incomes.borrow();
        let operands = Ref::map(operands, |ops| ops.as_slice());
        PhiIncomeIter { operands, index: 0 }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct PhiRef(pub InstRef);

impl ISubInstRef for PhiRef {
    type InstDataT = PhiNode;
    fn from_raw_nocheck(inst_ref: InstRef) -> Self {
        Self(inst_ref)
    }
    fn into_raw(self) -> InstRef {
        self.0
    }
}

impl PhiRef {
    /// 获取所有传入的基本块和对应的值
    pub fn incoming_uses(self, allocs: &IRAllocs) -> Ref<'_, [[Rc<Use>; 2]]> {
        self.to_inst(&allocs.insts).incoming_uses()
    }

    /// 定位指定基本块在传入列表中的索引
    pub fn locate_income_group(self, allocs: &IRAllocs, block: BlockRef) -> Option<usize> {
        self.to_inst(&allocs.insts).locate_income_group(block)
    }

    /// 定位指定基本块在传入列表中的值和基本块操作数索引
    pub fn locate_incomes(self, allocs: &IRAllocs, block: BlockRef) -> Option<(usize, usize)> {
        self.to_inst(&allocs.insts).locate_incomes(block)
    }

    /// 查找指定基本块的传入值和基本块操作数
    pub fn find_income(self, allocs: &IRAllocs, block: BlockRef) -> Option<Ref<'_, [Rc<Use>; 2]>> {
        self.to_inst(&allocs.insts).find_income(block)
    }

    /// 检查指定基本块是否存在传入值
    pub fn has_income(self, allocs: &IRAllocs, block: BlockRef) -> bool {
        self.to_inst(&allocs.insts).has_income(block)
    }

    /// 获取指定基本块的传入基本块操作数引用
    pub fn get_income_block_use(self, allocs: &IRAllocs, block: BlockRef) -> Option<Rc<Use>> {
        self.to_inst(&allocs.insts).get_income_block_use(block)
    }

    /// 获取指定基本块的传入值操作数引用
    pub fn get_income_value_use(self, allocs: &IRAllocs, block: BlockRef) -> Option<Rc<Use>> {
        self.to_inst(&allocs.insts).get_income_value_use(block)
    }

    /// 获取指定基本块的传入值
    pub fn get_income_value(self, allocs: &IRAllocs, block: BlockRef) -> Option<ValueSSA> {
        self.to_inst(&allocs.insts).get_income_value(block)
    }

    /// 获取指定索引处的传入基本块
    pub fn income_block_at(self, allocs: &IRAllocs, index: usize) -> BlockRef {
        self.to_inst(&allocs.insts).income_block_at(index)
    }

    /// 获取指定索引处的传入值
    pub fn income_value_at(self, allocs: &IRAllocs, index: usize) -> ValueSSA {
        self.to_inst(&allocs.insts).income_value_at(index)
    }

    /// 为指定前驱基本块设置一个传入值, 要求该基本块已经存在传入值.
    pub fn set_existing_income(
        self,
        allocs: &IRAllocs,
        block: BlockRef,
        value: ValueSSA,
    ) -> PhiRes {
        self.to_inst(&allocs.insts)
            .set_existing_income(allocs, block, value)
    }

    /// 为指定前驱基本块设置一个传入值, 如果该基本块已经存在传入值则覆盖.
    pub fn set_income(self, allocs: &IRAllocs, block: BlockRef, value: ValueSSA) -> PhiRes {
        self.to_inst(&allocs.insts).set_income(allocs, block, value)
    }

    /// 移除指定前驱基本块的传入值, 要求该基本块已经存在传入值.
    pub fn remove_income(self, allocs: &IRAllocs, block: BlockRef) -> PhiRes {
        self.to_inst(&allocs.insts).remove_income(block)
    }

    /// 移除指定组索引下的基本块传入值.
    /// 移除时会保持操作数列表的紧凑性, 通过与末尾元素交换并弹出末尾元素来实现.
    /// 这种方式会改变被交换元素的索引, 因此需要更新它们的 UseKind 以反映新的索引.
    pub fn remove_income_index(self, allocs: &IRAllocs, group_index: usize) {
        self.to_inst(&allocs.insts).remove_income_index(group_index);
    }

    /// 移除所有引用了无效基本块的传入值对
    pub fn retain_valid_income(self, allocs: &IRAllocs) {
        self.to_inst(&allocs.insts).retain_valid_income();
    }

    /// 重定向 Phi 节点中某个前驱基本块的引用到新的基本块
    pub fn redirect_income(
        self,
        allocs: &IRAllocs,
        income_bb_use: &Rc<Use>,
        new_block: BlockRef,
    ) -> PhiRes {
        self.to_inst(&allocs.insts)
            .redirect_income(allocs, income_bb_use, new_block)
    }
}

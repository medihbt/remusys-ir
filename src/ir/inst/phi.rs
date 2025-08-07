use crate::{
    ir::{
        BlockRef, IRAllocs, IRWriter, ISubInst, ISubValueSSA, InstCommon, InstData, InstRef,
        Opcode, Use, UseKind, ValueSSA,
        inst::{ISubInstRef, InstOperands},
    },
    typing::id::ValTypeID,
};
use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

#[derive(Debug)]
pub enum PhiError {
    IncomeBBNotFound(BlockRef),
}

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
    operands: RefCell<Vec<Rc<Use>>>,
    /// 前驱基本块到操作数索引的映射：BlockRef -> (value_index, block_index)
    incoming_map: RefCell<BTreeMap<BlockRef, (usize, usize)>>,
}

impl ISubInst for PhiNode {
    fn new_empty(opcode: Opcode) -> Self {
        if opcode != Opcode::Phi {
            panic!("Tried to create a PhiNode with non-Phi opcode");
        }
        Self {
            common: InstCommon::new(opcode, ValTypeID::Void),
            operands: RefCell::new(Vec::new()),
            incoming_map: RefCell::new(BTreeMap::new()),
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

    fn get_operands(&self) -> InstOperands {
        InstOperands::InRef(self.operands.borrow())
    }

    fn operands_mut(&mut self) -> &mut [Rc<Use>] {
        self.operands.get_mut().as_mut_slice()
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

        // 如果没有 incoming values，输出有效的空 phi
        let incoming_map = self.incoming_map.borrow();
        if incoming_map.is_empty() {
            return Ok(());
        }

        // 写入所有 incoming values: [ <value>, %<label> ], ...
        let operands = self.operands.borrow();
        for (i, (&block, &(value_idx, _))) in incoming_map.iter().enumerate() {
            if i > 0 {
                writer.write_str(", ")?;
            }
            writer.write_str(" [ ")?;

            // 写入值
            let incoming_val = operands[value_idx].get_operand();
            writer.write_operand(incoming_val)?;

            // 写入来源基本块
            writer.write_str(", ")?;
            writer.write_operand(block)?;

            writer.write_str(" ]")?;
        }

        Ok(())
    }
}

impl PhiNode {
    pub fn new(ret_type: ValTypeID) -> Self {
        Self {
            common: InstCommon::new(Opcode::Phi, ret_type),
            operands: RefCell::new(Vec::new()),
            incoming_map: RefCell::new(BTreeMap::new()),
        }
    }

    pub fn get_income_index(&self, block: BlockRef) -> Option<(usize, usize)> {
        self.incoming_map.borrow().get(&block).cloned()
    }
    pub fn get_income_uses(&self, block: BlockRef) -> Option<(Rc<Use>, Rc<Use>)> {
        self.get_income_index(block)
            .and_then(|(val_idx, block_idx)| {
                let ops = self.operands.borrow();
                Some((ops.get(val_idx)?.clone(), ops.get(block_idx)?.clone()))
            })
    }
    pub fn get_income_block_use(&self, block: BlockRef) -> Option<Rc<Use>> {
        self.get_income_index(block)
            .and_then(|(_, block_idx)| self.operands.borrow().get(block_idx).cloned())
    }
    pub fn get_income_value_use(&self, block: BlockRef) -> Option<Rc<Use>> {
        self.get_income_index(block)
            .and_then(|(val_idx, _)| self.operands.borrow().get(val_idx).cloned())
    }
    pub fn get_income_value(&self, block: BlockRef) -> Option<ValueSSA> {
        self.get_income_value_use(block)
            .map(|use_ref| use_ref.get_operand())
    }

    pub fn set_existing_income(
        &self,
        allocs: &IRAllocs,
        block: BlockRef,
        value: ValueSSA,
    ) -> Result<(), PhiError> {
        let (val_idx, _) = self
            .get_income_index(block)
            .ok_or(PhiError::IncomeBBNotFound(block))?;
        let mut ops = self.operands.borrow_mut();
        if let Some(use_ref) = ops.get_mut(val_idx) {
            use_ref.set_operand(allocs, value);
            Ok(())
        } else {
            Err(PhiError::IncomeBBNotFound(block))
        }
    }

    /// 为指定前驱基本块设置一个传入值, 如果该基本块已经存在传入值则覆盖.
    /// 如果该基本块不存在传入值则新增一对传入值和传入基本块操作数.
    pub fn set_income(
        &self,
        allocs: &IRAllocs,
        block: BlockRef,
        value: ValueSSA,
    ) -> Result<(), PhiError> {
        match self.set_existing_income(allocs, block, value) {
            Ok(()) => return Ok(()),
            Err(PhiError::IncomeBBNotFound(_)) => {}
        };

        let mut operands = self.operands.borrow_mut();
        let income_val_idx = operands.len();
        let income_block_idx = income_val_idx + 1;

        // 构建互引用关系: PhiIncomingValue 里存的是对应的基本块索引, PhiIncomingBlock 里存的是对应的值索引
        // 这样设计是为了方便在删除某个前驱基本块时, 可以通过值索引快速找到对应的值 Use
        // 而不需要遍历所有的 Use 列表。
        let value_use = Use::new(UseKind::PhiIncomingValue(block, income_block_idx as u32));
        let block_use = Use::new(UseKind::PhiIncomingBlock(income_val_idx as u32));

        value_use.set_operand(allocs, value);
        block_use.set_operand(allocs, ValueSSA::Block(block));

        operands.push(value_use);
        operands.push(block_use);

        self.incoming_map
            .borrow_mut()
            .insert(block, (income_val_idx, income_block_idx));

        Ok(())
    }

    /// 移除指定前驱基本块的传入值, 如果该基本块不存在传入值则返回错误.
    /// 移除时会保持操作数列表的紧凑性, 通过与末尾元素交换并弹出末尾元素来实现.
    /// 这种方式会改变被交换元素的索引, 因此需要更新它们的 UseKind 以反映新的索引.
    /// 同时也需要更新 incoming_map 中的索引映射.
    pub fn remove_income(&self, block: BlockRef) -> Result<(), PhiError> {
        let Some((val_idx, block_idx)) = self.get_income_index(block) else {
            return Err(PhiError::IncomeBBNotFound(block));
        };

        let mut ops = self.operands.borrow_mut();
        let len = ops.len();

        if block_idx == len - 1 {
            debug_assert_eq!(
                val_idx,
                len - 2,
                "Incoming value Use should be arranged before block Use"
            );
            ops.pop();
            ops.pop();
            let mut map = self.incoming_map.borrow_mut();
            map.remove(&block);
        } else {
            // Swap with the back and pop

            let back_block_use = ops.pop().unwrap();
            let back_value_use = ops.pop().unwrap();

            // 修复互引用关系: 这两个 Use 会被替换到被删除的 Use 位置, 需要更新它们的索引信息.
            // back_block_use 里存的是对应的值索引, back_value_use 里存的是对应的基本块索引
            let back_block = *BlockRef::from_ir(&back_block_use.get_operand());
            back_block_use
                .kind
                .set(UseKind::PhiIncomingBlock(val_idx as u32));
            back_value_use
                .kind
                .set(UseKind::PhiIncomingValue(back_block, block_idx as u32));

            ops[val_idx] = back_value_use;
            ops[block_idx] = back_block_use;

            drop(ops);

            // Update the map

            let mut map = self.incoming_map.borrow_mut();
            map.remove(&block);
            map.insert(back_block, (val_idx, block_idx));
        }
        Ok(())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct PhiRef(InstRef);

impl ISubInstRef for PhiRef {
    type InstDataT = PhiNode;
    fn from_raw_nocheck(inst_ref: InstRef) -> Self {
        Self(inst_ref)
    }
    fn into_raw(self) -> InstRef {
        self.0
    }
}

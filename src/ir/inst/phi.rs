use crate::{
    ir::{
        BlockRef, IRAllocs, ISubInst, ISubValueSSA, InstCommon, InstData, InstRef, Opcode, Use,
        UseKind, ValueSSA,
        inst::{ISubInstRef, InstOperands},
    },
    typing::id::ValTypeID,
};
use std::{cell::RefCell, collections::BTreeMap, rc::Rc};

#[derive(Debug)]
pub enum PhiError {
    IncomeBBNotFound(BlockRef),
}

#[derive(Debug)]
pub struct PhiNode {
    common: InstCommon,
    operands: RefCell<Vec<Rc<Use>>>,
    /// 把前驱基本块映射到对应的操作数索引.
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

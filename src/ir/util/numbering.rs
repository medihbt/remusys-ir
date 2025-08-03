use std::collections::BTreeMap;

use crate::{
    base::INullableValue,
    ir::{
        ValueSSA,
        block::BlockRef,
        global::{GlobalData, GlobalRef, func::FuncStorage},
        inst::InstRef,
        module::Module,
    },
    typing::id::ValTypeID,
};

pub struct IRValueNumberMap {
    /// The mapping from instruction to its value number.
    /// Sorted by the `InstRef`.
    pub inst_map: Box<[(InstRef, usize)]>,
    /// The mapping from block to its value number.
    /// Sorted by the `BlockRef`.
    pub block_map: Box<[(BlockRef, usize)]>,
    pub func: GlobalRef,
}

impl IRValueNumberMap {
    pub fn new_empty() -> Self {
        IRValueNumberMap {
            inst_map: Box::new([]),
            block_map: Box::new([]),
            func: GlobalRef::new_null(),
        }
    }
    pub fn from_func(module: &Module, func: GlobalRef, option: NumberOption) -> Self {
        let alloc_value = module.borrow_value_alloc();
        let alloc_block = &alloc_value.blocks;
        let alloc_inst = &alloc_value.insts;

        let mut inst_map = BTreeMap::new();
        let mut block_map = BTreeMap::new();

        let func_data = module.get_global(func);
        let func_data = match &*func_data {
            GlobalData::Func(f) => f,
            _ => panic!("Expected a function"),
        };

        let blocks_range = match func_data.get_blocks() {
            Some(b) => b.load_range(),
            None => panic!("Function has no blocks"),
        };
        let mut curr_number = func_data.get_nargs(&module.type_ctx);
        for (block_ref, block) in blocks_range.view(alloc_block) {
            block_map.insert(block_ref, curr_number);
            curr_number += 1;

            for (inst_ref, inst) in block.instructions.view(alloc_inst) {
                if (option.ignore_void && matches!(inst.get_value_type(), ValTypeID::Void))
                    || (option.ignore_terminator && inst.is_terminator())
                    || (option.ignore_guide && inst.is_guide_node())
                {
                    continue;
                }
                inst_map.insert(inst_ref, curr_number);
                curr_number += 1;
            }
        }

        let mut inst_map_vec = Vec::with_capacity(inst_map.len());
        let mut block_map_vec = Vec::with_capacity(block_map.len());

        for (inst_ref, number) in inst_map {
            inst_map_vec.push((inst_ref, number));
        }
        for (block_ref, number) in block_map {
            block_map_vec.push((block_ref, number));
        }

        Self {
            inst_map: inst_map_vec.into_boxed_slice(),
            block_map: block_map_vec.into_boxed_slice(),
            func,
        }
    }
}

impl IRValueNumberMap {
    pub fn inst_get_number(&self, inst: InstRef) -> Option<usize> {
        self.inst_map
            .binary_search_by_key(&inst, |(inst_ref, _)| *inst_ref)
            .ok()
            .map(|index| self.inst_map[index].1)
    }
    pub fn block_get_number(&self, block: BlockRef) -> Option<usize> {
        self.block_map
            .binary_search_by_key(&block, |(block_ref, _)| *block_ref)
            .ok()
            .map(|index| self.block_map[index].1)
    }
    pub fn value_get_number(&self, value: ValueSSA) -> Option<usize> {
        match value {
            ValueSSA::Inst(inst) => self.inst_get_number(inst),
            ValueSSA::Block(block) => self.block_get_number(block),
            ValueSSA::FuncArg(_, idx) => Some(idx as usize),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct NumberOption {
    pub ignore_void: bool,
    pub ignore_terminator: bool,
    pub ignore_guide: bool,
}

impl Default for NumberOption {
    fn default() -> Self {
        NumberOption {
            ignore_void: true,
            ignore_terminator: true,
            ignore_guide: true,
        }
    }
}

impl NumberOption {
    pub fn ignore_all() -> Self {
        NumberOption {
            ignore_void: true,
            ignore_terminator: true,
            ignore_guide: true,
        }
    }
    pub fn ignore_none() -> Self {
        NumberOption {
            ignore_void: false,
            ignore_terminator: false,
            ignore_guide: false,
        }
    }
}

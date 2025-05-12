use core::alloc;
use std::rc::Rc;

use crate::{
    base::{NullableValue, slablist::SlabRefListError, slabref::SlabRef},
    ir::{
        block::{BlockData, BlockRef},
        global::{
            GlobalData, GlobalRef,
            func::{self, FuncData},
        },
        inst::{InstData, InstError, InstRef, terminator::Jump},
        module::Module,
    },
    typing::types::FuncTypeRef,
};

pub struct IRBuilder {
    pub module: Rc<Module>,
    pub focus: IRBuilderFocus,
}

pub struct IRBuilderFocus {
    pub function: GlobalRef,
    pub block: BlockRef,
    pub inst: InstRef,
}

#[derive(Debug, Clone)]
pub enum IRBuilderError {
    GlobalDefExists(String, GlobalRef),
    GlobalDefNotFound(String),

    ListError(SlabRefListError),
    InstError(InstError),
    NullFocus,
    SplitFocusIsPhi(InstRef),
    SplitFocusIsGuideNode(InstRef),

    BlockHasNoTerminator(BlockRef),
}

impl IRBuilder {
    pub fn new(module: Rc<Module>) -> Self {
        Self {
            module,
            focus: IRBuilderFocus {
                function: GlobalRef::new_null(),
                block: BlockRef::new_null(),
                inst: InstRef::new_null(),
            },
        }
    }

    pub fn set_function(&mut self, function: GlobalRef) {
        self.focus.function = function;
    }

    pub fn set_block(&mut self, block: BlockRef) {
        self.focus.block = block;
    }

    pub fn set_inst(&mut self, inst: InstRef) {
        self.focus.inst = inst;
    }

    pub fn set_focus(&mut self, func: GlobalRef, block: BlockRef, inst: InstRef) {
        self.focus.function = func;
        self.focus.block = block;
        self.focus.inst = inst;
    }

    /// Switch the focus to the terminator of the current block.
    /// Returns the previous focus.
    pub fn switch_focus_to_terminator(&mut self) -> Result<InstRef, IRBuilderError> {
        if self.focus.function.is_null() || self.focus.block.is_null() {
            return Err(IRBuilderError::NullFocus);
        }

        let previous_focus = self.focus.inst;
        let alloc_value = self.module.borrow_value_alloc();
        let alloc_block = &alloc_value._alloc_block;

        let block = self.focus.block.to_slabref_unwrap(alloc_block);
        self.focus.inst = block
            .get_termiantor(&self.module)
            .ok_or(IRBuilderError::BlockHasNoTerminator(self.focus.block))?;
        Ok(previous_focus)
    }

    pub fn declare_function(
        &mut self,
        name: &str,
        functype: FuncTypeRef,
    ) -> Result<GlobalRef, IRBuilderError> {
        if let Some(global) = self.module.global_defs.borrow().get(name) {
            return Err(IRBuilderError::GlobalDefExists(name.to_string(), *global));
        }
        let func_data = FuncData::new_extern(functype, name.to_string());
        Ok(self.module.insert_global(GlobalData::Func(func_data)))
    }
    pub fn define_function_with_unreachable(
        &mut self,
        name: &str,
        functype: FuncTypeRef,
    ) -> Result<GlobalRef, IRBuilderError> {
        if let Some(global) = self.module.global_defs.borrow().get(name) {
            return Err(IRBuilderError::GlobalDefExists(name.to_string(), *global));
        }
        let func_data = FuncData::new_with_unreachable(&self.module, functype, name.to_string())
            .map_err(IRBuilderError::ListError)?;

        let (ret, entry, inst) = {
            let alloc_value = self.module.borrow_value_alloc();
            let alloc_block = &alloc_value._alloc_block;
            let entry = func_data
                .get_blocks()
                .unwrap()
                .get_front_ref(alloc_block)
                .unwrap();
            let inst = entry
                .to_slabref_unwrap(alloc_block)
                .get_termiantor(&self.module)
                .unwrap();
            let ret = self.module.insert_global(GlobalData::Func(func_data));
            (ret, entry, inst)
        };

        self.set_focus(ret, entry, inst);
        Ok(ret)
    }

    pub fn split_current_block_from_focus(&mut self) -> Result<BlockRef, IRBuilderError> {
        todo!("Implement split_current_block_from_focus");
    }
    /// Split the current block from the terminator.
    /// This will create a new block and insert a jump to it.
    pub fn split_current_block_from_terminator(&mut self) -> Result<BlockRef, IRBuilderError> {
        let module = self.module.as_ref();
        let curr_bb = self.focus.block;
        if curr_bb.is_null() {
            return Err(IRBuilderError::NullFocus);
        }
        let old_terminator = {
            let curr_bb_data = module.get_block(curr_bb);
            match curr_bb_data.get_termiantor(module) {
                Some(terminator) => terminator,
                None => return Err(IRBuilderError::BlockHasNoTerminator(curr_bb)),
            }
        };

        // Now create a new block. After that, a new jump instruction to this block will be created.
        let new_bb = {
            let block_data = BlockData::new_empty(module);
            module.insert_block(block_data)
        };
        let jump_to_new_bb = {
            let (common, jmp) = Jump::new(module, new_bb);
            module.insert_inst(InstData::Jump(common, jmp))
        };
        // The old terminator will be detached from the current block and inserted into the new block.
        module
            .get_block(curr_bb)
            .set_terminator(module, jump_to_new_bb)
            .map_err(|e| match e {
                InstError::ListError(le) => IRBuilderError::ListError(le),
                _ => Err(e).expect("IR Builder cannot handle these fatal errors. STOP."),
            })?;
        // Now we need to set the old terminator to the new block.
        module
            .get_block(new_bb)
            .set_terminator(module, old_terminator)
            .map_err(|e| match e {
                InstError::ListError(le) => IRBuilderError::ListError(le),
                _ => Err(e).expect("IR Builder cannot handle these fatal errors. STOP."),
            })?;
        Ok(new_bb)
    }
}

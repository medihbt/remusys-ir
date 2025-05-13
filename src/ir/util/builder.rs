use std::rc::Rc;

use crate::{
    base::{NullableValue, slablist::SlabRefListError, slabref::SlabRef},
    ir::{
        ValueSSA,
        block::{BlockData, BlockRef},
        cmp_cond::CmpCond,
        global::{GlobalData, GlobalRef, func::FuncData},
        inst::{
            InstData, InstError, InstRef,
            binop::BinOp,
            callop,
            cast::CastOp,
            cmp::CmpOp,
            gep::IndexPtrOp,
            load_store::{LoadOp, StoreOp},
            phi::PhiOp,
            sundury_inst::{self, SelectOp},
            terminator::Jump,
        },
        module::Module,
        opcode::Opcode,
    },
    typing::{id::ValTypeID, types::FuncTypeRef},
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
    InstIsTerminator(InstRef),
    InstIsGuideNode(InstRef),
    InstIsPhi(InstRef),

    InsertPosIsPhi(InstRef),
    InsertPosIsTerminator(InstRef),
    InsertPosIsGuideNode(InstRef),
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

    pub fn set_focus_func(&mut self, function: GlobalRef) {
        self.focus.function = function;
    }
    pub fn set_focus_block(&mut self, block: BlockRef) {
        self.focus.block = block;
    }
    pub fn set_focus_inst(&mut self, inst: InstRef) {
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

    /// Split the current block from the focus.
    ///
    /// This will split this block from the end and move all instructions from the focus to the new block.
    /// The focus will be set to the new block, while returning the old block.
    pub fn split_current_block_from_focus(&mut self) -> Result<BlockRef, IRBuilderError> {
        if self.focus.block.is_null() {
            return Err(IRBuilderError::NullFocus);
        }

        let new_bb = self.split_current_block_from_terminator()?;

        // Then move all instructions from the focus to the new block.
        todo!("Split the current block from the focus");
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

/// Instruction builder
impl IRBuilder {
    pub fn add_inst(&mut self, inst: InstData, replaces_terminator: bool) -> Result<InstRef, IRBuilderError> {
        todo!("Add instruction to the current block");
    }

    /// 添加 Phi 指令，不是终止子。
    pub fn add_phi_inst(&mut self, ret_type: ValTypeID) -> Result<InstRef, IRBuilderError> {
        let (common, phi_op) = PhiOp::new(ret_type, &self.module);
        self.add_inst(InstData::Phi(common, phi_op), false)
    }

    /// 添加 Store 指令。
    pub fn add_store_inst(
        &mut self,
        target: ValueSSA,
        source: ValueSSA,
        align: usize,
    ) -> Result<InstRef, IRBuilderError> {
        let valty = source.get_value_type(&self.module);
        let (common, store_op) = StoreOp::new(&self.module, valty, align, source, target)
            .map_err(IRBuilderError::InstError)?;
        let inst = InstData::Store(common, store_op);
        self.add_inst(inst)
    }

    /// 添加 Select 指令。
    pub fn add_select_inst(
        &mut self,
        cond: ValueSSA,
        true_val: ValueSSA,
        false_val: ValueSSA,
    ) -> Result<InstRef, IRBuilderError> {
        // 假设 sundury_inst::SelectOp 提供了 new 函数，新函数返回 (InstDataCommon, SelectOp)
        let (common, sel_op) = SelectOp::new(&self.module, cond, true_val, false_val)
            .map_err(IRBuilderError::InstError)?;
        let inst = InstData::Select(common, sel_op);
        self.add_inst(inst)
    }

    /// 添加 Binary Operation 指令。
    pub fn add_binop_inst(
        &mut self,
        opcode: Opcode,
        lhs: ValueSSA,
        rhs: ValueSSA,
    ) -> Result<InstRef, IRBuilderError> {
        let (common, bin_op) = BinOp::new_with_operands(&self.module, opcode, lhs, rhs)
            .map_err(IRBuilderError::InstError)?;
        let inst = InstData::BinOp(common, bin_op);
        self.add_inst(inst)
    }

    /// 添加 Compare 指令。
    pub fn add_cmp_inst(
        &mut self,
        cond: CmpCond,
        lhs: ValueSSA,
        rhs: ValueSSA,
    ) -> Result<InstRef, IRBuilderError> {
        let (common, cmp_op) = CmpOp::new_with_operands(&self.module, cond, lhs, rhs)
            .map_err(IRBuilderError::InstError)?;
        let inst = InstData::Cmp(common, cmp_op);
        self.add_inst(inst)
    }

    /// 添加 Cast 指令。
    pub fn add_cast_inst(
        &mut self,
        opcode: Opcode,
        ret_type: ValTypeID,
        from_value: ValueSSA,
    ) -> Result<InstRef, IRBuilderError> {
        let (common, cast_op) = CastOp::new(&self.module, opcode, ret_type, from_value)
            .map_err(IRBuilderError::InstError)?;
        let inst = InstData::Cast(common, cast_op);
        self.add_inst(inst)
    }

    /// 添加 GetElementPtr 指令。
    pub fn add_indexptr_inst(
        &mut self,
        base_pointee_ty: ValTypeID,
        base_align: usize,
        ret_align: usize,
        base_ptr: ValueSSA,
        indices: impl Iterator<Item = ValueSSA> + Clone,
    ) -> Result<InstRef, IRBuilderError> {
        let (common, gep_op) = IndexPtrOp::new_from_indices(
            &self.module,
            base_pointee_ty,
            base_align,
            ret_align,
            base_ptr,
            indices,
        )
        .map_err(IRBuilderError::InstError)?;
        let inst = InstData::IndexPtr(common, gep_op);
        self.add_inst(inst)
    }

    /// 添加 Call 指令。
    pub fn add_call_inst(
        &mut self,
        callee: GlobalRef,
        args: impl Iterator<Item = ValueSSA>,
    ) -> Result<InstRef, IRBuilderError> {
        let (common, call_op) = callop::CallOp::new_from_func(&self.module, callee, args)
            .map_err(IRBuilderError::InstError)?;
        let inst = InstData::Call(common, call_op);
        self.add_inst(inst)
    }

    pub fn add_load_inst(
        &mut self,
        source_ty: ValTypeID,
        source_align: usize,
        source: ValueSSA,
    ) -> Result<InstRef, IRBuilderError> {
        let (c, l) = LoadOp::new(&self.module, source_ty, source_align, source)
            .map_err(IRBuilderError::InstError)?;
        self.add_inst(InstData::Load(c, l))
    }
}

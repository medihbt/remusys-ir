use std::{cell::Ref, rc::Rc};

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
            sundury_inst::SelectOp,
            terminator::{Br, Jump, Ret, Switch},
        },
        module::Module,
        opcode::Opcode,
    },
    typing::{id::ValTypeID, types::FuncTypeRef},
};

pub struct IRBuilder {
    pub module: Rc<Module>,
    pub focus: IRBuilderExpandedFocus,
    pub focus_check: IRBuilderFocusCheckOption,
}

#[derive(Debug, Clone)]
pub struct IRBuilderExpandedFocus {
    pub function: GlobalRef,
    pub block: BlockRef,
    pub inst: InstRef,
}
#[derive(Debug, Clone)]
pub enum IRBuilderFocus {
    Block(BlockRef),
    Inst(InstRef),
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IRBuilderFocusCheckOption {
    /// When option `.0` is turned on:
    /// - while you add a normal instruction on a terminator focus,
    ///   the insertion will happen on the front of this terminator.
    /// - while you add a terminator instruction on terminator focus,
    ///   the insertion will be turned to replacing this terminator.
    /// - while you add a PHI instruction on terminator focus,
    ///   the insertion will be degraded to an insertion with block focus.
    /// - **Basic block splitting**: Splitting on a terminator focus will
    ///   do exactly the same thing as splitting on a block focus.
    ///
    /// When option `.0` is turned off:
    /// - You cannot add any instruction on a terminator focus.
    /// - **Basic block splitting**: Splitting on a terminator focus will
    ///   return an error.
    ///
    /// When option `.1` is turned on:
    /// - while you add a PHI instruction on a non-PHI instruction focus
    ///   (or vice versa), the insertion will be degraded to an insertion
    ///   with block focus.
    /// - **Basic block splitting**: Splitting on a PHI focus will be degraded
    ///   to splitting on the PHI-end guide node.
    Degrade(bool /* terminator */, bool /* phi */),

    /// Disable checking, ignore all the limits, just treat everything
    /// as a normal instruction.
    ///
    /// This option is unsafe and should be used with caution.
    Ignore,
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
            focus: IRBuilderExpandedFocus {
                function: GlobalRef::new_null(),
                block: BlockRef::new_null(),
                inst: InstRef::new_null(),
            },
            focus_check: IRBuilderFocusCheckOption::Degrade(false, false),
        }
    }

    pub fn get_focus_full(&self) -> IRBuilderExpandedFocus {
        self.focus.clone()
    }
    pub fn set_focus_full(&mut self, func: GlobalRef, block: BlockRef, inst: InstRef) {
        self.focus.function = func;
        self.focus.block = block;
        self.focus.inst = inst;
    }

    pub fn get_focus(&self) -> Option<IRBuilderFocus> {
        let IRBuilderExpandedFocus {
            function,
            block,
            inst,
        } = self.focus.clone();

        if function.is_null() {
            None
        } else if block.is_null() {
            None
        } else if inst.is_null() {
            Some(IRBuilderFocus::Block(block))
        } else {
            Some(IRBuilderFocus::Inst(inst))
        }
    }
    pub fn set_focus(&mut self, focus: IRBuilderFocus) {
        match focus {
            IRBuilderFocus::Block(block) => {
                self.focus.block = block;
                self.focus.inst = InstRef::new_null();
            }
            IRBuilderFocus::Inst(inst) => {
                self.focus.inst = inst;
                self.focus.block = match self.module.get_inst(inst).get_parent_bb() {
                    Some(block) => block,
                    None => panic!("Focus instruction should be attached to a live basic block."),
                };
            }
        }
        let function = self.module.get_block(self.focus.block).get_parent_func();
        if function.is_null() {
            panic!("Focus block should be attached to a live function.");
        }
        self.focus.function = function;
    }
    pub fn borrow_focus_function(&self) -> Ref<FuncData> {
        let focus_func = self.focus.function;
        Ref::map(self.module.get_global(focus_func), |global| match global {
            GlobalData::Func(func_data) => func_data,
            _ => panic!("Focus function should be a function."),
        })
    }

    pub fn set_terminator_degrade_option(&mut self, allow: bool) {
        match self.focus_check {
            IRBuilderFocusCheckOption::Degrade(_, phi) => {
                self.focus_check = IRBuilderFocusCheckOption::Degrade(allow, phi);
            }
            IRBuilderFocusCheckOption::Ignore => {
                self.focus_check = IRBuilderFocusCheckOption::Degrade(allow, false);
            }
        }
    }
    pub fn set_phi_degrade_option(&mut self, allow: bool) {
        match self.focus_check {
            IRBuilderFocusCheckOption::Degrade(terminator, _) => {
                self.focus_check = IRBuilderFocusCheckOption::Degrade(terminator, allow);
            }
            IRBuilderFocusCheckOption::Ignore => {
                self.focus_check = IRBuilderFocusCheckOption::Degrade(false, allow);
            }
        }
    }
    pub fn allows_terminator_degrade(&self) -> bool {
        match self.focus_check {
            IRBuilderFocusCheckOption::Degrade(allow, _) => allow,
            IRBuilderFocusCheckOption::Ignore => true,
        }
    }
    pub fn allows_phi_degrade(&self) -> bool {
        match self.focus_check {
            IRBuilderFocusCheckOption::Degrade(_, allow) => allow,
            IRBuilderFocusCheckOption::Ignore => true,
        }
    }
    pub fn is_full_strict_insert_mode(&self) -> bool {
        match self.focus_check {
            IRBuilderFocusCheckOption::Degrade(t, p) => !t && !p,
            IRBuilderFocusCheckOption::Ignore => true,
        }
    }

    /// Switch the focus to the terminator of the current block.
    /// Returns the previous focus.
    pub fn switch_focus_to_terminator(&mut self) -> Result<InstRef, IRBuilderError> {
        if self.focus.function.is_null() || self.focus.block.is_null() {
            return Err(IRBuilderError::NullFocus);
        }

        let previous_focus = self.focus.inst;
        let alloc_value = self.module.borrow_value_alloc();
        let alloc_block = &alloc_value.alloc_block;

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

        let (entry, inst) = {
            let alloc_value = self.module.borrow_value_alloc();
            let alloc_block = &alloc_value.alloc_block;
            let entry = func_data
                .get_blocks()
                .unwrap()
                .get_front_ref(alloc_block)
                .unwrap();
            let inst = entry
                .to_slabref_unwrap(alloc_block)
                .get_termiantor(&self.module)
                .unwrap();
            (entry, inst)
        };

        let ret = self.module.insert_global(GlobalData::Func(func_data));
        self.set_focus_full(ret, entry, inst);
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

    /// Split the current block from the terminator. New block will be the successor of the
    /// original one with the old terminator. The terminator of the new block will be changed
    /// to a jump instruction to the new block.
    ///
    /// The block focus will not be changed while the new block will be returned.
    /// If the instruction focus is a terminator, it will be set to the new jump instruction.
    ///
    /// There's no need to maintain the RCFG because RCFG connection is based on `Use`-like
    /// object `JumpTarget`, which will remain unchanged during the split. **However, the PHI
    /// nodes in the successors of the original block will be updated to point to the new block**.
    pub fn split_current_block_from_terminator(&mut self) -> Result<BlockRef, IRBuilderError> {
        let curr_bb = self.focus.block;
        if curr_bb.is_null() {
            return Err(IRBuilderError::NullFocus);
        }

        // Now create a new block. After that, a new jump instruction to this block will be created.
        let new_block = {
            let block = BlockData::new_empty(&self.module);
            self._insert_new_block(block)?
        };
        let (old_terminator, jump_to_new_bb) = self.focus_set_jump_to(new_block)?;

        if old_terminator.is_null() {
            return Err(IRBuilderError::BlockHasNoTerminator(curr_bb));
        }

        // Now we need to set the old terminator to the new block.
        self.module
            .get_block(new_block)
            .set_terminator(&self.module, old_terminator)
            .map_err(Self::_map_inst_error)?;

        // Now we need to update the PHI nodes in the successors of the original block.
        // collect the successors of the original block.
        Self::_replace_successor_phis_with_block(&self.module, old_terminator, curr_bb, new_block);

        // If the current focus is a terminator, we need to set the focus back to the
        // new jump instruction of the old block.
        if self.focus.inst == old_terminator {
            self.focus.inst = jump_to_new_bb;
        }
        Ok(new_block)
    }

    fn _map_inst_error(inst_err: InstError) -> IRBuilderError {
        match inst_err {
            InstError::ListError(e) => IRBuilderError::ListError(e),
            _ => Err(inst_err).expect("IR Builder cannot handle these fatal errors. STOP."),
        }
    }

    fn _insert_new_block(&self, block: BlockData) -> Result<BlockRef, IRBuilderError> {
        let block_ref = self.module.insert_block(block);
        self.borrow_focus_function()
            .add_block_ref(&self.module, block_ref)
            .map_err(IRBuilderError::ListError)?;
        Ok(block_ref)
    }

    fn _replace_successor_phis_with_block(
        module: &Module,
        old_terminator: InstRef,
        old_block: BlockRef,
        new_block: BlockRef,
    ) {
        let target_bbs = {
            let terminator_data = module.get_inst(old_terminator);
            let alloc_jt = module.borrow_jt_alloc();

            if let Some((_, t)) = terminator_data.as_terminator() {
                t.collect_jump_blocks(&alloc_jt)
            } else {
                Vec::new()
            }
        };

        // Now we need to update the PHI nodes in the successors of the original block.
        let alloc_value = module.borrow_value_alloc();
        let alloc_block = &alloc_value.alloc_block;
        let alloc_inst = &alloc_value.alloc_inst;
        for block in target_bbs {
            let bb_data = block.to_slabref_unwrap(alloc_block);
            for (_, idata) in bb_data.instructions.view(alloc_inst) {
                let mut phi_ops = match idata {
                    InstData::Phi(_, phi) => phi.get_from_all_mut(),
                    _ => break,
                };
                for (b, _) in phi_ops.iter_mut() {
                    if *b == old_block {
                        *b = new_block
                    }
                }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum InstInsertKind {
    Phi,
    Terminator,
    Normal,
}

/// Instruction builder
impl IRBuilder {
    /// ## Adding Instructions
    ///
    /// Add an instruction data to the `focus` position of a building module.
    ///
    /// ### Rules
    ///
    /// If the focus is a block: Call `BlockData::build_add_inst(inst, mod)` to add the instruction.
    ///
    /// - Normally, an instruction should be added to the front of the block terminator instruction,
    ///   basically the end of the block.
    /// - If the new instruction is a PHI node, it should be added to the end of "Phi Slice" at the
    ///   front of the block, right before the `PhiEnd` helper instruction node.
    /// - If the new instruction is a terminator, the function returns an error.
    /// - For all those cases above, the focus will remain unchanged.
    ///
    /// If the focus is a normal instruction:
    ///
    /// - You can only add a normal instruction to the focus position when none of the focus
    ///   degrade options are set.
    /// - When allowing terminator degrade, the terminator insertion will be degraded to a
    ///   terminator replacement.
    /// - When allowing phi degrade, the phi insertion will be degraded to a normal PHI
    ///   appending to the "Phi Slice" area.
    ///
    /// If the focus is a terminator instruction:
    ///
    /// - You cannot add any instruction to the focus position by default.
    /// - When allowing terminator degrade, the terminator insertion will be degraded to a
    ///   terminator replacement, while other insertion will be degraded to a block-level
    ///   appending.
    ///
    /// If the focus is a PHI instruction:
    ///
    /// - You can only add PHI instruction to the focus position. When the PHI instruction is
    ///   added, the focus will be switched to the new PHI instruction.
    /// - When allowing PHI-degrade, the normal instruction insertion will be degraded to a
    ///   block-level appending, while the terminator insertion will be degraded to a
    ///   terminator replacement.
    pub fn add_inst(&mut self, inst: InstData) -> Result<InstRef, IRBuilderError> {
        let (focus_func, focus_bb, focus_inst) =
            (self.focus.function, self.focus.block, self.focus.inst);

        if focus_func.is_null() || focus_bb.is_null() {
            return Err(IRBuilderError::NullFocus);
        }
        if focus_inst.is_null() {
            // Focus is a block.
            return self.add_inst_on_block_focus(inst);
        }

        // Focus is an instruction.
        let (degrade_terminator, degrade_phi) = match self.focus_check {
            IRBuilderFocusCheckOption::Degrade(t, p) => (t, p),
            IRBuilderFocusCheckOption::Ignore => {
                return self.add_inst_after_focus_ignore_check(inst);
            }
        };

        // Checking enabled.
        let focus_kind = match &*self.module.get_inst(focus_inst) {
            InstData::Phi(..) => InstInsertKind::Phi,
            x if x.is_terminator() => InstInsertKind::Terminator,
            _ => InstInsertKind::Normal,
        };
        let inst_kind = match &inst {
            InstData::Phi(..) => InstInsertKind::Phi,
            x if x.is_terminator() => InstInsertKind::Terminator,
            _ => InstInsertKind::Normal,
        };

        match (focus_kind, inst_kind) {
            (InstInsertKind::Normal, InstInsertKind::Normal) => {
                self.add_inst_after_focus_ignore_check(inst)
            }
            (InstInsertKind::Normal, inst_kind) => {
                let degrade_cond = match inst_kind {
                    InstInsertKind::Terminator => degrade_terminator,
                    InstInsertKind::Phi => degrade_phi,
                    _ => false,
                };
                if degrade_cond {
                    self.add_inst_on_block_focus(inst)
                } else {
                    Err(if inst_kind == InstInsertKind::Terminator {
                        IRBuilderError::InstIsTerminator(focus_inst)
                    } else {
                        IRBuilderError::InstIsPhi(focus_inst)
                    })
                }
            }

            (InstInsertKind::Phi, InstInsertKind::Phi) => {
                self.add_inst_after_focus_ignore_check(inst)
            }
            (InstInsertKind::Phi, _) => {
                if degrade_phi {
                    self.add_inst_on_block_focus(inst)
                } else {
                    Err(IRBuilderError::InsertPosIsPhi(focus_inst))
                }
            }

            (InstInsertKind::Terminator, InstInsertKind::Terminator) => self
                .focus_replace_terminator_with(inst)
                .map(|(_, new_termi)| new_termi),
            (InstInsertKind::Terminator, _) => {
                if degrade_terminator {
                    self.add_inst_on_block_focus(inst)
                } else {
                    Err(IRBuilderError::InsertPosIsTerminator(focus_inst))
                }
            }
        }
    }

    fn add_inst_after_focus_ignore_check(
        &mut self,
        inst: InstData,
    ) -> Result<InstRef, IRBuilderError> {
        let focus_inst = self.focus.inst;
        let new_ref = self.module.insert_inst(inst);
        focus_inst
            .add_next_inst(&self.module, new_ref)
            .map_err(|e| match e {
                InstError::ListError(le) => IRBuilderError::ListError(le),
                _ => Err(e).expect("IR Builder cannot handle these fatal errors. STOP."),
            })?;
        Ok(new_ref)
    }
    fn add_inst_on_block_focus(&mut self, inst: InstData) -> Result<InstRef, IRBuilderError> {
        let focus_block = self.focus.block;
        if focus_block.is_null() {
            return Err(IRBuilderError::NullFocus);
        }
        let new_ref = self.module.insert_inst(inst);
        let block_data = self.module.get_block(focus_block);
        block_data
            .build_add_inst(new_ref, &self.module)
            .map_err(|e| match e {
                InstError::ListError(le) => IRBuilderError::ListError(le),
                _ => Err(e).expect("IR Builder cannot handle these fatal errors. STOP."),
            })?;
        Ok(new_ref)
    }

    /// Adding PHI-Node. Note that this may be a dangerous operation because nearly all
    /// instruction focuses do not allow PHI-node insertion.
    ///
    /// You can enable PHI-degrade option to degrade the illegal insertion to a block-level
    /// insertion, or just switch the focus to a PHI-node or a block before calling this
    /// function.
    pub fn add_phi_inst(&mut self, ret_type: ValTypeID) -> Result<InstRef, IRBuilderError> {
        let (common, phi_op) = PhiOp::new(ret_type, &self.module);
        self.add_inst(InstData::Phi(common, phi_op))
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

    /// Terminator Replacement Function
    ///
    /// This function replaces the current terminator with a new one. If the old
    /// block has no terminator, the function will insert one.
    /// The original jump relationship will be LOST!
    ///
    /// ### Return
    ///
    /// - **Success branch**: A pair of terminators, `.0` is the old one, `.1` is the new one.
    /// - **Error branch**: An error.
    fn focus_replace_terminator_with(
        &mut self,
        terminator: InstData,
    ) -> Result<(InstRef, InstRef), IRBuilderError> {
        if self.focus.block.is_null() {
            return Err(IRBuilderError::NullFocus);
        }
        let old_terminator = InstRef::from_option(
            self.module
                .get_block(self.focus.block)
                .get_termiantor(&self.module),
        );
        // Replace the current terminator with the new one.
        let new_terminator = self.module.insert_inst(terminator);
        self.module
            .get_block(self.focus.block)
            .set_terminator(&self.module, new_terminator)
            .map_err(Self::_map_inst_error)?;
        Ok((old_terminator, new_terminator))
    }

    /// Terminator Replacement Function
    ///
    /// This function replaces the current terminator with a `Unreachable` instruction.
    /// The original jump relationship will be LOST!
    ///
    /// ### Return
    ///
    /// - **Success branch**: A pair of terminators, `.0` is the old one, `.1` is the new one.
    /// - **Error branch**: An error.
    pub fn focus_set_unreachable(&mut self) -> Result<(InstRef, InstRef), IRBuilderError> {
        let unreachable_i = {
            let mut alloc_use = self.module.borrow_use_alloc_mut();
            InstData::new_unreachable(&mut alloc_use)
        };
        self.focus_replace_terminator_with(unreachable_i)
    }

    /// Terminator Replacement Function
    ///
    /// This function replaces the current terminator with a `Return` instruction.
    /// The original jump relationship will be LOST!
    ///
    /// ### Return
    ///
    /// - **Success branch**: A pair of terminators, `.0` is the old one, `.1` is the new one.
    /// - **Error branch**: An error.
    pub fn focus_set_return(
        &mut self,
        ret_value: ValueSSA,
    ) -> Result<(InstRef, InstRef), IRBuilderError> {
        if self.focus.block.is_null() {
            Err(IRBuilderError::NullFocus)
        } else {
            let (alloc, ret) = Ret::new(&self.module, ret_value);
            self.focus_replace_terminator_with(InstData::Ret(alloc, ret))
        }
    }

    /// Terminator Replacement Function
    ///
    /// This function replaces the current terminator with a `Jump` instruction.
    /// The original jump relationship will be LOST!
    ///
    /// ### Return
    ///
    /// - **Success branch**: A pair of terminators, `.0` is the old one, `.1` is the new one.
    /// - **Error branch**: An error.
    pub fn focus_set_jump_to(
        &mut self,
        jump_to: BlockRef,
    ) -> Result<(InstRef, InstRef), IRBuilderError> {
        if self.focus.block.is_null() {
            Err(IRBuilderError::NullFocus)
        } else {
            let (alloc, jmp) = Jump::new(&self.module, jump_to);
            self.focus_replace_terminator_with(InstData::Jump(alloc, jmp))
        }
    }

    /// Terminator Replacement Function
    ///
    /// This function replaces the current terminator with a `Br` instruction.
    /// The original jump relationship will be LOST!
    ///
    /// ### Return
    ///
    /// - **Success branch**: A pair of terminators, `.0` is the old one, `.1` is the new one.
    /// - **Error branch**: An error.
    pub fn focus_set_branch_to(
        &mut self,
        cond: ValueSSA,
        if_true: BlockRef,
        if_false: BlockRef,
    ) -> Result<(InstRef, InstRef), IRBuilderError> {
        if self.focus.block.is_null() {
            Err(IRBuilderError::NullFocus)
        } else {
            let (alloc, br) = Br::new(&self.module, cond, if_true, if_false);
            self.focus_replace_terminator_with(InstData::Br(alloc, br))
        }
    }

    /// Terminator Replacement Function
    ///
    /// This function replaces the current terminator with a `Switch` instruction.
    /// The original jump relationship will be LOST!
    ///
    /// ### Return
    ///
    /// - **Success branch**: A pair of terminators, `.0` is the old one, `.1` is the new one.
    /// - **Error branch**: An error.
    pub fn focus_set_empty_switch(
        &mut self,
        cond: ValueSSA,
        default_block: BlockRef,
    ) -> Result<(InstRef, InstRef), IRBuilderError> {
        if self.focus.block.is_null() {
            Err(IRBuilderError::NullFocus)
        } else {
            let (alloc, switch) = Switch::new(&self.module, cond, default_block);
            self.focus_replace_terminator_with(InstData::Switch(alloc, switch))
        }
    }
    pub fn focus_set_switch_with_cases(
        &mut self,
        cond: ValueSSA,
        default_block: BlockRef,
        cases: impl Iterator<Item = (i128, BlockRef)>,
    ) -> Result<(InstRef, InstRef), IRBuilderError> {
        let (old_termi, switch_inst) = self.focus_set_empty_switch(cond, default_block)?;

        let value_alloc = self.module.borrow_value_alloc();
        match switch_inst.to_slabref_unwrap(&value_alloc.alloc_inst) {
            InstData::Switch(_, s) => {
                for (case, block) in cases {
                    s.set_case(&self.module, case, block);
                }
            }
            _ => unreachable!(),
        }

        Ok((old_termi, switch_inst))
    }
}

#[cfg(test)]
mod testing {
    use crate::ir::constant::data::ConstData;
    use crate::ir::util::writer::write_ir_module;
    use crate::typing::context::{PlatformPolicy, TypeContext};

    use super::IRBuilder;
    use super::*;

    #[test]
    fn test_ir_builder() {
        let platform_riscv32 = PlatformPolicy {
            ptr_nbits: 32,
            reg_nbits: 32,
        };
        let type_ctx = TypeContext::new_rc(platform_riscv32);
        let module = Rc::new(Module::new(
            "io.medihbt.RemusysIRTesting.test_ir_builder".into(),
            type_ctx.clone(),
        ));
        let mut builder = IRBuilder::new(module.clone());
        module.enable_rdfg().unwrap();
        module.enable_rcfg().unwrap();

        // Add function "main" to the module.
        // SysY source code:
        // ```SysY
        // int main(int argc, byte** argv) {
        //     return 0;
        // }
        // ```
        //
        // Remusys-IR code (Remusys-IR does not support value naming and named pointer):
        // ```Remusys-IR
        // define dso_local i32 @main(i32 %0, ptr %1) {
        // %2:
        //     ret i32 0
        // }
        // ```
        let main_func_ty =
            type_ctx.make_func_type(&[ValTypeID::Int(32), ValTypeID::Ptr], ValTypeID::Int(32));
        builder
            .define_function_with_unreachable("main", main_func_ty)
            .unwrap();

        builder
            .focus_set_return(ConstData::make_int_valssa(32, 0))
            .unwrap();

        // write to file `test_ir_builder.ll`
        let mut writer = std::fs::File::create("target/test_ir_builder.ll").unwrap();
        write_ir_module(module.as_ref(), &mut writer, true, true);

        // Now set the focus to the entry block.
        builder.focus.inst = InstRef::new_null();

        /*
           SysY source code:

           ```SysY
           int main(int argc, byte[][] argv) {
        -      return 0;
        +      return argc + argv[0][1];
           }
           ```

           Remusys-IR code (Remusys-IR does not support value naming):

           ```Remusys-IR
           define dso_local i32 @main(i32 %0, ptr %1) {
           %2:
               %3 = load ptr, ptr %1, align 4
               %4 = getelementptr i8, ptr %3, i32 1
               %5 = load i8, ptr %4, align 1
               %6 = zext i8 %5 to i32
               %7 = add i32 %0, %6
               ret i32 %7
           }
           ```
        */
        let main_func_ref = builder.focus.function;
        let load_3 = builder
            .add_load_inst(ValTypeID::Ptr, 4, ValueSSA::FuncArg(main_func_ref, 1))
            .unwrap();
        let gep_4 = builder
            .add_indexptr_inst(
                ValTypeID::Ptr,
                4,
                1,
                ValueSSA::Inst(load_3),
                vec![ConstData::make_int_valssa(32, 1)].into_iter(),
            )
            .unwrap();
        let load_5 = builder
            .add_load_inst(ValTypeID::Int(8), 1, ValueSSA::Inst(gep_4))
            .unwrap();
        let zext_6 = builder
            .add_cast_inst(Opcode::Zext, ValTypeID::Int(32), ValueSSA::Inst(load_5))
            .unwrap();
        let add_7 = builder
            .add_binop_inst(
                Opcode::Add,
                ValueSSA::FuncArg(main_func_ref, 0),
                ValueSSA::Inst(zext_6),
            )
            .unwrap();

        // Try to split the current block from the terminator.
        let old_focus = builder.focus.block;
        let new_focus = builder.split_current_block_from_terminator().unwrap();
        builder.set_focus(IRBuilderFocus::Block(new_focus));
        let phi_9 = builder.add_phi_inst(ValTypeID::Int(32)).unwrap();
        PhiOp::insert_from_value(phi_9, &module, old_focus, ValueSSA::Inst(add_7)).unwrap();

        builder.focus_set_return(ValueSSA::Inst(phi_9)).unwrap();

        // Try to split again.
        builder.set_focus(IRBuilderFocus::Block(old_focus));
        let _new2_focus = builder.split_current_block_from_terminator().unwrap();

        // write to file `test_ir_builder_chain_inst.ll`
        let mut writer = std::fs::File::create("target/test_ir_builder_chain_inst.ll").unwrap();
        write_ir_module(module.as_ref(), &mut writer, true, true);
    }
}

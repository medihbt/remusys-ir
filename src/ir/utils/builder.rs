use crate::{
    base::{INullableValue, SlabListError, SlabListRange, SlabRef},
    ir::{
        BlockData, BlockRef, CmpCond, Func, FuncRef, GlobalRef, IRAllocs, ISubGlobal, ISubInst,
        ISubValueSSA, ITraceableValue, IUser, InstData, InstRef, ManagedInst, Module, Opcode,
        UseKind, UserID, ValueSSA, Var,
        inst::{
            Alloca, BinOp, Br, CallOp, CastOp, CmpOp, ISubInstRef, IndexPtr, InstError, Jump,
            LoadOp, PhiNode, PhiRef, Ret, SelectOp, StoreOp, Switch,
        },
    },
    typing::{FuncTypeRef, IValType, TypeContext, ValTypeID},
};

pub struct IRBuilder {
    pub module: Module,
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

    ListError(SlabListError),
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

type InstBuildRes = Result<InstRef, IRBuilderError>;
type TermiBuildRes<'a> = Result<(ManagedInst<'a>, InstRef), IRBuilderError>;

impl IRBuilder {
    pub fn new(module: Module) -> Self {
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

    pub fn get_type_ctx(&self) -> &TypeContext {
        &self.module.type_ctx
    }

    pub fn get_focus_full(&self) -> IRBuilderExpandedFocus {
        self.focus.clone()
    }
    pub fn set_focus_full(&mut self, func: GlobalRef, block: BlockRef, inst: InstRef) {
        self.focus.function = func;
        self.focus.block = block;
        self.focus.inst = inst;
    }

    pub fn allocs_mut(&mut self) -> &mut IRAllocs {
        &mut self.module.allocs
    }

    pub fn get_focus(&self) -> Option<IRBuilderFocus> {
        let IRBuilderExpandedFocus { function, block, inst } = self.focus.clone();

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
                self.focus.block = inst.to_inst(&self.allocs_mut().insts).get_parent_bb();
            }
        }
        let function = {
            let block = self.focus.block;
            block.to_data(&self.allocs_mut().blocks).get_parent_func()
        };
        if function.is_null() {
            panic!("Focus block should be attached to a live function.");
        }
        self.focus.function = function;
    }
    pub fn focus_func_mut(&mut self) -> &mut Func {
        let focus_func = self.focus.function;
        let focus_func = FuncRef::from_real(focus_func, &self.allocs_mut().globals);
        focus_func.to_data_mut(&mut self.allocs_mut().globals)
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
        let prev_focus = self.focus.inst;
        let focus_block = self.focus.block;
        let allocs = self.allocs_mut();
        let terminator = focus_block
            .to_data(&allocs.blocks)
            .get_terminator_from_alloc(&allocs.insts);
        self.focus.inst = terminator.get_inst();
        Ok(prev_focus)
    }

    pub fn declare_function(
        &mut self,
        name: &str,
        functy: FuncTypeRef,
    ) -> Result<GlobalRef, IRBuilderError> {
        if let Some(global) = self.module.globals.borrow().get(name) {
            return Err(IRBuilderError::GlobalDefExists(name.to_string(), *global));
        }
        let func = Func::new_extern(functy, name, self.get_type_ctx());
        let funcref = GlobalRef::from_allocs(self.allocs_mut(), func.into_ir());
        funcref.register_to_symtab(&mut self.module);
        Ok(funcref)
    }
    pub fn define_function_with_unreachable(
        &mut self,
        name: &str,
        functy: FuncTypeRef,
    ) -> Result<GlobalRef, IRBuilderError> {
        if let Some(global) = self.module.globals.borrow().get(name) {
            return Err(IRBuilderError::GlobalDefExists(name.to_string(), *global));
        }
        let func = Func::new_with_unreachable(&mut self.module, functy, name);
        let (entry, inst) = {
            let entry = func.get_entry();
            let allocs = self.allocs_mut();
            let alloc_block = &allocs.blocks;
            let alloc_inst = &allocs.insts;
            let inst = entry
                .to_data(alloc_block)
                .get_terminator_from_alloc(alloc_inst);
            (entry, inst.get_inst())
        };
        let func = GlobalRef::from_allocs(self.allocs_mut(), func.into_ir());
        func.register_to_symtab(&mut self.module);
        self.set_focus_full(func, entry, inst);
        Ok(func)
    }

    pub fn declare_var(&mut self, name: &str, is_const: bool, content_ty: ValTypeID) -> GlobalRef {
        if let Some(_) = self.module.globals.borrow().get(name) {
            panic!("Global def `{name}` already exists");
        }
        let var = Var::new_extern(
            name.into(),
            content_ty,
            content_ty.get_align(self.get_type_ctx()).max(8),
        );
        var.set_readonly(is_const);
        let var = GlobalRef::from_allocs(self.allocs_mut(), var.into_ir());
        var.register_to_symtab(&mut self.module);
        var
    }
    pub fn define_var(
        &mut self,
        name: &str,
        is_const: bool,
        content_ty: ValTypeID,
        init: ValueSSA,
    ) -> Result<GlobalRef, IRBuilderError> {
        if let Some(&global) = self.module.globals.borrow().get(name) {
            return Err(IRBuilderError::GlobalDefExists(name.to_string(), global));
        }
        let var = Var::new_extern(
            name.into(),
            content_ty,
            content_ty.get_align(self.get_type_ctx()).max(8),
        );
        var.set_readonly(is_const);
        var.set_init(self.allocs_mut(), init);

        let var = GlobalRef::from_allocs(self.allocs_mut(), var.into_ir());
        var.register_to_symtab(&mut self.module);
        Ok(var)
    }

    /// Split the current block from the focus.
    ///
    /// This will split this block from the end and move all instructions from the focus to the new block.
    /// The focus will be set to the new block, while returning the old block.
    ///
    /// ### Instruction adjustment Rules
    ///
    /// If current focus is a block: Only the terminator will be moved to the new block.
    ///
    /// If checking is diabled while the focus is any instruction:
    ///
    /// - Instructions after the focus will be moved to the new block. The PHI nodes will be
    ///   added to the PHI-area of the new block, normal instructions will be added to the
    ///   end of the new block, while PHI-end guide node will remain unchanged.
    ///
    /// If current focus is a terminator:
    ///
    /// - When terminator degrade is enabled, the function works like a block focus.
    /// - When terminator degrade is disabled, the function will return an error.
    ///
    /// If current focus is a PHI node:
    ///
    /// - When PHI degrade is enabled, the function will move all instructions after the
    ///   PHI-end guide node (aka. non-PHI area) to the new block.
    /// - When PHI degrade is disabled, the function will return an error.
    ///
    /// If current focus is a PHI-end guide node or a normal instruction:
    ///
    /// - The function will move all instructions after the focus to the new block. You can
    ///   infer that the focus will remain unchanged.
    pub fn split_current_block_from_focus(&mut self) -> Result<BlockRef, IRBuilderError> {
        if self.focus.block.is_null() {
            return Err(IRBuilderError::NullFocus);
        }

        let inst_split_pos = if self.focus.inst.is_null() {
            // Focus is a block.
            InstRef::new_null()
        } else if self.focus_check == IRBuilderFocusCheckOption::Ignore {
            self.focus.inst
        } else {
            // Focus is an instruction.
            let focus_inst = self.focus.inst;
            let focus_kind = match focus_inst.to_inst(&self.allocs_mut().insts) {
                InstData::Phi(..) => FocusInstKind::Phi,
                x if x.is_terminator() => FocusInstKind::Terminator,
                _ => FocusInstKind::Normal,
            };

            // Case 1: Focus is a terminator.
            //
            // - When terminator degrade is disabled, the function will return an error.
            // - Otherwise, the function will degrade to a block-based split.
            if focus_kind == FocusInstKind::Terminator {
                if self.allows_terminator_degrade() {
                    InstRef::new_null()
                } else {
                    return Err(IRBuilderError::InsertPosIsTerminator(focus_inst));
                }
            }
            // Case 2: Focus is a PHI node.
            //
            // - When PHI degrade is disabled, the function will return an error.
            // - Otherwise, see the rules above: All instructions after the
            //   PHI-end guide node will be moved to the new block.
            else if focus_kind == FocusInstKind::Phi {
                if self.allows_phi_degrade() {
                    // Focus is a PHI node, move all instructions after the PHI-end guide node.
                    let block = self.focus.block;
                    block.to_data(&self.allocs_mut().blocks).phi_end
                } else {
                    return Err(IRBuilderError::InsertPosIsPhi(focus_inst));
                }
            }
            // Case 3: Focus is a PHI-end guide node or a normal instruction.
            //
            // - The function will move all instructions after the focus to the new block.
            // - The focus will be adjusted to the new block.
            else {
                focus_inst
            }
        };

        let new_bb = self.split_current_block_from_terminator()?;

        if inst_split_pos.is_null() {
            // Focus is a block, degrade to a terminator-based split.
            let old_focus = self.focus.block;
            self.set_focus(IRBuilderFocus::Block(new_bb));
            return Ok(old_focus);
        }

        // Now move all instructions after the `inst_split_pos` to the new block.
        // Step 1: Unplug all instructions after the `inst_split_pos` from the current block.
        let to_insert: Vec<InstRef> = {
            // let alloc_inst = &self.module.borrow_value_alloc().insts;
            let mut to_insert = Vec::new();

            let node_range =
                SlabListRange { node_head: inst_split_pos, node_tail: InstRef::new_null() };
            for (iref, inst) in node_range.view(&self.allocs_mut().insts) {
                match inst {
                    InstData::PhiInstEnd(..) => {}
                    // If the current instruction is a terminator, we need to stop.
                    x if x.is_terminator() => break,
                    _ => to_insert.push(iref),
                }
            }

            // Now unplug all instructions after the `inst_split_pos` from the current block.
            for iref in to_insert.iter() {
                iref.detach_self(self.allocs_mut())
                    .expect("Failed to unplug inst from `to_insert`");
            }

            to_insert
        };

        // Step 2: Insert all instructions to the new block.
        let allocs = self.allocs_mut();
        let new_block = new_bb.to_data(&allocs.blocks);
        for iref in to_insert {
            new_block.build_add_inst(&allocs.insts, iref);
        }

        // Step 3: Set the focus to the new block.
        let old_focus = self.focus.block;
        self.set_focus(IRBuilderFocus::Block(new_bb));
        return Ok(old_focus);
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
            let block = BlockData::new_empty(&mut self.module);
            self.insert_new_block(block)?
        };
        let (old_terminator, jump_to_new_bb) = self.focus_set_jump_to(new_block)?;

        if old_terminator.is_null() {
            return Err(IRBuilderError::BlockHasNoTerminator(curr_bb));
        }

        let old_terminator = old_terminator.release();
        let allocs = self.allocs_mut();
        let alloc_block = &allocs.blocks;

        new_block
            .to_data(alloc_block)
            .set_terminator(allocs, old_terminator)
            .map_err(Self::map_inst_error)?;

        // Now we need to update the PHI nodes in the successors of the original block.
        // collect the successors of the original block.
        Self::replace_successor_phis_with_block(&self.allocs_mut(), curr_bb, new_block);

        // If the current focus is a terminator, we need to set the focus back to the
        // new jump instruction of the old block.
        if self.focus.inst == old_terminator {
            self.focus.inst = jump_to_new_bb;
        }
        Ok(new_block)
    }

    fn map_inst_error(inst_err: InstError) -> IRBuilderError {
        match inst_err {
            InstError::ListError(e) => IRBuilderError::ListError(e),
            _ => Err(inst_err).expect("IR Builder cannot handle these fatal errors. STOP."),
        }
    }

    fn insert_new_block(&mut self, block: BlockData) -> Result<BlockRef, IRBuilderError> {
        let next_block = BlockRef::new(self.allocs_mut(), block);
        let focus_func = FuncRef(self.focus.function);
        let focus_block = self.focus.block;
        if focus_block.is_null() {
            let allocs = self.allocs_mut();
            focus_func
                .to_data(&allocs.globals)
                .add_block_ref(allocs, next_block)
                .map_err(IRBuilderError::ListError)?;
        } else {
            let allocs = self.allocs_mut();
            let body = focus_func.to_data(&allocs.globals).get_body().unwrap();
            body.node_add_next(&allocs.blocks, focus_block, next_block)
                .map_err(IRBuilderError::ListError)?;
        }

        Ok(next_block)
    }

    fn replace_successor_phis_with_block(
        allocs: &IRAllocs,
        old_block: BlockRef,
        new_block: BlockRef,
    ) {
        if old_block == new_block {
            return;
        }
        let old_users = old_block.to_data(&allocs.blocks).users();
        if old_users.is_empty() {
            return;
        }

        let new_users = new_block.to_data(&allocs.blocks).users();
        old_users.move_to_if(
            new_users,
            |u| matches!(u.kind.get(), UseKind::PhiIncomingBlock(_)),
            |u| {
                let UseKind::PhiIncomingBlock(value_use_index) = u.kind.get() else {
                    unreachable!();
                };
                // 这里不使用 set_operand(), 因为链表已经发生移动了.
                u.operand.set(new_block.into_ir());
                let phi_inst = {
                    let UserID::Inst(inst) = u.user.get() else { unreachable!() };
                    PhiRef::from_inst(inst, &allocs.insts)
                };
                let phi_operands = phi_inst.to_inst(&allocs.insts).get_operands();
                let phi_value_use = &phi_operands[value_use_index as usize];
                let UseKind::PhiIncomingValue(block_idx) = phi_value_use.kind.get() else {
                    panic!("PHI inst structure broken");
                };
                // 维护互引用关系: Value 边应当持有对 Block 的引用 -- 包括所指代的 block reference 和
                // PHI 指令中的索引.
                phi_value_use.kind.set(UseKind::PhiIncomingValue(block_idx));
            },
        )
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
enum FocusInstKind {
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
    pub fn add_inst(&mut self, inst: InstData) -> InstBuildRes {
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
        let focus_kind = match focus_inst.to_inst(&self.allocs_mut().insts) {
            InstData::Phi(..) => FocusInstKind::Phi,
            x if x.is_terminator() => FocusInstKind::Terminator,
            _ => FocusInstKind::Normal,
        };
        let inst_kind = match &inst {
            InstData::Phi(..) => FocusInstKind::Phi,
            x if x.is_terminator() => FocusInstKind::Terminator,
            _ => FocusInstKind::Normal,
        };

        match (focus_kind, inst_kind) {
            (FocusInstKind::Normal, FocusInstKind::Normal) => {
                self.add_inst_after_focus_ignore_check(inst)
            }
            (FocusInstKind::Normal, inst_kind) => {
                let degrade_cond = match inst_kind {
                    FocusInstKind::Terminator => degrade_terminator,
                    FocusInstKind::Phi => degrade_phi,
                    _ => false,
                };
                if degrade_cond {
                    self.add_inst_on_block_focus(inst)
                } else {
                    Err(if inst_kind == FocusInstKind::Terminator {
                        IRBuilderError::InstIsTerminator(focus_inst)
                    } else {
                        IRBuilderError::InstIsPhi(focus_inst)
                    })
                }
            }

            (FocusInstKind::Phi, FocusInstKind::Phi) => {
                self.add_inst_after_focus_ignore_check(inst)
            }
            (FocusInstKind::Phi, _) => {
                if degrade_phi {
                    self.add_inst_on_block_focus(inst)
                } else {
                    Err(IRBuilderError::InsertPosIsPhi(focus_inst))
                }
            }

            (FocusInstKind::Terminator, FocusInstKind::Terminator) => self
                .focus_replace_terminator_with(inst)
                .map(|(_, new_termi)| new_termi),
            (FocusInstKind::Terminator, _) => {
                if degrade_terminator {
                    self.add_inst_on_block_focus(inst)
                } else {
                    Err(IRBuilderError::InsertPosIsTerminator(focus_inst))
                }
            }
        }
    }

    fn add_inst_after_focus_ignore_check(&mut self, inst: InstData) -> InstBuildRes {
        let focus_inst = self.focus.inst;
        let focus_block = self.focus.block;
        let new_ref = InstRef::from_alloc(&mut self.allocs_mut().insts, inst);

        let allocs = self.allocs_mut();
        focus_block
            .insts_from_alloc(&allocs.blocks)
            .node_add_next(&allocs.insts, focus_inst, new_ref)
            .map_err(IRBuilderError::ListError)?;
        Ok(new_ref)
    }
    fn add_inst_on_block_focus(&mut self, inst: InstData) -> InstBuildRes {
        let focus_block = self.focus.block;
        if focus_block.is_null() {
            return Err(IRBuilderError::NullFocus);
        }

        let allocs = self.allocs_mut();
        let new_ref = InstRef::from_alloc(&mut allocs.insts, inst);
        focus_block
            .to_data(&allocs.blocks)
            .build_add_inst(&allocs.insts, new_ref);
        Ok(new_ref)
    }

    /// Adding PHI-Node. Note that this may be a dangerous operation because nearly all
    /// instruction focuses do not allow PHI-node insertion.
    ///
    /// You can enable PHI-degrade option to degrade the illegal insertion to a block-level
    /// insertion, or just switch the focus to a PHI-node or a block before calling this
    /// function.
    pub fn add_phi_inst(&mut self, ret_type: ValTypeID) -> InstBuildRes {
        self.add_inst(InstData::Phi(PhiNode::new(ret_type)))
    }

    /// 添加 Store 指令。
    pub fn add_store_inst(
        &mut self,
        target: ValueSSA,
        source: ValueSSA,
        align: usize,
    ) -> InstBuildRes {
        let Module { allocs, type_ctx, .. } = &mut self.module;
        let mut store_op = StoreOp::new(allocs, &type_ctx, source, target);
        store_op.source_align_log2 = align.ilog2() as u8;
        self.add_inst(InstData::Store(store_op))
    }

    /// 添加 Select 指令。
    pub fn add_select_inst(
        &mut self,
        cond: ValueSSA,
        true_val: ValueSSA,
        false_val: ValueSSA,
    ) -> InstBuildRes {
        let select = SelectOp::new(&self.allocs_mut(), cond, true_val, false_val);
        self.add_inst(InstData::Select(select))
    }

    /// 添加 Binary Operation 指令。
    pub fn add_binop_inst(&mut self, opcode: Opcode, lhs: ValueSSA, rhs: ValueSSA) -> InstBuildRes {
        let binop = BinOp::new(&self.allocs_mut(), opcode, lhs, rhs);
        self.add_inst(InstData::BinOp(binop))
    }

    /// 添加 Compare 指令。
    pub fn add_cmp_inst(&mut self, cond: CmpCond, lhs: ValueSSA, rhs: ValueSSA) -> InstBuildRes {
        let cmp = CmpOp::new(&self.allocs_mut(), cond, lhs, rhs);
        self.add_inst(InstData::Cmp(cmp))
    }

    /// 添加 Cast 指令。
    pub fn add_cast_inst(
        &mut self,
        opcode: Opcode,
        to_ty: ValTypeID,
        from: ValueSSA,
    ) -> InstBuildRes {
        let cast = CastOp::new(&self.allocs_mut(), opcode, to_ty, from);
        self.add_inst(InstData::Cast(cast))
    }

    /// 添加 GetElementPtr 指令。
    pub fn add_indexptr_inst<'a, T>(
        &mut self,
        base_pointee_ty: ValTypeID,
        base_align: usize,
        ret_align: usize,
        base_ptr: ValueSSA,
        // indices: impl Iterator<Item = ValueSSA> + Clone,
        indices: T,
    ) -> InstBuildRes
    where
        T: IntoIterator<Item = &'a ValueSSA> + Clone + 'a,
        T::IntoIter: Clone,
    {
        let Module { allocs, type_ctx, .. } = &mut self.module;
        let mut gep = IndexPtr::new(type_ctx, allocs, base_ptr, base_pointee_ty, indices);
        gep.storage_align_log2 = base_align.ilog2() as u8;
        gep.ret_align_log2 = ret_align.ilog2() as u8;
        self.add_inst(InstData::GEP(gep))
    }

    pub fn add_call_inst<'a>(
        &mut self,
        callee: GlobalRef,
        args: impl Iterator<Item = &'a ValueSSA> + Clone + 'a,
    ) -> InstBuildRes {
        let Module { allocs, type_ctx, .. } = &mut self.module;
        let call = CallOp::from_allocs(allocs, type_ctx, callee, args);
        self.add_inst(InstData::Call(call))
    }

    pub fn add_alloca_inst(&mut self, pointee_ty: ValTypeID, align_log2: u8) -> InstBuildRes {
        let alloca = Alloca::new(pointee_ty, align_log2);
        self.add_inst(InstData::Alloca(alloca))
    }

    pub fn add_load_inst(
        &mut self,
        source_ty: ValTypeID,
        source_align: usize,
        source: ValueSSA,
    ) -> InstBuildRes {
        let loadop = LoadOp::new(
            &self.allocs_mut(),
            source_ty,
            source,
            source_align.ilog2() as u8,
        );
        self.add_inst(InstData::Load(loadop))
    }

    /// 添加原子读-修改-写指令构建器。
    pub fn add_amormw_builder<'a>(
        &'a mut self,
        opcode: Opcode,
        valtype: ValTypeID,
    ) -> inst_builders::IRInstBuilderAmoRmw<'a> {
        inst_builders::IRInstBuilderAmoRmw::new(self, opcode, valtype)
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
    fn focus_replace_terminator_with<'a>(&'a mut self, terminator: InstData) -> TermiBuildRes<'a> {
        if self.focus.block.is_null() {
            return Err(IRBuilderError::NullFocus);
        }
        // Replace the current terminator with the new one.
        let new_terminator = InstRef::from_alloc(&mut self.allocs_mut().insts, terminator);

        let focus_block = self.focus.block;
        let allocs = self.allocs_mut();

        let old_terminator = focus_block
            .to_data(&allocs.blocks)
            .set_terminator(allocs, new_terminator)
            .map_err(Self::map_inst_error)?;
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
    pub fn focus_set_unreachable(&mut self) -> TermiBuildRes<'_> {
        if self.focus.block.is_null() {
            Err(IRBuilderError::NullFocus)
        } else {
            let unreachable_i = InstData::new_unreachable();
            self.focus_replace_terminator_with(unreachable_i)
        }
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
    pub fn focus_set_return(&mut self, retval: ValueSSA) -> TermiBuildRes<'_> {
        if self.focus.block.is_null() {
            return Err(IRBuilderError::NullFocus);
        }
        let allocs = self.allocs_mut();
        let ret_ty = retval.get_valtype(allocs);
        let ret = Ret::with_retval(allocs, ret_ty, retval);
        self.focus_replace_terminator_with(InstData::Ret(ret))
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
    pub fn focus_set_jump_to(&mut self, jump_to: BlockRef) -> TermiBuildRes<'_> {
        if self.focus.block.is_null() {
            Err(IRBuilderError::NullFocus)
        } else {
            let allocs = self.allocs_mut();
            let jump = Jump::new(&allocs.blocks, jump_to);
            self.focus_replace_terminator_with(InstData::Jump(jump))
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
    ) -> TermiBuildRes<'_> {
        if self.focus.block.is_null() {
            Err(IRBuilderError::NullFocus)
        } else {
            let allocs = self.allocs_mut();
            let br = Br::new(&allocs, cond, if_true, if_false);
            self.focus_replace_terminator_with(InstData::Br(br))
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
        default: BlockRef,
    ) -> TermiBuildRes<'_> {
        if self.focus.block.is_null() {
            Err(IRBuilderError::NullFocus)
        } else {
            let allocs = self.allocs_mut();
            let switch = Switch::new(allocs, cond, default);
            self.focus_replace_terminator_with(InstData::Switch(switch))
        }
    }

    /// Terminator Replacement Function
    ///
    /// This function replaces the current terminator with a `Switch` instruction with cases.
    /// The original jump relationship will be LOST!
    ///
    /// ### Return
    ///
    /// - **Success branch**: A pair of terminators, `.0` is the old one, `.1` is the new one.
    /// - **Error branch**: An error.
    pub fn focus_set_switch_with_cases(
        &mut self,
        cond: ValueSSA,
        default: BlockRef,
        cases: impl Iterator<Item = (i128, BlockRef)>,
    ) -> TermiBuildRes<'_> {
        if self.focus.block.is_null() {
            return Err(IRBuilderError::NullFocus);
        }
        let allocs = self.allocs_mut();
        let switch = Switch::new(allocs, cond, default);
        for (case, block) in cases {
            switch.set_case(&allocs.blocks, case, block);
        }
        self.focus_replace_terminator_with(InstData::Switch(switch))
    }
}

pub mod inst_builders {
    use crate::{
        ir::{
            IRBuilder, ISubInst, ISubInstRef, Opcode,
            inst::{AmoRmwBuilder, AmoRmwRef},
        },
        typing::ValTypeID,
    };
    use std::ops::{Deref, DerefMut};

    pub struct IRInstBuilderAmoRmw<'a> {
        ir_builder: &'a mut IRBuilder,
        inst_builder: AmoRmwBuilder,
    }

    impl<'a> Deref for IRInstBuilderAmoRmw<'a> {
        type Target = AmoRmwBuilder;
        fn deref(&self) -> &Self::Target {
            &self.inst_builder
        }
    }
    impl<'a> DerefMut for IRInstBuilderAmoRmw<'a> {
        fn deref_mut(&mut self) -> &mut AmoRmwBuilder {
            &mut self.inst_builder
        }
    }

    impl<'a> IRInstBuilderAmoRmw<'a> {
        pub fn new(ir_builder: &'a mut IRBuilder, opcode: Opcode, ty: ValTypeID) -> Self {
            Self { ir_builder, inst_builder: AmoRmwBuilder::new(opcode, ty) }
        }

        pub fn build(self) -> AmoRmwRef {
            let Self { ir_builder, inst_builder } = self;
            let inst = inst_builder.build(ir_builder.allocs_mut());
            let iref = ir_builder
                .add_inst(inst.into_ir())
                .expect("Failed to add AmoRmw instruction");
            AmoRmwRef::from_raw_nocheck(iref)
        }
    }
}

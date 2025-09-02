use crate::{
    base::{INullableValue, SlabListError, SlabListNodeRef, SlabRef},
    ir::{
        BlockData, BlockRef, CmpCond, Func, FuncRef, GlobalRef, IRAllocs, IRAllocsReadable,
        IRManaged, IRModuleCleaner, ISubGlobal, ISubInst, ISubInstRef, ISubValueSSA,
        ITraceableValue, IUser, InstData, InstKind, InstRef, Linkage, ManagedInst, Module, Opcode,
        UseKind, UserID, ValueSSA, Var, inst::*,
    },
    typing::{FuncTypeRef, IValType, TypeContext, ValTypeID},
};
use std::{collections::BTreeMap, rc::Rc};
use thiserror::Error;

#[derive(Debug, Clone)]
pub struct IRFullFocus {
    pub func: GlobalRef,
    pub block: BlockRef,
    pub inst: InstRef,
}

impl IRFullFocus {
    pub fn is_null(&self) -> bool {
        self.func.is_null() || self.block.is_null()
    }

    pub fn is_block_focus(&self) -> bool {
        !self.is_null() && self.inst.is_null()
    }

    pub fn is_inst_focus(&self) -> bool {
        !self.is_null() && !self.inst.is_null()
    }

    pub fn new_empty() -> Self {
        IRFullFocus {
            func: GlobalRef::new_null(),
            block: BlockRef::new_null(),
            inst: InstRef::new_null(),
        }
    }
}

#[derive(Debug, Clone)]
pub enum IRFocus {
    Block(BlockRef),
    Inst(InstRef),
}

impl IRFocus {
    pub fn from_full(full: &IRFullFocus) -> Option<Self> {
        if full.func.is_null() || full.block.is_null() {
            None
        } else if full.inst.is_null() {
            Some(IRFocus::Block(full.block))
        } else {
            Some(IRFocus::Inst(full.inst))
        }
    }

    pub fn to_full(&self, allocs: &impl IRAllocsReadable) -> IRFullFocus {
        match self {
            IRFocus::Block(block) => {
                let func = block.get_parent(allocs);
                IRFullFocus { func, block: *block, inst: InstRef::new_null() }
            }
            IRFocus::Inst(inst) => {
                let block = inst.get_parent(allocs);
                let func = block.get_parent(allocs);
                IRFullFocus { func, block, inst: *inst }
            }
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum IRFocusCheckOption {
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

#[derive(Debug, Clone, Error)]
pub enum IRBuildError {
    #[error("Global definition already exists: {0}")]
    GlobalDefExists(String, GlobalRef),
    #[error("Global definition not found: {0}")]
    GlobalDefNotFound(String),

    #[error("List error: {0:?}")]
    ListError(SlabListError),
    #[error("Null focus")]
    NullFocus,
    #[error("Split focus is PHI: {0:?}")]
    SplitFocusIsPhi(InstRef),
    #[error("Split focus is guide node: {0:?}")]
    SplitFocusIsGuideNode(InstRef),

    #[error("Block has no terminator: {0:?}")]
    BlockHasNoTerminator(BlockRef),
    #[error("Instruction is terminator: {0:?}")]
    InstIsTerminator(InstRef),
    #[error("Instruction is guide node: {0:?}")]
    InstIsGuideNode(InstRef),
    #[error("Instruction is PHI: {0:?}")]
    InstIsPhi(InstRef),

    #[error("Insert position is PHI: {0:?}")]
    InsertPosIsPhi(InstRef),
    #[error("Insert position is terminator: {0:?}")]
    InsertPosIsTerminator(InstRef),
    #[error("Insert position is guide node: {0:?}")]
    InsertPosIsGuideNode(InstRef),
    #[error("Invalid insert position: {0:?}")]
    InvalidInsertPos(InstRef),

    #[error("PHI Node error: {0:?}")]
    PhiNodeError(PhiError),
}

impl From<InstError> for IRBuildError {
    fn from(inst_err: InstError) -> Self {
        match inst_err {
            InstError::ListError(e) => IRBuildError::ListError(e),
            _ => Err(inst_err).expect("IR Builder cannot handle these fatal errors. STOP."),
        }
    }
}

type IRBuildRes<T = ()> = Result<T, IRBuildError>;
type InstBuildRes = Result<InstRef, IRBuildError>;
type TermiBuildRes<'a> = Result<(ManagedInst<'a>, InstRef), IRBuildError>;

#[derive(Debug, Clone, Copy, PartialEq)]
enum FocusInstKind {
    Phi,
    Terminator,
    Normal,
}

pub struct IRBuilder<ModuleT = Module> {
    pub module: ModuleT,
    pub full_focus: IRFullFocus,
    pub focus_check: IRFocusCheckOption,
}

impl IRBuilder<Module> {
    pub fn take(self) -> Module {
        self.module
    }

    pub fn new_host(name: impl ToString) -> Self {
        Self {
            module: Module::new_host_arch(name.to_string()),
            full_focus: IRFullFocus::new_empty(),
            focus_check: IRFocusCheckOption::Degrade(true, true),
        }
    }
}

impl<ModuleT> IRBuilder<ModuleT>
where
    ModuleT: AsRef<Module> + AsMut<Module>,
{
    pub fn new(module: ModuleT) -> Self {
        IRBuilder {
            module,
            full_focus: IRFullFocus::new_empty(),
            focus_check: IRFocusCheckOption::Degrade(true, true),
        }
    }

    pub fn get_allocs(&self) -> &IRAllocs {
        &self.module.as_ref().allocs
    }
    pub fn allocs_mut(&mut self) -> &mut IRAllocs {
        &mut self.module.as_mut().allocs
    }
    pub fn gc_cleaner(&mut self) -> IRModuleCleaner<'_> {
        self.module.as_mut().gc_cleaner()
    }
    pub fn type_ctx(&self) -> &Rc<TypeContext> {
        &self.module.as_ref().type_ctx
    }

    pub fn try_get_focus(&self) -> Option<IRFocus> {
        IRFocus::from_full(&self.full_focus)
    }
    pub fn get_focus(&self) -> IRFocus {
        self.try_get_focus().expect("Current focus is null.")
    }
    pub fn set_focus(&mut self, focus: IRFocus) -> &mut Self {
        self.full_focus = focus.to_full(self.module.as_ref());
        self
    }

    /// Switch the focus to the terminator of the current block.
    /// Returns the previous focus.
    pub fn switch_focus_to_terminator(&mut self) -> Result<InstRef, IRBuildError> {
        let mut focus = self.full_focus.clone();
        if focus.func.is_null() || focus.block.is_null() {
            return Err(IRBuildError::NullFocus);
        }
        let prev_focus = focus.inst;
        let focus_block = focus.block;
        let terminator = focus_block.get_terminator(self.get_allocs());
        focus.inst = terminator.get_inst();
        self.full_focus = focus;
        Ok(prev_focus)
    }
}

pub struct IRVarBuilder<'a> {
    pub module: &'a mut Module,
    pub name: String,
    pub content_ty: ValTypeID,
    pub is_const: bool,
    pub linkage: Linkage,
}

impl<'a> IRVarBuilder<'a> {
    pub fn new(module: &'a mut Module, name: impl ToString, content_ty: ValTypeID) -> Self {
        IRVarBuilder {
            module,
            name: name.to_string(),
            content_ty,
            is_const: false,
            linkage: Linkage::Extern,
        }
    }

    pub fn content_ty(&mut self, content_ty: ValTypeID) -> &mut Self {
        self.content_ty = content_ty;
        self
    }
    pub fn set_const(&mut self, is_const: bool) -> &mut Self {
        self.is_const = is_const;
        self
    }
    pub fn linkage(&mut self, linkage: Linkage) -> &mut Self {
        self.linkage = linkage;
        self
    }

    pub fn build_extern(self) -> GlobalRef {
        let var = Var::new_extern(
            self.name.into(),
            self.content_ty,
            self.content_ty.get_align(&self.module.type_ctx).max(8),
        );
        var.set_readonly(self.is_const);
        let var = GlobalRef::from_allocs(self.module, var.into_ir());
        var.register_to_symtab(self.module);
        var
    }
    pub fn build_define(self, initval: ValueSSA) -> IRBuildRes<GlobalRef> {
        if let Some(&global) = self.module.globals.get_mut().get(&self.name) {
            return Err(IRBuildError::GlobalDefExists(self.name, global));
        }
        let var = Var::new_extern(
            self.name.into(),
            self.content_ty,
            self.content_ty.get_align(&self.module.type_ctx).max(8),
        );
        var.set_readonly(self.is_const);
        var.set_init(self.module, initval);
        let var = GlobalRef::from_allocs(self.module, var.into_ir());
        var.register_to_symtab(self.module);
        Ok(var)
    }
}

impl<ModuleT> IRBuilder<ModuleT>
where
    ModuleT: AsRef<Module> + AsMut<Module>,
{
    pub fn declare_function(&mut self, name: &str, functy: FuncTypeRef) -> IRBuildRes<GlobalRef> {
        if let Some(global) = self.module.as_ref().globals.borrow().get(name) {
            return Err(IRBuildError::GlobalDefExists(name.to_string(), *global));
        }
        let func = Func::new_extern(functy, name, self.type_ctx());
        let funcref = GlobalRef::from_allocs(self.allocs_mut(), func.into_ir());
        funcref.register_to_symtab(self.module.as_mut());
        Ok(funcref)
    }

    pub fn define_function_with_unreachable(
        &mut self,
        name: &str,
        functy: FuncTypeRef,
    ) -> IRBuildRes<GlobalRef> {
        let module = self.module.as_mut();
        if let Some(global) = module.globals.borrow().get(name) {
            return Err(IRBuildError::GlobalDefExists(name.to_string(), *global));
        }
        let func = Func::new_with_unreachable(module, functy, name);
        let (block, inst) = {
            let entry = func.get_entry();
            let inst = entry.get_terminator(module);
            (entry, inst.get_inst())
        };
        let func = GlobalRef::from_allocs(module, func.into_ir());
        func.register_to_symtab(module);
        self.full_focus = IRFullFocus { func, block, inst };
        Ok(func)
    }

    pub fn var_builder(&mut self, name: impl ToString, content_ty: ValTypeID) -> IRVarBuilder<'_> {
        IRVarBuilder::new(self.module.as_mut(), name, content_ty)
    }
}

pub struct IRSwitchBuilder<'a, M: AsMut<Module> + AsRef<Module>> {
    pub editor: &'a mut IRBuilder<M>,
    pub cond: ValueSSA,
    pub default: BlockRef,
    pub cases: BTreeMap<i128, BlockRef>,
}

impl<'a, M: AsMut<Module> + AsRef<Module>> IRSwitchBuilder<'a, M> {
    pub fn add_case(&mut self, case: i128, block: BlockRef) -> &mut Self {
        self.cases.insert(case, block);
        self
    }

    pub fn build(self) -> IRBuildRes<(IRManaged<'a, InstRef>, SwitchRef)> {
        let Self { editor, cond, default, cases } = self;
        let (old, new) = editor.focus_set_switch(cond, default, cases.into_iter())?;
        let new = SwitchRef::from_raw_nocheck(new);
        Ok((old, new))
    }
}

impl<ModuleT> IRBuilder<ModuleT>
where
    ModuleT: AsMut<Module> + AsRef<Module>,
{
    fn null_block_focus(&self) -> bool {
        self.full_focus.block.is_null()
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
    pub fn focus_replace_terminator(&mut self, terminator: InstData) -> TermiBuildRes<'_> {
        let full_focus = self.full_focus.clone();
        if full_focus.block.is_null() {
            return Err(IRBuildError::NullFocus);
        }
        // Replace the current terminator with the new one.
        let allocs = self.allocs_mut();
        let new_terminator = InstRef::from_alloc(&mut allocs.insts, terminator);

        let old_terminator = full_focus
            .block
            .set_terminator(allocs, new_terminator)
            .map_err(IRBuildError::from)?
            .release();

        if full_focus.inst == old_terminator {
            self.full_focus.inst = new_terminator;
        }
        Ok((
            IRManaged::new(old_terminator, self.allocs_mut()),
            new_terminator,
        ))
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
        if self.null_block_focus() {
            Err(IRBuildError::NullFocus)
        } else {
            let jump = Jump::new(&self.get_allocs().blocks, jump_to);
            self.focus_replace_terminator(InstData::Jump(jump))
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
        if self.null_block_focus() {
            Err(IRBuildError::NullFocus)
        } else {
            let allocs = self.allocs_mut();
            let br = Br::new(&allocs, cond, if_true, if_false);
            self.focus_replace_terminator(InstData::Br(br))
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
    pub fn focus_set_switch(
        &mut self,
        cond: ValueSSA,
        default: BlockRef,
        cases: impl Iterator<Item = (i128, BlockRef)>,
    ) -> TermiBuildRes<'_> {
        if self.null_block_focus() {
            return Err(IRBuildError::NullFocus);
        }
        let allocs = self.allocs_mut();
        let switch = Switch::new(allocs, cond, default);
        for (case, block) in cases {
            switch.set_case(&allocs.blocks, case, block);
        }
        self.focus_replace_terminator(InstData::Switch(switch))
    }

    pub fn switch_builder(
        &mut self,
        cond: ValueSSA,
        default: BlockRef,
    ) -> IRSwitchBuilder<'_, ModuleT> {
        IRSwitchBuilder { editor: self, cond, default, cases: BTreeMap::new() }
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
        if self.null_block_focus() {
            Err(IRBuildError::NullFocus)
        } else {
            let unreachable_i = InstData::new_unreachable();
            self.focus_replace_terminator(unreachable_i)
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
        if self.null_block_focus() {
            return Err(IRBuildError::NullFocus);
        }
        let allocs = self.allocs_mut();
        let ret_ty = retval.get_valtype(allocs);
        let ret = Ret::with_retval(allocs, ret_ty, retval);
        self.focus_replace_terminator(InstData::Ret(ret))
    }
}

impl<ModuleT> IRBuilder<ModuleT>
where
    ModuleT: AsMut<Module> + AsRef<Module>,
{
    pub fn edit_allow_terminator_degrade(&mut self, allow: bool) {
        let focus_check = match self.focus_check {
            IRFocusCheckOption::Degrade(_, phi) => IRFocusCheckOption::Degrade(allow, phi),
            IRFocusCheckOption::Ignore => IRFocusCheckOption::Degrade(allow, false),
        };
        self.focus_check = focus_check;
    }
    pub fn edit_allow_phi_degrade(&mut self, allow: bool) {
        let focus_check = match self.focus_check {
            IRFocusCheckOption::Degrade(term, _) => IRFocusCheckOption::Degrade(term, allow),
            IRFocusCheckOption::Ignore => IRFocusCheckOption::Degrade(false, allow),
        };
        self.focus_check = focus_check;
    }
    pub fn allows_terminator_degrade(&self) -> bool {
        match self.focus_check {
            IRFocusCheckOption::Degrade(term, _) => term,
            IRFocusCheckOption::Ignore => true,
        }
    }
    pub fn allows_phi_degrade(&self) -> bool {
        match self.focus_check {
            IRFocusCheckOption::Degrade(_, phi) => phi,
            IRFocusCheckOption::Ignore => true,
        }
    }
    pub fn is_full_strict_insert_mode(&self) -> bool {
        match self.focus_check {
            IRFocusCheckOption::Degrade(t, p) => !t && !p,
            IRFocusCheckOption::Ignore => true,
        }
    }

    fn get_inst_split_pos(&self) -> IRBuildRes<InstRef> {
        let old_focus = self.full_focus.clone();
        if old_focus.inst.is_null() {
            // Focus is a block.
            Ok(InstRef::new_null())
        } else if self.focus_check == IRFocusCheckOption::Ignore {
            Ok(old_focus.inst)
        } else {
            self.get_inst_split_pos_by_check()
        }
    }

    fn focus_inst_get_kind(focus_inst: InstRef, allocs: &IRAllocs) -> FocusInstKind {
        match focus_inst.to_inst(&allocs.insts) {
            InstData::Phi(..) => FocusInstKind::Phi,
            x if x.is_terminator() => FocusInstKind::Terminator,
            _ => FocusInstKind::Normal,
        }
    }

    fn get_inst_split_pos_by_check(&self) -> IRBuildRes<InstRef> {
        let old_focus = self.full_focus.clone();
        // Focus is an instruction.
        let focus_inst = old_focus.inst;
        let allocs = self.get_allocs();
        let focus_kind = Self::focus_inst_get_kind(focus_inst, allocs);

        // Case 1: Focus is a terminator.
        //
        // - When terminator degrade is disabled, the function will return an error.
        // - Otherwise, the function will degrade to a block-based split.
        if focus_kind == FocusInstKind::Terminator {
            if self.allows_terminator_degrade() {
                Ok(InstRef::new_null())
            } else {
                Err(IRBuildError::InsertPosIsTerminator(focus_inst))
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
                Ok(old_focus.block.to_data(&allocs.blocks).phi_end)
            } else {
                Err(IRBuildError::InsertPosIsPhi(focus_inst))
            }
        }
        // Case 3: Focus is a PHI-end guide node or a normal instruction.
        //
        // - The function will move all instructions after the focus to the new block.
        // - The focus will be adjusted to the new block.
        else {
            Ok(focus_inst)
        }
    }

    fn insert_new_block(&mut self, block: BlockData) -> Result<BlockRef, IRBuildError> {
        let next_block = BlockRef::new(self.allocs_mut(), block);
        let old_focus = self.full_focus.clone();
        let focus_func = FuncRef(old_focus.func);
        let focus_block = old_focus.block;
        if focus_block.is_null() {
            let allocs = self.allocs_mut();
            focus_func
                .to_data(&allocs.globals)
                .add_block_ref(allocs, next_block)
                .map_err(IRBuildError::ListError)?;
        } else {
            let allocs = self.allocs_mut();
            let body = focus_func.to_data(&allocs.globals).get_body().unwrap();
            body.node_add_next(&allocs.blocks, focus_block, next_block)
                .map_err(IRBuildError::ListError)?;
        }
        Ok(next_block)
    }

    pub fn split_current_block_from_focus(&mut self) -> IRBuildRes<BlockRef> {
        let inst_split_pos = self.get_inst_split_pos()?;
        let IRFullFocus { func, block: old_block, .. } = self.full_focus;
        let new_bb = self.split_current_block_from_terminator()?;
        let new_focus = IRFullFocus { func, block: new_bb, inst: InstRef::new_null() };

        if inst_split_pos.is_null() {
            // Focus is a block, degrade to a terminator-based split.
            // Create new focus pointing to the new block
            self.full_focus = new_focus;
            return Ok(old_block);
        }

        // Now move all instructions after the `inst_split_pos` to the new block.
        let allocs = self.get_allocs();
        let new_block = new_bb.to_data(&allocs.blocks);
        loop {
            let Some(next) = inst_split_pos.get_next_ref(&allocs.insts) else {
                panic!("Invalid insert position: {inst_split_pos:?}");
            };
            match next.get_kind(allocs) {
                InstKind::PhiInstEnd => continue,
                x if x.is_terminator() => break,
                InstKind::ListGuideNode => break,
                _ => {}
            }
            next.detach_self(allocs)
                .expect("Failed to unplug inst from `to_insert`");
            new_block.build_add_inst(&allocs.insts, next);
        }
        // Create new focus pointing to the new block and return old block
        self.full_focus = new_focus;
        Ok(old_block)
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
    pub fn split_current_block_from_terminator(&mut self) -> IRBuildRes<BlockRef> {
        let IRFullFocus { block: old_block, inst: old_inst, .. } = self.full_focus;
        if old_block.is_null() {
            return Err(IRBuildError::NullFocus);
        }

        // Now create a new block. After that, a new jump instruction to this block will be created.
        let new_block = {
            let block = BlockData::new_empty(self.module.as_mut());
            self.insert_new_block(block)?
        };
        let (old_terminator, jump_to_new_bb) = self.focus_set_jump_to(new_block)?;
        if old_terminator.is_null() {
            return Err(IRBuildError::BlockHasNoTerminator(old_block));
        }
        let old_terminator = old_terminator.release();
        let allocs = self.allocs_mut();

        new_block
            .set_terminator(allocs, old_terminator)
            .map_err(IRBuildError::from)?;

        // Now we need to update the PHI nodes in the successors of the original block.
        // collect the successors of the original block.
        Self::replace_successor_phis_with_block(&self.allocs_mut(), old_block, new_block);

        // If the current focus is a terminator, we need to set the focus back to the
        // new jump instruction of the old block.
        if old_inst == old_terminator {
            self.full_focus.inst = jump_to_new_bb;
        }
        Ok(new_block)
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
                phi_value_use.kind.set(UseKind::PhiIncomingValue(block_idx));
            },
        )
    }
}

pub struct PhiBuilder<'a, M: AsMut<Module> + AsRef<Module>> {
    pub editor: &'a mut IRBuilder<M>,
    pub dest: ValTypeID,
    pub incomings: BTreeMap<BlockRef, ValueSSA>,
}

impl<'a, M: AsMut<Module> + AsRef<Module>> PhiBuilder<'a, M> {
    pub fn new(editor: &'a mut IRBuilder<M>, dest: ValTypeID) -> Self {
        PhiBuilder { editor, dest, incomings: BTreeMap::new() }
    }

    pub fn add_income(&mut self, block: BlockRef, value: ValueSSA) -> &mut Self {
        self.incomings.insert(block, value);
        self
    }

    pub fn build(self) -> IRBuildRes<InstRef> {
        let Self { editor, dest, incomings } = self;
        let phi = PhiNode::new(dest);
        for (bb, val) in incomings {
            phi.set_income(editor.get_allocs(), bb, val)
                .map_err(IRBuildError::PhiNodeError)?;
        }
        editor.add_inst(InstData::Phi(phi))
    }
}

impl<ModuleT> IRBuilder<ModuleT>
where
    ModuleT: AsMut<Module> + AsRef<Module>,
{
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
    pub fn add_inst(&mut self, inst: InstData) -> IRBuildRes<InstRef> {
        let IRFullFocus { func: focus_func, block: focus_bb, inst: focus_inst } = self.full_focus;
        if focus_func.is_null() || focus_bb.is_null() {
            return Err(IRBuildError::NullFocus);
        }
        if focus_inst.is_null() {
            // Focus is a block.
            return self.add_inst_on_block_focus(inst);
        }

        // Focus is an instruction.
        let (degrade_terminator, degrade_phi) = match self.focus_check {
            IRFocusCheckOption::Degrade(t, p) => (t, p),
            IRFocusCheckOption::Ignore => {
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
                        IRBuildError::InstIsTerminator(focus_inst)
                    } else {
                        IRBuildError::InstIsPhi(focus_inst)
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
                    Err(IRBuildError::InsertPosIsPhi(focus_inst))
                }
            }

            (FocusInstKind::Terminator, FocusInstKind::Terminator) => self
                .focus_replace_terminator(inst)
                .map(|(_, new_termi)| new_termi),
            (FocusInstKind::Terminator, _) => {
                if degrade_terminator {
                    self.add_inst_on_block_focus(inst)
                } else {
                    Err(IRBuildError::InsertPosIsTerminator(focus_inst))
                }
            }
        }
    }

    fn add_inst_after_focus_ignore_check(&mut self, inst: InstData) -> IRBuildRes<InstRef> {
        let IRFullFocus { block, inst: focus_inst, .. } = self.full_focus;
        let new_ref = InstRef::from_alloc(&mut self.allocs_mut().insts, inst);
        let allocs = self.allocs_mut();
        block
            .insts(allocs)
            .node_add_next(&allocs.insts, focus_inst, new_ref)
            .map_err(IRBuildError::ListError)?;
        Ok(new_ref)
    }
    fn add_inst_on_block_focus(&mut self, inst: InstData) -> IRBuildRes<InstRef> {
        let focus_bb = self.full_focus.block;
        let allocs = self.allocs_mut();
        let instref = InstRef::new(allocs, inst);
        focus_bb.add_inst(allocs, instref);
        return Ok(instref);
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

    /// Create a new PHI-Node builder.
    pub fn phi_builder(&mut self, ret_type: ValTypeID) -> PhiBuilder<'_, ModuleT> {
        PhiBuilder::new(self, ret_type)
    }

    /// 添加 Store 指令。
    pub fn add_store_inst(
        &mut self,
        target: ValueSSA,
        source: ValueSSA,
        align: usize,
    ) -> InstBuildRes {
        let module = self.module.as_mut();
        let mut store_op = StoreOp::new(&mut module.allocs, &module.type_ctx, source, target);
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

    pub fn add_call_inst(
        &mut self,
        callee: GlobalRef,
        args: impl Iterator<Item = ValueSSA> + Clone,
    ) -> InstBuildRes {
        let module = self.module.as_mut();
        let call = CallOp::from_allocs(&mut module.allocs, &module.type_ctx, callee, args);
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
}

pub struct GEPEditBuilder<'a, M: AsMut<Module> + AsRef<Module>> {
    inner: GEPBuilder,
    edit: &'a mut IRBuilder<M>,
}

impl<'a, M: AsMut<Module> + AsRef<Module>> GEPEditBuilder<'a, M> {
    pub fn base_ptr(mut self, base_ptr: ValueSSA) -> Self {
        self.inner.base_ptr(base_ptr);
        self
    }
    pub fn base_ty(mut self, base_ty: ValTypeID) -> Self {
        self.inner.base_ty(base_ty);
        self
    }
    pub fn storage_align_log2(mut self, align_log2: u8) -> Self {
        self.inner.storage_align_log2(align_log2);
        self
    }
    pub fn ret_align_log2(mut self, align_log2: u8) -> Self {
        self.inner.ret_align_log2(align_log2);
        self
    }
    pub fn add_index(mut self, index: ValueSSA) -> Self {
        self.inner.add_index(index);
        self
    }
    pub fn build(mut self, indices: impl IntoIterator<Item = ValueSSA>) -> IRBuildRes<InstRef> {
        let gep = self.inner.build(self.edit.module.as_ref(), indices);
        self.edit.add_inst(InstData::GEP(gep))
    }
}

impl<ModuleT> IRBuilder<ModuleT>
where
    ModuleT: AsMut<Module> + AsRef<Module>,
{
    pub fn gep_builder(
        &mut self,
        base_ptr: ValueSSA,
        base_ty: ValTypeID,
    ) -> GEPEditBuilder<'_, ModuleT> {
        GEPEditBuilder {
            inner: GEPBuilder::new(self.module.as_ref(), base_ptr, base_ty),
            edit: self,
        }
    }
}

pub struct AmoRmwEditBuilder<'a, M: AsMut<Module> + AsRef<Module>> {
    inner: AmoRmwBuilder,
    edit: &'a mut IRBuilder<M>,
}

impl<'a, M: AsMut<Module> + AsRef<Module>> AmoRmwEditBuilder<'a, M> {
    pub fn opcode(mut self, opcode: Opcode) -> Self {
        self.inner = self.inner.opcode(opcode);
        self
    }
    pub fn value_ty(mut self, value_ty: ValTypeID) -> Self {
        self.inner = self.inner.value_ty(value_ty);
        self
    }
    pub fn ordering(mut self, ordering: AmoOrdering) -> Self {
        self.inner = self.inner.ordering(ordering);
        self
    }
    pub fn volatile(mut self, is_volatile: bool) -> Self {
        self.inner = self.inner.volatile(is_volatile);
        self
    }
    pub fn align(mut self, align: usize) -> Self {
        self.inner = self.inner.align(align);
        self
    }
    pub fn align_log2(mut self, align_log2: u8) -> Self {
        self.inner = self.inner.align_log2(align_log2);
        self
    }
    pub fn scope(mut self, scope: SyncScope) -> Self {
        self.inner = self.inner.scope(scope);
        self
    }
    pub fn ptr_operand(mut self, ptr_operand: impl ISubValueSSA) -> Self {
        self.inner = self.inner.ptr_operand(ptr_operand);
        self
    }
    pub fn val_operand(mut self, val_operand: impl ISubValueSSA) -> Self {
        self.inner = self.inner.val_operand(val_operand);
        self
    }
    pub fn build(self) -> IRBuildRes<InstRef> {
        let amormw = self.inner.build(self.edit.get_allocs());
        self.edit.add_inst(InstData::AmoRmw(amormw))
    }
}

impl<ModuleT> IRBuilder<ModuleT>
where
    ModuleT: AsMut<Module> + AsRef<Module>,
{
    pub fn amo_rmw_builder(
        &mut self,
        opcode: Opcode,
        value_ty: ValTypeID,
    ) -> AmoRmwEditBuilder<'_, ModuleT> {
        AmoRmwEditBuilder { inner: AmoRmwBuilder::new(opcode, value_ty), edit: self }
    }
}

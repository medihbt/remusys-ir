use mtb_entity::EntityListError;
use thiserror::Error;

use crate::{
    ir::{
        BlockID, BlockObj, FuncBuilder, FuncID, GlobalID, GlobalVar, GlobalVarBuilder, GlobalVarID,
        IGlobalVarBuildable, IRAllocs, ISubInstID, ISubValueSSA, InstID, InstObj, ManagedInst,
        Module, PoolAllocatedDisposeErr,
        inst::{BrInstID, JumpInstID, SwitchInstID, UnreachableInstID},
    },
    typing::{ArchInfo, FuncTypeID, TypeContext, ValTypeID},
};

#[derive(Debug, Clone)]
pub struct IRFullFocus {
    pub func: FuncID,
    pub block: Option<BlockID>,
    pub inst: Option<InstID>,
}
impl IRFullFocus {
    pub fn is_block_focus(&self) -> bool {
        self.block.is_some() && self.inst.is_none()
    }
    pub fn is_inst_focus(&self) -> bool {
        self.block.is_some() && self.inst.is_some()
    }

    pub fn new_func_focus(func: FuncID) -> Self {
        Self { func, block: None, inst: None }
    }
}

#[derive(Debug, Clone)]
pub enum IRFocus {
    Block(BlockID),
    Inst(InstID),
}

impl IRFocus {
    pub fn from_full(full: &IRFullFocus) -> Option<Self> {
        match (full.block, full.inst) {
            (Some(b), None) => Some(IRFocus::Block(b)),
            (Some(_), Some(i)) => Some(IRFocus::Inst(i)),
            _ => None,
        }
    }

    pub fn to_full(&self, allocs: impl AsRef<IRAllocs>) -> IRFullFocus {
        let allocs = allocs.as_ref();
        match self {
            IRFocus::Block(block) => {
                let Some(func) = block.get_parent_func(allocs) else {
                    panic!("BlockID has no parent FuncID");
                };
                IRFullFocus { func, block: Some(*block), inst: None }
            }
            IRFocus::Inst(inst) => {
                let Some(block) = inst.get_parent(allocs) else {
                    panic!("InstID has no parent BlockID");
                };
                let Some(func) = block.get_parent_func(allocs) else {
                    panic!("BlockID has no parent FuncID");
                };
                IRFullFocus { func, block: Some(block), inst: Some(*inst) }
            }
        }
    }
}

/// 当某个操作的焦点不合法时, `IRBuilder` 应该做什么.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FocusDegradeOp {
    /// 降级为块级焦点
    AsBlockOp,
    /// 严格模式, 抛出错误
    Strict,
    /// 无视风险, 继续在原地操作
    Ignore,
}

#[derive(Debug, Clone, Copy)]
pub struct FocusDegradeConfig {
    /// 当尝试将指令添加到终结指令时的降级操作
    pub add_inst_to_terminator: FocusDegradeOp,
    /// 当尝试将 Phi 指令添加到非 Phi 结点时的降级操作
    pub add_phi_to_inst: FocusDegradeOp,
    /// 如果焦点在 Phi 指令上, 添加非 Phi 指令时应该执行的降级操作
    pub add_inst_to_phi: FocusDegradeOp,
    /// 如果焦点在 Phi 指令上, 拆分基本块时应该执行的降级操作
    pub split_block_on_phi: FocusDegradeOp,
    /// 如果焦点在终结指令上, 拆分基本块时应该执行的降级操作
    pub split_block_on_terminator: FocusDegradeOp,
}
impl Default for FocusDegradeConfig {
    /// 便捷且安全的默认策略：当目标位置不合法时退化为“块级操作”，
    /// 由 Builder 选择合理插入/拆分点：
    /// - 普通指令插入到终结符之前
    /// - 终结符作为块末尾追加（必要时先移动焦点到块）
    /// - Phi 指令插入到 phi_end 之前
    /// - 焦点在 Phi 时添加普通指令，改为在 phi 区段之后插入
    /// - 拆分块：在 phi 区段之后或终结符之前切分
    fn default() -> Self {
        Self {
            add_inst_to_terminator: FocusDegradeOp::AsBlockOp,
            add_phi_to_inst: FocusDegradeOp::AsBlockOp,
            add_inst_to_phi: FocusDegradeOp::Strict,
            split_block_on_phi: FocusDegradeOp::AsBlockOp,
            split_block_on_terminator: FocusDegradeOp::AsBlockOp,
        }
    }
}

#[derive(Debug, Error)]
pub enum IRBuildError {
    #[error("Global definition {0} already exists: {1:p}")]
    GlobalDefExists(String, GlobalID),
    #[error("Global definition not found: {0}")]
    GlobalDefNotFound(String),

    #[error("Instruction list error: {0:?}")]
    InstListError(EntityListError<InstObj>),
    #[error("Block list error: {0:?}")]
    BlockListError(EntityListError<BlockObj>),
    #[error("Null focus")]
    NullFocus,
    #[error("Split focus is PHI: %inst{0:p}")]
    SplitFocusIsPhi(InstID),
    #[error("Split focus is guide node: %inst{0:p}")]
    SplitFocusIsGuideNode(InstID),

    #[error("Block has no terminator: {0:?}")]
    BlockHasNoTerminator(BlockID),
    #[error("Instruction is terminator: %inst{0:p}")]
    InstIsTerminator(InstID),
    #[error("Instruction is guide node: %inst{0:p}")]
    InstIsGuideNode(InstID),
    #[error("Instruction is PHI: %inst{0:p}")]
    InstIsPhi(InstID),

    #[error("Insert position is PHI: %inst{0:p}")]
    InsertPosIsPhi(InstID),
    #[error("Insert position is terminator: %inst{0:p}")]
    InsertPosIsTerminator(InstID),
    #[error("Insert position is guide node: %inst{0:p}")]
    InsertPosIsGuideNode(InstID),
    #[error("Invalid insert position: %inst{0:p}")]
    InvalidInsertPos(InstID),

    #[error("Dispose error: {0:?}")]
    DisposeErr(#[from] PoolAllocatedDisposeErr),
}
impl From<EntityListError<InstObj>> for IRBuildError {
    fn from(e: EntityListError<InstObj>) -> Self {
        IRBuildError::InstListError(e)
    }
}
impl From<EntityListError<BlockObj>> for IRBuildError {
    fn from(e: EntityListError<BlockObj>) -> Self {
        IRBuildError::BlockListError(e)
    }
}
pub type IRBuildRes<T = ()> = Result<T, IRBuildError>;
pub type TermiBuildRes<'ir, T> = IRBuildRes<(ManagedInst<'ir>, T)>;

pub struct IRBuilder<ModuleT = Module> {
    pub module: ModuleT,
    pub full_focus: Option<IRFullFocus>,
    pub focus_degrade: FocusDegradeConfig,
}
impl IRBuilder<Module> {
    pub fn take(mut self) -> Module {
        self.module.allocs.free_disposed();
        self.module
    }
    #[inline(never)]
    pub fn new_inlined(arch: ArchInfo) -> Self {
        Self {
            module: Module::new(arch),
            full_focus: None,
            focus_degrade: FocusDegradeConfig::default(),
        }
    }
}
impl<ModuleT: AsRef<Module>> IRBuilder<ModuleT> {
    pub fn new(module: ModuleT) -> Self {
        Self {
            module,
            full_focus: None,
            focus_degrade: FocusDegradeConfig::default(),
        }
    }

    pub fn module(&self) -> &Module {
        self.module.as_ref()
    }
    pub fn allocs(&self) -> &IRAllocs {
        &self.module.as_ref().allocs
    }
    pub fn tctx(&self) -> &TypeContext {
        &self.module.as_ref().tctx
    }
    pub fn try_get_focus(&self) -> Option<IRFocus> {
        let Some(full) = &self.full_focus else {
            return None;
        };
        IRFocus::from_full(full)
    }
    pub fn set_focus(&mut self, focus: IRFocus) {
        let full = focus.to_full(self.allocs());
        self.full_focus = Some(full);
    }

    /// Switch the focus to the terminator of the current block.
    pub fn switch_focus_to_terminator(&mut self) -> IRBuildRes {
        let Some(mut focus) = self.full_focus.clone() else {
            return Err(IRBuildError::NullFocus);
        };
        let block = focus.block.ok_or(IRBuildError::NullFocus)?;
        let Some(termi) = block.try_get_terminator(self.allocs()) else {
            return Err(IRBuildError::BlockHasNoTerminator(block));
        };
        focus.inst = Some(termi);
        self.full_focus = Some(focus);
        Ok(())
    }

    pub fn build_global_var(
        &mut self,
        name: impl Into<String>,
        ty: ValTypeID,
        build: impl FnOnce(&mut GlobalVarBuilder),
    ) -> IRBuildRes<GlobalVarID> {
        let mut builder = GlobalVar::builder(name, ty);
        build(&mut builder);
        match builder.build_id(self.module()) {
            Ok(gid) => Ok(gid),
            Err(e) => Err(IRBuildError::GlobalDefExists(builder.name, e)),
        }
    }

    pub fn build_func(
        &mut self,
        name: impl Into<String>,
        functy: FuncTypeID,
        build: impl FnOnce(&mut FuncBuilder),
    ) -> IRBuildRes<FuncID> {
        let mut builder = FuncID::builder(self.tctx(), name.into(), functy);
        build(&mut builder);
        match builder.build_id(self.module()) {
            Ok(fid) => Ok(fid),
            Err(e) => Err(IRBuildError::GlobalDefExists(builder.name, e)),
        }
    }

    fn _isnull_block_focus(&self) -> bool {
        match &self.full_focus {
            Some(focus) => focus.block.is_none(),
            None => true,
        }
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
    pub fn focus_set_terminator(&mut self, termi: impl ISubInstID) -> IRBuildRes<ManagedInst<'_>> {
        let Some(mut focus) = self.full_focus.clone() else {
            return Err(IRBuildError::NullFocus);
        };
        let block = focus.block.ok_or(IRBuildError::NullFocus)?;
        let allocs = self.allocs();
        let termi = termi.into_ir();
        let managed = match block.try_set_terminator_inst(allocs, termi) {
            Ok(Some(managed)) => managed,
            Ok(None) => return Err(IRBuildError::BlockHasNoTerminator(block)),
            Err(e) => return Err(e.into()),
        };
        let old_termi = managed.release();
        focus.inst = Some(termi);
        self.full_focus = Some(focus);
        Ok(ManagedInst::new(self.allocs(), old_termi))
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
    pub fn focus_set_jump_to(&mut self, jump_to: BlockID) -> TermiBuildRes<'_, JumpInstID> {
        let jump = JumpInstID::new_uninit(self.allocs());
        jump.set_target(self.allocs(), jump_to);
        let managed = self.focus_set_terminator(jump)?;
        Ok((managed, jump))
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
        cond: impl ISubValueSSA,
        then_bb: BlockID,
        else_bb: BlockID,
    ) -> TermiBuildRes<'_, BrInstID> {
        let br = BrInstID::new(self.allocs(), cond.into_ir(), then_bb, else_bb);
        let managed = self.focus_set_terminator(br)?;
        Ok((managed, br))
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
    pub fn focus_set_switch_to(
        &mut self,
        discrim: impl ISubValueSSA,
        default_bb: BlockID,
        cases: impl IntoIterator<Item = (i64, BlockID)>,
    ) -> TermiBuildRes<'_, SwitchInstID> {
        let switch = SwitchInstID::from_cases(self.allocs(), discrim.into_ir(), cases, default_bb);
        let managed = self.focus_set_terminator(switch)?;
        Ok((managed, switch))
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
    pub fn focus_set_unreachable(&mut self) -> TermiBuildRes<'_, UnreachableInstID> {
        let unreach = UnreachableInstID::new(self.allocs());
        let managed = self.focus_set_terminator(unreach)?;
        Ok((managed, unreach))
    }
}

/// IR Builder as basic block splitter
impl<ModuleT: AsRef<Module>> IRBuilder<ModuleT> {}

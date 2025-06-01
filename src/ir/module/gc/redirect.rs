use crate::{
    base::{
        NullableValue,
        slablist::{SlabRefList, SlabRefListNode, SlabRefListNodeHead},
        slabref::SlabRef,
    },
    ir::{
        ValueSSA,
        block::{BlockRef, jump_target::JumpTargetRef},
        constant::expr::{ConstExprData, ConstExprRef},
        global::{GlobalData, GlobalRef},
        inst::{
            InstData, InstDataCommon, InstRef,
            terminator::JumpCommon,
            usedef::{UseKind, UseRef},
        },
        module::{Module, ModuleError},
    },
};

use super::{liveset::IRRefLiveSet, mark::MarkVisitor};

pub(super) struct Redirector<'a> {
    pub(super) module: &'a Module,
    pub(super) live_set: IRRefLiveSet,
}

impl<'a> Redirector<'a> {
    pub(super) fn from_marker(marker: MarkVisitor<'a>) -> Self {
        let module = marker.module;
        let marker_inner = marker.inner.into_inner();
        let live_set = marker_inner.live_set;
        Self { module, live_set }
    }
}

impl<'a> Redirector<'a> {
    pub(super) fn redirect_module(&self) -> Result<(), ModuleError> {
        self.redirect_insts().unwrap();
        self.redirect_blocks().unwrap();
        self.redirect_global_alloc().unwrap();
        self.redirect_exprs().unwrap();
        self.redirect_use().unwrap();
        self.redirect_jt().unwrap();
        self.redirect_global_def().unwrap();
        Ok(())
    }

    fn _redirect_value_ref(&self, value: &mut ValueSSA) -> Result<(), ModuleError> {
        match value {
            ValueSSA::Inst(inst) => self._redirect_inst_ref(inst, false),
            ValueSSA::Block(block) => self._redirect_block_ref(block, false),
            ValueSSA::Global(global) => self._redirect_global_ref(global, false),
            ValueSSA::ConstExpr(expr) => self._redirect_expr_ref(expr),
            ValueSSA::FuncArg(func, _) => self._redirect_global_ref(func, false),
            _ => Ok(()),
        }
    }
    fn _redirect_inst_ref(
        &self,
        inst_ref: &mut InstRef,
        nullable: bool,
    ) -> Result<(), ModuleError> {
        if inst_ref.is_null() {
            if nullable {
                return Ok(());
            } else {
                panic!("InstRef is null, but not nullable");
            }
        }
        let new_pos = self.live_set.get_value_new_pos(ValueSSA::Inst(*inst_ref))?;
        *inst_ref = InstRef::from_handle(new_pos);
        Ok(())
    }
    fn _redirect_block_ref(
        &self,
        block_ref: &mut BlockRef,
        nullable: bool,
    ) -> Result<(), ModuleError> {
        if block_ref.is_null() {
            if nullable {
                return Ok(());
            } else {
                panic!("BlockRef is null, but not nullable");
            }
        }
        let new_pos = self
            .live_set
            .get_value_new_pos(ValueSSA::Block(*block_ref))?;
        *block_ref = BlockRef::from_handle(new_pos);
        Ok(())
    }
    fn _redirect_global_ref(
        &self,
        global_ref: &mut GlobalRef,
        nullable: bool,
    ) -> Result<(), ModuleError> {
        if global_ref.is_null() {
            if nullable {
                return Ok(());
            } else {
                panic!("GlobalRef is null, but not nullable");
            }
        }
        let new_pos = self
            .live_set
            .get_value_new_pos(ValueSSA::Global(*global_ref))?;
        *global_ref = GlobalRef::from_handle(new_pos);
        Ok(())
    }
    fn _redirect_expr_ref(&self, expr_ref: &mut ConstExprRef) -> Result<(), ModuleError> {
        let new_pos = self
            .live_set
            .get_value_new_pos(ValueSSA::ConstExpr(*expr_ref))?;
        *expr_ref = ConstExprRef::from_handle(new_pos);
        Ok(())
    }
    fn _redirect_parent_bb(&self, parent_bb: &mut Option<BlockRef>) -> Result<(), ModuleError> {
        if let Some(bb) = parent_bb {
            self._redirect_block_ref(bb, true)?;
        }
        Ok(())
    }
    fn _redirect_use_ref(&self, use_ref: &mut UseRef) -> Result<(), ModuleError> {
        let new_pos = self.live_set.get_use_new_pos(*use_ref)?;
        *use_ref = UseRef::from_handle(new_pos);
        Ok(())
    }
    fn _redirect_jt_ref(&self, jt_ref: &mut JumpTargetRef) -> Result<(), ModuleError> {
        let new_pos = self.live_set.get_jt_new_pos(*jt_ref)?;
        *jt_ref = JumpTargetRef::from_handle(new_pos);
        Ok(())
    }

    fn _redirect_jump_common(&self, jmp_common: &mut JumpCommon) -> Result<(), ModuleError> {
        if jmp_common._condition.is_nonnull() {
            self._redirect_use_ref(&mut jmp_common._condition)?;
        }
        let jts = &mut jmp_common._targets;
        self._redirect_jt_ref(&mut jts._head)?;
        self._redirect_jt_ref(&mut jts._tail)?;
        Ok(())
    }
}

impl<'a> Redirector<'a> {
    fn redirect_insts(&self) -> Result<(), ModuleError> {
        for (oldpos, newpos) in self.live_set.insts.iter().enumerate() {
            // Skip if the instruction is not live
            if *newpos == usize::MAX {
                continue;
            }
            let oldpos = InstRef::from_handle(oldpos);
            let mut oldpos_data = self.module.mut_inst(oldpos);

            if let Some(common) = oldpos_data.common_mut() {
                // Redirect the instruction common data
                self.redirect_inst_common(InstRef::from_handle(*newpos), common)?;
            }

            // Redirect the instruction data
            match &mut *oldpos_data {
                InstData::ListGuideNode(head, parent) => {
                    self.redirect_inst_node_header(head.get_mut())?;
                    self._redirect_block_ref(parent.get_mut(), false)?;
                }
                InstData::PhiInstEnd(_) | InstData::Unreachable(_) => {}
                InstData::Ret(_, ret) => {
                    self._redirect_use_ref(&mut ret.retval)?;
                }
                InstData::Jump(_, jump) => {
                    self._redirect_jump_common(&mut jump.0)?;
                }
                InstData::Br(_, br) => {
                    self._redirect_jump_common(&mut br._common)?;
                    self._redirect_jt_ref(&mut br.if_true)?;
                    self._redirect_jt_ref(&mut br.if_false)?;
                }
                InstData::Switch(_, switch) => {
                    self._redirect_jump_common(&mut switch._common)?;
                    self._redirect_jt_ref(&mut switch._default)?;
                    for (_, jt) in &mut *switch._cases.borrow_mut() {
                        self._redirect_jt_ref(jt)?;
                    }
                    switch.sort_cases();
                }
                InstData::Phi(_, phi_op) => {
                    for phi_operand in &mut *phi_op.get_from_all_mut() {
                        self._redirect_block_ref(&mut phi_operand.from_bb, false)?;
                        self._redirect_use_ref(&mut phi_operand.from_bb_use)?;
                        self._redirect_use_ref(&mut phi_operand.from_value_use)?;
                    }
                }
                InstData::Alloca(..) => { /* Alloca instruction has no managed value */ }
                InstData::Load(_, load_op) => {
                    self._redirect_use_ref(&mut load_op.source)?;
                }
                InstData::Store(_, store_op) => {
                    self._redirect_use_ref(&mut store_op.source)?;
                    self._redirect_use_ref(&mut store_op.target)?;
                }
                InstData::Select(_, select_op) => {
                    self._redirect_use_ref(&mut select_op.cond)?;
                    self._redirect_use_ref(&mut select_op.true_val)?;
                    self._redirect_use_ref(&mut select_op.false_val)?;
                }
                InstData::BinOp(_, bin_op) => {
                    self._redirect_use_ref(&mut bin_op.lhs)?;
                    self._redirect_use_ref(&mut bin_op.rhs)?;
                }
                InstData::Cmp(_, cmp_op) => {
                    self._redirect_use_ref(&mut cmp_op.lhs)?;
                    self._redirect_use_ref(&mut cmp_op.rhs)?;
                }
                InstData::Cast(_, cast_op) => {
                    self._redirect_use_ref(&mut cast_op.from_op)?;
                }
                InstData::IndexPtr(_, gep) => {
                    self._redirect_use_ref(&mut gep.base_ptr)?;
                    for idx in &mut gep.indices {
                        self._redirect_use_ref(idx)?;
                    }
                }
                InstData::Call(_, call_op) => {
                    self._redirect_use_ref(&mut call_op.callee)?;
                    for arg in &mut call_op.args {
                        self._redirect_use_ref(arg)?;
                    }
                }
                InstData::Intrin(_) => todo!("Handle intrinsic instructions"),
            }
        }

        Ok(())
    }

    fn redirect_inst_common(
        &self,
        self_newpos: InstRef,
        common: &mut InstDataCommon,
    ) -> Result<(), ModuleError> {
        common.self_ref = self_newpos;

        if common.operands.is_valid() {
            let old_head_node = common.operands._head;
            let old_tail_node = common.operands._tail;
            common.operands._head =
                UseRef::from_handle(self.live_set.get_use_new_pos(old_head_node).unwrap());
            common.operands._tail =
                UseRef::from_handle(self.live_set.get_use_new_pos(old_tail_node).unwrap());
        }

        let inner = common.inner.get_mut();
        self.redirect_inst_node_header(&mut inner._node_head)?;
        self._redirect_parent_bb(&mut inner._parent_bb)?;
        Ok(())
    }

    fn redirect_inst_node_header(
        &self,
        header: &mut SlabRefListNodeHead,
    ) -> Result<(), ModuleError> {
        let old_prev = InstRef::from_handle(header.prev);
        let old_next = InstRef::from_handle(header.next);

        if old_prev.is_nonnull() {
            header.prev = self.live_set.get_value_new_pos(ValueSSA::Inst(old_prev))?;
        }
        if old_next.is_nonnull() {
            header.next = self.live_set.get_value_new_pos(ValueSSA::Inst(old_next))?;
        }
        Ok(())
    }
}

impl<'a> Redirector<'a> {
    pub(super) fn redirect_blocks(&self) -> Result<(), ModuleError> {
        for (oldpos, newpos) in self.live_set.blocks.iter().enumerate() {
            // Skip if the block is not live
            if *newpos == usize::MAX {
                continue;
            }
            let oldpos = BlockRef::from_handle(oldpos);
            let newpos = BlockRef::from_handle(*newpos);
            let mut oldpos_data = self.module.mut_block(oldpos);

            if !oldpos_data.is_guide() {
                self._redirect_inst_reflist_controller(&mut oldpos_data.instructions)
                    .unwrap();
                self._redirect_inst_ref(oldpos_data.phi_node_end.get_mut(), true)
                    .unwrap();
            }

            let inner = oldpos_data._inner.get_mut();
            inner._self_ref = newpos;
            self._redirect_global_ref(&mut inner._parent_func, true)
                .unwrap();
            self._redirect_block_node_header(&mut inner._node_head)
                .unwrap();
        }
        Ok(())
    }

    fn _redirect_inst_reflist_controller(
        &self,
        insts: &mut SlabRefList<InstRef>,
    ) -> Result<(), ModuleError> {
        if !insts.is_valid() {
            return Ok(());
        }
        self._redirect_inst_ref(&mut insts._head, false)?;
        self._redirect_inst_ref(&mut insts._tail, false)?;
        Ok(())
    }
    fn _redirect_block_node_header(
        &self,
        header: &mut SlabRefListNodeHead,
    ) -> Result<(), ModuleError> {
        let old_prev = BlockRef::from_handle(header.prev);
        let old_next = BlockRef::from_handle(header.next);

        if old_prev.is_nonnull() {
            header.prev = self.live_set.get_value_new_pos(ValueSSA::Block(old_prev))?;
        }
        if old_next.is_nonnull() {
            header.next = self.live_set.get_value_new_pos(ValueSSA::Block(old_next))?;
        }
        Ok(())
    }
}

impl<'a> Redirector<'a> {
    pub(super) fn redirect_global_alloc(&self) -> Result<(), ModuleError> {
        for (oldpos, newpos) in self.live_set.globals.iter().enumerate() {
            // Skip if the global is not live
            if *newpos == usize::MAX {
                continue;
            }
            let oldpos = GlobalRef::from_handle(oldpos);
            let newpos = GlobalRef::from_handle(*newpos);
            let mut oldpos_data = self.module.mut_global(oldpos);

            oldpos_data.common_mut().self_ref.set(newpos);

            match &mut *oldpos_data {
                GlobalData::Alias(alias) => {
                    self._redirect_global_ref(alias.target.get_mut(), false)
                        .unwrap();
                }
                GlobalData::Var(v) => {
                    let inner = v.inner.get_mut();
                    if inner.init.is_nonnull() {
                        self._redirect_value_ref(&mut inner.init).unwrap();
                    }
                }
                GlobalData::Func(func) => {
                    let mut body = func._body.borrow_mut();
                    if let Some(body) = body.as_mut() {
                        body.func = newpos;
                        self._redirect_block_ref(&mut body.entry, false).unwrap();
                        self._redirect_block_ref(&mut body.body._head, false)
                            .unwrap();
                        self._redirect_block_ref(&mut body.body._tail, false)
                            .unwrap();
                    }
                }
            }
        }
        Ok(())
    }

    pub(super) fn redirect_global_def(&self) -> Result<(), ModuleError> {
        let mut global_def = self.module.global_defs.borrow_mut();
        for (_, pos) in global_def.iter_mut() {
            self._redirect_global_ref(pos, false)?;
        }
        Ok(())
    }

    pub(super) fn redirect_exprs(&self) -> Result<(), ModuleError> {
        for (oldpos, newpos) in self.live_set.exprs.iter().enumerate() {
            // Skip if the expression is not live
            if *newpos == usize::MAX {
                continue;
            }
            let oldpos = ConstExprRef::from_handle(oldpos);
            let mut oldpos_data = self.module.mut_expr(oldpos);

            let elems = match &mut *oldpos_data {
                ConstExprData::Array(a) => a.elems.as_mut_slice(),
                ConstExprData::Struct(s) => s.elems.as_mut_slice(),
            };

            for elem in elems {
                self._redirect_value_ref(elem)?;
            }
        }
        Ok(())
    }

    pub(super) fn redirect_use(&self) -> Result<(), ModuleError> {
        for (oldpos, newpos) in self.live_set.uses.iter().enumerate() {
            // Skip if the use is not live
            if *newpos == usize::MAX {
                continue;
            }
            let oldpos = UseRef::from_handle(oldpos);
            let mut oldpos_data = self.module.mut_use(oldpos);

            self._redirect_inst_ref(oldpos_data._user.get_mut(), false)?;
            self._redirect_value_ref(oldpos_data._operand.get_mut())?;
            self._redirect_use_node_head(oldpos_data._node_head.get_mut())?;

            match oldpos_data.kind.get_mut() {
                UseKind::PhiIncomingBlock(rela_value_use) => {
                    self._redirect_use_ref(rela_value_use)?;
                }
                UseKind::PhiIncomingValue {
                    from_bb,
                    from_bb_use,
                } => {
                    self._redirect_block_ref(from_bb, false)?;
                    self._redirect_use_ref(from_bb_use)?;
                }
                _ => {}
            }
        }
        Ok(())
    }
    pub(super) fn redirect_jt(&self) -> Result<(), ModuleError> {
        for (oldpos, newpos) in self.live_set.jts.iter().enumerate() {
            // Skip if the jump target is not live
            if *newpos == usize::MAX {
                continue;
            }
            let oldpos = JumpTargetRef::from_handle(oldpos);
            let mut oldpos_data = self.module.mut_jt(oldpos);

            self._redirect_block_ref(oldpos_data._block.get_mut(), true)?;
            self._redirect_inst_ref(oldpos_data._terminator.get_mut(), false)?;
            self._redirect_jt_node_head(oldpos_data._node_head.get_mut())?;
        }
        Ok(())
    }

    fn _redirect_use_node_head(&self, header: &mut SlabRefListNodeHead) -> Result<(), ModuleError> {
        let old_prev = UseRef::from_handle(header.prev);
        let old_next = UseRef::from_handle(header.next);

        if old_prev.is_nonnull() {
            header.prev = self.live_set.get_use_new_pos(old_prev)?;
        }
        if old_next.is_nonnull() {
            header.next = self.live_set.get_use_new_pos(old_next)?;
        }
        Ok(())
    }
    fn _redirect_jt_node_head(&self, header: &mut SlabRefListNodeHead) -> Result<(), ModuleError> {
        let old_prev = JumpTargetRef::from_handle(header.prev);
        let old_next = JumpTargetRef::from_handle(header.next);

        if old_prev.is_nonnull() {
            header.prev = self.live_set.get_jt_new_pos(old_prev)?;
        }
        if old_next.is_nonnull() {
            header.next = self.live_set.get_jt_new_pos(old_next)?;
        }
        Ok(())
    }
}

use std::cell::RefCell;

use crate::{
    base::{NullableValue, slablist::SlabRefListNodeRef, slabref::SlabRef},
    ir::{
        IValueVisitor, ValueSSA,
        block::{BlockData, BlockRef, jump_target::JumpTargetRef},
        constant::{
            data::{ConstData, IConstDataVisitor},
            expr::{Array, ConstExprRef, IConstExprVisitor, Struct},
        },
        global::{Alias, GlobalRef, IGlobalObjectVisitor, Var, func::FuncData},
        inst::{binop::*, callop::*, usedef::UseRef, visitor::IInstVisitor, *},
        module::{Module, ModuleError},
    },
    typing::{id::ValTypeID, types::FloatTypeKind},
};

use super::liveset::IRRefLiveSet;

pub(super) struct MarkVisitor<'a> {
    inner: RefCell<MarkVisitorInner>,
    module: &'a Module,
}

struct MarkVisitorInner {
    live_set: IRRefLiveSet,
    curr_func: GlobalRef,
    curr_block: BlockRef,
    curr_inst: InstRef,
    mode: MarkMode,
}

#[derive(Debug, Clone)]
enum MarkMode {
    NoCompact,
    Compact(MarkCompactMode),
}

#[derive(Debug, Clone)]
struct MarkCompactMode {
    expr_top: usize,
    global_top: usize,
    inst_top: usize,
    block_top: usize,
    jt_top: usize,
    use_top: usize,
}

impl<'a> MarkVisitor<'a> {
    pub fn from_module(module: &'a Module, should_compact: bool) -> Self {
        let live_set = IRRefLiveSet::from_module_empty(module);
        let curr_func = GlobalRef::new_null();
        let curr_block = BlockRef::new_null();
        let curr_inst = InstRef::new_null();
        let mode = if should_compact {
            MarkMode::Compact(MarkCompactMode {
                expr_top: 0,
                global_top: 0,
                inst_top: 0,
                block_top: 0,
                jt_top: 0,
                use_top: 0,
            })
        } else {
            MarkMode::NoCompact
        };

        Self {
            inner: RefCell::new(MarkVisitorInner {
                live_set,
                curr_func,
                curr_block,
                curr_inst,
                mode,
            }),
            module,
        }
    }

    pub fn mark_value(&self, value: ValueSSA) -> Result<(), ModuleError> {
        let mode = self.inner.borrow().mode.clone();

        // discard non-reference value
        let old_pos = match value {
            ValueSSA::FuncArg(..) | ValueSSA::ConstData(..) | ValueSSA::None => {
                return Ok(());
            }
            ValueSSA::Block(b) => b.get_handle(),
            ValueSSA::Inst(i) => i.get_handle(),
            ValueSSA::ConstExpr(e) => e.get_handle(),
            ValueSSA::Global(g) => g.get_handle(),
        };

        // discard null reference
        if old_pos == usize::MAX {
            return Ok(());
        }

        let (new_pos, new_mode) = match mode {
            MarkMode::NoCompact => (old_pos, mode),
            MarkMode::Compact(mut c) => match value {
                ValueSSA::Block(_) => {
                    c.block_top += 1;
                    (c.block_top - 1, MarkMode::Compact(c))
                }
                ValueSSA::Inst(_) => {
                    c.inst_top += 1;
                    (c.inst_top - 1, MarkMode::Compact(c))
                }
                ValueSSA::ConstExpr(_) => {
                    c.expr_top += 1;
                    (c.expr_top - 1, MarkMode::Compact(c))
                }
                ValueSSA::Global(_) => {
                    c.global_top += 1;
                    (c.global_top - 1, MarkMode::Compact(c))
                }
                _ => unreachable!(),
            },
        };

        // update the live set
        let mut inner = self.inner.borrow_mut();
        inner.live_set.redirect_value(value, new_pos)?;
        inner.mode = new_mode;
        Ok(())
    }

    pub fn mark_use(&self, use_ref: UseRef) -> Result<(), ModuleError> {
        let mode = self.inner.borrow().mode.clone();

        // discard non-reference value
        let old_pos = use_ref.get_handle();
        // discard null reference
        if old_pos == usize::MAX {
            return Ok(());
        }

        let (new_pos, new_mode) = match mode {
            MarkMode::NoCompact => (old_pos, mode),
            MarkMode::Compact(mut c) => {
                c.use_top += 1;
                (c.use_top - 1, MarkMode::Compact(c))
            }
        };

        // update the live set
        let mut inner = self.inner.borrow_mut();
        inner.live_set.redirect_use(use_ref, new_pos)?;
        inner.mode = new_mode;
        Ok(())
    }
    pub fn mark_jt(&self, jt_ref: JumpTargetRef) -> Result<(), ModuleError> {
        let mode = self.inner.borrow().mode.clone();

        // discard non-reference value
        let old_pos = jt_ref.get_handle();
        // discard null reference
        if old_pos == usize::MAX {
            return Ok(());
        }

        let (new_pos, new_mode) = match mode {
            MarkMode::NoCompact => (old_pos, mode),
            MarkMode::Compact(mut c) => {
                c.jt_top += 1;
                (c.jt_top - 1, MarkMode::Compact(c))
            }
        };

        // update the live set
        let mut inner = self.inner.borrow_mut();
        inner.live_set.redirect_jt(jt_ref, new_pos)?;
        inner.mode = new_mode;
        Ok(())
    }

    pub fn mark_block(&self, block_ref: BlockRef) -> Result<(), ModuleError> {
        self.mark_value(ValueSSA::Block(block_ref))
    }
    pub fn mark_inst(&self, inst_ref: InstRef) -> Result<(), ModuleError> {
        self.mark_value(ValueSSA::Inst(inst_ref))
    }
    pub fn mark_global(&self, global_ref: GlobalRef) -> Result<(), ModuleError> {
        self.mark_value(ValueSSA::Global(global_ref))
    }
    pub fn mark_expr(&self, expr_ref: ConstExprRef) -> Result<(), ModuleError> {
        self.mark_value(ValueSSA::ConstExpr(expr_ref))
    }

    pub fn release_live_set(self) -> IRRefLiveSet {
        let inner = self.inner.into_inner();
        inner.live_set
    }

    fn mark_value_by_semantic(&self, value: ValueSSA) -> Result<(), ModuleError> {
        match value {
            ValueSSA::Block(block_ref) => self.mark_block(block_ref),
            ValueSSA::Inst(inst_ref) => self.mark_inst(inst_ref),
            ValueSSA::Global(global_ref) => self.mark_global(global_ref),
            ValueSSA::ConstExpr(expr_ref) => {
                self.mark_expr(expr_ref)?;
                let alloc_value = self.module.borrow_value_alloc();
                let alloc_expr = &alloc_value.alloc_expr;
                self.expr_visitor_dispatch(expr_ref, alloc_expr);
                Ok(())
            }
            _ => Ok(()),
        }
    }
}

impl<'a> IValueVisitor for MarkVisitor<'a> {
    fn read_block(&self, block: BlockRef, block_data: &BlockData) {
        // Mark the elements in the block
        let insts = unsafe { block_data.instructions.unsafe_load_readonly_view() };

        let alloc_value = self.module.borrow_value_alloc();
        let alloc_inst = &alloc_value.alloc_inst;
        let mut node = insts._head;
        while node.is_nonnull() {
            self.mark_inst(node).unwrap();

            let inst = node.to_slabref_unwrap(alloc_inst);
            self.inst_visitor_dispatch(node, inst);

            node = match node.get_next_ref(alloc_inst) {
                Some(n) => n,
                None => break,
            };
        }
    }

    fn read_func_arg(&self, _: GlobalRef, _: u32) {}
}

impl<'a> IConstDataVisitor for MarkVisitor<'a> {
    fn read_int_const(&self, _: u8, _: i128) {}
    fn read_float_const(&self, _: FloatTypeKind, _: f64) {}
    fn read_ptr_null(&self, _: ValTypeID) {}
    fn read_undef(&self, _: ValTypeID) {}
    fn read_zero(&self, _: ValTypeID) {}
    fn const_data_visitor_dispatch(&self, _: &ConstData) {}
}

impl<'a> IConstExprVisitor for MarkVisitor<'a> {
    fn read_array(&self, _: ConstExprRef, array_data: &Array) {
        let alloc_value = self.module.borrow_value_alloc();
        let alloc_expr = &alloc_value.alloc_expr;

        for i in &array_data.elems {
            self.mark_value(i.clone()).unwrap();
            match i {
                ValueSSA::ConstExpr(expr_ref) => {
                    self.expr_visitor_dispatch(*expr_ref, alloc_expr);
                }
                _ => {}
            }
        }
    }
    fn read_struct(&self, _: ConstExprRef, struct_data: &Struct) {
        let alloc_value = self.module.borrow_value_alloc();
        let alloc_expr = &alloc_value.alloc_expr;

        for i in &struct_data.elems {
            self.mark_value(i.clone()).unwrap();
            match i {
                ValueSSA::ConstExpr(expr_ref) => {
                    self.expr_visitor_dispatch(*expr_ref, alloc_expr);
                }
                _ => {}
            }
        }
    }
}

impl<'a> IGlobalObjectVisitor for MarkVisitor<'a> {
    fn read_global_variable(&self, global_ref: GlobalRef, gvar: &Var) {
        self.mark_value(ValueSSA::Global(global_ref)).unwrap();
        if let Some(init) = gvar.get_init() {
            self.mark_value_by_semantic(init).unwrap();
        }
    }

    fn read_global_alias(&self, global_ref: GlobalRef, galias: &Alias) {
        self.mark_value(ValueSSA::Global(global_ref)).unwrap();
        let aliasee = galias.target.get();
        if aliasee.is_nonnull()
            && !self
                .inner
                .borrow()
                .live_set
                .value_is_live(ValueSSA::Global(aliasee))
                .unwrap()
        {
            self.mark_value_by_semantic(ValueSSA::Global(aliasee)).unwrap();
        }
    }

    fn read_func(&self, global_ref: GlobalRef, gfunc: &FuncData) {
        todo!()
    }
}

impl<'a> IInstVisitor for MarkVisitor<'a> {
    fn read_phi_end(&self, inst_ref: InstRef) {
        todo!()
    }

    fn read_phi_inst(&self, inst_ref: InstRef, common: &InstDataCommon, phi: &phi::PhiOp) {
        todo!()
    }

    fn read_unreachable_inst(&self, inst_ref: InstRef, common: &InstDataCommon) {
        todo!()
    }

    fn read_ret_inst(&self, inst_ref: InstRef, common: &InstDataCommon, ret: &terminator::Ret) {
        todo!()
    }

    fn read_jump_inst(&self, inst_ref: InstRef, common: &InstDataCommon, jump: &terminator::Jump) {
        todo!()
    }

    fn read_br_inst(&self, inst_ref: InstRef, common: &InstDataCommon, br: &terminator::Br) {
        todo!()
    }

    fn read_switch_inst(
        &self,
        inst_ref: InstRef,
        common: &InstDataCommon,
        switch: &terminator::Switch,
    ) {
        todo!()
    }

    fn read_tail_call_inst(&self, inst_ref: InstRef, common: &InstDataCommon) {
        todo!()
    }

    fn read_load_inst(
        &self,
        inst_ref: InstRef,
        common: &InstDataCommon,
        load: &load_store::LoadOp,
    ) {
        todo!()
    }

    fn read_store_inst(
        &self,
        inst_ref: InstRef,
        common: &InstDataCommon,
        store: &load_store::StoreOp,
    ) {
        todo!()
    }

    fn read_select_inst(
        &self,
        inst_ref: InstRef,
        common: &InstDataCommon,
        select: &sundury_inst::SelectOp,
    ) {
        todo!()
    }

    fn read_bin_op_inst(&self, inst_ref: InstRef, common: &InstDataCommon, bin_op: &BinOp) {
        todo!()
    }

    fn read_cmp_inst(&self, inst_ref: InstRef, common: &InstDataCommon, cmp: &cmp::CmpOp) {
        todo!()
    }

    fn read_cast_inst(&self, inst_ref: InstRef, common: &InstDataCommon, cast: &cast::CastOp) {
        todo!()
    }

    fn read_index_ptr_inst(
        &self,
        inst_ref: InstRef,
        common: &InstDataCommon,
        index_ptr: &gep::IndexPtrOp,
    ) {
        todo!()
    }

    fn read_call_inst(&self, inst_ref: InstRef, common: &InstDataCommon, call: &CallOp) {
        todo!()
    }
}

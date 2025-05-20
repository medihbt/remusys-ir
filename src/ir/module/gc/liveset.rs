use std::usize;

use crate::{
    base::{slabref::SlabRef, NullableValue},
    ir::{
        block::jump_target::JumpTargetRef, inst::usedef::UseRef, module::{Module, ModuleError}, ValueSSA
    },
};

pub struct IRRefLiveSet {
    pub blocks: Box<[usize]>,
    pub insts: Box<[usize]>,
    pub exprs: Box<[usize]>,
    pub globals: Box<[usize]>,
    pub uses: Box<[usize]>,
    pub jts: Box<[usize]>,
}

impl IRRefLiveSet {
    pub fn from_module_empty(module: &Module) -> Self {
        let alloc_value = module.borrow_value_alloc();
        let alloc_use = module.borrow_use_alloc();
        let alloc_jt = module.borrow_jt_alloc();
        let alloc_block = &alloc_value.alloc_block;
        let alloc_expr = &alloc_value.alloc_expr;
        let alloc_global = &alloc_value.alloc_global;
        let alloc_inst = &alloc_value.alloc_inst;

        Self::from_capacity(
            alloc_block.capacity(),
            alloc_inst.capacity(),
            alloc_expr.capacity(),
            alloc_global.capacity(),
            alloc_use.capacity(),
            alloc_jt.capacity(),
        )
    }

    pub fn from_capacity(
        block: usize,
        inst: usize,
        expr: usize,
        global: usize,
        uses: usize,
        jts: usize,
    ) -> Self {
        Self {
            blocks: vec![usize::MAX; block].into_boxed_slice(),
            insts: vec![usize::MAX; inst].into_boxed_slice(),
            exprs: vec![usize::MAX; expr].into_boxed_slice(),
            globals: vec![usize::MAX; global].into_boxed_slice(),
            uses: vec![usize::MAX; uses].into_boxed_slice(),
            jts: vec![usize::MAX; jts].into_boxed_slice(),
        }
    }

    fn _value_alloc_mut(&mut self, value: ValueSSA) -> Result<(&mut [usize], usize), ModuleError> {
        match value {
            ValueSSA::Block(block) => Ok((&mut self.blocks, block.get_handle())),
            ValueSSA::Inst(inst) => Ok((&mut self.insts, inst.get_handle())),
            ValueSSA::ConstExpr(expr) => Ok((&mut self.exprs, expr.get_handle())),
            ValueSSA::Global(global) => Ok((&mut self.globals, global.get_handle())),
            _ => Err(ModuleError::DfgOperandNotReferece(value)),
        }
    }
    fn _get_value_alloc(&self, value: ValueSSA) -> Result<(&[usize], usize), ModuleError> {
        match value {
            ValueSSA::Block(block) => Ok((&self.blocks, block.get_handle())),
            ValueSSA::Inst(inst) => Ok((&self.insts, inst.get_handle())),
            ValueSSA::ConstExpr(expr) => Ok((&self.exprs, expr.get_handle())),
            ValueSSA::Global(global) => Ok((&self.globals, global.get_handle())),
            _ => Err(ModuleError::DfgOperandNotReferece(value)),
        }
    }
    pub fn redirect_value(&mut self, value: ValueSSA, new_pos: usize) -> Result<(), ModuleError> {
        let (alloc, old_pos) = self._value_alloc_mut(value)?;
        if old_pos >= alloc.len() {
            return Err(ModuleError::DfgReferenceOutOfRange(old_pos, alloc.len()));
        }
        alloc[old_pos] = new_pos;
        Ok(())
    }
    pub fn redirect_use(&mut self, use_ref: UseRef, new_pos: usize) -> Result<(), ModuleError> {
        if use_ref.get_handle() >= self.uses.len() {
            return Err(ModuleError::DfgReferenceOutOfRange(
                use_ref.get_handle(),
                self.uses.len(),
            ));
        }
        self.uses[use_ref.get_handle()] = new_pos;
        Ok(())
    }
    pub fn redirect_jt(
        &mut self,
        jt_ref: JumpTargetRef,
        new_pos: usize,
    ) -> Result<(), ModuleError> {
        if jt_ref.get_handle() >= self.jts.len() {
            return Err(ModuleError::DfgReferenceOutOfRange(
                jt_ref.get_handle(),
                self.jts.len(),
            ));
        }
        self.jts[jt_ref.get_handle()] = new_pos;
        Ok(())
    }

    pub fn mark_value_live(&mut self, value: ValueSSA) -> Result<(), ModuleError> {
        self.redirect_value(
            value,
            match value {
                ValueSSA::Block(block) => block.get_handle(),
                ValueSSA::Inst(inst) => inst.get_handle(),
                ValueSSA::ConstExpr(expr) => expr.get_handle(),
                ValueSSA::Global(global) => global.get_handle(),
                _ => return Err(ModuleError::DfgOperandNotReferece(value)),
            },
        )
    }
    pub fn mark_use_live(&mut self, use_ref: UseRef) -> Result<(), ModuleError> {
        self.redirect_use(use_ref, use_ref.get_handle())
    }
    pub fn mark_jt_live(&mut self, jt_ref: JumpTargetRef) -> Result<(), ModuleError> {
        self.redirect_jt(jt_ref, jt_ref.get_handle())
    }

    pub fn get_value_new_pos(&self, value: ValueSSA) -> Result<usize, ModuleError> {
        let (alloc, old_pos) = self._get_value_alloc(value)?;
        if old_pos >= alloc.len() {
            return Err(ModuleError::DfgReferenceOutOfRange(old_pos, alloc.len()));
        }
        let new_pos = alloc[old_pos];
        if new_pos == usize::MAX {
            return Err(ModuleError::NullReference);
        }
        if new_pos >= alloc.len() {
            return Err(ModuleError::DfgReferenceOutOfRange(new_pos, alloc.len()));
        }
        Ok(new_pos)
    }
    pub fn value_is_live(&self, value: ValueSSA) -> Result<bool, ModuleError> {
        match self.get_value_new_pos(value) {
            Ok(new_pos) => Ok(new_pos != usize::MAX),
            Err(ModuleError::NullReference) => Ok(false),
            Err(e) => Err(e),
        }
    }

    pub fn get_use_new_pos(&self, use_ref: UseRef) -> Result<usize, ModuleError> {
        if use_ref.is_null() {
            return Err(ModuleError::NullReference);
        }
        if use_ref.get_handle() >= self.uses.len() {
            return Err(ModuleError::DfgReferenceOutOfRange(
                use_ref.get_handle(),
                self.uses.len(),
            ));
        }
        let new_pos = self.uses[use_ref.get_handle()];
        if new_pos == usize::MAX {
            return Err(ModuleError::NullReference);
        }
        if new_pos >= self.uses.len() {
            return Err(ModuleError::DfgReferenceOutOfRange(new_pos, self.uses.len()));
        }
        Ok(new_pos)
    }
    pub fn use_is_live(&self, use_ref: UseRef) -> Result<bool, ModuleError> {
        match self.get_use_new_pos(use_ref) {
            Ok(new_pos) => Ok(new_pos != usize::MAX),
            Err(ModuleError::NullReference) => Ok(false),
            Err(e) => Err(e),
        }
    }

    pub fn get_jt_new_pos(&self, jt_ref: JumpTargetRef) -> Result<usize, ModuleError> {
        if jt_ref.is_null() {
            return Err(ModuleError::NullReference);
        }
        if jt_ref.get_handle() >= self.jts.len() {
            return Err(ModuleError::DfgReferenceOutOfRange(
                jt_ref.get_handle(),
                self.jts.len(),
            ));
        }
        let new_pos = self.jts[jt_ref.get_handle()];
        if new_pos == usize::MAX {
            return Err(ModuleError::NullReference);
        }
        if new_pos >= self.jts.len() {
            return Err(ModuleError::DfgReferenceOutOfRange(new_pos, self.jts.len()));
        }
        Ok(new_pos)
    }
    pub fn jt_is_live(&self, jt_ref: JumpTargetRef) -> Result<bool, ModuleError> {
        match self.get_jt_new_pos(jt_ref) {
            Ok(new_pos) => Ok(new_pos != usize::MAX),
            Err(ModuleError::NullReference) => Ok(false),
            Err(e) => Err(e),
        }
    }
}

use std::cell::RefCell;

use slab::Slab;

use crate::{
    base::{INullableValue, SlabListRange, SlabRef, SlabRefList},
    ir::{
        ValueSSA,
        global::GlobalRef,
        inst::{InstRef, UseData, UseRef},
    },
    typing::{context::TypeContext, types::FuncTypeRef},
};

use super::ModuleError;

#[derive(Debug, Clone)]
pub struct RdfgPerValue {
    pub valueref: ValueSSA,
    pub uses: RefCell<Vec<UseRef>>,
}

impl RdfgPerValue {
    pub fn new(valueref: ValueSSA) -> Self {
        Self { valueref, uses: RefCell::new(Vec::new()) }
    }

    pub fn is_null(&self) -> bool {
        self.valueref.is_null()
    }
    pub fn new_null() -> Self {
        Self::new(ValueSSA::new_null())
    }

    pub fn add_user_use(&self, user: UseRef) {
        let mut uses = self.uses.borrow_mut();
        if !uses.contains(&user) {
            uses.push(user);
        }
    }
    pub fn remove_user_use(&self, user: UseRef) {
        let mut uses = self.uses.borrow_mut();
        if let Some(pos) = uses.iter().position(|u| *u == user) {
            uses.remove(pos);
        }
    }
    pub fn n_users_use(&self) -> usize {
        self.uses.borrow().len()
    }
    pub fn has_user(&self) -> bool {
        self.n_users_use() > 0
    }

    pub fn collect_users(&self, alloc_use: &Slab<UseData>) -> Vec<InstRef> {
        let mut users = Vec::new();
        for user in self.uses.borrow().iter() {
            users.push(user.get_user(alloc_use));
        }
        users.sort_unstable();
        users.dedup();
        users
    }
}

#[derive(Debug, Clone)]
pub struct FuncArgRdfg {
    pub func_ref: GlobalRef,
    pub arg_rdfg: Option<Box<[RdfgPerValue]>>,
}

impl FuncArgRdfg {
    pub fn new(func_ref: GlobalRef, nargs: u32) -> Self {
        let mut arg_rdfg = Vec::with_capacity(nargs as usize);
        for i in 0..nargs {
            arg_rdfg.push(RdfgPerValue::new(ValueSSA::FuncArg(func_ref, i)));
        }
        Self { func_ref, arg_rdfg: Some(arg_rdfg.into_boxed_slice()) }
    }
    pub fn new_null() -> Self {
        Self { func_ref: GlobalRef::new_null(), arg_rdfg: None }
    }
    pub fn is_null(&self) -> bool {
        self.func_ref.is_null()
    }

    pub fn get_arg(&self, arg: usize) -> Option<&RdfgPerValue> {
        self.arg_rdfg.as_ref().map(|args| &args[arg])
    }
}

pub struct RdfgAlloc {
    pub global: Vec<RdfgPerValue>,
    pub expr: Vec<RdfgPerValue>,
    pub inst: Vec<RdfgPerValue>,
    pub block: Vec<RdfgPerValue>,
    pub func_arg: Vec<FuncArgRdfg>,
}

impl RdfgAlloc {
    pub fn new_with_capacity(global: usize, expr: usize, inst: usize, block: usize) -> Self {
        Self {
            global: vec![RdfgPerValue::new_null(); global],
            expr: vec![RdfgPerValue::new_null(); expr],
            inst: vec![RdfgPerValue::new_null(); inst],
            block: vec![RdfgPerValue::new_null(); block],
            func_arg: vec![FuncArgRdfg::new_null(); global],
        }
    }

    /// Will not insert a new node if the node is already allocated.
    fn _alloc_node_for_reference(alloc: &mut Vec<RdfgPerValue>, handle: usize, value: ValueSSA) {
        if alloc.len() <= handle {
            alloc.resize(handle + 1, RdfgPerValue::new_null());
        }
        let node = &mut alloc[handle];
        if node.is_null() {
            *node = RdfgPerValue::new(value);
        }
    }
    /// Will not insert a new node if the node is already allocated.
    fn _alloc_arg_nodes_for_func(alloc: &mut Vec<FuncArgRdfg>, func_ref: GlobalRef, nargs: usize) {
        let handle = func_ref.get_handle();
        if alloc.len() <= handle {
            alloc.resize(handle + 1, FuncArgRdfg::new_null());
        }
        let node = &mut alloc[handle];
        if node.is_null() {
            *node = FuncArgRdfg::new(func_ref, nargs as u32);
        }
    }
    /// Will not insert a new node if the node is already allocated.
    pub fn alloc_node(
        &mut self,
        operand: ValueSSA,
        maybe_func: Option<FuncTypeRef>,
        type_ctx: &TypeContext,
    ) -> Result<(), ModuleError> {
        let (alloc, handle) = match operand {
            ValueSSA::Global(global) => {
                Self::_alloc_node_for_reference(&mut self.global, global.get_handle(), operand);
                if let Some(func) = maybe_func {
                    let nargs = func.get_nargs(type_ctx);
                    Self::_alloc_arg_nodes_for_func(&mut self.func_arg, global, nargs);
                }
                return Ok(());
            }
            ValueSSA::ConstExpr(expr) => (&mut self.expr, expr.get_handle()),
            ValueSSA::Inst(inst) => (&mut self.inst, inst.get_handle()),
            ValueSSA::Block(block) => (&mut self.block, block.get_handle()),
            _ /* Value semantoc items should not insert */ => {
                return Err(ModuleError::DfgOperandNotReferece(operand));
            }
        };
        Self::_alloc_node_for_reference(alloc, handle, operand);
        Ok(())
    }

    pub fn free_node(&mut self, value: ValueSSA) -> Result<(), ModuleError> {
        match value {
            ValueSSA::Global(global) => {
                self.global[global.get_handle()] = RdfgPerValue::new_null();
                if self.func_arg.len() > global.get_handle() {
                    return Ok(());
                }
                let func_arg_rdfg = &mut self.func_arg[global.get_handle()];
                if func_arg_rdfg.is_null() {
                    return Ok(());
                }
                *func_arg_rdfg = FuncArgRdfg::new_null();
            }
            ValueSSA::ConstExpr(expr) => {
                self.expr[expr.get_handle()] = RdfgPerValue::new_null();
            }
            ValueSSA::Inst(inst) => {
                self.inst[inst.get_handle()] = RdfgPerValue::new_null();
            }
            ValueSSA::Block(block) => {
                self.block[block.get_handle()] = RdfgPerValue::new_null();
            }
            _ /* Value semantoc items should not insert */ => {
                return Err(ModuleError::DfgOperandNotReferece(value));
            }
        }
        Ok(())
    }

    pub fn get_node(&self, value: ValueSSA) -> Result<&RdfgPerValue, ModuleError> {
        match value {
            ValueSSA::Global(global) => Ok(&self.global[global.get_handle()]),
            ValueSSA::ConstExpr(expr) => Ok(&self.expr[expr.get_handle()]),
            ValueSSA::Inst(inst) => Ok(&self.inst[inst.get_handle()]),
            ValueSSA::Block(block) => Ok(&self.block[block.get_handle()]),
            ValueSSA::FuncArg(func_ref, index) => {
                let func_ref = func_ref.get_handle();
                let func_arg_rdfg = &self.func_arg[func_ref];
                Ok(func_arg_rdfg.get_arg(index as usize).unwrap())
            }
            _ /* Value semantoc items should not insert */ => {
                Err(ModuleError::DfgOperandNotReferece(value))
            },
        }
    }

    pub fn insert_new_inst_view(
        &mut self,
        inst: InstRef,
        operand_view: &SlabRefList<UseRef>,
        alloc_use: &Slab<UseData>,
        type_ctx: &TypeContext,
    ) -> Result<(), ModuleError> {
        self.alloc_node(ValueSSA::Inst(inst), None, type_ctx)?;

        for (useref, usedata) in operand_view.view(alloc_use) {
            let operand = usedata.get_operand();
            match self.get_node(operand) {
                Ok(node) => node.add_user_use(useref),
                Err(ModuleError::DfgOperandNotReferece(_)) => { /* ignore */ }
                Err(x) => Err(x).unwrap(),
            };
        }
        Ok(())
    }
    pub fn insert_new_inst_range(
        &mut self,
        inst: InstRef,
        operand_range: SlabListRange<UseRef>,
        alloc_use: &Slab<UseData>,
        type_ctx: &TypeContext,
    ) -> Result<(), ModuleError> {
        self.alloc_node(ValueSSA::Inst(inst), None, type_ctx)?;

        for (useref, usedata) in operand_range.view(alloc_use) {
            let operand = usedata.get_operand();
            match self.get_node(operand) {
                Ok(node) => node.add_user_use(useref),
                Err(ModuleError::DfgOperandNotReferece(_)) => { /* ignore */ }
                Err(x) => Err(x).unwrap(),
            };
        }
        Ok(())
    }
}

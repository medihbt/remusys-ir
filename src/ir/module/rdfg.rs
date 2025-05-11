use std::cell::RefCell;

use slab::Slab;

use crate::{
    base::slabref::SlabRef,
    ir::{
        ValueSSA,
        global::GlobalRef,
        inst::{
            InstData, InstRef,
            usedef::{UseData, UseRef},
        },
    },
    typing::{context::TypeContext, id::ValTypeID},
};

use super::ModuleAllocErr;

type ValueRDFGNode = Option<Vec<UseRef>>;
type ReverseDFGAllocVec = Vec<RefCell<ValueRDFGNode>>;

#[derive(Clone)]
pub(super) struct ArgRDFGNodesPerFunc {
    arg_rdfg_nodes: Box<[RefCell<Vec<UseRef>>]>,
}

pub struct RDFGAllocs {
    pub(super) _alloc_global: ReverseDFGAllocVec,
    pub(super) _alloc_expr: ReverseDFGAllocVec,
    pub(super) _alloc_inst: ReverseDFGAllocVec,
    pub(super) _alloc_block: ReverseDFGAllocVec,
    pub(super) _alloc_func_arg: Vec<RefCell<Option<ArgRDFGNodesPerFunc>>>,
}

impl RDFGAllocs {
    fn alloc_node_for_referenced_value(
        &mut self,
        operand: ValueSSA,
    ) -> Result<(), super::ModuleAllocErr> {
        let (handle, allocator) = match operand {
            ValueSSA::Global(global) => (global.get_handle(), &mut self._alloc_global),
            ValueSSA::ConstExpr(expr) => (expr.get_handle(), &mut self._alloc_expr),
            ValueSSA::Inst(inst) => (inst.get_handle(), &mut self._alloc_inst),
            ValueSSA::Block(block) => (block.get_handle(), &mut self._alloc_block),
            _ /* Value semantoc items should not insert */ => panic!(
                "Module::_insert_new_referenced_operand: \
                operand {:?} is not a reference semantic value",
                operand
            ),
        };

        if allocator.len() <= handle {
            allocator.resize(handle + 1, RefCell::new(None));
        }
        let mut slot = allocator[handle].borrow_mut();
        if let None = *slot {
            *slot = Some(Vec::new());
        }
        Ok(())
    }

    fn alloc_node_for_funcarg(&mut self, func_ref: GlobalRef, nargs: usize) {
        let handle = func_ref.get_handle();
        if self._alloc_func_arg.len() <= handle {
            self._alloc_func_arg.resize(handle + 1, RefCell::new(None));
        }
        if let Some(_) = *self._alloc_func_arg[handle].borrow() {
            return;
        }
        *self._alloc_func_arg[handle].borrow_mut() = Some(ArgRDFGNodesPerFunc {
            arg_rdfg_nodes: vec![RefCell::new(Vec::new()); nargs].into_boxed_slice(),
        });
    }

    fn _alloc_node(
        &mut self,
        operand: ValueSSA,
        maybe_func: Option<ValTypeID>,
        type_ctx: &TypeContext,
    ) -> Result<(), super::ModuleAllocErr> {
        match operand {
            ValueSSA::ConstExpr(_) | ValueSSA::Inst(_) | ValueSSA::Block(_) => {
                self.alloc_node_for_referenced_value(operand)
            }
            ValueSSA::Global(g) => {
                self.alloc_node_for_referenced_value(operand)?;

                match maybe_func {
                    Some(ValTypeID::Func(func)) => {
                        let nargs = func.get_nargs(type_ctx);
                        self.alloc_node_for_funcarg(g, nargs);
                    }
                    Some(_) => panic!("Type mismatch: requires Func but got {:?}", maybe_func),
                    None => {}
                }
                Ok(())
            }
            ValueSSA::FuncArg(..) => {
                panic!(
                    "Func argument RDFG nodes should be allocated \
                        the same time when function is inserted into the module!"
                );
            }
            _ => Err(ModuleAllocErr::DfgOperandNotReferece(operand)),
        }
    }

    /// Insert an instruction into the reverse DFG.
    fn _insert_inst(
        &mut self,
        inst: InstRef,
        inst_alloc: &Slab<InstData>,
        use_alloc: &Slab<UseData>,
    ) -> Result<(), ModuleAllocErr> {
        self.alloc_node_for_referenced_value(ValueSSA::Inst(inst))?;
        let inst_ref = inst.to_slabref_unwrap(inst_alloc);

        let opreand_list = match inst_ref.get_common() {
            Some(commmon) => &commmon.operands,
            None => return Ok(()),
        };

        for useref in opreand_list.view(use_alloc) {
            let operand = useref.get_operand(use_alloc);
            if operand.is_none() {
                continue;
            }
            self.edit_node(operand, |u| add_value_for_vecset(u, useref))?;
        }

        Ok(())
    }

    /// Insert a new node into the reverse DFG.
    /// If this node is an instruction, it will also modify the operands of the instruction
    /// to complete the reverse DFG.
    pub(super) fn insert_node(
        &mut self,
        operand: ValueSSA,
        maybe_func: Option<ValTypeID>,
        type_ctx: &TypeContext,
        inst_alloc: &Slab<InstData>,
        use_alloc: &Slab<UseData>,
    ) -> Result<(), ModuleAllocErr> {
        match operand {
            ValueSSA::Inst(inst) => self._insert_inst(inst, inst_alloc, use_alloc),
            _ => self._alloc_node(operand, maybe_func, type_ctx),
        }
    }

    pub(super) fn edit_node(
        &self,
        operand: ValueSSA,
        editor: impl FnOnce(&mut Vec<UseRef>),
    ) -> Result<(), ModuleAllocErr> {
        let (handle, alloc) = match operand {
            ValueSSA::Global(global) => (global.get_handle(), &self._alloc_global),
            ValueSSA::ConstExpr(expr) => (expr.get_handle(), &self._alloc_expr),
            ValueSSA::Inst(inst) => (inst.get_handle(), &self._alloc_inst),
            ValueSSA::Block(block) => (block.get_handle(), &self._alloc_block),
            ValueSSA::FuncArg(func_ref, index) => {
                let func_handle = func_ref.get_handle();
                let func_arg_rdfg = &self._alloc_func_arg[func_handle].borrow();
                let func_arg_rdfg = match func_arg_rdfg.as_ref() {
                    Some(func_arg_rdfg) => func_arg_rdfg,
                    None => return Err(ModuleAllocErr::FuncArgRefBroken(func_ref, index)),
                };
                if func_arg_rdfg.arg_rdfg_nodes.len() <= index as usize {
                    panic!(
                        "Operand overflow: {} > {}",
                        index,
                        func_arg_rdfg.arg_rdfg_nodes.len()
                    );
                }
                let arg_rdfg = &func_arg_rdfg.arg_rdfg_nodes[index as usize];
                editor(&mut *arg_rdfg.borrow_mut());
                return Ok(());
            },
            _ /* Value semantoc items should not insert */ => {
                return Err(ModuleAllocErr::DfgOperandNotReferece(operand))
            }
        };

        if alloc.len() <= handle {
            return Err(ModuleAllocErr::OperandOverflow(alloc.len(), handle));
        }

        let mut node = alloc[handle].borrow_mut();
        match node.as_mut() {
            Some(node) => {
                editor(node);
                Ok(())
            }
            None => Err(ModuleAllocErr::DfgReverseTrackingNotEnabled),
        }
    }
}

impl RDFGAllocs {
    pub fn new_with_capacity(global: usize, expr: usize, inst: usize, block: usize) -> Self {
        Self {
            _alloc_global: vec![RefCell::new(None); global],
            _alloc_expr: vec![RefCell::new(None); expr],
            _alloc_inst: vec![RefCell::new(None); inst],
            _alloc_block: vec![RefCell::new(None); block],

            // Func argument RDFG nodes are allocated when the function is inserted
            _alloc_func_arg: vec![RefCell::new(None); global],
        }
    }
}

fn add_value_for_vecset<T: PartialEq + Clone>(vec: &mut Vec<T>, value: T) {
    if vec.iter().find(|v| **v == value).is_none() {
        vec.push(value);
    }
}
#[allow(dead_code)]
fn remove_value_for_vecset<T: PartialEq>(vec: &mut Vec<T>, value: T) {
    if let Some(pos) = vec.iter().position(|v| *v == value) {
        vec.remove(pos);
    }
}

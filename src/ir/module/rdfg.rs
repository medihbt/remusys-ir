use std::cell::RefCell;

use crate::{
    base::slabref::SlabRef,
    ir::{ValueSSA, global::GlobalRef, inst::usedef::UseRef},
    typing::{context::TypeContext, id::ValTypeID},
};

use super::ModuleAllocErr;

type ValueRDFGNode = Option<Vec<UseRef>>;
type ReverseDFGAllocVec = Vec<RefCell<ValueRDFGNode>>;

#[derive(Clone)]
struct ArgRDFGNodesPerFunc {
    arg_rdfg_nodes: Box<[RefCell<Vec<UseRef>>]>,
}

pub struct RDFGAllocs {
    _alloc_global: ReverseDFGAllocVec,
    _alloc_expr: ReverseDFGAllocVec,
    _alloc_inst: ReverseDFGAllocVec,
    _alloc_block: ReverseDFGAllocVec,
    _alloc_func_arg: Vec<RefCell<Option<ArgRDFGNodesPerFunc>>>,
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

    pub(super) fn alloc_node(
        &mut self,
        operand: ValueSSA,
        maybe_functy: ValTypeID,
        is_function: bool,
        type_ctx: &TypeContext,
    ) -> Result<(), super::ModuleAllocErr> {
        match operand {
            ValueSSA::ConstExpr(_) | ValueSSA::Inst(_) | ValueSSA::Block(_) => {
                self.alloc_node_for_referenced_value(operand)
            }
            ValueSSA::Global(g) => {
                self.alloc_node_for_referenced_value(operand)?;

                if is_function {
                    let nargs = match maybe_functy {
                        ValTypeID::Func(f) => f.get_nargs(type_ctx),
                        _ => panic!("Type mismatch: requires Func but got {:?}", maybe_functy),
                    };
                    self.alloc_node_for_funcarg(g, nargs);
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

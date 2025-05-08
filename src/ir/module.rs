use core::panic;
use std::{
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    rc::Rc,
};

use slab::Slab;

use crate::{
    base::slabref::SlabRef,
    typing::{context::TypeContext, id::ValTypeID},
};

use super::{
    ValueSSA,
    block::{
        BlockData, BlockRef,
        jump_target::{JumpTargetData, JumpTargetRef},
    },
    constant::expr::{ConstExprData, ConstExprRef},
    global::{GlobalData, GlobalRef},
    inst::{
        InstData, InstRef,
        usedef::{UseData, UseRef},
    },
};

pub struct Module {
    pub name: String,
    pub type_ctx: Rc<TypeContext>,
    pub global_defs: RefCell<HashMap<String, GlobalRef>>,
    pub(super) _alloc_value: RefCell<ModuleAllocatorInner>,
    pub(super) _alloc_use: RefCell<Slab<UseData>>,
    pub(super) _alloc_jt: RefCell<Slab<JumpTargetData>>,
    pub(super) _alloc_reverse_dfg: RefCell<Option<ReverseDFGAllocs>>,
}

pub struct ModuleAllocatorInner {
    pub(super) _alloc_global: Slab<GlobalData>,
    pub(super) _alloc_expr: Slab<ConstExprData>,
    pub(super) _alloc_inst: Slab<InstData>,
    pub(super) _alloc_block: Slab<BlockData>,
}

type ValueRDFGNode = Option<Vec<UseRef>>;
type ReverseDFGAllocVec = Vec<RefCell<ValueRDFGNode>>;

#[derive(Clone)]
struct RDFGNodePerFunc {
    func_ref: GlobalRef,
    arg_rdfg_nodes: Box<[RefCell<Vec<UseRef>>]>,
}

pub struct ReverseDFGAllocs {
    _alloc_global: ReverseDFGAllocVec,
    _alloc_expr: ReverseDFGAllocVec,
    _alloc_inst: ReverseDFGAllocVec,
    _alloc_block: ReverseDFGAllocVec,
    _alloc_func_arg: Vec<RefCell<Option<RDFGNodePerFunc>>>,
}

impl Module {
    pub fn new(name: String, type_ctx: Rc<TypeContext>) -> Self {
        let inner = ModuleAllocatorInner {
            _alloc_global: Slab::with_capacity(32),
            _alloc_expr: Slab::with_capacity(4096),
            _alloc_inst: Slab::with_capacity(1024),
            _alloc_block: Slab::with_capacity(512),
        };
        Self {
            name,
            type_ctx,
            global_defs: RefCell::new(HashMap::new()),
            _alloc_value: RefCell::new(inner),
            _alloc_use: RefCell::new(Slab::with_capacity(4096)),
            _alloc_jt: RefCell::new(Slab::with_capacity(1024)),
            _alloc_reverse_dfg: RefCell::new(None),
        }
    }

    pub fn borrow_value_alloc<'a>(&'a self) -> Ref<'a, ModuleAllocatorInner> {
        self._alloc_value.borrow()
    }
    pub fn borrow_value_alloc_mut<'a>(&'a self) -> RefMut<'a, ModuleAllocatorInner> {
        self._alloc_value.borrow_mut()
    }

    pub fn borrow_use_alloc<'a>(&'a self) -> Ref<'a, Slab<UseData>> {
        self._alloc_use.borrow()
    }
    pub fn borrow_use_alloc_mut<'a>(&'a self) -> RefMut<'a, Slab<UseData>> {
        self._alloc_use.borrow_mut()
    }

    pub fn borrow_jt_alloc<'a>(&'a self) -> Ref<'a, Slab<JumpTargetData>> {
        self._alloc_jt.borrow()
    }
    pub fn borrow_jt_alloc_mut<'a>(&'a self) -> RefMut<'a, Slab<JumpTargetData>> {
        self._alloc_jt.borrow_mut()
    }
}

/// Adding and removing allocated items.
/// well... removing allocated items is prohibited; you can only use GC to remove them.
impl Module {
    pub fn get_global(&self, global: GlobalRef) -> Ref<GlobalData> {
        let inner = self.borrow_value_alloc();
        Ref::map(inner, |inner| {
            inner._alloc_global.get(global.get_handle()).unwrap()
        })
    }
    pub fn mut_global(&self, global: GlobalRef) -> RefMut<GlobalData> {
        let inner = self.borrow_value_alloc_mut();
        RefMut::map(inner, |inner| {
            inner._alloc_global.get_mut(global.get_handle()).unwrap()
        })
    }
    pub fn insert_global(&self, data: GlobalData) -> GlobalRef {
        let name = data.get_common().name.clone();
        let pointee_type = data.get_common().content_ty;
        let data_is_func = matches!(&data, GlobalData::Func(_));

        let ret = {
            let mut inner = self.borrow_value_alloc_mut();
            let id = inner._alloc_global.insert(data);
            GlobalRef::from_handle(id)
        };
        /* Try add this handle as operand. */
        self._insert_new_referenced_operand(ValueSSA::Global(ret.clone()));

        /* If this is a function, insert a new argument RDFG collection. */
        if data_is_func {
            let pointee_func_type = match pointee_type {
                ValTypeID::Func(functy_ref) => functy_ref,
                _ => panic!("Invalid type: requires function type"),
            };
            let nargs = pointee_func_type.get_nargs(&self.type_ctx);
            self._insert_funcarg_rdfgs(ret, nargs);
        }

        self.global_defs.borrow_mut().insert(name, ret);

        ret
    }

    pub fn get_expr(&self, expr: ConstExprRef) -> Ref<ConstExprData> {
        let inner = self.borrow_value_alloc();
        Ref::map(inner, |inner| {
            inner._alloc_expr.get(expr.get_handle()).unwrap()
        })
    }
    pub fn mut_expr(&self, expr: ConstExprRef) -> RefMut<ConstExprData> {
        let inner = self.borrow_value_alloc_mut();
        RefMut::map(inner, |inner| {
            inner._alloc_expr.get_mut(expr.get_handle()).unwrap()
        })
    }
    pub fn insert_expr(&self, data: ConstExprData) -> ConstExprRef {
        let ret = {
            let mut inner = self.borrow_value_alloc_mut();
            let id = inner._alloc_expr.insert(data);
            ConstExprRef::from_handle(id)
        };

        /* Try add this handle as operand. */
        self._insert_new_referenced_operand(ValueSSA::ConstExpr(ret.clone()));
        ret
    }

    pub fn get_inst(&self, inst: InstRef) -> Ref<InstData> {
        let inner = self.borrow_value_alloc();
        Ref::map(inner, |inner| {
            inner._alloc_inst.get(inst.get_handle()).unwrap()
        })
    }
    pub fn mut_inst(&self, inst: InstRef) -> RefMut<InstData> {
        let inner = self.borrow_value_alloc_mut();
        RefMut::map(inner, |inner| {
            inner._alloc_inst.get_mut(inst.get_handle()).unwrap()
        })
    }
    pub fn insert_inst(&self, data: InstData) -> InstRef {
        let mut inner = self.borrow_value_alloc_mut();
        let id = inner._alloc_inst.insert(data);
        let ret = InstRef::from_handle(id);

        /* Modify the slab reference to point to this */
        ret.to_slabref_unwrap_mut(&mut inner._alloc_inst)
            .common_mut()
            .map(|c| c.self_ref = ret.clone());

        /* Try add this handle as operand. */
        self._insert_new_referenced_operand(ValueSSA::Inst(ret.clone()));
        ret
    }

    pub fn get_block(&self, block: BlockRef) -> Ref<BlockData> {
        let inner = self.borrow_value_alloc();
        Ref::map(inner, |inner| {
            inner._alloc_block.get(block.get_handle()).unwrap()
        })
    }
    pub fn mut_block(&self, block: BlockRef) -> RefMut<BlockData> {
        let inner = self.borrow_value_alloc_mut();
        RefMut::map(inner, |inner| {
            inner._alloc_block.get_mut(block.get_handle()).unwrap()
        })
    }
    pub fn insert_block(&self, data: BlockData) -> BlockRef {
        let ret = {
            let mut inner = self.borrow_value_alloc_mut();
            let id = inner._alloc_block.insert(data);
            BlockRef::from_handle(id)
        };

        /* Try add this handle as operand. */
        self._insert_new_referenced_operand(ValueSSA::Block(ret.clone()));
        ret
    }

    pub fn get_use(&self, use_ref: UseRef) -> Ref<UseData> {
        let inner = self.borrow_use_alloc();
        Ref::map(inner, |inner| use_ref.to_slabref_unwrap(inner))
    }
    pub fn mut_use(&self, use_ref: UseRef) -> RefMut<UseData> {
        let inner = self.borrow_use_alloc_mut();
        RefMut::map(inner, |inner| use_ref.to_slabref_unwrap_mut(inner))
    }
    pub fn insert_use(&self, data: UseData) -> UseRef {
        let mut inner = self.borrow_use_alloc_mut();
        let id = inner.insert(data);
        UseRef::from_handle(id)
    }

    pub fn get_jt(&self, use_ref: JumpTargetRef) -> Ref<JumpTargetData> {
        let inner = self.borrow_jt_alloc();
        Ref::map(inner, |inner| use_ref.to_slabref_unwrap(inner))
    }
    pub fn mut_jt(&self, use_ref: JumpTargetRef) -> RefMut<JumpTargetData> {
        let inner = self.borrow_jt_alloc_mut();
        RefMut::map(inner, |inner| use_ref.to_slabref_unwrap_mut(inner))
    }
    pub fn insert_jt(&self, data: JumpTargetData) -> JumpTargetRef {
        let mut inner = self.borrow_jt_alloc_mut();
        let id = inner.insert(data);
        JumpTargetRef::from_handle(id)
    }

    /// Implement a 'mark-sweep' algorithm to reduce usage of those allocators.
    /// If the module owns its type context uniquely, it also collects garbages in
    /// its type context.
    ///
    /// This function cannot change the reference addresses of `Value`.
    pub fn gc_mark_sweep(&self, _external_live_set: impl Iterator<Item = ValueSSA>) {
        todo!()
    }
}

pub enum ModuleAllocErr {
    DfgOperandNotReferece(ValueSSA),
    DfgReverseTrackingNotEnabled,
    OperandOverflow(usize /* requried */, usize /* real */),
    FuncArgRefBroken(GlobalRef, u32 /* index */),
}

/// Module as DFG reverse map.
impl Module {
    fn _insert_new_referenced_operand(&self, operand: ValueSSA) {
        let mut alloc_grd = self._alloc_reverse_dfg.borrow_mut();
        let alloc = match alloc_grd.as_mut() {
            Some(alloc) => alloc,
            None => {
                /* DFG tracking is not enabled. Just return. */
                return;
            }
        };

        let (handle, allocator) = match operand {
            ValueSSA::Global(global) => (global.get_handle(), &mut alloc._alloc_global),
            ValueSSA::ConstExpr(expr) => (expr.get_handle(), &mut alloc._alloc_expr),
            ValueSSA::Inst(inst) => (inst.get_handle(), &mut alloc._alloc_inst),
            ValueSSA::Block(block) => (block.get_handle(), &mut alloc._alloc_block),
            _ /* Value semantoc items should not insert */ => panic!(
                "Module::_insert_new_referenced_operand: \
                operand {:?} is not a reference semantic value",
                operand
            ),
        };

        if allocator.len() <= handle {
            allocator.resize(handle + 1, RefCell::new(None));
        }
        *allocator[handle].borrow_mut() = Some(Vec::new());
    }

    fn _insert_funcarg_rdfgs(&self, func_ref: GlobalRef, nargs: usize) {
        let handle = func_ref.get_handle();
        let mut alloc_grd = self._alloc_reverse_dfg.borrow_mut();
        let alloc = match alloc_grd.as_mut() {
            Some(alloc) => alloc,
            None => {
                /* DFG tracking is not enabled. Just return. */
                return;
            }
        };

        if alloc._alloc_func_arg.len() <= handle {
            alloc._alloc_func_arg.resize(handle + 1, RefCell::new(None));
        }
        *alloc._alloc_func_arg[handle].borrow_mut() = Some(RDFGNodePerFunc {
            func_ref: func_ref.clone(),
            arg_rdfg_nodes: vec![RefCell::new(Vec::new()); nargs].into_boxed_slice(),
        });
    }

    /// Edit the reverse DFG node of the operand.
    ///
    /// NOTE: This is NOT a DFG node insertion or deletion, so mutable RDFG node allocator
    /// should NOT be mutable. Let RefCell of the final RDFG node work for its responsibility.
    fn _edit_rdfg_node(
        &self,
        operand: ValueSSA,
        editor: impl FnOnce(&mut Vec<UseRef>),
    ) -> Result<(), ModuleAllocErr> {
        // Check if DFG reverse-graph tracking is enabled.
        let rdfg_alloc = self._alloc_reverse_dfg.borrow();
        let rdfg_alloc = match &*rdfg_alloc {
            Some(alloc) => alloc,
            None => {
                /* DFG tracking is not enabled. Just return. */
                return Err(ModuleAllocErr::DfgReverseTrackingNotEnabled);
            }
        };

        // Get the handle and allocator for the operand.
        // If the operand is a function argument, use a different control flow to get the RDFG node.
        // Otherwise, use the handle to get the RDFG node from the allocator.
        let (handle, rdfg_alloc) = match operand {
            ValueSSA::Global(global) => (global.get_handle(), &rdfg_alloc._alloc_global),
            ValueSSA::ConstExpr(expr) => (expr.get_handle(), &rdfg_alloc._alloc_expr),
            ValueSSA::Inst(inst) => (inst.get_handle(), &rdfg_alloc._alloc_inst),
            ValueSSA::Block(block) => (block.get_handle(), &rdfg_alloc._alloc_block),
            ValueSSA::FuncArg(func_ref, index) => {
                let func_handle = func_ref.get_handle();
                let func_arg_rdfg = &rdfg_alloc._alloc_func_arg[func_handle].borrow();
                let func_arg_rdfg = match func_arg_rdfg.as_ref() {
                    Some(func_arg_rdfg) => func_arg_rdfg,
                    None => {
                        return Err(ModuleAllocErr::FuncArgRefBroken(func_ref, index));
                    },
                };
                if func_arg_rdfg.arg_rdfg_nodes.len() <= index as usize {
                    panic!("Operand overflow: {} > {}", index, func_arg_rdfg.arg_rdfg_nodes.len());
                }
                let arg_rdfg = &func_arg_rdfg.arg_rdfg_nodes[index as usize];
                editor(&mut *arg_rdfg.borrow_mut());
                return Ok(());
            },
            _ /* Value semantoc items should not insert */ => {
                return Err(ModuleAllocErr::DfgOperandNotReferece(operand))
            }
        };

        if rdfg_alloc.len() <= handle {
            return Err(ModuleAllocErr::OperandOverflow(handle, rdfg_alloc.len()));
        }
        let mut rdfg_node = rdfg_alloc[handle].borrow_mut();
        let rdfg_node = match rdfg_node.as_mut() {
            Some(rdfg_node) => rdfg_node,
            None => {
                return Err(ModuleAllocErr::DfgReverseTrackingNotEnabled);
            }
        };
        editor(rdfg_node);
        Ok(())
    }

    /// Check if DFG reverse-graph tracking is enabled. `false` right after initialization.
    ///
    /// - To enable DFG tracking, call `self.enable_dfg_tracking()`.
    /// - To disable DFG tracking, call `self.disable_dfg_tracking()`.
    /// - To disable DFG tracking and take out all DFG reverse-graphs for the module,
    ///   call `self.steal_tracking_dfg()`.
    pub fn dfg_tracking_enabled(&self) -> bool {
        self._alloc_reverse_dfg.borrow().is_some()
    }

    /// Enable DFG reverse-graph tracking.
    ///
    /// This function will activate allocators for all types of values,
    /// traverse through control flow in the functions in this module,
    /// find all the operands of each instruction, and create a reverse
    /// mapping from the operands to the 'use' belonging to instructions
    /// who use them.
    pub fn enable_dfg_tracking(&self) -> Result<(), ModuleAllocErr> {
        todo!("Enable DFG tracking");
    }

    /// Disable DFG reverse-graph tracking.
    ///
    /// **WARNING**: This function will simply shut down all DFG reverse-graphs.
    /// Passes which depend on DFG reverse-graphs will be broken.
    pub fn disable_dfg_tracking(&self) {
        *self._alloc_reverse_dfg.borrow_mut() = None;
    }

    /// Disable DFG reverse-graph tracking and take out all DFG reverse-graphs
    /// for the module.
    pub fn steal_tracking_dfg(&self) -> Option<ReverseDFGAllocs> {
        self._alloc_reverse_dfg.borrow_mut().take()
    }

    pub fn operand_add_use(&self, operand: ValueSSA, useref: UseRef) -> Result<(), ModuleAllocErr> {
        self._edit_rdfg_node(operand, |v| v.push(useref))
    }

    pub fn operand_del_use(&self, operand: ValueSSA, useref: UseRef) -> Result<(), ModuleAllocErr> {
        self._edit_rdfg_node(operand, |v| {
            if let Some(pos) = v.iter().position(|x| *x == useref) {
                v.swap_remove(pos);
            } else {
                panic!("Cannot find use reference in operand");
            }
        })
    }
}

#[cfg(test)]
mod testing {

    use crate::{
        ir::{ValueSSA, constant::data::ConstData, global::GlobalData},
        typing::{context::PlatformPolicy, id::ValTypeID},
    };

    #[test]
    fn test_module() {
        use super::Module;
        use crate::typing::context::TypeContext;

        let type_ctx = TypeContext::new_rc(PlatformPolicy::new_host());
        let module = Module::new("test_module".to_string(), type_ctx.clone());
        assert_eq!(module.name, "test_module");

        // translate SysY source `int a = 0;` to IR: Create an integer global variable `a` and initialize it to 0.
        let global_data = GlobalData::new_variable(
            "a".to_string(),
            ValTypeID::Int(32),
            ValueSSA::ConstData(ConstData::Int(32, 0)),
        );

        module.insert_global(global_data);

        assert!(module.global_defs.borrow().contains_key("a"));
    }
}

use std::{
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    rc::Rc,
};

use rdfg::RDFGAllocs;
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
        terminator::TerminatorInst,
        usedef::{UseData, UseRef},
    },
};

pub mod rdfg;

pub struct Module {
    pub name: String,
    pub type_ctx: Rc<TypeContext>,
    pub global_defs: RefCell<HashMap<String, GlobalRef>>,
    pub(super) _alloc_value: RefCell<ModuleAllocatorInner>,
    pub(super) _alloc_use: RefCell<Slab<UseData>>,
    pub(super) _alloc_jt: RefCell<Slab<JumpTargetData>>,
    pub(super) _alloc_reverse_dfg: RefCell<Option<rdfg::RDFGAllocs>>,
}

pub struct ModuleAllocatorInner {
    pub(super) _alloc_global: Slab<GlobalData>,
    pub(super) _alloc_expr: Slab<ConstExprData>,
    pub(super) _alloc_inst: Slab<InstData>,
    pub(super) _alloc_block: Slab<BlockData>,
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

        // Modify the slab reference of its instructions to point to this.
        {
            let inner = self.borrow_value_alloc();
            let alloc_block = &inner._alloc_block;
            let alloc_global = &inner._alloc_global;

            ret.to_slabref_unwrap(alloc_global)
                ._init_set_self_reference(alloc_block, ret);
        }

        /* Try add this handle as operand. */
        self._rdfg_alloc_node(
            ValueSSA::Global(ret),
            if data_is_func {
                Some(pointee_type)
            } else {
                None
            },
        )
        .unwrap();

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
        self._rdfg_alloc_node(ValueSSA::ConstExpr(ret), None)
            .unwrap();
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

        // Modify the slab reference to point to this,
        ret.to_slabref_unwrap_mut(&mut inner._alloc_inst)
            ._inst_init_self_reference(ret, &self.borrow_use_alloc());

        // Modify the jump targets if this instruction is a terminator.
        let mut jt_alloc = self.borrow_jt_alloc_mut();
        match ret.to_slabref_unwrap(&inner._alloc_inst) {
            InstData::Jump(_, j) => j._jt_init_set_self_reference(ret, &mut jt_alloc),
            InstData::Br(_, br) => br._jt_init_set_self_reference(ret, &mut jt_alloc),
            InstData::Switch(_, s) => s._jt_init_set_self_reference(ret, &mut jt_alloc),
            _ => {}
        }
        // Try add this handle as operand.
        // If this instruction has operands, add them to the reverse graph.
        self._rdfg_alloc_node(ValueSSA::Inst(ret), None).unwrap();
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

        // Modify the slab reference of its instructions to point to this.
        // Now the `parent_bb` of the instructions will not be `null` anymore.
        let inner = self.borrow_value_alloc();
        ret.to_slabref_unwrap(&inner._alloc_block)
            .init_set_self_reference(ret, &inner._alloc_inst);

        /* Try add this handle as operand. */
        self._rdfg_alloc_node(ValueSSA::Block(ret), None).unwrap();
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
    pub fn gc_mark_sweep(&self, _extern_roots: impl Iterator<Item = ValueSSA>) {
        todo!()
    }

    /// Implement a 'mark-compact' algorithm to reduce usage of those allocators.
    /// If the module owns its type context uniquely, it also collects garbages in
    /// its type context.
    ///
    /// **WARNING**: This function WILL CHANGE the reference addresses of `Value`.
    ///
    /// ### Arguments
    ///
    /// - `extern_roots`: The roots of the module. This is used to mark the values
    ///   that are still in use.
    /// - `reserve_times`: The number of times to reserve the allocator. This is used
    ///   to reserve the allocators for the next allocation.
    pub fn gc_mark_compact(
        &self,
        _extern_roots: impl Iterator<Item = ValueSSA>,
        _reserve_times: f32,
    ) {
        todo!()
    }
}

#[derive(Debug)]
pub enum ModuleAllocErr {
    DfgOperandNotReferece(ValueSSA),
    DfgReverseTrackingNotEnabled,
    OperandOverflow(usize /* required */, usize /* real */),
    FuncArgRefBroken(GlobalRef, u32 /* index */),
}

/// Module as DFG reverse map.
impl Module {
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
        todo!();
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
    pub fn steal_tracking_dfg(&self) -> Option<rdfg::RDFGAllocs> {
        self._alloc_reverse_dfg.borrow_mut().take()
    }

    pub(crate) fn operand_add_use(
        &self,
        operand: ValueSSA,
        useref: UseRef,
    ) -> Result<(), ModuleAllocErr> {
        self._borrow_rdfg_alloc()?
            .edit_node(operand, |v| v.push(useref))
    }

    pub(crate) fn operand_del_use(
        &self,
        operand: ValueSSA,
        useref: UseRef,
    ) -> Result<(), ModuleAllocErr> {
        self._borrow_rdfg_alloc()?.edit_node(operand, |v| {
            if let Some(pos) = v.iter().position(|x| *x == useref) {
                v.swap_remove(pos);
            } else {
                panic!("Cannot find use reference in operand");
            }
        })
    }

    fn _borrow_rdfg_alloc(&self) -> Result<Ref<RDFGAllocs>, ModuleAllocErr> {
        let alloc_rdfg = self._alloc_reverse_dfg.borrow();
        match *alloc_rdfg {
            None => Err(ModuleAllocErr::DfgReverseTrackingNotEnabled),
            Some(_) => Ok(Ref::map(alloc_rdfg, |alloc| alloc.as_ref().unwrap())),
        }
    }
    fn _borrow_rdfg_alloc_mut(&self) -> Result<RefMut<RDFGAllocs>, ModuleAllocErr> {
        let alloc_rdfg = self._alloc_reverse_dfg.borrow_mut();
        match *alloc_rdfg {
            None => Err(ModuleAllocErr::DfgReverseTrackingNotEnabled),
            Some(_) => Ok(RefMut::map(alloc_rdfg, |alloc| alloc.as_mut().unwrap())),
        }
    }
    fn _rdfg_alloc_node(
        &self,
        operand: ValueSSA,
        maybe_func: Option<ValTypeID>,
    ) -> Result<(), ModuleAllocErr> {
        let mut alloc_rdfg = match self._borrow_rdfg_alloc_mut() {
            Ok(alloc) => alloc,
            Err(ModuleAllocErr::DfgReverseTrackingNotEnabled) => return Ok(()),
            Err(e) => return Err(e),
        };

        // Now RDFG is enabled, we can insert the node.
        alloc_rdfg.insert_node(
            operand,
            maybe_func,
            &self.type_ctx,
            &self.borrow_value_alloc()._alloc_inst,
            &self.borrow_use_alloc(),
        )
    }
}

/// Module as context maintainer.
impl Module {
    /// Perform a basic check on the module.
    pub fn perform_basic_check(&self) {
        let alloc_value = self.borrow_value_alloc();
        let alloc_global = &alloc_value._alloc_global;
        let alloc_block = &alloc_value._alloc_block;

        for (_, global) in alloc_global {
            let func_body = match global {
                GlobalData::Func(func) => match func.get_blocks() {
                    Some(body) => body,
                    None => continue,
                },
                _ => continue,
            };

            for block in func_body.view(alloc_block) {
                block
                    .to_slabref_unwrap(alloc_block)
                    .perform_basic_check(self);
            }
        }
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
        module.perform_basic_check();
    }
}

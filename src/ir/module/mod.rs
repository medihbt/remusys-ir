use std::{
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    rc::Rc,
};

use rdfg::RdfgAlloc;
use slab::Slab;

use crate::{
    base::{NullableValue, slabref::SlabRef},
    typing::{context::TypeContext, id::ValTypeID},
};

use super::{
    ValueSSA,
    block::{
        BlockData, BlockRef,
        jump_target::{JumpTargetData, JumpTargetRef},
    },
    constant::expr::{ConstExprData, ConstExprRef},
    global::{GlobalData, GlobalRef, func::FuncStorage},
    inst::{
        InstData, InstRef,
        terminator::TerminatorInst,
        usedef::{UseData, UseRef},
    },
};

pub mod gc;
pub mod rcfg;
pub mod rdfg;

pub struct Module {
    pub name: String,
    pub type_ctx: Rc<TypeContext>,
    pub global_defs: RefCell<HashMap<String, GlobalRef>>,
    pub(super) _alloc_value: RefCell<ModuleAllocatorInner>,
    pub(super) _alloc_use: RefCell<Slab<UseData>>,
    pub(super) _alloc_jt: RefCell<Slab<JumpTargetData>>,
    pub(super) _rdfg_alloc: RefCell<Option<rdfg::RdfgAlloc>>,
    pub(super) _rcfg_alloc: RefCell<Option<rcfg::RcfgAlloc>>,
}

pub struct ModuleAllocatorInner {
    pub(crate) alloc_global: Slab<GlobalData>,
    pub(crate) alloc_expr: Slab<ConstExprData>,
    pub(crate) alloc_inst: Slab<InstData>,
    pub(crate) alloc_block: Slab<BlockData>,
}

impl Module {
    pub fn new(name: String, type_ctx: Rc<TypeContext>) -> Self {
        let inner = ModuleAllocatorInner {
            alloc_global: Slab::with_capacity(32),
            alloc_expr: Slab::with_capacity(4096),
            alloc_inst: Slab::with_capacity(1024),
            alloc_block: Slab::with_capacity(512),
        };
        Self {
            name,
            type_ctx,
            global_defs: RefCell::new(HashMap::new()),
            _alloc_value: RefCell::new(inner),
            _alloc_use: RefCell::new(Slab::with_capacity(4096)),
            _alloc_jt: RefCell::new(Slab::with_capacity(1024)),
            _rdfg_alloc: RefCell::new(None),
            _rcfg_alloc: RefCell::new(None),
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
            inner.alloc_global.get(global.get_handle()).unwrap()
        })
    }
    pub fn mut_global(&self, global: GlobalRef) -> RefMut<GlobalData> {
        let inner = self.borrow_value_alloc_mut();
        RefMut::map(inner, |inner| {
            inner.alloc_global.get_mut(global.get_handle()).unwrap()
        })
    }
    pub fn insert_global(&self, data: GlobalData) -> GlobalRef {
        let name = data.get_common().name.clone();
        let pointee_type = data.get_common().content_ty;
        let data_is_func = matches!(&data, GlobalData::Func(_));

        let ret = {
            let mut inner = self.borrow_value_alloc_mut();
            let id = inner.alloc_global.insert(data);
            GlobalRef::from_handle(id)
        };

        // Modify the slab reference of its instructions to point to this.
        {
            let inner = self.borrow_value_alloc();
            let alloc_block = &inner.alloc_block;
            let alloc_global = &inner.alloc_global;

            ret.to_slabref_unwrap(alloc_global)
                ._init_set_self_reference(alloc_block, ret);
        }

        /* Try add this handle as operand. */
        let maybe_func = if data_is_func {
            match pointee_type {
                ValTypeID::Func(f) => Some(f),
                _ => panic!("Requires function type but got {:?}", pointee_type),
            }
        } else {
            None
        };
        // Try add this handle as operand.
        // If this is a function, also add its argument list to the reverse graph.
        if let Some(mut rdfg) = self.borrow_rdfg_alloc_mut() {
            rdfg.alloc_node(ValueSSA::Global(ret), maybe_func, &self.type_ctx)
                .unwrap();
        }
        // Add the global value to the name table.
        self.global_defs.borrow_mut().insert(name, ret);
        ret
    }

    pub fn get_expr(&self, expr: ConstExprRef) -> Ref<ConstExprData> {
        let inner = self.borrow_value_alloc();
        Ref::map(inner, |inner| {
            inner.alloc_expr.get(expr.get_handle()).unwrap()
        })
    }
    pub fn mut_expr(&self, expr: ConstExprRef) -> RefMut<ConstExprData> {
        let inner = self.borrow_value_alloc_mut();
        RefMut::map(inner, |inner| {
            inner.alloc_expr.get_mut(expr.get_handle()).unwrap()
        })
    }
    pub fn insert_expr(&self, data: ConstExprData) -> ConstExprRef {
        let ret = {
            let mut inner = self.borrow_value_alloc_mut();
            let id = inner.alloc_expr.insert(data);
            ConstExprRef::from_handle(id)
        };
        // Try add this handle as operand.
        if let Some(mut rdfg) = self.borrow_rdfg_alloc_mut() {
            rdfg.alloc_node(ValueSSA::ConstExpr(ret), None, &self.type_ctx)
                .unwrap();
        }
        ret
    }

    pub fn get_inst(&self, inst: InstRef) -> Ref<InstData> {
        let inner = self.borrow_value_alloc();
        Ref::map(inner, |inner| {
            inner.alloc_inst.get(inst.get_handle()).unwrap()
        })
    }
    pub fn mut_inst(&self, inst: InstRef) -> RefMut<InstData> {
        let inner = self.borrow_value_alloc_mut();
        RefMut::map(inner, |inner| {
            inner.alloc_inst.get_mut(inst.get_handle()).unwrap()
        })
    }
    pub fn insert_inst(&self, data: InstData) -> InstRef {
        let operand_view = unsafe { data.load_operand_view() };
        let jt_view = match data.as_terminator() {
            Some((_, t)) => unsafe {
                t.get_jump_targets()
                    .map(|jt| jt.unsafe_load_readonly_view())
            },
            None => None,
        };

        let ret = {
            let mut inner = self.borrow_value_alloc_mut();
            let id = inner.alloc_inst.insert(data);
            let ret = InstRef::from_handle(id);

            // Modify the slab reference to point to this,
            ret.to_slabref_unwrap_mut(&mut inner.alloc_inst)
                ._inst_init_self_reference(ret, &self.borrow_use_alloc());

            // Modify the jump targets if this instruction is a terminator.
            let mut alloc_jt = self.borrow_jt_alloc_mut();
            match ret.to_slabref_unwrap(&inner.alloc_inst) {
                InstData::Jump(_, j) => j._jt_init_set_self_reference(ret, &mut alloc_jt),
                InstData::Br(_, br) => br._jt_init_set_self_reference(ret, &mut alloc_jt),
                InstData::Switch(_, s) => s._jt_init_set_self_reference(ret, &mut alloc_jt),
                _ => {}
            }
            ret
        };

        // Try add this handle as operand.
        // If this instruction has operands, add them to the reverse graph.
        // self._rdfg_alloc_node(ValueSSA::Inst(ret), None).unwrap();
        if let Some(mut rdfg) = self.borrow_rdfg_alloc_mut() {
            let alloc_use = self.borrow_use_alloc();
            if let Some(operand_view) = operand_view {
                rdfg.insert_new_inst(ret, &operand_view, &alloc_use, &self.type_ctx)
                    .unwrap();
            } else {
                rdfg.alloc_node(ValueSSA::Inst(ret), None, &self.type_ctx)
                    .unwrap();
            }
        }

        // If this instruction is a terminator, add its jump targets to the reverse graph.
        if let (Some(jt_view), Some(rcfg)) = (jt_view, self.borrow_rcfg_alloc()) {
            let alloc_jt = self.borrow_jt_alloc();
            for (jt, jt_data) in jt_view.view(&alloc_jt) {
                let block = jt_data._block.get();
                if !block.is_null() {
                    rcfg.get_node(block).add_predecessor(jt);
                }
            }
        }

        ret
    }

    pub fn get_block(&self, block: BlockRef) -> Ref<BlockData> {
        let inner = self.borrow_value_alloc();
        Ref::map(inner, |inner| {
            inner.alloc_block.get(block.get_handle()).unwrap()
        })
    }
    pub fn mut_block(&self, block: BlockRef) -> RefMut<BlockData> {
        let inner = self.borrow_value_alloc_mut();
        RefMut::map(inner, |inner| {
            inner.alloc_block.get_mut(block.get_handle()).unwrap()
        })
    }
    pub fn insert_block(&self, data: BlockData) -> BlockRef {
        let ret = {
            let mut inner = self.borrow_value_alloc_mut();
            let id = inner.alloc_block.insert(data);
            BlockRef::from_handle(id)
        };

        // Modify the slab reference of its instructions to point to this.
        // Now the `parent_bb` of the instructions will not be `null` anymore.
        let inner = self.borrow_value_alloc();
        ret.to_slabref_unwrap(&inner.alloc_block)
            .init_set_self_reference(ret, &inner.alloc_inst);

        /* Try add this handle as operand. */
        if let Some(mut rdfg) = self.borrow_rdfg_alloc_mut() {
            rdfg.alloc_node(ValueSSA::Block(ret), None, &self.type_ctx)
                .unwrap();
        }
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
    pub fn gc_mark_sweep(&self, extern_roots: impl Iterator<Item = ValueSSA>) {
        gc::module_gc_mark_sweep(self, extern_roots).unwrap();
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
pub enum ModuleError {
    NullReference,

    DfgReferenceOutOfRange(usize, usize /* index */),
    DfgOperandNotReferece(ValueSSA),
    RDFGNotEnabled,
    OperandOverflow(usize /* required */, usize /* real */),
    FuncArgRefBroken(GlobalRef, u32 /* index */),

    RCFGNotEnabled,
    RCFGEnabled,
}

/// Module as DFG reverse map.
impl Module {
    /// Check if DFG reverse-graph tracking is enabled. `false` right after initialization.
    ///
    /// - To enable DFG tracking, call `self.enable_dfg_tracking()`.
    /// - To disable DFG tracking, call `self.disable_dfg_tracking()`.
    /// - To disable DFG tracking and take out all DFG reverse-graphs for the module,
    ///   call `self.steal_tracking_dfg()`.
    pub fn rdfg_enabled(&self) -> bool {
        self._rdfg_alloc.borrow().is_some()
    }

    /// Enable DFG reverse-graph tracking.
    ///
    /// This function will activate allocators for all types of values,
    /// traverse through control flow in the functions in this module,
    /// find all the operands of each instruction, and create a reverse
    /// mapping from the operands to the 'use' belonging to instructions
    /// who use them.
    pub fn enable_rdfg(&self) -> Result<(), ModuleError> {
        let type_ctx = self.type_ctx.as_ref();
        let self_alloc = self.borrow_value_alloc();
        let global_alloc = &self_alloc.alloc_global;
        let expr_alloc = &self_alloc.alloc_expr;
        let inst_alloc = &self_alloc.alloc_inst;
        let block_alloc = &self_alloc.alloc_block;
        let global_defs = &self.global_defs.borrow();
        let mut rdfg_alloc = RdfgAlloc::new_with_capacity(
            global_alloc.capacity(),
            expr_alloc.capacity(),
            inst_alloc.capacity(),
            block_alloc.capacity(),
        );

        // Step 1: Allocate nodes for all referenced values
        for (handle, _) in expr_alloc {
            let expr_ref = ConstExprRef::from_handle(handle);
            let expr_val = ValueSSA::ConstExpr(expr_ref);
            rdfg_alloc.alloc_node(expr_val, None, type_ctx)?;
        }
        // Step 1.1: Add all live global values. For live function definition, add their bodies to
        // `all_live_funcbody` for block scanning.
        let mut all_live_funcbody = Vec::with_capacity(global_alloc.len());
        for (handle, data) in global_alloc {
            if !global_defs.contains_key(data.get_name()) {
                continue;
            }
            let maybe_func = match data {
                GlobalData::Func(f) => {
                    if let Some(body) = f.get_blocks() {
                        let body_view = unsafe { body.unsafe_load_readonly_view() };
                        if !body_view.is_empty() {
                            all_live_funcbody.push(body_view);
                        }
                    }
                    Some(f.get_stored_func_type())
                }
                _ => None,
            };
            let global_ref = GlobalRef::from_handle(handle);
            let global_val = ValueSSA::Global(global_ref);
            rdfg_alloc.alloc_node(global_val, maybe_func, type_ctx)?;
        }
        // Step 1.2: Allocate nodes for all live basic blocks. If this block is not empty,
        // add instruction list to `live_insts`
        let mut live_insts = Vec::with_capacity(inst_alloc.len());
        for body_view in all_live_funcbody {
            for (blockref, block) in body_view.view(block_alloc) {
                let blockval = ValueSSA::Block(blockref);
                rdfg_alloc.alloc_node(blockval, None, type_ctx)?;

                let insts = unsafe { block.instructions.unsafe_load_readonly_view() };
                if !insts.is_empty() {
                    live_insts.push(insts);
                }
            }
        }
        // Step 1.3: Allocate nodes for all live instructions. If this instruction is not empty,
        // add operand uses to `live_opreands`.
        let use_alloc = self.borrow_use_alloc();
        let mut live_opreands = Vec::with_capacity(use_alloc.len());
        for inst_list in live_insts {
            for (instref, inst) in inst_list.view(inst_alloc) {
                rdfg_alloc.alloc_node(ValueSSA::Inst(instref), None, type_ctx)?;
                match inst {
                    InstData::ListGuideNode(..) | InstData::PhiInstEnd(..) | InstData::Jump(..) => {
                    }
                    _ => {
                        let operand_view = unsafe {
                            inst.get_common_unwrap()
                                .operands
                                .unsafe_load_readonly_view()
                        };
                        if !operand_view.is_empty() {
                            live_opreands.push(operand_view);
                        }
                    }
                }
            }
        }

        // Step 2: Add instruction operands
        let mut live_uses = Vec::with_capacity(use_alloc.len());
        for use_list in &live_opreands {
            for (useref, usedata) in use_list.view(&use_alloc) {
                let operand = usedata.get_operand();
                match operand {
                    ValueSSA::None | ValueSSA::ConstData(..) => {}
                    _ => live_uses.push((useref, operand)),
                }
            }
        }
        for (useref, operand) in live_uses {
            match rdfg_alloc.get_node(operand) {
                Ok(node) => node.add_user_use(useref),
                Err(ModuleError::DfgOperandNotReferece(_)) => {
                    // Every instruction may have some operands that are not references.
                    // Just ignore the error.
                }
                Err(_) => todo!(),
            };
        }

        // Step 3: Insert RDFG into this module
        *self._rdfg_alloc.borrow_mut() = Some(rdfg_alloc);

        Ok(())
    }

    /// Disable DFG reverse-graph tracking.
    ///
    /// **WARNING**: This function will simply shut down all DFG reverse-graphs.
    /// Passes which depend on DFG reverse-graphs will be broken.
    pub fn disable_rdfg(&self) {
        *self._rdfg_alloc.borrow_mut() = None;
    }

    /// Disable DFG reverse-graph tracking and take out all DFG reverse-graphs
    /// for the module.
    pub fn steal_tracking_dfg(&self) -> Option<rdfg::RdfgAlloc> {
        self._rdfg_alloc.borrow_mut().take()
    }

    pub(crate) fn operand_add_use(
        &self,
        operand: ValueSSA,
        useref: UseRef,
    ) -> Result<(), ModuleError> {
        match self
            .borrow_rdfg_alloc()
            .ok_or(ModuleError::RDFGNotEnabled)?
            .get_node(operand)
        {
            Ok(node) => node.add_user_use(useref),
            Err(ModuleError::DfgOperandNotReferece(_)) => {
                // Every instruction may have some operands that are not references.
                // Just ignore the error.
            }
            Err(x) => return Err(x),
        }
        Ok(())
    }

    pub(crate) fn operand_del_use(
        &self,
        operand: ValueSSA,
        useref: UseRef,
    ) -> Result<(), ModuleError> {
        match self
            .borrow_rdfg_alloc()
            .ok_or(ModuleError::RDFGNotEnabled)?
            .get_node(operand)
        {
            Ok(node) => node.remove_user_use(useref),
            Err(ModuleError::DfgOperandNotReferece(_)) => {
                // Every instruction may have some operands that are not references.
                // Just ignore the error.
            }
            Err(x) => return Err(x),
        }
        Ok(())
    }

    pub fn borrow_rdfg_alloc(&self) -> Option<Ref<RdfgAlloc>> {
        let alloc_rdfg = self._rdfg_alloc.borrow();
        if let None = *alloc_rdfg {
            return None;
        }
        Some(Ref::map(alloc_rdfg, |alloc| alloc.as_ref().unwrap()))
    }
    pub fn borrow_rdfg_alloc_mut(&self) -> Option<RefMut<RdfgAlloc>> {
        let alloc_rdfg = self._rdfg_alloc.borrow_mut();
        if let None = *alloc_rdfg {
            return None;
        }
        Some(RefMut::map(alloc_rdfg, |alloc| alloc.as_mut().unwrap()))
    }
}

/// Module as control flow graph maintainer.
impl Module {
    pub fn borrow_rcfg_alloc(&self) -> Option<Ref<rcfg::RcfgAlloc>> {
        if let None = *self._rcfg_alloc.borrow() {
            return None;
        }
        Some(Ref::map(self._rcfg_alloc.borrow(), |alloc| {
            alloc.as_ref().unwrap()
        }))
    }
    pub fn borrow_rcfg_alloc_mut(&self) -> Option<RefMut<rcfg::RcfgAlloc>> {
        if let None = *self._rcfg_alloc.borrow() {
            return None;
        }
        Some(RefMut::map(self._rcfg_alloc.borrow_mut(), |alloc| {
            alloc.as_mut().unwrap()
        }))
    }

    pub fn rcfg_enabled(&self) -> bool {
        self._rcfg_alloc.borrow().is_some()
    }
    pub fn enable_rcfg(&self) -> Result<(), ModuleError> {
        if self._rcfg_alloc.borrow().is_some() {
            return Err(ModuleError::RCFGEnabled);
        }

        // Step 1: Collect all live blocks and allocate nodes for them.
        let alloc_value = self.borrow_value_alloc();
        let alloc_global = &alloc_value.alloc_global;
        let alloc_block = &alloc_value.alloc_block;
        let alloc_inst = &alloc_value.alloc_inst;
        let mut live_funcbody = Vec::with_capacity(alloc_global.len());
        for (_, global) in self.global_defs.borrow().iter() {
            match global.to_slabref_unwrap(alloc_global) {
                GlobalData::Func(func) => {
                    if let Some(body) = func.get_blocks() {
                        let body_view = unsafe { body.unsafe_load_readonly_view() };
                        if !body_view.is_empty() {
                            live_funcbody.push(body_view);
                        }
                    }
                }
                _ => continue,
            };
        }
        let mut live_bb = Vec::with_capacity(alloc_block.len());
        for body_view in live_funcbody {
            for (blockref, block) in body_view.view(alloc_block) {
                live_bb.push((
                    blockref,
                    block.instructions.get_back_ref(alloc_inst).unwrap(),
                ));
            }
        }

        // Step 2: Allocate nodes for all live basic blocks.
        let mut rcfg_alloc = rcfg::RcfgAlloc::new_with_capacity(alloc_block.capacity());
        for (block, _) in &live_bb {
            rcfg_alloc.alloc_node(*block);
        }

        // Step 3: Insert jump relationship for all live basic blocks.
        let alloc_jt = self.borrow_jt_alloc();
        for (block, inst) in &live_bb {
            let inst_data = inst.to_slabref_unwrap(alloc_inst);
            if !inst_data.is_terminator() {
                panic!("Block {:?} does not contain a terminator", block);
            }

            let jts_view = if let Some((_, t)) = inst_data.as_terminator() {
                let jts = match t.get_jump_targets() {
                    Some(jts) => jts,
                    None => continue,
                };
                unsafe { jts.unsafe_load_readonly_view() }
            } else {
                continue;
            };

            for (jt, jt_data) in jts_view.view(&alloc_jt) {
                let block = jt_data._block.get();
                if block.is_null() {
                    continue;
                }
                rcfg_alloc.get_node(block).add_predecessor(jt);
            }
        }

        // Step 4: Insert RCFG into this module
        *self._rcfg_alloc.borrow_mut() = Some(rcfg_alloc);

        Ok(())
    }
}

/// Module as context maintainer.
impl Module {
    /// Perform a basic check on the module.
    pub fn perform_basic_check(&self) {
        let alloc_value = self.borrow_value_alloc();
        let alloc_global = &alloc_value.alloc_global;
        let alloc_block = &alloc_value.alloc_block;

        for (_, global) in alloc_global {
            let func_body = match global {
                GlobalData::Func(func) => match func.get_blocks() {
                    Some(body) => body,
                    None => continue,
                },
                _ => continue,
            };

            for (_, block) in func_body.view(alloc_block) {
                block.perform_basic_check(self);
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

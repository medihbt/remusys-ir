use std::cell::Cell;

use slab::Slab;

use crate::{
    base::{
        NullableValue,
        slablist::{
            SlabRefList, SlabRefListError, SlabRefListNode, SlabRefListNodeHead, SlabRefListNodeRef,
        },
        slabref::SlabRef,
    },
    impl_slabref,
    typing::id::ValTypeID,
};

use super::{
    ValueSSA,
    constant::data::ConstData,
    global::GlobalRef,
    inst::{InstData, InstError, InstRef, terminator},
    module::Module,
};

pub mod jump_target;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct BlockRef(usize);

impl_slabref!(BlockRef, BlockData);

impl SlabRefListNodeRef for BlockRef {
    fn on_node_push_next(
        curr: Self,
        next: Self,
        alloc: &Slab<BlockData>,
    ) -> Result<(), SlabRefListError> {
        if curr == next {
            return Err(SlabRefListError::RepeatedNode(next.get_handle()));
        }
        let self_parent = curr.to_slabref_unwrap(alloc).get_parent_func();
        next.to_slabref_unwrap(alloc).set_parent_func(self_parent);
        Ok(())
    }

    fn on_node_push_prev(
        curr: Self,
        prev: Self,
        alloc: &Slab<BlockData>,
    ) -> Result<(), SlabRefListError> {
        if curr == prev {
            Err(SlabRefListError::RepeatedNode(prev.get_handle()))
        } else {
            let self_parent = curr.to_slabref_unwrap(alloc).get_parent_func();
            prev.to_slabref_unwrap(alloc).set_parent_func(self_parent);
            Ok(())
        }
    }

    fn on_node_unplug(curr: Self, alloc: &Slab<BlockData>) -> Result<(), SlabRefListError> {
        let self_data = curr.to_slabref_unwrap(alloc);
        if self_data.get_parent_func().is_null() {
            Err(SlabRefListError::UnpluggedItemAttached(curr.get_handle()))
        } else {
            self_data.set_parent_func(GlobalRef::new_null());
            Ok(())
        }
    }
}

/// Basic block data.
pub struct BlockData {
    pub instructions: SlabRefList<InstRef>,
    pub phi_node_end: Cell<InstRef>,
    pub(super) _inner: Cell<BlockDataInner>,
}

#[derive(Debug, Clone, Copy)]
pub struct BlockDataInner {
    pub(super) _node_head: SlabRefListNodeHead,
    pub(super) _self_ref: BlockRef,
    pub(super) _parent_func: GlobalRef,
    pub(super) _id: usize,
}

impl BlockDataInner {
    fn insert_node_head(mut self, node_head: SlabRefListNodeHead) -> Self {
        self._node_head = node_head;
        self
    }
    fn insert_self_ref(mut self, self_ref: BlockRef) -> Self {
        self._self_ref = self_ref;
        self
    }
    pub(super) fn insert_parent_func(mut self, parent_func: GlobalRef) -> Self {
        self._parent_func = parent_func;
        self
    }
    fn insert_id(mut self, id: usize) -> Self {
        self._id = id;
        self
    }
    pub(super) fn assign_to(&self, cell: &Cell<BlockDataInner>) {
        cell.set(*self);
    }
}

impl SlabRefListNode for BlockData {
    fn new_guide() -> Self {
        Self {
            instructions: SlabRefList::new_guide(),
            phi_node_end: Cell::new(InstRef::new_null()),
            _inner: Cell::new(BlockDataInner {
                _node_head: SlabRefListNodeHead::new(),
                _self_ref: BlockRef::new_null(),
                _parent_func: GlobalRef::new_null(),
                _id: 0,
            }),
        }
    }

    fn load_node_head(&self) -> SlabRefListNodeHead {
        self._inner.get()._node_head
    }

    fn store_node_head(&self, node_head: SlabRefListNodeHead) {
        self._inner
            .get()
            .insert_node_head(node_head)
            .assign_to(&self._inner);
    }
}

impl BlockData {
    pub fn get_parent_func(&self) -> GlobalRef {
        self._inner.get()._parent_func
    }
    pub fn set_parent_func(&self, parent_func: GlobalRef) {
        self._inner
            .get()
            .insert_parent_func(parent_func)
            .assign_to(&self._inner);
    }

    pub fn get_id(&self) -> usize {
        self._inner.get()._id
    }
    pub fn set_id(&self, id: usize) {
        self._inner.get().insert_id(id).assign_to(&self._inner);
    }

    pub fn get_termiantor(&self, module: &Module) -> Option<InstRef> {
        let alloc_value = module.borrow_value_alloc();
        let alloc_inst = &alloc_value._alloc_inst;
        let back_inst = match self.instructions.get_back_ref(alloc_inst) {
            Some(inst) => inst,
            None => return None,
        };
        if module.get_inst(back_inst).is_terminator() {
            Some(back_inst)
        } else {
            None
        }
    }
    pub fn has_terminator(&self, module: &Module) -> bool {
        self.get_termiantor(module).is_some()
    }
    pub fn set_terminator(&self, module: &Module, terminator: InstRef) -> Result<(), InstError> {
        if let Some(old) = self.get_termiantor(module) {
            old.detach_self(module)?;
        }
        self.instructions._tail.add_prev_inst(module, terminator)
    }

    pub fn build_add_inst(&self, inst: InstRef, module: &Module) -> Result<(), InstError> {
        if let InstData::Phi(..) = &*module.get_inst(inst) {
            self.build_add_phi(inst, module)
        } else if let Some(terminator) = self.get_termiantor(module) {
            if module.get_inst(terminator).is_terminator() {
                return Err(InstError::ReplicatedTerminator(terminator, inst));
            }
            terminator.add_prev_inst(module, inst)
        } else {
            self.instructions._tail.add_prev_inst(module, inst)
        }
    }
    pub fn build_add_phi(&self, inst: InstRef, module: &Module) -> Result<(), InstError> {
        match &*module.get_inst(inst) {
            InstData::Phi(..) => {
                let phi_node_end = self.phi_node_end.get();
                phi_node_end.add_prev_inst(module, inst)
            }
            _ => panic!("Expected a phi node but got {:?}", inst),
        }
    }

    /// Set the self reference of the block.
    /// Then, initialize all instructions in the block with the self reference.
    /// This function is called when the block is allocated into the module.
    pub(super) fn init_set_self_reference(&self, self_ref: BlockRef, alloc_inst: &Slab<InstData>) {
        self._inner
            .get()
            .insert_self_ref(self_ref)
            .assign_to(&self._inner);
        let mut noderef = self.instructions._head;
        while noderef.is_nonnull() {
            let inst = noderef.to_slabref_unwrap(alloc_inst);
            match inst {
                InstData::ListGuideNode(_, bb) => bb.set(self_ref),
                _ => {
                    let inner = &inst.get_common_unwrap().inner;
                    inner
                        .get()
                        .insert_parent_bb(Some(self_ref))
                        .assign_to(&inner);
                }
            }
            noderef = InstRef::from_option(noderef.get_next_ref(alloc_inst));
        }
    }

    /// Perform a basic check on the block data.
    ///
    /// ### Rules
    ///
    /// 1. Block should be initialized with its self reference and parent function.
    /// 2. Block should have a terminator instruction at the end.
    /// 3. Block should contain a `PhiEnd` node to split `Phi` nodes and other instructions.
    ///    All `Phi` nodes should be before the `PhiEnd` node while other instructions should be after it.
    /// 4. Every instruction in the block should pass its operand check.
    pub(super) fn perform_basic_check(&self, module: &Module) {
        // Preparations: allocators, etc.
        let alloc_value = module.borrow_value_alloc();
        let alloc_inst = &alloc_value._alloc_inst;

        // 1. Check if the block is initialized with its self reference and parent function.
        let self_ref = if self._inner.get()._self_ref.is_nonnull() {
            self._inner.get()._self_ref
        } else {
            panic!("Block is not initialized with its self reference.");
        };
        if self._inner.get()._parent_func.is_null() {
            panic!("Block is not initialized with its parent function.");
        }

        // 2. Check if the block has a terminator instruction at the end.
        if self.get_termiantor(module).is_none() {
            panic!("Block does not have a terminator instruction at the end.");
        }

        // 3. Check if the block contains a `PhiEnd` node to split `Phi` nodes and other instructions.
        //    All `Phi` nodes should be before the `PhiEnd` node while other instructions should be after it.
        let phi_node_end = self.phi_node_end.get();
        if phi_node_end.is_null() {
            panic!(
                "Block does not contain a `PhiEnd` node to split `Phi` nodes and other instructions."
            );
        }
        if !module.get_inst(phi_node_end).is_attached() {
            panic!("`PhiEnd` node is not attached to the block.");
        }

        // 3.1. Traverse through all PHI nodes (from entry to `PhiEnd`) and check if they are valid.
        {
            let mut noderef = self.instructions._head.get_next_ref(alloc_inst).unwrap();
            while noderef != phi_node_end {
                let inst = noderef.to_slabref_unwrap(alloc_inst);
                if !inst.is_attached() {
                    panic!("`Phi` node is not attached to the block.");
                }
                match inst {
                    InstData::Phi(..) => {}
                    _ => panic!("Expected a `Phi` node but got {:?}", inst.get_opcode()),
                }
                if inst.get_parent_bb() != Some(self_ref) {
                    panic!("`Phi` node is not attached to the block.");
                }
                inst.check_operands(module).unwrap();
                noderef = InstRef::from_handle(inst.load_node_head().next);
            }
        }

        // 4. Check if all instructions in the block pass their operand check.
        {
            let mut noderef = self.instructions._head.get_next_ref(alloc_inst).unwrap();
            while noderef.is_nonnull() {
                let inst = noderef.to_slabref_unwrap(alloc_inst);
                if inst.get_parent_bb() != Some(self_ref) {
                    panic!("Instruction is not attached to the block.");
                }
                inst.check_operands(module).unwrap();
                noderef = InstRef::from_handle(inst.load_node_head().next);
            }
        }
    }
}

impl BlockData {
    pub fn new_empty(module: &Module) -> Self {
        let ret = Self {
            instructions: SlabRefList::from_slab(&mut module.borrow_value_alloc_mut()._alloc_inst),
            phi_node_end: Cell::new(InstRef::new_null()),
            _inner: Cell::new(BlockDataInner {
                _node_head: SlabRefListNodeHead::new(),
                _self_ref: BlockRef::new_null(),
                _parent_func: GlobalRef::new_null(),
                _id: 0,
            }),
        };

        let phi_end = module.insert_inst(InstData::new_phi_end());
        ret.instructions
            .push_back_ref(&mut module.borrow_value_alloc_mut()._alloc_inst, phi_end)
            .unwrap();
        ret.phi_node_end.set(phi_end);
        ret
    }

    pub fn new_unreachable(module: &Module) -> Result<Self, SlabRefListError> {
        let ret = Self::new_empty(module);
        let unreachable_inst = InstData::new_unreachable(&mut module.borrow_use_alloc_mut());
        let unreachable_inst = module.insert_inst(unreachable_inst);
        ret.instructions.push_back_ref(
            &mut module.borrow_value_alloc_mut()._alloc_inst,
            unreachable_inst,
        )?;
        Ok(ret)
    }

    pub fn new_return_zero(module: &Module, valtype: ValTypeID) -> Result<Self, SlabRefListError> {
        let ret_bb = Self::new_empty(module);

        let (ret_common, ret_inst) =
            terminator::Ret::new(module, ValueSSA::ConstData(ConstData::Zero(valtype)));
        let ret_inst = module.insert_inst(InstData::Ret(ret_common, ret_inst));

        ret_bb
            .instructions
            .push_back_ref(&mut module.borrow_value_alloc_mut()._alloc_inst, ret_inst)?;
        Ok(ret_bb)
    }
}

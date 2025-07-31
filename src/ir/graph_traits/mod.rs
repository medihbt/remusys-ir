use super::module::Module;
use crate::base::{SlabListNodeRef, SlabListRange, SlabRef, SlabRefList};
use slab::Slab;
use std::{cell::Ref, ops::Deref};

pub mod block;
pub mod inst;

pub trait IRGraphEdge: SlabListNodeRef {
    type UserT: SlabRef;
    type OperandT;

    fn module_borrow_self_alloc<'a>(module: &'a Module) -> Ref<'a, Slab<Self::RefObject>>;

    fn graph_get_user_from_alloc(&self, alloc: &Slab<Self::RefObject>) -> Self::UserT;
    fn graph_get_user_from_module(&self, module: &Module) -> Self::UserT {
        let alloc = Self::module_borrow_self_alloc(module);
        self.graph_get_user_from_alloc(alloc.deref())
    }

    fn graph_get_operand_from_alloc(&self, alloc: &Slab<Self::RefObject>) -> Self::OperandT;
    fn graph_get_operand_from_module(&self, module: &Module) -> Self::OperandT {
        let alloc = Self::module_borrow_self_alloc(module);
        self.graph_get_operand_from_alloc(alloc.deref())
    }
}

pub trait IRGraphEdgeHolder: SlabRef {
    type EdgeT: SlabListNodeRef;

    fn module_borrow_edge_holder_alloc<'a>(module: &'a Module) -> Ref<'a, Slab<Self::RefObject>>;

    fn graph_edges_from_data<'a>(data: &'a Self::RefObject)
    -> Option<&'a SlabRefList<Self::EdgeT>>;
    fn graph_edges_from_alloc<'a>(
        &self,
        alloc: &'a Slab<Self::RefObject>,
    ) -> Option<&'a SlabRefList<Self::EdgeT>> {
        Self::graph_edges_from_data(self.to_data(alloc))
    }
    fn graph_edges_from_module<'a>(
        &self,
        module: &'a Module,
    ) -> Option<Ref<'a, SlabRefList<Self::EdgeT>>>
    where
        <Self as SlabRef>::RefObject: 'a,
    {
        let alloc: Ref<'a, Slab<<Self as SlabRef>::RefObject>> =
            Self::module_borrow_edge_holder_alloc(module);
        if self.graph_edges_from_alloc(&alloc).is_none() {
            return None;
        }
        Some(Ref::map(alloc, |alloc| {
            self.graph_edges_from_alloc(alloc).unwrap()
        }))
    }

    fn graph_load_edges_range_from_data(
        data: &Self::RefObject,
    ) -> Option<SlabListRange<Self::EdgeT>> {
        Self::graph_edges_from_data(data).map(|l| l.load_range())
    }
    fn graph_load_edges_range_from_alloc(
        &self,
        alloc: &Slab<Self::RefObject>,
    ) -> Option<SlabListRange<Self::EdgeT>> {
        self.graph_edges_from_alloc(alloc).map(|l| l.load_range())
    }
    fn graph_load_edges_range_from_module(
        &self,
        module: &Module,
    ) -> Option<SlabListRange<Self::EdgeT>> {
        self.graph_edges_from_module(module).map(|l| l.load_range())
    }
}

pub trait IRGraphNode: SlabListNodeRef {
    type OperandT;
    type ReverseGraphNodeT;
    type EdgeHolderT: IRGraphEdgeHolder<EdgeT = Self::EdgeT>;
    type EdgeT: IRGraphEdge<UserT = Self::EdgeHolderT, OperandT = Self::OperandT>;

    fn module_borrow_self_alloc<'a>(module: &'a Module) -> Ref<'a, Slab<Self::RefObject>>;

    /// Collects all operands from the module into a vector.
    /// If `dedup` is true, it will remove duplicates.
    fn graph_collect_operands_from_module(
        self,
        module: &Module,
        dedup: bool,
    ) -> Vec<Self::OperandT>;

    /// Get the reverse graph node of the operand from the module.
    fn get_operand_reverse_graph<'a>(
        module: &'a Module,
        operand: &Self::OperandT,
    ) -> Option<Ref<'a, Self::ReverseGraphNodeT>>;

    fn edge_holder_from_allocs(
        &self,
        alloc_self: &Slab<Self::RefObject>,
        alloc_edge_holder: &Slab<<Self::EdgeHolderT as SlabRef>::RefObject>,
    ) -> Self::EdgeHolderT;
    fn edge_holder_from_module(&self, module: &Module) -> Self::EdgeHolderT {
        let alloc_self = Self::module_borrow_self_alloc(module);
        let alloc_edge_holder = Self::EdgeHolderT::module_borrow_edge_holder_alloc(module);
        self.edge_holder_from_allocs(alloc_self.deref(), alloc_edge_holder.deref())
    }

    fn graph_get_edges_from_alloc<'a>(
        &self,
        alloc_self: &Slab<Self::RefObject>,
        alloc_edge_holder: &Slab<<Self::EdgeHolderT as SlabRef>::RefObject>,
        alloc_edge: &'a Slab<<Self::EdgeHolderT as SlabRef>::RefObject>,
    ) -> Option<&'a SlabRefList<Self::EdgeT>> {
        let edge_holder = self.edge_holder_from_allocs(alloc_self, alloc_edge_holder);
        edge_holder.graph_edges_from_alloc(alloc_edge)
    }
    fn graph_get_edges_from_module<'a>(
        &self,
        module: &'a Module,
    ) -> Option<Ref<'a, SlabRefList<Self::EdgeT>>>
    where
        <<Self as IRGraphNode>::EdgeHolderT as SlabRef>::RefObject: 'a,
    {
        let edge_holder = self.edge_holder_from_module(module);
        edge_holder.graph_edges_from_module(module)
    }
}

use std::{cell::Ref, collections::HashSet};

use slab::Slab;

use crate::{
    base::{slablist::SlabRefList, slabref::SlabRef},
    ir::{
        ValueSSA,
        inst::{
            InstData, InstRef,
            usedef::{UseData, UseRef},
        },
        module::Module,
    },
};

use super::{IRGraphEdge, IRGraphEdgeHolder, IRGraphNode};

impl IRGraphEdge for UseRef {
    type UserT = InstRef;
    type OperandT = ValueSSA;

    fn module_borrow_self_alloc<'a>(module: &'a Module) -> Ref<'a, Slab<UseData>> {
        module.borrow_use_alloc()
    }
    fn graph_get_user_from_alloc(&self, alloc: &Slab<UseData>) -> InstRef {
        self.to_slabref_unwrap(alloc).get_user()
    }
    fn graph_get_operand_from_alloc(&self, alloc: &Slab<UseData>) -> ValueSSA {
        self.to_slabref_unwrap(alloc).get_operand()
    }
}

impl IRGraphEdgeHolder for InstRef {
    type EdgeT = UseRef;

    fn module_borrow_edge_holder_alloc<'a>(module: &'a Module) -> Ref<'a, Slab<InstData>> {
        Ref::map(module.borrow_value_alloc(), |alloc_value| {
            &alloc_value.alloc_inst
        })
    }

    fn graph_edges_from_data<'a>(data: &'a InstData) -> Option<&'a SlabRefList<UseRef>> {
        match data.get_common() {
            Some(common) => Some(&common.operands),
            None => None,
        }
    }
}

impl IRGraphNode for InstRef {
    type OperandT = ValueSSA;
    type EdgeHolderT = Self;
    type EdgeT = UseRef;

    fn module_borrow_self_alloc<'a>(module: &'a Module) -> Ref<'a, Slab<InstData>> {
        Ref::map(module.borrow_value_alloc(), |alloc_value| {
            &alloc_value.alloc_inst
        })
    }
    fn edge_holder_from_allocs(&self, _: &Slab<InstData>, _: &Slab<InstData>) -> Self {
        self.clone()
    }

    fn graph_collect_operands_from_module(self, module: &Module, dedup: bool) -> Vec<ValueSSA> {
        let edges = unsafe {
            match self.graph_load_edges_from_module(module) {
                Some(edges) => edges,
                None => return vec![],
            }
        };
        let mut operands = Vec::with_capacity(edges.len());
        let mut dedup_set = HashSet::new();
        for (_, usedata) in edges.view(&module.borrow_use_alloc()) {
            let operand = usedata.get_operand();
            if dedup && !dedup_set.insert(operand) {
                continue;
            }
            operands.push(operand);
        }
        operands
    }
}

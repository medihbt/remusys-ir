use crate::{
    base::{
        slablist::{SlabRefListError, SlabRefListNode, SlabRefListNodeHead, SlabRefListNodeRef},
        slabref::SlabRef,
    },
    impl_slabref,
    mir::{
        inst::{
            branch::{BLink, CondBr, RegCondBr, UncondBr},
            call_ret::{MirCall, MirReturn},
            cmp::{CmpOP, CondCmpOP},
            data_process::{
                BFMOp, BinCSROp, BinOp, CondSelect, CondSet, CondUnaryOp, ExtROp, TernaryOp,
                UnaCSROp, UnaryOp,
            },
            load_store::{LoadStoreLiteral, LoadStoreRRI, LoadStoreRRR},
            opcode::MirOP,
            switch::{BinSwitch, TabSwitch},
        },
        module::MirModule,
        operand::MirOperand,
    },
};
use slab::Slab;
use std::cell::{Cell, Ref};

pub mod branch;
pub mod call_ret;
pub mod cmp;
pub mod cond;
pub mod data_process;
pub mod load_store;
pub mod opcode;
pub mod switch;

/// MIR and AArch64 assembly instructions.
#[derive(Debug, Clone)]
pub enum MirInst {
    GuideNode(MirInstCommon),
    Nullary(MirInstCommon),

    // Branch instructions
    CondBr(CondBr),
    UncondBr(UncondBr),
    BLink(BLink),
    RegCondBr(RegCondBr),

    // Loads and stores
    LoadStoreRRR(LoadStoreRRR),
    LoadStoreRRI(LoadStoreRRI),
    LoadStoreLiteral(LoadStoreLiteral),

    // Data processing instructions
    Bin(BinOp),
    BinCSR(BinCSROp),
    Unary(UnaryOp),
    UnaryCSR(UnaCSROp),
    BFM(BFMOp),
    ExtR(ExtROp),
    Tri(TernaryOp),

    Cmp(CmpOP),

    // Conditional instructions
    CondSelect(CondSelect),
    CondUnary(CondUnaryOp),
    CondSet(CondSet),
    CondCmp(CondCmpOP),

    // Pseudo instructions
    Call(MirCall),
    MirReturn(MirReturn),
    TabSwitch(TabSwitch),
    BinSwitch(BinSwitch),
}

#[derive(Debug, Clone)]
pub struct MirInstCommon {
    node_head: Cell<SlabRefListNodeHead>,
    opcode: MirOP,
    operands_placeholder: [Cell<MirOperand>; 0],
}

impl MirInstCommon {
    pub fn new(opcode: MirOP) -> Self {
        Self {
            node_head: Cell::new(SlabRefListNodeHead::new()),
            opcode,
            operands_placeholder: [],
        }
    }
}

pub trait IMirSubInst {
    fn from_mir_inst(inst: &MirInst) -> Option<&Self>;
    fn into_mir_inst(self) -> MirInst;
    fn common(&self) -> &MirInstCommon;
    fn operands(&self) -> &[Cell<MirOperand>];
    fn accepts_opcode(opcode: MirOP) -> bool;
    fn new_empty(opcode: MirOP) -> Self;

    fn get_opcode(&self) -> MirOP {
        self.common().opcode
    }
}

impl MirInst {
    pub fn get_common(&self) -> &MirInstCommon {
        match self {
            MirInst::GuideNode(common) | MirInst::Nullary(common) => common,
            MirInst::CondBr(inst) => &inst.common,
            MirInst::UncondBr(inst) => &inst.common,
            MirInst::BLink(inst) => &inst.common,
            MirInst::RegCondBr(inst) => &inst.common,
            MirInst::LoadStoreRRR(inst) => &inst.common,
            MirInst::LoadStoreRRI(inst) => &inst.common,
            MirInst::LoadStoreLiteral(inst) => &inst.common,
            MirInst::Bin(bin_op) => bin_op.common(),
            MirInst::BinCSR(bin_csr_op) => bin_csr_op.common(),
            MirInst::Unary(unary_op) => unary_op.common(),
            MirInst::BFM(bfmop) => bfmop.common(),
            MirInst::ExtR(ext_rop) => ext_rop.common(),
            MirInst::Tri(ternary_op) => ternary_op.common(),
            MirInst::Cmp(cmp_op) => cmp_op.common(),
            MirInst::CondSelect(cond_select) => cond_select.common(),
            MirInst::CondUnary(cond_unary_op) => cond_unary_op.common(),
            MirInst::CondSet(cond_set) => cond_set.common(),
            MirInst::CondCmp(cond_cmp_op) => cond_cmp_op.common(),
            MirInst::Call(call) => &call.common,
            MirInst::MirReturn(mir_return) => &mir_return.common,
            MirInst::TabSwitch(tab_switch) => &tab_switch.common,
            MirInst::BinSwitch(bin_switch) => &bin_switch.common,
            MirInst::UnaryCSR(inst) => inst.common(),
        }
    }
    pub fn get_opcode(&self) -> MirOP {
        self.get_common().opcode
    }
    pub fn operands(&self) -> &[Cell<MirOperand>] {
        match self {
            MirInst::GuideNode(common) | MirInst::Nullary(common) => &common.operands_placeholder,
            MirInst::CondBr(inst) => &inst.operands,
            MirInst::UncondBr(inst) => &inst.operands,
            MirInst::BLink(inst) => &inst.operands,
            MirInst::RegCondBr(inst) => &inst.operands,
            MirInst::LoadStoreRRR(load_store_rrr) => &load_store_rrr.operands,
            MirInst::LoadStoreRRI(load_store_rri) => &load_store_rri.operands,
            MirInst::LoadStoreLiteral(load_store_literal) => &load_store_literal.operands,
            MirInst::Bin(bin_op) => bin_op.operands(),
            MirInst::Unary(unary_op) => unary_op.operands(),
            MirInst::BFM(bfmop) => bfmop.operands(),
            MirInst::ExtR(ext_rop) => ext_rop.operands(),
            MirInst::Tri(ternary_op) => ternary_op.operands(),
            MirInst::Cmp(cmp_op) => cmp_op.operands(),
            MirInst::CondSelect(cond_select) => cond_select.operands(),
            MirInst::CondUnary(cond_unary_op) => cond_unary_op.operands(),
            MirInst::CondSet(cond_set) => cond_set.operands(),
            MirInst::CondCmp(cond_cmp_op) => cond_cmp_op.operands(),
            MirInst::Call(call) => call.operands.as_slice(),
            MirInst::MirReturn(ret) => ret.operands(),
            MirInst::TabSwitch(tab_switch) => &tab_switch.operands,
            MirInst::BinSwitch(bin_switch) => &bin_switch.operands,
            MirInst::BinCSR(bin_csrop) => bin_csrop.operands(),
            MirInst::UnaryCSR(inst) => inst.operands(),
        }
    }
}

impl SlabRefListNode for MirInst {
    fn new_guide() -> Self {
        Self::GuideNode(MirInstCommon::new(MirOP::Nop))
    }
    fn load_node_head(&self) -> SlabRefListNodeHead {
        self.get_common().node_head.get()
    }
    fn store_node_head(&self, node_head: SlabRefListNodeHead) {
        self.get_common().node_head.set(node_head);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct MirInstRef(usize);
impl_slabref!(MirInstRef, MirInst);

impl SlabRefListNodeRef for MirInstRef {
    fn on_node_push_next(_: Self, _: Self, _: &Slab<MirInst>) -> Result<(), SlabRefListError> {
        Ok(())
    }
    fn on_node_push_prev(_: Self, _: Self, _: &Slab<MirInst>) -> Result<(), SlabRefListError> {
        Ok(())
    }
    fn on_node_unplug(_: Self, _: &Slab<MirInst>) -> Result<(), SlabRefListError> {
        Ok(())
    }
}

impl MirInstRef {
    pub fn from_alloc(alloc: &mut Slab<MirInst>, data: MirInst) -> Self {
        let index = alloc.insert(data);
        if index == usize::MAX {
            panic!("Failed to allocate MirInst in slab");
        }
        MirInstRef(index)
    }
    pub fn from_module(module: &MirModule, data: MirInst) -> Self {
        let mut alloc = module.borrow_alloc_inst_mut();
        MirInstRef::from_alloc(&mut alloc, data)
    }

    pub fn get_common(self, alloc: &Slab<MirInst>) -> &MirInstCommon {
        self.to_slabref_unwrap(alloc).get_common()
    }
    pub fn get_opcode(self, alloc: &Slab<MirInst>) -> MirOP {
        self.get_common(alloc).opcode
    }
    pub fn operands(self, alloc: &Slab<MirInst>) -> &[Cell<MirOperand>] {
        self.to_slabref_unwrap(alloc).operands()
    }
    pub fn noperands(self, alloc: &Slab<MirInst>) -> usize {
        self.operands(alloc).len()
    }

    pub fn data_from_module(self, module: &MirModule) -> Ref<MirInst> {
        let alloc = module.borrow_alloc_inst();
        Ref::map(alloc, |a| a.get(self.0).expect("Invalid MirInstRef"))
    }
    pub fn get_common_from_module(self, module: &MirModule) -> Ref<MirInstCommon> {
        let alloc = module.borrow_alloc_inst();
        Ref::map(alloc, |a| self.get_common(a))
    }
    pub fn get_opcode_from_module(self, module: &MirModule) -> MirOP {
        self.get_common_from_module(module).opcode
    }
    pub fn operands_from_module(self, module: &MirModule) -> Ref<[Cell<MirOperand>]> {
        let alloc = module.borrow_alloc_inst();
        Ref::map(alloc, |a| self.operands(a))
    }
    pub fn noperands_from_module(self, module: &MirModule) -> usize {
        self.operands_from_module(module).len()
    }
}

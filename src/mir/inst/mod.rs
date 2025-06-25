use crate::{
    base::slablist::{SlabRefListNode, SlabRefListNodeHead},
    mir::{
        inst::{
            branch::{BLink, CondBr, RegCondBr, UncondBr},
            call::MirCall,
            cmp::{CmpOP, CondCmpOP},
            data_process::{
                BFMOp, BinOp, CondSelect, CondSet, CondUnaryOp, ExtROp, TernaryOp, UnaryOp,
            },
            load_store::{LoadStoreLiteral, LoadStoreRRI, LoadStoreRRR},
            opcode::MirOP,
            switch::{BinSwitch, TabSwitch},
        },
        operand::MirOperand,
    },
};
use std::cell::Cell;

pub mod branch;
pub mod call;
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
    Unary(UnaryOp),
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

impl MirInst {
    pub fn get_common(&self) -> &MirInstCommon {
        match self {
            MirInst::GuideNode(common) | MirInst::Nullary(common) => common,
            MirInst::CondBr(inst) => &inst.common,
            MirInst::UncondBr(inst) => &inst.common,
            MirInst::BLink(inst) => &inst.common,
            MirInst::RegCondBr(inst) => &inst.common,
            MirInst::LoadStoreRRR(load_store_rrr) => &load_store_rrr.common,
            MirInst::LoadStoreRRI(load_store_rri) => &load_store_rri.common,
            MirInst::LoadStoreLiteral(load_store_literal) => &load_store_literal.common,
            MirInst::Bin(bin_op) => &bin_op.common,
            MirInst::Unary(unary_op) => &unary_op.common,
            MirInst::BFM(bfmop) => &bfmop.common,
            MirInst::ExtR(ext_rop) => &ext_rop.common,
            MirInst::Tri(ternary_op) => &ternary_op.common,
            MirInst::Cmp(cmp_op) => &cmp_op.common,
            MirInst::CondSelect(cond_select) => &cond_select.common,
            MirInst::CondUnary(cond_unary_op) => &cond_unary_op.common,
            MirInst::CondSet(cond_set) => &cond_set.common,
            MirInst::CondCmp(cond_cmp_op) => &cond_cmp_op.common,
            MirInst::Call(call) => &call.common,
            MirInst::TabSwitch(tab_switch) => &tab_switch.common,
            MirInst::BinSwitch(bin_switch) => &bin_switch.common,
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
            MirInst::BFM(bfmop) => &bfmop.operands,
            MirInst::ExtR(ext_rop) => &ext_rop.operands,
            MirInst::Tri(ternary_op) => &ternary_op.operands,
            MirInst::Cmp(cmp_op) => &cmp_op.operands,
            MirInst::CondSelect(cond_select) => &cond_select.operands,
            MirInst::CondUnary(cond_unary_op) => &cond_unary_op.operands,
            MirInst::CondSet(cond_set) => &cond_set.operands,
            MirInst::CondCmp(cond_cmp_op) => &cond_cmp_op.operands,
            MirInst::Call(call) => call.operands.as_slice(),
            MirInst::TabSwitch(tab_switch) => &tab_switch.operands,
            MirInst::BinSwitch(bin_switch) => &bin_switch.operands,
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

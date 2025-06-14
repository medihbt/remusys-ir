use std::cell::Cell;

use crate::{
    base::{
        slablist::{SlabRefListError, SlabRefListNode, SlabRefListNodeHead, SlabRefListNodeRef},
        slabref::SlabRef,
    },
    ir::cmp_cond::CmpCond,
    mir::inst::fixop::{
        FixOPInst,
        branch::{BLink, BrRegCond, CondBr, UncondBr},
        data_process::BinOP,
        load_store::{LoadStoreRRR, LoadStoreRX},
    },
};
use opcode::AArch64OP;
use slab::Slab;

pub mod fixop;
pub mod opcode;

#[derive(Debug, Clone)]
pub enum MachineInst {
    GuideNode(MachineInstCommonBase), // A guide node for the instruction list, used for padding.
    NOP(FixOPInst),                   // No operation, used for padding.

    Nullary(MachineInstCommonBase),
    CondBr(CondBr),
    UncondBr(UncondBr),
    BLink(BLink),
    BrRegCond(BrRegCond),

    LoadStoreRRR(LoadStoreRRR), // Load/Store with register
    LoadStoreRX(LoadStoreRX),   // Load/Store with immediate or label

    BinOP(BinOP),
}

impl MachineInst {
    pub fn get_common(&self) -> &MachineInstCommonBase {
        match self {
            MachineInst::GuideNode(common) => common,
            MachineInst::NOP(nop) => &nop.common,
            MachineInst::Nullary(common) => common,
            MachineInst::CondBr(cond_br) => &cond_br.common,
            MachineInst::UncondBr(uncond_br) => &uncond_br.common,
            MachineInst::BLink(blink) => &blink.common,
            MachineInst::BrRegCond(brcond) => &brcond.common,
            MachineInst::LoadStoreRRR(load_store_rrr) => &load_store_rrr.common,
            MachineInst::LoadStoreRX(load_store_rx) => &load_store_rx.0.common,
            MachineInst::BinOP(bin_op) => &bin_op.common,
        }
    }
}

impl SlabRefListNode for MachineInst {
    fn new_guide() -> Self {
        MachineInst::GuideNode(MachineInstCommonBase::new(AArch64OP::Nop))
    }
    fn load_node_head(&self) -> SlabRefListNodeHead {
        self.get_common().self_head.get()
    }
    fn store_node_head(&self, node_head: SlabRefListNodeHead) {
        self.get_common().self_head.set(node_head);
    }
}

#[derive(Debug, Clone)]
pub struct MachineInstCommonBase {
    pub self_head: Cell<SlabRefListNodeHead>,
    pub opcode: AArch64OP,
}

impl MachineInstCommonBase {
    pub fn new(opcode: AArch64OP) -> Self {
        Self {
            self_head: Cell::new(SlabRefListNodeHead::new()),
            opcode,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum BrCondFlag {
    EQ = 0b0000,
    NE = 0b0001,
    CS = 0b0010, // Carry Set
    CC = 0b0011, // Carry Clear
    MI = 0b0100, // Minus
    PL = 0b0101, // Plus
    VS = 0b0110, // Overflow Set
    VC = 0b0111, // Overflow Clear
    HI = 0b1000, // Unsigned Higher
    LS = 0b1001, // Unsigned Lower or Same
    GE = 0b1010, // Signed Greater or Equal
    LT = 0b1011, // Signed Less Than
    GT = 0b1100, // Signed Greater Than
    LE = 0b1101, // Signed Less Than or Equal
    AL = 0b1110, // Always
    NV = 0b1111, // Never
}

impl BrCondFlag {
    pub fn from_cmp_cond(cond: CmpCond) -> Self {
        let signed = if cond.is_float() {
            true
        } else {
            cond.is_signed_ordered()
        };
        #[rustfmt::skip]
        return match cond.get_basic_cond() {
            CmpCond::LT => if signed { Self::LT } else { Self::CC },
            CmpCond::EQ => Self::EQ,
            CmpCond::GT => if signed { Self::GT } else { Self::HI },
            CmpCond::LE => if signed { Self::LE } else { Self::LS },
            CmpCond::NE => Self::NE,
            CmpCond::GE => if signed { Self::GE } else { Self::CS },
            CmpCond::ALWAYS => Self::AL,
            CmpCond::NEVER  => Self::NV,
            _ => unreachable!(),
        };
    }

    pub fn get_name(self) -> &'static str {
        #[rustfmt::skip]
        return match self {
            Self::EQ => "EQ", Self::NE => "NE", Self::CS => "CS", Self::CC => "CC",
            Self::MI => "MI", Self::PL => "PL", Self::VS => "VS", Self::VC => "VC",
            Self::HI => "HI", Self::LS => "LS", Self::GE => "GE", Self::LT => "LT",
            Self::GT => "GT", Self::LE => "LE", Self::AL => "AL", Self::NV => "NV",
        };
    }
}

impl ToString for BrCondFlag {
    fn to_string(&self) -> String {
        self.get_name().to_string()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct MachineInstRef(u32);

#[rustfmt::skip]
impl SlabRef for MachineInstRef {
    type RefObject = MachineInst;
    fn from_handle(handle: usize) -> Self { MachineInstRef(handle as u32)  }
    fn get_handle (&self)        -> usize { self.0 as usize  }
}
impl SlabRefListNodeRef for MachineInstRef {
    fn on_node_push_next(_: Self, _: Self, _: &Slab<MachineInst>) -> Result<(), SlabRefListError> {
        Ok(())
    }
    fn on_node_push_prev(_: Self, _: Self, _: &Slab<MachineInst>) -> Result<(), SlabRefListError> {
        Ok(())
    }
    fn on_node_unplug(_: Self, _: &Slab<MachineInst>) -> Result<(), SlabRefListError> {
        Ok(())
    }
}

use slab::Slab;

use crate::base::slabref::SlabRef;

use super::{
    InstData, InstDataCommon, InstRef,
    binop::BinOp,
    callop::CallOp,
    cast::CastOp,
    cmp::CmpOp,
    gep::IndexPtrOp,
    load_store::{LoadOp, StoreOp},
    phi::PhiOp,
    sundury_inst::SelectOp,
    terminator::{Br, Jump, Ret, Switch},
};

pub trait IInstVisitor {
    fn read_phi_end(&self, inst_ref: InstRef);
    fn read_phi_inst(&self, inst_ref: InstRef, common: &InstDataCommon, phi: &PhiOp);
    fn read_unreachable_inst(&self, inst_ref: InstRef, common: &InstDataCommon);
    fn read_ret_inst(&self, inst_ref: InstRef, common: &InstDataCommon, ret: &Ret);
    fn read_jump_inst(&self, inst_ref: InstRef, common: &InstDataCommon, jump: &Jump);
    fn read_br_inst(&self, inst_ref: InstRef, common: &InstDataCommon, br: &Br);
    fn read_switch_inst(&self, inst_ref: InstRef, common: &InstDataCommon, switch: &Switch);
    fn read_tail_call_inst(&self, inst_ref: InstRef, common: &InstDataCommon);
    fn read_load_inst(&self, inst_ref: InstRef, common: &InstDataCommon, load: &LoadOp);
    fn read_store_inst(&self, inst_ref: InstRef, common: &InstDataCommon, store: &StoreOp);
    fn read_select_inst(&self, inst_ref: InstRef, common: &InstDataCommon, select: &SelectOp);
    fn read_bin_op_inst(&self, inst_ref: InstRef, common: &InstDataCommon, bin_op: &BinOp);
    fn read_cmp_inst(&self, inst_ref: InstRef, common: &InstDataCommon, cmp: &CmpOp);
    fn read_cast_inst(&self, inst_ref: InstRef, common: &InstDataCommon, cast: &CastOp);
    fn read_index_ptr_inst(
        &self,
        inst_ref: InstRef,
        common: &InstDataCommon,
        index_ptr: &IndexPtrOp,
    );
    fn read_call_inst(&self, inst_ref: InstRef, common: &InstDataCommon, call: &CallOp);

    fn inst_visitor_dispatch(&self, inst_ref: InstRef, alloc_inst: &Slab<InstData>) {
        let inst_data = inst_ref.to_slabref_unwrap(alloc_inst);
        match inst_data {
            InstData::ListGuideNode(..) => {}
            InstData::PhiInstEnd(..) => self.read_phi_end(inst_ref),
            InstData::Unreachable(c) => self.read_unreachable_inst(inst_ref, c),
            InstData::Ret(c, ret) => self.read_ret_inst(inst_ref, c, ret),
            InstData::Jump(c, jump) => self.read_jump_inst(inst_ref, c, jump),
            InstData::Br(c, br) => self.read_br_inst(inst_ref, c, br),
            InstData::Switch(c, switch) => self.read_switch_inst(inst_ref, c, switch),
            InstData::TailCall(c) => self.read_tail_call_inst(inst_ref, c),
            InstData::Load(c, load) => self.read_load_inst(inst_ref, c, load),
            InstData::Store(c, store) => self.read_store_inst(inst_ref, c, store),
            InstData::Select(c, select) => self.read_select_inst(inst_ref, c, select),
            InstData::BinOp(c, bin_op) => self.read_bin_op_inst(inst_ref, c, bin_op),
            InstData::Cmp(c, cmp) => self.read_cmp_inst(inst_ref, c, cmp),
            InstData::Cast(c, cast) => self.read_cast_inst(inst_ref, c, cast),
            InstData::IndexPtr(c, index_ptr) => self.read_index_ptr_inst(inst_ref, c, index_ptr),
            InstData::Call(c, call) => self.read_call_inst(inst_ref, c, call),
            _ => {}
        }
    }
}

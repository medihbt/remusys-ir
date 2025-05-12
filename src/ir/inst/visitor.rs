use slab::Slab;

use crate::base::slabref::SlabRef;

use super::{
    InstData, InstDataCommon, InstRef,
    phi::PhiOp,
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

    fn inst_visitor_dispatch(&self, inst_ref: InstRef, inst_alloc: &Slab<InstData>) {
        let inst_data = inst_ref.to_slabref_unwrap(inst_alloc);
        match inst_data {
            InstData::ListGuideNode(..) => {}
            InstData::PhiInstEnd(..) => self.read_phi_end(inst_ref),
            InstData::Unreachable(c) => self.read_unreachable_inst(inst_ref, c),
            InstData::Ret(c, ret) => self.read_ret_inst(inst_ref, c, ret),
            InstData::Jump(inst_data_common, jump) => todo!(),
            InstData::Br(inst_data_common, br) => todo!(),
            InstData::Switch(inst_data_common, switch) => todo!(),
            InstData::TailCall(inst_data_common) => todo!(),
            InstData::Phi(inst_data_common, phi_op) => todo!(),
            InstData::Load(inst_data_common, load_op) => todo!(),
            InstData::Store(inst_data_common, store_op) => todo!(),
            InstData::Select(inst_data_common, select_op) => todo!(),
            InstData::BinOp(inst_data_common, bin_op) => todo!(),
            InstData::Cmp(inst_data_common, cmp_op) => todo!(),
            InstData::Cast(inst_data_common, cast_op) => todo!(),
            InstData::IndexPtr(inst_data_common, index_ptr_op) => todo!(),
            InstData::Call(inst_data_common, call_op) => todo!(),
            InstData::DynCall(inst_data_common) => todo!(),
            InstData::Intrin(inst_data_common) => todo!(),
        }
    }
}

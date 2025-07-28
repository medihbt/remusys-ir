use std::collections::VecDeque;

use crate::{
    base::slabref::SlabRef,
    mir::{
        inst::{IMirSubInst, MirInstRef, impls::*, inst::MirInst, opcode::MirOP},
        module::{MirModule, func::MirFunc},
        operand::{IMirSubOperand, reg::*},
    },
};

enum PostLowerAction {
    PushFront(MirInst),
    DeleteThis,
}

fn lower_mir_ldrlit_g64(inst: &MirLdrLitG64, actions: &mut VecDeque<PostLowerAction>) {
    use PostLowerAction::*;

    let dst = GPR64::from_real(inst.get_dst());
    let src = inst.get_src();
    let addr = GPR64::from_real(inst.get_tmp_addr());

    assert!(addr.is_physical());
    assert!(!dst.is_virtual());

    let adr_inst = Adr::new(MirOP::Adr, addr, src);
    actions.push_back(PushFront(adr_inst.into_mir()));

    let ldr_inst = LoadGr64BaseS::new(MirOP::LdrGr64BaseS, dst, addr, src);
    actions.push_back(PushFront(ldr_inst.into_mir()));
    actions.push_back(DeleteThis);
}

fn lower_mir_ldrlit_g32(inst: &MirLdrLitG32, actions: &mut VecDeque<PostLowerAction>) {
    use PostLowerAction::*;

    let dst = GPR32::from_real(inst.get_dst());
    let src = inst.get_src();
    let addr = GPR64::from_real(inst.get_tmp_addr());

    assert!(addr.is_physical());
    assert!(!dst.is_virtual());

    let adr_inst = Adr::new(MirOP::Adr, addr, src);
    actions.push_back(PushFront(adr_inst.into_mir()));
    let ldr_inst = LoadGr32BaseS::new(MirOP::LdrGr32BaseS, dst, addr, src);
    actions.push_back(PushFront(ldr_inst.into_mir()));
    actions.push_back(DeleteThis);
}

fn lower_mir_ldrlit_f64(inst: &MirLdrLitF64, actions: &mut VecDeque<PostLowerAction>) {
    use PostLowerAction::*;

    let dst = FPR64::from_real(inst.get_dst());
    let src = inst.get_src();
    let addr = GPR64::from_real(inst.get_tmp_addr());

    assert!(addr.is_physical());
    assert!(!dst.is_virtual());

    let adr_inst = Adr::new(MirOP::Adr, addr, src);
    actions.push_back(PushFront(adr_inst.into_mir()));
    let ldr_inst = LoadF64BaseS::new(MirOP::LdrF64BaseS, dst, addr, src);
    actions.push_back(PushFront(ldr_inst.into_mir()));
    actions.push_back(DeleteThis);
}

fn lower_mir_ldrlit_f32(inst: &MirLdrLitF32, actions: &mut VecDeque<PostLowerAction>) {
    use PostLowerAction::*;

    let dst = FPR32::from_real(inst.get_dst());
    let src = inst.get_src();
    let addr = GPR64::from_real(inst.get_tmp_addr());

    assert!(addr.is_physical());
    assert!(!dst.is_virtual());

    let adr_inst = Adr::new(MirOP::Adr, addr, src);
    actions.push_back(PushFront(adr_inst.into_mir()));
    let ldr_inst = LoadF32BaseS::new(MirOP::LdrF32BaseS, dst, addr, src);
    actions.push_back(PushFront(ldr_inst.into_mir()));
    actions.push_back(DeleteThis);
}

fn lower_mir_strlit_g64(inst: &MirStrLitG64, actions: &mut VecDeque<PostLowerAction>) {
    use PostLowerAction::*;

    let src = GPR64::from_real(inst.get_rd());
    let dst = inst.get_to();
    let addr = GPR64::from_real(inst.get_tmp_addr());

    assert!(addr.is_physical());
    assert!(!src.is_virtual());

    let adr_inst = Adr::new(MirOP::Adr, addr, dst);
    actions.push_back(PushFront(adr_inst.into_mir()));

    let str_inst = StoreGr64BaseS::new(MirOP::StrGr64BaseS, src, addr, dst);
    actions.push_back(PushFront(str_inst.into_mir()));
    actions.push_back(DeleteThis);
}

fn lower_mir_strlit_g32(inst: &MirStrLitG32, actions: &mut VecDeque<PostLowerAction>) {
    use PostLowerAction::*;

    let src = GPR32::from_real(inst.get_rd());
    let dst = inst.get_to();
    let addr = GPR64::from_real(inst.get_tmp_addr());

    assert!(addr.is_physical());
    assert!(!src.is_virtual());

    let adr_inst = Adr::new(MirOP::Adr, addr, dst);
    actions.push_back(PushFront(adr_inst.into_mir()));
    let str_inst = StoreGr32BaseS::new(MirOP::StrGr32BaseS, src, addr, dst);
    actions.push_back(PushFront(str_inst.into_mir()));
    actions.push_back(DeleteThis);
}

fn lower_mir_strlit_f64(inst: &MirStrLitF64, actions: &mut VecDeque<PostLowerAction>) {
    use PostLowerAction::*;

    let src = FPR64::from_real(inst.get_rd());
    let dst = inst.get_to();
    let addr = GPR64::from_real(inst.get_tmp_addr());

    assert!(addr.is_physical());
    assert!(!src.is_virtual());

    let adr_inst = Adr::new(MirOP::Adr, addr, dst);
    actions.push_back(PushFront(adr_inst.into_mir()));
    let str_inst = StoreF64BaseS::new(MirOP::StrF64BaseS, src, addr, dst);
    actions.push_back(PushFront(str_inst.into_mir()));
    actions.push_back(DeleteThis);
}

fn lower_mir_strlit_f32(inst: &MirStrLitF32, actions: &mut VecDeque<PostLowerAction>) {
    use PostLowerAction::*;

    let src = FPR32::from_real(inst.get_rd());
    let dst = inst.get_to();
    let addr = GPR64::from_real(inst.get_tmp_addr());

    assert!(addr.is_physical());
    assert!(!src.is_virtual());

    let adr_inst = Adr::new(MirOP::Adr, addr, dst);
    actions.push_back(PushFront(adr_inst.into_mir()));
    let str_inst = StoreF32BaseS::new(MirOP::StrF32BaseS, src, addr, dst);
    actions.push_back(PushFront(str_inst.into_mir()));
    actions.push_back(DeleteThis);
}

fn post_lower_an_inst(inst: &MirInst, actions: &mut VecDeque<PostLowerAction>) {
    match inst {
        MirInst::MirLdrLitG64(inst) => lower_mir_ldrlit_g64(inst, actions),
        MirInst::MirLdrLitG32(inst) => lower_mir_ldrlit_g32(inst, actions),
        MirInst::MirLdrLitF64(inst) => lower_mir_ldrlit_f64(inst, actions),
        MirInst::MirLdrLitF32(inst) => lower_mir_ldrlit_f32(inst, actions),
        MirInst::MirStrLitG64(inst) => lower_mir_strlit_g64(inst, actions),
        MirInst::MirStrLitG32(inst) => lower_mir_strlit_g32(inst, actions),
        MirInst::MirStrLitF64(inst) => lower_mir_strlit_f64(inst, actions),
        MirInst::MirStrLitF32(inst) => lower_mir_strlit_f32(inst, actions),
        _ => {}
    }
}

pub(super) fn post_lower_a_function(func: &MirFunc, module: &mut MirModule) {
    let allocs = module.allocs.get_mut();
    let insts = func.dump_insts_when(&allocs.block, &allocs.inst, |inst| {
        matches!(
            inst,
            MirInst::MirLdrLitG64(_)
                | MirInst::MirLdrLitG32(_)
                | MirInst::MirLdrLitF64(_)
                | MirInst::MirLdrLitF32(_)
                | MirInst::MirStrLitG64(_)
                | MirInst::MirStrLitG32(_)
                | MirInst::MirStrLitF64(_)
                | MirInst::MirStrLitF32(_)
        )
    });
    let mut actions = VecDeque::new();
    for (bref, iref) in insts {
        let mut self_deleted = false;
        post_lower_an_inst(iref.to_slabref_unwrap(&allocs.inst), &mut actions);
        while let Some(action) = actions.pop_front() {
            match action {
                PostLowerAction::PushFront(new_inst) => {
                    if self_deleted {
                        panic!("Not implemented: PushFront after DeleteThis");
                    }
                    let new_inst = MirInstRef::from_alloc(&mut allocs.inst, new_inst);
                    bref.get_insts(&allocs.block)
                        .node_add_prev(&allocs.inst, iref, new_inst)
                        .expect("Failed to add new inst");
                }
                PostLowerAction::DeleteThis => {
                    bref.get_insts(&allocs.block)
                        .unplug_node(&allocs.inst, iref)
                        .expect("Failed to unplug old inst");
                    allocs.inst.remove(iref.get_handle());
                    self_deleted = true;
                }
            }
        }
    }
}

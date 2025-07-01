use std::cell::Ref;

use slab::Slab;

use crate::{
    ir::{
        block::BlockRef, inst::{usedef::UseData, InstData, InstRef}, ValueSSA
    },
    mir::{
        inst::{branch::UncondBr, data_process::UnaryOp, opcode::MirOP, MirInst},
        module::block::MirBlockRef,
        operand::{
            reg::{PhysReg, RegUseFlags, SubRegIndex}, symbol::SymbolOperand, MirOperand
        },
        translate::mirgen::{
            data_gen::GlobalDataUnit, func_gen::{IRTrackableValue, SSAValueMap}, FuncTranslator
        },
        util::builder::MirBuilder,
    },
    typing::{context::TypeContext, id::ValTypeID},
};

struct OperandMapping<'a> {
    func_translator: &'a mut FuncTranslator<'a>,
    value_map: &'a SSAValueMap,
    bb_map: &'a [(usize, BlockRef, MirBlockRef)],
}

impl<'a> OperandMapping<'a> {
    fn get_mir_value(&self, operand: &ValueSSA) -> MirOperand {
        type G = GlobalDataUnit;
        let type_ctx = &self.func_translator.ir_module.type_ctx;
        match operand {
            ValueSSA::None => MirOperand::None,
            ValueSSA::ConstData(c) => {
                let data = G::from_const_data(c.clone(), type_ctx);
                let data = match data {
                    G::Bytes(_) | G::Halfs(_) | G::Words(_) | G::Dwords(_) => {
                        panic!("Unexpected data type in MIR translation")
                    }
                    G::Byte(b) => b as i64,
                    G::Half(h) => h as i64,
                    G::Long(l) => l as i64,
                    G::Quad(q) => q as i64,
                };
                MirOperand::ImmConst(data)
            }
            ValueSSA::ConstExpr(_) => panic!("ConstExpr should not be used in MIR translation"),
            ValueSSA::FuncArg(func, index) => {
                let info = self
                    .value_map
                    .find(IRTrackableValue::FuncArg(*func, *index));
                match info {
                    Some(info) => MirOperand::from(info.reg),
                    _ => panic!(
                        "Function argument not found in SSA value map: {:?} at index {}",
                        func, index
                    ),
                }
            }
            ValueSSA::Inst(inst) => {
                let info = self.value_map.find(IRTrackableValue::Inst(*inst));
                match info {
                    Some(info) => MirOperand::from(info.reg),
                    None => panic!("Instruction not found in SSA value map: {:?}", inst),
                }
            }
            ValueSSA::Block(bb) => self
                .bb_map
                .binary_search_by_key(bb, |(_, block_ref, _)| *block_ref)
                .map_or_else(
                    |_| panic!("Block not found in SSA value map: {:?}", bb),
                    |index| MirOperand::Label(self.bb_map[index].2),
                ),
            ValueSSA::Global(gref) => self.func_translator.global_map.get(gref).map_or_else(
                || panic!("Global not found in SSA value map: {:?}", gref),
                |mir_ref| MirOperand::Symbol(SymbolOperand::Global(*mir_ref)),
            ),
        }
    }

    fn get_ret_type(&self) -> ValTypeID {
        self.func_translator.mir_rc.ret_ir_type
    }

    fn borrow_alloc_use(&self) -> Ref<Slab<UseData>> {
        self.func_translator.ir_module.borrow_use_alloc()
    }
    fn mir_build(&mut self, build_fn: impl FnOnce(&mut MirBuilder)) {
        build_fn(self.func_translator.mir_builder);
    }
}

fn inst_dispatch(operand_mapping: &mut OperandMapping, inst_ref: InstRef, inst_data: &InstData) {
    match inst_data {
        InstData::ListGuideNode(..) | InstData::PhiInstEnd(_) | InstData::Phi(..) => {
            // These are not actual instructions, but rather metadata or control flow
            // structures that do not require translation to MIR.
            return;
        }
        InstData::Unreachable(_) => {
            // Unreachable instructions are not translated to MIR.
            return;
        }
        InstData::Ret(_, ret) => {
            if !matches!(operand_mapping.get_ret_type(), ValTypeID::Void) {
                // Instruction has return value
                let retval = ret.retval.get_operand(&operand_mapping.borrow_alloc_use());
                let retval = operand_mapping.get_mir_value(&retval);
                // operand_mapping.mir_builder().add
                let mov_retval_inst = UnaryOp::new(MirOP::Mov, None);
                mov_retval_inst.rd().set(MirOperand::PhysReg(PhysReg::X(
                    0,
                    SubRegIndex::new(6, 0),
                    RegUseFlags::DEF,
                )));
                mov_retval_inst.rhs().set(retval);
                operand_mapping.mir_build(|builder| {
                    builder.add_inst(MirInst::Unary(mov_retval_inst));
                });
            }
            // Add a return instruction to the MIR builder
            operand_mapping.mir_build(|builder| {
                builder.add_inst(MirInst::UncondBr(UncondBr::new(MirOP::Ret)));
            });
        }
        InstData::Jump(c, jump) => todo!(),
        InstData::Br(inst_data_common, br) => todo!(),
        InstData::Switch(inst_data_common, switch) => todo!(),
        InstData::Alloca(inst_data_common, alloca) => todo!(),
        InstData::Load(inst_data_common, load_op) => todo!(),
        InstData::Store(inst_data_common, store_op) => todo!(),
        InstData::Select(inst_data_common, select_op) => todo!(),
        InstData::BinOp(inst_data_common, bin_op) => todo!(),
        InstData::Cmp(inst_data_common, cmp_op) => todo!(),
        InstData::Cast(inst_data_common, cast_op) => todo!(),
        InstData::IndexPtr(inst_data_common, index_ptr_op) => todo!(),
        InstData::Call(inst_data_common, call_op) => todo!(),
        InstData::Intrin(inst_data_common) => todo!(),
    }
}

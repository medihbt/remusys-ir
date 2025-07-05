use std::cell::Ref;

use slab::Slab;

use crate::{
    base::NullableValue,
    ir::{
        ValueSSA,
        block::{BlockRef, jump_target::JumpTargetData},
        global::GlobalRef,
        inst::{InstData, InstRef, gep::IndexChainNode, usedef::UseData},
        opcode::Opcode,
    },
    mir::{
        inst::{MirInst, MirInstRef, branch::UncondBr, call_ret::MirReturn, opcode::MirOP},
        module::block::MirBlockRef,
        operand::{MirOperand, reg::VReg, suboperand::IMirSubOperand},
        translate::mirgen::{
            FuncTranslator,
            data_gen::GlobalDataUnit,
            func_gen::{IRTrackableValue, SSAValueMap},
        },
        util::builder::MirBuilder,
    },
    typing::id::ValTypeID,
};

pub(super) struct OpMap<'a> {
    pub value_map: &'a SSAValueMap,
    pub bb_map: &'a [(usize, BlockRef, MirBlockRef)],
    pub last_pstate_modifier: Option<(InstRef, MirInstRef)>,
}

impl<'a> OpMap<'a> {
    fn get_mir_value(&self, func_translator: &FuncTranslator, operand: &ValueSSA) -> MirOperand {
        type G = GlobalDataUnit;
        let type_ctx = &func_translator.ir_module.type_ctx;
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
                MirOperand::Imm(data)
            }
            ValueSSA::ConstExpr(_) => panic!("ConstExpr should not be used in MIR translation"),
            ValueSSA::FuncArg(func, index) => {
                let info = self
                    .value_map
                    .find(IRTrackableValue::FuncArg(*func, *index));
                match info {
                    Some(info) => info.reg.into_mirop(),
                    _ => panic!(
                        "Function argument not found in SSA value map: {:?} at index {}",
                        func, index
                    ),
                }
            }
            ValueSSA::Inst(inst) => {
                let info = self.value_map.find(IRTrackableValue::Inst(*inst));
                match info {
                    Some(info) => info.reg.into_mirop(),
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
            ValueSSA::Global(gref) => func_translator.global_map.get(gref).map_or_else(
                || panic!("Global not found in SSA value map: {:?}", gref),
                |mir_ref| MirOperand::Global(*mir_ref),
            ),
        }
    }

    fn get_ret_type(&self, func_translator: &mut FuncTranslator) -> ValTypeID {
        func_translator.mir_rc.ret_ir_type
    }

    fn borrow_alloc_use<'b>(func_translator: &'b mut FuncTranslator) -> Ref<'b, Slab<UseData>> {
        func_translator.ir_module.borrow_use_alloc()
    }
    fn borrow_alloc_jt<'b>(
        func_translator: &'b mut FuncTranslator,
    ) -> Ref<'b, Slab<JumpTargetData>> {
        func_translator.ir_module.borrow_jt_alloc()
    }
    fn mir_build(func_translator: &mut FuncTranslator, build_fn: impl FnOnce(&mut MirBuilder)) {
        build_fn(func_translator.mir_builder);
    }
    fn add_inst(func_translator: &mut FuncTranslator, inst: MirInst) -> MirInstRef {
        func_translator.mir_builder.add_inst(inst)
    }

    pub fn inst_dispatch(
        &self,
        func_translator: &mut FuncTranslator,
        inst_ref: InstRef,
        inst_data: &InstData,
    ) {
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
                let mir_return = MirReturn::new(ret.retval.is_nonnull());
                if ret.retval.is_nonnull() {
                    let retval_ir = ret
                        .retval
                        .get_operand(&Self::borrow_alloc_use(func_translator));
                    let retval_mir = self.get_mir_value(&func_translator, &retval_ir);
                    mir_return.retval().unwrap().set(retval_mir);
                }
                Self::add_inst(func_translator, MirInst::MirReturn(mir_return));
            }
            InstData::Jump(c, jump) => {
                let jump_block = jump.get_block(&Self::borrow_alloc_jt(func_translator));
                let jump_label =
                    self.get_mir_value(&*func_translator, &ValueSSA::Block(jump_block));
                let binst = UncondBr::new(MirOP::Branch);
                binst.label().set(jump_label);
                Self::add_inst(func_translator, MirInst::UncondBr(binst));
            }
            InstData::Cmp(c, cmp) => {}
            InstData::Br(_, br) => {
                let cond = br.get_cond(&Self::borrow_alloc_use(func_translator));
                let alloc_jt = Self::borrow_alloc_jt(func_translator);
                let if_true = br.if_true.get_block(&alloc_jt);
                let if_false = br.if_false.get_block(&alloc_jt);
                drop(alloc_jt);

                // Translate the branch instruction to MIR.
                self.translate_branch_inst(func_translator, inst_ref, &cond, if_true, if_false);
            }
            InstData::Switch(_c, _switch) => todo!(
                "Switch instruction translation: Implement this branch if `if-to-switch` pass is implemented"
            ),
            InstData::Alloca(..) => {
                // Alloca instructions are typically handled by the stack layout and do not
                // require a direct translation to MIR.
                return;
            }
            InstData::Load(_, load) => {
                let source_ptr = load
                    .source
                    .get_operand(&Self::borrow_alloc_use(func_translator));
                let mir_dest = self.get_mir_value(&func_translator, &ValueSSA::Inst(inst_ref));
                let mir_dest = match mir_dest {
                    MirOperand::VReg(v) => v,
                    _ => {
                        panic!(
                            "Expected a virtual register for load destination, found: {mir_dest:?}"
                        )
                    }
                };
                self.translate_load_inst(func_translator, inst_ref, &source_ptr, mir_dest);
            }
            InstData::Store(_, store) => {
                let alloc_use = Self::borrow_alloc_use(func_translator);
                let source = store.source.get_operand(&alloc_use);
                let dest_ptr = store.target.get_operand(&alloc_use);
                drop(alloc_use);
                self.translate_store_inst(func_translator, inst_ref, &source, &dest_ptr);
            }
            InstData::Select(_, select) => {
                let alloc_use = Self::borrow_alloc_use(func_translator);
                let cond = select.cond.get_operand(&alloc_use);
                let if_true = select.true_val.get_operand(&alloc_use);
                let if_false = select.false_val.get_operand(&alloc_use);
                drop(alloc_use);
                self.translate_select_inst(func_translator, inst_ref, &cond, &if_true, &if_false);
            }
            InstData::BinOp(c, bin) => {
                let alloc_use = Self::borrow_alloc_use(func_translator);
                let lhs = bin.lhs.get_operand(&alloc_use);
                let rhs = bin.rhs.get_operand(&alloc_use);
                drop(alloc_use);
                self.translate_bin_op_inst(func_translator, inst_ref, c.opcode, &lhs, &rhs);
            }
            InstData::Cast(c, cast) => {
                let alloc_use = Self::borrow_alloc_use(func_translator);
                let source = cast.from_op.get_operand(&alloc_use);
                let target_type = c.ret_type;
                drop(alloc_use);
                self.translate_cast_inst(func_translator, inst_ref, c.opcode, &source, target_type);
            }
            InstData::IndexPtr(_, gep) => {
                let base_ptr = gep
                    .base_ptr
                    .get_operand(&Self::borrow_alloc_use(func_translator));
                let index_chain = gep.dump_index_chain(func_translator.ir_module);
                self.translate_gep_inst(func_translator, inst_ref, &base_ptr, &index_chain);
            }
            InstData::Call(_, call) => {
                let callee = call
                    .callee
                    .get_operand(&Self::borrow_alloc_use(func_translator));
                let args = call
                    .args
                    .iter()
                    .map(|v| v.get_operand(&Self::borrow_alloc_use(func_translator)))
                    .collect::<Vec<_>>();
                if let ValueSSA::Global(gref) = callee {
                    self.translate_call_inst(func_translator, inst_ref, gref, &args);
                } else {
                    panic!("Expected a global reference for call, found: {callee:?}");
                }
            }
            InstData::Intrin(_) => todo!("No intrinsic support in MIR translation yet"),
        }
    }

    /// Translates a binary branch instruction to MIR.
    ///
    /// - is PState modifier: `FALSE`
    /// - is PState reader: `TRUE`
    ///
    /// Possible MIR translation for a branch instruction:
    ///
    /// - When condition operand is a `PState` modifier close to the branch (meaing no PState modifier is on
    ///   the execution path from its operand to the branch):
    ///   - add a `b.<cond>` instruction for true branch
    ///   - add a `b` instruction for false branch
    /// - When condition operand is a `PState` modifier far from the branch:
    ///   - add a condition setting (cond, 1, 0) after the PState modifier instruction
    ///   - add a `cbnz %modifier_result, <if_true>` instruction for the true branch
    ///   - add a `b <if_false>` instruction for the false branch
    /// - When condition operand is a non-PState modifier instruction (returning `i1` condition):
    ///   - add a `cbnz %cond_vreg, <if_true>` instruction for the true branch
    ///   - add a `b <if_false>` instruction for the false branch
    /// - When condition operand is a constant (not really often in O1 optimization -- because SSA should eliminate such cases):
    ///   - add a `b` instruction for the proper branch
    fn translate_branch_inst(
        &self,
        func_translator: &mut FuncTranslator,
        inst_ref: InstRef,
        cond: &ValueSSA,
        if_true: BlockRef,
        if_false: BlockRef,
    ) {
        todo!(
            "Implement branch instruction translation {inst_ref:?}: if {cond:?} then {if_true:?} else {if_false:?}"
        );
    }

    /// Translates a load instruction to MIR.
    ///
    /// - is PState modifier: `FALSE`
    /// - is PState reader: `FALSE`
    ///
    /// Possible MIR translation for a load instruction:
    ///
    /// - Dest operand is a VirtReg representing the load instruction itself.
    /// - If source operand is a register operand: add a `load <source>, <dest>, ?ZR` instruction (LoadStoreRRR, r2 = ZR)
    /// - If source operand is a global: add a `load <label>, <dest>` instruction (LoadStoreLiteral)
    /// - Source operand should not be a constant or a constant expression. If appears, panic with an error.
    fn translate_load_inst(
        &self,
        func_translator: &mut FuncTranslator,
        inst_ref: InstRef,
        source_ptr: &ValueSSA,
        mir_dest: VReg,
    ) {
        todo!(
            "Implement load instruction translation {inst_ref:?}: load {source_ptr:?} to {mir_dest:?}"
        );
    }

    /// Translates a store instruction to MIR.
    ///
    /// - is PState modifier: `FALSE`
    /// - is PState reader: `FALSE`
    ///
    /// Possible MIR translation for a store instruction:
    ///
    /// - MIR store instruction Source operand should be a register operand(VirtReg or PhysReg).
    ///     - If IR store source is a VirtReg | PhysReg, then it is used as a source operand.
    ///     - If IR store source is a constant, add a 'mov <dest>, #const' instruction
    ///     - If IR store source is a global, add a 'mov <dest>, <global>' instruction
    /// - If dest operand is a register: Add a `store <source>, <dest>, ?ZR` instruction (LoadStoreRRR, r2 = ZR)
    /// - If dest operand is a global: Add a `store <source>, <label>` instruction (LoadStoreLiteral)
    fn translate_store_inst(
        &self,
        func_translator: &mut FuncTranslator,
        inst_ref: InstRef,
        source: &ValueSSA,
        dest_ptr: &ValueSSA,
    ) {
        todo!(
            "Implement store instruction translation {inst_ref:?}: store {source:?} to {dest_ptr:?}"
        );
    }

    /// Translate binary select operation to MIR.
    ///
    /// - is PState modifier: `FALSE`
    /// - is PState reader: `TRUE`
    ///
    /// Possible MIR translation for a select instruction:
    ///
    /// (Implement it later)
    fn translate_select_inst(
        &self,
        func_translator: &mut FuncTranslator,
        inst_ref: InstRef,
        cond: &ValueSSA, // type limitation: `i1` as CSR
        if_true: &ValueSSA,
        if_false: &ValueSSA,
    ) {
        todo!(
            "Implement select instruction translation {inst_ref:?}: select if {cond:?} then {if_true:?} else {if_false:?}"
        );
    }

    /// Translate binary operation to MIR.
    fn translate_bin_op_inst(
        &self,
        func_translator: &mut FuncTranslator,
        inst_ref: InstRef,
        opcode: Opcode,
        lhs: &ValueSSA,
        rhs: &ValueSSA,
    ) {
        todo!(
            "Implement binary operation instruction translation {inst_ref:?}: {opcode:?} {lhs:?} {rhs:?}"
        );
    }

    /// Translate cast operation to MIR.
    ///
    /// - is PState modifier: `FALSE`
    /// - is PState reader: `FALSE`
    ///
    /// Possible MIR translation for a cast instruction (by opcode):
    ///
    /// - IR `zext` => AArch64 `UXTB`, `UXTH`, `UXTW` instructions
    /// - IR `sext` => AArch64 `SXTB`, `SXTH`, `SXTW` instructions
    /// - IR `trunc` => AArch64 `BIC` instruction (to clear upper bits)
    /// - IR `bitcast` int to int => AArch64 `MOV` instruction (no-op for integer types)
    /// - IR `bitcast` float to int => AArch64 `MOV` instruction
    /// - IR `ptrtoint` => AArch64 `MOV` instruction (pointer to integer conversion)
    /// - IR `inttoptr` => AArch64 `MOV` instruction (integer to pointer conversion)
    /// - IR `fptosi` float to i32 => AArch64 `FCVTZS` instruction
    /// - IR `sitofp` i32 to float => AArch64 `SCVTF` instruction
    /// - IR `fptoui` float to i32 => AArch64 `FCVTZU` instruction
    /// - IR `uitofp` i32 to float => AArch64 `UCVTF` instruction
    /// - IR `fpext` => AArch64 `FCVT` instruction (to extend float precision)
    /// - IR `fptrunc` => AArch64 `FCVT` instruction (to truncate float precision)
    fn translate_cast_inst(
        &self,
        func_translator: &mut FuncTranslator,
        inst_ref: InstRef,
        opcode: Opcode,
        source: &ValueSSA,
        target_type: ValTypeID,
    ) {
        todo!(
            "Implement cast instruction translation {inst_ref:?}: {opcode:?} {source:?} to {target_type:?}"
        );
    }

    /// translate a `GEP` instruction to MIR.
    ///
    /// - is PState modifier: `FALSE`
    /// - is PState reader: `FALSE`
    ///
    /// Possible MIR translation for a GEP instruction:
    ///
    /// - A series of `add` instructions to calculate the address.
    fn translate_gep_inst(
        &self,
        func_translator: &mut FuncTranslator,
        inst_ref: InstRef,
        base_ptr: &ValueSSA,
        index_chain: &[IndexChainNode],
    ) {
        todo!(
            "Implement GEP instruction translation {:?}: GEP {:?} with indices {:?}",
            inst_ref,
            base_ptr,
            index_chain
        );
    }

    /// Translate a call instruction to MIR.
    ///
    /// - is PState modifier: `TRUE` -- because we cannot guarantee that the callee does not modify PState.
    /// - is PState reader: `TRUE` -- because we cannot guarantee that the callee does not read PState.
    ///
    /// Possible MIR translation for a call instruction:
    ///
    /// - callee should be a global function -- becasuse SysY does not support function pointers or virtual functions.
    /// - add a `call <callee>, <args>` MIR pesudo-instruction.
    /// - if the callee returns a value, it should be stored in a virtual register
    ///   (the return value is not a PState modifier, so it can be used
    ///   as a regular value).
    fn translate_call_inst(
        &self,
        func_translator: &mut FuncTranslator,
        inst_ref: InstRef,
        callee: GlobalRef, // callee function reference
        args: &[ValueSSA], // arguments to the call
    ) {
        todo!(
            "Implement call instruction translation {inst_ref:?}: call {callee:?} with args {:?}",
            args
        );
    }
}

pub(super) fn do_inst_dispatch(
    func_translator: &mut FuncTranslator,
    inst_ref: InstRef,
    inst_data: &InstData,
    value_map: &SSAValueMap,
    bb_map: &[(usize, BlockRef, MirBlockRef)],
    last_pstate_modifier: Option<(InstRef, MirInstRef)>,
) -> Option<(InstRef, MirInstRef)> {
    let inst_builder = OpMap {
        value_map,
        bb_map,
        last_pstate_modifier,
    };
    inst_builder.inst_dispatch(func_translator, inst_ref, inst_data);
    let OpMap {
        value_map: _,
        bb_map: _,
        last_pstate_modifier,
    } = inst_builder;
    last_pstate_modifier
}

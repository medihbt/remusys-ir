use std::{collections::BTreeMap, rc::Rc};

use crate::{
    ir::{
        ValueSSA,
        block::{BlockData, BlockRef},
        constant::data::ConstData,
        global::GlobalRef,
        inst::{InstData, InstRef},
        module::Module,
    },
    mir::{
        inst::{MirInst, data_process, opcode::MirOP},
        module::{
            ModuleItemRef,
            block::{MirBlock, MirBlockRef},
            func::MirFunc,
        },
        operand::{MirOperand, reg::RegOperand, symbol::SymbolOperand},
        translate::ir_pass::phi_node_ellimination::CopyMap,
        util::builder::{MirBuilder, MirFocus},
    },
    opt::analysis::cfg::snapshot::CfgSnapshot,
    typing::{context::TypeContext, id::ValTypeID, types::FloatTypeKind},
};

pub(super) struct FuncTranslator<'a> {
    pub mir_builder: &'a mut MirBuilder<'a>,
    pub ir_module: &'a Module,
    pub ir_ref: GlobalRef,
    pub cfg: &'a CfgSnapshot,
    pub mir_ref: ModuleItemRef,
    pub mir_rc: Rc<MirFunc>,
    pub phi_copies: &'a CopyMap,
    pub global_map: &'a BTreeMap<GlobalRef, ModuleItemRef>,
}

enum IRValueKind {
    Arg,
    SpilledArg,
    VirtReg,
    StackAlloc,
}

#[derive(Debug, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum IRTrackableValue {
    /// `(Function, ArgIndex)`
    FuncArg(GlobalRef, u32),
    Inst(InstRef),
}

impl IRTrackableValue {
    fn from_value(value: &ValueSSA) -> Self {
        match value {
            ValueSSA::Inst(inst_ref) => IRTrackableValue::Inst(*inst_ref),
            ValueSSA::FuncArg(func_ref, arg_index) => {
                IRTrackableValue::FuncArg(*func_ref, *arg_index)
            }
            _ => panic!("Cannot track non-inst or func-arg values"),
        }
    }
    fn to_value(self) -> ValueSSA {
        match self {
            IRTrackableValue::Inst(inst_ref) => ValueSSA::Inst(inst_ref),
            IRTrackableValue::FuncArg(func_ref, arg_index) => {
                ValueSSA::FuncArg(func_ref, arg_index)
            }
        }
    }
}

struct IRValueInfo {
    key: IRTrackableValue,
    ty: ValTypeID,
    kind: IRValueKind,
    reg: RegOperand,
    stackpos: Option<u32>,
}

struct IRValueMap(Vec<IRValueInfo>);

impl IRValueMap {
    fn new() -> Self {
        IRValueMap(Vec::new())
    }
    fn push(&mut self, info: IRValueInfo) {
        self.0.push(info);
    }
    fn finish_construct(&mut self) {
        self.0.sort_by_key(|info| info.key.clone());
    }

    fn find(&self, key: IRTrackableValue) -> Option<&IRValueInfo> {
        self.0
            .binary_search_by_key(&key, |info| info.key.clone())
            .ok()
            .map(|idx| &self.0[idx])
    }
    fn find_by_valssa(&self, value: &ValueSSA) -> Option<&IRValueInfo> {
        self.find(IRTrackableValue::from_value(value))
    }
}

impl<'a> FuncTranslator<'a> {
    pub(super) fn translate(&mut self) {
        // Begin translate
        self.mir_builder
            .set_focus(MirFocus::Func(Rc::clone(&self.mir_rc)));
        // Add all basic blocks quickly
        let mut bb_map = Vec::with_capacity(self.cfg.nodes.len());
        for (i, node) in self.cfg.nodes.iter().enumerate() {
            let mir_bb = MirBlock::new(
                format!(".Lbb{}.{}", i, self.mir_rc.get_name()),
                &mut self.mir_builder.mir_module.borrow_alloc_inst_mut(),
            );
            let mir_bb_ref = MirBlockRef::from_module(self.mir_builder.mir_module, mir_bb);
            self.mir_builder.add_block(mir_bb_ref, false);
            bb_map.push((i, node.block, mir_bb_ref));
        }
        if bb_map.is_sorted_by_key(|(_, bb, _)| bb) {
            bb_map.sort_by_key(|(_, bb, _)| *bb);
        }
        // Allocate virtual registers or make virtual variables for the function.
        let value_map =
            Self::translate_allocate_vregs(&self.mir_rc, self.ir_ref, &bb_map, self.ir_module);

        // Now translate all instructions in the basic blocks.
        for (bb_index, ir_bb_ref, mir_bb_ref) in bb_map.iter() {
            self.translate_one_block(
                *bb_index,
                &self.ir_module.get_block(*ir_bb_ref),
                *mir_bb_ref,
                &value_map,
                &bb_map,
            );
        }
    }

    fn translate_allocate_vregs(
        mir_func: &MirFunc,
        ir_func_ref: GlobalRef,
        bb_map: &[(usize, BlockRef, MirBlockRef)],
        ir_module: &Module,
    ) -> IRValueMap {
        let mut value_map = IRValueMap::new();
        let type_ctx = &ir_module.type_ctx;

        let mut args_count = 0;
        let arg_types = mir_func.arg_ir_types.as_slice();
        for reg in mir_func.arg_regs.iter() {
            value_map.push(IRValueInfo {
                key: IRTrackableValue::FuncArg(ir_func_ref, args_count as u32),
                ty: arg_types[args_count],
                kind: IRValueKind::Arg,
                reg: RegOperand::Phys(*reg),
                stackpos: None,
            });
            args_count += 1;
        }

        for spilled_arg in mir_func.borrow_spilled_args().iter() {
            value_map.push(IRValueInfo {
                key: IRTrackableValue::FuncArg(ir_func_ref, args_count as u32),
                ty: spilled_arg.irtype,
                kind: IRValueKind::SpilledArg,
                reg: RegOperand::Virt(spilled_arg.virtreg),
                // We cannot determine the stack position yet since
                // layout of variables is not finalized.
                stackpos: None,
            });
            args_count += 1;
        }

        for (_, bb_ref, _) in bb_map.iter() {
            Self::translate_allocate_vregs_for_values(
                ir_module,
                *bb_ref,
                mir_func,
                &mut value_map,
                type_ctx,
            );
        }

        value_map.finish_construct();
        value_map
    }

    fn translate_allocate_vregs_for_values(
        ir_module: &Module,
        bb_ref: BlockRef,
        mir_func: &MirFunc,
        value_map: &mut IRValueMap,
        type_ctx: &TypeContext,
    ) {
        let bb_data = ir_module.get_block(bb_ref);
        let alloc_value = ir_module.borrow_value_alloc();
        let alloc_inst = &alloc_value.alloc_inst;
        for (inst_ref, inst) in bb_data.instructions.view(alloc_inst) {
            if !inst.get_value_type().makes_instance()
                || inst.is_terminator()
                || matches!(inst, InstData::PhiInstEnd(..))
            {
                continue;
            }
            let (vreg, is_stack_alloc) = if let InstData::Alloca(_, a) = inst {
                mir_func.add_variable(a.pointee_ty, type_ctx, true)
            } else {
                mir_func.add_variable(inst.get_value_type(), type_ctx, false)
            };
            value_map.push(IRValueInfo {
                key: IRTrackableValue::Inst(inst_ref),
                ty: inst.get_value_type(),
                kind: if is_stack_alloc {
                    IRValueKind::StackAlloc
                } else {
                    IRValueKind::VirtReg
                },
                reg: RegOperand::Virt(vreg),
                stackpos: None,
            });
        }
    }

    fn translate_one_block(
        &mut self,
        bb_index: usize,
        ir_bb_data: &BlockData,
        mir_bb_ref: MirBlockRef,
        value_map: &IRValueMap,
        mir_bb_map: &[(usize, BlockRef, MirBlockRef)],
    ) {
        let bb_ref = self.cfg.nodes[bb_index].block;
        self.mir_builder
            .set_focus(MirFocus::Block(Rc::clone(&self.mir_rc), mir_bb_ref));
        let alloc_value = self.ir_module.borrow_value_alloc();
        let alloc_inst = &alloc_value.alloc_inst;
        for (inst_ref, inst) in ir_bb_data.instructions.view(alloc_inst) {
            if inst.is_terminator() {
                // Insert copy instructions for PHI nodes before terminators.
                self.add_block_phi_copies(bb_ref, value_map, mir_bb_map);
            }
            todo!("Translate instruction: {inst_ref:?} in block {bb_index}");
        }
    }

    fn add_block_phi_copies(
        &mut self,
        ir_block: BlockRef,
        value_map: &IRValueMap,
        block_map: &[(usize, BlockRef, MirBlockRef)],
    ) {
        let phi_copies = self.phi_copies.find_copies(ir_block);
        for copy in phi_copies {
            let source_value = copy.from;
            let target_value = copy.phi.clone();
            let target_value = IRTrackableValue::Inst(target_value.into());

            let target_reg = value_map.find(target_value).unwrap();
            let source_reg = self.translate_operand(value_map, block_map, &source_value);

            let copy_inst = data_process::UnaryOp::new(MirOP::Mov, None);
            MirOperand::set_as_reg(copy_inst.rd(), target_reg.reg.clone());
            copy_inst.rhs().set(source_reg);
            self.mir_builder.add_inst(MirInst::Unary(copy_inst));
        }
    }

    fn translate_operand(
        &self,
        local_map: &IRValueMap,
        block_map: &[(usize, BlockRef, MirBlockRef)],
        ir_value: &ValueSSA,
    ) -> MirOperand {
        match ir_value {
            ValueSSA::None => MirOperand::None,
            ValueSSA::ConstData(data) => self.translate_const_data_operand(data),
            ValueSSA::ConstExpr(_) => {
                panic!("Constant expressions are not supported in MIR translation yet");
            }
            ValueSSA::FuncArg(global_ref, index) => {
                let key = IRTrackableValue::FuncArg(*global_ref, *index);
                if let Some(info) = local_map.find(key.clone()) {
                    MirOperand::from(info.reg)
                } else {
                    panic!("Cannot find function argument: {key:?}");
                }
            }
            ValueSSA::Block(block_ref) => {
                let index = block_map
                    .binary_search_by_key(&block_ref, |(_, bb, _)| bb)
                    .unwrap();
                MirOperand::Label(block_map[index].2)
            }
            ValueSSA::Inst(inst_ref) => {
                if let Some(info) = local_map.find(IRTrackableValue::Inst(*inst_ref)) {
                    MirOperand::from(info.reg)
                } else {
                    panic!("Cannot find instruction value: {inst_ref:?}");
                }
            }
            ValueSSA::Global(global_ref) => {
                if let Some(mir_ref) = self.global_map.get(global_ref) {
                    MirOperand::Symbol(SymbolOperand::Global(*mir_ref))
                } else {
                    panic!("Cannot find global value: {global_ref:?}");
                }
            }
        }
    }
    fn translate_const_data_operand(&self, data: &ConstData) -> MirOperand {
        match data {
            ConstData::Undef(ty) | ConstData::Zero(ty) => {
                Self::make_typed_const_zero(*ty, &self.ir_module.type_ctx)
            }
            ConstData::PtrNull(_) => MirOperand::ImmConst(0),
            ConstData::Int(bit, val) => {
                MirOperand::ImmConst(ConstData::iconst_value_get_real_signed(*bit, *val) as i64)
            }
            ConstData::Float(kind, val) => match kind {
                FloatTypeKind::Ieee32 => MirOperand::ImmConst((*val as f32).to_bits() as i64),
                FloatTypeKind::Ieee64 => MirOperand::ImmConst((*val as f64).to_bits() as i64),
            },
        }
    }
    fn make_typed_const_zero(ir_type: ValTypeID, type_ctx: &TypeContext) -> MirOperand {
        let size = ir_type
            .get_instance_size(type_ctx)
            .expect("Type must have a defined size");
        let align = ir_type
            .get_instance_align(type_ctx)
            .expect("Type must have a defined alignment");
        if !matches!(align, 1 | 2 | 4 | 8) || size > 8 {
            panic!("Size and align cannot meet the requirements: size={size}, align={align}");
        };
        MirOperand::ImmConst(0)
    }
}

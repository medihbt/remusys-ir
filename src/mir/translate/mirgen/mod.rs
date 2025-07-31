use std::{
    collections::{BTreeMap, VecDeque},
    rc::Rc,
};

use log::debug;

use crate::{
    base::{INullableValue, SlabRef},
    ir::{
        block::BlockRef,
        global::GlobalRef,
        inst::{InstData, InstDataKind, InstRef},
        module::Module as IRModule,
        util::numbering::{IRValueNumberMap, NumberOption},
    },
    mir::{
        inst::MirInstRef,
        module::{
            MirModule,
            block::{MirBlock, MirBlockRef},
            func::MirFunc,
        },
        operand::{
            IMirSubOperand,
            reg::{FPR32, FPR64, GPR32, GPR64, RegOperand},
        },
        translate::{
            ir_pass::phi_node_ellimination::CopyMap,
            mirgen::{
                globalgen::{MirGlobalItems, MirGlobalMapFormatter},
                instgen::{InstDispatchError, InstDispatchState, dispatch_inst},
                operandgen::{InstRetval, OperandMap},
            },
        },
        util::builder::{MirBuilder, MirFocus},
    },
    opt::{
        analysis::cfg::{dfs::CfgDfsSeq, snapshot::CfgSnapshot},
        util::DfsOrder,
    },
    typing::{id::ValTypeID, types::FloatTypeKind},
};

pub mod datagen;
pub mod globalgen;
pub mod instgen;
pub mod operandgen;

pub fn codegen_ir_to_mir(
    ir_module: Rc<IRModule>,
    copy_map: CopyMap,
    mut cfgs: Vec<CfgSnapshot>,
) -> MirModule {
    // `cfgs` is a map from function reference to CFG snapshot.
    if !cfgs.is_sorted_by_key(|cfg| cfg.func) {
        cfgs.sort_by_key(|cfg| cfg.func);
    }
    MirTranslateCtx::new(ir_module.clone(), copy_map).do_translate(&cfgs)
}

pub struct MirTranslateCtx {
    ir_module: Rc<IRModule>,
    mir_module: MirModule,
    copy_map: CopyMap,
    number_maps: BTreeMap<GlobalRef, IRValueNumberMap>,
}

pub struct MirBlockInfo {
    pub ir: BlockRef,
    pub mir: MirBlockRef,
    pub insts: Vec<InstTranslateInfo>,
}

#[derive(Debug, Clone, Copy)]
pub struct InstTranslateInfo {
    pub ir: InstRef,
    pub ty: ValTypeID,
    pub kind: InstDataKind,
}

#[derive(Debug, Clone, Copy)]
struct AllocaInfo {
    pub ir: InstRef,
    pub _align_log2: u8,
    pub pointee_ty: ValTypeID,
}

impl MirTranslateCtx {
    fn new(ir_module: Rc<IRModule>, copy_map: CopyMap) -> Self {
        let name = ir_module.name.clone();
        Self {
            ir_module,
            mir_module: MirModule::new(name),
            copy_map,
            number_maps: BTreeMap::new(),
        }
    }

    /// Translate IR module to MIR module.
    fn do_translate(mut self, cfgs: &[CfgSnapshot]) -> MirModule {
        let mut builder = MirBuilder::new(&mut self.mir_module);
        // Step 0: 为每个 MIR 全局量分配位置, 其中全局变量和外部量会立即初始化
        let globals = MirGlobalItems::build_mir(&self.ir_module, &mut builder);

        debug!(
            "{:#?}",
            MirGlobalMapFormatter::new(&globals, &self.ir_module)
        );

        // Step 1: 翻译每个函数的 CFG
        for cfg in cfgs {
            let ir_func_ref = cfg.func;
            let mir_func_info = globals
                .find_func(ir_func_ref)
                .expect("MIR function info not found for IR function reference");
            let numbers = IRValueNumberMap::from_func(
                &self.ir_module,
                ir_func_ref,
                NumberOption::ignore_all(),
            );
            self.number_maps.insert(ir_func_ref, numbers);
            let mir_func = Rc::clone(&mir_func_info.rc);
            self.do_translate_function(cfg, &globals, &mir_func);
        }

        self.mir_module
    }

    fn do_translate_function(
        &mut self,
        cfg: &CfgSnapshot,
        globals: &MirGlobalItems,
        mir_func: &Rc<MirFunc>,
    ) {
        // Step 1.1 为每个函数的基本块分配 MIR 块引用
        let dfs_seq = CfgDfsSeq::new_from_snapshot(cfg, DfsOrder::Pre);
        let mut block_map = self.alloc_mirref_for_blocks(Rc::clone(mir_func), dfs_seq);

        // Step 1.2 处理每个基本块的前驱和后继关系
        for &MirBlockInfo { ir, mir, .. } in &block_map {
            self.complete_mir_block_succ_and_pred(cfg, ir, mir, &block_map);
        }

        // Step 1.3 dump 出所有指令, 并为函数确定参数布局和栈布局
        let allocas = self.dump_insts_and_layout(&mut block_map);
        // Step 1.4 为每个指令分配虚拟寄存器
        let vregs = {
            self.ir_module
                .enable_rdfg()
                .expect("RDFG must be enabled to allocate storage for MIR instructions");
            self.allocate_storage_for_insts(&mut block_map, &allocas, mir_func)
        };

        // Step 1.5 翻译每个基本块的指令
        let (operand_map, inst_template) =
            OperandMap::build_from_func(Rc::clone(mir_func), globals, vregs.clone(), block_map);
        let entry_block = operand_map.blocks[0].mir;
        for inst in inst_template {
            let mut mir_builder = MirBuilder::new(&mut self.mir_module);
            mir_builder.set_focus(MirFocus::Block(Rc::clone(mir_func), entry_block));
            mir_builder.add_inst(inst);
        }
        for i in 0..operand_map.blocks.len() {
            self.inst_dispatch_for_one_mir_block(&operand_map, i);
        }
    }

    /// Step 1.1: 为每个函数的基本块分配 MIR 块引用
    fn alloc_mirref_for_blocks(&mut self, func: Rc<MirFunc>, dfs: CfgDfsSeq) -> Vec<MirBlockInfo> {
        let mut block_map = Vec::with_capacity(dfs.nodes.len());
        for (idx, node) in dfs.nodes.iter().enumerate() {
            let bb = node.block;
            let mir_bb = MirBlock::new(
                if idx == 0 {
                    func.get_name().to_string()
                } else {
                    format!(".LBB.{}.{:02}", func.get_name(), idx)
                },
                &mut self.mir_module.borrow_alloc_inst_mut(),
            );
            let mir_bb_ref = MirBlockRef::from_module(&self.mir_module, mir_bb);
            block_map.push(MirBlockInfo { ir: bb, mir: mir_bb_ref, insts: Vec::new() });
            let mut mir_builder = MirBuilder::new(&mut self.mir_module);
            mir_builder.set_focus(MirFocus::Func(Rc::clone(&func)));
            mir_builder.add_block(mir_bb_ref, false);
        }
        block_map.sort_by_key(|info| info.ir);
        block_map
    }

    /// Step 1.2: 设置 MIR 基本块的后继和前驱关系
    fn complete_mir_block_succ_and_pred(
        &self,
        cfg: &CfgSnapshot,
        ir_bb: BlockRef,
        mir_bb: MirBlockRef,
        block_map: &[MirBlockInfo],
    ) {
        let node = cfg
            .block_get_node(ir_bb)
            .expect("Block node not found in CFG");
        let mut mir_bb_alloc = self.mir_module.borrow_alloc_block_mut();
        let mir_bb_data = mir_bb.to_data_mut(&mut mir_bb_alloc);

        // 设置 MIR 基本块的后继
        for &(_, succ) in node.next_set.iter() {
            let mir_succ = match block_map.binary_search_by_key(&succ, |info| info.ir) {
                Ok(idx) => idx,
                Err(_) => {
                    let numbers = self
                        .number_maps
                        .get(&cfg.func)
                        .expect("Number map not found for function");
                    let number = numbers
                        .block_get_number(succ)
                        .expect("Block number not found");
                    let cfg_func = cfg.func;
                    panic!(
                        "Successor block {succ:?} (%{number}) not found in block map for function {cfg_func:?}"
                    );
                }
            };
            let mir_succ = block_map[mir_succ].mir;
            mir_bb_data.successors.insert(mir_succ);
        }

        // 设置 MIR 基本块的前驱
        for &(_, pred) in node.prev_set.iter() {
            let mir_pred = match block_map.binary_search_by_key(&pred, |info| info.ir) {
                Ok(idx) => idx,
                Err(_) => {
                    let numbers = self
                        .number_maps
                        .get(&cfg.func)
                        .expect("Number map not found for function");
                    let number = numbers
                        .block_get_number(pred)
                        .expect("Block number not found");
                    let func = cfg.func;
                    println!("Found unreachable block {pred:?} (%{number}) for function {func:?}");
                    continue;
                }
            };
            let mir_pred = block_map[mir_pred].mir;
            mir_bb_data.predecessors.insert(mir_pred);
        }
    }

    /// Step 1.3: Dump 出所有指令, 并为函数确定参数布局和栈布局
    fn dump_insts_and_layout(&mut self, block_map: &mut [MirBlockInfo]) -> Vec<AllocaInfo> {
        let mut allocas = Vec::new();
        for MirBlockInfo { ir, insts, .. } in block_map {
            let (insts_in_block, len) = self
                .ir_module
                .get_block(*ir)
                .instructions
                .load_range_and_length();
            insts.reserve(len);

            let alloc_value = self.ir_module.borrow_value_alloc();
            let alloc_inst = &alloc_value.alloc_inst;

            for (ir, inst) in insts_in_block.view(alloc_inst) {
                if let InstData::Alloca(_, a) = inst {
                    allocas.push(AllocaInfo {
                        ir,
                        _align_log2: a.align_log2,
                        pointee_ty: a.pointee_ty,
                    });
                }
                let ty = inst.get_value_type();
                let kind = inst.get_kind();
                type K = InstDataKind;
                if matches!(kind, K::ListGuideNode | K::PhiInstEnd | K::Intrin) {
                    continue; // 跳过不需要翻译的指令
                }
                insts.push(InstTranslateInfo { ir, ty, kind });
            }
        }
        allocas
    }

    /// 为指令分配虚拟寄存器.
    fn allocate_storage_for_insts(
        &mut self,
        block_map: &mut [MirBlockInfo],
        allocas: &[AllocaInfo],
        func: &MirFunc,
    ) -> Vec<(InstRef, InstRetval)> {
        let mut vregs = Vec::new();
        let type_ctx = &self.ir_module.type_ctx;

        // 为所有 alloca 分配表示栈位置的虚拟寄存器.
        for alloca_info in allocas {
            let vreg = func.add_spilled_variable(alloca_info.pointee_ty, type_ctx);
            vregs.push((alloca_info.ir, InstRetval::Reg(vreg.into())));
        }

        // Remusys-IR 的指令本身也表示它的返回值操作数, 因此为每个有返回值的指令分配一个虚拟寄存器.
        for MirBlockInfo { insts, .. } in block_map {
            for InstTranslateInfo { ir, ty, kind } in insts {
                type K = InstDataKind;
                debug!("Translating instruction {ir:?} with type {ty:?} and kind {kind:?}");

                if !self.ir_module.inst_has_user(*ir).unwrap_or(false) {
                    // 如果指令没有用户, 则分配寄存器或存储空间, 而是把它标记为 "wasted".
                    vregs.push((*ir, InstRetval::Wasted));
                    continue;
                }

                match kind {
                    K::ListGuideNode | K::PhiInstEnd | K::Unreachable => {
                        // 功能结点不需要分配寄存器或存储空间
                        continue;
                    }
                    K::Ret | K::Jump | K::Br | K::Store | K::Switch => {
                        // 基本块终止指令不需要分配寄存器或存储空间
                        continue;
                    }
                    InstDataKind::Cmp => continue, // 比较指令不需要分配寄存器或存储空间
                    InstDataKind::Alloca => continue, // 分配指令已经在 `allocas` 中处理

                    _ => {}
                }
                let mut inner = func.borrow_inner_mut();
                let alloc_reg = &mut inner.vreg_alloc;
                let vreg = match ty {
                    ValTypeID::Void => {
                        // Void 类型的指令不需要分配寄存器
                        continue;
                    }
                    ValTypeID::Int(32) => {
                        let to_insert = GPR32::new_empty();
                        let vreg = alloc_reg.insert_gp(to_insert.into_real());
                        RegOperand::from(vreg)
                    }
                    ValTypeID::Ptr | ValTypeID::Int(_) => {
                        let to_insert = GPR64::new_empty();
                        let vreg = alloc_reg.insert_gp(to_insert.into_real());
                        RegOperand::from(vreg)
                    }
                    ValTypeID::Float(FloatTypeKind::Ieee32) => {
                        let to_insert = FPR32::new_empty();
                        let vreg = alloc_reg.insert_float(to_insert.into_real());
                        RegOperand::from(vreg)
                    }
                    ValTypeID::Float(FloatTypeKind::Ieee64) => {
                        let to_insert = FPR64::new_empty();
                        let vreg = alloc_reg.insert_float(to_insert.into_real());
                        RegOperand::from(vreg)
                    }
                    _ => panic!("Unsupported type for MIR instruction: {ty:?}"),
                };
                vregs.push((*ir, InstRetval::Reg(vreg.into())));
            }
        }
        vregs.sort_by_key(|(k, _)| *k);
        vregs
    }

    /// Step 1.5: 翻译每个基本块的指令
    fn inst_dispatch_for_one_mir_block(&mut self, operand_map: &OperandMap, block_index: usize) {
        let block_info = &operand_map.blocks[block_index];
        let ir_block = block_info.ir;
        let mir_block = block_info.mir;

        let mut mir_builder = MirBuilder::new(&mut self.mir_module);
        mir_builder.set_focus(MirFocus::Block(Rc::clone(&operand_map.func), mir_block));

        // Step .1: 生成基本块的 MIR
        let mut state = InstDispatchState::new();
        let mut inst_queue = VecDeque::with_capacity(block_info.insts.len());
        for ir_inst in &block_info.insts {
            let mut func_inner = operand_map.func.borrow_inner_mut();
            let vreg_alloc = &mut func_inner.vreg_alloc;
            match dispatch_inst(
                &self.ir_module,
                &mut state,
                *ir_inst,
                operand_map,
                vreg_alloc,
                &mut inst_queue,
            ) {
                Ok(()) => {
                    let mut inst_ref = MirInstRef::new_null();
                    while let Some(inst) = inst_queue.pop_front() {
                        inst_ref = mir_builder.add_inst(inst);
                    }
                    if state.pstate_modifier_matches(ir_inst.ir) {
                        // 如果当前指令修改了 PState, 则更新状态
                        state.last_pstate_modifier = Some((ir_inst.ir, inst_ref));
                    }
                    if state.has_call {
                        // 如果当前指令包含调用, 则设置函数的 has_call 标志
                        operand_map.func.has_call.set(true);
                    }
                }
                Err(InstDispatchError::ShouldNotTranslate(..)) => {}
                Err(e) => panic!("Instruction dispatching error {e:?}"),
            }
            state.inst_level_reset();
        }

        // Step .2: 为每个 Phi 添加拷贝函数.
        for phi_copy in self.copy_map.find_copies(ir_block) {
            let phi_copy = phi_copy.clone();
            let phi_reg = operand_map
                .find_operand_for_inst(phi_copy.phi.into())
                .expect("Phi register not found");
            let InstRetval::Reg(phi_reg) = phi_reg else {
                panic!("Expected a register for phi copy, found: {phi_reg:?}");
            };
            let from_val = operand_map
                .find_operand_no_constdata(&phi_copy.from)
                .expect("From value not found");
            instgen::make_copy_inst(phi_reg, from_val, &mut inst_queue);
            while let Some(inst) = inst_queue.pop_front() {
                mir_builder.add_inst(inst);
            }
        }
    }
}

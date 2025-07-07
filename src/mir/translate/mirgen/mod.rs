use std::{cell::Ref, rc::Rc};

use slab::Slab;

use crate::{
    base::slabref::SlabRef,
    ir::{
        block::{self, BlockRef},
        global::{GlobalData, GlobalRef},
        inst::{InstData, InstDataKind, InstRef},
        module::Module as IRModule,
    },
    mir::{
        module::{
            MirGlobalRef, MirModule,
            block::{MirBlock, MirBlockRef},
            func::MirFunc,
        },
        translate::{ir_pass::phi_node_ellimination::CopyMap, mirgen::globalgen::MirGlobalItems},
        util::builder::{MirBuilder, MirFocus},
    },
    opt::{
        analysis::cfg::{dfs::CfgDfsSeq, snapshot::CfgSnapshot},
        util::DfsOrder,
    },
    typing::id::ValTypeID,
};

mod constgen;
mod globalgen;
mod instgen;
mod operandgen;

pub(super) fn codegen_ir_to_mir(
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

struct MirTranslateCtx {
    ir_module: Rc<IRModule>,
    mir_module: MirModule,
    copy_map: CopyMap,
}

struct MirBlockInfo {
    pub ir:  BlockRef,
    pub mir: MirBlockRef,
}

struct InstTranslateInfo {
    pub ir: InstRef,
    pub ty: ValTypeID,
    pub kind: InstDataKind,
}

impl MirTranslateCtx {
    fn new(ir_module: Rc<IRModule>, copy_map: CopyMap) -> Self {
        let name = ir_module.name.clone();
        Self {
            ir_module,
            mir_module: MirModule::new(name),
            copy_map,
        }
    }

    /// Translate IR module to MIR module.
    fn do_translate(mut self, cfgs: &[CfgSnapshot]) -> MirModule {
        let mut builder = MirBuilder::new(&mut self.mir_module);
        // Step 0: 为每个 MIR 全局量分配位置, 其中全局变量和外部量会立即初始化
        let globals = MirGlobalItems::build_mir(&self.ir_module, &mut builder);

        // Step 1: 翻译每个函数的 CFG
        for cfg in cfgs {
            let ir_func_ref = cfg.func;
            let mir_func_info = globals
                .find_func(ir_func_ref)
                .expect("MIR function info not found for IR function reference");
            let mir_func = Rc::clone(&mir_func_info.rc);
            self.do_translate_function(cfg, &globals, ir_func_ref, &mir_func);
        }

        self.mir_module
    }

    fn do_translate_function(
        &mut self,
        cfg: &CfgSnapshot,
        globals: &MirGlobalItems,
        ir_func: GlobalRef,
        mir_func: &Rc<MirFunc>,
    ) {
        // Step 1.1 为每个函数的基本块分配 MIR 块引用
        let dfs_seq = CfgDfsSeq::new_from_snapshot(cfg, DfsOrder::Pre);
        let block_map = self.alloc_mirref_for_blocks(Rc::clone(mir_func), dfs_seq);

        // Step 1.2 处理每个基本块的前驱和后继关系
        for &MirBlockInfo { ir, mir } in &block_map {
            self.complete_mir_block_succ_and_pred(cfg, ir, mir, &block_map);
        }

        // Step 1.3 为函数确定参数布局和栈布局
        let mut allocas = Vec::new();
        let mut all_insts = Vec::new();

        todo!("build a block-to-inst mapping for MIR translation");

        for &MirBlockInfo { ir, mir: _ } in &block_map {
            let (insts, len) = self
                .ir_module
                .get_block(ir)
                .instructions
                .load_range_and_length();
            all_insts.reserve(len);
            let alloc_value = self.ir_module.borrow_value_alloc();
            let alloc_inst = &alloc_value.alloc_inst;
            for (ir, inst) in insts.view(alloc_inst) {
                if let InstData::Alloca(_, a) = inst {
                    allocas.push((ir, a.align_log2, a.pointee_ty));
                }
                let ty = inst.get_value_type();
                let kind = inst.get_kind();
                type K = InstDataKind;
                if matches!(kind, K::ListGuideNode | K::PhiInstEnd | K::Intrin) {
                    continue; // 跳过不需要翻译的指令
                }
                all_insts.push(InstTranslateInfo { ir, ty, kind });
            }
        }

        todo!("handle allocas for MIR function stack layout");

        // Step 1.? 翻译每个基本块的指令
        for &MirBlockInfo { ir, mir } in &block_map {
            self.inst_dispatch_for_one_mir_block(ir, mir, mir_func, globals, &block_map);
        }
    }

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
            block_map.push(MirBlockInfo {
                ir: bb,
                mir: mir_bb_ref,
            });
            let mut mir_builder = MirBuilder::new(&mut self.mir_module);
            mir_builder.set_focus(MirFocus::Func(Rc::clone(&func)));
            mir_builder.add_block(mir_bb_ref, false);
        }
        block_map.sort_by_key(|info| info.ir);
        block_map
    }

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
        let mir_bb_data = mir_bb.to_slabref_unwrap_mut(&mut mir_bb_alloc);

        // 设置 MIR 基本块的后继
        for &(_, succ) in node.next_set.iter() {
            let mir_succ = block_map
                .binary_search_by_key(&succ, |info| info.ir)
                .expect("Successor block not found in block map");
            let mir_succ = block_map[mir_succ].mir;
            mir_bb_data.successors.insert(mir_succ);
        }

        // 设置 MIR 基本块的前驱
        for &(_, pred) in node.prev_set.iter() {
            let mir_pred = block_map
                .binary_search_by_key(&pred, |info| info.ir)
                .expect("Predecessor block not found in block map");
            let mir_pred = block_map[mir_pred].mir;
            mir_bb_data.predecessors.insert(mir_pred);
        }
    }

    fn inst_dispatch_for_one_mir_block(
        &mut self,
        ir_bb: BlockRef,
        mir_bb: MirBlockRef,
        mir_func: &Rc<MirFunc>,
        globals: &MirGlobalItems,
        blocks: &[MirBlockInfo],
    ) {
        // Step 0:
    }
}

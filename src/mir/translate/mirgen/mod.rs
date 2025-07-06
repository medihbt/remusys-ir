use std::{cell::Ref, rc::Rc};

use slab::Slab;

use crate::{
    ir::{global::GlobalData, inst::InstData, module::Module as IRModule},
    mir::{module::MirModule, translate::ir_pass::phi_node_ellimination::CopyMap},
    opt::analysis::cfg::snapshot::CfgSnapshot,
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
    let mut ctx = MirTranslateCtx::new(ir_module.clone(), copy_map, cfgs);
    ctx.do_translate();
    ctx.mir_module
}

struct MirTranslateCtx {
    ir_module: Rc<IRModule>,
    mir_module: MirModule,
    copy_map: CopyMap,
    cfgs: Vec<CfgSnapshot>,
}

impl MirTranslateCtx {
    fn new(ir_module: Rc<IRModule>, copy_map: CopyMap, cfgs: Vec<CfgSnapshot>) -> Self {
        let name = ir_module.name.clone();
        Self {
            ir_module,
            mir_module: MirModule::new(name),
            copy_map,
            cfgs,
        }
    }

    fn borrow_ir_inst_alloc(&self) -> Ref<Slab<InstData>> {
        Ref::map(self.ir_module.borrow_value_alloc(), |a| &a.alloc_inst)
    }
    fn borrow_ir_global_alloc(&self) -> Ref<Slab<GlobalData>> {
        Ref::map(self.ir_module.borrow_value_alloc(), |a| &a.alloc_global)
    }

    fn do_translate(&mut self) {
        todo!(
            "Implement MIR translation for IR module: {}",
            self.ir_module.name
        );
    }
}

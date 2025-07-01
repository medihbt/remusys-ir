use crate::{
    ir::module::Module as IRModule,
    mir::{
        module::MirModule,
        translate::{
            ir_pass::phi_node_ellimination::CopyMap,
            mirgen::{
                func_gen::FuncTranslator,
                global_gen::{GlobalStatistics, IrMirGlobalInfo},
            },
        },
        util::builder::{MirBuilder, MirFocus},
    },
    opt::analysis::cfg::snapshot::CfgSnapshot,
};

mod data_gen;
mod func_gen;
mod global_gen;
mod inst_dispatch;

pub(super) fn codegen_ir_to_mir(
    ir_module: &IRModule,
    copy_map: &CopyMap,
    cfgs: &[CfgSnapshot],
) -> MirModule {
    let mut mir_module = MirModule::new(ir_module.name.clone());
    let mut builder = MirBuilder::new(&mut mir_module);
    let all_globals = GlobalStatistics::new(ir_module);

    let IrMirGlobalInfo { mapping, funcs } = all_globals.make_global_items(&mut builder, ir_module);
    drop(builder);

    // Now start translating IR functions to MIR functions.
    for (fref, mref, mfunc) in funcs {
        let mut builder = MirBuilder::new(&mut mir_module);
        builder.set_focus(MirFocus::Func(mfunc.clone()));
        let mut func_translator = FuncTranslator {
            mir_builder: &mut builder,
            ir_module,
            ir_ref: fref,
            cfg: cfgs.iter().find(|cfg| cfg.func == fref).unwrap(),
            mir_ref: mref,
            mir_rc: mfunc,
            phi_copies: copy_map,
            global_map: &mapping,
        };
        func_translator.translate();
    }
    mir_module
}

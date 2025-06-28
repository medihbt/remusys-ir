use std::{collections::BTreeMap, rc::Rc};

use slab::Slab;

use crate::{
    base::slabref::SlabRef,
    ir::{
        global::{GlobalData, GlobalRef},
        module::Module as IRModule,
    },
    mir::{
        module::{MirModule, ModuleItemRef, global::Section},
        translate::{
            ir_pass::phi_node_ellimination::CopyMap, mirgen::global_statistics::GlobalStatistics,
        },
        util::builder::MirBuilder,
    },
    opt::analysis::cfg::snapshot::CfgSnapshot,
    typing::context::TypeContext,
};

mod data_gen;
mod global_statistics;

pub(super) fn codegen_ir_to_mir(
    ir_module: &IRModule,
    copy_map: &CopyMap,
    cfgs: &[CfgSnapshot],
) -> MirModule {
    let mut mir_module = MirModule::new(ir_module.name.clone());
    let mut builder = MirBuilder::new(&mut mir_module);
    let all_globals = GlobalStatistics::new(ir_module);

    let type_ctx = Rc::clone(&ir_module.type_ctx);

    // translate extern variables
    let alloc_value = ir_module.borrow_value_alloc();
    let alloc_global = &alloc_value.alloc_global;

    let mut symbol_map =
        translate_global::insert_globals(&mut builder, alloc_global, &all_globals, &type_ctx);

    // Now start translating IR functions to MIR functions.
    for cfg in cfgs {
        let ir_func = cfg.func;
        let mir_func = symbol_map
            .get(&ir_func)
            .expect("Function should have been registered in symbol map");
        todo!(
            "Translate IR function to MIR function: {}",
            cfg.func.get_name_with_module(ir_module)
        );
    }
    mir_module
}

mod translate_global {
    use super::*;

    pub(super) fn insert_globals(
        builder: &mut MirBuilder,
        alloc_global: &Slab<GlobalData>,
        all_globals: &GlobalStatistics,
        type_ctx: &TypeContext,
    ) -> BTreeMap<GlobalRef, ModuleItemRef> {
        let mut symbol_map = BTreeMap::new();

        // Insert extern variables.
        for &xvar in all_globals.extern_vars() {
            insert_extern_variable(builder, &mut symbol_map, alloc_global, xvar, type_ctx);
        }

        // Insert extern functions.
        for &xvar in all_globals.extern_funcs() {
            todo!(
                "Insert extern function: {}",
                xvar.to_slabref_unwrap(alloc_global).get_name()
            );
        }

        // Insert functions.
        for &xvar in all_globals.funcs() {
            todo!(
                "Insert function: {}",
                xvar.to_slabref_unwrap(alloc_global).get_name()
            );
        }

        // Insert global constants.
        for &gconst in all_globals.global_consts() {
            todo!(
                "Insert global constant: {}",
                gconst.to_slabref_unwrap(alloc_global).get_name()
            );
        }

        // Insert global variables.
        for &gvar in all_globals.global_vars() {
            todo!(
                "Insert global variable: {}",
                gvar.to_slabref_unwrap(alloc_global).get_name()
            );
        }

        // Insert global zero-initialized variables.
        for &zvar in all_globals.global_zero_inits() {
            todo!(
                "Insert global zero-initialized variable: {}",
                zvar.to_slabref_unwrap(alloc_global).get_name()
            );
        }

        symbol_map
    }

    fn insert_extern_variable(
        builder: &mut MirBuilder,
        symbol_map: &mut BTreeMap<GlobalRef, ModuleItemRef>,
        alloc_global: &Slab<GlobalData>,
        xvar: GlobalRef,
        type_ctx: &TypeContext,
    ) {
        let gvar = match xvar.to_slabref_unwrap(alloc_global) {
            GlobalData::Var(var) => var,
            _ => panic!("Expected an extern variable, found: {:?}", xvar),
        };
        let commmon = &gvar.common;
        let name = commmon.name.clone();
        let ty = commmon.content_ty;
        let section = if gvar.is_readonly() {
            Section::RoData
        } else {
            Section::Data
        };
        let (mir_sym, _) = builder.extern_variable(name, section, ty, type_ctx);
        symbol_map.insert(xvar, mir_sym);
    }
}

use std::{fmt::Debug, ops::ControlFlow, rc::Rc};

use log::debug;

use crate::{
    base::SlabRef,
    ir::{
        FuncStorage, GlobalData, GlobalKind, GlobalRef, ISubGlobal, ISubValueSSA,
        Module as IRModule, PtrStorage, ValueSSA,
    },
    mir::{
        module::{
            MirGlobalRef,
            func::MirFunc,
            global::{MirGlobalData, MirGlobalVariable, Section},
        },
        translate::mirgen::datagen::DataGen,
        util::builder::MirBuilder,
    },
    typing::IValType,
};

#[derive(Debug, Clone)]
struct GlobalStatistics {
    all_globals: Box<[GlobalRef]>,
    /// order:
    ///
    /// - extern global variables (section: GOT or PLT)
    /// - extern functions (section: GOT or PLT)
    /// - functions (section: Text)
    /// - global constants (section: ROData)
    /// - global variables (section: Data)
    /// - global variables with zero initializeer (section: BSS)
    extern_func_off: u32,
    func_off: u32,
    global_const_off: u32,
    global_var_off: u32,
    global_zero_init_off: u32,
}

impl GlobalStatistics {
    pub(super) fn extern_vars(&self) -> &[GlobalRef] {
        &self.all_globals[..self.extern_func_off as usize]
    }
    pub(super) fn extern_funcs(&self) -> &[GlobalRef] {
        &self.all_globals[self.extern_func_off as usize..self.func_off as usize]
    }
    pub(super) fn funcs(&self) -> &[GlobalRef] {
        &self.all_globals[self.func_off as usize..self.global_const_off as usize]
    }
    pub(super) fn global_consts(&self) -> &[GlobalRef] {
        &self.all_globals[self.global_const_off as usize..self.global_var_off as usize]
    }
    pub(super) fn global_vars(&self) -> &[GlobalRef] {
        &self.all_globals[self.global_var_off as usize..self.global_zero_init_off as usize]
    }
    pub(super) fn global_zero_inits(&self) -> &[GlobalRef] {
        &self.all_globals[self.global_zero_init_off as usize..]
    }

    pub(super) fn new(ir_module: &IRModule) -> Self {
        let mut extern_vars = Vec::new();
        let mut extern_funcs = Vec::new();
        let mut funcs = Vec::new();
        let mut global_consts = Vec::new();
        let mut global_vars = Vec::new();
        let mut global_zero_inits = Vec::new();

        ir_module.forall_globals(true, |gref, gdata| {
            let name = gdata.get_name();
            match gdata {
                GlobalData::Var(var) => match var.get_kind() {
                    GlobalKind::ExternVar | GlobalKind::ExternConst => {
                        debug!("Discovered extern constant | var: {gref:?} name {name}");
                        extern_vars.push(gref);
                    }
                    GlobalKind::Var => {
                        let init = var.get_init();
                        assert!(
                            init != ValueSSA::None,
                            "Global variable {gref:?} has no initializer"
                        );
                        if init.is_zero(&ir_module.allocs) {
                            debug!("Discovered global zero-init variable: {gref:?} name {name}",);
                            global_zero_inits.push(gref);
                        } else {
                            debug!("Discovered global variable: {gref:?} name {name}");
                            global_vars.push(gref);
                        }
                    }
                    GlobalKind::Const => {
                        debug!("Discovered global constant: {gref:?} name {name}");
                        global_consts.push(gref);
                    }
                    _ => unreachable!("Unexpected global kind: {:?}", var.get_kind()),
                },
                GlobalData::Func(func) => match func.get_kind() {
                    GlobalKind::ExternFunc => {
                        debug!("Discovered extern function: {gref:?} name {name}");
                        extern_funcs.push(gref);
                    }
                    GlobalKind::Func => {
                        debug!("Discovered function: {gref:?} name {name}");
                        funcs.push(gref);
                    }
                    _ => unreachable!("Unexpected global kind: {:?}", func.get_kind()),
                },
            }
            ControlFlow::Continue(())
        });

        // Make it mutable, since we'll reuse its storage.
        let mut all_globals = Vec::with_capacity(
            extern_vars.len()
                + extern_funcs.len()
                + funcs.len()
                + global_consts.len()
                + global_vars.len()
                + global_zero_inits.len(),
        );

        // Fill it with all globals in the order of their categories.
        all_globals.clear();
        all_globals.extend(extern_vars.into_iter());

        let extern_func_off = all_globals.len() as u32;
        all_globals.extend(extern_funcs.into_iter());

        let func_off = all_globals.len() as u32;
        all_globals.extend(funcs.into_iter());

        let global_const_off = all_globals.len() as u32;
        all_globals.extend(global_consts.into_iter());

        let global_var_off = all_globals.len() as u32;
        all_globals.extend(global_vars.into_iter());

        let global_zero_init_off = all_globals.len() as u32;
        all_globals.extend(global_zero_inits.into_iter());

        Self {
            all_globals: all_globals.into_boxed_slice(),
            extern_func_off,
            func_off,
            global_const_off,
            global_var_off,
            global_zero_init_off,
        }
    }
}

#[derive(Debug, Clone)]
pub struct MirFuncInfo {
    pub key: GlobalRef,
    pub mir: MirGlobalRef,
    pub rc: Rc<MirFunc>,
}

/// A read-only map of all MIR globals, no `new` method
#[derive(Clone)]
pub struct MirGlobalItems {
    pub all: Box<[(GlobalRef, MirGlobalRef)]>,
    pub funcs: Box<[MirFuncInfo]>,
    pub extern_funcs: Box<[MirFuncInfo]>,
}

impl MirGlobalItems {
    pub fn find_func(&self, ir_ref: GlobalRef) -> Option<&MirFuncInfo> {
        debug!("Searching for function with: {:?}", ir_ref);
        match self.funcs.binary_search_by_key(&ir_ref, |f| f.key) {
            Ok(idx) => Some(&self.funcs[idx]),
            Err(_) => match self.extern_funcs.binary_search_by_key(&ir_ref, |f| f.key) {
                Ok(idx) => Some(&self.extern_funcs[idx]),
                Err(_) => None,
            },
        }
    }
    pub fn find_mir_ref(&self, ir_ref: GlobalRef) -> Option<MirGlobalRef> {
        match self.all.binary_search_by_key(&ir_ref, |(gref, _)| *gref) {
            Ok(idx) => Some(self.all[idx].1),
            Err(_) => None,
        }
    }

    /// Builds MIR globals from the IR module and the global statistics.
    fn build_mir_from_statistics(
        ir_module: &IRModule,
        mir_builder: &mut MirBuilder,
        statistics: &GlobalStatistics,
    ) -> Self {
        let mut all_globals = Vec::with_capacity(statistics.all_globals.len());
        let mut funcs = Vec::with_capacity(statistics.funcs().len());
        let mut extern_funcs = Vec::with_capacity(statistics.extern_funcs().len());
        let allocs = &ir_module.allocs;

        for &gref in statistics.extern_vars() {
            let global = gref.to_data(&allocs.globals);
            let name = global.get_name().to_string();
            let ty = global.get_stored_pointee_type();
            debug!("Translating extern variable: {gref:?} name {name}");
            let section = if global.is_readonly() { Section::RoData } else { Section::Data };
            let (mir_ref, _) = mir_builder.extern_variable(name, section, ty, &ir_module.type_ctx);
            all_globals.push((gref, mir_ref));
        }

        for &gref in statistics.extern_funcs() {
            let GlobalData::Func(funcdef) = gref.to_data(&allocs.globals) else {
                panic!("Expected a function type for extern function, got {gref:?}");
            };
            let name = funcdef.get_name().to_string();
            let func_ty = funcdef.get_stored_func_type();
            let (mir_ref, _) = mir_builder.extern_func(name.clone(), func_ty, &ir_module.type_ctx);
            debug!("Translating extern function: {gref:?} name {name}");
            extern_funcs.push(MirFuncInfo {
                key: gref,
                mir: mir_ref.clone(),
                rc: Rc::new(MirFunc::new_extern(name, func_ty, &ir_module.type_ctx)),
            });
            all_globals.push((gref, mir_ref));
        }

        for &gref in statistics.funcs() {
            let GlobalData::Func(funcdef) = gref.to_data(&allocs.globals) else {
                panic!("Expected a function type for MIR function, got {gref:?}");
            };
            let name = funcdef.get_name().to_string();
            let func_ty = funcdef.get_stored_func_type();
            debug!("Translating function: {gref:?} name {name}");
            let mir_func = MirFunc::new_define(
                name,
                func_ty,
                &ir_module.type_ctx,
                &mut mir_builder.mir_module.borrow_alloc_block_mut(),
            );
            let (mir_ref, rc) = mir_builder.push_func(mir_func, false);
            funcs.push(MirFuncInfo { key: gref, mir: mir_ref.clone(), rc });
            all_globals.push((gref, mir_ref));
        }
        for &gref in statistics.global_consts() {
            Self::translate_global_var(
                ir_module,
                mir_builder,
                &mut all_globals,
                gref,
                Section::RoData,
                "global constant",
            );
        }
        for &gref in statistics.global_vars() {
            Self::translate_global_var(
                ir_module,
                mir_builder,
                &mut all_globals,
                gref,
                Section::Data,
                "global variable",
            );
        }
        for &gref in statistics.global_zero_inits() {
            Self::translate_global_var(
                ir_module,
                mir_builder,
                &mut all_globals,
                gref,
                Section::Bss,
                "global zero-init variable",
            );
        }
        all_globals.sort_by_key(|(gref, _)| *gref);
        funcs.sort_by_key(|f| f.key);
        extern_funcs.sort_by_key(|f| f.key);
        Self {
            all: all_globals.into_boxed_slice(),
            funcs: funcs.into_boxed_slice(),
            extern_funcs: extern_funcs.into_boxed_slice(),
        }
    }

    pub fn build_mir(ir_module: &IRModule, mir_builder: &mut MirBuilder) -> Self {
        let statistics = GlobalStatistics::new(ir_module);
        Self::build_mir_from_statistics(ir_module, mir_builder, &statistics)
    }

    fn translate_global_var(
        ir_module: &IRModule,
        mir_builder: &mut MirBuilder<'_>,
        all_globals: &mut Vec<(GlobalRef, MirGlobalRef)>,
        gref: GlobalRef,
        section: Section,
        category: &str,
    ) {
        let allocs = &ir_module.allocs;
        let GlobalData::Var(global) = gref.to_data(&allocs.globals) else {
            let name = gref.to_data(&allocs.globals).get_name();
            panic!("Expected a global variable, got {gref:?} name {name}");
        };
        let name = global.get_name().to_string();
        debug!("trnslating {category} reference {gref:?} name {name} section {section:?}");
        let initval = global.get_init();
        assert!(
            initval != ValueSSA::None,
            "{category} {name} has no initializer"
        );
        let constdef = if section != Section::Bss {
            let mut data_gen = DataGen::new();
            match data_gen.add_ir_value(initval, &ir_module.type_ctx, &allocs.exprs) {
                Ok(()) => {}
                Err(e) => {
                    panic!("Failed to add IR value for global constant {name}: {e}");
                }
            }
            log::debug!(
                "datagen from name {name} with {} units",
                data_gen.data.len()
            );
            MirGlobalVariable::with_init(
                name.to_string(),
                section,
                global.get_stored_pointee_type(),
                data_gen.collect_data(section),
                &ir_module.type_ctx,
            )
        } else {
            let global_type = global.get_stored_pointee_type();
            let size = global_type.get_size(&ir_module.type_ctx);
            let align_log2 = global_type.get_align_log2(&ir_module.type_ctx);
            let initval = MirGlobalData::new_zeroinit(
                Section::Bss,
                align_log2,
                size.div_ceil(1 << align_log2),
            );
            MirGlobalVariable::with_init(
                name.into(),
                Section::Bss,
                global_type,
                vec![initval],
                &ir_module.type_ctx,
            )
        };
        let (mir_ref, _) = mir_builder.push_variable(constdef);
        all_globals.push((gref, mir_ref));
    }
}

impl std::fmt::Debug for MirGlobalItems {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn get_funcs(funcs: &[MirFuncInfo]) -> Vec<String> {
            let mut func_names = Vec::with_capacity(funcs.len());
            for func in funcs {
                let func_name = func.rc.get_name();
                let func_mir = func.mir.get_handle();
                let func_ir = func.key.get_handle();
                func_names.push(format!(
                    "func `{func_name}` (MIR idx {func_mir}, IR idx {func_ir})"
                ));
            }
            func_names
        }
        fn get_alls(globals: &[(GlobalRef, MirGlobalRef)]) -> Vec<String> {
            let mut global_names = Vec::with_capacity(globals.len());
            for (gref, mir_ref) in globals {
                let mir_idx = mir_ref.get_handle();
                let ir_idx = gref.get_handle();
                global_names.push(format!("ir {ir_idx} -> mir {mir_idx}"));
            }
            global_names
        }
        f.debug_struct("MirGlobalItems")
            .field("all", &get_alls(&self.all))
            .field("funcs", &get_funcs(&self.funcs))
            .field("extern_funcs", &get_funcs(&self.extern_funcs))
            .finish()
    }
}

pub struct MirGlobalMapFormatter<'a> {
    pub globals: &'a MirGlobalItems,
    pub ir_module: &'a IRModule,
}

impl std::fmt::Debug for MirGlobalMapFormatter<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        fn get_funcs(funcs: &[MirFuncInfo]) -> Vec<String> {
            let mut func_names = Vec::with_capacity(funcs.len());
            for func in funcs {
                let func_name = func.rc.get_name();
                let func_mir = func.mir.get_handle();
                let func_ir = func.key.get_handle();
                func_names.push(format!(
                    "func `{func_name}` (MIR idx {func_mir}, IR idx {func_ir})"
                ));
            }
            func_names
        }
        fn get_alls(globals: &[(GlobalRef, MirGlobalRef)], ir_module: &IRModule) -> Vec<String> {
            let mut global_names = Vec::with_capacity(globals.len());
            for (gref, mir_ref) in globals {
                let ir_name = gref.get_name(&ir_module.allocs);
                let mir_idx = mir_ref.get_handle();
                let ir_idx = gref.get_handle();
                global_names.push(format!("ir {ir_idx} -> mir {mir_idx}: {ir_name}"));
            }
            global_names
        }
        f.debug_struct("MirGlobalItems")
            .field("all", &get_alls(&self.globals.all, self.ir_module))
            .field("funcs", &get_funcs(&self.globals.funcs))
            .field("extern_funcs", &get_funcs(&self.globals.extern_funcs))
            .finish()
    }
}

impl<'a> MirGlobalMapFormatter<'a> {
    pub fn new(globals: &'a MirGlobalItems, ir_module: &'a IRModule) -> Self {
        Self { globals, ir_module }
    }
}

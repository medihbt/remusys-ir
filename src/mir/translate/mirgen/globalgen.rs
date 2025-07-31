use std::{fmt::Debug, rc::Rc};

use log::debug;

use crate::{
    base::SlabRef,
    ir::{
        PtrStorage,
        global::{GlobalData, GlobalRef, func::FuncStorage},
        module::Module as IRModule,
    },
    mir::{
        module::{
            MirGlobalRef,
            func::MirFunc,
            global::{MirGlobalVariable, Section},
        },
        translate::mirgen::datagen::DataGen,
        util::builder::MirBuilder,
    },
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

        // Make it mutable, since we'll reuse its storage.
        let mut all_globals = ir_module.dump_globals(false);

        for &gref in &all_globals {
            let global = ir_module.get_global(gref);
            let global = &*global;
            let name = global.get_name();
            match global {
                GlobalData::Var(var) => {
                    if var.is_extern() {
                        debug!("Discovered extern variable: {gref:?} name {name}");
                        extern_vars.push(gref);
                        continue;
                    }
                    if var.is_readonly() {
                        debug!("Discovered global constant: {gref:?} name {name}");
                        global_consts.push(gref);
                        continue;
                    }
                    let init = match var.get_init() {
                        None => panic!("Has handled above"),
                        Some(init) => init,
                    };
                    if init.binary_is_zero(ir_module) {
                        debug!("Discovered global 0-init variable: {gref:?} name {name}");
                        global_zero_inits.push(gref);
                    } else {
                        debug!("Discovered global variable: {gref:?} name {name}");
                        global_vars.push(gref);
                    }
                }
                GlobalData::Func(func_data) => {
                    if func_data.is_extern() {
                        debug!("Discovered extern function: {gref:?} name {name}");
                        extern_funcs.push(gref);
                    } else {
                        debug!("Discovered function: {gref:?} name {name}");
                        funcs.push(gref);
                    }
                }
                GlobalData::Alias(_) => todo!("I'm too lazy to handle alias global data"),
            }
        }

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

        for &gref in statistics.extern_vars() {
            let global = ir_module.get_global(gref);
            let name = global.get_name().to_string();
            let ty = global.get_stored_pointee_type();
            let section = if global.is_readonly() { Section::RoData } else { Section::Data };
            debug!("Translating extern variable: {gref:?} name {name} section {section:?}");
            let (mir_ref, _) = mir_builder.extern_variable(name, section, ty, &ir_module.type_ctx);
            all_globals.push((gref, mir_ref));
        }

        for &gref in statistics.extern_funcs() {
            let global = ir_module.get_global(gref);
            let global = &*global;
            let name = global.get_name().to_string();
            let funcdef = match global {
                GlobalData::Func(func) => func,
                _ => panic!("Expected a function type for extern function, got {global:?}"),
            };
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
            let global = ir_module.get_global(gref);
            let global = &*global;
            let name = global.get_name().to_string();
            let funcdef = match global {
                GlobalData::Func(func) => func,
                _ => panic!("Expected a function type for MIR function, got {global:?}"),
            };
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
        let global = ir_module.get_global(gref);
        let name = global.get_name();
        debug!("trnslating {category} reference {gref:?} name {name} section {section:?}");
        let global = match &*global {
            GlobalData::Var(var) => var,
            GlobalData::Func(_) => {
                panic!("Expected a global constant, got function {name}")
            }
            GlobalData::Alias(_) => {
                panic!("Expected a global constant, got alias {name}")
            }
        };
        let initval = match global.get_init() {
            Some(init) => init,
            None => panic!("{category} {name} has no initializer"),
        };
        let mut data_gen = DataGen::new();
        let alloc_value = ir_module.borrow_value_alloc();
        let alloc_expr = &alloc_value.alloc_expr;
        match data_gen.add_ir_value(initval, &ir_module.type_ctx, alloc_expr) {
            Ok(()) => {}
            Err(e) => {
                panic!("Failed to add IR value for global constant {name}: {e}");
            }
        }
        let constdef = MirGlobalVariable::with_init(
            name.to_string(),
            section,
            global.get_stored_pointee_type(),
            data_gen.collect_data(section),
            &ir_module.type_ctx,
        );
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
                let ir_name = ir_module.get_global(*gref).get_name().to_string();
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

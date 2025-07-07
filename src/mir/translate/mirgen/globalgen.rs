use std::{collections::BTreeMap, rc::Rc};

use crate::{
    ir::{
        PtrStorage,
        global::{GlobalData, GlobalRef, func::FuncStorage},
        module::Module as IRModule,
    },
    mir::{
        module::{MirGlobalRef, func::MirFunc, global::Section},
        util::builder::MirBuilder,
    },
    typing::id::ValTypeID,
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum GlobalKind {
    ExternVar,
    ExternFunc,
    Func,
    GlobalConst,
    GlobalVar,
    GlobalZeroInit,
    ALL,
}

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
            match global {
                GlobalData::Var(var) => {
                    if var.is_extern() {
                        extern_vars.push(gref);
                        continue;
                    }
                    if var.is_readonly() {
                        global_consts.push(gref);
                        continue;
                    }
                    let init = match var.get_init() {
                        None => unreachable!("Has handled above"),
                        Some(init) => init,
                    };
                    if init.binary_is_zero(ir_module) {
                        global_zero_inits.push(gref);
                    } else {
                        global_vars.push(gref);
                    }
                }
                GlobalData::Func(func_data) => {
                    if func_data.is_extern() {
                        extern_funcs.push(gref);
                    } else {
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

        Self {
            all_globals: all_globals.into_boxed_slice(),
            extern_func_off,
            func_off,
            global_const_off,
            global_var_off,
            global_zero_init_off,
        }
    }

    fn foreach_item(&self, mut f: impl FnMut(GlobalRef, GlobalKind)) {
        for &gref in self.extern_vars() {
            f(gref, GlobalKind::ExternVar);
        }
        for &gref in self.extern_funcs() {
            f(gref, GlobalKind::ExternFunc);
        }
        for &gref in self.funcs() {
            f(gref, GlobalKind::Func);
        }
        for &gref in self.global_consts() {
            f(gref, GlobalKind::GlobalConst);
        }
        for &gref in self.global_vars() {
            f(gref, GlobalKind::GlobalVar);
        }
        for &gref in self.global_zero_inits() {
            f(gref, GlobalKind::GlobalZeroInit);
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
#[derive(Debug, Clone)]
pub struct MirGlobalItems {
    pub all: Box<[(GlobalRef, MirGlobalRef)]>,
    pub funcs: Box<[MirFuncInfo]>,
}

impl MirGlobalItems {
    pub fn find_func(&self, ir_ref: GlobalRef) -> Option<&MirFuncInfo> {
        match self.funcs.binary_search_by_key(&ir_ref, |f| f.key) {
            Ok(idx) => Some(&self.funcs[idx]),
            Err(_) => None,
        }
    }
    pub fn find_mir_ref(&self, ir_ref: GlobalRef) -> Option<MirGlobalRef> {
        match self.all.binary_search_by_key(&ir_ref, |(gref, _)| *gref) {
            Ok(idx) => Some(self.all[idx].1),
            Err(_) => None,
        }
    }

    /// Builds MIR globals from the IR module and the global statistics.
    fn build_mir_from_statusics(
        ir_module: &IRModule,
        mir_builder: &mut MirBuilder,
        statistics: &GlobalStatistics,
    ) -> Self {
        let mut all_globals = Vec::with_capacity(statistics.all_globals.len());
        let mut funcs = Vec::with_capacity(statistics.funcs().len());

        for &gref in statistics.extern_vars() {
            let global = ir_module.get_global(gref);
            let name = global.get_name().to_string();
            let ty = global.get_stored_pointee_type();
            let section = if global.is_readonly() {
                Section::RoData
            } else {
                Section::Data
            };
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
            let (mir_ref, _) = mir_builder.extern_func(name, func_ty, &ir_module.type_ctx);
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
            let mir_func = MirFunc::new_define(
                name,
                func_ty,
                &ir_module.type_ctx,
                &mut mir_builder.mir_module.borrow_alloc_block_mut(),
            );
            let (mir_ref, rc) = mir_builder.push_func(mir_func, false);
            funcs.push(MirFuncInfo {
                key: gref,
                mir: mir_ref.clone(),
                rc,
            });
            all_globals.push((gref, mir_ref));
        }
        for gref in statistics.global_consts() {
            todo!("Handle global constants in MIR generation");
        }
        for gref in statistics.global_vars() {
            todo!("Handle global variables in MIR generation");
        }
        for gref in statistics.global_zero_inits() {
            todo!("Handle global variables with zero initializer in MIR generation");
        }
        all_globals.sort_by_key(|(gref, _)| *gref);
        funcs.sort_by_key(|f| f.key);
        Self {
            all: all_globals.into_boxed_slice(),
            funcs: funcs.into_boxed_slice(),
        }
    }

    pub fn build_mir(ir_module: &IRModule, mir_builder: &mut MirBuilder) -> Self {
        let statistics = GlobalStatistics::new(ir_module);
        Self::build_mir_from_statusics(ir_module, mir_builder, &statistics)
    }
}

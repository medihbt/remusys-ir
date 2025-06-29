use std::{collections::BTreeMap, rc::Rc};

use slab::Slab;

use crate::{
    base::slabref::SlabRef,
    ir::{
        ValueSSA,
        constant::{
            data::ConstData,
            expr::{ConstExprData, ConstExprRef},
        },
        global::{GlobalData, GlobalRef, func::FuncStorage},
        module::Module,
    },
    mir::{
        module::{
            ModuleItemRef,
            func::MirFunc,
            global::{MirGlobalData, MirGlobalVariable, Section},
        },
        translate::mirgen::data_gen::translate_const_init,
        util::builder::MirBuilder,
    },
};

pub(super) struct GlobalStatistics {
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

pub(super) struct IrMirGlobalInfo {
    pub mapping: BTreeMap<GlobalRef, ModuleItemRef>,
    pub funcs: Vec<(GlobalRef, ModuleItemRef, Rc<MirFunc>)>,
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

    pub(super) fn new(ir_module: &super::IRModule) -> Self {
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
                    if global_init_is_zero(&init, ir_module) {
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

        GlobalStatistics {
            all_globals: all_globals.into_boxed_slice(),
            extern_func_off,
            func_off,
            global_const_off,
            global_var_off,
            global_zero_init_off,
        }
    }

    pub(super) fn make_global_items(
        &self,
        mir_builder: &mut MirBuilder,
        ir_module: &Module,
    ) -> IrMirGlobalInfo {
        let mut mapping = BTreeMap::new();
        let mut funcs = Vec::new();

        // Handle extern global variables
        for &gvar_ref in self.extern_vars() {
            let (item_ref, _) = extern_global_variable(ir_module, mir_builder, gvar_ref);
            mapping.insert(gvar_ref, item_ref);
        }
        // Handle extern functions
        for &gfunc_ref in self.extern_funcs() {
            let (item_ref, _) = extern_function(ir_module, mir_builder, gfunc_ref);
            mapping.insert(gfunc_ref, item_ref);
        }
        // Handle functions
        for &gfunc_ref in self.funcs() {
            /* leave it undefined -- since not all symbols are finely set up */
            let (item_ref, func) = extern_function(ir_module, mir_builder, gfunc_ref);
            mapping.insert(gfunc_ref, item_ref);
            funcs.push((gfunc_ref, item_ref, func));
        }
        // Handle global constants
        for &gvar_ref in self.global_consts() {
            let (item_ref, _) = define_global_variable(ir_module, mir_builder, gvar_ref, true);
            mapping.insert(gvar_ref, item_ref);
        }
        // Handle global variables
        for &gvar_ref in self.global_vars() {
            let (item_ref, _) = define_global_variable(ir_module, mir_builder, gvar_ref, false);
            mapping.insert(gvar_ref, item_ref);
        }
        // Handle global variables with zero initializer
        for &gvar_ref in self.global_zero_inits() {
            let (item_ref, _) = define_global_variable(ir_module, mir_builder, gvar_ref, true);
            mapping.insert(gvar_ref, item_ref);
        }
        IrMirGlobalInfo { mapping, funcs }
    }
}

fn global_init_is_zero(value: &ValueSSA, module: &super::IRModule) -> bool {
    match value {
        ValueSSA::ConstData(data) => const_data_is_zero(data),
        ValueSSA::ConstExpr(expr) => {
            let alloc_value = module.borrow_value_alloc();
            const_expr_is_zero(*expr, &alloc_value.alloc_expr)
        }
        ValueSSA::Block(_) | ValueSSA::Inst(_) | ValueSSA::FuncArg(..) | ValueSSA::Global(_) => {
            panic!("Unexpected value type in global statistics: {:?}", value)
        }
        ValueSSA::None => panic!("ValueSSA::None should not be used in global statistics"),
    }
}

fn const_data_is_zero(const_data: &ConstData) -> bool {
    match const_data {
        ConstData::Undef(_) | ConstData::Zero(_) | ConstData::PtrNull(_) => true,
        ConstData::Int(_, value) => *value == 0,
        ConstData::Float(_, fp) => fp.to_bits() == 0,
    }
}

fn const_expr_is_zero(expr: ConstExprRef, alloc_expr: &Slab<ConstExprData>) -> bool {
    let expr_data = expr.to_slabref_unwrap(alloc_expr);
    match expr_data {
        ConstExprData::Array(a) => const_aggr_is_zero(&a.elems, alloc_expr),
        ConstExprData::Struct(s) => const_aggr_is_zero(&s.elems, alloc_expr),
    }
}

fn const_aggr_is_zero(const_aggr: &[ValueSSA], alloc_expr: &Slab<ConstExprData>) -> bool {
    for value in const_aggr {
        let is_zero = match value {
            ValueSSA::ConstData(data) => const_data_is_zero(data),
            ValueSSA::ConstExpr(expr) => const_expr_is_zero(*expr, alloc_expr),
            ValueSSA::Global(_) => false,
            ValueSSA::FuncArg(..) | ValueSSA::Block(_) | ValueSSA::Inst(_) => {
                panic!("Unexpected value type in global statistics: {:?}", value)
            }
            ValueSSA::None => {
                panic!("ValueSSA::None should not be used in global statistics");
            }
        };
        if !is_zero {
            return false;
        }
    }
    true
}

fn extern_global_variable(
    ir_module: &Module,
    mir_builder: &mut MirBuilder,
    gvar_ref: GlobalRef,
) -> (ModuleItemRef, Rc<MirGlobalVariable>) {
    let gdata = ir_module.get_global(gvar_ref);
    let gdata = match &*gdata {
        GlobalData::Var(var) => var,
        _ => panic!("Expected GlobalData::Var, found: {:?}", gdata.get_name()),
    };
    let name = gdata.get_name();
    let section = if gdata.is_readonly() {
        Section::RoData
    } else {
        Section::Data
    };
    assert!(
        gdata.is_extern(),
        "Expected extern global variable, found: {}",
        name
    );
    mir_builder.extern_variable(
        name.into(),
        section,
        gdata.get_stored_pointee_type(),
        &ir_module.type_ctx,
    )
}

fn define_global_variable(
    ir_module: &Module,
    mir_builder: &mut MirBuilder,
    gvar_ref: GlobalRef,
    zero_init: bool,
) -> (ModuleItemRef, Rc<MirGlobalVariable>) {
    let gdata = ir_module.get_global(gvar_ref);
    let gdata = match &*gdata {
        GlobalData::Var(var) => var,
        _ => panic!("Expected GlobalData::Var, found: {:?}", gdata.get_name()),
    };
    let name = gdata.get_name();
    let section = if gdata.is_readonly() {
        Section::RoData
    } else if zero_init {
        Section::Bss
    } else {
        Section::Data
    };
    let mir_type = gdata.get_stored_pointee_type();
    let align = gdata.get_stored_pointee_align();
    let align_log2 = if align.is_power_of_two() {
        align.trailing_zeros() as u8
    } else {
        panic!("Align {} NOT power of 2", align)
    };

    let type_ctx = &ir_module.type_ctx;
    let mut mir_gvar = MirGlobalVariable::new_extern(name.into(), section, mir_type, type_ctx);
    mir_gvar.mark_defined();

    let instance_size = mir_type
        .get_instance_size(type_ctx)
        .expect("global variable must have a defined size");
    mir_gvar.initval = if zero_init {
        vec![MirGlobalData::new_bytes_vec(
            section,
            vec![0; instance_size],
        )]
    } else {
        translate_const_init(gdata.get_init().unwrap(), section, ir_module)
    };
    mir_gvar.common.size = mir_type
        .get_instance_size(type_ctx)
        .expect("global variable must have a defined size");
    mir_gvar.common.align_log2 = align_log2;
    mir_builder.push_variable(mir_gvar)
}

fn extern_function(
    ir_module: &Module,
    mir_builder: &mut MirBuilder,
    gfunc_ref: GlobalRef,
) -> (ModuleItemRef, Rc<MirFunc>) {
    let func_data = ir_module.get_global(gfunc_ref);
    let func_data = match &*func_data {
        GlobalData::Func(func) => func,
        _ => panic!(
            "Expected GlobalData::Func, found: {:?}",
            func_data.get_name()
        ),
    };
    mir_builder.extern_func(
        func_data.get_name().into(),
        func_data.get_stored_func_type(),
        &ir_module.type_ctx,
    )
}

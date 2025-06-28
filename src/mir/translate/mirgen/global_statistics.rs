use slab::Slab;

use crate::{
    base::slabref::SlabRef,
    ir::{
        ValueSSA,
        constant::{
            data::ConstData,
            expr::{ConstExprData, ConstExprRef},
        },
        global::{GlobalData, GlobalRef},
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
}

fn global_init_is_zero(value: &ValueSSA, module: &super::IRModule) -> bool {
    match value {
        ValueSSA::ConstData(data) => const_data_is_zero(data),
        ValueSSA::ConstExpr(expr) => todo!(),
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
            ValueSSA::Global(g) => false,
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

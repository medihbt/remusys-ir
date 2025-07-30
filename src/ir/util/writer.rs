//! IR Writer implementation.

use basic_value_formatting::format_value_by_ref;
use slab::Slab;

use crate::{
    base::slabref::SlabRef,
    ir::{
        IValueVisitor, ValueSSA,
        block::{BlockData, BlockRef, jump_target::JumpTargetData},
        constant::{
            data::{ConstData, IConstDataVisitor},
            expr::{Array, ConstExprRef, IConstExprVisitor, Struct},
        },
        global::{
            self, Alias, GlobalData, GlobalRef, IGlobalObjectVisitor,
            func::{FuncData, FuncStorage},
        },
        inst::{
            InstData, InstDataCommon, InstRef,
            alloca::Alloca,
            binop::BinOp,
            callop::CallOp,
            cast::CastOp,
            cmp::CmpOp,
            gep::IndexPtrOp,
            load_store::{LoadOp, StoreOp},
            phi::PhiOp,
            select::SelectOp,
            terminator::{Br, Jump, Ret, Switch},
            usedef::UseData,
            visitor::IInstVisitor,
        },
        module::{Module, ModuleAllocatorInner},
    },
    typing::{id::ValTypeID, types::FloatTypeKind},
};

use std::{
    cell::{Cell, Ref, RefCell},
    io::Write as IoWrite,
};

pub fn write_ir_module(
    module: &Module,
    writer: &mut dyn IoWrite,
    prints_rdfg: bool,
    prints_rcfg: bool,
    prints_slabref: bool,
) {
    let mut module_writer = ModuleValueWriter::new(module, writer);
    module_writer.prints_rdfg = if module.rdfg_enabled() {
        prints_rdfg
    } else {
        module_writer.write_str("; DFG tracking disabled, will not print\n");
        false
    };
    module_writer.prints_rcfg = if module.rcfg_enabled() {
        prints_rcfg
    } else {
        module_writer.write_str("; CFG tracking disabled, will not print\n");
        false
    };
    module_writer.prints_slabref = prints_slabref;
    module_writer.process_module();
}

pub fn write_ir_expr(module: &Module, writer: &mut dyn IoWrite, expr: ConstExprRef) {
    writeln!(
        writer,
        "{}",
        format_value_by_ref(
            &module.borrow_value_alloc(),
            &module.type_ctx,
            &[],
            &[],
            ValueSSA::ConstExpr(expr)
        )
    )
    .unwrap();
}

struct ModuleValueWriter<'a> {
    module: &'a Module,
    alloc_value: Ref<'a, ModuleAllocatorInner>,
    alloc_use: Ref<'a, Slab<UseData>>,
    alloc_jt: Ref<'a, Slab<JumpTargetData>>,

    writer: RefCell<&'a mut dyn IoWrite>,

    inst_id_map: RefCell<Vec<usize>>,
    block_id_map: RefCell<Vec<usize>>,
    live_func_def: RefCell<Vec<GlobalRef>>,

    current_indent: Cell<usize>,
    prints_rdfg: bool,
    prints_rcfg: bool,
    prints_slabref: bool,
}

struct ModuleWriterIndentGuard<'a, 'b: 'a> {
    module: &'a ModuleValueWriter<'b>,
    prev_indent: usize,
}

impl<'a, 'b> Drop for ModuleWriterIndentGuard<'a, 'b> {
    fn drop(&mut self) {
        self.module.current_indent.set(self.prev_indent);
    }
}

impl<'a> ModuleValueWriter<'a> {
    fn new(module: &'a Module, writer: &'a mut dyn IoWrite) -> Self {
        let alloc_value = module.borrow_value_alloc();
        let inst_id_map_capacity = alloc_value.alloc_inst.capacity();
        let block_id_map_capcity = alloc_value.alloc_block.capacity();
        let live_func_def_len = alloc_value.alloc_global.len();
        Self {
            module: module,
            alloc_value: alloc_value,
            alloc_use: module.borrow_use_alloc(),
            alloc_jt: module.borrow_jt_alloc(),
            writer: RefCell::new(writer),
            inst_id_map: RefCell::new(vec![usize::MAX; inst_id_map_capacity]),
            block_id_map: RefCell::new(vec![usize::MAX; block_id_map_capcity]),
            live_func_def: RefCell::new(Vec::with_capacity(live_func_def_len)),
            current_indent: Cell::new(0),
            prints_rdfg: false,
            prints_rcfg: false,
            prints_slabref: false,
        }
    }

    fn process_module(&self) {
        let global_defs = self.module.global_defs.borrow();
        let globals: Vec<_> = global_defs.iter().map(|(_, gref)| gref).collect();
        for global in globals {
            self.global_object_visitor_dispatch(*global, &self.alloc_value.alloc_global);
        }

        let live_funcs = self.live_func_def.borrow();
        for func in &*live_funcs {
            if self.prints_slabref {
                self.write_fmt(format_args!("; {:?}\n", func));
            }
            let func = match func.to_data(&self.alloc_value.alloc_global) {
                GlobalData::Func(f) => f,
                _ => panic!("Invalid global data kind: Not Function"),
            };
            self.write_funcdef(func);
        }
    }
    fn write_funcdef(&self, func: &FuncData) {
        // Header syntax: `define dso_local <return type> @<name>(<args>)`
        self.write_func_header(func);

        // Then write body.
        self.write_str(" {");
        for (block, block_data) in func
            .get_blocks()
            .unwrap()
            .view(&self.alloc_value.alloc_block)
        {
            self.wrap_indent();
            self.read_block(block, block_data);
        }
        self.write_str("\n}\n");
    }
    fn write_func_header(&self, func: &FuncData) {
        let type_ctx = self.module.type_ctx.as_ref();
        let self_type = func.get_stored_func_type();
        let ret_type = self_type.get_return_type(type_ctx);
        let args_type = self_type.get_args(type_ctx);

        let (leading, is_funcdef) =
            if func.is_extern() { ("declare", false) } else { ("define dso_local", true) };

        self.write_fmt(format_args!(
            "{} {} @{}({})",
            leading,
            ret_type.get_display_name(type_ctx),
            func.get_name(),
            args_type
                .iter()
                .enumerate()
                .map(|(i, t)| {
                    let arg_type = t.get_display_name(type_ctx);
                    if is_funcdef { format!("{} %{}", arg_type, i) } else { arg_type }
                })
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    fn write_str(&self, s: &str) {
        self.writer.borrow_mut().write_all(s.as_bytes()).unwrap();
    }
    fn write_fmt(&self, fmtargs: std::fmt::Arguments) {
        self.writer.borrow_mut().write_fmt(fmtargs).unwrap();
    }
    fn wrap_indent(&self) {
        self.write_str("\n");
        for _ in 0..self.current_indent.get() {
            self.write_str("    ");
        }
    }
    fn add_indent<'b>(&'b self) -> ModuleWriterIndentGuard<'b, 'a>
    where
        'a: 'b,
    {
        let prev_indent = self.current_indent.get();
        self.current_indent.set(prev_indent + 1);
        ModuleWriterIndentGuard { module: self, prev_indent }
    }

    fn add_def_if_live_func(&self, func: GlobalRef, func_data: &FuncData) -> bool {
        if !func_data.is_extern() {
            self.live_func_def.borrow_mut().push(func);
            self.number_function(func_data);
            true
        } else {
            false
        }
    }
    fn number_function(&self, func: &FuncData) {
        let nargs = func.get_nargs(&self.module.type_ctx);
        let blocks = func.get_blocks();
        let blocks = &*blocks.unwrap();

        let mut current_id = nargs;
        for (bb, bb_data) in blocks.view(&self.alloc_value.alloc_block) {
            current_id = self.number_block(bb, bb_data, current_id);
        }
    }
    fn number_block(&self, block: BlockRef, block_data: &BlockData, initial_id: usize) -> usize {
        let mut block_id_map = self.block_id_map.borrow_mut();
        let mut inst_id_map = self.inst_id_map.borrow_mut();

        block_id_map[block.get_handle()] = initial_id;
        let mut curr_id = initial_id + 1;

        for (inst, inst_data) in block_data.instructions.view(&self.alloc_value.alloc_inst) {
            match inst_data {
                InstData::Unreachable(..)
                | InstData::Ret(..)
                | InstData::Jump(..)
                | InstData::Br(..)
                | InstData::Switch(..)
                | InstData::PhiInstEnd(..)
                | InstData::Store(..) => continue,
                _ => {}
            }
            match inst_data.get_value_type() {
                ValTypeID::Void => {}
                _ => {
                    inst_id_map[inst.get_handle()] = curr_id;
                    curr_id += 1;
                }
            }
        }
        curr_id
    }
    fn block_get_id(&self, block: BlockRef) -> Option<usize> {
        let ret = self.block_id_map.borrow()[block.get_handle()];
        if ret == usize::MAX { None } else { Some(ret) }
    }
    fn inst_get_id(&self, inst: InstRef) -> Option<usize> {
        let ret = self.inst_id_map.borrow()[inst.get_handle()];
        if ret == usize::MAX { None } else { Some(ret) }
    }
    fn block_getid_unwrap(&self, block: BlockRef) -> usize {
        match self.block_get_id(block) {
            Some(x) => x,
            None => panic!("Block {:?} not numbered", block),
        }
    }
    fn inst_getid_unwrap(&self, inst: InstRef) -> usize {
        match self.inst_get_id(inst) {
            Some(x) => x,
            None => panic!("Instruction {:?} not numbered", inst),
        }
    }

    fn format_value_by_ref(&self, value: ValueSSA) -> String {
        let inner = self.module.borrow_value_alloc();
        basic_value_formatting::format_value_by_ref(
            &inner,
            &self.module.type_ctx,
            &self.inst_id_map.borrow(),
            &self.block_id_map.borrow(),
            value,
        )
    }
}

impl IValueVisitor for ModuleValueWriter<'_> {
    /// Block syntax:
    ///
    /// ```llvm
    /// %<block id>:
    ///     inst 0
    ///     inst 1
    ///     ...
    ///     terminator
    /// ```
    fn read_block(&self, block: BlockRef, block_data: &BlockData) {
        // ID
        let block_id = self.block_getid_unwrap(block);
        if self.prints_slabref {
            self.write_fmt(format_args!("\n; {:?}\n", block));
        }
        self.write_fmt(format_args!("{}:", block_id));
        if self.prints_rdfg {
            self.write_value_users(ValueSSA::Block(block));
        }
        if self.prints_rcfg {
            self.write_block_predecessors(block);
        }

        let _g = self.add_indent();
        let insts = block_data.instructions.view(&self.alloc_value.alloc_inst);
        for (inst_ref, inst_data) in insts {
            if self.prints_slabref {
                self.wrap_indent();
                self.wrap_indent();
                self.write_fmt(format_args!("; {:?}", inst_ref));
            }
            match inst_data {
                InstData::PhiInstEnd(..) => {
                    self.wrap_indent();
                    self.write_str("; <Phi Ending and normal instruction beginning>");
                    continue;
                }
                _ => self.wrap_indent(),
            }
            if self.prints_rdfg {
                self.write_value_users(ValueSSA::Inst(inst_ref));
            }
            self.inst_visitor_dispatch(inst_ref, inst_data);
        }
    }

    fn read_func_arg(&self, _: GlobalRef, _: u32) {}
}

impl<'a> ModuleValueWriter<'a> {
    fn write_block_predecessors(&self, block_ref: BlockRef) {
        let rcfg = self.module.borrow_rcfg_alloc().unwrap();
        let pred_jt = rcfg
            .get_node(block_ref)
            .dump_pred_blocks(&self.alloc_jt, &self.alloc_value.alloc_inst);
        self.write_fmt(format_args!(
            "; Predecessors: {}",
            pred_jt
                .iter()
                .map(|b| self.format_value_by_ref(ValueSSA::Block(*b)))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }
    fn write_value_users(&self, value: ValueSSA) {
        let rdfg = self.module.borrow_rdfg_alloc().unwrap();
        let users = rdfg.get_node(value).unwrap();
        self.write_fmt(format_args!(
            "; Users: {}",
            users
                .collect_users(&self.alloc_use)
                .iter()
                .map(|u| self.format_value_by_ref(ValueSSA::Inst(*u)))
                .collect::<Vec<_>>()
                .join(", ")
        ));
        self.wrap_indent();
    }
}

impl IConstDataVisitor for ModuleValueWriter<'_> {
    fn read_int_const(&self, _: u8, _: i128) {}
    fn read_float_const(&self, _: FloatTypeKind, _: f64) {}
    fn read_ptr_null(&self, _: ValTypeID) {}
    fn read_undef(&self, _: ValTypeID) {}
    fn read_zero(&self, _: ValTypeID) {}
}

impl IConstExprVisitor for ModuleValueWriter<'_> {
    fn read_array(&self, _: ConstExprRef, _: &Array) {}
    fn read_struct(&self, _: ConstExprRef, _: &Struct) {}
}

impl IGlobalObjectVisitor for ModuleValueWriter<'_> {
    /// Syntax: `@<name> = external|dso_local global <type> [initializer], align <align>`
    fn read_global_variable(&self, _: GlobalRef, gvar: &global::Var) {
        let gvar_kind = if gvar.is_readonly() { "constant" } else { "global" };
        if gvar.is_extern() {
            self.write_fmt(format_args!(
                "@{} = external {} {}, align {}\n",
                gvar.common.name,
                gvar_kind,
                gvar.common
                    .content_ty
                    .get_display_name(&self.module.type_ctx),
                gvar.get_stored_pointee_align()
            ));
        } else {
            self.write_fmt(format_args!(
                "@{} = dso_local {} {} {}, align {}\n",
                gvar.common.name,
                gvar_kind,
                gvar.common
                    .content_ty
                    .get_display_name(&self.module.type_ctx),
                self.format_value_by_ref(gvar.get_init().unwrap()),
                gvar.get_stored_pointee_align()
            ));
        }
    }

    /// Syntax: `@<name> = alias <type>, <target>`
    fn read_global_alias(&self, _: GlobalRef, galias: &Alias) {
        self.write_fmt(format_args!(
            "@{} = alias {} {}",
            galias.common.name,
            galias
                .common
                .content_ty
                .get_display_name(&self.module.type_ctx),
            self.format_value_by_ref(ValueSSA::Global(galias.target.get()))
        ));
    }

    /// Function declaration syntax: `declare <type> @<name>(<arg types>)`
    /// Function definition will be collected and handled in the other place.
    fn read_func(&self, global_ref: GlobalRef, func: &FuncData) {
        if self.add_def_if_live_func(global_ref, func) {
            // Function definitions, return.
            return;
        }
        if self.prints_slabref {
            self.write_fmt(format_args!("; {:?}\n", global_ref));
        }
        self.write_func_header(func);
        self.wrap_indent();
    }
}

impl IInstVisitor for ModuleValueWriter<'_> {
    /// Hidden, no syntax
    fn read_phi_end(&self, _: InstRef) {
        // No syntax
        self.write_str("; <Phi Ending and normal instruction beginning>");
    }

    /// Syntax: `%<name> = phi <type> [<value>, %<block>], ...`
    fn read_phi_inst(&self, inst_ref: InstRef, common: &InstDataCommon, phi: &PhiOp) {
        self.write_fmt(format_args!(
            "%{} = phi {} {}",
            self.inst_getid_unwrap(inst_ref),
            common.ret_type.get_display_name(&self.module.type_ctx),
            phi.get_from_all()
                .iter()
                .map(|po| format!(
                    "[{}, %{}]",
                    self.format_value_by_ref(po.from_value_use.get_operand(&self.alloc_use)),
                    self.block_getid_unwrap(po.from_bb)
                ))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    /// Syntax: `unreachable`
    fn read_unreachable_inst(&self, _: InstRef, _: &InstDataCommon) {
        self.write_str("unreachable");
    }

    /// Syntax: `ret <type> <value>`
    fn read_ret_inst(&self, _: InstRef, common: &InstDataCommon, ret: &Ret) {
        if let ValTypeID::Void = common.ret_type {
            self.write_str("ret void");
            return;
        }
        self.write_fmt(format_args!(
            "ret {} {}",
            common.ret_type.get_display_name(&self.module.type_ctx),
            self.format_value_by_ref(ret.retval.get_operand(&self.alloc_use))
        ));
    }

    /// Syntax: `br label %<block>`
    fn read_jump_inst(&self, _: InstRef, _: &InstDataCommon, jump: &Jump) {
        self.write_fmt(format_args!(
            "br label %{}",
            self.block_getid_unwrap(jump.get_block(&self.alloc_jt))
        ));
    }

    /// Syntax: `br <cond>, label %<true block>, label %<false block>`
    fn read_br_inst(&self, _: InstRef, _: &InstDataCommon, br: &Br) {
        self.write_fmt(format_args!(
            "br i1 {}, label %{}, label %{}",
            self.format_value_by_ref(br.get_cond(&self.alloc_use)),
            self.block_getid_unwrap(br.if_true.get_block(&self.alloc_jt)),
            self.block_getid_unwrap(br.if_false.get_block(&self.alloc_jt)),
        ));
    }

    /// Syntax:
    /// ```llvm-ir
    /// switch <type> <value>, label %<default block>, [
    ///     <value1>, label %<case block>,
    ///     <value2>, label %<case block>,
    ///     ...
    /// ]
    /// ```
    fn read_switch_inst(&self, _: InstRef, _: &InstDataCommon, switch: &Switch) {
        let cond = switch.get_cond(&self.alloc_use);
        let cond_type = cond.get_value_type(&self.module);
        self.write_fmt(format_args!(
            "switch {} {}, label %{}, [",
            cond_type.get_display_name(&self.module.type_ctx),
            self.format_value_by_ref(cond),
            self.block_getid_unwrap(switch.get_default(&self.alloc_jt)),
        ));

        let grd = self.add_indent();
        for (c, j) in &*switch.borrow_cases() {
            self.wrap_indent();
            self.write_fmt(format_args!(
                "{}, label %{}",
                c,
                self.block_getid_unwrap(j.get_block(&self.alloc_jt))
            ));
        }
        drop(grd);

        self.wrap_indent();
        self.write_str("]");
    }

    /// Syntax: `%<name> = alloca <type>, align <align>`
    fn read_alloca_inst(&self, inst_ref: InstRef, _: &InstDataCommon, alloca: &Alloca) {
        let type_ctx = self.module.type_ctx.as_ref();
        self.write_fmt(format_args!(
            "%{} = alloca {}, align {}",
            self.inst_getid_unwrap(inst_ref),
            alloca.pointee_ty.get_display_name(type_ctx),
            1 << alloca.align_log2
        ));
    }

    /// Syntax: `%<name> = load <type>, ptr %<ptr>, align <align>`
    fn read_load_inst(&self, inst_ref: InstRef, _: &InstDataCommon, load: &LoadOp) {
        self.write_fmt(format_args!(
            "%{} = load {}, ptr {}, align {}",
            self.inst_getid_unwrap(inst_ref),
            load.source_ty.get_display_name(&self.module.type_ctx),
            self.format_value_by_ref(load.source.get_operand(&self.alloc_use)),
            load.align.get()
        ));
    }

    /// Syntax: `store <type> <value>, ptr <ptr>, align <align>`
    fn read_store_inst(&self, _: InstRef, _: &InstDataCommon, store: &StoreOp) {
        let type_ctx = self.module.type_ctx.as_ref();
        let alloc_use = &*self.alloc_use;
        self.write_fmt(format_args!(
            "store {} {}, ptr {}, align {}",
            store.target_ty.get_display_name(type_ctx),
            self.format_value_by_ref(store.source.get_operand(alloc_use)),
            self.format_value_by_ref(store.target.get_operand(alloc_use)),
            store.align.get()
        ));
    }

    /// Syntax: `%<name> = select <type>, i1 <cond>, <true value>, <false value>`
    fn read_select_inst(&self, inst_ref: InstRef, common: &InstDataCommon, select: &SelectOp) {
        let type_ctx = self.module.type_ctx.as_ref();
        let alloc_use = &*self.alloc_use;
        let inst_ty = common.ret_type.get_display_name(type_ctx);
        self.write_fmt(format_args!(
            "%{} = select i1 {}, {} {}, {} {}",
            self.inst_getid_unwrap(inst_ref),
            self.format_value_by_ref(select.cond.get_operand(alloc_use)),
            inst_ty,
            self.format_value_by_ref(select.true_val.get_operand(alloc_use)),
            inst_ty,
            self.format_value_by_ref(select.false_val.get_operand(alloc_use))
        ));
    }

    /// Syntax: `%<name> = <op> <type> <value1>, <value2>`
    fn read_bin_op_inst(&self, inst_ref: InstRef, common: &InstDataCommon, bin_op: &BinOp) {
        let type_ctx = self.module.type_ctx.as_ref();
        let alloc_use = &*self.alloc_use;
        self.write_fmt(format_args!(
            "%{} = {} {} {}, {}",
            self.inst_getid_unwrap(inst_ref),
            common.opcode.get_name(),
            common.ret_type.get_display_name(type_ctx),
            self.format_value_by_ref(bin_op.lhs.get_operand(alloc_use)),
            self.format_value_by_ref(bin_op.rhs.get_operand(alloc_use)),
        ));
    }

    /// Syntax: `%<name> = <op> <cond> <type> <value1>, <value2>`
    fn read_cmp_inst(&self, inst_ref: InstRef, common: &InstDataCommon, cmp: &CmpOp) {
        let type_ctx = self.module.type_ctx.as_ref();
        let alloc_use = &*self.alloc_use;
        self.write_fmt(format_args!(
            "%{} = {} {} {} {}, {}",
            self.inst_getid_unwrap(inst_ref),
            common.opcode.get_name(),
            cmp.cond.to_string(),
            cmp.cmp_ty.get_display_name(type_ctx),
            self.format_value_by_ref(cmp.lhs.get_operand(alloc_use)),
            self.format_value_by_ref(cmp.rhs.get_operand(alloc_use)),
        ));
    }

    /// Syntax: `%<name> = <op> <type> <value> to <type>`
    fn read_cast_inst(&self, inst_ref: InstRef, common: &InstDataCommon, cast: &CastOp) {
        let type_ctx = self.module.type_ctx.as_ref();
        let alloc_use = &*self.alloc_use;

        let from_value = cast.from_op.get_operand(alloc_use);
        let from_valuety = from_value.get_value_type(&self.module);

        self.write_fmt(format_args!(
            "%{} = {} {} {} to {}",
            self.inst_getid_unwrap(inst_ref),
            common.opcode.get_name(),
            from_valuety.get_display_name(type_ctx),
            self.format_value_by_ref(from_value),
            common.ret_type.get_display_name(type_ctx),
        ));
    }

    /// Syntax: `%<name> = getelementptr <index0 type>, ptr %<ptr>, <index type> <index>, ...`
    fn read_index_ptr_inst(&self, inst_ref: InstRef, _: &InstDataCommon, index_ptr: &IndexPtrOp) {
        let type_ctx = self.module.type_ctx.as_ref();
        let alloc_use = &*self.alloc_use;

        self.write_fmt(format_args!(
            "%{} = getelementptr {}, ptr {}, {}",
            self.inst_getid_unwrap(inst_ref),
            index_ptr.base_pointee_ty.get_display_name(type_ctx),
            self.format_value_by_ref(index_ptr.base_ptr.get_operand(alloc_use)),
            index_ptr
                .indices
                .iter()
                .map(|uidx| {
                    let val = uidx.get_operand(alloc_use);
                    format!(
                        "{} {}",
                        val.get_value_type(&self.module).get_display_name(type_ctx),
                        self.format_value_by_ref(val)
                    )
                })
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    /// Syntax: `%<name> = call <type> @<name>(<args>)`
    fn read_call_inst(&self, inst_ref: InstRef, common: &InstDataCommon, call: &CallOp) {
        let type_ctx = self.module.type_ctx.as_ref();
        let alloc_use = &*self.alloc_use;

        match common.ret_type {
            ValTypeID::Void => {}
            _ => self.write_fmt(format_args!("%{} = ", self.inst_getid_unwrap(inst_ref))),
        }
        #[rustfmt::skip]
        self.write_fmt(format_args!(
            "call {} {}({})",
            common.ret_type.get_display_name(type_ctx),
            self.format_value_by_ref(call.callee.get_operand(alloc_use)),
            call.args.iter().map(|u| {
                let operand = u.get_operand(alloc_use);
                let operand_ty = operand.get_value_type(&self.module);
                format!(
                    "{} {}",
                    operand_ty.get_display_name(type_ctx),
                    self.format_value_by_ref(operand)
                )
            }).collect::<Vec<_>>().join(", ")
        ));
    }
}

mod basic_value_formatting {
    use std::{
        cell::RefCell,
        io::{Cursor, Read, Write},
    };

    use crate::typing::context::TypeContext;

    use super::*;
    use slab::Slab;

    pub fn format_value_by_ref(
        module: &ModuleAllocatorInner,
        type_ctx: &TypeContext,
        inst_id_map: &[usize],
        block_id_map: &[usize],
        value: ValueSSA,
    ) -> String {
        let formatter = BasicValueFormatter {
            module,
            type_ctx,
            inst_id_map,
            block_id_map,
            str_writer: RefCell::new(Cursor::new(vec![])),
        };
        formatter.value_visitor_diapatch(value, module);
        formatter.extract_string()
    }

    struct BasicValueFormatter<'a> {
        pub module: &'a ModuleAllocatorInner,
        pub type_ctx: &'a TypeContext,
        pub inst_id_map: &'a [usize],
        pub block_id_map: &'a [usize],
        pub str_writer: RefCell<Cursor<Vec<u8>>>,
    }
    impl<'a> BasicValueFormatter<'a> {
        fn write_str(&self, s: &str) {
            self.str_writer
                .borrow_mut()
                .write_all(s.as_bytes())
                .unwrap();
        }
        fn write_fmt(&self, fmtargs: std::fmt::Arguments) {
            self.str_writer.borrow_mut().write_fmt(fmtargs).unwrap();
        }
        pub fn extract_string(&self) -> String {
            let mut str_writer = self.str_writer.borrow_mut();
            let mut buf = vec![];
            str_writer.set_position(0);
            str_writer.read_to_end(&mut buf).unwrap();
            String::from_utf8(buf).unwrap()
        }
    }
    impl IValueVisitor for BasicValueFormatter<'_> {
        fn read_block(&self, block: BlockRef, _: &BlockData) {
            self.write_str("%");
            self.write_str(self.block_id_map[block.get_handle()].to_string().as_str());
        }
        fn read_func_arg(&self, _: GlobalRef, index: u32) {
            self.write_fmt(format_args!("%{}", index));
        }
    }
    impl IConstDataVisitor for BasicValueFormatter<'_> {
        fn read_int_const(&self, nbits: u8, value: i128) {
            let real_value = ConstData::iconst_value_get_real_signed(nbits, value);
            if nbits == 1 {
                /* boolean */
                self.write_str(if real_value == 0 { "false" } else { "true" });
            } else {
                /* normal number */
                self.write_str(real_value.to_string().as_str());
            }
        }

        fn read_float_const(&self, fp_kind: FloatTypeKind, value: f64) {
            match fp_kind {
                FloatTypeKind::Ieee32 => self.write_str(format!("{:.32e}", value as f32).as_str()),
                FloatTypeKind::Ieee64 => self.write_str(format!("{:.32e}", value).as_str()),
            }
        }

        fn read_ptr_null(&self, _: ValTypeID) {
            self.write_str("null");
        }

        fn read_undef(&self, _: ValTypeID) {
            self.write_str("undef");
        }

        fn read_zero(&self, ty: ValTypeID) {
            match ty {
                ValTypeID::Void => self.write_str("void"),
                ValTypeID::Ptr => self.write_str("null"),
                ValTypeID::Int(..) => self.write_str("0"),
                ValTypeID::Float(..) => self.write_str("0.0"),
                ValTypeID::Array(..) | ValTypeID::Struct(..) | ValTypeID::StructAlias(..) => {
                    self.write_str("zeroinitializer")
                }
                ValTypeID::Func(..) => panic!("Function is not instantiable"),
            }
        }
    }

    impl IConstExprVisitor for BasicValueFormatter<'_> {
        fn read_array(&self, _: ConstExprRef, array_data: &Array) {
            if let Some(str_literal) = self.try_format_string_literal(&array_data.elems) {
                self.write_str(&str_literal);
                return;
            }
            self.write_str("[");
            for (i, item) in array_data.elems.iter().enumerate() {
                if i > 0 {
                    self.write_str(", ");
                }
                let elem_ty = array_data.arrty.get_element_type(self.type_ctx);
                self.write_fmt(format_args!("{} ", elem_ty.get_display_name(self.type_ctx)));
                self.value_visitor_diapatch(item.clone(), self.module);
            }
            self.write_str("]");
        }

        fn read_struct(&self, _: ConstExprRef, s: &Struct) {
            self.write_str("{");
            for (i, item) in s.elems.iter().enumerate() {
                if i > 0 {
                    self.write_str(", ");
                }
                let struct_ty = match s.structty {
                    ValTypeID::Struct(s) => s,
                    ValTypeID::StructAlias(sa) => sa.get_aliasee(self.type_ctx),
                    _ => panic!("Invalid type in constant expression"),
                };
                let elem_ty = struct_ty
                    .get_element_type(self.type_ctx, i)
                    .expect("index out of bounds in struct type");
                self.write_fmt(format_args!("{} ", elem_ty.get_display_name(self.type_ctx)));
                self.value_visitor_diapatch(item.clone(), self.module);
            }
            self.write_str("}");
        }
    }

    impl<'a> BasicValueFormatter<'a> {
        fn try_format_string_literal(&self, elems: &[ValueSSA]) -> Option<String> {
            if elems.is_empty() {
                return Some("\"\"".to_string());
            }
            let mut ret = String::with_capacity(elems.len() + 2);
            ret.push('"');
            for i in elems {
                let c: char = if let ValueSSA::ConstData(ConstData::Int(8, c)) = i {
                    let c = ConstData::iconst_value_get_real_unsigned(8, *c);
                    (c as u8).into()
                } else {
                    return None; // Not a valid string literal
                };
                if c.is_ascii_alphabetic() || c.is_ascii_digit() || c.is_ascii_punctuation() {
                    ret.push(c);
                } else {
                    let hex_str = format!("{:02x}", c as u32);
                    ret.push('\\');
                    ret.push_str(&hex_str);
                }
            }
            ret.push('"');
            Some(ret)
        }
    }

    impl IGlobalObjectVisitor for BasicValueFormatter<'_> {
        fn global_object_visitor_dispatch(&self, globl: GlobalRef, alloc: &Slab<GlobalData>) {
            self.write_str("@");
            self.write_str(globl.to_data(alloc).get_common().name.as_str());
        }
        fn read_global_variable(&self, _: GlobalRef, _: &crate::ir::global::Var) {}
        fn read_global_alias(&self, _: GlobalRef, _: &crate::ir::global::Alias) {}
        fn read_func(&self, _: GlobalRef, _: &crate::ir::global::func::FuncData) {}
    }
    impl IInstVisitor for BasicValueFormatter<'_> {
        fn read_phi_end(&self, _: InstRef) {}
        fn read_phi_inst(&self, _: InstRef, _: &InstDataCommon, _: &PhiOp) {}
        fn read_unreachable_inst(&self, _: InstRef, _: &InstDataCommon) {}
        fn read_ret_inst(&self, _: InstRef, _: &InstDataCommon, _: &Ret) {}
        fn read_jump_inst(&self, _: InstRef, _: &InstDataCommon, _: &Jump) {}
        fn read_br_inst(&self, _: InstRef, _: &InstDataCommon, _: &Br) {}
        fn read_switch_inst(&self, _: InstRef, _: &InstDataCommon, _: &Switch) {}
        fn read_alloca_inst(&self, _: InstRef, _: &InstDataCommon, _: &Alloca) {}
        fn read_load_inst(&self, _: InstRef, _: &InstDataCommon, _: &LoadOp) {}
        fn read_store_inst(&self, _: InstRef, _: &InstDataCommon, _: &StoreOp) {}
        fn read_select_inst(&self, _: InstRef, _: &InstDataCommon, _: &SelectOp) {}
        fn read_bin_op_inst(&self, _: InstRef, _: &InstDataCommon, _: &BinOp) {}
        fn read_cmp_inst(&self, _: InstRef, _: &InstDataCommon, _: &CmpOp) {}
        fn read_cast_inst(&self, _: InstRef, _: &InstDataCommon, _: &CastOp) {}
        fn read_index_ptr_inst(&self, _: InstRef, _: &InstDataCommon, _: &IndexPtrOp) {}
        fn read_call_inst(&self, _: InstRef, _: &InstDataCommon, _: &CallOp) {}

        fn inst_visitor_dispatch(&self, inst_ref: InstRef, _: &InstData) {
            self.write_str("%");
            self.write_str(self.inst_id_map[inst_ref.get_handle()].to_string().as_str());
        }
    }
}

#[cfg(test)]
mod testing {
    use super::*;
    use crate::{
        ir::opcode::Opcode,
        typing::context::{PlatformPolicy, TypeContext},
    };

    fn create_func_main(module: &Module) -> GlobalRef {
        let type_ctx = module.type_ctx.as_ref();
        let func_type = type_ctx.make_func_type(
            vec![ValTypeID::Int(32), ValTypeID::Ptr].as_slice(),
            ValTypeID::Int(32),
            false,
        );
        let func_main_data =
            FuncData::new_with_unreachable(module, func_type, "main".into()).unwrap();
        let func_main_ref = module.insert_global(GlobalData::Func(func_main_data));

        // create a `return 0` instruction
        let (c, r) = Ret::new(module, ValueSSA::ConstData(ConstData::Int(32, 0)));
        let ret_inst = module.insert_inst(InstData::Ret(c, r));

        // Add the instruction `%3 = add i32 %0, 10`
        let (c, r) = BinOp::new_with_operands(
            module,
            Opcode::Add,
            ValueSSA::FuncArg(func_main_ref, 0),
            ValueSSA::ConstData(ConstData::Int(32, 10)),
        )
        .unwrap();
        let add_inst = module.insert_inst(InstData::BinOp(c, r));

        // find the entry and insert `return 0`
        let entry_block = {
            let func_main_data = module.get_global(func_main_ref);
            let func_main_data = match &*func_main_data {
                GlobalData::Func(f) => f,
                _ => panic!("Invalid global data kind: Not Function"),
            };
            func_main_data.get_entry()
        };

        module
            .get_block(entry_block)
            .set_terminator(module, ret_inst)
            .unwrap();

        // Then add the add instruction to the entry block
        module
            .get_block(entry_block)
            .build_add_inst(add_inst, module)
            .unwrap();
        func_main_ref
    }

    #[test]
    fn writer_test() {
        let platform = PlatformPolicy::new_host();
        let type_ctx = TypeContext::new_rc(platform);
        let module = Module::new("io.medihbt.WriterTest".into(), type_ctx);

        let main_func = create_func_main(&module);

        // write the module to file `io.medihbt.WriterTest.Basic.ll`
        let mut file = std::fs::File::create("target/io.medihbt.WriterTest.Basic.ll").unwrap();
        write_ir_module(&module, &mut file, false, false, true);

        // Find entry block of the function `main`. we'll use it later.
        let entry_block = {
            let main_func_data = module.get_global(main_func);
            let main_func_data = match &*main_func_data {
                GlobalData::Func(f) => f,
                _ => panic!("Invalid global data kind: Not Function"),
            };
            main_func_data.get_entry()
        };

        // Add an instruction to the function `main`.
        // Source code: `char c = argv[0][10];` with load-GEP-load.
        // Remusys IR:
        // ```llvm
        // %4 = load ptr, ptr %1, align 1
        // %5 = getelementptr i8, ptr %4, i32 10
        // %6 = load i8, ptr %5, align 1
        // ```
        let load_argv0 = {
            let (c, r) =
                LoadOp::new(&module, ValTypeID::Ptr, 8, ValueSSA::FuncArg(main_func, 1)).unwrap();
            module.insert_inst(InstData::Load(c, r))
        };
        let gep_index_argv = {
            let (c, r) = IndexPtrOp::new_from_indices(
                &module,
                ValTypeID::Int(8),
                8,
                8,
                ValueSSA::Inst(load_argv0),
                [ValueSSA::ConstData(ConstData::Int(32, 10))]
                    .iter()
                    .map(|v| *v),
            )
            .unwrap();
            module.insert_inst(InstData::IndexPtr(c, r))
        };
        let load_argv0_10 = {
            let (c, r) = LoadOp::new(
                &module,
                ValTypeID::Int(8),
                1,
                ValueSSA::Inst(gep_index_argv),
            )
            .unwrap();
            module.insert_inst(InstData::Load(c, r))
        };

        // insert into the entry block.
        module
            .get_block(entry_block)
            .build_add_inst(load_argv0, &module)
            .unwrap();
        module
            .get_block(entry_block)
            .build_add_inst(gep_index_argv, &module)
            .unwrap();
        module
            .get_block(entry_block)
            .build_add_inst(load_argv0_10, &module)
            .unwrap();

        // print the module to file `io.medihbt.WriterTest.LoadArgv.ll`
        let mut file = std::fs::File::create("target/io.medihbt.WriterTest.LoadArgv.ll").unwrap();
        write_ir_module(&module, &mut file, false, false, true);

        // Now expand the result to i32 and let return instruction use this value.
        // Source code: `return (int)c;` with cast.
        // Remusys IR:
        // ```llvm
        // %7 = zext i8 %6 to i32
        // ret i32 %7
        // ```

        let cast = {
            let (c, r) = CastOp::new(
                &module,
                Opcode::Zext,
                ValTypeID::Int(32),
                ValueSSA::Inst(load_argv0_10),
            )
            .unwrap();
            module.insert_inst(InstData::Cast(c, r))
        };
        module
            .get_block(entry_block)
            .build_add_inst(cast, &module)
            .unwrap();

        {
            let alloc_value = module.borrow_value_alloc();
            let alloc_inst = &alloc_value.alloc_inst;
            let alloc_block = &alloc_value.alloc_block;

            let terminator = entry_block
                .to_data(alloc_block)
                .get_termiantor(&module)
                .unwrap();

            match terminator.to_data(alloc_inst) {
                InstData::Ret(_, r) => {
                    r.retval.set_operand(&module, ValueSSA::Inst(cast));
                }
                _ => panic!("Invalid terminator type"),
            }
        };
        // write the module to file `io.medihbt.WriterTest.CastReturn.ll`
        let mut file = std::fs::File::create("target/io.medihbt.WriterTest.CastReturn.ll").unwrap();
        write_ir_module(&module, &mut file, false, false, false);
    }
}

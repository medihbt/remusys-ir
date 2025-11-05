use crate::{
    base::INullableValue,
    ir::{
        ArrayExpr, ArrayExprID, BlockID, BlockObj, ConstData, ExprID, ExprObj, FuncID, FuncObj,
        GlobalID, GlobalKind, GlobalObj, GlobalVar, IPtrUniqueUser, IPtrValue, IRAllocs,
        IRNumberValueMap, ISubExprID, ISubGlobal, ISubGlobalID, ISubInst, ISubInstID, ISubValueSSA,
        ITraceableValue, InstID, InstObj, JumpTargetKind, Module, NumberOption, PoolAllocatedID,
        PredList, UseID, UserList, ValueSSA, inst::*,
    },
    typing::{FPKind, IValType, ScalarType, TypeContext, ValTypeID},
};
use log::warn;
use std::{
    cell::{Cell, Ref, RefCell},
    collections::{BTreeMap, HashMap},
    io::Write,
};

pub struct IRWriterStat {
    curr_func: Cell<Option<FuncID>>,
}

#[derive(Debug, Clone, Copy, Default)]
pub struct IRWriteOption {
    pub show_ptrid: bool,
    pub show_users: bool,
    pub show_preds: bool,
}
impl IRWriteOption {
    pub fn loud() -> Self {
        Self { show_ptrid: true, show_users: true, show_preds: true }
    }
    pub fn quiet() -> Self {
        Self { show_ptrid: false, show_users: false, show_preds: false }
    }
}

impl IRWriterStat {
    pub fn new() -> Self {
        Self { curr_func: Cell::new(None) }
    }

    pub fn hold_curr_func<'stat>(&'stat self, funcid: FuncID) -> impl Drop + 'stat {
        let prev_func = self.curr_func.replace(Some(funcid));
        struct Guard<'stat> {
            stat: &'stat IRWriterStat,
            prev_func: Option<FuncID>,
        }
        impl<'stat> Drop for Guard<'stat> {
            fn drop(&mut self) {
                self.stat.curr_func.set(self.prev_func);
            }
        }
        Guard { stat: self, prev_func }
    }
}

pub struct IRWriter<'ir> {
    pub output: RefCell<&'ir mut dyn Write>,
    pub stat: IRWriterStat,
    pub option: IRWriteOption,
    pub tctx: &'ir TypeContext,
    pub allocs: &'ir IRAllocs,
    pub indent: Cell<usize>,
    pub numbering: RefCell<Option<IRNumberValueMap>>,
    pub globals: Vec<(GlobalID, GlobalKind)>,
    type_names: RefCell<HashMap<ValTypeID, String>>,
    str_literals: RefCell<BTreeMap<ArrayExprID, String>>,
}

impl<'ir> IRWriter<'ir> {
    pub fn from_module(output: &'ir mut dyn Write, module: &'ir Module) -> Self {
        Self {
            output: RefCell::new(output),
            stat: IRWriterStat::new(),
            option: IRWriteOption::default(),
            tctx: &module.tctx,
            allocs: &module.allocs,
            indent: Cell::new(0),
            numbering: RefCell::new(None),
            globals: Self::make_symbols(module),
            type_names: RefCell::new(HashMap::new()),
            str_literals: RefCell::new(BTreeMap::new()),
        }
    }

    fn make_symbols(module: &'ir Module) -> Vec<(GlobalID, GlobalKind)> {
        let symbols = module.symbols.borrow();
        let allocs = &module.allocs;
        let mut globals = Vec::with_capacity(symbols.len());
        for (_, gid) in symbols.iter() {
            globals.push((*gid, gid.get_kind(allocs)));
        }
        globals.sort_unstable_by(|&(lp, lk), &(rp, rk)| {
            lk.cmp(&rk)
                .then_with(|| lp.get_name(allocs).cmp(rp.get_name(allocs)))
        });
        globals
    }

    pub fn write(&self, buf: &[u8]) -> std::io::Result<usize> {
        self.output.borrow_mut().write(buf)
    }
    pub fn flush(&self) -> std::io::Result<()> {
        self.output.borrow_mut().flush()
    }
    pub fn write_str(&self, s: &str) -> std::io::Result<()> {
        self.output.borrow_mut().write_all(s.as_bytes())
    }
    pub fn write_all(&self, buf: &[u8]) -> std::io::Result<()> {
        self.output.borrow_mut().write_all(buf)
    }
    pub fn write_fmt(&self, args: std::fmt::Arguments) -> std::io::Result<()> {
        let mut output = self.output.borrow_mut();
        output.write_fmt(args)
    }
    pub fn numbers(&self) -> Ref<'_, IRNumberValueMap> {
        Ref::map(self.numbering.borrow(), |opt| {
            opt.as_ref().expect("IRWriter numbering is not set")
        })
    }
    pub fn set_numbers(&self, func: FuncID) {
        let option = NumberOption::ignore_all();
        self.numbering
            .replace(IRNumberValueMap::new(self.allocs, func, option));
    }

    pub fn inc_indent(&self) {
        self.indent.set(self.indent.get() + 1);
    }
    pub fn dec_indent(&self) {
        let curr = self.indent.get();
        assert!(curr > 0, "Indent level cannot be negative");
        self.indent.set(curr - 1);
    }
    pub fn wrap_indent(&self) {
        self.write_str("\n").unwrap();
        for _ in 0..self.indent.get() {
            self.write_str("    ").unwrap();
        }
    }
    pub fn write_type(&self, ty: ValTypeID) -> std::io::Result<()> {
        let mut type_names = self.type_names.borrow_mut();
        if let Some(name) = type_names.get(&ty) {
            return self.write_str(name);
        }
        match ty {
            ValTypeID::Void => self.write_str("void"),
            ValTypeID::Ptr => self.write_str("ptr"),
            ValTypeID::Int(bits) => write!(self, "i{bits}"),
            ValTypeID::Float(FPKind::Ieee32) => self.write_str("float"),
            ValTypeID::Float(FPKind::Ieee64) => self.write_str("double"),
            ValTypeID::StructAlias(sa) => write!(self, "%{}", sa.get_name(self.tctx)),
            _ => {
                let name = ty.get_display_name(self.tctx);
                self.write_str(&name)?;
                type_names.insert(ty, name);
                Ok(())
            }
        }
    }

    pub fn write_operand(&self, operand: impl ISubValueSSA) -> std::io::Result<()> {
        match operand.into_ir() {
            ValueSSA::None => self.write_str("poison"),
            ValueSSA::ConstData(c) => self.format_const_data(c),
            ValueSSA::ConstExpr(e) => self.format_expr(e),
            ValueSSA::AggrZero(_) => self.write_str("zeroinitializer"),
            ValueSSA::FuncArg(funcid, argid) => {
                assert_eq!(
                    Some(funcid),
                    self.stat.curr_func.get(),
                    "FuncArg can only be used in its own function"
                );
                write!(self, "%{argid}")
            }
            ValueSSA::Block(b) => self.write_block_operand(Some(b)),
            ValueSSA::Inst(i) => self.write_inst_operand(i),
            ValueSSA::Global(g) => {
                write!(self, "@{}", g.get_name(self.allocs))
            }
        }
    }
    fn write_inst_operand(&self, i: InstID) -> std::io::Result<()> {
        let numbers = self.numbering.borrow();
        let Some(numbers) = &*numbers else {
            warn!("Instruction can only be used in its own function");
            return write!(self, "%inst:{:#x}", i.get_indexed(self.allocs).0);
        };
        if let Some(id) = numbers.inst_get_number(i) {
            write!(self, "%{id}")
        } else {
            write!(self, "%inst:{:#x}", i.get_indexed(self.allocs).0)
        }
    }
    fn write_block_operand(&self, b: Option<BlockID>) -> std::io::Result<()> {
        let Some(b) = b else {
            return self.write_str("%NULL_BLOCK");
        };
        let numbers = self.numbering.borrow();
        let Some(numbers) = &*numbers else {
            warn!("Block can only be used in its own function");
            return write!(self, "%block:{:#x}", b.get_indexed(self.allocs).0);
        };
        if let Some(id) = numbers.block_get_number(b) {
            write!(self, "%{id}")
        } else {
            write!(self, "%block:{:#x}", b.get_indexed(self.allocs).0)
        }
    }
    fn format_const_data(&self, data: ConstData) -> std::io::Result<()> {
        match data {
            ConstData::Undef(_) => self.write_str("undef"),
            ConstData::Zero(ty) => match ty {
                ScalarType::Ptr => self.write_str("null"),
                ScalarType::Int(_) => self.write_str("0"),
                ScalarType::Float(_) => self.write_str("0.0"),
            },
            ConstData::PtrNull(_) => self.write_str("null"),
            ConstData::Int(apint) => {
                if apint.bits() == 1 {
                    write!(self, "{}", !apint.is_zero())
                } else {
                    write!(self, "{}", apint.as_signed())
                }
            }
            ConstData::Float(FPKind::Ieee32, fp) => {
                write!(self.output.borrow_mut(), "{:.20e}", fp as f32)
            }
            ConstData::Float(FPKind::Ieee64, fp) => {
                write!(self.output.borrow_mut(), "{:.20e}", fp)
            }
        }
    }
    fn format_expr(&self, expr: ExprID) -> std::io::Result<()> {
        if expr.is_zero_const(self.allocs) {
            return self.write_str("zeroinitializer");
        }
        let (elems, begin_s, end_s) = match expr.deref_ir(self.allocs) {
            ExprObj::Array(a) => {
                if self.try_write_string(ArrayExprID(expr), a)? {
                    return Ok(());
                } else {
                    (a.elems.as_slice(), "[", "]")
                }
            }
            ExprObj::Struct(s) => (s.fields.as_slice(), "{", "}"),
            ExprObj::FixVec(v) => (v.elems.as_slice(), "<", ">"),
        };
        self.format_aggregate(elems, begin_s, end_s)
    }
    fn try_write_string(&self, aid: ArrayExprID, a: &ArrayExpr) -> std::io::Result<bool> {
        let ValTypeID::Int(8) = a.elemty else {
            return Ok(false);
        };
        if let Some(cached) = self.str_literals.borrow().get(&aid) {
            self.write_str(cached)?;
            return Ok(true);
        }
        let bytes = {
            use std::fmt::Write;
            let mut bytes = String::with_capacity(a.elems.len() + 4);
            bytes.push_str("c\"");
            for (_, u) in a.user_iter(self.allocs) {
                let ValueSSA::ConstData(cx) = u.operand.get() else {
                    return Ok(false);
                };
                let ch = match cx {
                    ConstData::Int(ch) => ch.as_unsigned() as u8,
                    ConstData::Zero(_) => 0u8,
                    _ => return Ok(false),
                };
                match ch {
                    b'"' => bytes.push_str("\\22"),
                    b'\\' => bytes.push_str("\\5c"),
                    0x20..=0x7e if ch.is_ascii_graphic() => bytes.push(ch as char),
                    b' ' => bytes.push(' '),
                    _ => write!(bytes, "\\{ch:02x}").unwrap(),
                }
            }
            bytes.push('"');
            bytes
        };
        self.write_str(&bytes)?;
        self.str_literals.borrow_mut().insert(aid, bytes);
        Ok(true)
    }
    fn format_aggregate(&self, elems: &[UseID], begin_s: &str, end_s: &str) -> std::io::Result<()> {
        self.write_str(begin_s)?;
        let allocs = self.allocs;
        for (i, useid) in elems.iter().enumerate() {
            if i > 0 {
                self.write_str(", ")?;
            }
            let operand = useid.get_operand(allocs);
            self.write_type(operand.get_valtype(allocs))?;
            self.write_operand(operand)?;
        }
        self.write_str(end_s)
    }
    fn writeln_entity_id(&self, id: impl Into<PoolAllocatedID>) -> std::io::Result<()> {
        if !self.option.show_ptrid {
            return Ok(());
        }
        let id = id.into();
        let indexed = id.get_indexed(self.allocs);
        let prefix = match id {
            PoolAllocatedID::Block(_) => "; .id = %block:",
            PoolAllocatedID::Inst(_) => "; .id = %inst:",
            PoolAllocatedID::Expr(_) => "; .id = %expr:",
            PoolAllocatedID::Global(_) => "; .id = %global:",
            PoolAllocatedID::Use(_) => "; .id = %use:",
            PoolAllocatedID::JumpTarget(_) => "; .id = %jt:",
        };
        if let Some(index) = indexed {
            write!(self, "{prefix}{:#x}, addr = ", index)?;
        } else {
            write!(self, "{prefix}<INVALID>, addr = ")?;
        }
        match id {
            PoolAllocatedID::Block(b) => write!(self, "{:p}", b.inner()),
            PoolAllocatedID::Inst(i) => write!(self, "{i:p}"),
            PoolAllocatedID::Expr(e) => write!(self, "{e:p}"),
            PoolAllocatedID::Global(g) => write!(self, "{g:p}"),
            PoolAllocatedID::Use(u) => write!(self, "{:p}", u.inner()),
            PoolAllocatedID::JumpTarget(jt) => write!(self, "{:p}", jt.inner()),
        }?;
        self.wrap_indent();
        Ok(())
    }
    fn writeln_users(&self, users: &UserList) -> std::io::Result<()> {
        let alloc_use = &self.allocs.uses;
        if !self.option.show_users || users.is_empty(alloc_use) {
            return Ok(());
        }
        self.write_str("; users = [")?;
        let mut first = true;
        for (_, u) in users.iter(alloc_use) {
            if !first {
                self.write_str(", ")?;
            }
            first = false;
            let Some(user) = u.user.get() else {
                self.write_str("%NULL_USER")?;
                continue;
            };
            write!(self, "({:?}, ", u.get_kind())?;
            self.write_operand(user)?;
            self.write_str(")")?;
        }
        self.write_str("]")?;
        self.wrap_indent();
        Ok(())
    }
    fn writeln_preds(&self, preds: &PredList) -> std::io::Result<()> {
        let alloc_jt = &self.allocs.jts;
        if !self.option.show_preds || preds.is_empty(alloc_jt) {
            return Ok(());
        }
        self.write_str("; preds = [")?;
        let mut first = true;
        for (_, p) in preds.iter(alloc_jt) {
            if !first {
                self.write_str(", ")?;
            }
            first = false;
            let Some(pred) = p.terminator.get() else {
                self.write_str("%NULL_TERMINATOR")?;
                continue;
            };
            write!(self, "({:?}, ", p.get_kind())?;
            self.write_operand(pred)?;
            self.write_str(")")?;
        }
        self.write_str("]")?;
        self.wrap_indent();
        Ok(())
    }

    pub fn write_module(&self) {
        // %{name} = type {struct}
        self.tctx.foreach_aliases(|name, _, aliasee| {
            write!(self, "%{name} = type ").unwrap();
            self.write_type(aliasee.into_ir()).unwrap();
            self.wrap_indent();
        });

        let Some((_, mut curr_kind)) = self.globals.first().copied() else {
            return;
        };
        for &(gid, gkind) in &self.globals {
            if gkind != curr_kind {
                self.wrap_indent();
                curr_kind = gkind;
            }
            let gobj = gid.deref_ir(self.allocs);
            self.writeln_entity_id(gid).unwrap();
            self.writeln_users(gobj.users()).unwrap();
            match gobj {
                GlobalObj::Func(f) => self.format_func(FuncID(gid), f),
                GlobalObj::Var(g) => self.format_global_var(g),
            }
            self.wrap_indent();
        }
    }
    fn format_global_var(&self, gvar: &GlobalVar) {
        let name = gvar.get_name();
        let prefix = gvar.get_linkage_prefix(self.allocs);
        write!(self, "@{name} = {prefix} ").unwrap();
        self.write_type(gvar.get_ptr_pointee_type()).unwrap();

        if let Some(init) = gvar.get_init(self.allocs).to_option() {
            self.write_str(" ").unwrap();
            self.write_operand(init).unwrap();
        };
        write!(self, ", align {}", gvar.get_ptr_pointee_align()).unwrap();
    }
    /// Syntax:
    ///
    /// ```llvm
    /// ; for external function
    /// declare [linkage] [ret_type] @function_name(<arg_ty0>, <arg_ty1>, [...]) [ret_type_attributes]
    ///
    /// ; for defined function
    /// define [linkage] [ret_type] @function_name(<arg_ty0> %0, <arg_ty1> %1, [...]) [ret_type_attributes] {
    ///     ; function body
    /// }
    /// ```
    fn format_func(&self, func_id: FuncID, func: &FuncObj) {
        let _stat = self.stat.hold_curr_func(func_id);
        let name = func.get_name();
        self.write_str(func.get_linkage_prefix(self.allocs))
            .unwrap();
        self.write_str(" ").unwrap();
        self.write_type(func.ret_type).unwrap();
        self.write_str(" @").unwrap();
        self.write_str(name).unwrap();
        self.write_str("(").unwrap();
        let is_extern = func.is_extern(self.allocs);
        for arg in &func.args {
            if arg.index > 0 {
                self.write_str(", ").unwrap();
            }
            self.write_type(arg.ty).unwrap();
            if !is_extern {
                write!(self, " %{}", arg.index).unwrap();
            }
        }
        if func.is_vararg {
            let prompt = if func.args.is_empty() { "..." } else { ", ..." };
            self.write_str(prompt).unwrap();
        }
        self.write_str(")").unwrap();

        if is_extern {
            self.write_str(" ; extern").unwrap();
            self.wrap_indent();
            return;
        }
        let Some(body) = &func.body else {
            panic!("Function body must be present for defined function {name}");
        };

        self.write_str(" {").unwrap();
        self.set_numbers(func_id);
        self.format_block(body.entry, body.entry.deref_ir(self.allocs));
        self.wrap_indent();
        for (block_id, block) in body.blocks.iter(&self.allocs.blocks) {
            let block_id = BlockID(block_id);
            if block_id == body.entry {
                continue;
            }
            self.format_block(block_id, block);
            self.wrap_indent();
        }
        self.write_str("}").unwrap();
    }

    fn format_block(&self, block_id: BlockID, block: &BlockObj) {
        let number = self.numbers().block_get_number(block_id);
        if number.is_some() {
            self.wrap_indent();
        }
        self.writeln_entity_id(block_id).unwrap();
        self.writeln_users(block.users()).unwrap();
        self.writeln_preds(block.get_preds()).unwrap();
        if let Some(number) = number {
            write!(self, "{number}:").unwrap();
        }

        self.inc_indent();
        let insts = block.get_body().insts.iter(&self.allocs.insts);
        for (inst_id, inst) in insts {
            self.wrap_indent();
            let number = self.numbers().inst_get_number(inst_id);
            self.format_inst(inst_id, inst, number);
        }
        self.dec_indent();
    }
    fn format_inst(&self, inst_id: InstID, inst: &InstObj, number: Option<usize>) {
        if let InstObj::PhiInstEnd(_) = inst {
            // PhiInstEnd 不占用编号
        } else {
            self.writeln_entity_id(inst_id).unwrap();
            self.writeln_users(inst.users()).unwrap();
        }
        match inst {
            InstObj::GuideNode(_) => {}
            InstObj::PhiInstEnd(_) => {
                let id = inst_id.get_indexed(self.allocs);
                write!(self, ";=====:: Phi Inst End Node (id:{:#x}) ::=====", id.0).unwrap()
            }
            InstObj::Unreachable(_) => self.write_str("unreachable").unwrap(),
            InstObj::Ret(ret_inst) => {
                self.write_str("ret ").unwrap();
                let ret_type = ret_inst.get_valtype();
                if ret_type == ValTypeID::Void {
                    self.write_str("void").unwrap();
                } else {
                    self.write_type(ret_type).unwrap();
                    self.write_str(" ").unwrap();
                    self.write_operand(ret_inst.get_retval(self.allocs))
                        .unwrap();
                }
            }
            InstObj::Jump(jump) => {
                self.write_str("br label ").unwrap();
                self.write_block_operand(jump.get_target(self.allocs))
                    .unwrap();
            }
            InstObj::Br(br_inst) => {
                self.write_str("br i1 ").unwrap();
                self.write_operand(br_inst.get_cond(self.allocs)).unwrap();
                self.write_str(", label ").unwrap();
                self.write_block_operand(br_inst.get_then(self.allocs))
                    .unwrap();
                self.write_str(", label ").unwrap();
                self.write_block_operand(br_inst.get_else(self.allocs))
                    .unwrap();
            }
            InstObj::Switch(switch) => self.format_switch_inst(switch),
            InstObj::Alloca(alloca) => {
                if let Some(number) = number {
                    write!(self, "%{number} = ").unwrap();
                }
                self.write_str("alloca ").unwrap();
                self.write_type(alloca.pointee_ty).unwrap();
                write!(self, ", align {}", alloca.get_ptr_pointee_align()).unwrap();
            }
            InstObj::GEP(gep) => self.format_gep_inst(number, gep),
            InstObj::Load(load) => self.format_load_inst(number, load),
            InstObj::Store(store) => self.format_store_inst(store),
            InstObj::AmoRmw(amo_rmw) => self.format_amormw_inst(number, amo_rmw),
            InstObj::BinOP(binop) => self.format_binop_inst(number, binop),
            InstObj::Call(call) => self.format_call_inst(number, call),
            InstObj::Cast(cast) => self.format_cast_inst(number, cast),
            InstObj::Cmp(cmp) => self.format_cmp_inst(number, cmp),
            InstObj::IndexExtract(index_extract) => {
                self.format_index_extract_inst(number, index_extract)
            }
            InstObj::FieldExtract(field_extract) => {
                self.format_field_extract_inst(number, field_extract)
            }
            InstObj::IndexInsert(index_insert) => {
                self.format_index_insert_inst(number, index_insert)
            }
            InstObj::FieldInsert(field_insert) => {
                self.format_field_insert_inst(number, field_insert)
            }
            InstObj::Phi(phi) => self.format_phi_inst(number, phi),
            InstObj::Select(select) => self.format_select_inst(number, select),
        }
    }

    /// ```llvm
    /// switch <intty> <value>, label <defaultdest> [
    ///     <intty> <val0>, label <dest0>
    ///     <intty> <val1>, label <dest1>
    ///     <intty> <val2>, label <dest2>
    ///     ...
    /// ]
    /// ```
    fn format_switch_inst(&self, switch: &SwitchInst) {
        let allocs = self.allocs;
        self.write_str("switch ").unwrap();
        // 写入条件操作数的类型和值
        let cond = switch.get_discrim(self.allocs);
        let cond_type = cond.get_valtype(self.allocs);
        self.write_type(cond_type).unwrap();
        self.write_str(" ").unwrap();
        self.write_operand(cond).unwrap();
        // 写入默认目标块
        self.write_str(", label ").unwrap();
        self.write_block_operand(switch.get_default_bb(self.allocs))
            .unwrap();
        // 写入各个分支目标块
        self.write_str(" [").unwrap();
        self.inc_indent();
        for &case in &*switch.case_jts() {
            self.wrap_indent();
            let JumpTargetKind::SwitchCase(case_val) = case.get_kind(allocs) else {
                panic!("Invalid JumpTargetKind in Switch instruction");
            };
            self.write_type(cond_type).unwrap();
            write!(self, " {case_val}, label ").unwrap();
            self.write_block_operand(case.get_block(self.allocs))
                .unwrap();
        }
        self.dec_indent();
        self.wrap_indent();
        self.write_str("]").unwrap();
    }

    /// ```llvm
    /// getelementptr inbounds <1st unpacked ty>, ptr %<ptr>, <intty0> <sindex0>, <intty1> <sindex1>, ...
    /// ```
    fn format_gep_inst(&self, number: Option<usize>, gep: &GEPInst) {
        let allocs = self.allocs;
        if let Some(number) = number {
            write!(self, "%{number} = ").unwrap();
        }
        self.write_str("getelementptr ").unwrap();
        if gep.get_inbounds() {
            self.write_str("inbounds ").unwrap();
        }
        self.write_type(gep.initial_ty).unwrap();
        self.write_str(" ptr ").unwrap();
        self.write_operand(gep.get_base(self.allocs)).unwrap();
        for &index_use in gep.index_uses() {
            let index = index_use.get_operand(allocs);
            let index_ty = index.get_valtype(allocs);
            let ValTypeID::Int(_) = index_ty else {
                panic!("GEP index must be integer type but got {index_ty:?}");
            };
            self.write_str(", ").unwrap();
            self.write_type(index_ty).unwrap();
            self.write_str(" ").unwrap();
            self.write_operand(index).unwrap();
        }
    }

    /// ```llvm
    /// %<result> = load <ty>, ptr <pointer>, align <alignment>
    /// ```
    fn format_load_inst(&self, number: Option<usize>, load: &LoadInst) {
        if let Some(number) = number {
            write!(self, "%{number} = ").unwrap();
        }
        self.write_str("load ").unwrap();
        let pointee_ty = load.get_valtype();
        self.write_type(pointee_ty).unwrap();
        self.write_str(", ptr ").unwrap();
        self.write_operand(load.get_source(self.allocs)).unwrap();
        write!(self, ", align {}", load.get_operand_pointee_align()).unwrap();
    }

    /// ```llvm
    /// store <ty> <value>, ptr <pointer>, align <alignment>
    /// ```
    fn format_store_inst(&self, store: &StoreInst) {
        let allocs = self.allocs;
        self.write_str("store ").unwrap();
        self.write_type(store.source_ty).unwrap();
        self.write_str(" ").unwrap();
        self.write_operand(store.get_source(allocs)).unwrap();
        self.write_str(", ptr ").unwrap();
        self.write_operand(store.get_target(allocs)).unwrap();
        write!(self, ", align {}", store.get_operand_pointee_align()).unwrap();
    }

    /// ```llvm
    /// %id = atomicrmw [volatile] <operation> ptr <pointer>, <ty> <value> [syncscope("<target-scope>")] <ordering>[, align <alignment>]  ; yields ty
    /// ```
    fn format_amormw_inst(&self, number: Option<usize>, amo_rmw: &AmoRmwInst) {
        if let Some(number) = number {
            write!(self, "%{number} = ").unwrap();
        }
        self.write_str("atomicrmw ").unwrap();
        if amo_rmw.is_volatile {
            self.write_str("volatile ").unwrap();
        }
        write!(self, "{} ptr ", amo_rmw.subop_name()).unwrap();
        self.write_operand(amo_rmw.get_pointer(self.allocs))
            .unwrap();
        self.write_str(", ").unwrap();
        self.write_type(amo_rmw.value_ty).unwrap();
        self.write_str(" ").unwrap();
        self.write_operand(amo_rmw.get_value(self.allocs)).unwrap();
        if amo_rmw.scope != SyncScope::System {
            write!(self, " syncscope(\"{}\")", amo_rmw.scope.as_str()).unwrap();
        }
        write!(self, " {}", amo_rmw.ordering.as_str()).unwrap();
        if amo_rmw.align_log2 > 0 {
            write!(self, ", align {}", 1 << amo_rmw.align_log2).unwrap();
        }
    }

    /// ```llvm
    /// %<result> = <opcode> <ty> <op1>, <op2>
    /// ```
    fn format_binop_inst(&self, number: Option<usize>, binop: &BinOPInst) {
        let allocs = self.allocs;
        if let Some(number) = number {
            write!(self, "%{number} = ").unwrap();
        }
        write!(self, "{} ", binop.get_opcode().get_name()).unwrap();
        self.write_type(binop.get_valtype()).unwrap();
        self.write_str(" ").unwrap();
        self.write_operand(binop.get_lhs(allocs)).unwrap();
        self.write_str(", ").unwrap();
        self.write_operand(binop.get_rhs(allocs)).unwrap();
    }

    /// ```llvm
    /// ; has retval:
    /// %result = call <ret_type> @function_name(<arg_types>, ...)
    /// ; has retval and is vararg:
    /// %result = call <ret_type> (...) @function_name(<arg_types>, ...)
    /// ; returns void:
    /// call void @function_name(<arg_types>, ...)
    /// ; is vararg:
    /// call (...) @function_name(<arg_types>, ...)
    /// ```
    fn format_call_inst(&self, number: Option<usize>, call: &CallInst) {
        let allocs = self.allocs;
        if let Some(number) = number {
            write!(self, "%{number} = ").unwrap();
        }
        self.write_str("call ").unwrap();
        self.write_type(call.get_valtype()).unwrap();
        if call.is_vararg {
            self.write_str(" (...)").unwrap();
        }

        self.write_str(" ").unwrap();
        self.write_operand(call.get_callee(allocs)).unwrap();
        self.write_str("(").unwrap();
        for (i, arg_use) in call.arg_uses().iter().enumerate() {
            if i > 0 {
                self.write_str(", ").unwrap();
            }
            let arg = arg_use.get_operand(allocs);
            let arg_ty = call
                .callee_ty
                .get_args(self.tctx)
                .get(i)
                .copied()
                .unwrap_or(arg.get_valtype(allocs));
            self.write_type(arg_ty).unwrap();
            self.write_str(" ").unwrap();
            self.write_operand(arg).unwrap();
        }
        self.write_str(")").unwrap();
    }

    /// ```llvm
    /// %<result> = <op> <type> <value> to <type>
    /// ```
    fn format_cast_inst(&self, number: Option<usize>, cast: &CastInst) {
        let allocs = self.allocs;
        if let Some(number) = number {
            write!(self, "%{number} = ").unwrap();
        }
        write!(self, "{} ", cast.get_opcode().get_name()).unwrap();
        self.write_type(cast.from_ty).unwrap();
        self.write_str(" ").unwrap();
        self.write_operand(cast.get_from(allocs)).unwrap();
        self.write_str(" to ").unwrap();
        self.write_type(cast.get_valtype()).unwrap();
    }

    /// ```llvm
    /// %<result> = <op> <cond> <type> <lhs>, <rhs>
    /// ```
    fn format_cmp_inst(&self, number: Option<usize>, cmp: &CmpInst) {
        let allocs = self.allocs;
        if let Some(number) = number {
            write!(self, "%{number} = ").unwrap();
        }
        let opcode = cmp.get_opcode().get_name();
        let cond = cmp.cond;
        write!(self, "{opcode} {cond} ").unwrap();
        self.write_type(cmp.operand_ty).unwrap();
        self.write_str(" ").unwrap();
        self.write_operand(cmp.get_lhs(allocs)).unwrap();
        self.write_str(", ").unwrap();
        self.write_operand(cmp.get_rhs(allocs)).unwrap();
    }

    /// ```llvm
    /// %<id> = extractelement <aggr_type> %<aggr>, <index_ty> %<index>
    /// ```
    fn format_index_extract_inst(&self, number: Option<usize>, inst: &IndexExtractInst) {
        let allocs = self.allocs;
        if let Some(number) = number {
            write!(self, "%{number} = ").unwrap();
        }
        self.write_str("extractelement ").unwrap();
        self.write_type(inst.aggr_ty.into_ir()).unwrap();
        self.write_str(" ").unwrap();
        self.write_operand(inst.get_aggr(allocs)).unwrap();
        self.write_str(", ").unwrap();
        let index = inst.get_index(allocs);
        let index_ty = index.get_valtype(allocs);
        self.write_type(index_ty).unwrap();
        self.write_str(" ").unwrap();
        self.write_operand(index).unwrap();
    }

    /// ```llvm
    /// %<id> = extractvalue <aggr_type> %a, <field_idx0>, <field_idx1>, ...
    /// ```
    fn format_field_extract_inst(&self, number: Option<usize>, inst: &FieldExtractInst) {
        let allocs = self.allocs;
        if let Some(number) = number {
            write!(self, "%{number} = ").unwrap();
        }
        self.write_str("extractvalue ").unwrap();
        self.write_type(inst.aggr_type.into_ir()).unwrap();
        self.write_str(" ").unwrap();
        self.write_operand(inst.get_aggr(allocs)).unwrap();
        for &idx in inst.get_field_indices() {
            write!(self, ", {idx}").unwrap();
        }
    }

    /// ```llvm
    /// %<result> = insertelement <aggr_type> %<aggr>, <elem_type> %<elem>, <index_type> %<index>
    /// ```
    fn format_index_insert_inst(&self, number: Option<usize>, inst: &IndexInsertInst) {
        let allocs = self.allocs;
        if let Some(number) = number {
            write!(self, "%{number} = ").unwrap();
        }
        self.write_str("insertelement ").unwrap();
        // aggregate
        self.write_type(inst.get_valtype()).unwrap();
        self.write_str(" ").unwrap();
        self.write_operand(inst.get_aggr(allocs)).unwrap();
        // element
        self.write_str(", ").unwrap();
        self.write_type(inst.get_elem_type()).unwrap();
        self.write_str(" ").unwrap();
        self.write_operand(inst.get_elem(allocs)).unwrap();
        // index
        self.write_str(", ").unwrap();
        let index = inst.get_index(allocs);
        let index_ty = index.get_valtype(allocs);
        self.write_type(index_ty).unwrap();
        self.write_str(" ").unwrap();
        self.write_operand(index).unwrap();
    }

    /// ```llvm
    /// %<result> = insertvalue <aggr_type> %<aggr>, <elem_type> %<elem>, <idx0>, <idx1>, ...
    /// ```
    fn format_field_insert_inst(&self, number: Option<usize>, inst: &FieldInsertInst) {
        let allocs = self.allocs;
        if let Some(number) = number {
            write!(self, "%{number} = ").unwrap();
        }
        self.write_str("insertvalue ").unwrap();
        self.write_type(inst.get_valtype()).unwrap();
        self.write_str(" ").unwrap();
        self.write_operand(inst.get_aggr(allocs)).unwrap();
        self.write_str(", ").unwrap();
        self.write_type(inst.get_elem_type()).unwrap();
        self.write_str(" ").unwrap();
        self.write_operand(inst.get_elem(allocs)).unwrap();
        for &idx in inst.get_field_indices() {
            write!(self, ", {idx}").unwrap();
        }
    }

    /// ```llvm
    /// %<id> = phi <type> [ <value0>, %<label0> ], [ <value1>, %<label1> ], ...
    /// ```
    fn format_phi_inst(&self, number: Option<usize>, phi: &PhiInst) {
        let allocs = self.allocs;
        if let Some(number) = number {
            write!(self, "%{number} = ").unwrap();
        }
        self.write_str("phi ").unwrap();
        self.write_type(phi.get_valtype()).unwrap();
        let mut first = true;
        for slot_pair in phi.incoming_uses().iter() {
            if first {
                self.write_str(" ").unwrap();
            } else {
                self.write_str(", ").unwrap();
            }
            first = false;
            self.write_str("[").unwrap();
            // value
            let val = slot_pair[0].get_operand(allocs);
            self.write_operand(val).unwrap();
            self.write_str(", ").unwrap();
            // block label (without the 'label' keyword in phi)
            let bb_val = slot_pair[1].get_operand(allocs);
            let bb = match bb_val {
                ValueSSA::Block(b) => b,
                _ => panic!("Expected BlockID in Phi operand slot, found {bb_val:?}"),
            };
            self.write_block_operand(Some(bb)).unwrap();
            self.write_str("]").unwrap();
        }
    }

    /// ```llvm
    /// %<name> = select <type>, i1 <cond>, <true value>, <false value>
    /// ```
    fn format_select_inst(&self, number: Option<usize>, select: &SelectInst) {
        let allocs = self.allocs;
        if let Some(number) = number {
            write!(self, "%{number} = ").unwrap();
        }
        self.write_str("select ").unwrap();
        self.write_type(select.get_valtype()).unwrap();
        self.write_str(", i1 ").unwrap();
        self.write_operand(select.get_cond(allocs)).unwrap();
        self.write_str(", ").unwrap();
        self.write_operand(select.get_then(allocs)).unwrap();
        self.write_str(", ").unwrap();
        self.write_operand(select.get_else(allocs)).unwrap();
    }
}

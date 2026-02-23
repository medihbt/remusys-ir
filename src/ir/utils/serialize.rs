//! The IR serialization API, which converts the IR into a human-readable string format
//! and generate source mapping information if needed.

use smallvec::SmallVec;
use smol_str::{SmolStrBuilder, ToSmolStr, format_smolstr};

use crate::{
    SymbolStr,
    ir::{inst::*, *},
    typing::*,
};
use std::{collections::HashMap, io::Write, path::Path, rc::Rc};

pub fn module_tostring(module: &Module, option: IRWriteOption) -> IRWriteRes<String> {
    let placeholder = IRNameMap::default();
    module_tostring_named(module, &placeholder, option)
}
pub fn module_tostring_named(
    module: &Module,
    names: &IRNameMap,
    option: IRWriteOption,
) -> IRWriteRes<String> {
    let mut serializer = IRSerializer::new_buffered(module, names);
    serializer.set_options(option).fmt_module()?;
    Ok(serializer.extract_string())
}
pub fn module_tostring_mapped(
    module: &Module,
    names: &IRNameMap,
    option: IRWriteOption,
) -> IRWriteRes<(String, SourceRangeMap)> {
    let mut serializer = IRSerializer::new_buffered(module, names);
    serializer
        .enable_srcmap()
        .set_options(option)
        .fmt_module()?;
    let srcmap = serializer.writer.srcmap.take().unwrap();
    Ok((serializer.extract_string(), srcmap))
}
pub fn write_ir_to_file(path: impl AsRef<Path>, module: &Module, option: IRWriteOption) {
    let str = module_tostring(module, option).unwrap();
    if cfg!(not(miri)) {
        std::fs::write(path, str).unwrap();
    }
}

#[derive(Debug, thiserror::Error)]
pub enum IRWriteErr {
    #[error("IO Error: {0}")]
    IO(#[from] std::io::Error),

    #[error("Function '{0}' is external and cannot be written.")]
    FuncIsExtern(SymbolStr),

    #[error("JumpTarget CFG edge has no target block.")]
    JumpToNoTarget,

    #[error("Phi instruction has incoming value that is not a block. (real: {0:?})")]
    PhiIncomeNotBlock(ValueSSA),
}
pub type IRWriteRes<T = ()> = Result<T, IRWriteErr>;

#[derive(Debug, Clone, Copy, Default)]
pub struct IRWriteOption {
    pub show_indexed: bool,
    pub show_users: bool,
    pub show_preds: bool,
    pub mangle_unexported: bool,
    pub llvm_compatible: bool,
}
impl IRWriteOption {
    pub fn loud() -> Self {
        Self {
            show_indexed: true,
            show_users: true,
            show_preds: true,
            mangle_unexported: true,
            llvm_compatible: false,
        }
    }
    pub fn quiet() -> Self {
        Self::default()
    }

    pub fn show_indexed(self, val: bool) -> Self {
        Self { show_indexed: val, ..self }
    }
    pub fn show_users(self, val: bool) -> Self {
        Self { show_users: val, ..self }
    }
    pub fn show_preds(self, val: bool) -> Self {
        Self { show_preds: val, ..self }
    }
    pub fn unexported_name_mangle(self, val: bool) -> Self {
        Self { mangle_unexported: val, ..self }
    }
    pub fn llvm_compatible(self, val: bool) -> Self {
        Self { llvm_compatible: val, ..self }
    }
}

#[derive(Clone)]
enum NameMapRepr<'a> {
    Name(&'a IRNameMap),
    Number(Rc<FuncNumberMap<'a>>),
}
impl<'a> NameMapRepr<'a> {
    fn get_local_name(&self, val: impl IValueConvert) -> Option<SymbolStr> {
        match self {
            NameMapRepr::Name(m) => m.get_local_name(val),
            NameMapRepr::Number(m) => m.get_local_name(val),
        }
    }

    fn name_map(&self) -> &'a IRNameMap {
        match self {
            NameMapRepr::Name(m) => m,
            NameMapRepr::Number(m) => m.names,
        }
    }
}

#[derive(Clone)]
struct Env<'ir, 'names> {
    module: &'ir Module,
    names: NameMapRepr<'names>,
    option: IRWriteOption,
}

impl<'ir, 'names> Env<'ir, 'names> {
    fn nameof_global(&self, id: GlobalID) -> SymbolStr {
        let Self { module, option, .. } = self;
        let allocs = &module.allocs;
        let name = id.clone_name(allocs);
        if module.symbol_is_exported(id) || !option.mangle_unexported {
            name
        } else {
            let idx = id.get_entity_index(allocs);
            format_smolstr!("noexport.{name}.{idx:x}")
        }
    }
}

#[derive(Default)]
struct Cache {
    type_names: HashMap<ValTypeID, SymbolStr>,
    str_literals: HashMap<ExprID, Option<SymbolStr>>,
    llvm_map: LLVMAdaptMapping,
}

impl Cache {
    fn value_as_u8(v: ValueSSA) -> Option<u8> {
        v.as_apint().map(|a| a.as_unsigned() as u8)
    }
    fn expr_as_litstr(&mut self, env: &Env, expr: ExprID) -> Option<SymbolStr> {
        fn array_as_litstr(env: &Env, arr: &impl IArrayExpr) -> Option<SymbolStr> {
            let allocs = &env.module.allocs;
            let ValTypeID::Int(8) = arr.get_elem_type() else {
                return None;
            };
            let bytes = arr.value_iter(allocs).map(Cache::value_as_u8);
            bytes_as_litstr(bytes)
        }
        fn bytes_as_litstr(iter: impl Iterator<Item = Option<u8>>) -> Option<SymbolStr> {
            use std::fmt::Write;
            let mut buff = SmolStrBuilder::new();
            buff.push_str("c\"");
            for ch in iter {
                let ch = ch?;
                match ch {
                    b'"' => buff.push_str("\\22"),
                    b'\\' => buff.push_str("\\5c"),
                    0x20..=0x7e if ch.is_ascii_graphic() => buff.push(ch as char),
                    b' ' => buff.push(' '),
                    _ => write!(buff, "\\{ch:02x}").unwrap(),
                }
            }
            buff.push('"');
            Some(buff.finish())
        }

        if let Some(s) = self.str_literals.get(&expr) {
            return s.clone();
        }
        let allocs = &env.module.allocs;
        let s = match expr.deref_ir(allocs) {
            ExprObj::Array(a) => array_as_litstr(env, a),
            ExprObj::DataArray(da) => array_as_litstr(env, da),
            ExprObj::SplatArray(sa) => array_as_litstr(env, sa),
            ExprObj::KVArray(_) => None,
            ExprObj::Struct(_) | ExprObj::FixVec(_) => None,
        };
        self.str_literals.insert(expr, s.clone());
        s
    }
}

struct Writer<W: Write> {
    writer: SourceMapWriter<W>,
    srcmap: Option<SourceRangeMap>,
    indent: usize,
}
impl<W: Write> Writer<W> {
    fn new(writer: W) -> Self {
        Self {
            writer: SourceMapWriter::new(writer),
            indent: 0,
            srcmap: None,
        }
    }

    fn write_str(&mut self, s: &str) -> IRWriteRes {
        self.writer.write_all(s.as_bytes()).map_err(IRWriteErr::IO)
    }
    fn write_fmt(&mut self, args: std::fmt::Arguments) -> IRWriteRes {
        self.writer.write_fmt(args).map_err(IRWriteErr::IO)
    }

    /// write a list of spaces for indent.
    fn indent(&mut self) -> IRWriteRes {
        for _ in 0..self.indent {
            self.write_str("    ")?;
        }
        Ok(())
    }
    /// write a new line and a list of spaces for indent.
    fn wrap_indent(&mut self) -> IRWriteRes {
        self.write_str("\n")?;
        self.indent()
    }

    fn curr_pos(&self) -> IRSourcePos {
        self.writer.curr_pos
    }
}

pub struct FmtCtx<'ir, 'names, 'ctx, W: Write> {
    env: Env<'ir, 'names>,
    cache: &'ctx mut Cache,
    writer: &'ctx mut Writer<W>,
}
impl<'ir, 'names, 'ctx, W: Write> FmtCtx<'ir, 'names, 'ctx, W> {
    fn scoped_with_names<'sub, 'subnames>(
        &'sub mut self,
        names: NameMapRepr<'subnames>,
    ) -> FmtCtx<'ir, 'subnames, 'sub, W> {
        let env = Env { module: self.env.module, names, option: self.env.option };
        FmtCtx { env, cache: &mut *self.cache, writer: &mut *self.writer }
    }

    fn write_str(&mut self, s: &str) -> IRWriteRes {
        self.writer.write_str(s)
    }
    fn write_fmt(&mut self, args: std::fmt::Arguments<'_>) -> IRWriteRes {
        self.writer.write_fmt(args)
    }
    fn map_value(&mut self, v: ValueSSA) -> ValueSSA {
        let Self { cache, env, .. } = self;
        if env.option.llvm_compatible { cache.llvm_map.map_value(env.module, v) } else { v }
    }
    fn fmt_type(&mut self, ty: ValTypeID) -> IRWriteRes {
        let name = self.type_name(ty);
        self.write_str(&name)
    }
    fn type_name(&mut self, ty: ValTypeID) -> SymbolStr {
        let Self { cache, env, .. } = self;
        match ty {
            ValTypeID::Void => SymbolStr::new_inline("void"),
            ValTypeID::Ptr => SymbolStr::new_inline("ptr"),
            ValTypeID::Int(1) => SymbolStr::new_inline("i1"),
            ValTypeID::Int(8) => SymbolStr::new_inline("i8"),
            ValTypeID::Int(16) => SymbolStr::new_inline("i16"),
            ValTypeID::Int(32) => SymbolStr::new_inline("i32"),
            ValTypeID::Int(64) => SymbolStr::new_inline("i64"),
            ValTypeID::Float(FPKind::Ieee32) => SymbolStr::new_inline("float"),
            ValTypeID::Float(FPKind::Ieee64) => SymbolStr::new_inline("double"),
            ty => cache
                .type_names
                .entry(ty)
                .or_insert_with(|| ty.get_display_name(&env.module.tctx))
                .clone(),
        }
    }

    fn insert_range(&mut self, id: impl Into<PoolAllocatedID>, range: IRSourceRange) {
        let id = id.into();
        if let Some(srcmap) = &mut self.writer.srcmap {
            srcmap.insert_range(id, range);
        }
    }
    fn fmt_use(&mut self, use_id: UseID) -> IRWriteRes {
        let allocs = &self.env.module.allocs;
        let val = use_id.get_operand(allocs);
        let begin_pos = self.writer.curr_pos();
        self.fmt_value(val)?;
        let end_pos = self.writer.curr_pos();
        self.insert_range(use_id, (begin_pos, end_pos));
        Ok(())
    }
    fn fmt_jt(&mut self, jt_id: JumpTargetID) -> IRWriteRes {
        let allocs = &self.env.module.allocs;
        let begin_pos = self.writer.curr_pos();
        self.fmt_block_target(jt_id.get_block(allocs))?;
        let end_pos = self.writer.curr_pos();
        self.insert_range(jt_id, (begin_pos, end_pos));
        Ok(())
    }
    fn fmt_block_target(&mut self, bb: Option<BlockID>) -> IRWriteRes {
        let block_id = match bb {
            Some(bb) => ValueSSA::Block(bb),
            None => ValueSSA::None,
        };
        self.fmt_value_mapped(block_id)
    }

    fn fmt_value(&mut self, val: ValueSSA) -> IRWriteRes {
        let mapped = self.map_value(val);
        self.fmt_value_mapped(mapped)
    }

    fn fmt_value_mapped(&mut self, val: ValueSSA) -> IRWriteRes {
        let allocs = &self.env.module.allocs;
        if let Some(name) = self.env.names.get_local_name(val) {
            return write!(self, "%{name}");
        }
        match val {
            ValueSSA::None => self.write_str("none"),
            ValueSSA::AggrZero(_) => self.write_str("zeroinitializer"),
            ValueSSA::ConstData(cdata) => self.fmt_const_data(cdata),
            ValueSSA::ConstExpr(expr) => self.fmt_expr(expr),
            ValueSSA::FuncArg(func, id) => {
                write!(self, "%arg[{id}]:{func:?}")
            }
            ValueSSA::Block(block_id) => {
                write!(self, "%block:{:x}", block_id.get_entity_index(allocs))
            }
            ValueSSA::Inst(inst_id) => {
                write!(self, "%inst:{:x}", inst_id.get_entity_index(allocs))
            }
            ValueSSA::Global(global) => {
                write!(self, "@{}", self.env.nameof_global(global))
            }
        }
    }

    fn fmt_const_data(&mut self, data: ConstData) -> IRWriteRes {
        match data {
            ConstData::Undef(_) => self.write_str("undef"),
            ConstData::Zero(ty) => match ty {
                ScalarType::Ptr => self.write_str("null"),
                ScalarType::Int(_) => self.write_str("0"),
                ScalarType::Float(_) => self.write_str("0.0"),
            },
            ConstData::PtrNull => self.write_str("null"),
            ConstData::Int(apint) => {
                if apint.bits() == 1 {
                    write!(self, "{}", !apint.is_zero())
                } else {
                    write!(self, "{}", apint.as_signed())
                }
            }
            ConstData::Float(FPKind::Ieee32, fp) => {
                write!(self, "{:.20e}", fp as f32)
            }
            ConstData::Float(FPKind::Ieee64, fp) => write!(self, "{fp:.20e}"),
        }
    }

    fn fmt_aggr_values(
        &mut self,
        elems: impl IntoIterator<Item = ValueSSA>,
        begin_s: &str,
        end_s: &str,
    ) -> IRWriteRes {
        write!(self, "{begin_s} ")?;
        let allocs = &self.env.module.allocs;
        for (i, elem) in elems.into_iter().enumerate() {
            if i > 0 {
                self.write_str(", ")?;
            }
            let mapped = self.map_value(elem);
            self.fmt_type(mapped.get_valtype(allocs))?;
            self.write_str(" ")?;
            self.fmt_value_mapped(mapped)?;
        }
        write!(self, " {end_s}")
    }

    fn fmt_sparse_kvarr(&mut self, kv: &KVArrayExpr) -> IRWriteRes {
        let allocs = &self.env.module.allocs;
        let default_val = kv.get_default(allocs);

        self.write_str("sparse [ ")?;
        let mut first = true;
        for (idx, val, _) in kv.elem_iter(allocs) {
            if first {
                first = false;
            } else {
                self.write_str(", ")?;
            }
            write!(self, "[{idx}] = ")?;
            self.fmt_type(kv.elemty)?;
            self.write_str(" ")?;
            self.fmt_value(val)?;
        }

        if !first {
            self.write_str(", ")?;
        }
        self.write_str("..= ")?;
        self.fmt_type(kv.elemty)?;
        self.write_str(" ")?;
        self.fmt_value(default_val)?;
        self.write_str(" ]")
    }

    fn fmt_expr(&mut self, expr: ExprID) -> IRWriteRes {
        let (allocs, tctx) = (&self.env.module.allocs, &self.env.module.tctx);
        if expr.is_zero_const(allocs) {
            return self.write_str("zeroinitializer");
        }
        if let Some(s) = self.cache.expr_as_litstr(&self.env, expr) {
            return self.write_str(&s);
        }

        match expr.deref_ir(allocs) {
            ExprObj::Array(arr) => self.fmt_aggr_values(arr.value_iter(allocs), "[", "]"),
            ExprObj::DataArray(da) => {
                let data_iter = (0..da.data.len()).map(|n| da.data.index_get(n));
                self.fmt_aggr_values(data_iter, "[", "]")
            }
            ExprObj::SplatArray(splat) => {
                let elem = splat.get_elem(allocs);
                let data_iter = std::iter::repeat_n(elem, splat.get_nelems());
                self.fmt_aggr_values(data_iter, "[", "]")
            }
            ExprObj::KVArray(kv) => self.fmt_sparse_kvarr(kv),
            ExprObj::Struct(struc) => {
                let (begin_s, end_s) =
                    if struc.structty.is_packed(tctx) { ("<{", "}>") } else { ("{", "}") };
                let vals = struc.fields.iter().map(|u| u.get_operand(allocs));
                self.fmt_aggr_values(vals, begin_s, end_s)
            }
            ExprObj::FixVec(vec) => {
                let vals = vec.elems.iter().map(|u| u.get_operand(allocs));
                self.fmt_aggr_values(vals, "<", ">")
            }
        }
    }

    fn fmtln_preds(&mut self, block: &BlockObj) -> IRWriteRes {
        self.write_str("; preds = [")?;
        let allocs = &self.env.module.allocs;
        for (_, pred) in block.get_preds().iter(&allocs.jts) {
            let kind = pred.get_kind();
            let Some(termi) = pred.terminator.get() else {
                write!(self, "(kind={kind:?} with no terminator), ")?;
                continue;
            };
            let block = termi.get_parent(allocs);
            write!(self, "(kind={kind:?}, from=")?;
            self.fmt_block_target(block)?;
            self.write_str("), ")?;
        }
        self.write_str("]")?;
        self.writer.wrap_indent()
    }
    fn fmtln_users(&mut self, traceable: &dyn ITraceableValue) -> IRWriteRes {
        let allocs = &self.env.module.allocs;
        self.write_str("; users = [")?;
        for (_, uobj) in traceable.user_iter(allocs) {
            let kind = uobj.get_kind();
            let user = match uobj.user.get() {
                Some(u) => u.into_ir(),
                None => ValueSSA::None,
            };
            write!(self, "({kind:?}, user=")?;
            self.fmt_value_mapped(user)?;
            self.write_str("), ")?;
        }
        self.write_str("]")?;
        self.writer.wrap_indent()
    }

    fn fmt_attr(&mut self, attr: &Attribute) -> IRWriteRes {
        match attr {
            Attribute::NoUndef => self.write_str("noundef"),
            Attribute::IntExt(iext) => self.write_str(iext.as_str()),
            Attribute::PtrReadOnly => self.write_str("readonly"),
            Attribute::PtrNoCapture => self.write_str("nocapture"),
            Attribute::FuncNoReturn => self.write_str("noreturn"),
            Attribute::FuncInline(inline) => self.write_str(inline.as_str()),
            Attribute::FuncAlignStack(log2) => write!(self, "alignstack({})", 1 << log2),
            Attribute::FuncPure => self.write_str("pure"),
            Attribute::ArgPtrTarget(target) => {
                let (name, ty) = match *target {
                    PtrArgTargetAttr::ByRef(ty) => ("byref", ty),
                    PtrArgTargetAttr::ByVal(ty) => ("byval", ty),
                    PtrArgTargetAttr::DynArray(ty) => ("elementtype", ty),
                };
                let tyname = self.type_name(ty);
                write!(self, "{name}({tyname})")
            }
            Attribute::ArgPtrDerefBytes(nbytes) => write!(self, "dereferenceable({})", nbytes),
        }
    }
    fn fmt_attrs(&mut self, attrs: &AttrSet) -> IRWriteRes {
        for attr in attrs.iter() {
            self.fmt_attr(&attr)?;
            self.write_str(" ")?;
        }
        Ok(())
    }

    fn fmt_global(&mut self, global_id: GlobalID) -> IRWriteRes {
        let allocs = &self.env.module.allocs;
        match global_id.deref_ir(allocs) {
            GlobalObj::Var(gvar) => self.fmt_global_var(GlobalVarID::raw_from(global_id), gvar),
            GlobalObj::Func(func) => self.fmt_func(FuncID::raw_from(global_id), func),
        }
    }
    fn fmt_type_aliases(&mut self) -> IRWriteRes {
        let tctx = &self.env.module.tctx;
        let mut aliases = Vec::with_capacity(tctx.allocs.borrow().aliases.len());
        tctx.foreach_aliases(|name, _, sid| {
            aliases.push((name.clone(), sid.into_ir()));
        });
        for (name, sid) in aliases {
            let struc_name = self.type_name(sid.into_ir());
            write!(self, "%{name} = type {struc_name}")?;
            self.writer.wrap_indent()?;
        }
        Ok(())
    }

    fn fmt_module(&mut self) -> IRWriteRes {
        self.fmt_type_aliases()?;
        let module = self.env.module;
        let (allocs, symbols) = (&module.allocs, module.symbols.borrow());

        let globals = {
            let mut globs: SmallVec<[GlobalVarID; 16]> = symbols.var_pool.iter().copied().collect();
            globs.sort_by_key(|g| (g.get_kind(allocs), g.get_name(allocs)));
            globs
        };
        let funcs = {
            let mut funcs: SmallVec<[FuncID; 16]> = symbols.func_pool.iter().copied().collect();
            funcs.sort_by_key(|f| (f.get_kind(allocs), f.get_name(allocs)));
            funcs
        };
        drop(symbols);

        for gid in globals {
            self.fmt_global(gid.raw_into())?;
            self.writer.wrap_indent()?;
        }
        for fid in funcs {
            self.fmt_global(fid.raw_into())?;
            self.writer.wrap_indent()?;
        }
        self.writer.writer.flush().map_err(IRWriteErr::IO)
    }

    /// Syntax:
    ///
    /// ```llvm
    /// @global_name = [linkage] [type init_value | type], align <alignment>
    /// ```
    fn fmt_global_var(&mut self, id: GlobalVarID, gvar: &GlobalVar) -> IRWriteRes {
        let allocs = &self.env.module.allocs;
        let gid = id.raw_into();

        self.fmt_global_header_prefix(gid, "gvar")?;

        let name = self.env.nameof_global(gid);
        let prefix = gvar.get_linkage_prefix(allocs);
        let begin_pos = self.writer.curr_pos();
        write!(self, "@{name} = {prefix} ")?;

        if let Some(tls) = gvar.tls_model.get() {
            write!(self, "thread_local({}) ", tls.get_ir_text())?;
        }
        if !gvar.is_extern(allocs) {
            self.fmt_use(gvar.initval[0])?;
        }
        write!(self, ", align {}", gvar.get_ptr_pointee_align())?;

        let end_pos = self.writer.curr_pos();
        if let Some(srcmap) = &mut self.writer.srcmap {
            srcmap.insert_range(gid, (begin_pos, end_pos));
        }
        Ok(())
    }
    fn fmt_global_header_prefix(&mut self, gid: GlobalID, kind: &str) -> IRWriteRes {
        let allocs = &self.env.module.allocs;
        let options = self.env.option;
        if options.show_indexed {
            let (index, gene) = (gid.0.get_order(), gid.0.get_generation());
            write!(self, "; {kind} index={index:x}, gen={gene:x}")?;
            self.writer.wrap_indent()?;
        }
        if options.show_users {
            self.fmtln_users(gid.deref_ir(allocs))?;
        }
        Ok(())
    }

    fn fmt_func_header(&mut self, func_id: FuncID, func: &FuncObj) -> IRWriteRes<IRSourceRange> {
        let allocs = &self.env.module.allocs;
        let gid = func_id.raw_into();
        let name = self.env.nameof_global(gid);

        self.fmt_global_header_prefix(gid, "func")?;

        let begin_pos = self.writer.curr_pos();
        self.write_str(func.get_linkage_prefix(allocs))?;
        self.fmt_attrs(&func.attrs())?;
        self.write_str(" ")?;
        let retty = self.type_name(func.ret_type);
        write!(self, "{retty} @{name}(")?;

        for (idx, arg) in func.args.iter().enumerate() {
            if idx > 0 {
                self.write_str(", ")?;
            }
            self.fmt_type(arg.ty)?;
            self.fmt_attrs(&arg.attrs())?;
            if func.is_extern(allocs) {
                continue;
            }

            let arg_id = FuncArgID(func_id, idx as u32);
            let begin_pos = self.writer.curr_pos();
            match self.env.names.get_local_name(arg_id) {
                Some(name) => write!(self, " %{name}")?,
                None => write!(self, " %{idx}")?,
            };
            let end_pos = self.writer.curr_pos();
            if let Some(srcmap) = &mut self.writer.srcmap {
                srcmap.funcarg_insert_range(arg_id, (begin_pos, end_pos));
            }
        }
        if func.is_vararg {
            let prompt = if func.args.is_empty() { "..." } else { ", ..." };
            self.write_str(prompt)?;
        }
        self.write_str(")")?;
        Ok((begin_pos, self.writer.curr_pos()))
    }
    fn fmt_func(&mut self, func_id: FuncID, func: &FuncObj) -> IRWriteRes {
        match &self.env.names {
            NameMapRepr::Number(numbers) if numbers.func == func_id => {
                self.do_fmt_func(func_id, func)
            }
            /* external functions only format headers */
            _ if func.body.is_none() => self.do_fmt_func(func_id, func),
            names => {
                let names = names.name_map();
                let allocs = &self.env.module.allocs;
                let option = NumberOption::ignore_all();
                let numbers = Rc::new(FuncNumberMap::new(allocs, func_id, names, option));
                let mut func_writer = self.scoped_with_names(NameMapRepr::Number(numbers));
                func_writer.do_fmt_func(func_id, func)
            }
        }
    }
    fn do_fmt_func(&mut self, func_id: FuncID, func: &FuncObj) -> IRWriteRes {
        let allocs = &self.env.module.allocs;

        let (begin_pos, end_pos) = self.fmt_func_header(func_id, func)?;
        if func.is_extern(allocs) {
            self.write_str("; extern")?;
            self.writer.wrap_indent()?;
            self.insert_range(func_id.raw_into(), (begin_pos, end_pos));
            return Ok(());
        }

        self.write_str("{")?;
        let entry = func.entry_unwrap();
        self.fmt_block(entry, entry.deref_ir(allocs))?;

        for (bb_id, bb_obj) in func.block_iter(allocs) {
            if bb_id == entry {
                continue;
            }
            self.fmt_block(bb_id, bb_obj)?;
        }
        self.writer.wrap_indent()?;
        self.write_str("}")?;
        self.insert_range(func_id.raw_into(), (begin_pos, self.writer.curr_pos()));
        Ok(())
    }
    fn fmt_block(&mut self, block_id: BlockID, block: &BlockObj) -> IRWriteRes {
        let (allocs, option) = {
            let env = &self.env;
            (&env.module.allocs, env.option)
        };
        self.writer.wrap_indent()?;
        if option.show_indexed {
            let (index, gene) = (block_id.0.get_order(), block_id.0.get_generation());
            write!(
                self,
                "; block id=%block:{index:x}, gen={gene:x}"
            )?;
            self.writer.wrap_indent()?;
        }
        if option.show_preds {
            self.fmtln_preds(block)?;
        }
        if option.show_users {
            self.fmtln_users(block)?;
        }
        let begin_pos = self.writer.curr_pos();
        let name = match self.env.names.get_local_name(block_id) {
            Some(name) => name,
            None => format_smolstr!("block:{:x}", block_id.get_entity_index(allocs)),
        };
        write!(self, "{name}:")?;
        self.writer.indent += 1;

        for (inst_id, inst) in block.get_insts().iter(&allocs.insts) {
            self.writer.wrap_indent()?;
            if option.show_indexed {
                let (index, gene) = (inst_id.0.get_order(), inst_id.0.get_generation());
                write!(
                    self,
                    "; inst id=%inst:{index:x}, gen={gene:x}"
                )?;
                self.writer.wrap_indent()?;
            }
            if option.show_users {
                self.fmtln_users(inst)?;
            }
            let begin_pos = self.writer.curr_pos();
            if let Some(name) = self.env.names.get_local_name(inst_id)
                && inst.serialize_has_number()
            {
                write!(self, "%{name} = ")?;
            }
            inst.serialize_ir(self)?;
            let end_pos = self.writer.curr_pos();
            self.insert_range(inst_id, (begin_pos, end_pos));
        }
        self.writer.indent -= 1;
        let end_pos = self.writer.curr_pos();
        self.insert_range(block_id, (begin_pos, end_pos));
        Ok(())
    }
}

trait IRSerializeInst: ISubInst {
    fn serialize_has_number(&self) -> bool;

    fn serialize_ir<W: Write>(&self, ctx: &mut FmtCtx<'_, '_, '_, W>) -> IRWriteRes;
}

impl IRSerializeInst for InstObj {
    fn serialize_has_number(&self) -> bool {
        use crate::ir::inst::InstObj::*;
        !matches!(
            self,
            GuideNode(_) | PhiInstEnd(_) | Ret(_) | Jump(_) | Br(_) | Switch(_) | Unreachable(_)
        ) && self.get_valtype() != ValTypeID::Void
    }

    fn serialize_ir<W: Write>(&self, ctx: &mut FmtCtx<'_, '_, '_, W>) -> IRWriteRes {
        match self {
            InstObj::GuideNode(_) => Ok(()),
            InstObj::PhiInstEnd(_) => ctx.write_str("; ==== Phi Section End ===="),
            InstObj::Unreachable(inst) => inst.serialize_ir(ctx),
            InstObj::Ret(inst) => inst.serialize_ir(ctx),
            InstObj::Jump(inst) => inst.serialize_ir(ctx),
            InstObj::Br(inst) => inst.serialize_ir(ctx),
            InstObj::Switch(inst) => inst.serialize_ir(ctx),
            InstObj::Alloca(inst) => inst.serialize_ir(ctx),
            InstObj::GEP(inst) => inst.serialize_ir(ctx),
            InstObj::Load(inst) => inst.serialize_ir(ctx),
            InstObj::Store(inst) => inst.serialize_ir(ctx),
            InstObj::AmoRmw(inst) => inst.serialize_ir(ctx),
            InstObj::BinOP(inst) => inst.serialize_ir(ctx),
            InstObj::Call(inst) => inst.serialize_ir(ctx),
            InstObj::Cast(inst) => inst.serialize_ir(ctx),
            InstObj::Cmp(inst) => inst.serialize_ir(ctx),
            InstObj::IndexExtract(inst) => inst.serialize_ir(ctx),
            InstObj::FieldExtract(inst) => inst.serialize_ir(ctx),
            InstObj::IndexInsert(inst) => inst.serialize_ir(ctx),
            InstObj::FieldInsert(inst) => inst.serialize_ir(ctx),
            InstObj::Phi(inst) => inst.serialize_ir(ctx),
            InstObj::Select(inst) => inst.serialize_ir(ctx),
        }
    }
}

impl IRSerializeInst for UnreachableInst {
    fn serialize_has_number(&self) -> bool {
        false
    }

    fn serialize_ir<W: Write>(&self, ctx: &mut FmtCtx<'_, '_, '_, W>) -> IRWriteRes {
        ctx.write_str("unreachable")
    }
}

impl IRSerializeInst for RetInst {
    fn serialize_has_number(&self) -> bool {
        false
    }

    fn serialize_ir<W: Write>(&self, ctx: &mut FmtCtx<'_, '_, '_, W>) -> IRWriteRes {
        if self.get_valtype() == ValTypeID::Void {
            ctx.write_str("ret void")
        } else {
            ctx.write_str("ret ")?;
            ctx.fmt_type(self.get_valtype())?;
            ctx.write_str(" ")?;
            ctx.fmt_use(self.retval_use())
        }
    }
}

impl IRSerializeInst for JumpInst {
    fn serialize_has_number(&self) -> bool {
        false
    }

    fn serialize_ir<W: Write>(&self, ctx: &mut FmtCtx<'_, '_, '_, W>) -> IRWriteRes {
        ctx.write_str("br label ")?;
        ctx.fmt_jt(self.target_jt())
    }
}

impl IRSerializeInst for BrInst {
    fn serialize_has_number(&self) -> bool {
        false
    }

    fn serialize_ir<W: Write>(&self, ctx: &mut FmtCtx<'_, '_, '_, W>) -> IRWriteRes {
        ctx.write_str("br i1 ")?;
        ctx.fmt_use(self.cond_use())?;
        ctx.write_str(", label ")?;
        ctx.fmt_jt(self.then_jt())?;
        ctx.write_str(", label ")?;
        ctx.fmt_jt(self.else_jt())
    }
}

impl IRSerializeInst for SwitchInst {
    fn serialize_has_number(&self) -> bool {
        false
    }

    fn serialize_ir<W: Write>(&self, ctx: &mut FmtCtx<'_, '_, '_, W>) -> IRWriteRes {
        let allocs = &ctx.env.module.allocs;
        let cond_ty = self.discrim_ty.to_smolstr();
        write!(ctx, "switch {cond_ty} ")?;
        ctx.fmt_use(self.discrim_use())?;
        ctx.write_str(", label ")?;
        ctx.fmt_jt(self.default_jt())?;
        if self.case_jts().is_empty() {
            return ctx.write_str(" []");
        }
        ctx.write_str(" [")?;
        ctx.writer.indent += 1;
        for (case_jt, case_val, _) in self.cases_iter(allocs) {
            ctx.writer.wrap_indent()?;
            write!(ctx, "{cond_ty} {case_val}, label ")?;
            ctx.fmt_jt(case_jt)?;
        }
        ctx.writer.indent -= 1;
        ctx.writer.wrap_indent()?;
        ctx.write_str(" ]")
    }
}

impl IRSerializeInst for AllocaInst {
    fn serialize_has_number(&self) -> bool {
        self.get_valtype() != ValTypeID::Void
    }

    fn serialize_ir<W: Write>(&self, ctx: &mut FmtCtx<'_, '_, '_, W>) -> IRWriteRes {
        ctx.write_str("alloca ")?;
        ctx.fmt_type(self.pointee_ty)?;
        ctx.write_str(" ")?;
        write!(ctx, ", align {}", self.get_ptr_pointee_align())?;
        Ok(())
    }
}

impl IRSerializeInst for GEPInst {
    fn serialize_has_number(&self) -> bool {
        self.get_valtype() != ValTypeID::Void
    }

    fn serialize_ir<W: Write>(&self, ctx: &mut FmtCtx<'_, '_, '_, W>) -> IRWriteRes {
        ctx.write_str("getelementptr ")?;
        if self.get_inbounds() {
            ctx.write_str("inbounds ")?;
        }
        ctx.fmt_type(self.initial_ty)?;
        ctx.write_str(", ptr ")?;
        ctx.fmt_use(self.base_use())?;

        let allocs = &ctx.env.module.allocs;
        for &index_use in self.index_uses() {
            let index = index_use.get_operand(allocs);
            let index_ty = index.get_valtype(allocs);
            ctx.write_str(", ")?;
            ctx.fmt_type(index_ty)?;
            ctx.write_str(" ")?;
            ctx.fmt_use(index_use)?;
        }
        Ok(())
    }
}

impl IRSerializeInst for LoadInst {
    fn serialize_has_number(&self) -> bool {
        self.get_valtype() != ValTypeID::Void
    }

    fn serialize_ir<W: Write>(&self, ctx: &mut FmtCtx<'_, '_, '_, W>) -> IRWriteRes {
        let pointee_ty = ctx.type_name(self.get_valtype());
        write!(ctx, "load {pointee_ty}, ptr ")?;
        ctx.fmt_use(self.source_use())?;
        write!(ctx, ", align {}", self.get_operand_pointee_align())
    }
}

impl IRSerializeInst for StoreInst {
    fn serialize_has_number(&self) -> bool {
        false
    }

    fn serialize_ir<W: Write>(&self, ctx: &mut FmtCtx<'_, '_, '_, W>) -> IRWriteRes {
        let source_ty = ctx.type_name(self.source_ty);
        write!(ctx, "store {source_ty} ")?;
        ctx.fmt_use(self.source_use())?;
        ctx.write_str(", ptr ")?;
        ctx.fmt_use(self.target_use())?;
        write!(ctx, ", align {}", self.get_operand_pointee_align())
    }
}

impl IRSerializeInst for AmoRmwInst {
    fn serialize_has_number(&self) -> bool {
        self.get_valtype() != ValTypeID::Void
    }

    fn serialize_ir<W: Write>(&self, ctx: &mut FmtCtx<'_, '_, '_, W>) -> IRWriteRes {
        let subop_name = self.subop_name();
        if self.is_volatile {
            write!(ctx, "atomicrmw volatile {subop_name} ptr ")?;
        } else {
            write!(ctx, "atomicrmw {subop_name} ptr ")?;
        }
        ctx.fmt_use(self.pointer_use())?;

        let value_ty = ctx.type_name(self.value_ty);
        write!(ctx, ", {value_ty} ")?;
        ctx.fmt_use(self.value_use())?;
        if self.scope != SyncScope::System {
            write!(ctx, " syncscope(\"{}\")", self.scope.as_str())?;
        }
        write!(ctx, " {}", self.ordering.as_str())?;
        if self.align_log2 > 0 {
            write!(ctx, ", align {}", 1 << self.align_log2)?;
        }
        Ok(())
    }
}

impl IRSerializeInst for BinOPInst {
    fn serialize_has_number(&self) -> bool {
        self.get_valtype() != ValTypeID::Void
    }

    fn serialize_ir<W: Write>(&self, ctx: &mut FmtCtx<'_, '_, '_, W>) -> IRWriteRes {
        let opcode = self.get_opcode().get_name();
        let flags = self.get_flags();
        let ty = ctx.type_name(self.get_valtype());

        if flags.is_empty() {
            write!(ctx, "{opcode} {ty} ")?;
        } else {
            write!(ctx, "{opcode} {flags} {ty} ")?;
        }
        ctx.fmt_use(self.lhs_use())?;
        ctx.write_str(", ")?;
        ctx.fmt_use(self.rhs_use())
    }
}

impl IRSerializeInst for CallInst {
    fn serialize_has_number(&self) -> bool {
        self.get_valtype() != ValTypeID::Void
    }

    fn serialize_ir<W: Write>(&self, ctx: &mut FmtCtx<'_, '_, '_, W>) -> IRWriteRes {
        let ret_ty = ctx.type_name(self.get_valtype());
        if self.is_vararg {
            write!(ctx, "call {ret_ty} (...) ")?;
        } else {
            write!(ctx, "call {ret_ty} ")?;
        }
        ctx.fmt_use(self.callee_use())?;
        ctx.write_str("(")?;

        let allocs = &ctx.env.module.allocs;
        let tctx = &ctx.env.module.tctx;
        for (i, &arg_use) in self.arg_uses().iter().enumerate() {
            if i > 0 {
                ctx.write_str(", ")?;
            }
            let arg = arg_use.get_operand(allocs);
            let arg_ty = self
                .callee_ty
                .get_args(tctx)
                .get(i)
                .copied()
                .unwrap_or(arg.get_valtype(allocs));
            let arg_ty = ctx.type_name(arg_ty);
            write!(ctx, "{arg_ty} ")?;
            ctx.fmt_use(arg_use)?;
        }
        ctx.write_str(")")
    }
}

impl IRSerializeInst for CastInst {
    fn serialize_has_number(&self) -> bool {
        self.get_valtype() != ValTypeID::Void
    }

    fn serialize_ir<W: Write>(&self, ctx: &mut FmtCtx<'_, '_, '_, W>) -> IRWriteRes {
        let opcode = self.get_opcode().get_name();
        let from_ty = ctx.type_name(self.from_ty);
        write!(ctx, "{opcode} {from_ty} ")?;
        ctx.fmt_use(self.from_use())?;
        let to_ty = ctx.type_name(self.get_valtype());
        write!(ctx, " to {to_ty}")
    }
}

impl IRSerializeInst for CmpInst {
    fn serialize_has_number(&self) -> bool {
        self.get_valtype() != ValTypeID::Void
    }

    fn serialize_ir<W: Write>(&self, ctx: &mut FmtCtx<'_, '_, '_, W>) -> IRWriteRes {
        let opcode = self.get_opcode().get_name();
        let cond = self.cond;
        let operand_ty = ctx.type_name(self.operand_ty);
        write!(ctx, "{opcode} {cond} {operand_ty} ")?;
        ctx.fmt_use(self.lhs_use())?;
        ctx.write_str(", ")?;
        ctx.fmt_use(self.rhs_use())
    }
}

impl IRSerializeInst for IndexExtractInst {
    fn serialize_has_number(&self) -> bool {
        self.get_valtype() != ValTypeID::Void
    }

    fn serialize_ir<W: Write>(&self, ctx: &mut FmtCtx<'_, '_, '_, W>) -> IRWriteRes {
        let allocs = &ctx.env.module.allocs;
        let aggrty = ctx.type_name(self.aggr_type.into_ir());
        let index_ty = ctx.type_name(self.get_index(allocs).get_valtype(allocs));

        write!(ctx, "extractelement {aggrty} ")?;
        ctx.fmt_use(self.aggr_use())?;

        write!(ctx, ", {index_ty} ")?;
        ctx.fmt_use(self.index_use())
    }
}

impl IRSerializeInst for FieldExtractInst {
    fn serialize_has_number(&self) -> bool {
        self.get_valtype() != ValTypeID::Void
    }

    fn serialize_ir<W: Write>(&self, ctx: &mut FmtCtx<'_, '_, '_, W>) -> IRWriteRes {
        let aggr_ty = ctx.type_name(self.aggr_type.into_ir());
        write!(ctx, "extractvalue {aggr_ty} ")?;
        ctx.fmt_use(self.aggr_use())?;
        for &idx in self.get_field_indices() {
            write!(ctx, ", {idx}")?;
        }
        Ok(())
    }
}

impl IRSerializeInst for IndexInsertInst {
    fn serialize_has_number(&self) -> bool {
        self.get_valtype() != ValTypeID::Void
    }

    fn serialize_ir<W: Write>(&self, ctx: &mut FmtCtx<'_, '_, '_, W>) -> IRWriteRes {
        let allocs = &ctx.env.module.allocs;
        let aggr_ty = ctx.type_name(self.get_valtype());
        let elem_ty = ctx.type_name(self.get_elem_type());
        let index_ty = ctx.type_name(self.get_index(allocs).get_valtype(allocs));

        write!(ctx, "insertelement {aggr_ty} ")?;
        ctx.fmt_use(self.aggr_use())?;

        write!(ctx, ", {elem_ty} ")?;
        ctx.fmt_use(self.elem_use())?;

        write!(ctx, ", {index_ty} ")?;
        ctx.fmt_use(self.index_use())
    }
}

impl IRSerializeInst for FieldInsertInst {
    fn serialize_has_number(&self) -> bool {
        self.get_valtype() != ValTypeID::Void
    }

    fn serialize_ir<W: Write>(&self, ctx: &mut FmtCtx<'_, '_, '_, W>) -> IRWriteRes {
        let aggr_ty = ctx.type_name(self.get_valtype());
        let elem_ty = ctx.type_name(self.get_elem_type());

        write!(ctx, "insertvalue {aggr_ty} ")?;
        ctx.fmt_use(self.aggr_use())?;

        write!(ctx, ", {elem_ty} ")?;
        ctx.fmt_use(self.elem_use())?;

        for &idx in self.get_field_indices() {
            write!(ctx, ", {idx}")?;
        }
        Ok(())
    }
}

impl IRSerializeInst for PhiInst {
    fn serialize_has_number(&self) -> bool {
        self.get_valtype() != ValTypeID::Void
    }

    fn serialize_ir<W: Write>(&self, ctx: &mut FmtCtx<'_, '_, '_, W>) -> IRWriteRes {
        let ty = ctx.type_name(self.get_valtype());

        write!(ctx, "phi {ty} ")?;
        let mut first = true;
        for [uval, ublk] in self.incoming_uses().iter() {
            ctx.write_str(if first { " [" } else { ", [" })?;
            first = false;

            ctx.fmt_use(*uval)?;
            ctx.write_str(", label ")?;
            ctx.fmt_use(*ublk)?;
            ctx.write_str("]")?;
        }
        Ok(())
    }
}

impl IRSerializeInst for SelectInst {
    fn serialize_has_number(&self) -> bool {
        self.get_valtype() != ValTypeID::Void
    }

    fn serialize_ir<W: Write>(&self, ctx: &mut FmtCtx<'_, '_, '_, W>) -> IRWriteRes {
        let ty = ctx.type_name(self.get_valtype());
        write!(ctx, "select {ty}, i1 ")?;
        ctx.fmt_use(self.cond_use())?;
        ctx.write_str(", ")?;
        ctx.fmt_use(self.then_use())?;
        ctx.write_str(", ")?;
        ctx.fmt_use(self.else_use())
    }
}

pub trait SerializeIR<'ir, 'names, W: Write> {
    #[doc(hidden)]
    fn _protected_tear(&mut self) -> FmtCtx<'ir, 'names, '_, W>;

    fn set_options(&mut self, options: IRWriteOption) -> &mut Self;
    fn enable_srcmap(&mut self) -> &mut Self;
    fn dump_srcmap(&mut self) -> Option<SourceRangeMap>;

    fn fmt_global(&mut self, global_id: GlobalID) -> IRWriteRes {
        self._protected_tear().fmt_global(global_id)
    }
    fn fmt_module(&mut self) -> IRWriteRes {
        self._protected_tear().fmt_module()
    }
    fn fmt_func(&mut self, func_id: FuncID) -> IRWriteRes {
        let mut ctx = self._protected_tear();
        let allocs = &ctx.env.module.allocs;
        let func = func_id.deref_ir(allocs);
        ctx.fmt_func(func_id, func)
    }
    fn fmt_block(&mut self, block_id: BlockID) -> IRWriteRes {
        let mut ctx = self._protected_tear();
        let allocs = &ctx.env.module.allocs;
        let block = block_id.deref_ir(allocs);
        ctx.fmt_block(block_id, block)
    }
    fn fmt_inst(&mut self, inst_id: InstID) -> IRWriteRes {
        let mut ctx = self._protected_tear();
        let allocs = &ctx.env.module.allocs;
        let inst = inst_id.deref_ir(allocs);
        inst.serialize_ir(&mut ctx)
    }
    fn fmt_operand(&mut self, op: ValueSSA) -> IRWriteRes {
        self._protected_tear().fmt_value_mapped(op)
    }
    fn fmt_use_info(&mut self, useid: UseID) -> IRWriteRes {
        let mut ctx = self._protected_tear();
        let allocs = &ctx.env.module.allocs;
        let kind = useid.get_kind(allocs);
        write!(ctx, "Use(kind: {kind}, value:")?;
        ctx.fmt_value_mapped(useid.get_operand(allocs))?;
        ctx.write_str(")")
    }
    fn fmt_jt_info(&mut self, jtid: JumpTargetID) -> IRWriteRes {
        let mut ctx = self._protected_tear();
        let allocs = &ctx.env.module.allocs;
        let kind = jtid.get_kind(allocs);
        write!(ctx, "JumpTarget(kind: {kind}, block:")?;
        let value = match jtid.get_block(allocs) {
            Some(bb_id) => ValueSSA::Block(bb_id),
            None => ValueSSA::None,
        };
        ctx.fmt_value_mapped(value)?;
        ctx.write_str(")")
    }
}

pub struct IRSerializer<'ir, 'names, W: Write> {
    writer: Writer<W>,
    module: &'ir Module,
    names: &'names IRNameMap,
    options: IRWriteOption,
    cache: Cache,
}
impl<'ir, 'names, W: Write> SerializeIR<'ir, 'names, W> for IRSerializer<'ir, 'names, W> {
    fn _protected_tear(&mut self) -> FmtCtx<'ir, 'names, '_, W> {
        FmtCtx {
            env: Env {
                module: self.module,
                names: NameMapRepr::Name(self.names),
                option: self.options,
            },
            writer: &mut self.writer,
            cache: &mut self.cache,
        }
    }

    fn set_options(&mut self, options: IRWriteOption) -> &mut Self {
        self.options = options;
        self
    }
    fn enable_srcmap(&mut self) -> &mut Self {
        self.writer.srcmap = Some(SourceRangeMap::default());
        self
    }

    fn dump_srcmap(&mut self) -> Option<SourceRangeMap> {
        self.writer.srcmap.take()
    }
}
impl<'ir, 'names> IRSerializer<'ir, 'names, Vec<u8>> {
    pub fn new_buffered(module: &'ir Module, names: &'names IRNameMap) -> Self {
        Self::new(Vec::new(), module, names)
    }

    pub fn extract_string(self) -> String {
        String::from_utf8(self.writer.writer.writer)
            .expect("IRSerializer buffer should be valid UTF-8")
    }
}
impl<'ir, 'names, W: Write> IRSerializer<'ir, 'names, W> {
    pub fn new(writer: W, module: &'ir Module, names: &'names IRNameMap) -> Self {
        Self {
            writer: Writer::new(writer),
            module,
            names,
            options: IRWriteOption::default(),
            cache: Cache::default(),
        }
    }

    pub fn with_func<'a>(
        &'a mut self,
        func: FuncID,
        f: impl FnOnce(&mut FuncSerializer<'ir, 'names, &'a mut dyn Write>) -> IRWriteRes,
    ) -> IRWriteRes {
        let allocs = &self.module.allocs;
        if func.is_extern(allocs) {
            return Err(IRWriteErr::FuncIsExtern(func.clone_name(allocs)));
        }
        let numbers = FuncNumberMap::new(allocs, func, self.names, NumberOption::ignore_all());
        let mut func_serializer = FuncSerializer {
            writer: Writer::new(&mut self.writer.writer.writer as &mut dyn Write),
            module: self.module,
            numbers: Rc::new(numbers),
            options: self.options,
            cache: Cache::default(),
        };
        if self.writer.srcmap.is_some() {
            func_serializer.enable_srcmap();
        }
        f(&mut func_serializer)?;
        let FuncSerializer { mut writer, .. } = func_serializer;
        if let Some(srcmap) = &mut self.writer.srcmap {
            let func_srcmap = writer.srcmap.take().unwrap();
            srcmap.update_merge(&func_srcmap);
        }
        Ok(())
    }
}

pub struct FuncSerializer<'ir, 'names, W: Write> {
    writer: Writer<W>,
    module: &'ir Module,
    numbers: Rc<FuncNumberMap<'names>>,
    options: IRWriteOption,
    cache: Cache,
}

impl<'ir, 'names, W: Write> SerializeIR<'ir, 'names, W> for FuncSerializer<'ir, 'names, W> {
    fn _protected_tear(&mut self) -> FmtCtx<'ir, 'names, '_, W> {
        let Self { writer, module, numbers: names, options, cache } = self;
        let env = Env {
            module,
            names: NameMapRepr::Number(names.clone()),
            option: *options,
        };
        FmtCtx { env, cache, writer }
    }

    fn set_options(&mut self, options: IRWriteOption) -> &mut Self {
        self.options = options;
        self
    }
    fn enable_srcmap(&mut self) -> &mut Self {
        self.writer.srcmap = Some(SourceRangeMap::default());
        self
    }

    fn dump_srcmap(&mut self) -> Option<SourceRangeMap> {
        self.writer.srcmap.take()
    }
}
impl<'ir, 'names> FuncSerializer<'ir, 'names, Vec<u8>> {
    pub fn new_buffered(module: &'ir Module, func: FuncID, names: &'names IRNameMap) -> Self {
        Self::new(Vec::new(), module, func, names)
    }
    pub fn with_numbers_buffered(module: &'ir Module, numbers: Rc<FuncNumberMap<'names>>) -> Self {
        Self::with_numbers(Vec::new(), module, numbers)
    }

    pub fn extract_string(self) -> String {
        String::from_utf8(self.writer.writer.writer)
            .expect("FuncSerializer buffer should be valid UTF-8")
    }
    pub fn extract_symstr(self) -> SymbolStr {
        SymbolStr::new(self.extract_string())
    }
}
impl<'ir, 'names, W: Write> FuncSerializer<'ir, 'names, W> {
    pub fn new(writer: W, module: &'ir Module, func: FuncID, names: &'names IRNameMap) -> Self {
        let numbers = Rc::new(FuncNumberMap::new(
            &module.allocs,
            func,
            names,
            NumberOption::ignore_all(),
        ));
        Self::with_numbers(writer, module, numbers)
    }
    pub fn with_numbers(
        writer: W,
        module: &'ir Module,
        numbers: Rc<FuncNumberMap<'names>>,
    ) -> Self {
        Self {
            writer: Writer::new(writer),
            module,
            numbers,
            options: IRWriteOption::default(),
            cache: Cache::default(),
        }
    }
}

//！The refactored new IR writer.
//! Target:
//!
//! - let the writer generate source mapping while writing, including all mempool allocated objects.
//! - try to erase most of `RefCell` or other interior mutability.

use smol_str::SmolStrBuilder;

use crate::{
    SymbolStr,
    ir::{inst::*, *},
    typing::*,
};
use std::{
    collections::HashMap,
    io::{self, Write},
};

#[derive(Clone, Copy)]
enum NameMapRepr<'a> {
    Name(&'a IRNameMap),
    Number(&'a FuncNumberMap<'a>),
}
impl<'a> NameMapRepr<'a> {
    fn get_local_name(&self, allocs: &IRAllocs, val: impl IValueConvert) -> Option<SymbolStr> {
        match self {
            NameMapRepr::Name(m) => m.get_local_name(allocs, val),
            NameMapRepr::Number(m) => m.get_local_name(allocs, val),
        }
    }

    fn name_map(&self) -> &'a IRNameMap {
        match self {
            NameMapRepr::Name(m) => m,
            NameMapRepr::Number(m) => m.names,
        }
    }
}

#[derive(Clone, Copy)]
struct Env<'a> {
    module: &'a Module,
    names: NameMapRepr<'a>,
    option: IRWriteOption,
}

#[derive(Default)]
struct Cache {
    type_names: HashMap<ValTypeID, SymbolStr>,
    str_literals: HashMap<ExprID, Option<SymbolStr>>,
    llvm_map: LLVMAdaptMapping,
}

impl Cache {
    fn new() -> Self {
        Self::default()
    }

    fn map_value(&mut self, env: &Env, value: ValueSSA) -> ValueSSA {
        if env.option.llvm_compatible { self.llvm_map.map_value(env.module, value) } else { value }
    }
    fn type_name(&mut self, env: &Env, ty: ValTypeID) -> SymbolStr {
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
            ty => self
                .type_names
                .entry(ty)
                .or_insert_with(|| ty.get_display_name(&env.module.tctx))
                .clone(),
        }
    }

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
        self.write_str("(\n")?;
        self.indent()
    }

    fn curr_pos(&self) -> IRSourcePos {
        self.writer.curr_pos
    }
}

struct FmtValue<'a, 'b, W: Write> {
    env: &'a Env<'b>,
    cache: &'a mut Cache,
    writer: &'a mut Writer<W>,
}

impl<'a, 'b, W: Write> FmtValue<'a, 'b, W> {
    fn new(env: &'a Env<'b>, cache: &'a mut Cache, writer: &'a mut Writer<W>) -> Self {
        Self { env, cache, writer }
    }

    fn write_str(&mut self, s: &str) -> IRWriteRes {
        self.writer.write_str(s)
    }
    fn write_fmt(&mut self, args: std::fmt::Arguments<'_>) -> IRWriteRes {
        self.writer.write_fmt(args)
    }
    fn map_value(&mut self, v: ValueSSA) -> ValueSSA {
        self.cache.map_value(self.env, v)
    }
    fn fmt_type(&mut self, ty: ValTypeID) -> IRWriteRes {
        let name = self.cache.type_name(self.env, ty);
        self.write_str(&name)
    }

    fn fmt_use(&mut self, use_id: UseID) -> IRWriteRes {
        let allocs = &self.env.module.allocs;
        let val = use_id.get_operand(allocs);
        let begin_pos = self.writer.curr_pos();
        self.fmt_value(val)?;
        let end_pos = self.writer.curr_pos();
        if let Some(srcmap) = &mut self.writer.srcmap {
            srcmap.primary_insert_range(allocs, use_id, (begin_pos, end_pos));
        }
        Ok(())
    }
    fn fmt_jt(&mut self, jt_id: JumpTargetID) -> IRWriteRes {
        let allocs = &self.env.module.allocs;
        let block_id = match jt_id.get_block(allocs) {
            Some(bb) => ValueSSA::Block(bb),
            None => ValueSSA::None,
        };
        let begin_pos = self.writer.curr_pos();
        self.write_str("label ")?;
        self.fmt_value_mapped(block_id)?;
        let end_pos = self.writer.curr_pos();
        if let Some(srcmap) = &mut self.writer.srcmap {
            srcmap.primary_insert_range(allocs, jt_id, (begin_pos, end_pos));
        }
        Ok(())
    }

    fn fmt_value(&mut self, val: ValueSSA) -> IRWriteRes {
        let mapped = self.map_value(val);
        self.fmt_value_mapped(mapped)
    }

    fn fmt_value_mapped(&mut self, val: ValueSSA) -> IRWriteRes {
        let allocs = &self.env.module.allocs;
        if let Some(name) = self.env.names.get_local_name(allocs, val) {
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
                let name = global.clone_name(allocs);
                if self.env.module.symbol_is_exported(global) {
                    write!(self, "@{name}")
                } else {
                    let idx = global.get_entity_index(allocs);
                    write!(self, "@{name}.unpinned.{idx:x}")
                }
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
        if let Some(s) = self.cache.expr_as_litstr(self.env, expr) {
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

    fn fmt_global(&mut self, global_id: GlobalID) -> IRWriteRes {
        todo!("global serialization")
    }
}

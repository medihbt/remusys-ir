//！The refactored new IR writer.
//! Target:
//!
//! - let the writer generate source mapping while writing, including all mempool allocated objects.
//! - try to erase most of `RefCell` or other interior mutability.

use smol_str::SmolStrBuilder;

use crate::{
    SymbolStr,
    ir::{inst::*, module::allocs::IPoolAllocated, *},
    typing::*,
};
use std::{
    collections::HashMap,
    io::{self, Write},
};

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

struct Env<'a> {
    module: &'a Module,
    names: Option<&'a IRNameMap>,
    option: IRWriteOption,
}

#[derive(Default)]
struct Cache {
    type_names: HashMap<ValTypeID, SymbolStr>,
    str_literals: HashMap<ExprID, Option<SymbolStr>>,
    llvm_map: LLVMAdaptMapping,
    srcmap: Option<SourceRangeMap>,
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
        if let Some(s) = self.str_literals.get(&expr) {
            return s.clone();
        }
        let allocs = &env.module.allocs;
        let s = match expr.deref_ir(allocs) {
            ExprObj::Array(a) => Self::array_as_litstr(env, a),
            ExprObj::DataArray(da) => Self::array_as_litstr(env, da),
            ExprObj::SplatArray(sa) => Self::array_as_litstr(env, sa),
            ExprObj::KVArray(kv) => Self::array_as_litstr(env, kv),
            ExprObj::Struct(_) | ExprObj::FixVec(_) => None,
        };
        self.str_literals.insert(expr, s.clone());
        s
    }
    fn array_as_litstr(env: &Env, arr: &impl IArrayExpr) -> Option<SymbolStr> {
        let allocs = &env.module.allocs;
        let ValTypeID::Int(8) = arr.get_elem_type() else {
            return None;
        };
        let bytes = arr.value_iter(allocs).map(Self::value_as_u8);
        Self::bytes_as_litstr(bytes)
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
}

struct Writer<W: Write> {
    writer: SourceMapWriter<W>,
    indent: usize,
}
impl<W: Write> Writer<W> {
    fn new(writer: W) -> Self {
        Self { writer: SourceMapWriter::new(writer), indent: 0 }
    }

    fn write_str(&mut self, s: &str) -> IRWriteRes {
        self.writer.write_all(s.as_bytes()).map_err(IRWriteErr::IO)
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

use crate::{
    ir::{IRAllocs, IRValueNumberMap, ISubValueSSA, ValueSSA},
    typing::{context::TypeContext, id::ValTypeID},
};
use std::{
    cell::{Cell, RefCell},
    io::Write,
};

pub struct IRWriter<'a> {
    pub output: RefCell<&'a mut dyn Write>,
    pub type_ctx: &'a TypeContext,
    pub allocs: &'a IRAllocs,
    pub indent_level: Cell<usize>,
    pub numbering: IRValueNumberMap,
}

impl<'a> Write for IRWriter<'a> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.output.get_mut().write(buf)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.output.get_mut().flush()
    }
    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.output.get_mut().write_all(buf)
    }
    fn write_fmt(&mut self, fmt: std::fmt::Arguments<'_>) -> std::io::Result<()> {
        self.output.get_mut().write_fmt(fmt)
    }
}

impl<'a> IRWriter<'a> {
    pub fn write(&self, buf: &[u8]) -> std::io::Result<usize> {
        self.output.borrow_mut().write(buf)
    }
    pub fn flush(&self) -> std::io::Result<()> {
        self.output.borrow_mut().flush()
    }
    pub fn write_all(&self, buf: &[u8]) -> std::io::Result<()> {
        self.output.borrow_mut().write_all(buf)
    }
    pub fn write_fmt(&self, fmt: std::fmt::Arguments<'_>) -> std::io::Result<()> {
        self.output.borrow_mut().write_fmt(fmt)
    }

    pub fn inc_indent(&self) {
        self.indent_level.set(self.indent_level.get() + 1);
    }
    pub fn dec_indent(&self) {
        let level = self.indent_level.get();
        if level > 0 {
            self.indent_level.set(level - 1);
        }
    }
    pub fn guard_inc_indent(&'a self) -> impl Drop + 'a {
        self.inc_indent();
        struct Guard<'a>(&'a IRWriter<'a>);
        impl<'a> Drop for Guard<'a> {
            fn drop(&mut self) {
                self.0.dec_indent();
            }
        }
        Guard(self)
    }

    pub fn wrap_indent(&self) {
        for _ in 0..self.indent_level.get() {
            self.output
                .borrow_mut()
                .write_all(b"    ")
                .expect("Failed to write indentation");
        }
    }

    pub fn write_str(&self, s: &str) -> std::io::Result<()> {
        self.output.borrow_mut().write_all(s.as_bytes())
    }

    pub fn write_operand(&self, operand: impl ISubValueSSA) -> std::io::Result<()> {
        match operand.into_ir() {
            ValueSSA::None => write!(self.output.borrow_mut(), "none"),
            ValueSSA::ConstData(data) => data.fmt_ir(self),
            ValueSSA::ConstExpr(expr) => expr.fmt_ir(self),
            ValueSSA::FuncArg(global_ref, _) => {
                let name = global_ref.get_name_from_alloc(&self.allocs.globals);
                write!(self.output.borrow_mut(), "@{}", name)
            }
            ValueSSA::Block(block_ref) => {
                let id = self.numbering.block_get_number(block_ref).unwrap();
                write!(self.output.borrow_mut(), "%{id}")
            }
            ValueSSA::Inst(inst_ref) => {
                let id = self.numbering.inst_get_number(inst_ref).unwrap();
                write!(self.output.borrow_mut(), "%{id}")
            }
            ValueSSA::Global(global_ref) => {
                let name = global_ref.get_name_from_alloc(&self.allocs.globals);
                write!(self.output.borrow_mut(), "@{}", name)
            }
        }
    }
    pub fn write_type(&self, ty: ValTypeID) -> std::io::Result<()> {
        write!(self, "{}", ty.get_display_name(&self.type_ctx))
    }
}

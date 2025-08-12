use crate::typing::{TypeAllocs, TypeContext};
use std::{
    cell::{Ref, RefCell},
    io::Write,
};

pub struct TypeFormatter<'a, T: Write> {
    pub output: RefCell<&'a mut T>,
    pub tctx: &'a TypeContext,
    pub allocs: Ref<'a, TypeAllocs>,
}

impl<'a, T: Write> Write for TypeFormatter<'a, T> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.output.get_mut().write(buf)
    }
    fn flush(&mut self) -> std::io::Result<()> {
        self.output.get_mut().flush()
    }
    fn write_fmt(&mut self, fmt: std::fmt::Arguments<'_>) -> std::io::Result<()> {
        self.output.get_mut().write_fmt(fmt)
    }
    fn write_all(&mut self, buf: &[u8]) -> std::io::Result<()> {
        self.output.get_mut().write_all(buf)
    }
    fn write_vectored(&mut self, bufs: &[std::io::IoSlice<'_>]) -> std::io::Result<usize> {
        self.output.get_mut().write_vectored(bufs)
    }
}

impl<'a, T: Write> TypeFormatter<'a, T> {
    pub fn new(output: &'a mut T, tctx: &'a TypeContext) -> Self {
        let allocs = tctx.allocs.borrow();
        Self { output: RefCell::new(output), tctx, allocs }
    }

    pub fn write_str(&self, s: &str) -> std::io::Result<()> {
        self.output.borrow_mut().write_all(s.as_bytes())
    }
    pub fn write(&self, buf: &[u8]) -> std::io::Result<usize> {
        self.output.borrow_mut().write(buf)
    }
    pub fn flush(&self) -> std::io::Result<()> {
        self.output.borrow_mut().flush()
    }
    pub fn write_fmt(&self, fmt: std::fmt::Arguments<'_>) -> std::io::Result<()> {
        self.output.borrow_mut().write_fmt(fmt)
    }
    pub fn write_all(&self, buf: &[u8]) -> std::io::Result<()> {
        self.output.borrow_mut().write_all(buf)
    }
    pub fn write_vectored(&self, bufs: &[std::io::IoSlice<'_>]) -> std::io::Result<usize> {
        self.output.borrow_mut().write_vectored(bufs)
    }
}

use crate::typing::{TypeAllocs, TypeContext};
use std::{
    cell::{Ref, RefCell},
    fmt::Write,
};

pub struct TypeFormatter<'a, T: Write> {
    pub output: RefCell<&'a mut T>,
    pub tctx: &'a TypeContext,
    pub allocs: Ref<'a, TypeAllocs>,
}

impl<'a, T: Write> Write for TypeFormatter<'a, T> {
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.output.get_mut().write_str(s)
    }
    fn write_char(&mut self, c: char) -> std::fmt::Result {
        self.output.get_mut().write_char(c)
    }
    fn write_fmt(&mut self, args: std::fmt::Arguments<'_>) -> std::fmt::Result {
        self.output.get_mut().write_fmt(args)
    }
}

impl<'a, T: Write> TypeFormatter<'a, T> {
    pub fn new(output: &'a mut T, tctx: &'a TypeContext) -> Self {
        let allocs = tctx.allocs.borrow();
        Self { output: RefCell::new(output), tctx, allocs }
    }

    pub fn write_str(&self, s: &str) -> std::fmt::Result {
        self.output.borrow_mut().write_str(s)
    }
    pub fn write_fmt(&self, fmt: std::fmt::Arguments<'_>) -> std::fmt::Result {
        self.output.borrow_mut().write_fmt(fmt)
    }
}

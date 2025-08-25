use crate::{
    base::{INullableValue, SlabRef},
    ir::{
        FuncRef, GlobalKind, GlobalRef, IRAllocsRef, IRValueNumberMap, ISubValueSSA, Module,
        PredList, UserList, ValueSSA,
    },
    typing::{IValType, TypeContext, ValTypeID},
};
use std::{
    cell::{Cell, Ref, RefCell},
    collections::HashMap,
    io::Write,
};

pub fn write_ir_module(module: &Module, output: &mut dyn Write) {
    let writer = IRWriter::from_module(output, module);
    writer.write_module()
}
pub fn write_ir_module_quiet(module: &Module, output: &mut dyn Write) {
    let writer = IRWriter::from_module_quiet(output, module);
    writer.write_module()
}

pub struct IRWriterStat {
    pub curr_func: Cell<FuncRef>,
}

impl IRWriterStat {
    pub fn new() -> Self {
        Self { curr_func: Cell::new(FuncRef(GlobalRef::new_null())) }
    }

    pub fn hold_curr_func<'a>(&'a self, func_ref: FuncRef) -> impl Drop + 'a {
        let prev_func = self.curr_func.replace(func_ref);
        struct Guard<'a> {
            stat: &'a IRWriterStat,
            prev_func: FuncRef,
        }
        impl<'a> Drop for Guard<'a> {
            fn drop(&mut self) {
                self.stat.curr_func.set(self.prev_func);
            }
        }
        Guard { stat: self, prev_func }
    }
}

pub struct IRWriter<'a> {
    pub output: RefCell<&'a mut dyn Write>,
    pub option: IRWriterOption,
    pub type_ctx: &'a TypeContext,
    pub allocs: IRAllocsRef<'a>,
    pub indent_level: Cell<usize>,
    pub numbering: RefCell<IRValueNumberMap>,
    pub globals: Vec<GlobalRef>,
    pub stat: IRWriterStat,
    type_str_cache: RefCell<HashMap<ValTypeID, String>>,
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
    pub fn new_loud(
        output: &'a mut dyn Write,
        type_ctx: &'a TypeContext,
        allocs: IRAllocsRef<'a>,
        globals: impl IntoIterator<Item = GlobalRef>,
    ) -> Self {
        Self {
            output: RefCell::new(output),
            option: IRWriterOption { show_slabref: true, show_users: true, show_preds: true },
            type_ctx,
            allocs,
            indent_level: Cell::new(0),
            numbering: RefCell::new(IRValueNumberMap::new_empty()),
            globals: globals.into_iter().collect(),
            stat: IRWriterStat::new(),
            type_str_cache: RefCell::new(HashMap::new()),
        }
    }

    pub fn from_module(output: &'a mut dyn Write, module: &'a Module) -> Self {
        Self::new_loud(
            output,
            &module.type_ctx,
            IRAllocsRef::Dyn(module.allocs.borrow()),
            module.globals.borrow().iter().map(|(_, &gref)| gref),
        )
    }

    pub fn from_module_quiet(output: &'a mut dyn Write, module: &'a Module) -> Self {
        let mut writer = Self::from_module(output, module);
        writer.option =
            IRWriterOption { show_slabref: false, show_users: false, show_preds: false };
        writer
    }

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
        let mut writer = self.output.borrow_mut();
        write!(&mut writer, "\n").expect("Failed to write newline");
        for _ in 0..self.indent_level.get() {
            write!(&mut writer, "    ").expect("Failed to write indentation");
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
            ValueSSA::AggrZero(_) => self.write_str("zeroinitializer"),
            ValueSSA::FuncArg(gref, id) => {
                assert_eq!(
                    self.stat.curr_func.get().0,
                    gref,
                    "FuncArg can only be used in its own function"
                );
                write!(self, "%{id}")
            }
            ValueSSA::Block(block_ref) => {
                let id = self.borrow_numbers().block_get_number(block_ref);
                if let Some(id) = id {
                    write!(self, "%{id}")
                } else {
                    write!(self, "%UnnamedBlock({})", block_ref.get_handle())
                }
            }
            ValueSSA::Inst(inst_ref) => {
                let id = self.borrow_numbers().inst_get_number(inst_ref);
                if let Some(id) = id {
                    write!(self, "%{id}")
                } else {
                    write!(self, "%UnnamedInst({})", inst_ref.get_handle())
                }
            }
            ValueSSA::Global(global_ref) => {
                let name = global_ref.get_name(&self.allocs);
                write!(self, "@{name}")
            }
        }
    }
    pub fn write_type(&self, ty: ValTypeID) -> std::io::Result<()> {
        if let Some(name) = self.type_str_cache.borrow().get(&ty) {
            self.write_str(name)
        } else {
            let name = ty.get_display_name(self.type_ctx);
            self.type_str_cache.borrow_mut().insert(ty, name.clone());
            self.write_str(&name)
        }
    }

    pub fn borrow_numbers(&self) -> Ref<IRValueNumberMap> {
        self.numbering.borrow()
    }

    pub fn write_module(&self) {
        // %{name} = type {struct}
        self.type_ctx.read_struct_aliases(|name, aliasee| {
            write!(self, "%{name} = type ").unwrap();
            self.write_type(ValTypeID::Struct(aliasee))
                .expect("Failed to write type aliases");
            self.wrap_indent();
        });

        if self.globals.is_empty() {
            return;
        }
        let globals = {
            let mut globals = Vec::with_capacity(self.globals.capacity());
            for &g in self.globals.iter() {
                globals.push((g, GlobalKind::from_global(g, &self.allocs)));
            }
            globals.sort_by_key(|(_, k)| *k);
            globals
        };

        let (_, mut curr_kind) = globals[0];
        for &(g, kind) in &globals {
            if kind != curr_kind {
                self.wrap_indent();
                curr_kind = kind;
            }
            g.fmt_ir(self).expect("Failed to write IR");
            self.wrap_indent();
        }
    }

    pub fn write_users(&self, users: &UserList) {
        if !self.option.show_users || users.is_empty() {
            return;
        }
        self.write_str("; Users: [ ").unwrap();
        for (i, user) in users.iter().enumerate() {
            if i > 0 {
                self.write_str(", ").unwrap();
            }
            write!(self, "({:?}, ", user.kind.get()).unwrap();
            self.write_operand(user.user.get()).unwrap();
            self.write_str(")").unwrap();
        }
        self.write_str(" ]").unwrap();
        self.wrap_indent();
    }
    pub fn write_ref(&self, value: impl SlabRef, kind: &str) {
        if !self.option.show_slabref {
            return;
        }
        write!(self, "; {kind}({})", value.get_handle()).unwrap();
        self.wrap_indent();
    }
    pub fn write_pred(&self, preds: &PredList) {
        if !self.option.show_preds || preds.is_empty() {
            return;
        }
        self.write_str("; Preds: [ ").unwrap();
        for (i, pred) in preds.iter().enumerate() {
            if i > 0 {
                self.write_str(", ").unwrap();
            }
            write!(self, "({:?}, ", pred.kind).unwrap();
            self.write_operand(pred.get_terminator_inst().get_parent(&*self.allocs))
                .unwrap();
            self.write_str(")").unwrap();
        }
        self.write_str(" ]").unwrap();
        self.wrap_indent();
    }
}

pub struct IRWriterOption {
    pub show_slabref: bool,
    pub show_users: bool,
    pub show_preds: bool,
}

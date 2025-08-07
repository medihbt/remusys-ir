use super::GlobalDataCommon;
use crate::{
    ir::{GlobalData, GlobalKind, GlobalRef, IRWriter, ISubValueSSA, ValueSSA, global::ISubGlobal},
    typing::id::ValTypeID,
};
use std::cell::Cell;

#[derive(Debug)]
pub struct Var {
    pub common: GlobalDataCommon,
    pub inner: Cell<VarInner>,
}

#[derive(Debug, Clone, Copy)]
pub struct VarInner {
    pub readonly: bool,
    pub init: ValueSSA,
}

impl ISubGlobal for Var {
    fn from_ir(data: &GlobalData) -> Option<&Self> {
        match data {
            GlobalData::Var(var) => Some(var),
            _ => None,
        }
    }
    fn into_ir(self) -> GlobalData {
        GlobalData::Var(self)
    }

    fn get_common(&self) -> &GlobalDataCommon {
        &self.common
    }
    fn common_mut(&mut self) -> &mut GlobalDataCommon {
        &mut self.common
    }

    fn get_kind(&self) -> GlobalKind {
        let is_extern = self.is_extern();
        let is_const = self.is_extern();
        match (is_extern, is_const) {
            (true, true) => GlobalKind::ExternConst,
            (true, false) => GlobalKind::ExternVar,
            (false, true) => GlobalKind::Const,
            (false, false) => GlobalKind::Var,
        }
    }

    fn is_readonly(&self) -> bool {
        self.inner.get().readonly
    }
    fn is_extern(&self) -> bool {
        matches!(self.inner.get().init, ValueSSA::None)
    }

    fn fmt_ir(&self, _: GlobalRef, writer: &IRWriter) -> std::io::Result<()> {
        write!(
            writer,
            "@{} = {} ",
            self.common.name,
            self.get_kind().get_ir_prefix()
        )?;
        writer.write_type(self.common.content_ty)?;

        if let ValueSSA::None = self.get_init() {
            writeln!(writer, "; no initial value")?;
        } else {
            write!(writer, "= ")?;
            self.get_init().fmt_ir(writer)?;
            writeln!(writer)?;
        }
        Ok(())
    }
}

impl Var {
    pub fn set_readonly(&self, readonly: bool) {
        let mut inner = self.inner.get();
        inner.readonly = readonly;
        self.inner.set(inner);
    }

    pub fn get_init(&self) -> ValueSSA {
        self.inner.get().init
    }
    pub fn set_init(&self, init: ValueSSA) {
        let mut inner = self.inner.get();
        inner.init = init;
        self.inner.set(inner);
    }

    pub fn new_extern(name: String, content_ty: ValTypeID, content_align: usize) -> Self {
        Self {
            common: GlobalDataCommon::new(name, content_ty, content_align),
            inner: Cell::new(VarInner { readonly: false, init: ValueSSA::None }),
        }
    }
}

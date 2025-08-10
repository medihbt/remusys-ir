use super::GlobalDataCommon;
use crate::{
    ir::{
        GlobalData, GlobalKind, GlobalRef, IRWriter, ISubValueSSA, ValueSSA,
        global::{ISubGlobal, Linkage},
    },
    typing::id::ValTypeID,
};
use std::cell::Cell;

/// 全局变量
///
/// ### LLVM IR 语法
///
/// ```llvm
/// @var_name = external constant <type>, align <align> ; 外部常量
/// @var_name = extern global <type>, align <align> ; 外部变量
/// @var_name = dso_local global <type> <initval>, align <align> ; 全局变量
/// @var_name = dso_local constant <type> <initval>, align <align> ; 全局常量
/// ```
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
    fn get_linkage(&self) -> Linkage {
        if self.is_extern() { Linkage::Extern } else { self.common.linkage.get() }
    }
    fn set_linkage(&self, linkage: Linkage) {
        self.common.linkage.set(linkage);
        if linkage == Linkage::Extern {
            self.set_init(ValueSSA::None);
        }
    }

    fn fmt_ir(&self, _: GlobalRef, writer: &IRWriter) -> std::io::Result<()> {
        write!(
            writer,
            "@{} = {} ",
            self.common.name,
            self.get_kind().get_ir_prefix(self.get_linkage())
        )?;
        writer.write_type(self.common.content_ty)?;

        if let ValueSSA::None = self.get_init() {
        } else {
            write!(writer, " ")?;
            self.get_init().fmt_ir(writer)?;
        }

        write!(writer, ", align {}", self.common.content_align)
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
        if init != ValueSSA::None {
            self.common.linkage.set(Linkage::DSOLocal);
        }
    }

    pub fn new_extern(name: String, content_ty: ValTypeID, content_align: usize) -> Self {
        Self {
            common: GlobalDataCommon::new(name, content_ty, content_align),
            inner: Cell::new(VarInner { readonly: false, init: ValueSSA::None }),
        }
    }
}

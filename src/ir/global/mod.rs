use slab::Slab;

use crate::{
    base::{INullableValue, SlabRef},
    ir::{
        IRAllocs, IRWriter, ISubValueSSA, ITraceableValue, Module, PtrStorage, UserList, ValueSSA,
        Var, global::func::Func,
    },
    typing::id::ValTypeID,
};
use std::{
    cell::{Cell, Ref},
    num::NonZero,
    ops::ControlFlow,
};

pub(super) mod func;
pub(super) mod var;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Linkage {
    Extern,
    DSOLocal,
    Private,
}

#[derive(Debug)]
pub enum GlobalData {
    Var(Var),
    Func(Func),
}

impl ISubGlobal for GlobalData {
    fn from_ir(data: &GlobalData) -> Option<&Self> {
        Some(data)
    }
    fn into_ir(self) -> GlobalData {
        self
    }
    fn get_common(&self) -> &GlobalDataCommon {
        match self {
            GlobalData::Var(var) => &var.common,
            GlobalData::Func(func) => &func.common,
        }
    }
    fn common_mut(&mut self) -> &mut GlobalDataCommon {
        match self {
            GlobalData::Var(var) => &mut var.common,
            GlobalData::Func(func) => &mut func.common,
        }
    }

    fn get_kind(&self) -> GlobalKind {
        match self {
            GlobalData::Var(var) => var.get_kind(),
            GlobalData::Func(func) => func.get_kind(),
        }
    }

    fn is_readonly(&self) -> bool {
        match self {
            GlobalData::Var(var) => var.is_readonly(),
            GlobalData::Func(_) => true, // Functions are considered read-only in this context
        }
    }

    fn is_extern(&self) -> bool {
        match self {
            GlobalData::Var(var) => var.is_extern(),
            GlobalData::Func(func) => func.is_extern(),
        }
    }

    fn get_linkage(&self) -> Linkage {
        match self {
            GlobalData::Var(var) => var.get_linkage(),
            GlobalData::Func(func) => func.get_linkage(),
        }
    }

    fn set_linkage(&self, linkage: Linkage) {
        match self {
            GlobalData::Var(var) => var.set_linkage(linkage),
            GlobalData::Func(func) => func.set_linkage(linkage),
        }
    }

    fn fmt_ir(&self, self_ref: GlobalRef, writer: &IRWriter) -> std::io::Result<()> {
        writer.write_ref(self_ref, "Global");
        writer.write_users(self.users());
        match self {
            GlobalData::Var(var) => var.fmt_ir(self_ref, writer),
            GlobalData::Func(func) => func.fmt_ir(self_ref, writer),
        }
    }
}

#[derive(Debug)]
pub struct GlobalDataCommon {
    pub name: String,
    pub content_ty: ValTypeID,
    pub content_align: usize,
    pub self_ref: GlobalRef,
    pub users: UserList,
    pub linkage: Cell<Linkage>,
}

pub trait ISubGlobal {
    fn from_ir(data: &GlobalData) -> Option<&Self>;
    fn into_ir(self) -> GlobalData;

    fn get_common(&self) -> &GlobalDataCommon;
    fn common_mut(&mut self) -> &mut GlobalDataCommon;

    fn get_kind(&self) -> GlobalKind;

    /// 判断该全局量是否为外部符号.
    fn is_extern(&self) -> bool;

    /// 获取该全局量的链接属性.
    fn get_linkage(&self) -> Linkage;

    /// 设置该全局量的链接属性.
    fn set_linkage(&self, linkage: Linkage);

    /// 该全局变量所示的 ELF 段是否只读.
    /// 只读的全局量不允许被修改, 但可以被读取.
    fn is_readonly(&self) -> bool;

    fn get_name(&self) -> &str {
        &self.get_common().name
    }

    fn fmt_ir(&self, self_ref: GlobalRef, writer: &IRWriter) -> std::io::Result<()>;
}

impl<T: ISubGlobal> PtrStorage for T {
    fn get_stored_pointee_type(&self) -> ValTypeID {
        self.get_common().content_ty
    }
    fn get_stored_pointee_align(&self) -> Option<NonZero<usize>> {
        NonZero::new(self.get_common().content_align)
    }
}

impl GlobalDataCommon {
    pub fn new(name: String, content_ty: ValTypeID, content_align: usize) -> Self {
        debug_assert!(
            content_align.is_power_of_two(),
            "Content alignment must be a power of two buf got {content_align}"
        );
        GlobalDataCommon {
            name,
            content_ty,
            content_align,
            self_ref: GlobalRef::new_null(),
            users: UserList::new_empty(),
            linkage: Cell::new(Linkage::Extern),
        }
    }

    pub fn new_empty() -> Self {
        GlobalDataCommon {
            name: String::new(),
            content_ty: ValTypeID::Void,
            content_align: 0,
            self_ref: GlobalRef::new_null(),
            users: UserList::new_empty(),
            linkage: Cell::new(Linkage::Extern),
        }
    }
}

impl ITraceableValue for GlobalData {
    fn users(&self) -> &UserList {
        match self {
            GlobalData::Var(var) => &var.common.users,
            GlobalData::Func(func) => &func.common.users,
        }
    }

    fn has_single_reference_semantics(&self) -> bool {
        true
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GlobalRef(usize);

impl SlabRef for GlobalRef {
    type RefObject = GlobalData;
    fn from_handle(handle: usize) -> Self {
        GlobalRef(handle)
    }
    fn get_handle(&self) -> usize {
        self.0
    }
}

impl ISubValueSSA for GlobalRef {
    fn try_from_ir(value: &ValueSSA) -> Option<&Self> {
        if let ValueSSA::Global(gref) = value { Some(gref) } else { None }
    }
    fn into_ir(self) -> ValueSSA {
        ValueSSA::Global(self)
    }
    fn is_zero(&self, _: &IRAllocs) -> bool {
        false
    }
    fn get_valtype(self, _: &IRAllocs) -> ValTypeID {
        ValTypeID::Ptr
    }
    fn try_gettype_noalloc(self) -> Option<ValTypeID> {
        Some(ValTypeID::Ptr)
    }
    fn fmt_ir(&self, writer: &IRWriter) -> std::io::Result<()> {
        self.to_data(&writer.allocs.globals).fmt_ir(*self, writer)
    }
}

impl GlobalRef {
    pub fn from_allocs(allocs: &mut IRAllocs, mut data: GlobalData) -> Self {
        let ret = Self::from_handle(allocs.globals.vacant_key());
        data.common_mut().self_ref = ret;
        for user in data.users() {
            user.operand.set(ValueSSA::Global(ret));
        }
        let GlobalData::Func(func) = &mut data else {
            allocs.globals.insert(data);
            return ret;
        };
        if let Some(body) = func.get_body() {
            body.forall_nodes(&allocs.blocks, |_, block| {
                block.set_parent_func(ret);
                ControlFlow::Continue(())
            });
        }
        allocs.globals.insert(data);
        return ret;
    }
    pub fn from_module(module: &Module, data: GlobalData) -> Self {
        let mut allocs = module.allocs.borrow_mut();
        Self::from_allocs(&mut allocs, data)
    }
    pub fn from_mut_module(module: &mut Module, data: GlobalData) -> Self {
        let allocs = module.allocs.get_mut();
        Self::from_allocs(allocs, data)
    }

    /// Registers this global reference to the module's symbol table.
    pub fn register_to_symtab(self, module: &Module) {
        let name = self.get_name(module).to_string();
        module.globals.borrow_mut().insert(name, self);
    }

    pub fn get_name_from_alloc<'a>(self, alloc: &'a Slab<GlobalData>) -> &'a str {
        self.to_data(alloc).get_name()
    }
    pub fn get_name<'a>(self, module: &'a Module) -> Ref<'a, str> {
        let allocs = module.allocs.borrow();
        Ref::map(allocs, |allocs| self.get_name_from_alloc(&allocs.globals))
    }
    pub fn get_name_from_mut_module<'a>(self, module: &'a mut Module) -> &'a str {
        let alloc = module.allocs.get_mut();
        self.get_name_from_alloc(&alloc.globals)
    }

    pub fn get_content_type_from_alloc(self, alloc: &Slab<GlobalData>) -> ValTypeID {
        self.to_data(alloc).get_common().content_ty
    }
    pub fn get_content_type(self, module: &Module) -> ValTypeID {
        let allocs = module.allocs.borrow();
        self.get_content_type_from_alloc(&allocs.globals)
    }
    pub fn get_content_type_from_mut_module(self, module: &mut Module) -> ValTypeID {
        let alloc = module.allocs.get_mut();
        self.get_content_type_from_alloc(&alloc.globals)
    }

    pub fn is_extern(self, allocs: &IRAllocs) -> bool {
        self.to_data(&allocs.globals).is_extern()
    }
    pub fn is_extern_from_alloc(self, alloc: &Slab<GlobalData>) -> bool {
        self.to_data(alloc).is_extern()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub enum GlobalKind {
    ExternVar,
    ExternConst,
    Var,
    Const,
    ExternFunc,
    Func,
}

impl GlobalKind {
    pub fn from_global(gref: GlobalRef, allocs: &IRAllocs) -> Self {
        Self::from_data(gref.to_data(&allocs.globals))
    }
    pub fn from_data(data: &GlobalData) -> Self {
        data.get_kind()
    }

    pub fn get_ir_prefix(self, linkage: Linkage) -> &'static str {
        use GlobalKind::*;
        use Linkage::*;
        match (self, linkage) {
            (ExternVar, _) | (Var, Extern) => "external global",
            (ExternConst, _) | (Const, Extern) => "external constant",
            (Var, DSOLocal) => "dso_local global",
            (Var, Private) => "private global",
            (Const, DSOLocal) => "dso_local constant",
            (Const, Private) => "private constant",
            (ExternFunc, _) | (Func, Extern) => "declare",
            (Func, DSOLocal) => "define dso_local",
            (Func, Private) => "define internal",
        }
    }
}

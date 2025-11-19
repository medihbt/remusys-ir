use crate::{
    base::INullableValue,
    impl_traceable_from_common,
    ir::{
        GlobalID, GlobalKind, IRAllocs, ISubGlobalID, IUser, Module, OperandSet, UseID, UseKind,
        ValueSSA,
        global::{GlobalCommon, GlobalObj, ISubGlobal, Linkage},
    },
    typing::ValTypeID,
};
use mtb_entity_slab::{IPoliciedID, PtrID};
use std::{cell::Cell, sync::Arc};

#[derive(Clone)]
pub struct GlobalVar {
    pub common: GlobalCommon,
    pub initval: [UseID; 1],
    pub readonly: Cell<bool>,
}
impl_traceable_from_common!(GlobalVar, true);
impl IUser for GlobalVar {
    fn get_operands(&self) -> OperandSet<'_> {
        OperandSet::Fixed(&self.initval)
    }

    fn operands_mut(&mut self) -> &mut [UseID] {
        &mut self.initval
    }
}
impl ISubGlobal for GlobalVar {
    fn get_common(&self) -> &GlobalCommon {
        &self.common
    }
    fn common_mut(&mut self) -> &mut GlobalCommon {
        &mut self.common
    }

    fn try_from_ir_ref(g: &GlobalObj) -> Option<&Self> {
        match g {
            GlobalObj::Var(v) => Some(v),
            _ => None,
        }
    }
    fn try_from_ir_mut(g: &mut GlobalObj) -> Option<&mut Self> {
        match g {
            GlobalObj::Var(v) => Some(v),
            _ => None,
        }
    }
    fn try_from_ir(g: GlobalObj) -> Option<Self> {
        match g {
            GlobalObj::Var(v) => Some(v),
            _ => None,
        }
    }
    fn into_ir(self) -> GlobalObj {
        GlobalObj::Var(self)
    }

    fn is_extern(&self, allocs: &IRAllocs) -> bool {
        self.initval[0].get_operand(allocs).is_null()
    }
    fn is_readonly(&self) -> bool {
        self.readonly.get()
    }
    fn get_linkage_prefix(&self, allocs: &IRAllocs) -> &'static str {
        let is_readonly = self.is_readonly();
        let linkage = self.get_linkage(allocs);
        match (is_readonly, linkage) {
            (true, Linkage::External) => "external constant",
            (true, Linkage::DSOLocal) => "dso_local constant",
            (true, Linkage::Private) => "internal constant",
            (false, Linkage::External) => "extern global",
            (false, Linkage::DSOLocal) => "dso_local global",
            (false, Linkage::Private) => "internal global",
        }
    }
    fn get_kind(&self, allocs: &IRAllocs) -> GlobalKind {
        match (self.is_extern(allocs), self.is_readonly()) {
            (true, false) => GlobalKind::ExternVar,
            (true, true) => GlobalKind::ExternConst,
            (false, false) => GlobalKind::VarDef,
            (false, true) => GlobalKind::ConstDef,
        }
    }
}
impl GlobalVar {
    pub fn builder(name: impl Into<String>, content_ty: ValTypeID) -> GlobalVarBuilder {
        GlobalVarBuilder::new(name, content_ty)
    }

    pub fn set_readonly(&self, ro: bool) {
        self.readonly.set(ro);
    }
    pub fn set_linkage(&self, linkage: Linkage) {
        self.common.back_linkage.set(linkage);
    }

    pub fn get_init(&self, allocs: &IRAllocs) -> ValueSSA {
        self.initval[0].get_operand(allocs)
    }
    pub fn set_init(&self, allocs: &IRAllocs, val: ValueSSA) {
        self.initval[0].set_operand(allocs, val);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GlobalVarID(pub PtrID<GlobalObj, <GlobalID as IPoliciedID>::PolicyT>);
impl std::fmt::Debug for GlobalVarID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GlobalVarID({:p})", self.0)
    }
}
impl ISubGlobalID for GlobalVarID {
    type GlobalT = GlobalVar;

    fn from_raw_ptr(id: PtrID<GlobalObj, <GlobalID as IPoliciedID>::PolicyT>) -> Self {
        GlobalVarID(id)
    }
    fn into_raw_ptr(self) -> PtrID<GlobalObj, <GlobalID as IPoliciedID>::PolicyT> {
        self.0
    }
}
impl GlobalVarID {
    pub fn builder(name: impl Into<String>, content_ty: ValTypeID) -> GlobalVarBuilder {
        GlobalVarBuilder::new(name, content_ty)
    }

    pub fn is_readonly(self, allocs: &IRAllocs) -> bool {
        self.deref_ir(allocs).is_readonly()
    }
    pub fn set_readonly(self, allocs: &IRAllocs, ro: bool) {
        self.deref_ir(&allocs).set_readonly(ro);
    }

    pub fn enable_init(self, allocs: &IRAllocs, initval: ValueSSA) {
        assert_ne!(
            initval,
            ValueSSA::None,
            "Cannot enable init with null ValueSSA"
        );
        let obj = self.deref_ir(allocs);
        obj.initval[0].set_operand(allocs, initval);
        if self.get_back_linkage(allocs) == Linkage::External {
            self.set_back_linkage(allocs, Linkage::DSOLocal);
        }
    }
    pub fn init_use(self, allocs: &IRAllocs) -> UseID {
        self.deref_ir(allocs).initval[0]
    }
    pub fn get_init(self, allocs: &IRAllocs) -> ValueSSA {
        self.init_use(allocs).get_operand(allocs)
    }
}

pub trait IGlobalVarBuildable: Clone {
    fn inner(&self) -> &GlobalVarBuilder;
    fn inner_mut(&mut self) -> &mut GlobalVarBuilder;

    fn new(name: impl Into<String>, content_ty: ValTypeID) -> Self;
    fn name(&mut self, name: impl Into<String>) -> &mut Self {
        self.inner_mut().name = name.into();
        self
    }
    fn edit_name(&mut self, name: impl FnOnce(&mut String)) -> &mut Self {
        name(&mut self.inner_mut().name);
        self
    }
    fn content_ty(&mut self, ty: ValTypeID) -> &mut Self {
        self.inner_mut().content_ty = ty;
        self
    }
    fn align_log(&mut self, align_log: u8) -> &mut Self {
        self.inner_mut().align_log = align_log;
        self
    }
    fn align(&mut self, align: u32) -> Option<&mut Self> {
        if !align.is_power_of_two() {
            return None;
        }
        self.inner_mut().align_log = align.trailing_zeros() as u8;
        Some(self)
    }
    fn initval(&mut self, val: ValueSSA) -> &mut Self {
        self.inner_mut().initval = val;
        self
    }
    fn readonly(&mut self, ro: bool) -> &mut Self {
        self.inner_mut().readonly = ro;
        self
    }
    fn linkage(&mut self, linkage: Linkage) -> &mut Self {
        self.inner_mut().back_linkage = linkage;
        self
    }
    fn make_extern(&mut self) -> &mut Self {
        self.inner_mut().initval = ValueSSA::None;
        self
    }
    fn make_private(&mut self) -> &mut Self {
        self.inner_mut().back_linkage = Linkage::Private;
        self
    }

    fn build_obj(&self, allocs: &IRAllocs) -> GlobalVar {
        let inner = self.inner();
        let gvar = GlobalVar {
            common: GlobalCommon::new(
                Arc::from(inner.name.as_str()),
                inner.content_ty,
                inner.align_log,
                allocs,
            ),
            initval: [UseID::new(allocs, UseKind::GlobalInit)],
            readonly: Cell::new(inner.readonly),
        };
        // Apply linkage preference first
        gvar.set_linkage(inner.back_linkage);
        if inner.initval.is_nonnull() {
            gvar.set_init(allocs, inner.initval);
            // If an initializer is present, ensure it's not External linkage
            if inner.back_linkage == Linkage::External {
                gvar.set_linkage(Linkage::DSOLocal);
            }
        }
        gvar
    }
    fn build_id(&self, module: &Module) -> Result<GlobalVarID, GlobalID> {
        let allocs = &module.allocs;
        let gvar = self.build_obj(allocs);
        GlobalVarID::allocate_export(module, gvar)
    }
    fn build_pinned(&self, module: &Module) -> GlobalVarID {
        let allocs = &module.allocs;
        let gvar = self.build_obj(allocs);
        GlobalVarID::allocate_pinned(module, gvar)
    }
    fn build_unpinned(&self, allocs: &IRAllocs) -> GlobalVarID {
        let gvar = self.build_obj(allocs);
        GlobalVarID::allocate_unpinned(allocs, gvar)
    }
}
#[derive(Clone)]
pub struct GlobalVarBuilder {
    pub name: String,
    pub content_ty: ValTypeID,
    pub align_log: u8,
    pub initval: ValueSSA,
    pub readonly: bool,
    pub back_linkage: Linkage,
}
impl IGlobalVarBuildable for GlobalVarBuilder {
    fn inner(&self) -> &GlobalVarBuilder {
        self
    }
    fn inner_mut(&mut self) -> &mut GlobalVarBuilder {
        self
    }

    fn new(name: impl Into<String>, content_ty: ValTypeID) -> Self {
        Self {
            name: name.into(),
            content_ty,
            align_log: 0,
            initval: ValueSSA::None,
            readonly: false,
            back_linkage: Linkage::DSOLocal,
        }
    }
}

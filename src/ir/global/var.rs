use crate::{
    base::INullableValue,
    impl_traceable_from_common,
    ir::{
        GlobalID, IRAllocs, ISubGlobalID, IUser, OperandSet, UseID, UseKind, UserList, ValueSSA,
        global::{GlobalCommon, GlobalObj, ISubGlobal, Linkage},
    },
    typing::ValTypeID,
};
use mtb_entity::PtrID;
use std::cell::Cell;

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
}
impl GlobalVar {
    pub fn new_extern(
        name: String,
        allocs: &IRAllocs,
        ty: ValTypeID,
        align_log: u8,
        is_const: bool,
    ) -> Self {
        Self {
            common: GlobalCommon {
                name,
                content_ty: ty,
                content_align_log: align_log,
                users: Some(UserList::new(&allocs.uses)),
                back_linkage: Cell::new(Linkage::External),
            },
            initval: [UseID::new(UseKind::GlobalInit, allocs)],
            readonly: Cell::new(is_const),
        }
    }

    pub fn is_readonly(&self) -> bool {
        self.readonly.get()
    }
    pub fn set_readonly(&self, ro: bool) {
        self.readonly.set(ro);
    }
    pub fn set_linkage(&self, linkage: Linkage) {
        self.common.back_linkage.set(linkage);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GlobalVarID(pub GlobalID);

impl ISubGlobalID for GlobalVarID {
    type GlobalT = GlobalVar;

    fn raw_from_ir(id: PtrID<GlobalObj>) -> Self {
        GlobalVarID(id)
    }
    fn into_ir(self) -> PtrID<GlobalObj> {
        self.0
    }
}
impl GlobalVarID {
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

use crate::{
    ir::{
        AttrClass, AttrSet, Attribute, AttributePos, BlockID, GlobalID, GlobalObj, IPtrUniqueUser,
        IPtrValue, IRAllocs, ISubGlobal, ISubGlobalID, ISubValueSSA, ITraceableValue, IUser,
        IValueConvert, Module, OperandSet, TerminatorID, UseID, UserList, ValueClass, ValueSSA,
        global::{GlobalCommon, Linkage},
        inst::{RetInstID, UnreachableInstID},
    },
    typing::{FuncTypeID, IValType, TypeContext, ValTypeID},
};
use mtb_entity_slab::{EntityList, EntityListIter, IPoliciedID, PtrID};
use smallvec::SmallVec;
use std::{
    cell::{Cell, Ref, RefCell, RefMut},
    sync::Arc,
};

pub trait IFuncValue: IPtrValue {
    fn get_pointee_func_type(&self) -> FuncTypeID {
        FuncTypeID::from_ir(self.get_ptr_pointee_type())
    }

    fn get_return_type(&self, tctx: &TypeContext) -> ValTypeID {
        self.get_pointee_func_type().get_ret_type(tctx)
    }
    fn get_arg_types<'t>(&self, tctx: &'t TypeContext) -> Ref<'t, [ValTypeID]> {
        self.get_pointee_func_type().get_args(tctx)
    }
    fn get_nargs(&self, tctx: &TypeContext) -> usize {
        self.get_pointee_func_type().get_nargs(tctx)
    }
    fn get_arg_type(&self, tctx: &TypeContext, index: usize) -> Option<ValTypeID> {
        self.get_arg_types(tctx).get(index).copied()
    }
    fn is_vararg(&self, tctx: &TypeContext) -> bool {
        self.get_pointee_func_type().is_vararg(tctx)
    }
}
pub trait IFuncUniqueUser: IPtrUniqueUser {
    fn get_operand_func_type(&self) -> FuncTypeID {
        FuncTypeID::from_ir(self.get_operand_pointee_type())
    }
    fn get_operand_return_type(&self, tctx: &TypeContext) -> ValTypeID {
        self.get_operand_func_type().get_ret_type(tctx)
    }
    fn get_operand_arg_types<'t>(&self, tctx: &'t TypeContext) -> Ref<'t, [ValTypeID]> {
        self.get_operand_func_type().get_args(tctx)
    }
    fn get_operand_nargs(&self, tctx: &TypeContext) -> usize {
        self.get_operand_func_type().get_nargs(tctx)
    }
    fn get_operand_arg_type(&self, tctx: &TypeContext, index: usize) -> Option<ValTypeID> {
        self.get_operand_arg_types(tctx).get(index).copied()
    }
}

pub struct FuncArg {
    pub ty: ValTypeID,
    pub index: u32,
    pub users: UserList,
    pub attrs: RefCell<AttrSet>,
    pub(in crate::ir) func: Cell<Option<FuncID>>,
}
impl ITraceableValue for FuncArg {
    fn users(&self) -> &UserList {
        &self.users
    }
    fn get_valtype(&self) -> ValTypeID {
        self.ty
    }
    fn has_unique_ref_semantics(&self) -> bool {
        true
    }
}
impl FuncArg {
    pub fn new(allocs: &IRAllocs, ty: ValTypeID, index: u32) -> Self {
        Self {
            ty,
            index,
            users: UserList::new(&allocs.uses),
            attrs: RefCell::new(AttrSet::default()),
            func: Cell::new(None),
        }
    }

    pub fn try_get_func(&self) -> Result<FuncID, &'static str> {
        self.func
            .get()
            .ok_or("FuncArg does not have a parent FuncID assigned")
    }

    pub fn attrs(&self) -> Ref<'_, AttrSet> {
        self.attrs.borrow()
    }
    pub fn attrs_mut(&self) -> RefMut<'_, AttrSet> {
        self.attrs.borrow_mut()
    }
    pub fn set_attr(&mut self, attr: Attribute) -> &mut Self {
        self.attrs.borrow_mut().set_attr(attr);
        self
    }
    pub fn has_attr_class(&self, class: AttrClass) -> bool {
        self.attrs.borrow().has_attr_class(class)
    }
    pub fn del_attr_class(&self, class: AttrClass) -> &Self {
        self.attrs.borrow_mut().clean_attr(class);
        self
    }
}

pub struct FuncObj {
    pub common: GlobalCommon,
    pub args: Box<[FuncArg]>,
    pub ret_type: ValTypeID,
    pub is_vararg: bool,
    pub body: Option<FuncBody>,
    pub attrs: RefCell<AttrSet>,
}

pub struct FuncBody {
    pub blocks: EntityList<BlockID>,
    pub entry: BlockID,
}

impl ITraceableValue for FuncObj {
    fn try_get_users(&self) -> Option<&crate::ir::UserList> {
        self.get_common().users.as_ref()
    }
    fn users(&self) -> &crate::ir::UserList {
        self.get_common().users.as_ref().unwrap()
    }
    fn get_valtype(&self) -> ValTypeID {
        ValTypeID::Ptr
    }
    fn has_unique_ref_semantics(&self) -> bool {
        true
    }
}
impl IUser for FuncObj {
    fn get_operands(&self) -> OperandSet<'_> {
        OperandSet::Fixed(&[])
    }
    fn operands_mut(&mut self) -> &mut [UseID] {
        &mut []
    }
}
impl IFuncValue for FuncObj {}
impl ISubGlobal for FuncObj {
    fn get_common(&self) -> &GlobalCommon {
        &self.common
    }
    fn common_mut(&mut self) -> &mut GlobalCommon {
        &mut self.common
    }

    fn try_from_ir_ref(g: &GlobalObj) -> Option<&Self> {
        match g {
            GlobalObj::Func(f) => Some(f),
            _ => None,
        }
    }
    fn try_from_ir_mut(g: &mut GlobalObj) -> Option<&mut Self> {
        match g {
            GlobalObj::Func(f) => Some(f),
            _ => None,
        }
    }
    fn try_from_ir(g: GlobalObj) -> Option<Self> {
        match g {
            GlobalObj::Func(f) => Some(f),
            _ => None,
        }
    }
    fn into_ir(self) -> GlobalObj {
        GlobalObj::Func(self)
    }

    fn is_extern(&self, _: &IRAllocs) -> bool {
        self.body.is_none()
    }
    fn is_readonly(&self) -> bool {
        true
    }
    fn get_linkage_prefix(&self, allocs: &IRAllocs) -> &'static str {
        match self.get_linkage(allocs) {
            Linkage::External => "declare",
            Linkage::DSOLocal => "define dso_local",
            Linkage::Private => "define internal",
        }
    }
    fn get_kind(&self, allocs: &IRAllocs) -> super::GlobalKind {
        use super::GlobalKind::*;
        if self.is_extern(allocs) { ExternFunc } else { FuncDef }
    }
}
impl FuncObj {
    pub fn builder(tctx: &TypeContext, name: impl Into<String>, functy: FuncTypeID) -> FuncBuilder {
        FuncBuilder::new(tctx, name, functy)
    }

    pub fn get_nargs(&self) -> usize {
        self.args.len()
    }

    pub fn attrs(&self) -> Ref<'_, AttrSet> {
        self.attrs.borrow()
    }
    pub fn attrs_mut(&self) -> RefMut<'_, AttrSet> {
        self.attrs.borrow_mut()
    }
    pub fn set_attr(&mut self, attr: Attribute) -> &mut Self {
        self.attrs.borrow_mut().set_attr(attr);
        self
    }
    pub fn has_attr_class(&self, class: AttrClass) -> bool {
        self.attrs.borrow().has_attr_class(class)
    }
    pub fn del_attr_class(&self, class: AttrClass) -> &Self {
        self.attrs.borrow_mut().clean_attr(class);
        self
    }

    pub fn get_blocks(&self) -> Option<&EntityList<BlockID>> {
        self.body.as_ref().map(|b| &b.blocks)
    }
    pub fn get_entry(&self) -> Option<BlockID> {
        self.body.as_ref().map(|b| b.entry)
    }
    pub fn blocks_unwrap(&self) -> &EntityList<BlockID> {
        match self.get_blocks() {
            Some(blocks) => blocks,
            None => panic!("Function {} does not have a body", self.get_name()),
        }
    }
    pub fn entry_unwrap(&self) -> BlockID {
        match self.get_entry() {
            Some(entry) => entry,
            None => panic!("Function {} does not have a body", self.get_name()),
        }
    }
    pub fn block_iter<'ir>(&'ir self, allocs: &'ir IRAllocs) -> EntityListIter<'ir, BlockID> {
        self.blocks_unwrap().iter(&allocs.blocks)
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FuncID(pub PtrID<GlobalObj, <GlobalID as IPoliciedID>::PolicyT>);
impl std::fmt::Debug for FuncID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "FuncID({:p})", self.0)
    }
}
impl ISubGlobalID for FuncID {
    type GlobalT = FuncObj;

    fn from_raw_ptr(ptr: PtrID<GlobalObj, <GlobalID as IPoliciedID>::PolicyT>) -> Self {
        FuncID(ptr)
    }
    fn into_raw_ptr(self) -> PtrID<GlobalObj, <GlobalID as IPoliciedID>::PolicyT> {
        self.0
    }
}
impl IValueConvert for FuncID {
    fn try_from_value(value: ValueSSA, module: &Module) -> Option<Self> {
        let ValueSSA::Global(gid) = value else {
            return None;
        };
        FuncID::try_from_global(&module.allocs, gid)
    }
    fn into_value(self) -> ValueSSA {
        ValueSSA::Global(self.raw_into())
    }
}
impl FuncID {
    pub fn builder(tctx: &TypeContext, name: impl Into<String>, functy: FuncTypeID) -> FuncBuilder {
        FuncBuilder::new(tctx, name, functy)
    }

    pub fn get_body(self, allocs: &IRAllocs) -> Option<&FuncBody> {
        self.deref_ir(allocs).body.as_ref()
    }
    pub fn get_blocks(self, allocs: &IRAllocs) -> Option<&EntityList<BlockID>> {
        self.get_body(allocs).map(|b| &b.blocks)
    }
    pub fn get_entry(self, allocs: &IRAllocs) -> Option<BlockID> {
        self.get_body(allocs).map(|b| b.entry)
    }
    pub fn body_unwrap(self, allocs: &IRAllocs) -> &FuncBody {
        match self.get_body(allocs) {
            Some(body) => body,
            None => panic!("Function {} does not have a body", self.get_name(allocs)),
        }
    }
    pub fn blocks_unwrap(self, allocs: &IRAllocs) -> &EntityList<BlockID> {
        self.deref_ir(allocs).blocks_unwrap()
    }
    pub fn entry_unwrap(self, allocs: &IRAllocs) -> BlockID {
        self.deref_ir(allocs).entry_unwrap()
    }
    pub fn try_blocks_iter(self, allocs: &IRAllocs) -> Option<EntityListIter<'_, BlockID>> {
        self.get_blocks(allocs)
            .map(|blocks| blocks.iter(&allocs.blocks))
    }
    pub fn blocks_iter(self, allocs: &IRAllocs) -> EntityListIter<'_, BlockID> {
        self.deref_ir(allocs).block_iter(allocs)
    }

    pub fn get_arg(self, allocs: &IRAllocs, index: usize) -> Option<&FuncArg> {
        let func = self.deref_ir(allocs);
        func.args.get(index)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FuncArgID(pub FuncID, pub u32);

impl ISubValueSSA for FuncArgID {
    fn get_class(self) -> ValueClass {
        ValueClass::FuncArg
    }
    fn try_from_ir(ir: ValueSSA) -> Option<Self> {
        match ir {
            ValueSSA::FuncArg(f, i) => Some(FuncArgID(f, i)),
            _ => None,
        }
    }
    fn into_ir(self) -> ValueSSA {
        let Self(f, i) = self;
        ValueSSA::FuncArg(f, i)
    }

    fn get_valtype(self, allocs: &IRAllocs) -> ValTypeID {
        self.deref_ir(allocs).ty
    }
    fn is_zero_const(self, _: &IRAllocs) -> bool {
        false
    }

    fn can_trace(self) -> bool {
        true
    }
    fn try_get_users(self, allocs: &IRAllocs) -> Option<&UserList> {
        Some(&self.deref_ir(allocs).users)
    }
}
impl FuncArgID {
    pub fn deref_ir(self, allocs: &IRAllocs) -> &FuncArg {
        let FuncArgID(func_id, index) = self;
        let func = func_id.deref_ir(allocs);
        &func.args[index as usize]
    }
    pub fn deref_ir_mut(self, allocs: &mut IRAllocs) -> &mut FuncArg {
        let FuncArgID(func_id, index) = self;
        let func = func_id.deref_ir_mut(allocs);
        &mut func.args[index as usize]
    }

    pub fn get_func_id(self) -> FuncID {
        self.0
    }
    pub fn get_index(self) -> u32 {
        self.1
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FuncTerminateMode {
    Unreachable,
    ReturnDefault,
    ReturnVal(ValueSSA),
}

#[derive(Debug, Clone)]
pub struct FuncBuilder {
    pub name: String,
    functype: FuncTypeID,
    ret_type: ValTypeID,
    arg_types: SmallVec<[ValTypeID; 8]>,
    is_vararg: bool,
    pub linkage: Linkage,
    pub terminate_mode: FuncTerminateMode,
    pub attrs: AttrSet,
    pub arg_attrs: Box<[AttrSet]>,
}
impl FuncBuilder {
    pub fn new(tctx: &TypeContext, name: impl Into<String>, functype: FuncTypeID) -> Self {
        let arg_attrs = {
            let nargs = functype.get_nargs(tctx);
            let mut v = Vec::with_capacity(nargs);
            for _ in 0..nargs {
                v.push(AttrSet::new(AttributePos::FUNCARG));
            }
            v.into_boxed_slice()
        };
        Self {
            name: name.into(),
            functype,
            ret_type: functype.get_ret_type(tctx),
            arg_types: SmallVec::from_slice(&functype.get_args(tctx)),
            is_vararg: false,
            linkage: Linkage::External,
            terminate_mode: FuncTerminateMode::Unreachable,
            attrs: AttrSet::new(AttributePos::FUNC),
            arg_attrs,
        }
    }

    pub fn linkage(&mut self, linkage: Linkage) -> &mut Self {
        self.linkage = linkage;
        self
    }
    pub fn make_extern(&mut self) -> &mut Self {
        self.linkage = Linkage::External;
        self
    }
    pub fn make_defined(&mut self) -> &mut Self {
        self.linkage = Linkage::DSOLocal;
        self
    }
    pub fn make_private(&mut self) -> &mut Self {
        self.linkage = Linkage::Private;
        self
    }
    pub fn name(&mut self, name: impl Into<String>) -> &mut Self {
        self.name = name.into();
        self
    }
    pub fn edit_name(&mut self, f: impl FnOnce(&mut String)) -> &mut Self {
        f(&mut self.name);
        self
    }
    pub fn terminate_mode(&mut self, mode: FuncTerminateMode) -> &mut Self {
        self.terminate_mode = mode;
        self
    }

    pub fn is_extern(&self) -> bool {
        self.linkage == Linkage::External
    }
    pub fn is_defined(&self) -> bool {
        self.linkage != Linkage::External
    }
    pub fn add_attr(&mut self, attr: Attribute) -> &mut Self {
        self.attrs.set_attr(attr);
        self
    }
    pub fn del_attr_class(&mut self, class: AttrClass) -> &mut Self {
        self.attrs.clean_attr(class);
        self
    }
    pub fn add_arg_attr(&mut self, index: usize, attr: Attribute) -> &mut Self {
        if let Some(arg_attr) = self.arg_attrs.get_mut(index) {
            arg_attr.set_attr(attr);
        }
        self
    }
    pub fn del_arg_attr_class(&mut self, index: usize, class: AttrClass) -> &mut Self {
        if let Some(arg_attr) = self.arg_attrs.get_mut(index) {
            arg_attr.clean_attr(class);
        }
        self
    }

    pub fn build_obj(&self, allocs: &IRAllocs) -> FuncObj {
        let args = {
            let mut v = Vec::with_capacity(self.arg_types.len());
            for (i, &ty) in self.arg_types.iter().enumerate() {
                v.push(FuncArg::new(allocs, ty, i as u32));
            }
            v.into_boxed_slice()
        };
        let body = if self.is_extern() {
            None
        } else {
            let blocks = EntityList::new(&allocs.blocks);
            let terminator = self.build_terminator(allocs);
            let entry = BlockID::new_with_terminator(allocs, terminator.into_ir());
            match blocks.push_back_id(entry, &allocs.blocks) {
                Ok(_) => {}
                Err(e) => {
                    let name = &self.name;
                    panic!("Failed to add entry block to function {name}: {e:?}")
                }
            }
            let body = FuncBody { blocks, entry };
            Some(body)
        };
        let name = Arc::from(self.name.as_str());
        let content_ty = self.functype.into_ir();
        let common = GlobalCommon::new(name, content_ty, 0, allocs);
        let f = FuncObj {
            common,
            args,
            ret_type: self.ret_type,
            is_vararg: self.is_vararg,
            body,
            attrs: RefCell::new(self.attrs.clone()),
        };
        f.set_back_linkage(self.linkage);
        f
    }

    pub fn build_id(&self, module: &Module) -> Result<FuncID, GlobalID> {
        let allocs = &module.allocs;
        let func = self.build_obj(allocs);
        FuncID::allocate_export(module, func)
    }
    pub fn build_pinned(&self, module: &Module) -> FuncID {
        let allocs = &module.allocs;
        let func = self.build_obj(allocs);
        FuncID::allocate_pinned(module, func)
    }
    pub fn build_unpinned(&self, allocs: &IRAllocs) -> FuncID {
        let func = self.build_obj(allocs);
        FuncID::allocate_unpinned(allocs, func)
    }

    fn build_terminator(&self, allocs: &IRAllocs) -> TerminatorID {
        match self.terminate_mode {
            FuncTerminateMode::Unreachable => {
                let unreach = UnreachableInstID::new(allocs);
                TerminatorID::Unreachable(unreach)
            }
            FuncTerminateMode::ReturnDefault => {
                let retval = ValueSSA::new_zero(self.ret_type).unwrap_or(ValueSSA::None);
                let ret = RetInstID::with_retval(allocs, retval);
                TerminatorID::Ret(ret)
            }
            FuncTerminateMode::ReturnVal(val) => {
                assert_eq!(
                    val.get_valtype(allocs),
                    self.ret_type,
                    "Return value type does not match function return type"
                );
                let ret = RetInstID::with_retval(allocs, val);
                TerminatorID::Ret(ret)
            }
        }
    }
}

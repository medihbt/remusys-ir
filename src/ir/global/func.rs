use crate::{
    impl_traceable_from_common,
    ir::{
        BlockID, BlockObj, GlobalID, GlobalObj, IPtrUniqueUser, IPtrValue, IRAllocs, ISubGlobal,
        ISubGlobalID, ISubValueSSA, ITraceableValue, IUser, Module, OperandSet, TerminatorID,
        UseID, UserList, ValueClass, ValueSSA,
        global::{GlobalCommon, Linkage},
        inst::{RetInstID, UnreachableInstID},
    },
    typing::{FuncTypeID, IValType, TypeContext, ValTypeID},
};
use mtb_entity::EntityList;
use smallvec::SmallVec;
use std::{
    cell::{Cell, Ref},
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
    pub(in crate::ir) func: Cell<Option<FuncID>>,
}
impl ITraceableValue for FuncArg {
    fn users(&self) -> &UserList {
        &self.users
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
            func: Cell::new(None),
        }
    }

    pub fn try_get_func(&self) -> Result<FuncID, &'static str> {
        self.func
            .get()
            .ok_or("FuncArg does not have a parent FuncID assigned")
    }
}

pub struct FuncObj {
    pub common: GlobalCommon,
    pub args: Box<[FuncArg]>,
    pub ret_type: ValTypeID,
    pub is_vararg: bool,
    pub body: Option<FuncBody>,
}

pub struct FuncBody {
    pub blocks: EntityList<BlockObj>,
    pub entry: BlockID,
}

impl_traceable_from_common!(FuncObj, true);
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

    // fn _init_self_id(&self, self_id: GlobalID, allocs: &IRAllocs) {
    //     self.user_init_self_id(allocs, UserID::Global(self_id));
    //     let func_id = FuncID(self_id);
    //     for arg in self.args.iter() {
    //         arg.func.set(Some(func_id));
    //         arg.traceable_init_self_id(allocs, ValueSSA::FuncArg(func_id, arg.index));
    //     }
    //     let Some(body) = &self.body else {
    //         return;
    //     };
    //     body.blocks
    //         .forall_with_sentinel(&allocs.blocks, |_, block| {
    //             block.set_parent_func(func_id);
    //             true
    //         });
    // }
    // fn dispose(&self, module: &Module) -> GlobalDisposeRes {
    //     if self.is_disposed() {
    //         return Err(GlobalDisposeError::AlreadyDisposed(None));
    //     }
    //     self.common.common_dispose(module)?;
    //     let allocs = &module.allocs;
    //     for arg in self.args.iter() {
    //         arg.func.set(None);
    //         arg.traceable_dispose(allocs);
    //     }
    //     if let Some(body) = &self.body {
    //         dispose_entity_list(&body.blocks, allocs);
    //     }
    //     self.user_dispose(allocs);
    //     Ok(())
    // }
}
impl FuncObj {
    pub fn builder(tctx: &TypeContext, name: impl Into<String>, functy: FuncTypeID) -> FuncBuilder {
        FuncBuilder::new(&tctx, name, functy)
    }

    pub fn get_nargs(&self) -> usize {
        self.args.len()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct FuncID(pub GlobalID);
impl ISubGlobalID for FuncID {
    type GlobalT = FuncObj;
    fn raw_from_ir(id: GlobalID) -> Self {
        FuncID(id)
    }
    fn into_ir(self) -> GlobalID {
        self.0
    }
}
impl FuncID {
    pub fn builder(tctx: &TypeContext, name: impl Into<String>, functy: FuncTypeID) -> FuncBuilder {
        FuncBuilder::new(&tctx, name, functy)
    }

    pub fn get_body(self, allocs: &IRAllocs) -> Option<&FuncBody> {
        self.deref_ir(allocs).body.as_ref()
    }
    pub fn get_blocks(self, allocs: &IRAllocs) -> Option<&EntityList<BlockObj>> {
        self.get_body(allocs).map(|b| &b.blocks)
    }
    pub fn get_entry(self, allocs: &IRAllocs) -> Option<BlockID> {
        self.get_body(allocs).map(|b| b.entry)
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
}
impl FuncBuilder {
    pub fn new(tctx: &TypeContext, name: impl Into<String>, functype: FuncTypeID) -> Self {
        Self {
            name: name.into(),
            functype,
            ret_type: functype.get_ret_type(tctx),
            arg_types: SmallVec::from_slice(&functype.get_args(tctx)),
            is_vararg: false,
            linkage: Linkage::External,
            terminate_mode: FuncTerminateMode::Unreachable,
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

    pub fn build_item(&self, allocs: &IRAllocs) -> FuncObj {
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
        };
        f.set_back_linkage(self.linkage);
        f
    }

    pub fn build_id(&self, module: &Module) -> Result<FuncID, GlobalID> {
        let allocs = &module.allocs;
        let func = self.build_item(allocs);
        let func_id = FuncID::allocate(allocs, func);
        func_id.register_to(module)
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

use crate::{
    impl_traceable_from_common,
    ir::{
        BlockID, BlockObj, GlobalID, GlobalObj, IPtrUniqueUser, IPtrValue, IRAllocs, ISubGlobal,
        ISubGlobalID, ISubValueSSA, ITraceableValue, IUser, OperandSet, UseID, UserList,
        ValueClass, ValueSSA, global::GlobalCommon,
    },
    typing::{FuncTypeID, IValType, TypeContext, ValTypeID},
};
use mtb_entity::EntityList;
use std::cell::Ref;

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
        Self { ty, index, users: UserList::new(&allocs.uses) }
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
}
impl FuncObj {
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
    pub fn get_body(self, allocs: &IRAllocs) -> Option<&FuncBody> {
        self.deref_ir(allocs).body.as_ref()
    }
    pub fn get_blocks(self, allocs: &IRAllocs) -> Option<&EntityList<BlockObj>> {
        self.get_body(allocs).map(|b| &b.blocks)
    }
    pub fn get_entry(self, allocs: &IRAllocs) -> Option<BlockID> {
        self.get_body(allocs).map(|b| b.entry)
    }

    pub fn is_extern(self, allocs: &IRAllocs) -> bool {
        self.deref_ir(allocs).is_extern(allocs)
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

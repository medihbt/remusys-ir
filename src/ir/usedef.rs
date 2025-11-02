use crate::{
    ir::{
        ExprID, GlobalID, IRAllocs, ISubValueSSA, InstID, ValueClass, ValueSSA,
        constant::expr::ISubExprID, global::ISubGlobalID, inst::ISubInstID,
    },
    typing::ValTypeID,
};
use mtb_entity::{
    EntityAlloc, EntityListHead, EntityRingList, EntityRingListReadIter, IEntityAllocID,
    IEntityRingListNode, PtrID,
};
use std::{
    cell::{Cell, Ref},
    ops::Deref,
};

pub enum OperandSet<'ir> {
    Fixed(&'ir [UseID]),
    Celled(Ref<'ir, [UseID]>),
    Phi(Ref<'ir, [[UseID; 2]]>),
}
impl<'ir> Clone for OperandSet<'ir> {
    fn clone(&self) -> Self {
        use OperandSet::*;
        match self {
            Fixed(slice) => Fixed(slice),
            Celled(cs) => Celled(Ref::clone(cs)),
            Phi(ps) => Phi(Ref::clone(ps)),
        }
    }
}
impl<'ir> Deref for OperandSet<'ir> {
    type Target = [UseID];
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}
impl<'ops: 'ir, 'ir> IntoIterator for &'ops OperandSet<'ir> {
    type Item = &'ops UseID;
    type IntoIter = std::slice::Iter<'ops, UseID>;

    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
    }
}
impl<'ir> OperandSet<'ir> {
    pub fn as_slice(&self) -> &[UseID] {
        use OperandSet::*;
        match self {
            Fixed(slice) => slice,
            Celled(cs) => cs.as_ref(),
            Phi(ps) => ps.as_flattened(),
        }
    }
}

pub struct OperandUseIter<'ir> {
    operands: OperandSet<'ir>,
    index: usize,
}
impl<'ir> Iterator for OperandUseIter<'ir> {
    type Item = UseID;

    fn next(&mut self) -> Option<Self::Item> {
        let slice = self.operands.as_slice();
        if self.index >= slice.len() {
            return None;
        }
        let ret = slice[self.index];
        self.index += 1;
        Some(ret)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let len = self.operands.as_slice().len();
        let remaining = len.saturating_sub(self.index);
        (remaining, Some(remaining))
    }
}
impl<'ir> ExactSizeIterator for OperandUseIter<'ir> {
    fn len(&self) -> usize {
        let len = self.operands.as_slice().len();
        len.saturating_sub(self.index)
    }
}
impl<'ir> DoubleEndedIterator for OperandUseIter<'ir> {
    fn next_back(&mut self) -> Option<Self::Item> {
        let slice = self.operands.as_slice();
        if self.index >= slice.len() {
            return None;
        }
        self.index += 1;
        Some(slice[slice.len() - self.index])
    }
}
impl<'ir> IntoIterator for OperandSet<'ir> {
    type Item = UseID;
    type IntoIter = OperandUseIter<'ir>;

    fn into_iter(self) -> Self::IntoIter {
        OperandUseIter { operands: self, index: 0 }
    }
}

pub trait IUser: ITraceableValue {
    /// 获取该用户的所有操作数.
    fn get_operands(&self) -> OperandSet<'_>;

    /// 获取该用户的所有操作数的可变引用.
    fn operands_mut(&mut self) -> &mut [UseID];

    /// 获取指定索引处的操作数对应的 SSA 值.
    fn get_operand(&self, allocs: &IRAllocs, index: usize) -> ValueSSA {
        let Some(&use_id) = self.get_operands().as_slice().get(index) else {
            return ValueSSA::None;
        };
        use_id.get_operand(allocs)
    }

    fn operands_iter(&self) -> OperandUseIter<'_> {
        self.get_operands().into_iter()
    }

    fn user_dispose(&self, allocs: &IRAllocs) {
        let operands = self.get_operands();
        for use_id in operands.into_iter() {
            use_id.dispose(allocs);
        }
        self.traceable_dispose(allocs);
    }
    fn user_init_self_id(&self, allocs: &IRAllocs, id: impl Into<UserID>) {
        let user_id = id.into();
        self.traceable_init_self_id(allocs, user_id.into());
        let operands = self.get_operands();
        for use_id in operands.into_iter() {
            use_id.set_user(allocs, Some(user_id));
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub enum UserID {
    Expr(ExprID),
    Inst(InstID),
    Global(GlobalID),
}

impl From<ExprID> for UserID {
    fn from(id: ExprID) -> Self {
        UserID::Expr(id)
    }
}
impl From<InstID> for UserID {
    fn from(id: InstID) -> Self {
        UserID::Inst(id)
    }
}
impl From<GlobalID> for UserID {
    fn from(id: GlobalID) -> Self {
        UserID::Global(id)
    }
}
impl Into<ValueSSA> for UserID {
    fn into(self) -> ValueSSA {
        match self {
            UserID::Expr(id) => ValueSSA::ConstExpr(id),
            UserID::Inst(id) => ValueSSA::Inst(id),
            UserID::Global(id) => ValueSSA::Global(id),
        }
    }
}
impl ISubValueSSA for UserID {
    fn get_class(self) -> ValueClass {
        match self {
            UserID::Expr(_) => ValueClass::ConstExpr,
            UserID::Inst(_) => ValueClass::Inst,
            UserID::Global(_) => ValueClass::Global,
        }
    }
    fn try_from_ir(ir: ValueSSA) -> Option<Self> {
        match ir {
            ValueSSA::ConstExpr(id) => Some(UserID::Expr(id)),
            ValueSSA::Inst(id) => Some(UserID::Inst(id)),
            ValueSSA::Global(id) => Some(UserID::Global(id)),
            _ => None,
        }
    }
    fn into_ir(self) -> ValueSSA {
        match self {
            UserID::Expr(id) => ValueSSA::ConstExpr(id),
            UserID::Inst(id) => ValueSSA::Inst(id),
            UserID::Global(id) => ValueSSA::Global(id),
        }
    }

    fn get_valtype(self, allocs: &IRAllocs) -> ValTypeID {
        match self {
            UserID::Expr(id) => id.get_valtype(allocs),
            UserID::Inst(id) => id.get_valtype(allocs),
            UserID::Global(id) => id.get_valtype(allocs),
        }
    }

    fn can_trace(self) -> bool {
        true
    }
    fn try_get_users(self, allocs: &IRAllocs) -> Option<&UserList> {
        match self {
            UserID::Expr(id) => id.try_get_users(allocs),
            UserID::Inst(id) => id.try_get_users(allocs),
            UserID::Global(id) => id.try_get_users(allocs),
        }
    }
}
impl UserID {
    pub fn get_operands(self, allocs: &IRAllocs) -> OperandSet<'_> {
        match self {
            UserID::Expr(id) => id.deref_ir(allocs).get_operands(),
            UserID::Inst(id) => id.deref_ir(allocs).get_operands(),
            UserID::Global(id) => id.deref_ir(allocs).get_operands(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UseKind {
    Sentinel,
    BinOpLhs,
    BinOpRhs,
    CallOpCallee,
    CallOpArg(u32),
    CastOpFrom,
    CmpLhs,
    CmpRhs,
    GepBase,
    GepIndex(u32),
    LoadSource,
    StoreSource,
    StoreTarget,

    IndexExtractAggr,
    IndexExtractIndex,
    FieldExtractAggr,

    IndexInsertAggr,
    IndexInsertElem,
    IndexInsertIndex,

    FieldInsertAggr,
    FieldInsertElem,

    /// PHI 指令的 incoming block. 语义是: 这个 Use 处在 PHI 指令 incoming 列表的第几组.
    PhiIncomingBlock(u32),

    /// PHI 指令的 incoming SSA 值. 语义是: 这个 Use 处在 PHI 指令 incoming 列表的第几组.
    PhiIncomingValue(u32),

    SelectCond,
    SelectThen,
    SelectElse,
    BranchCond,
    SwitchCond,
    RetValue,

    AmoRmwPtr,
    AmoRmwVal,

    // 以下为非指令操作数
    GlobalInit,
    ArrayElem(usize),
    StructField(usize),
    VecElem(usize),

    // 非法值, 用于占位
    DisposedUse,
}

impl UseKind {
    pub fn is_phi_incoming(&self) -> bool {
        matches!(
            self,
            UseKind::PhiIncomingBlock(_) | UseKind::PhiIncomingValue(_)
        )
    }
    pub fn is_inst_operand(&self) -> bool {
        match self {
            UseKind::Sentinel
            | UseKind::GlobalInit
            | UseKind::ArrayElem(_)
            | UseKind::StructField(_)
            | UseKind::VecElem(_) => false,
            _ => true,
        }
    }
    pub fn get_user_kind(&self) -> ValueClass {
        match self {
            Self::GlobalInit => ValueClass::Global,
            Self::ArrayElem(_) | Self::StructField(_) | Self::VecElem(_) => ValueClass::ConstExpr,
            _ => ValueClass::Inst,
        }
    }
}

#[derive(Clone)]
pub struct Use {
    list_head: Cell<EntityListHead<Use>>,
    kind: Cell<UseKind>,
    pub user: Cell<Option<UserID>>,
    pub operand: Cell<ValueSSA>,
}

impl IEntityRingListNode for Use {
    fn load_head(&self) -> EntityListHead<Self> {
        self.list_head.get()
    }
    fn store_head(&self, head: EntityListHead<Self>) {
        self.list_head.set(head);
    }

    fn is_sentinel(&self) -> bool {
        matches!(self.kind.get(), UseKind::Sentinel)
    }

    fn new_sentinel() -> Self {
        Self {
            list_head: Cell::new(EntityListHead::none()),
            kind: Cell::new(UseKind::Sentinel),
            user: Cell::new(None),
            operand: Cell::new(ValueSSA::None),
        }
    }

    fn ring_list_node_dispose(&self, alloc: &EntityAlloc<Self>) {
        self.detach(alloc)
            .expect("Use ring list node dispose detach failed");
        self.user.set(None);
        self.operand.set(ValueSSA::None);
    }
}
impl Use {
    pub fn is_disposed(&self) -> bool {
        matches!(self.kind.get(), UseKind::DisposedUse)
    }

    pub fn get_kind(&self) -> UseKind {
        self.kind.get()
    }
    pub fn set_kind(&self, kind: UseKind) {
        assert_ne!(
            kind,
            UseKind::DisposedUse,
            "Please call `use.dispose()` to dispose a Use"
        );
        self.kind.set(kind);
    }
    pub fn dispose(&self, allocs: &IRAllocs) {
        if self.is_disposed() {
            return;
        }
        self.kind.set(UseKind::DisposedUse);
        self.detach(&allocs.uses)
            .expect("Use dispose detach failed");
        self.user.set(None);
        self.operand.set(ValueSSA::None);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct UseID(pub PtrID<Use>);

impl std::fmt::Debug for UseID {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let ptr = unsafe { self.0.direct_deref() as *const _ };
        write!(f, "UseID({ptr:p})")
    }
}
impl UseID {
    pub fn inner(self) -> PtrID<Use> {
        self.0
    }

    pub fn deref_ir(self, allocs: &IRAllocs) -> &Use {
        self.inner().deref(&allocs.uses)
    }

    pub fn get_kind(self, allocs: &IRAllocs) -> UseKind {
        self.deref_ir(allocs).kind.get()
    }
    pub fn set_kind(self, allocs: &IRAllocs, kind: UseKind) {
        self.deref_ir(allocs).set_kind(kind);
    }
    pub fn is_phi_incoming(self, allocs: &IRAllocs) -> bool {
        self.get_kind(allocs).is_phi_incoming()
    }

    pub fn get_user(self, allocs: &IRAllocs) -> Option<UserID> {
        self.deref_ir(allocs).user.get()
    }
    pub fn set_user(self, allocs: &IRAllocs, user: Option<UserID>) {
        self.deref_ir(allocs).user.set(user);
    }

    pub fn get_operand(self, allocs: &IRAllocs) -> ValueSSA {
        self.deref_ir(allocs).operand.get()
    }
    pub fn raw_set_operand(self, allocs: &IRAllocs, operand: ValueSSA) {
        self.deref_ir(allocs).operand.set(operand);
    }
    pub fn set_operand(self, allocs: &IRAllocs, operand: ValueSSA) -> bool {
        let obj = self.deref_ir(allocs);
        if obj.operand.get() == operand {
            return true;
        }
        obj.detach(&allocs.uses)
            .expect("Use set_operand detach failed");
        obj.operand.set(operand);
        operand.try_add_user(allocs, self)
    }
    pub fn clean_operand(self, allocs: &IRAllocs) {
        let obj = self.deref_ir(allocs);
        if obj.operand.get() == ValueSSA::None {
            return;
        }
        obj.detach(&allocs.uses)
            .expect("Use clean_operand detach failed");
        obj.operand.set(ValueSSA::None);
    }
    pub fn dispose(self, allocs: &IRAllocs) {
        self.deref_ir(allocs).dispose(allocs);
        allocs.push_disposed(self);
    }

    pub fn new(allocs: &IRAllocs, kind: UseKind) -> Self {
        assert_ne!(kind, UseKind::DisposedUse, "Cannot allocate a disposed Use");
        let obj = Use {
            list_head: Cell::new(EntityListHead::none()),
            kind: Cell::new(kind),
            user: Cell::new(None),
            operand: Cell::new(ValueSSA::None),
        };
        UseID(allocs.uses.allocate(obj))
    }
}

pub struct UseIter<'ir>(EntityRingListReadIter<'ir, Use>);

impl<'ir> Iterator for UseIter<'ir> {
    type Item = (UseID, &'ir Use);

    fn next(&mut self) -> Option<Self::Item> {
        self.0.next().map(|(id, obj)| (UseID(id), obj))
    }
}

pub type UserList = EntityRingList<Use>;

pub trait ITraceableValue {
    fn try_get_users(&self) -> Option<&UserList> {
        Some(self.users())
    }

    /// 这个 Value 的用户列表.
    ///
    /// 注意, 只有当 Value 具有引用唯一性时, 这个列表才能反映该 Value 的所有使用者.
    /// 对于 `ConstExpr` 等不可变值, 使用者将分散在多个实例的不同 `UserList` 中.
    fn users(&self) -> &UserList;

    /// 获取该 Value 的所有使用者迭代器.
    fn user_iter<'ir>(&'ir self, allocs: &'ir IRAllocs) -> UseIter<'ir> {
        UseIter(self.users().iter(&allocs.uses))
    }

    /// 这个 Value 是否具有引用唯一性.
    fn has_unique_ref_semantics(&self) -> bool;

    fn add_user(&self, new_use: UseID, allocs: &IRAllocs) {
        self.users()
            .push_back_id(new_use.inner(), &allocs.uses)
            .expect("ITraceableValue add_user failed");
    }

    fn has_users(&self, allocs: &IRAllocs) -> bool {
        !self.users().is_empty(&allocs.uses)
    }
    fn has_single_user(&self, allocs: &IRAllocs) -> bool {
        self.users().is_single(&allocs.uses)
    }
    fn user_count(&self, allocs: &IRAllocs) -> usize {
        self.users().len(&allocs.uses)
    }

    /// 检查是否有多个不同的用户指令使用了该值
    ///
    /// ### 返回
    ///
    /// - `true` - 如果有多个不同的用户指令
    /// - `false` - 如果没有用户或只有一个用户指令
    ///
    /// ### 注意
    ///
    /// * 即使一个指令多次使用了该值 (例如作为多个操作数), 只要该指令是唯一的用户，
    ///   仍然返回 `false`.
    /// * 只有当 Value 具有引用唯一性时, 这个列表才能反映该 Value 的所有使用者.
    ///   对于 `ConstExpr` 等不可变值, 使用者可能分散在多个实例的不同 `UserList` 中,
    ///   该函数可能导致结果误报.
    fn has_multiple_users(&self, allocs: &IRAllocs) -> bool {
        let users = self.users();
        let mut first_user = None;
        for (_, u) in users.iter(&allocs.uses) {
            let user = u.user.get();
            match (first_user, user) {
                (None, Some(u)) => first_user = Some(u),
                (Some(x), Some(u)) if x != u => return true,
                _ => continue,
            }
        }
        false
    }

    fn traceable_dispose(&self, allocs: &IRAllocs) {
        let Some(users) = self.try_get_users() else {
            return;
        };
        users.clean(&allocs.uses);
        UseID(users.sentinel).dispose(allocs);
    }
    fn traceable_init_self_id(&self, allocs: &IRAllocs, id: ValueSSA) {
        self.users().forall_with_sentinel(&allocs.uses, |_, u| {
            u.operand.set(id);
            true
        });
    }
}

#[macro_export]
macro_rules! impl_traceable_from_common {
    ($TyName:ident, $has_unique_ref_semantics:expr) => {
        impl $crate::ir::ITraceableValue for $TyName {
            fn try_get_users(&self) -> Option<&$crate::ir::UserList> {
                self.get_common().users.as_ref()
            }

            fn users(&self) -> &$crate::ir::UserList {
                let Some(users) = &self.get_common().users else {
                    panic!(concat!(
                        stringify!($TyName),
                        " users list is not initialized"
                    ));
                };
                users
            }

            fn has_unique_ref_semantics(&self) -> bool {
                $has_unique_ref_semantics
            }
        }
    };
}

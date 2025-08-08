use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};

use crate::{
    base::{INullableValue, IWeakListNode, WeakList, WeakListIter},
    ir::{IRAllocs, ISubValueSSA, InstRef, ValueSSA},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UseKind {
    GuideNode,
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

    /// PHI 指令的 incoming block. 语义是: 这个 Use 处在 PHI 指令 incoming 列表的第几组.
    ///
    /// 语义变更: 原本存储的是对应 Value 的索引, 现在统一存储组号.
    PhiIncomingBlock(u32),

    /// PHI 指令的 incoming SSA 值. 语义是: 这个 Use 处在 PHI 指令 incoming 列表的第几组.
    ///
    /// 语义变更: 原本存储的是对应 Block 的索引, 现在统一存储组号.
    PhiIncomingValue(u32),

    SelectCond,
    SelectTrue,
    SelectFalse,
    BranchCond,
    SwitchCond,
    RetValue,
}

#[derive(Debug, Clone)]
pub struct Use {
    head: RefCell<(Weak<Use>, Weak<Use>)>,
    pub kind: Cell<UseKind>,
    pub inst: Cell<InstRef>,
    pub operand: Cell<ValueSSA>,
}

impl Use {
    pub fn get_operand(&self) -> ValueSSA {
        self.operand.get()
    }

    pub fn set_operand<T: ISubValueSSA>(self: &Rc<Self>, allocs: &IRAllocs, operand: T) {
        self.do_set_operand(allocs, operand.into_ir());
    }
    fn do_set_operand(self: &Rc<Self>, allocs: &IRAllocs, operand: ValueSSA) {
        let old_operand = self.operand.get();
        if old_operand == operand {
            return; // No change
        }
        self.detach();
        self.operand.set(operand);
        operand.add_user_rc(allocs, self);
    }

    /// Detach this Use from its current instruction and remove the operand.
    pub fn clean_operand(self: &Rc<Self>) {
        if self.operand.get() == ValueSSA::None {
            return; // Already cleaned
        }
        self.operand.set(ValueSSA::None);
        self.detach();
    }

    pub fn new(kind: UseKind) -> Rc<Self> {
        Rc::new(Use {
            head: RefCell::new((Weak::new(), Weak::new())),
            kind: Cell::new(kind),
            inst: Cell::new(InstRef::new_null()),
            operand: Cell::new(ValueSSA::None),
        })
    }
}

impl IWeakListNode for Use {
    fn load_head(&self) -> (Weak<Self>, Weak<Self>) {
        self.head.borrow().clone()
    }

    fn store_head(&self, head: (Weak<Self>, Weak<Self>)) {
        *self.head.borrow_mut() = head;
    }

    fn new_sentinel() -> Rc<Self> {
        Rc::new(Use {
            head: RefCell::new((Weak::new(), Weak::new())),
            kind: Cell::new(UseKind::GuideNode),
            inst: Cell::new(InstRef::new_null()),
            operand: Cell::new(ValueSSA::None),
        })
    }

    fn is_sentinel(&self) -> bool {
        self.kind.get() == UseKind::GuideNode
    }

    /// 操作数被销毁时触发该函数, 主动清理引用关系.
    fn on_list_finalize(&self) {
        self.operand.set(ValueSSA::None);
    }
}

pub type UserList = WeakList<Use>;
pub type UserIter = WeakListIter<Use>;

pub trait ITraceableValue {
    /// 这个 Value 的用户列表.
    ///
    /// 注意, 只有当 Value 具有引用唯一性时, 这个列表才能反映该 Value 的所有使用者.
    /// 对于 `ConstExpr` 等不可变值, 使用者将分散在多个实例的不同 `UserList` 中.
    fn users(&self) -> &UserList;

    /// 这个 Value 是否具有引用唯一性.
    fn has_single_reference_semantics(&self) -> bool;

    fn add_user(&self, use_ref: Weak<Use>) {
        let user_list = self.users();
        user_list.push_back(use_ref);
    }

    fn has_users(&self) -> bool {
        !self.users().is_empty()
    }
    fn has_single_user(&self) -> bool {
        self.users().is_single()
    }
    fn user_count(&self) -> usize {
        self.users().len()
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
    fn has_multiple_users(&self) -> bool {
        let users = self.users();
        let mut first_user: Option<InstRef> = None;
        for user_use in users.iter() {
            let user = user_use.inst.get();
            match first_user {
                None => first_user = Some(user),
                Some(existing_user) if existing_user != user => return true,
                Some(_) => continue,
            }
        }
        false
    }
}

impl Drop for Use {
    /// 当 Use 被销毁时，自动将其从所属的用户列表中移除
    fn drop(&mut self) {
        self.detach();
    }
}

use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};

use crate::{
    base::{INullableValue, IWeakListNode, SlabRef, WeakList, WeakListIter},
    ir::{BlockRef, FuncArgRef, IRAllocs, InstRef, ValueSSA},
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
    PhiIncomingBlock(u32),
    PhiIncomingValue(BlockRef, u32),
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
    pub fn set_operand(self: &Rc<Self>, allocs: &IRAllocs, operand: ValueSSA) {
        let old_operand = self.operand.get();
        if old_operand == operand {
            return; // No change
        }
        self.detach();
        self.operand.set(operand);

        match operand {
            ValueSSA::FuncArg(func, id) => FuncArgRef(func, id as usize)
                .to_data(&allocs.globals)
                .add_user(Rc::downgrade(self)),
            ValueSSA::Block(block) => block.to_data(&allocs.blocks).add_user(Rc::downgrade(self)),
            ValueSSA::Inst(inst) => inst.to_data(&allocs.insts).add_user(Rc::downgrade(self)),
            ValueSSA::Global(global) => global
                .to_data(&allocs.globals)
                .add_user(Rc::downgrade(self)),
            ValueSSA::ConstExpr(const_expr) => const_expr
                .to_data(&allocs.exprs)
                .add_user(Rc::downgrade(self)),
            _ => {}
        }
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
}

pub type UserList = WeakList<Use>;
pub type UserIter = WeakListIter<Use>;

pub trait ITraceableValue {
    /// 这个 Value 的用户列表.
    ///
    /// 注意, 只有当 Value 具有引用唯一性时, 这个列表才能反映
    /// 该 Value 的所有使用者. 对于 `ConstExpr` 等不可变值,
    /// 使用者将分散在多个实例的不同 `UserList` 中.
    fn users(&self) -> &UserList;

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
}

impl Drop for Use {
    /// 当 Use 被销毁时，自动将其从所属的用户列表中移除
    fn drop(&mut self) {
        self.detach();
    }
}

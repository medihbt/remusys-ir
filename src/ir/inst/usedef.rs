use std::{
    cell::{Cell, RefCell},
    rc::{Rc, Weak},
};

use crate::{
    base::{INullableValue, IWeakListNode, WeakList, WeakListIter},
    ir::{ExprRef, GlobalRef, IRAllocs, ISubValueSSA, InstRef, UserID, ValueSSA, ValueSSAClass},
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

    AmoRmwPtr,
    AmoRmwVal,

    // 以下为非指令操作数
    GlobalInit,
    ArrayElem(usize),
    StructField(usize),
    VecElem(usize),
}

impl UseKind {
    pub const fn is_phi_incoming(self) -> bool {
        matches!(
            self,
            UseKind::PhiIncomingBlock(_) | UseKind::PhiIncomingValue(_)
        )
    }

    pub const fn is_inst_operand(self) -> bool {
        !matches!(
            self,
            UseKind::GuideNode
                | UseKind::GlobalInit
                | UseKind::ArrayElem(_)
                | UseKind::StructField(_)
        )
    }

    pub const fn get_user_kind(self) -> ValueSSAClass {
        match self {
            Self::GlobalInit => ValueSSAClass::Global,
            Self::ArrayElem(_) | Self::StructField(_) => ValueSSAClass::ConstExpr,
            _ => ValueSSAClass::Inst,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Use {
    head: RefCell<(Weak<Use>, Weak<Use>)>,
    pub kind: Cell<UseKind>,
    pub user: Cell<UserID>,
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
            user: Self::user_null(kind),
            kind: Cell::new(kind),
            operand: Cell::new(ValueSSA::None),
        })
    }

    fn user_null(kind: UseKind) -> Cell<UserID> {
        use UseKind::*;
        let id = match kind {
            GlobalInit => UserID::Global(GlobalRef::new_null()),
            ArrayElem(_) | StructField(_) => UserID::Expr(ExprRef::new_null()),
            _ => UserID::Inst(InstRef::new_null()),
        };
        Cell::new(id)
    }

    pub fn get_user_kind(&self) -> ValueSSAClass {
        self.kind.get().get_user_kind()
    }
}

impl IWeakListNode for Use {
    fn load_head(&self) -> (Weak<Self>, Weak<Self>) {
        self.head.borrow().clone()
    }

    fn store_head(&self, head: (Weak<Self>, Weak<Self>)) {
        self.head.replace(head);
    }

    fn new_sentinel() -> Rc<Self> {
        Rc::new(Use {
            head: RefCell::new((Weak::new(), Weak::new())),
            kind: Cell::new(UseKind::GuideNode),
            user: Cell::new(UserID::None),
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
        let mut first_user = UserID::None;
        for user_use in users.iter() {
            let user = user_use.user.get();
            match first_user {
                UserID::None => first_user = user,
                x if x != user => return true,
                _ => continue,
            }
        }
        false
    }

    fn replace_user_operand_with(&self, allocs: &IRAllocs, new_operand: ValueSSA) {
        let users = self.users();
        let Some(mut current) = users.front() else {
            return;
        };
        loop {
            let next = current.get_next();
            if current.get_operand() == new_operand {
                return; // No change
            }
            // 这里如果 set 了一个新的 operand, 那会出现两种情况:
            //
            // 1. 如果 new_operand 是 traceable 的, 那 current 会被 detach() 然后 attach 到 new_operand 上.
            // 2. 如果 new_operand 不是 traceable 的, 那 current 会被 detach() 然后不 attach 到任何地方.
            //
            // 由于 next 已经被提前获取, 所以不会影响循环. 下一次循环时, current 会被更新为 next, 因此在 loop 第一行之后
            // 所有对 current 的操作都不会影响链表的后续部分.
            current.set_operand(allocs, new_operand);
            let next = next.upgrade().expect("next Use should not be dropped");
            if next.is_sentinel() {
                break;
            }
            current = next;
        }
    }
}

impl Drop for Use {
    /// 当 Use 被销毁时，自动将其从所属的用户列表中移除
    fn drop(&mut self) {
        self.detach();
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        base::{APInt, SlabRef},
        ir::{ConstData, FuncArgRef, FuncRef, IRBuilder, IRFocus, IUser, Opcode},
        typing::{FuncTypeRef, ValTypeID},
    };

    use super::*;

    #[test]
    fn test_use_set_operand() {
        let mut builder = IRBuilder::new_host("demo");
        let main_functy = FuncTypeRef::new(
            builder.type_ctx(),
            ValTypeID::Int(32),
            false,
            [ValTypeID::Int(32), ValTypeID::Ptr],
        );
        let main_func = builder
            .define_function_with_unreachable("main", main_functy)
            .unwrap();
        let main_func = FuncRef(main_func);
        let entry = main_func.get_entry(builder.get_allocs()).unwrap();
        builder.set_focus(IRFocus::Block(entry));

        /*
        %2:
            %3 = add i32 %0, 1
            %4 = sub i32 %0, 1
            %5 = mul i32 %3, %4
            ret i32 %5
         */
        let add_inst = builder
            .add_binop_inst(
                Opcode::Add,
                ValueSSA::FuncArg(main_func.into_ir(), 0),
                ValueSSA::ConstData(ConstData::Int(APInt::from(1))),
            )
            .unwrap();
        let sub_inst = builder
            .add_binop_inst(
                Opcode::Sub,
                ValueSSA::FuncArg(main_func.into_ir(), 0),
                ValueSSA::ConstData(ConstData::Int(APInt::from(1))),
            )
            .unwrap();
        let mul_inst = builder
            .add_binop_inst(
                Opcode::Mul,
                ValueSSA::Inst(add_inst),
                ValueSSA::Inst(sub_inst),
            )
            .unwrap();
        let _ret_inst = builder
            .focus_set_return(ValueSSA::Inst(mul_inst))
            .unwrap()
            .1;
        FuncArgRef(main_func.into_ir(), 0)
            .to_data(&builder.get_allocs().globals)
            .replace_user_operand_with(
                builder.get_allocs(),
                ValueSSA::ConstData(ConstData::Int(APInt::from(42))),
            );
        assert_eq!(
            add_inst.to_data(&builder.get_allocs().insts).get_operand(0),
            ValueSSA::ConstData(ConstData::Int(APInt::from(42)))
        );
        assert_eq!(
            sub_inst.to_data(&builder.get_allocs().insts).get_operand(0),
            ValueSSA::ConstData(ConstData::Int(APInt::from(42)))
        );
    }
}

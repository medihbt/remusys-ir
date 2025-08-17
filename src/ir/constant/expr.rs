use std::rc::Rc;

use crate::{
    base::SlabRef,
    ir::{
        Array, IRAllocs, IRWriter, IReferenceValue, ISubValueSSA, ITraceableValue, IUser, IUserRef,
        OperandSet, Struct, Use, UserID, UserList, ValueSSA,
    },
    typing::ValTypeID,
};
use slab::Slab;

#[derive(Debug, Clone)]
pub enum ConstExprData {
    Array(Array),
    Struct(Struct),
}

impl ITraceableValue for ConstExprData {
    fn users(&self) -> &UserList {
        match self {
            ConstExprData::Array(data) => &data.common.users,
            ConstExprData::Struct(data) => &data.common.users,
        }
    }

    fn has_single_reference_semantics(&self) -> bool {
        false
    }
}

impl IUser for ConstExprData {
    fn get_operands<'a>(&'a self) -> OperandSet<'a> {
        match self {
            ConstExprData::Array(data) => data.get_operands(),
            ConstExprData::Struct(data) => data.get_operands(),
        }
    }

    fn operands_mut<'a>(&'a mut self) -> &'a mut [Rc<Use>] {
        match self {
            ConstExprData::Array(data) => data.operands_mut(),
            ConstExprData::Struct(data) => data.operands_mut(),
        }
    }
}

impl ISubExpr for ConstExprData {
    fn get_common(&self) -> &ExprCommon {
        match self {
            ConstExprData::Array(data) => &data.common,
            ConstExprData::Struct(data) => &data.common,
        }
    }

    fn is_aggregate(&self) -> bool {
        matches!(self, ConstExprData::Array(_) | ConstExprData::Struct(_))
    }

    fn fmt_ir(&self, writer: &IRWriter) -> std::io::Result<()> {
        match self {
            ConstExprData::Array(data) => data.fmt_ir(writer),
            ConstExprData::Struct(data) => data.fmt_ir(writer),
        }
    }
}

impl ConstExprData {
    pub fn get_value_type(&self) -> ValTypeID {
        match self {
            ConstExprData::Array(data) => ValTypeID::Array(data.arrty),
            ConstExprData::Struct(data) => data.structty,
        }
    }
}

#[derive(Debug)]
pub struct ExprCommon {
    /// 有哪些指令使用了该常量表达式引用
    ///
    /// **重要限制**: ConstExpr 不是引用唯一的，相同值的常量表达式可能
    /// 有多个不同的 ConstExprRef。因此这个 UserList 只能反映使用了
    /// **当前这个引用** 的指令，而不是使用了相同**值**的所有指令。
    ///
    /// 要获得完整的使用信息，需要先运行 Const Expression Compression Pass
    /// 将所有值相同的表达式合并，之后 UserList 才能准确反映该值的所有使用者。
    ///
    /// ### 示例
    ///
    /// ```ignore
    /// // 编译前：两个相同值的不同引用
    /// let ref1 = ConstExpr([1, 2, 3]);  // users: [inst1, inst3]  
    /// let ref2 = ConstExpr([1, 2, 3]);  // users: [inst2, inst4]
    ///
    /// // 压缩后：合并为同一个引用
    /// let merged = ConstExpr([1, 2, 3]); // users: [inst1, inst2, inst3, inst4]
    /// ```
    pub users: UserList,
}

impl Clone for ExprCommon {
    /// 克隆时不克隆 users 列表，保持为空。因为 UserList 在设计上就不支持深拷贝.
    fn clone(&self) -> Self {
        Self { users: UserList::new_empty() }
    }
}

impl ExprCommon {
    pub fn new() -> Self {
        Self { users: UserList::new_empty() }
    }
}

pub trait ISubExpr: IUser {
    fn get_common(&self) -> &ExprCommon;
    fn is_aggregate(&self) -> bool;
    fn fmt_ir(&self, writer: &IRWriter) -> std::io::Result<()>;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ExprRef(usize);

impl SlabRef for ExprRef {
    type RefObject = ConstExprData;
    fn from_handle(handle: usize) -> Self {
        ExprRef(handle)
    }
    fn get_handle(&self) -> usize {
        self.0
    }
}

impl IReferenceValue for ExprRef {
    type ValueDataT = ConstExprData;

    fn to_value_data<'a>(self, allocs: &'a IRAllocs) -> &'a Self::ValueDataT
    where
        Self::ValueDataT: 'a,
    {
        self.to_data(&allocs.exprs)
    }

    fn to_value_data_mut<'a>(self, allocs: &'a mut IRAllocs) -> &'a mut Self::ValueDataT
    where
        Self::ValueDataT: 'a,
    {
        self.to_data_mut(&mut allocs.exprs)
    }
}

impl IUserRef for ExprRef {}

impl ISubValueSSA for ExprRef {
    fn try_from_ir(value: ValueSSA) -> Option<Self> {
        match value {
            ValueSSA::ConstExpr(x) => Some(x),
            _ => None,
        }
    }

    fn into_ir(self) -> ValueSSA {
        ValueSSA::ConstExpr(self)
    }

    fn get_valtype(self, allocs: &IRAllocs) -> ValTypeID {
        self.to_data(&allocs.exprs).get_value_type()
    }

    fn try_gettype_noalloc(self) -> Option<ValTypeID> {
        None
    }

    fn is_zero(&self, allocs: &IRAllocs) -> bool {
        match self.to_data(&allocs.exprs) {
            ConstExprData::Array(data) => data.is_zero(allocs),
            ConstExprData::Struct(data) => data.is_zero(allocs),
        }
    }

    fn fmt_ir(&self, writer: &IRWriter) -> std::io::Result<()> {
        match self.to_data(&writer.allocs.exprs) {
            ConstExprData::Array(arr) => arr.fmt_ir(writer),
            ConstExprData::Struct(str) => str.fmt_ir(writer),
        }
    }
}

impl ExprRef {
    pub fn from_alloc(alloc: &mut Slab<ConstExprData>, mut data: ConstExprData) -> Self {
        let ret = ExprRef(alloc.vacant_key());
        for user in data.users() {
            user.operand.set(ValueSSA::ConstExpr(ret));
        }
        for operands in data.operands_mut() {
            operands.user.set(UserID::Expr(ret));
        }
        alloc.insert(data);
        ret
    }
}

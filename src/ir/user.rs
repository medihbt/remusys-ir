use std::{cell::Ref, ops::Deref, rc::Rc, usize};

use crate::{
    ir::{
        ExprRef, GlobalRef, IRAllocs, IRAllocsEditable, IRAllocsReadable, IRWriter,
        IReferenceValue, ISubValueSSA, InstRef, Use, ValueSSA,
    },
    typing::ValTypeID,
};

#[derive(Debug)]
pub enum OperandSet<'a> {
    Fixed(&'a [Rc<Use>]),
    InRef(Ref<'a, [Rc<Use>]>),
    Phi(Ref<'a, Vec<[Rc<Use>; 2]>>),
}

impl Deref for OperandSet<'_> {
    type Target = [Rc<Use>];
    fn deref(&self) -> &Self::Target {
        self.as_slice()
    }
}

impl<'ops: 'inst, 'inst> IntoIterator for &'ops OperandSet<'inst> {
    type Item = &'ops Rc<Use>;
    type IntoIter = std::slice::Iter<'ops, Rc<Use>>;

    fn into_iter(self) -> Self::IntoIter {
        self.as_slice().iter()
    }
}

impl<'a> OperandSet<'a> {
    pub fn as_slice(&self) -> &[Rc<Use>] {
        match self {
            OperandSet::Fixed(ops) => ops,
            OperandSet::InRef(ops_ref) => ops_ref.deref(),
            OperandSet::Phi(ops_ref) => ops_ref.deref().as_flattened(),
        }
    }
}

pub struct OperandIter<'a> {
    operands: OperandSet<'a>,
    index: usize,
}

impl<'a> Iterator for OperandIter<'a> {
    type Item = ValueSSA;

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.operands.as_slice().len() {
            return None;
        }
        let val = self.operands.as_slice()[self.index].get_operand();
        self.index += 1;
        Some(val)
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.operands.as_slice().len() - self.index;
        (remaining, Some(remaining))
    }
}

impl<'a> ExactSizeIterator for OperandIter<'a> {
    fn len(&self) -> usize {
        self.operands.as_slice().len() - self.index
    }
}

impl<'a> OperandIter<'a> {
    pub fn new(operands: OperandSet<'a>) -> Self {
        Self { operands, index: 0 }
    }
}

pub trait IUser {
    fn get_operands(&self) -> OperandSet<'_>;

    fn operands_mut(&mut self) -> &mut [Rc<Use>];

    fn get_use_at(&self, index: usize) -> Option<Rc<Use>> {
        self.get_operands().as_slice().get(index).cloned()
    }
    fn get_operand(&self, index: usize) -> ValueSSA {
        self.get_use_at(index)
            .map_or(ValueSSA::None, |use_ref| use_ref.get_operand())
    }

    fn operands_iter(&self) -> OperandIter<'_> {
        OperandIter::new(self.get_operands())
    }
}

pub trait IUserRef: ISubValueSSA + IReferenceValue<ValueDataT: IUser> + Copy {
    fn user_operands<'a>(self, allocs: &'a IRAllocs) -> OperandSet<'a>
    where
        Self::ValueDataT: 'a,
    {
        self.to_value_data(allocs).get_operands()
    }
    fn user_operands_mut<'a>(self, allocs: &'a mut IRAllocs) -> &'a mut [Rc<Use>]
    where
        Self::ValueDataT: 'a,
    {
        self.to_value_data_mut(allocs).operands_mut()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum UserID {
    None,
    Inst(InstRef),
    Expr(ExprRef),
    Global(GlobalRef),
}

impl From<InstRef> for UserID {
    fn from(value: InstRef) -> Self {
        Self::Inst(value)
    }
}

impl From<ExprRef> for UserID {
    fn from(value: ExprRef) -> Self {
        Self::Expr(value)
    }
}

impl From<GlobalRef> for UserID {
    fn from(value: GlobalRef) -> Self {
        Self::Global(value)
    }
}

impl ISubValueSSA for UserID {
    fn try_from_ir(value: ValueSSA) -> Option<Self> {
        match value {
            ValueSSA::Inst(x) => Some(UserID::Inst(x)),
            ValueSSA::ConstExpr(x) => Some(UserID::Expr(x)),
            ValueSSA::Global(x) => Some(UserID::Global(x)),
            _ => None,
        }
    }

    fn into_ir(self) -> ValueSSA {
        match self {
            UserID::Inst(x) => ValueSSA::Inst(x),
            UserID::Expr(x) => ValueSSA::ConstExpr(x),
            UserID::Global(x) => ValueSSA::Global(x),
            UserID::None => ValueSSA::None,
        }
    }

    fn is_zero(&self, allocs: &IRAllocs) -> bool {
        match self {
            UserID::Expr(expr_ref) => expr_ref.is_zero(allocs),
            _ => false,
        }
    }

    fn get_valtype(self, allocs: &IRAllocs) -> ValTypeID {
        match self {
            UserID::None => ValTypeID::Void,
            UserID::Global(_) => ValTypeID::Ptr,
            UserID::Inst(inst_ref) => inst_ref.get_valtype(allocs),
            UserID::Expr(expr_ref) => expr_ref.get_valtype(allocs),
        }
    }

    fn try_gettype_noalloc(self) -> Option<ValTypeID> {
        match self {
            UserID::None => Some(ValTypeID::Void),
            UserID::Global(_) => Some(ValTypeID::Ptr),
            _ => None,
        }
    }

    fn fmt_ir(&self, writer: &IRWriter) -> std::io::Result<()> {
        match self {
            UserID::None => Ok(()),
            UserID::Inst(inst_ref) => inst_ref.fmt_ir(writer),
            UserID::Expr(expr_ref) => expr_ref.fmt_ir(writer),
            UserID::Global(global_ref) => global_ref.fmt_ir(writer),
        }
    }
}

impl UserID {
    pub fn user_operands(self, allocs: &impl IRAllocsReadable) -> OperandSet<'_> {
        let allocs = allocs.get_allocs_ref();
        match self {
            UserID::None => OperandSet::Fixed(&[]),
            UserID::Inst(inst_ref) => inst_ref.user_operands(allocs),
            UserID::Expr(expr_ref) => expr_ref.user_operands(allocs),
            UserID::Global(global_ref) => global_ref.user_operands(allocs),
        }
    }
    pub fn user_operands_mut(self, allocs: &mut impl IRAllocsEditable) -> &mut [Rc<Use>] {
        let allocs = allocs.get_allocs_mutref();
        match self {
            UserID::None => &mut [],
            UserID::Inst(inst_ref) => inst_ref.user_operands_mut(allocs),
            UserID::Expr(expr_ref) => expr_ref.user_operands_mut(allocs),
            UserID::Global(global_ref) => global_ref.user_operands_mut(allocs),
        }
    }
}

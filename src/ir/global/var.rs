use crate::ir::{
    IUser, OperandSet, UseID,
    global::{GlobalCommon, GlobalObj, ISubGlobal},
};
use std::cell::Cell;

#[derive(Clone)]
pub struct GlobalVar {
    pub common: GlobalCommon,
    pub initval: [UseID; 1],
    pub readonly: Cell<bool>,
}
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
}

use crate::base::slabref::SlabRef;
use crate::ir::ValueRef;

use super::InstRef;

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub struct UseRef(usize);

pub struct UseData {
    pub user:    InstRef,
    pub operand: Option<ValueRef>,
    pub prev:    Option<UseRef>,
    pub next:    Option<UseRef>,
}

impl SlabRef for UseRef {
    type Item = UseData;

    fn from_handle(handle: usize) -> Self { UseRef(handle) }
    fn get_handle (&self) -> usize { self.0 }
}

impl UseData {
    pub fn new(user: InstRef) -> Self {
        Self {
            user,
            operand: None,
            prev:    None,
            next:    None,
        }
    }
    pub fn new_with_operand(user: InstRef, operand: ValueRef) -> Self {
        Self {
            user,
            operand: Some(operand),
            prev:    None,
            next:    None,
        }
    }
}
//! Edge-Phi 指令: 类似于 Phi 指令, 但使用控制流边的 JumpTarget 来区分不同的输入值.
//! 与基本块不同, JumpTarget 不是可追踪的操作数 Value, 不会因为指令/基本块的变换而变化.

use smallvec::SmallVec;
use std::{
    cell::{Ref, RefCell},
    collections::HashMap,
};

use crate::{
    ir::{inst::*, *},
    typing::*,
};

pub struct EdgePhiInst {
    common: InstCommon,
    inner: RefCell<EdgePhiInner>,
}
struct EdgePhiInner {
    operands: SmallVec<[UseID; 2]>,
    pos_map: HashMap<JumpTargetID, usize>,
}
impl_traceable_from_common!(EdgePhiInst, false);

impl IUser for EdgePhiInst {
    fn get_operands(&self) -> OperandSet<'_> {
        let inner = self.inner.borrow();
        let ops = Ref::map(inner, |inner| inner.operands.as_slice());
        OperandSet::Celled(ops)
    }
    fn operands_mut(&mut self) -> &mut [UseID] {
        self.inner.get_mut().operands.as_mut_slice()
    }
}
impl ISubInst for EdgePhiInst {
    fn get_common(&self) -> &InstCommon {
        &self.common
    }
    fn common_mut(&mut self) -> &mut InstCommon {
        &mut self.common
    }

    fn try_from_ir_ref(inst: &InstObj) -> Option<&Self> {
        match inst {
            InstObj::EdgePhi(ephi) => Some(ephi),
            _ => None,
        }
    }
    fn try_from_ir_mut(inst: &mut InstObj) -> Option<&mut Self> {
        match inst {
            InstObj::EdgePhi(ephi) => Some(ephi),
            _ => None,
        }
    }
    fn try_from_ir(inst: InstObj) -> Option<Self> {
        match inst {
            InstObj::EdgePhi(ephi) => Some(ephi),
            _ => None,
        }
    }

    fn into_ir(self) -> InstObj {
        InstObj::EdgePhi(self)
    }
    fn try_get_jts(&self) -> Option<JumpTargets<'_>> {
        None
    }
}

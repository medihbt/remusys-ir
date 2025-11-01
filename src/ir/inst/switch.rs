use std::cell::RefCell;

use crate::{
    impl_traceable_from_common,
    ir::{
        BlockID, IRAllocs, ISubInst, ISubInstID, ISubValueSSA, ITerminatorID, ITerminatorInst,
        IUser, InstID, InstObj, JumpTargetID, JumpTargetKind, JumpTargets, Opcode, OperandSet,
        UseID, UseKind, ValueSSA, inst::InstCommon,
    },
    typing::ValTypeID,
};

pub struct SwitchInst {
    pub common: InstCommon,
    discrim: [UseID; 1],
    targets: RefCell<Vec<JumpTargetID>>,
}

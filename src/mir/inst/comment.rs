use crate::{
    base::SlabListNode,
    mir::{
        fmt::FuncFormatContext,
        inst::{IMirSubInst, MirInstCommon, inst::MirInst, opcode::MirOP},
        operand::MirOperand,
    },
};
use std::{
    cell::{Cell, RefCell},
    fmt::{Debug, Write},
};

#[derive(Clone)]
pub struct MirComment {
    _common: MirInstCommon,
    pub comment: RefCell<String>,
}

impl Debug for MirComment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "MirComment: {}", self.comment.borrow())
    }
}

impl IMirSubInst for MirComment {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        &mut self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &[]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &[]
    }
    fn accepts_opcode(_: MirOP) -> bool {
        true
    }
    fn new_empty(opcode: MirOP) -> Self {
        MirComment {
            _common: MirInstCommon::new(opcode),
            comment: RefCell::new(String::new()),
        }
    }

    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::MirComment(comment) => Some(comment),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::MirComment(self)
    }
}

impl MirComment {
    pub fn new(comment: String) -> Self {
        MirComment {
            _common: MirInstCommon::new(MirOP::MirComment),
            comment: RefCell::new(comment),
        }
    }

    pub fn set_comment(&self, comment: String) {
        *self.comment.borrow_mut() = comment;
    }
    pub fn get_comment(&self) -> String {
        self.comment.borrow().clone()
    }

    pub fn fmt_asm(&self, f: &mut FuncFormatContext) -> std::fmt::Result {
        write!(f, "# {}", self.get_comment())
    }
}

#[derive(Debug, Clone)]
pub struct MirCommentedInst(pub Box<MirInst>);

impl IMirSubInst for MirCommentedInst {
    fn get_common(&self) -> &MirInstCommon {
        self.0.get_common()
    }
    fn common_mut(&mut self) -> &mut MirInstCommon {
        self.0.common_mut()
    }
    fn get_opcode(&self) -> MirOP {
        MirOP::MirCommentedInst
    }

    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &[]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &[]
    }
    fn accepts_opcode(_: MirOP) -> bool {
        true
    }
    fn new_empty(_: MirOP) -> Self {
        Self(Box::new(MirInst::new_guide()))
    }
    fn from_mir(mir_inst: &MirInst) -> Option<&Self> {
        match mir_inst {
            MirInst::MirCommentedInst(commented_inst) => Some(commented_inst),
            _ => None,
        }
    }
    fn into_mir(self) -> MirInst {
        MirInst::MirCommentedInst(self)
    }
}

impl MirCommentedInst {
    pub fn fmt_asm(&self, f: &mut FuncFormatContext) -> std::fmt::Result {
        f.write_str("# ")?;
        f.format_inst(&self.0)
    }

    pub fn new(inst: MirInst) -> Self {
        MirCommentedInst(Box::new(inst))
    }
}

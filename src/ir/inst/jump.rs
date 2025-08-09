use std::rc::Rc;

use slab::Slab;

use crate::{
    ir::{
        BlockData, BlockRef, IRWriter, ISubInst, ITerminatorInst, InstCommon, InstData, InstRef,
        JumpTarget, JumpTargetKind, Opcode, Use,
        block::jump_target::JumpTargets,
        inst::{ISubInstRef, InstOperands},
    },
    typing::id::ValTypeID,
};

/// 无条件跳转到某个基本块
///
/// ### LLVM IR 语法
///
/// ```llvm
/// br label <block>
/// ```
#[derive(Debug)]
pub struct Jump {
    common: InstCommon,
    target: [Rc<JumpTarget>; 1],
}

impl ISubInst for Jump {
    fn new_empty(_: Opcode) -> Self {
        Self {
            common: InstCommon::new(Opcode::Jmp, ValTypeID::Void),
            target: [JumpTarget::new(JumpTargetKind::Jump)],
        }
    }
    fn try_from_ir(inst: &InstData) -> Option<&Self> {
        match inst {
            InstData::Jump(jump) => Some(jump),
            _ => None,
        }
    }
    fn try_from_ir_mut(inst: &mut InstData) -> Option<&mut Self> {
        match inst {
            InstData::Jump(jump) => Some(jump),
            _ => None,
        }
    }
    fn into_ir(self) -> InstData {
        InstData::Jump(self)
    }
    fn get_common(&self) -> &InstCommon {
        &self.common
    }
    fn common_mut(&mut self) -> &mut InstCommon {
        &mut self.common
    }
    fn is_terminator(&self) -> bool {
        true
    }
    fn get_operands(&self) -> InstOperands {
        InstOperands::Fixed(&[])
    }
    fn operands_mut(&mut self) -> &mut [Rc<Use>] {
        &mut []
    }

    fn init_self_reference(&mut self, self_ref: InstRef) {
        InstData::basic_init_self_reference(self_ref, self);
        for jt in &self.target {
            jt.terminator.set(self_ref);
        }
    }

    fn fmt_ir(&self, _: Option<usize>, writer: &IRWriter) -> std::io::Result<()> {
        writer.write_str("br label ")?;
        writer.write_operand(self.get_target())
    }

    fn cleanup(&self) {
        InstData::basic_cleanup(self);
        // 清理跳转目标
        for jt in &self.target {
            jt.clean_block();
        }
    }
}

impl ITerminatorInst for Jump {
    fn read_jts<T>(&self, reader: impl FnOnce(&[Rc<JumpTarget>]) -> T) -> T {
        reader(&self.target)
    }

    fn jts_mut(&mut self) -> &mut [Rc<JumpTarget>] {
        &mut self.target
    }

    fn get_jts(&self) -> JumpTargets {
        JumpTargets::Fixed(&self.target)
    }
}

impl Jump {
    pub fn new(alloc: &Slab<BlockData>, target: BlockRef) -> Self {
        let ret = Self {
            common: InstCommon::new(Opcode::Jmp, ValTypeID::Void),
            target: [JumpTarget::new(JumpTargetKind::Jump)],
        };
        ret.target[0].set_block(alloc, target);
        ret
    }

    pub fn get_target(&self) -> BlockRef {
        self.target[0].get_block()
    }
    pub fn set_target(&mut self, alloc: &Slab<BlockData>, target: BlockRef) {
        self.target[0].set_block(alloc, target);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct JumpRef(InstRef);

impl ISubInstRef for JumpRef {
    type InstDataT = Jump;
    fn from_raw_nocheck(inst_ref: InstRef) -> Self {
        Self(inst_ref)
    }
    fn into_raw(self) -> InstRef {
        self.0
    }
}

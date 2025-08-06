use crate::{
    ir::{
        block::jump_target::JumpTargets, inst::{ISubInstRef, InstOperands}, BlockData, BlockRef, IRAllocs, IRWriter, ISubInst, ITerminatorInst, InstCommon, InstData, InstRef, JumpTarget, JumpTargetKind, Opcode, Use, UseKind, ValueSSA
    },
    typing::id::ValTypeID,
};
use slab::Slab;
use std::rc::Rc;

/// 条件分支指令: 根据布尔条件表达式的值，跳转到不同的基本块。
/// 
/// ### LLVM 语法
/// 
/// ```llvm
/// br i1 <cond>, label <if_true>, label <if_false>
/// ```
#[derive(Debug)]
pub struct Br {
    common: InstCommon,

    /// 条件分支的操作数. 包含一个条件表达式，类型为布尔值。
    cond: [Rc<Use>; 1],

    /// 跳转目标列表
    ///
    /// * `[0] = if_true`: 条件为真时跳转的目标
    /// * `[1] = if_false`: 条件为假时跳转的目标
    targets: [Rc<JumpTarget>; 2],
}

impl ISubInst for Br {
    fn new_empty(_: Opcode) -> Self {
        Self {
            common: InstCommon::new(Opcode::Br, ValTypeID::Void),
            cond: [Use::new(UseKind::BranchCond)],
            targets: [
                JumpTarget::new(JumpTargetKind::BrTrue),
                JumpTarget::new(JumpTargetKind::BrFalse),
            ],
        }
    }
    fn try_from_ir(inst: &InstData) -> Option<&Self> {
        match inst {
            InstData::Br(br) => Some(br),
            _ => None,
        }
    }
    fn try_from_ir_mut(inst: &mut InstData) -> Option<&mut Self> {
        match inst {
            InstData::Br(br) => Some(br),
            _ => None,
        }
    }
    fn into_ir(self) -> InstData {
        InstData::Br(self)
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
        InstOperands::Fixed(&self.cond)
    }
    fn operands_mut(&mut self) -> &mut [Rc<Use>] {
        &mut self.cond
    }

    fn init_self_reference(&mut self, self_ref: InstRef) {
        self.common_mut().self_ref = self_ref;
        for user in &self.get_common().users {
            user.operand.set(ValueSSA::Inst(self_ref));
        }
        for operand in self.operands_mut() {
            operand.inst.set(self_ref);
        }
        for jt in &self.targets {
            jt.terminator.set(self_ref);
        }
    }

    fn fmt_ir(&self, _: Option<usize>, writer: &IRWriter) -> std::io::Result<()> {
        write!(writer, "br i1 ")?;
        writer.write_operand(self.get_cond())?;
        write!(writer, ", label ")?;
        writer.write_operand(self.get_if_true())?;
        write!(writer, ", label ")?;
        writer.write_operand(self.get_if_false())
    }
}

impl ITerminatorInst for Br {
    fn read_jts<T>(&self, reader: impl FnOnce(&[Rc<JumpTarget>]) -> T) -> T {
        reader(&self.targets)
    }

    fn jts_mut(&mut self) -> &mut [Rc<JumpTarget>] {
        &mut self.targets
    }

    fn get_jts(&self) -> JumpTargets {
        JumpTargets::Fixed(&self.targets)
    }
}

impl Br {
    pub fn new(allocs: &IRAllocs, cond: ValueSSA, if_true: BlockRef, if_false: BlockRef) -> Self {
        let br = Self::new_empty(Opcode::Br);
        br.cond[0].set_operand(allocs, cond);
        br.targets[0].set_block(&allocs.blocks, if_true);
        br.targets[1].set_block(&allocs.blocks, if_false);
        br
    }

    pub fn cond(&self) -> &Rc<Use> {
        &self.cond[0]
    }
    pub fn get_cond(&self) -> ValueSSA {
        self.cond[0].get_operand()
    }
    pub fn set_cond(&mut self, allocs: &IRAllocs, cond: ValueSSA) {
        self.cond[0].set_operand(allocs, cond);
    }
}

impl Br {
    pub fn jump_targets(&self) -> &[Rc<JumpTarget>] {
        &self.targets
    }

    pub fn if_true(&self) -> &Rc<JumpTarget> {
        &self.targets[0]
    }
    pub fn get_if_true(&self) -> BlockRef {
        self.targets[0].get_block()
    }
    pub fn set_if_true(&mut self, alloc: &Slab<BlockData>, block: BlockRef) {
        self.targets[0].set_block(alloc, block);
    }

    pub fn if_false(&self) -> &Rc<JumpTarget> {
        &self.targets[1]
    }
    pub fn get_if_false(&self) -> BlockRef {
        self.targets[1].get_block()
    }
    pub fn set_if_false(&mut self, alloc: &Slab<BlockData>, block: BlockRef) {
        self.targets[1].set_block(alloc, block);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct BrRef(InstRef);

impl ISubInstRef for BrRef {
    type InstDataT = Br;

    fn from_raw_nocheck(inst_ref: InstRef) -> Self {
        BrRef(inst_ref)
    }
    fn into_raw(self) -> InstRef {
        self.0
    }
}

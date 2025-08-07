use crate::{
    ir::{
        IRAllocs, IRWriter, ISubInst, ITerminatorInst, InstCommon, InstData, InstRef, JumpTarget,
        Opcode, Use, UseKind, ValueSSA,
        block::jump_target::JumpTargets,
        inst::{ISubInstRef, InstOperands},
    },
    typing::id::ValTypeID,
};
use std::rc::Rc;

/// 返回指令
///
/// ### LLVM 语法
///
/// ```llvm
/// ret <ty> <value> ; when returns a value
/// ret void ; when returns nothing
/// ```
#[derive(Debug)]
pub struct Ret {
    common: InstCommon,
    operands: [Rc<Use>; 1],
}

impl ISubInst for Ret {
    fn new_empty(_: Opcode) -> Self {
        Self {
            common: InstCommon::new(Opcode::Ret, ValTypeID::Void),
            operands: [Use::new(UseKind::RetValue)],
        }
    }
    fn try_from_ir(inst: &InstData) -> Option<&Self> {
        match inst {
            InstData::Ret(ret) => Some(ret),
            _ => None,
        }
    }
    fn try_from_ir_mut(inst: &mut InstData) -> Option<&mut Self> {
        match inst {
            InstData::Ret(ret) => Some(ret),
            _ => None,
        }
    }
    fn into_ir(self) -> InstData {
        InstData::Ret(self)
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
        InstOperands::Fixed(&self.operands)
    }
    fn operands_mut(&mut self) -> &mut [Rc<Use>] {
        &mut self.operands
    }

    fn fmt_ir(&self, _: Option<usize>, writer: &IRWriter) -> std::io::Result<()> {
        writer.write_str("ret ")?;
        if self.common.ret_type != ValTypeID::Void {
            writer.write_type(self.common.ret_type)?;
            writer.write_str(" ")?;
            writer.write_operand(self.get_retval())?;
        } else {
            writer.write_str("void")?;
        }
        Ok(())
    }
}

impl ITerminatorInst for Ret {
    fn read_jts<T>(&self, reader: impl FnOnce(&[Rc<JumpTarget>]) -> T) -> T {
        reader(&[])
    }

    fn jts_mut(&mut self) -> &mut [Rc<JumpTarget>] {
        &mut []
    }

    fn get_jts(&self) -> JumpTargets {
        JumpTargets::Fixed(&[])
    }
}

impl Ret {
    pub fn new_raw(ret_ty: ValTypeID) -> Self {
        Self {
            common: InstCommon::new(Opcode::Ret, ret_ty),
            operands: [Use::new(UseKind::RetValue)],
        }
    }
    pub fn with_retval(allocs: &IRAllocs, ret_ty: ValTypeID, ret_value: ValueSSA) -> Self {
        let ret = Self::new_raw(ret_ty);
        ret.operands[0].set_operand(allocs, ret_value);
        ret
    }

    pub fn retval(&self) -> &Rc<Use> {
        &self.operands[0]
    }
    pub fn get_retval(&self) -> ValueSSA {
        self.operands[0].get_operand()
    }
    pub fn set_retval(&mut self, allocs: &IRAllocs, ret_value: ValueSSA) {
        self.operands[0].set_operand(allocs, ret_value);
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RetRef(InstRef);

impl ISubInstRef for RetRef {
    type InstDataT = Ret;
    fn from_raw_nocheck(inst_ref: InstRef) -> Self {
        Self(inst_ref)
    }
    fn into_raw(self) -> InstRef {
        self.0
    }
}

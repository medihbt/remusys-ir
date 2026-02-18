use crate::ir::*;
use std::io::Write;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum IRLocation {
    Module,
    GlobalVar(GlobalVarID),
    Func(FuncID),
    Block(BlockID),
    Inst(InstID),
    Use(UseID),
    JumpTarget(JumpTargetID),
    Operand(ValueSSA),
}
impl From<GlobalVarID> for IRLocation {
    fn from(gvar: GlobalVarID) -> Self {
        IRLocation::GlobalVar(gvar)
    }
}
impl From<FuncID> for IRLocation {
    fn from(func: FuncID) -> Self {
        IRLocation::Func(func)
    }
}
impl From<BlockID> for IRLocation {
    fn from(block: BlockID) -> Self {
        IRLocation::Block(block)
    }
}
impl From<InstID> for IRLocation {
    fn from(inst: InstID) -> Self {
        IRLocation::Inst(inst)
    }
}
impl From<UseID> for IRLocation {
    fn from(use_id: UseID) -> Self {
        IRLocation::Use(use_id)
    }
}
impl From<JumpTargetID> for IRLocation {
    fn from(jt: JumpTargetID) -> Self {
        IRLocation::JumpTarget(jt)
    }
}
impl From<ValueSSA> for IRLocation {
    fn from(op: ValueSSA) -> Self {
        IRLocation::Operand(op)
    }
}
impl IRLocation {
    pub fn describe(&self, module: &Module, names: &IRNameMap, out: &mut dyn Write) {
        if let IRLocation::Module = self {
            write!(out, "Module {}", module.name).unwrap();
            return;
        }
        let mut serializer = IRSerializer::new(&mut *out, module, names);
        let res = match self {
            IRLocation::Module => unreachable!(),
            IRLocation::GlobalVar(gvar) => serializer.fmt_global(gvar.raw_into()),
            IRLocation::Func(func_id) => serializer.fmt_func(*func_id),
            IRLocation::Block(block_id) => serializer.fmt_block(*block_id),
            IRLocation::Inst(inst_id) => serializer.fmt_inst(*inst_id),
            IRLocation::Use(use_id) => serializer.fmt_use_info(*use_id),
            IRLocation::JumpTarget(jt) => serializer.fmt_jt_info(*jt),
            IRLocation::Operand(val) => serializer.fmt_operand(*val),
        };
        drop(serializer);
        if let Err(e) = res {
            write!(out, "Error describing location: {e}").unwrap();
        }
    }
}

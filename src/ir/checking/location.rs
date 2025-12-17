use crate::ir::{
    BlockID, FuncID, GlobalVarID, IRAllocs, IRWriteOption, IRWriter, ISubGlobal, ISubGlobalID,
    ISubInstID, InstID, JumpTargetID, Module, UseID, UserID, ValueSSA, WriteIR,
};
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
    pub fn describe(&self, module: &Module, out: &mut dyn Write) {
        let mut writer = IRWriter::from_module(out, module);
        let allocs = &module.allocs;
        match self {
            IRLocation::Module => write!(writer, "Module {}", module.name).unwrap(),
            IRLocation::GlobalVar(gvar) => writer.fmt_global_var(*gvar).unwrap(),
            IRLocation::Func(func_id) => {
                let func = func_id.deref_ir(allocs);
                write!(writer, "Function @{}", func.get_name()).unwrap();
            }
            IRLocation::Block(block_id) => {
                let Some(parent_func) = block_id.get_parent_func(allocs) else {
                    write!(writer, "{:?} (orphan)", block_id).unwrap();
                    return;
                };
                writer
                    .switch_to_func(parent_func)
                    .unwrap()
                    .fmt_block_target(Some(*block_id))
                    .unwrap();
            }
            IRLocation::Inst(inst_id) => {
                Self::describe_inst(&mut writer, allocs, *inst_id);
            }
            IRLocation::Use(use_id) => {
                let user = use_id.get_user(allocs);
                let kind = use_id.get_kind(allocs);
                let operand = use_id.get_operand(allocs);
                writeln!(
                    writer,
                    "Use {use_id:?} (kind: {kind:?}, user: {user:?}, operand: {operand:?})"
                )
                .unwrap();

                if let Some(UserID::Inst(inst)) = user {
                    write!(writer, "Related inst:").unwrap();
                    Self::describe_inst(&mut writer, allocs, inst);
                    writeln!(writer).unwrap();
                }
            }
            IRLocation::JumpTarget(jt) => {
                let kind = jt.get_kind(allocs);
                let termi = jt.get_terminator(allocs);
                let block = jt.get_block(allocs);
                write!(
                    writer,
                    "JumpTarget {jt:?} (kind: {kind:?}, termi: {termi:?}, block: {block:?})"
                )
                .unwrap();

                if let Some(termi_id) = termi {
                    write!(writer, "\nRelated terminator:").unwrap();
                    Self::describe_inst(&mut writer, allocs, termi_id);
                    writeln!(writer).unwrap();
                }
            }
            IRLocation::Operand(op) => writer.fmt_operand(*op).unwrap(),
        }
    }

    fn describe_inst(writer: &mut IRWriter<'_>, allocs: &IRAllocs, inst_id: InstID) {
        let opcode = inst_id.get_opcode(allocs);
        let Some(parent_bb) = inst_id.get_parent(allocs) else {
            write!(writer, "{inst_id:?} opcode {opcode:?} (orphan)").unwrap();
            return;
        };
        let Some(parent_func) = parent_bb.get_parent_func(allocs) else {
            write!(writer, "{inst_id:?} opcode {opcode:?} (orphan block)").unwrap();
            return;
        };
        writer.set_option(IRWriteOption::loud());
        let inst_writer = writer.switch_to_func(parent_func).unwrap();
        inst_writer.fmt_instid(inst_id).unwrap();
    }
}

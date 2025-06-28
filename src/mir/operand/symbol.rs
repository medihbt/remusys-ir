use crate::mir::module::{ModuleItemRef, block::MirBlockRef};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolOperand {
    Label(MirBlockRef),
    /// Index of global symbol table.
    Global(ModuleItemRef),
}

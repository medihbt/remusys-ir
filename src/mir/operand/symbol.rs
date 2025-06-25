use crate::mir::module::block::MirBlockRef;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SymbolOperand {
    Label(MirBlockRef),
    /// Index of global symbol table.
    Global(u32),
}

use crate::ir::block::BlockRef;

pub mod block;
pub mod func;

pub enum SymbolRef {
    Block(BlockRef),
    Global(usize),
}

pub enum GlobalSymbol {

}
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddrMode {
    BaseOnly,
    BaseOffset,
    PreIndex,
    PostIndex,
    Literal,
    PseudoImmMaker,
}

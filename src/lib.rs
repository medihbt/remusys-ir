pub use {mtb_entity_slab, slab, smol_str};

pub mod base;
pub mod ir;
pub mod opt;
pub mod testing;
pub mod typing;

/// Remusys-IR uses SmolStr as symbol string so that
/// it can optimize for both memory usage and performance
pub type SymbolStr = smol_str::SmolStr;

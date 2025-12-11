//! IR checking utilities.

mod dominance_check;
mod location;
mod sanity;

pub use self::{
    dominance_check::{
        DominanceCheckErr, DominanceCheckRes, FuncDominanceCheck, assert_func_dominance,
        assert_module_dominance, module_dominance_check,
    },
    location::IRLocation,
    sanity::{IRSanityErr, IRSanityRes, assert_module_sane, basic_sanity_check},
};

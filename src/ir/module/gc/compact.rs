use crate::ir::module::Module;

use super::liveset::IRRefLiveSet;

pub(super) struct Redirector<'a> {
    module: &'a Module,
    live_set: &'a IRRefLiveSet,
}


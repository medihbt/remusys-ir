use crate::ir::GlobalID;
use std::cell::Cell;

struct WriterStat {
    curr_func: Cell<Option<GlobalID>>,
}

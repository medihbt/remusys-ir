use crate::ir::{FuncID, GlobalID};
use std::cell::{Cell, RefCell};

pub struct IRWriterStat {
    curr_func: Cell<Option<FuncID>>,
}

impl IRWriterStat {
    pub fn new() -> Self {
        Self { curr_func: Cell::new(None) }
    }

    pub fn hold_curr_func<'stat>(&'stat self, funcid: FuncID) -> impl Drop + 'stat {
        let prev_func = self.curr_func.replace(Some(funcid));
        struct Guard<'stat> {
            stat: &'stat IRWriterStat,
            prev_func: Option<FuncID>,
        }
        impl<'stat> Drop for Guard<'stat> {
            fn drop(&mut self) {
                self.stat.curr_func.set(self.prev_func);
            }
        }
        Guard { stat: self, prev_func }
    }
}

pub struct IRWriter<'ir> {
    pub output: RefCell<&'ir mut dyn std::io::Write>,
    pub stat: IRWriterStat,
}

use crate::ir::{BlockID, FuncID, IRAllocs, ISubInstID, InstID};

#[derive(Debug, Clone)]
pub struct IRFullFocus {
    pub func: FuncID,
    pub block: Option<BlockID>,
    pub inst: Option<InstID>,
}
impl IRFullFocus {
    pub fn is_block_focus(&self) -> bool {
        self.block.is_some() && self.inst.is_none()
    }
    pub fn is_inst_focus(&self) -> bool {
        self.block.is_some() && self.inst.is_some()
    }

    pub fn new_func_focus(func: FuncID) -> Self {
        Self { func, block: None, inst: None }
    }
}

#[derive(Debug, Clone)]
pub enum IRFocus {
    Block(BlockID),
    Inst(InstID),
}

impl IRFocus {
    pub fn from_full(full: &IRFullFocus) -> Option<Self> {
        match (full.block, full.inst) {
            (Some(b), None) => Some(IRFocus::Block(b)),
            (Some(_), Some(i)) => Some(IRFocus::Inst(i)),
            _ => None,
        }
    }

    pub fn to_full(&self, allocs: impl AsRef<IRAllocs>) -> IRFullFocus {
        let allocs = allocs.as_ref();
        match self {
            IRFocus::Block(block) => {
                let Some(func) = block.get_parent_func(allocs) else {
                    panic!("BlockID has no parent FuncID");
                };
                IRFullFocus { func, block: Some(*block), inst: None }
            }
            IRFocus::Inst(inst) => {
                let Some(block) = inst.get_parent(allocs) else {
                    panic!("InstID has no parent BlockID");
                };
                let Some(func) = block.get_parent_func(allocs) else {
                    panic!("BlockID has no parent FuncID");
                };
                IRFullFocus { func, block: Some(block), inst: Some(*inst) }
            }
        }
    }
}

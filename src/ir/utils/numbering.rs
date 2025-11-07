use crate::{
    ir::{
        BlockID, FuncID, IRAllocs, ISubGlobalID, ISubInst, ISubValueSSA, InstID, InstObj, ValueSSA,
    },
    typing::ValTypeID,
};

#[derive(Debug, Clone, Copy)]
pub struct NumberOption {
    pub ignore_void: bool,
    pub ignore_terminator: bool,
    pub ignore_guide: bool,
}

impl NumberOption {
    pub fn ignore_all() -> Self {
        NumberOption {
            ignore_void: true,
            ignore_terminator: true,
            ignore_guide: true,
        }
    }
    pub fn ignore_none() -> Self {
        NumberOption {
            ignore_void: false,
            ignore_terminator: false,
            ignore_guide: false,
        }
    }
}

pub struct IRNumberValueMap {
    pub insts: Box<[(InstID, usize)]>,
    pub blocks: Box<[(BlockID, usize)]>,
    pub func: FuncID,
}

impl IRNumberValueMap {
    pub fn new(allocs: &IRAllocs, func: FuncID, option: NumberOption) -> Option<Self> {
        let (mut blocks, inst_num) = Self::collect_blocks(allocs, func)?;

        let mut insts = Vec::with_capacity(inst_num);
        let mut number_counter = func.deref_ir(allocs).get_nargs();
        for (bid, bnum) in blocks.iter_mut() {
            *bnum = number_counter;
            number_counter += 1;
            for (inst_id, inst) in bid.get_insts(allocs).iter(&allocs.insts) {
                if option.ignore_void && inst.get_valtype() == ValTypeID::Void {
                    continue;
                }
                if option.ignore_terminator && inst.is_terminator() {
                    continue;
                }
                if option.ignore_guide && matches!(inst, InstObj::PhiInstEnd(_)) {
                    continue;
                }
                insts.push((inst_id, number_counter));
                number_counter += 1;
            }
        }
        blocks.sort_unstable_by_key(Self::first);
        insts.sort_unstable_by_key(Self::first);
        let insts = insts.into_boxed_slice();
        Some(Self { insts, blocks, func })
    }

    fn collect_blocks(allocs: &IRAllocs, func: FuncID) -> Option<(Box<[(BlockID, usize)]>, usize)> {
        let body = func.get_body(allocs)?;
        let nblocks = body.blocks.len();
        let mut blocks = Vec::with_capacity(nblocks);
        let mut inst_num = 0;
        for (i, (bid, bb)) in body.blocks.iter(&allocs.blocks).enumerate() {
            blocks.push((BlockID(bid), i));
            inst_num += bb.get_body().insts.len() - 1; // exclude PhiEnd splitter
        }
        Some((blocks.into_boxed_slice(), inst_num))
    }
    fn first<T: Copy, U>((p, _): &(T, U)) -> T {
        *p
    }

    pub fn inst_get_number(&self, inst: InstID) -> Option<usize> {
        self.insts
            .binary_search_by_key(&inst, Self::first)
            .ok()
            .map(|idx| self.insts[idx].1)
    }
    pub fn block_get_number(&self, block: BlockID) -> Option<usize> {
        self.blocks
            .binary_search_by_key(&block, Self::first)
            .ok()
            .map(|idx| self.blocks[idx].1)
    }
    pub fn value_get_number(&self, value: impl ISubValueSSA) -> Option<usize> {
        match value.into_ir() {
            ValueSSA::Inst(inst) => self.inst_get_number(inst),
            ValueSSA::Block(block) => self.block_get_number(block),
            ValueSSA::FuncArg(_, argidx) => Some(argidx as usize),
            _ => None,
        }
    }
}

use smallvec::{SmallVec, smallvec};
use smol_str::ToSmolStr;

use crate::{
    SymbolStr,
    ir::{
        indexed_ir::{IPoolAllocatedIndex, PoolAllocatedIndex},
        inst::*,
        module::allocs::IPoolAllocated,
        *,
    },
};
use std::collections::HashMap;

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

/// Persistent storage of explicit/string names.
/// Only holds names that are considered persistent (from source or externally registered).
#[derive(Debug, Clone, Default)]
pub struct IRNameMap {
    pub funcs: HashMap<GlobalIndex, Box<[Option<SymbolStr>]>>,
    pub insts: HashMap<InstIndex, SymbolStr>,
    pub blocks: HashMap<BlockIndex, SymbolStr>,
}

impl IRNameMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn insert_func_args(&mut self, func_index: GlobalIndex, nargs: usize) {
        self.funcs
            .entry(func_index)
            .or_insert_with(|| vec![None; nargs].into_boxed_slice());
    }
    pub fn set_func_arg(&mut self, func_index: GlobalIndex, arg: usize, name: SymbolStr) {
        self.funcs
            .entry(func_index)
            .or_insert_with(|| vec![None; arg + 1].into_boxed_slice());
        let Some(func_info) = self.funcs.get_mut(&func_index) else {
            return;
        };
        if let Some(slot) = func_info.get_mut(arg) {
            *slot = Some(name);
        }
    }

    pub fn insert_inst(&mut self, idx: InstIndex, name: SymbolStr) {
        self.insts.insert(idx, name);
    }
    pub fn insert_block(&mut self, idx: BlockIndex, name: SymbolStr) {
        self.blocks.insert(idx, name);
    }

    /// Return persistent name for a value, if present.
    pub fn get_local_name(&self, allocs: &IRAllocs, val: impl IValueConvert) -> Option<SymbolStr> {
        let val = val.into_value();
        match val {
            ValueSSA::FuncArg(func_id, index) => {
                let func_index = func_id.to_indexed(allocs);
                let info = self.funcs.get(&func_index)?;
                info.get(index as usize).and_then(|opt| opt.clone())
            }
            ValueSSA::Block(block_id) => {
                let block_index = block_id.to_indexed(allocs);
                self.blocks.get(&block_index).cloned()
            }
            ValueSSA::Inst(inst_id) => {
                let inst_index = inst_id.to_indexed(allocs);
                self.insts.get(&inst_index).cloned()
            }
            _ => None,
        }
    }

    pub fn gc(&mut self, allocs: &IRAllocs) {
        self.funcs
            .retain(|index, _| index.as_primary(allocs).is_some());
        self.blocks
            .retain(|index, _| index.as_primary(allocs).is_some());
        self.insts
            .retain(|index, _| index.as_primary(allocs).is_some());
    }
}

pub struct FuncNumberMap<'a> {
    pub names: &'a IRNameMap,
    pub func: FuncID,
    pub args: SmallVec<[u32; 4]>,
    pub insts: HashMap<InstID, usize>,
    pub blocks: HashMap<BlockID, usize>,
}

impl<'a> FuncNumberMap<'a> {
    pub fn new(
        allocs: &IRAllocs,
        func_id: FuncID,
        names: &'a IRNameMap,
        option: NumberOption,
    ) -> Self {
        let mut number = 0usize;
        let args = Self::number_args(allocs, func_id, names, &mut number);
        let body = func_id.body_unwrap(allocs);
        let mut blocks = HashMap::with_capacity(body.blocks.len());
        let mut insts = HashMap::new();

        for (bbid, bb) in body.blocks.iter(&allocs.blocks) {
            let block_index = bbid.to_indexed(allocs);
            if !names.blocks.contains_key(&block_index) {
                blocks.insert(bbid, number);
                number += 1;
            }

            for (instid, inst) in bb.get_insts().iter(&allocs.insts) {
                if option.ignore_guide && matches!(inst, InstObj::PhiInstEnd(_)) {
                    continue;
                }
                if option.ignore_terminator && inst.is_terminator() {
                    continue;
                }
                if option.ignore_void && inst.get_valtype() == ValTypeID::Void {
                    continue;
                }
                let inst_index = instid.to_indexed(allocs);
                if !names.insts.contains_key(&inst_index) {
                    insts.insert(instid, number);
                    number += 1;
                }
            }
        }

        Self { names, func: func_id, args, insts, blocks }
    }

    fn number_args(
        allocs: &IRAllocs,
        func_id: FuncID,
        names: &'a IRNameMap,
        number: &mut usize,
    ) -> SmallVec<[u32; 4]> {
        let func_obj = func_id.deref_ir(allocs);
        let mut args: SmallVec<[u32; 4]> = smallvec![u32::MAX; func_obj.args.len()];
        for (i, arg) in args.iter_mut().enumerate() {
            let name = names.get_local_name(allocs, FuncArgID(func_id, i as u32));
            if name.is_some() {
                continue;
            }
            *arg = *number as u32;
            *number += 1;
        }
        args
    }

    pub fn get_local_name(&self, allocs: &IRAllocs, val: impl IValueConvert) -> Option<SymbolStr> {
        // prefer persistent
        if let Some(p) = self.names.get_local_name(allocs, val) {
            return Some(p);
        }
        let val = val.into_value();
        match val {
            ValueSSA::FuncArg(func_id, index) if func_id == self.func => {
                let idx = index as usize;
                if idx < self.args.len() && self.args[idx] != u32::MAX {
                    Some(self.args[idx].to_smolstr())
                } else {
                    None
                }
            }
            ValueSSA::Block(block_id) => self.blocks.get(&block_id).map(ToSmolStr::to_smolstr),
            ValueSSA::Inst(inst_id) => self.insts.get(&inst_id).map(ToSmolStr::to_smolstr),
            _ => None,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct IRSourcePos {
    pub byte_offset: usize,
    pub line: usize,
    pub column_nchars: usize,
}
pub type IRSourceRange = (IRSourcePos, IRSourcePos);

impl std::default::Default for IRSourcePos {
    fn default() -> Self {
        Self::INITIAL
    }
}
impl IRSourcePos {
    pub const INITIAL: Self = Self { byte_offset: 0, line: 1, column_nchars: 0 };

    pub fn advance(&mut self, s: &str) {
        for c in s.chars() {
            if c == '\n' {
                self.line += 1;
                self.column_nchars = 0;
            } else {
                self.column_nchars += 1;
            }
        }
        self.byte_offset += s.len();
    }
}

/// A wrapper around a writer that tracks the current
/// source position (line, column, byte offset) as it writes.
pub struct SourceMapWriter<W> {
    pub writer: W,
    pub curr_pos: IRSourcePos,
}
impl<W> std::fmt::Write for SourceMapWriter<W>
where
    W: std::fmt::Write,
{
    fn write_str(&mut self, s: &str) -> std::fmt::Result {
        self.writer.write_str(s)?;
        self.curr_pos.advance(s);
        Ok(())
    }
}
impl<W> std::io::Write for SourceMapWriter<W>
where
    W: std::io::Write,
{
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        let s = std::str::from_utf8(buf)
            .map_err(|e| std::io::Error::new(std::io::ErrorKind::InvalidData, e))?;
        self.writer.write(buf)?;
        self.curr_pos.advance(s);
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.writer.flush()
    }
}
impl<W> SourceMapWriter<W> {
    pub fn new(writer: W) -> Self {
        Self { writer, curr_pos: IRSourcePos::INITIAL }
    }
    pub fn with_pos(writer: W, curr_pos: IRSourcePos) -> Self {
        Self { writer, curr_pos }
    }
}

#[derive(Debug, Clone, Default)]
pub struct SourceRangeMap {
    pub insts: HashMap<InstIndex, IRSourceRange>,
    pub blocks: HashMap<BlockIndex, IRSourceRange>,
    pub globals: HashMap<GlobalIndex, IRSourceRange>,
    pub funcargs: HashMap<GlobalIndex, Box<[Option<IRSourceRange>]>>,
    pub uses: HashMap<UseIndex, IRSourceRange>,
    pub jts: HashMap<JumpTargetIndex, IRSourceRange>,
}

impl SourceRangeMap {
    pub fn new() -> Self {
        Self::default()
    }

    /// Remove source ranges for any IR entities that have been deallocated.
    pub fn gc(&mut self, allocs: &IRAllocs) {
        self.insts.retain(|index, _| {
            index
                .try_deref_ir(allocs)
                .is_some_and(|i| !i.obj_disposed())
        });
        self.blocks.retain(|index, _| {
            index
                .try_deref_ir(allocs)
                .is_some_and(|b| !b.obj_disposed())
        });
        self.globals.retain(|index, _| {
            index
                .try_deref_ir(allocs)
                .is_some_and(|g| !g.obj_disposed())
        });
        self.funcargs.retain(|index, _| {
            index
                .try_deref_ir(allocs)
                .is_some_and(|f| !f.obj_disposed())
        });
        self.uses.retain(|index, _| {
            index
                .try_deref_ir(allocs)
                .is_some_and(|u| !u.obj_disposed())
        });
        self.jts.retain(|index, _| {
            index
                .try_deref_ir(allocs)
                .is_some_and(|j| !j.obj_disposed())
        });
    }

    pub fn primary_insert_range(
        &mut self,
        allocs: &IRAllocs,
        id: impl Into<PoolAllocatedID>,
        range: IRSourceRange,
    ) {
        let id = PoolAllocatedIndex::from_primary(allocs, id.into());
        self.index_insert_range(id, range);
    }
    pub fn primary_get_range(
        &self,
        allocs: &IRAllocs,
        id: impl Into<PoolAllocatedID>,
    ) -> Option<&IRSourceRange> {
        let id = PoolAllocatedIndex::from_primary(allocs, id.into());
        self.index_get_range(id)
    }
    pub fn index_insert_range(&mut self, id: impl Into<PoolAllocatedIndex>, range: IRSourceRange) {
        let id = id.into();
        match id {
            PoolAllocatedIndex::Inst(inst_index) => {
                self.insts.insert(inst_index, range);
            }
            PoolAllocatedIndex::Block(block_index) => {
                self.blocks.insert(block_index, range);
            }
            PoolAllocatedIndex::Global(global_index) => {
                self.globals.insert(global_index, range);
            }
            PoolAllocatedIndex::Use(use_index) => {
                self.uses.insert(use_index, range);
            }
            PoolAllocatedIndex::JT(jt_index) => {
                self.jts.insert(jt_index, range);
            }
            _ => { /* ignore others */ }
        }
    }
    pub fn index_get_range(&self, id: impl Into<PoolAllocatedIndex>) -> Option<&IRSourceRange> {
        let id = id.into();
        match id {
            PoolAllocatedIndex::Inst(inst_index) => self.insts.get(&inst_index),
            PoolAllocatedIndex::Block(block_index) => self.blocks.get(&block_index),
            PoolAllocatedIndex::Global(global_index) => self.globals.get(&global_index),
            PoolAllocatedIndex::Use(use_index) => self.uses.get(&use_index),
            PoolAllocatedIndex::JT(jt_index) => self.jts.get(&jt_index),
            _ => None,
        }
    }
    pub fn funcarg_insert_range(
        &mut self,
        allocs: &IRAllocs,
        arg: FuncArgID,
        range: IRSourceRange,
    ) {
        let FuncArgID(func, idx) = arg;
        let func_index = func.to_indexed(allocs);
        let args = self
            .funcargs
            .entry(func_index)
            .or_insert_with(|| vec![None; idx as usize].into_boxed_slice());
        args[idx as usize] = Some(range);
    }
    pub fn funcarg_get_range(&self, allocs: &IRAllocs, arg: FuncArgID) -> Option<IRSourceRange> {
        let FuncArgID(func, idx) = arg;
        let func_index = func.to_indexed(allocs);
        let args = self.funcargs.get(&func_index)?;
        args.get(idx as usize)?.as_ref().copied()
    }

    /// Get all source ranges associated with the given value's users.
    pub fn all_user_ranges(&self, allocs: &IRAllocs, value: ValueSSA) -> Vec<IRSourceRange> {
        let Some(traceable) = value.as_dyn_traceable(allocs) else {
            return Vec::new();
        };
        let mut ranges = Vec::new();
        for (useid, _) in traceable.user_iter(allocs) {
            let use_index = useid.to_indexed(allocs);
            if let Some(range) = self.uses.get(&use_index) {
                ranges.push(*range);
            }
        }
        ranges
    }
    /// Get all source ranges associated with the given basic block's predecessors.
    pub fn all_pred_ranges(&self, allocs: &IRAllocs, bb: BlockID) -> Vec<IRSourceRange> {
        let mut ranges = Vec::new();
        let preds = bb.get_preds(allocs);
        for (jt_id, _) in preds.iter(&allocs.jts) {
            let jt_index = jt_id.to_indexed(allocs);
            if let Some(range) = self.jts.get(&jt_index) {
                ranges.push(*range);
            }
        }
        ranges
    }
    /// Get all source ranges associated with the given basic block's successors.
    pub fn all_succ_ranges(&self, allocs: &IRAllocs, bb: BlockID) -> Vec<IRSourceRange> {
        let mut ranges = Vec::new();
        let succs = bb.get_succs(allocs);
        for jt_id in succs.iter() {
            let jt_index = jt_id.to_indexed(allocs);
            if let Some(range) = self.jts.get(&jt_index) {
                ranges.push(*range);
            }
        }
        ranges
    }

    /// Merge source ranges from another map, overwriting any existing ranges for the same entities.
    pub fn update_merge(&mut self, new_map: &Self) {
        self.insts
            .extend(new_map.insts.iter().map(|(k, v)| (*k, *v)));
        self.blocks
            .extend(new_map.blocks.iter().map(|(k, v)| (*k, *v)));
        self.globals
            .extend(new_map.globals.iter().map(|(k, v)| (*k, *v)));
        self.uses.extend(new_map.uses.iter().map(|(k, v)| (*k, *v)));
        self.jts.extend(new_map.jts.iter().map(|(k, v)| (*k, *v)));
    }
}

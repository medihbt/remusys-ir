use smol_str::format_smolstr;

use crate::{
    SymbolStr,
    ir::{
        indexed_ir::{IPoolAllocatedIndex, PoolAllocatedIndex},
        inst::*,
        module::allocs::IPoolAllocated,
        *,
    },
};
use std::collections::{HashMap, HashSet};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum LocalName {
    Str(SymbolStr),
    Cnt(usize),
}

impl<'s> From<&'s str> for LocalName {
    fn from(value: &'s str) -> Self {
        if let Ok(index) = value.parse::<usize>() {
            LocalName::Cnt(index)
        } else {
            LocalName::Str(SymbolStr::from(value))
        }
    }
}
impl From<SymbolStr> for LocalName {
    fn from(value: SymbolStr) -> Self {
        if let Ok(index) = value.parse::<usize>() {
            LocalName::Cnt(index)
        } else {
            LocalName::Str(value)
        }
    }
}
impl From<LocalName> for SymbolStr {
    fn from(val: LocalName) -> Self {
        match val {
            LocalName::Str(s) => s,
            LocalName::Cnt(i) => format_smolstr!("{i}"),
        }
    }
}
impl std::fmt::Display for LocalName {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            LocalName::Str(s) => write!(f, "%{s}"),
            LocalName::Cnt(i) => write!(f, "%{i}"),
        }
    }
}
impl LocalName {
    pub fn as_symbol_str(&self) -> SymbolStr {
        self.clone().into()
    }
}

#[derive(Debug, Clone)]
pub struct IRFuncNameInfo {
    pub index: GlobalIndex,
    pub args: Box<[Option<LocalName>]>,
}
impl IRFuncNameInfo {
    fn new(index: GlobalIndex, nargs: usize) -> Self {
        Self { index, args: vec![None; nargs].into_boxed_slice() }
    }
}

#[derive(Debug, Clone, Default)]
pub struct IRNameMap {
    pub funcs: HashMap<GlobalIndex, IRFuncNameInfo>,
    pub insts: HashMap<InstIndex, LocalName>,
    pub blocks: HashMap<BlockIndex, LocalName>,
}

impl IRNameMap {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn number_func(&mut self, allocs: &IRAllocs, func: FuncID, option: NumberOption) {
        let func_index = func.to_indexed(allocs);
        let mut number = 0;
        let func_info = self.funcs.entry(func_index).or_insert_with(|| {
            let nargs = func.deref_ir(allocs).args.len();
            IRFuncNameInfo::new(func_index, nargs)
        });

        for arg in &mut func_info.args {
            *arg = Some(Self::number_name(arg.take(), &mut number));
        }
        for (bbid, _) in func.blocks_iter(allocs) {
            let bbindex = bbid.to_indexed(allocs);
            let bbname = Self::number_name(self.blocks.remove(&bbindex), &mut number);
            self.blocks.insert(bbindex, bbname);

            for (instid, inst) in bbid.insts_iter(allocs) {
                if option.ignore_guide && matches!(inst, InstObj::PhiInstEnd(_)) {
                    continue;
                }
                if option.ignore_terminator && inst.is_terminator() {
                    continue;
                }
                if option.ignore_void && inst.get_valtype() == ValTypeID::Void {
                    continue;
                }
                let instindex = instid.to_indexed(allocs);
                let instname = Self::number_name(self.insts.remove(&instindex), &mut number);
                self.insts.insert(instindex, instname);
            }
        }
    }
    pub fn del_numbers(&mut self) {
        self.funcs.values_mut().for_each(|info| {
            for arg in &mut info.args {
                if let Some(LocalName::Cnt(_)) = arg {
                    *arg = None;
                }
            }
        });
        self.blocks
            .retain(|_, name| matches!(name, LocalName::Str(_)));
        self.insts
            .retain(|_, name| matches!(name, LocalName::Str(_)));
    }

    pub fn all_names_of_func(&mut self, allocs: &IRAllocs, func: FuncID) -> HashSet<SymbolStr> {
        let mut set = HashSet::new();
        let func_index = func.to_indexed(allocs);

        // collect argument names if present in the map
        if let Some(info) = self.funcs.get(&func_index) {
            for arg in info.args.iter().flatten() {
                set.insert(arg.as_symbol_str());
            }
        }

        // collect block and instruction names for this function
        for (bbid, _) in func.blocks_iter(allocs) {
            let bbindex = bbid.to_indexed(allocs);
            if let Some(name) = self.blocks.get(&bbindex) {
                set.insert(name.as_symbol_str());
            }

            for (instid, _) in bbid.insts_iter(allocs) {
                let instindex = instid.to_indexed(allocs);
                if let Some(name) = self.insts.get(&instindex) {
                    set.insert(name.as_symbol_str());
                }
            }
        }
        set
    }

    pub fn gc(&mut self, allocs: &IRAllocs) {
        self.funcs
            .retain(|index, _| index.as_primary(allocs).is_some());
        self.blocks
            .retain(|index, _| index.as_primary(allocs).is_some());
        self.insts
            .retain(|index, _| index.as_primary(allocs).is_some());
    }

    fn number_name(name: Option<LocalName>, number: &mut usize) -> LocalName {
        match name {
            Some(LocalName::Str(s)) => LocalName::Str(s),
            _ => {
                *number += 1;
                LocalName::Cnt(*number - 1)
            }
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
}

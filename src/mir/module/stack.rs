use crate::{
    mir::{
        module::vreg_alloc::VirtRegAlloc,
        operand::{IMirSubOperand, physreg_set::MirPhysRegSet, reg::*},
        translate::mirgen::operandgen::DispatchedReg,
    },
    typing::{context::TypeContext, id::ValTypeID, types::FloatTypeKind},
};
use std::{cell::Cell, collections::BTreeSet};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum StackItemKind {
    /// Represents a variable in the stack layout.
    Variable,
    /// Represents a saved register in the stack layout.
    SavedReg,
    /// Represents a spilled argument in the stack layout.
    SpilledArg,
}

/// Represents an item in the MIR stack layout.
/// Each item corresponds to a variable or argument in the function's stack frame.
#[derive(Debug, Clone)]
pub struct MirStackItem {
    /// The type of the item, which determines its size and alignment.
    pub irtype: ValTypeID,
    /// The index of the item in the stack layout array.
    pub index: usize,
    /// The virtual register pointer to the stack slot.
    pub stackpos_reg: GPR64,
    /// The offset of the item in its own section inside the stack.
    /// * If `is_arg` is false, this is the offset from the start of the stack frame.
    /// * If `is_arg` is true, the real offset should add the `vars_size` to this value.
    pub offset: i64,
    /// The size of the item in bytes.
    pub size: u64,
    /// The size of the item in bytes, including padding for alignment.
    pub size_with_padding: u64,
    /// The log base 2 of the alignment of the item.
    pub align_log2: u8,
    /// Whether the item is a spilled argument.
    pub kind: StackItemKind,
}

#[derive(Debug, Clone, Copy)]
pub struct SavedReg {
    pub preg: RegOperand,
}

impl SavedReg {
    pub fn new(preg: RegOperand) -> Self {
        Self { preg }
    }
    pub fn get_size_bytes(&self) -> u64 {
        let bits_log2 = self.preg.get_subreg_index().get_bits_log2();
        (1 << bits_log2) as u64 / 8
    }
    pub fn get_align_bytes(&self) -> u64 {
        let bits_log2 = self.preg.get_subreg_index().get_bits_log2();
        (1 << bits_log2) as u64 / 8
    }
    pub fn matches_saved_preg(&self, preg: RegOperand) -> bool {
        assert!(preg.is_physical());
        self.preg.get_id() == preg.get_id()
    }
}

#[derive(Debug, Clone)]
pub struct MirStackLayout {
    /// The stack layout for variables.
    pub vars: Vec<MirStackItem>,
    /// Cellee-saved registers that will be restored only at the end of the function. Positioned
    /// after the variables section in the stack frame.
    ///
    /// #### Which registers are saved?
    ///
    /// In AAPCS64 ABI, these are the registers that are callee-saved:
    ///
    /// - `x19` to `x28` (general-purpose registers, should save when regalloc pass allocates them)
    /// - `x29` (frame pointer, should save in `main` function or function has instruction translated
    ///    from IR `DynAlloca` instruction.
    /// - `x30` (link register, should save when the function is not a leaf function)
    /// - `d8` to `d15` (floating-point registers, should save when regalloc pass allocates them)
    ///
    /// In register allocation, these registers will be marked as "Tier 2" -- Usually costs
    /// more to spill and restore than "Tier 1" registers.
    ///
    /// NOTE that not all saved registers are in this list, since some instructions like
    /// `call (MIR pesudo op)` will also save some registers to the stack temporarily.
    ///
    /// #### Where are saved registers positioned?
    ///
    /// Remusys-MIR has no opreand kind representing "stack position", while it uses virtual registers.
    /// Maybe we'll offer a function to help users know whether a virtual register is representing
    /// a stack position or not.
    pub saved_regs: Vec<SavedReg>,
    /// The stack layout for spilled arguments.
    pub args: Vec<MirStackItem>,
    /// The total size of the variables section in the stack frame.
    pub vars_size: u64,
    /// The total size of the arguments section in the stack frame.
    pub args_size: u64,

    finished_arg_build: bool,
    _saved_regs_size_cache: Cell<u64>,
}

impl Default for MirStackLayout {
    fn default() -> Self {
        Self::new()
    }
}

impl MirStackLayout {
    pub fn new() -> Self {
        Self {
            vars: Vec::new(),
            args: Vec::new(),
            saved_regs: Vec::new(),
            vars_size: 0,
            args_size: 0,
            finished_arg_build: false,
            _saved_regs_size_cache: Cell::new(0),
        }
    }
    pub fn init_saved_regs_as_aapcs_callee(&mut self) -> usize {
        self.reinit_saved_regs(MirPhysRegSet::new_aapcs_callee())
    }
    pub fn reinit_saved_regs(&mut self, saved_regs: MirPhysRegSet) -> usize {
        self.saved_regs.clear();
        self._saved_regs_size_cache.set(0);
        self.saved_regs.reserve(saved_regs.num_regs() as usize);
        for preg in saved_regs {
            self.saved_regs.push(SavedReg::new(preg));
        }
        self._saved_regs_size_cache.set(0);
        self.saved_regs.len()
    }
    pub fn saved_regs_section_size(&self) -> u64 {
        if self._saved_regs_size_cache.get() != 0 {
            let size = self._saved_regs_size_cache.get();
            if size % 16 != 0 {
                panic!("Saved registers section size must be a multiple of 16, found: {size}");
            }
            return size;
        }
        let mut size: u64 = 0;
        for reg in &self.saved_regs {
            size = size.next_multiple_of(reg.get_align_bytes());
            size += reg.get_size_bytes();
        }
        size = size.next_multiple_of(16); // Ensure 16-byte alignment
        self._saved_regs_size_cache.set(size);
        size
    }
    pub fn foreach_saved_regs(&self, mut read_reg_and_offset: impl FnMut(&SavedReg, u64)) {
        let mut offset = 0;
        for reg in &self.saved_regs {
            read_reg_and_offset(reg, offset);
            offset = offset.next_multiple_of(reg.get_align_bytes());
            offset += reg.get_size_bytes();
        }
    }

    pub fn find_saved_preg(&self, preg: GPR64) -> Option<&SavedReg> {
        assert!(preg.is_physical());
        self.saved_regs
            .iter()
            .find(|&reg| reg.matches_saved_preg(preg.into()))
    }

    pub fn find_vreg_spilled_arg_pos(&self, vreg: GPR64) -> Option<u32> {
        for i in &self.args {
            if i.stackpos_reg.same_pos_as(vreg) {
                return Some(i.index as u32);
            }
        }
        None
    }
    pub fn find_vreg_variable_pos(&self, vreg: GPR64) -> Option<u32> {
        if vreg.is_physical() {
            return None;
        }
        for pos in &self.vars {
            if pos.stackpos_reg.same_pos_as(vreg) {
                return Some(pos.index as u32);
            }
        }
        None
    }

    pub fn find_vreg_stackpos(&self, vreg: GPR64) -> Option<(StackItemKind, usize)> {
        if let Some(x) = self.find_vreg_spilled_arg_pos(vreg) {
            return Some((StackItemKind::SpilledArg, x as usize));
        }
        if let Some(x) = self.find_vreg_variable_pos(vreg) {
            return Some((StackItemKind::Variable, x as usize));
        }
        None
    }
    pub fn vreg_is_stackpos(&self, vreg: GPR64) -> bool {
        self.find_vreg_stackpos(vreg).is_some()
    }

    /// Adds a spilled argument to the stack layout.
    pub(super) fn add_spilled_arg(
        &mut self,
        irtype: ValTypeID,
        vreg_alloc: &mut VirtRegAlloc,
    ) -> &mut MirStackItem {
        assert!(
            self.finished_arg_build == false,
            "Cannot add more args after building the stack layout"
        );
        let (natural_size, align_log2) = match irtype {
            ValTypeID::Float(FloatTypeKind::Ieee32) => (4, 2),
            ValTypeID::Float(FloatTypeKind::Ieee64) => (8, 3),
            ValTypeID::Int(32) => (4, 2),
            ValTypeID::Int(64) | ValTypeID::Ptr => (8, 3),
            _ => panic!("Unsupported type for spilled argument: {irtype:?}"),
        };
        let size = natural_size.max(8);
        // Ensure at least 8-byte alignment
        let align_log2 = align_log2.max(3);
        let new_top = self.args_size.next_multiple_of(1u64 << align_log2);
        let item = MirStackItem {
            irtype,
            index: self.args.len(),
            stackpos_reg: GPR64::from_real(vreg_alloc.insert_gp(GPR64::new_empty().into_real())),
            offset: new_top as i64,
            size,
            size_with_padding: 0,
            align_log2,
            kind: StackItemKind::SpilledArg,
        };
        self.args_size = new_top + size;
        self.args.push(item);
        self.args.last_mut().unwrap()
    }

    pub(super) fn finish_arg_building(&mut self) {
        if self.finished_arg_build {
            return;
        }
        self.finished_arg_build = true;

        // Align the args size to 16.
        self.args_size = self.args_size.next_multiple_of(16);
        // Calculate the size with padding for each argument
        let nargs = self.args.len();
        if nargs == 0 {
            return;
        }
        let args = self.args.as_mut_slice();
        for i in 1..nargs {
            let curr_offset = args[i - 1].offset;
            let next_offset = args[i].offset;
            args[i - 1].size_with_padding = (next_offset - curr_offset) as u64;
        }
        // The last argument's size is the remaining space in the stack frame
        if let Some(last_arg) = self.args.last_mut() {
            last_arg.size_with_padding = self.args_size - last_arg.offset as u64;
        }
    }

    pub fn add_variable(
        &mut self,
        irtype: ValTypeID,
        type_ctx: &TypeContext,
        vreg_alloc: &mut VirtRegAlloc,
    ) -> &mut MirStackItem {
        let size = irtype.get_instance_size(type_ctx);
        let align = irtype.get_instance_align(type_ctx);

        let (size, align_log2) = match (size, align) {
            (Some(size), Some(align)) if align.is_power_of_two() => {
                (size, align.trailing_zeros() as u8)
            }
            _ => panic!(
                "Invalid size or alignment for type `{irtype:?}`: size={size:?}, align={align:?}",
            ),
        };
        self.add_variable_item(irtype, vreg_alloc, size, align_log2)
    }

    fn add_variable_item(
        &mut self,
        irtype: ValTypeID,
        vreg_alloc: &mut VirtRegAlloc,
        size: usize,
        align_log2: u8,
    ) -> &mut MirStackItem {
        let new_top = self.vars_size.next_multiple_of(1u64 << align_log2);
        if let Some(prev_top) = self.vars.last_mut() {
            // Update the previous top item to reflect the new size
            // This is necessary to ensure that the size_with_padding is correct
            // for the previous item after adding a new item.
            let prev_offset = prev_top.offset;
            prev_top.size_with_padding = (new_top as i64 - prev_offset) as u64;
        }
        let item = MirStackItem {
            irtype,
            index: self.vars.len(),
            stackpos_reg: vreg_alloc.insert_gpr64(GPR64::new_empty()),
            offset: new_top as i64,
            size: size as u64,
            size_with_padding: size as u64, // Placeholder, will be updated later
            align_log2,
            kind: StackItemKind::Variable,
        };
        self.vars_size = new_top + size as u64;
        self.vars.push(item);
        self.vars.last_mut().unwrap()
    }

    pub fn add_spilled_virtreg_variable(
        &mut self,
        vreg: RegOperand,
        vreg_alloc: &mut VirtRegAlloc,
    ) -> &mut MirStackItem {
        assert!(
            vreg.is_virtual(),
            "Expected a virtual register, found ID {:?}",
            vreg.get_id()
        );
        let pure = DispatchedReg::from_reg(vreg);
        let (irtype, size, align_log2) = match pure {
            DispatchedReg::F32(_) => (ValTypeID::Float(FloatTypeKind::Ieee32), 4, 2),
            DispatchedReg::F64(_) => (ValTypeID::Float(FloatTypeKind::Ieee64), 8, 3),
            DispatchedReg::G32(_) => (ValTypeID::Int(32), 4, 2),
            DispatchedReg::G64(_) => (ValTypeID::Int(64), 8, 3),
        };
        self.add_variable_item(irtype, vreg_alloc, size, align_log2)
    }

    /// 重排变量区域的内存, 使其紧凑一些.
    pub fn rearrange_var_section(&mut self, valid_stackpos: &BTreeSet<RegID>) {
        if self.vars.is_empty() {
            return;
        }
        self.vars.retain(|item| {
            if item.stackpos_reg.is_virtual() {
                valid_stackpos.contains(&item.stackpos_reg.get_id())
            } else {
                panic!("Expected a virtual register, found item {item:#?}");
            }
        });
        self.vars.sort_by(|a, b| {
            a.align_log2
                .cmp(&b.align_log2)
                .then(a.size.cmp(&b.size))
                .then(a.offset.cmp(&b.offset))
        });
        let mut offset: u64 = 0;
        for (index, item) in self.vars.iter_mut().enumerate() {
            offset = offset.next_multiple_of(1u64 << item.align_log2);
            item.offset = offset as i64;
            item.size_with_padding = item.size;
            item.index = index;
            offset += item.size;
        }
        self.vars_size = offset.next_multiple_of(16);
    }
}

use std::cell::Cell;

use slab::Slab;

use crate::{
    mir::operand::{
        IMirSubOperand,
        reg::{FPR64, GPR64, GPReg, RegID, RegOperand, VFReg},
    },
    typing::{context::TypeContext, id::ValTypeID, types::FloatTypeKind},
};

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
    pub virtreg: RegOperand,
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

impl MirStackItem {
    // pub fn offset_from_sp(&self, layout: &MirStackLayout) -> i64 {
    //     match self.kind {
    //         StackItemKind::Variable => self.offset,
    //         StackItemKind::SavedReg => self.offset + layout.vars_size as i64,
    //         StackItemKind::SpilledArg => {
    //             // Spilled arguments are offset from the end of the variable section
    //             self.offset + (layout.vars_size + layout.saved_regs_size) as i64
    //         }
    //     }
    // }
}

#[derive(Debug, Clone)]
pub struct MirStackLayout {
    /// The stack layout for variables.
    pub vars: Vec<MirStackItem>,
    /// Cellee-saved registers that will be restored only at the end of the function. Positioned
    /// after the variables section in the stack frame.
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
    pub saved_regs: Vec<RegOperand>,
    /// The stack layout for spilled arguments.
    pub args: Vec<MirStackItem>,
    /// The total size of the variables section in the stack frame.
    pub vars_size: u64,
    /// The total size of the arguments section in the stack frame.
    pub args_size: u64,

    finished_arg_build: bool,
    _saved_regs_size_cache: Cell<u64>,
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

    /// Updates the stack top size based on the current size and alignment.
    fn update_stack_top(curr_size: u64, align_log2: u8) -> u64 {
        let next_align = 1u64 << align_log2;
        let pmask = next_align - 1;
        let nmask = !pmask;
        let new_size_base = curr_size & nmask;
        if curr_size & pmask != 0 {
            new_size_base + next_align
        } else {
            new_size_base
        }
    }

    /// Adds a spilled argument to the stack layout.
    pub fn add_spilled_arg(
        &mut self,
        irtype: ValTypeID,
        vreg_alloc: &mut VirtRegAlloc,
    ) -> &mut MirStackItem {
        assert!(
            self.finished_arg_build == false,
            "Cannot add more args after building the stack layout"
        );
        let (natural_size, natural_align_log2) = match irtype {
            ValTypeID::Ptr => (8, 3),
            ValTypeID::Int(bits) => match bits {
                8 => (1, 0),
                16 => (2, 1),
                32 => (4, 2),
                64 => (8, 3),
                _ => panic!("Unsupported integer size: {}", bits),
            },
            ValTypeID::Float(fp_kind) => match fp_kind {
                FloatTypeKind::Ieee32 => (4, 2),
                FloatTypeKind::Ieee64 => (8, 3),
            },
            _ => panic!("Requires ptr/int/float as args but got `{irtype:?}`"),
        };

        let size = natural_size.max(8);
        let align_log2 = natural_align_log2.max(3);
        let new_top = Self::update_stack_top(self.args_size, align_log2);
        let item = MirStackItem {
            irtype,
            index: self.args.len(),
            /* this `virtreg` is a pointer to the stack slot, not a register */
            virtreg: vreg_alloc.alloc(false),
            offset: new_top as i64,
            size: size as u64,
            // Placeholder, will be updated on `finish_arg_building()`
            size_with_padding: 0,
            align_log2: align_log2 as u8,
            kind: StackItemKind::SpilledArg,
        };
        self.args_size = new_top + size as u64;
        self.args.push(item);
        self.args.last_mut().unwrap()
    }

    pub fn finish_arg_building(&mut self) {
        if self.finished_arg_build {
            return;
        }
        self.finished_arg_build = true;

        // Align the args size to 16.
        self.args_size = Self::update_stack_top(self.args_size, 4);
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
        let new_top = Self::update_stack_top(self.vars_size, align_log2);
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
            virtreg: vreg_alloc.alloc(false), // This is a pointer to the stack slot, not a register
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
}

#[derive(Debug, Clone)]
pub struct VirtRegAlloc {
    pub general: Slab<GPReg>,
    pub float: Slab<VFReg>,
}

impl VirtRegAlloc {
    pub fn new() -> Self {
        Self {
            general: Slab::new(),
            float: Slab::new(),
        }
    }

    pub fn insert_gp_for_index(&mut self, vreg: GPReg) -> u32 {
        let index = self.general.vacant_key() as u32;
        let vreg = vreg.insert_id(RegID::Virt(index));
        self.general.insert(vreg);
        index
    }
    pub fn insert_float_for_index(&mut self, vreg: VFReg) -> u32 {
        let index = self.float.vacant_key() as u32;
        let vreg = vreg.insert_id(RegID::Virt(index));
        self.float.insert(vreg);
        index
    }
    pub fn insert_gp(&mut self, vreg: GPReg) -> GPReg {
        let index = self.insert_gp_for_index(vreg.into_real());
        self.general[index as usize]
    }
    pub fn insert_float(&mut self, vreg: VFReg) -> VFReg {
        let index = self.insert_float_for_index(vreg.into_real());
        self.float[index as usize]
    }

    pub fn alloc_gp(&mut self) -> &mut GPReg {
        let index = self.insert_gp_for_index(GPR64::new_empty().into_real());
        &mut self.general[index as usize]
    }
    pub fn alloc_float(&mut self) -> &mut VFReg {
        let index = self.insert_float_for_index(FPR64::new_empty().into_real());
        &mut self.float[index as usize]
    }
    pub fn alloc(&mut self, is_float: bool) -> RegOperand {
        if is_float {
            RegOperand::from(*self.alloc_float())
        } else {
            RegOperand::from(*self.alloc_gp())
        }
    }

    pub fn dealloc_gp(&mut self, vreg: GPReg) -> bool {
        let id = match vreg.get_id() {
            RegID::Virt(id) => id,
            _ => panic!("Expected a virtual GP register, found {:?}", vreg.get_id()),
        };
        self.general.try_remove(id as usize).is_some()
    }
    pub fn dealloc_float(&mut self, vreg: VFReg) -> bool {
        let id = match vreg.get_id() {
            RegID::Virt(id) => id,
            _ => panic!("Expected a virtual VF register, found {:?}", vreg.get_id()),
        };
        self.float.try_remove(id as usize).is_some()
    }

    pub fn dealloc(&mut self, vreg: RegOperand) -> bool {
        let RegOperand(id, si, uf, is_fp) = vreg;
        if is_fp {
            let vfreg = VFReg(id, si, uf);
            self.dealloc_float(vfreg)
        } else {
            let gpreg = GPReg(id, si, uf);
            self.dealloc_gp(gpreg)
        }
    }
}

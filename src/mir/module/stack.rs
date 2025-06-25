use slab::Slab;

use crate::{
    mir::operand::reg::VirtReg,
    typing::{context::TypeContext, id::ValTypeID, types::FloatTypeKind},
};

/// Represents an item in the MIR stack layout.
/// Each item corresponds to a variable or argument in the function's stack frame.
#[derive(Debug, Clone)]
pub struct MirStackItem {
    /// The type of the item, which determines its size and alignment.
    pub irtype: ValTypeID,
    /// The index of the item in the stack layout array.
    pub index: usize,
    /// The virtual register pointer to the stack slot.
    pub virtreg: VirtReg,
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
    pub is_arg: bool,
}

impl MirStackItem {
    pub fn offset_from_sp(&self, layout: &MirStackLayout) -> i64 {
        if self.is_arg {
            self.offset + layout.vars_size as i64
        } else {
            self.offset
        }
    }
}

#[derive(Debug, Clone)]
pub struct MirStackLayout {
    /// The stack layout for variables.
    pub vars: Vec<MirStackItem>,
    /// The stack layout for spilled arguments.
    pub args: Vec<MirStackItem>,
    /// The total size of the variables section in the stack frame.
    pub vars_size: u64,
    /// The total size of the arguments section in the stack frame.
    pub args_size: u64,
    finished_arg_build: bool,
}

impl MirStackLayout {
    pub fn new() -> Self {
        Self {
            vars: Vec::new(),
            args: Vec::new(),
            vars_size: 0,
            args_size: 0,
            finished_arg_build: false,
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
            virtreg: *vreg_alloc.alloc(false),
            offset: new_top as i64,
            size: size as u64,
            // Placeholder, will be updated on `finish_arg_building()`
            size_with_padding: 0,
            align_log2: align_log2 as u8,
            is_arg: true,
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
            virtreg: *vreg_alloc.alloc(matches!(irtype, ValTypeID::Float(_))),
            offset: new_top as i64,
            size: size as u64,
            size_with_padding: size as u64, // Placeholder, will be updated later
            align_log2,
            is_arg: false,
        };
        self.vars_size = new_top + size as u64;
        self.vars.push(item);
        self.vars.last_mut().unwrap()
    }
}

#[derive(Debug, Clone)]
pub struct VirtRegAlloc {
    pub general: Slab<VirtReg>,
    pub float: Slab<VirtReg>,
}

impl VirtRegAlloc {
    pub fn new() -> Self {
        Self {
            general: Slab::new(),
            float: Slab::new(),
        }
    }

    fn do_alloc(slab: &mut Slab<VirtReg>, mapper: impl Fn(u32) -> VirtReg) -> u32 {
        let index = slab.vacant_key() as u32;
        slab.insert(mapper(index));
        index
    }
    pub fn alloc_gp(&mut self) -> &mut VirtReg {
        let index = Self::do_alloc(&mut self.general, VirtReg::new_long);
        &mut self.general[index as usize]
    }
    pub fn alloc_float(&mut self) -> &mut VirtReg {
        let index = Self::do_alloc(&mut self.float, VirtReg::new_float);
        &mut self.float[index as usize]
    }
    pub fn alloc(&mut self, is_float: bool) -> &mut VirtReg {
        if is_float {
            self.alloc_float()
        } else {
            self.alloc_gp()
        }
    }
    pub fn dealloc(&mut self, vreg: VirtReg) -> bool {
        let (id, slab) = match vreg {
            VirtReg::General(id, ..) => (id, &mut self.general),
            VirtReg::Float(id, ..) => (id, &mut self.float),
        };
        slab.try_remove(id as usize).is_some()
    }
}

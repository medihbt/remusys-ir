use slab::Slab;

use crate::{
    mir::operand::reg::VirtReg,
    typing::{context::TypeContext, id::ValTypeID, types::FloatTypeKind},
};

#[derive(Debug, Clone)]
pub struct MirStackItem {
    pub vreg:   VirtReg,
    pub irtype: ValTypeID,
    /// offset from `SP` in bytes.
    pub offset: i64,
    pub size: u64,
    pub align_log2: u8,
    pub is_arg: bool,
}

#[derive(Debug, Clone)]
pub struct MirStackLayout {
    pub variables: Vec<MirStackItem>,
    pub args: Vec<MirStackItem>,
    pub var_stack_size: u64,
    pub args_stack_size: u64,
    var_align_log2: u8,
    arg_align_log2: u8,
}

impl MirStackLayout {
    pub fn new() -> Self {
        Self {
            variables: Vec::new(),
            args: Vec::new(),
            var_stack_size: 0,
            args_stack_size: 0,
            var_align_log2: 0,
            arg_align_log2: 0,
        }
    }

    fn update_stack_top_aligned(
        curr_size: u64,
        curr_align_log2: u8,
        new_align_log2: u8,
    ) -> (u64, u8) {
        let next_align_log2 = curr_align_log2.max(new_align_log2);
        let next_align = 1u64 << next_align_log2;
        let pmask = next_align - 1;
        let nmask = !pmask;
        let new_size_base = curr_size & nmask;
        let new_size = if curr_size & pmask != 0 {
            new_size_base + next_align
        } else {
            new_size_base
        };
        (new_size, next_align_log2)
    }
    pub(super) fn update_arg_stack_top(&mut self, new_align_log2: u8, strict_aligned: bool) {
        let curr_align_log2 = if strict_aligned {
            self.var_align_log2
        } else {
            0
        };
        let (new_size, _) =
            Self::update_stack_top_aligned(self.args_stack_size, curr_align_log2, new_align_log2);
        self.args_stack_size = new_size;
    }
    fn update_var_stack_top(&mut self, new_align_log2: u8, strict_aligned: bool) {
        let curr_align_log2 = if strict_aligned {
            self.arg_align_log2
        } else {
            0
        };
        let (new_size, _) =
            Self::update_stack_top_aligned(self.var_stack_size, curr_align_log2, new_align_log2);
        self.var_stack_size = new_size;
    }

    pub(super) fn push_arg(
        &mut self,
        vreg_alloc: &mut VirtRegAlloc,
        // passes primitive types and pointer only.
        arg_ty: ValTypeID,
    ) -> VirtReg {
        let (is_float, size, align_log2) = match arg_ty {
            ValTypeID::Ptr => (false, 8, 3),
            ValTypeID::Int(bits) => (false, Self::round_to(bits), Self::round_to(bits)),
            ValTypeID::Float(fpkind) => match fpkind {
                FloatTypeKind::Ieee32 => (true, 4, 2),
                FloatTypeKind::Ieee64 => (true, 8, 3),
            },
            _ => panic!("Invalid argument type for stack layout: {arg_ty:?}"),
        };
        let vreg = vreg_alloc.alloc(is_float);
        vreg.subreg_index_mut().set_bits_log2(align_log2);
        self.update_arg_stack_top(align_log2, false);
        let arg_item = MirStackItem {
            vreg: vreg.clone(),
            irtype: arg_ty,
            offset: self.args_stack_size as i64,
            size: size as u64,
            align_log2,
            is_arg: true,
        };
        self.args.push(arg_item);
        self.args_stack_size += size as u64;
        vreg.clone()
    }

    /// Pushes a variable onto the stack and returns a pointer to the allocated virtual register.
    pub(super) fn push_spilled_variable(
        &mut self,
        vreg_alloc: &mut VirtRegAlloc,
        irtype: ValTypeID,
        type_ctx: &TypeContext,
    ) -> VirtReg {
        let size = irtype.get_instance_size(type_ctx);
        let align = irtype.get_instance_align(type_ctx);
        let (size_bytes, align_log2) = match (size, align) {
            (Some(s), Some(a)) => (s, a.trailing_zeros() as u8),
            _ => panic!("Invalid type for variable allocation: {irtype:?}"),
        };
        // returns pointer to the allocated virtual register.
        let vreg = vreg_alloc.alloc_gp();
        // aarch64 pointers are 64 bits (1 << 6).
        vreg.subreg_index_mut().insert_bits_log2(6);
        let vreg = vreg.clone();
        self.update_var_stack_top(align_log2, false);
        let var_item = MirStackItem {
            vreg,
            irtype,
            offset: self.var_stack_size as i64,
            size: size_bytes as u64,
            align_log2,
            is_arg: false,
        };
        self.variables.push(var_item);
        self.var_stack_size += size_bytes as u64;
        vreg
    }

    fn round_to(bits: u8) -> u8 {
        if bits == 0 {
            0
        } else {
            (bits - 1).next_power_of_two().trailing_zeros() as u8
        }
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

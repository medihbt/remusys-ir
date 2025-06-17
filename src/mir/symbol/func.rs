use crate::{
    base::slablist::SlabRefList,
    mir::{operand::virtreg::VirtReg, symbol::block::MachineBlockRef},
    typing::{context::TypeContext, id::ValTypeID},
};

pub struct MachineFunc {
    pub name: String,
    pub body: SlabRefList<MachineBlockRef>,
    pub virt_reg_alloc: VirtRegAlloc,
    pub stack_size: usize,
    pub stack_align: usize,
    pub spill_stack: Vec<VariableInfo>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VirtRegAlloc {
    pub general_max: u32,
    pub float_max: u32,
}

pub struct VariableInfo {
    pub virtreg: VirtReg,
    pub irtype: ValTypeID,
    pub stack_offset: usize,
    pub size_bytes: usize,
    pub align_bytes: usize,
}

impl MachineFunc {
    pub fn is_extern(&self) -> bool {
        self.body.is_empty()
    }

    pub fn push_spilled(&mut self, val_type: ValTypeID, type_ctx: &TypeContext) -> usize {
        let size_bytes = match val_type.get_instance_size(type_ctx) {
            Some(size) => size,
            None => panic!("Cannot allocate memory for type: {:?}", val_type),
        };
        let align_bytes = match val_type.get_instance_align(type_ctx) {
            Some(align) => align,
            None => panic!("Cannot allocate memory for type: {:?}", val_type),
        };
        let ptr_reg_id = self.virt_reg_alloc.general_max;
        self.virt_reg_alloc.general_max += 1;

        let virtreg = VirtReg::new_long(ptr_reg_id);

        let (stack_offset, new_stack_top) =
            Self::update_stack_top(self.stack_size, size_bytes, align_bytes);
        self.stack_size = new_stack_top;
        let spill_stack_index = self.spill_stack.len();
        self.spill_stack.push(VariableInfo {
            virtreg,
            irtype: val_type,
            stack_offset,
            size_bytes,
            align_bytes,
        });
        spill_stack_index
    }

    fn update_stack_top(top: usize, size_bytes: usize, align_bytes: usize) -> (usize, usize) {
        // align the current size to the alignment
        let alloc_pos = (top + align_bytes - 1) / align_bytes * align_bytes;
        let new_top = alloc_pos + size_bytes;
        (alloc_pos, new_top)
    }
}

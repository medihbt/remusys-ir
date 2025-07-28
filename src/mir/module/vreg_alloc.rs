use crate::mir::operand::{IMirSubOperand, reg::*};
use slab::Slab;

#[derive(Debug, Clone)]
pub struct VirtRegAlloc {
    pub general: Slab<GPReg>,
    pub float: Slab<VFReg>,
}

impl VirtRegAlloc {
    pub fn new() -> Self {
        Self { general: Slab::new(), float: Slab::new() }
    }

    pub fn insert_gp_for_index(&mut self, greg: GPReg) -> u32 {
        let index = self.general.vacant_key() as u32;
        let vreg = greg.insert_id(RegID::Virt(index));
        self.general.insert(vreg);
        index
    }
    pub fn insert_float_for_index(&mut self, vreg: VFReg) -> u32 {
        let index = self.float.vacant_key() as u32;
        let vreg = vreg.insert_id(RegID::Virt(index));
        self.float.insert(vreg);
        index
    }
    pub fn insert_reg_for_index(&mut self, reg: RegOperand) -> (bool, u32) {
        let RegOperand(_, si, uf, is_fp) = reg;
        let index = if is_fp {
            self.insert_float_for_index(VFReg(0, si, uf))
        } else {
            self.insert_gp_for_index(GPReg(0, si, uf))
        };
        (is_fp, index)
    }
    pub fn insert_gp(&mut self, vreg: GPReg) -> GPReg {
        let index = self.insert_gp_for_index(vreg.into_real());
        self.general[index as usize]
    }
    pub fn insert_gpr64(&mut self, vreg: GPR64) -> GPR64 {
        let index = self.insert_gp_for_index(vreg.into_real());
        GPR64::from_real(self.general[index as usize])
    }
    pub fn insert_gpr32(&mut self, vreg: GPR32) -> GPR32 {
        let index = self.insert_gp_for_index(vreg.into_real());
        GPR32::from_real(self.general[index as usize])
    }
    pub fn insert_float(&mut self, vreg: VFReg) -> VFReg {
        let index = self.insert_float_for_index(vreg.into_real());
        self.float[index as usize]
    }
    pub fn insert_fpr64(&mut self, vreg: FPR64) -> FPR64 {
        let index = self.insert_float_for_index(vreg.into_real());
        FPR64::from_real(self.float[index as usize])
    }
    pub fn insert_fpr32(&mut self, vreg: FPR32) -> FPR32 {
        let index = self.insert_float_for_index(vreg.into_real());
        FPR32::from_real(self.float[index as usize])
    }
    pub fn insert_reg(&mut self, reg: RegOperand) -> RegOperand {
        let (is_fp, index) = self.insert_reg_for_index(reg);
        if is_fp { self.float[index as usize].into() } else { self.general[index as usize].into() }
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

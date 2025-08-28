use crate::mir::operand::{IMirSubOperand, reg::*};
use slab::Slab;

#[derive(Debug, Clone)]
pub struct VirtRegAlloc {
    pub general: Slab<(GPReg, bool)>,
    pub float: Slab<(VFReg, bool)>,
    pub stackpos: Slab<GPR64>,
    free_g32: Vec<GPR32>,
    free_g64: Vec<GPR64>,
    free_f32: Vec<FPR32>,
    free_f64: Vec<FPR64>,
}

enum FreePool<'a> {
    G32(&'a mut Vec<GPR32>),
    G64(&'a mut Vec<GPR64>),
    F32(&'a mut Vec<FPR32>),
    F64(&'a mut Vec<FPR64>),
}

impl VirtRegAlloc {
    pub fn new() -> Self {
        Self {
            general: Slab::new(),
            stackpos: Slab::new(),
            float: Slab::new(),
            free_g32: Vec::new(),
            free_g64: Vec::new(),
            free_f32: Vec::new(),
            free_f64: Vec::new(),
        }
    }

    /// ### 参数
    ///
    /// - `greg`: 样板寄存器, 提供 si 和 uf, 不提供 id.
    pub fn insert_gp_for_index(&mut self, greg: GPReg) -> u32 {
        let GPReg(_, si, uf) = greg;
        match self.try_cached_free_gp(si.get_bits_log2()) {
            Some(FreePool::G32(pool)) => {
                let mut gpr32 = pool.pop().unwrap();
                gpr32.1 = uf;
                let RegID::Virt(index) = gpr32.get_id() else {
                    panic!("Expected a virtual GPR32, found {:?}", gpr32.get_id());
                };
                let (slot, is_valid) = self.general.get_mut(index as usize).unwrap();
                *is_valid = true;
                *slot = gpr32.into_real();
                index
            }
            Some(FreePool::G64(pool)) => {
                let mut gpr64 = pool.pop().unwrap();
                gpr64.1 = uf;
                let RegID::Virt(index) = gpr64.get_id() else {
                    panic!("Expected a virtual GPR64, found {:?}", gpr64.get_id());
                };
                let (slot, is_valid) = self.general.get_mut(index as usize).unwrap();
                *is_valid = true;
                *slot = gpr64.into_real();
                index
            }
            None => {
                // 原有逻辑
                let index = self.general.vacant_key() as u32;
                let vreg = greg.insert_id(RegID::Virt(index));
                self.general.insert((vreg, true));
                index
            }
            _ => unreachable!(),
        }
    }
    /// ### 参数
    ///
    /// - `vreg`: 样板寄存器, 提供 si 和 uf, 不提供 id.
    pub fn insert_float_for_index(&mut self, vreg: VFReg) -> u32 {
        let VFReg(_, si, uf) = vreg;
        match self.try_cached_free_fp(si.get_bits_log2()) {
            Some(FreePool::F32(pool)) => {
                let mut fpr32 = pool.pop().unwrap();
                fpr32.1 = uf;
                let RegID::Virt(index) = fpr32.get_id() else {
                    panic!("Expected a virtual FPR32, found {:?}", fpr32.get_id());
                };
                let (slot, is_valid) = self.float.get_mut(index as usize).unwrap();
                *is_valid = true;
                *slot = fpr32.into_real();
                index
            }
            Some(FreePool::F64(pool)) => {
                let mut fpr64 = pool.pop().unwrap();
                fpr64.1 = uf;
                let RegID::Virt(index) = fpr64.get_id() else {
                    panic!("Expected a virtual FPR64, found {:?}", fpr64.get_id());
                };
                let (slot, is_valid) = self.float.get_mut(index as usize).unwrap();
                *is_valid = true;
                *slot = fpr64.into_real();
                index
            }
            None => {
                // 原有逻辑
                let index = self.float.vacant_key() as u32;
                let vreg = vreg.insert_id(RegID::Virt(index));
                self.float.insert((vreg, true));
                index
            }
            _ => unreachable!(),
        }
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
        self.general[index as usize].0
    }
    pub fn insert_gpr64(&mut self, vreg: GPR64) -> GPR64 {
        let index = self.insert_gp_for_index(vreg.into_real());
        GPR64::from_real(self.general[index as usize].0)
    }
    pub fn insert_gpr32(&mut self, vreg: GPR32) -> GPR32 {
        let index = self.insert_gp_for_index(vreg.into_real());
        GPR32::from_real(self.general[index as usize].0)
    }
    pub fn alloc_stackpos(&mut self) -> GPR64 {
        let index = self.stackpos.vacant_key() as u32;
        let vreg = GPR64::new_empty().insert_id(RegID::StackPos(index));
        self.stackpos.insert(vreg);
        vreg
    }
    pub fn insert_float(&mut self, vreg: VFReg) -> VFReg {
        let index = self.insert_float_for_index(vreg.into_real());
        self.float[index as usize].0
    }
    pub fn insert_fpr64(&mut self, vreg: FPR64) -> FPR64 {
        let index = self.insert_float_for_index(vreg.into_real());
        FPR64::from_real(self.float[index as usize].0)
    }
    pub fn insert_fpr32(&mut self, vreg: FPR32) -> FPR32 {
        let index = self.insert_float_for_index(vreg.into_real());
        FPR32::from_real(self.float[index as usize].0)
    }
    pub fn insert_reg(&mut self, reg: RegOperand) -> RegOperand {
        let (is_fp, index) = self.insert_reg_for_index(reg);
        if is_fp {
            self.float[index as usize].0.into()
        } else {
            self.general[index as usize].0.into()
        }
    }

    pub fn alloc_gp(&mut self) -> &mut GPReg {
        let index = self.insert_gp_for_index(GPR64::new_empty().into_real());
        &mut self.general[index as usize].0
    }
    pub fn alloc_float(&mut self) -> &mut VFReg {
        let index = self.insert_float_for_index(FPR64::new_empty().into_real());
        &mut self.float[index as usize].0
    }
    pub fn alloc(&mut self, is_float: bool) -> RegOperand {
        if is_float {
            RegOperand::from(*self.alloc_float())
        } else {
            RegOperand::from(*self.alloc_gp())
        }
    }

    pub fn dealloc_gp(&mut self, vreg: GPReg) -> bool {
        match vreg.get_id() {
            RegID::StackPos(id) => self.stackpos.try_remove(id as usize).is_some(),
            RegID::Virt(id) => {
                if !self.general.contains(id as usize) {
                    return false;
                }
                let (slotreg, valid) = &mut self.general[id as usize];
                if !*valid {
                    return false;
                }
                *valid = false;
                match slotreg.get_bits_log2() {
                    5 => self.free_g32.push(GPR32::from_real(*slotreg)),
                    6 => self.free_g64.push(GPR64::from_real(*slotreg)),
                    _ => panic!(
                        "Unsupported GP register size log2: {}",
                        slotreg.get_bits_log2()
                    ),
                }
                true
            }
            _ => panic!("Expected a virtual GP register, found {:?}", vreg.get_id()),
        }
    }
    pub fn dealloc_stackpos(&mut self, vreg: GPR64) -> bool {
        let id = match vreg.get_id() {
            RegID::StackPos(id) => id,
            _ => panic!(
                "Expected a stack position register, found {:?}",
                vreg.get_id()
            ),
        };
        self.stackpos.try_remove(id as usize).is_some()
    }
    pub fn dealloc_float(&mut self, vreg: VFReg) -> bool {
        let id = match vreg.get_id() {
            RegID::Virt(id) => id,
            _ => panic!("Expected a virtual VF register, found {:?}", vreg.get_id()),
        };
        let Some((slotreg, valid)) = self.float.get_mut(id as usize) else {
            return false;
        };
        if !*valid {
            return false;
        }
        *valid = false;
        match slotreg.get_bits_log2() {
            5 => self.free_f32.push(FPR32::from_real(*slotreg)),
            6 => self.free_f64.push(FPR64::from_real(*slotreg)),
            _ => panic!(
                "Unsupported VF register size log2: {}",
                slotreg.get_bits_log2()
            ),
        }
        true
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

    fn try_cached_free_gp(&mut self, bits_log2: u8) -> Option<FreePool<'_>> {
        match bits_log2 {
            5 if !self.free_g32.is_empty() => Some(FreePool::G32(&mut self.free_g32)),
            6 if !self.free_g64.is_empty() => Some(FreePool::G64(&mut self.free_g64)),
            _ => None,
        }
    }
    fn try_cached_free_fp(&mut self, bits_log2: u8) -> Option<FreePool<'_>> {
        match bits_log2 {
            5 if !self.free_f32.is_empty() => Some(FreePool::F32(&mut self.free_f32)),
            6 if !self.free_f64.is_empty() => Some(FreePool::F64(&mut self.free_f64)),
            _ => None,
        }
    }
    pub fn restore_cache(&mut self) {
        while let Some(gpr32) = self.free_g32.pop() {
            let RegID::Virt(index) = gpr32.get_id() else {
                panic!("Expected a virtual GPR32, found {:?}", gpr32.get_id());
            };
            let (slot, is_valid) = self.general.get_mut(index as usize).unwrap();
            *is_valid = true;
            *slot = gpr32.into_real();
        }

        while let Some(gpr64) = self.free_g64.pop() {
            let RegID::Virt(index) = gpr64.get_id() else {
                panic!("Expected a virtual GPR64, found {:?}", gpr64.get_id());
            };
            let (slot, is_valid) = self.general.get_mut(index as usize).unwrap();
            *is_valid = true;
            *slot = gpr64.into_real();
        }

        while let Some(fpr32) = self.free_f32.pop() {
            let RegID::Virt(index) = fpr32.get_id() else {
                panic!("Expected a virtual FPR32, found {:?}", fpr32.get_id());
            };
            let (slot, is_valid) = self.float.get_mut(index as usize).unwrap();
            *is_valid = true;
            *slot = fpr32.into_real();
        }

        while let Some(fpr64) = self.free_f64.pop() {
            let RegID::Virt(index) = fpr64.get_id() else {
                panic!("Expected a virtual FPR64, found {:?}", fpr64.get_id());
            };
            let (slot, is_valid) = self.float.get_mut(index as usize).unwrap();
            *is_valid = true;
            *slot = fpr64.into_real();
        }
    }
}

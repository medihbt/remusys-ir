use std::fmt::Debug;

use crate::{
    mir::operand::{RegUseFlags, SubRegIndex},
    typing::{id::ValTypeID, types::FloatTypeKind},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum VirtReg {
    General(u32, SubRegIndex, RegUseFlags),
    Float(u32, SubRegIndex, RegUseFlags),
    Zero(RegUseFlags),
    SP(RegUseFlags),
}

impl VirtReg {
    pub fn new_long(reg_id: u32) -> Self {
        VirtReg::General(reg_id, SubRegIndex::new(6, 0), RegUseFlags::NONE)
    }
    pub fn new_int(reg_id: u32) -> Self {
        VirtReg::General(reg_id, SubRegIndex::new(5, 0), RegUseFlags::NONE)
    }
    pub fn new_double(reg_id: u32) -> Self {
        VirtReg::Float(reg_id, SubRegIndex::new(6, 0), RegUseFlags::NONE)
    }
    pub fn new_float(reg_id: u32) -> Self {
        VirtReg::Float(reg_id, SubRegIndex::new(5, 0), RegUseFlags::NONE)
    }

    pub fn new_from_type(ir_type: ValTypeID, reg_id: u32) -> Self {
        match ir_type {
            ValTypeID::Void => VirtReg::Zero(RegUseFlags::NONE),
            ValTypeID::Ptr => VirtReg::new_long(reg_id),
            ValTypeID::Int(bits) => {
                if bits <= 32 {
                    VirtReg::new_int(reg_id)
                } else {
                    VirtReg::new_long(reg_id)
                }
            }
            ValTypeID::Float(fp_kind) => match fp_kind {
                FloatTypeKind::Ieee32 => VirtReg::new_float(reg_id),
                FloatTypeKind::Ieee64 => VirtReg::new_double(reg_id),
            },
            ValTypeID::Array(_)
            | ValTypeID::Struct(_)
            | ValTypeID::StructAlias(_)
            | ValTypeID::Func(_) => panic!(
                "Cannot create VirtReg from non-primitive type: {:?}",
                ir_type
            ),
        }
    }

    pub fn use_flags_mut(&mut self) -> &mut RegUseFlags {
        match self {
            VirtReg::General(_, _, uf)
            | VirtReg::Float(_, _, uf)
            | VirtReg::Zero(uf)
            | VirtReg::SP(uf) => uf,
        }
    }
    pub fn get_use_flags(&self) -> RegUseFlags {
        match self {
            VirtReg::General(_, _, uf)
            | VirtReg::Float(_, _, uf)
            | VirtReg::Zero(uf)
            | VirtReg::SP(uf) => *uf,
        }
    }
    pub fn add_use_flag(&mut self, flag: RegUseFlags) {
        self.use_flags_mut().insert(flag);
    }
    pub fn insert_use_flags(mut self, flag: RegUseFlags) -> Self {
        self.add_use_flag(flag);
        self
    }
    pub fn del_use_flag(&mut self, flag: RegUseFlags) {
        self.use_flags_mut().remove(flag);
    }
    pub fn extract_use_flag(mut self, flag: RegUseFlags) -> Self {
        self.del_use_flag(flag);
        self
    }

    pub fn get_bits(&self) -> u8 {
        match self {
            VirtReg::General(_, si, _) | VirtReg::Float(_, si, _) => 1 << si.get_bits_log2(),
            VirtReg::Zero(_) | VirtReg::SP(_) => 64
        }
    }
}

use std::{
    cell::{Ref, RefCell, RefMut},
    path::{Path, PathBuf},
};

use bitflags::bitflags;
use slab::Slab;

use crate::mir::{
    inst::MachineInst,
    symbol::{block::MachineBlock, global::MachineGlobalData},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MIRFormatMode {
    MIR,
    Assembly,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SectionKind {
    Bss,
    Data,
    ROData,
    Text,
}

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq)]
    pub struct Permission: u8 {
        const READ  = 0b0001;
        const WRITE = 0b0010;
        const EXEC  = 0b0100;
    }
}

impl Permission {
    pub fn readable(self) -> bool {
        self.contains(Permission::READ)
    }
    pub fn writable(self) -> bool {
        self.contains(Permission::WRITE)
    }
    pub fn executable(self) -> bool {
        self.contains(Permission::EXEC)
    }
}

impl SectionKind {
    pub fn get_name(self) -> &'static str {
        match self {
            Self::Text => ".text",
            Self::Data => ".data",
            Self::Bss => ".bss",
            Self::ROData => ".rodata",
        }
    }

    pub fn get_permission(self) -> Permission {
        match self {
            Self::Text => Permission::READ | Permission::EXEC,
            Self::Data => Permission::READ | Permission::WRITE,
            Self::Bss => Permission::READ | Permission::WRITE,
            Self::ROData => Permission::READ,
        }
    }

    pub fn readable(self) -> bool {
        self.get_permission().readable()
    }
    pub fn writable(self) -> bool {
        self.get_permission().writable()
    }
    pub fn executable(self) -> bool {
        self.get_permission().executable()
    }
}

/**
 AArch64 MIR module representation.
*/
pub struct MachineMod {
    pub name: String,
    pub alloc: RefCell<MachineAlloc>,
    pub globals: Vec<MachineGlobalData>,
}

pub struct MachineAlloc {
    pub alloc_block: Slab<MachineBlock>,
    pub alloc_inst: Slab<MachineInst>,
}

impl MachineMod {
    pub fn new(name: String) -> Self {
        Self {
            name,
            alloc: RefCell::new(MachineAlloc {
                alloc_block: Slab::new(),
                alloc_inst: Slab::new(),
            }),
            globals: Vec::new(),
        }
    }

    pub fn make_path(&self, base_dir: &Path, format_mode: MIRFormatMode) -> PathBuf {
        let mut path = PathBuf::from(base_dir);
        path.push(&self.name);
        path.set_extension(match format_mode {
            MIRFormatMode::MIR => "mir",
            MIRFormatMode::Assembly => "s",
        });
        path
    }

    pub fn borrow_alloc_block(&self) -> Ref<'_, Slab<MachineBlock>> {
        Ref::map(self.alloc.borrow(), |alloc| &alloc.alloc_block)
    }
    pub fn borrow_alloc_block_mut(&self) -> RefMut<'_, Slab<MachineBlock>> {
        RefMut::map(self.alloc.borrow_mut(), |alloc| &mut alloc.alloc_block)
    }
    pub fn borrow_alloc_inst(&self) -> Ref<'_, Slab<MachineInst>> {
        Ref::map(self.alloc.borrow(), |alloc| &alloc.alloc_inst)
    }
    pub fn borrow_alloc_inst_mut(&self) -> RefMut<'_, Slab<MachineInst>> {
        RefMut::map(self.alloc.borrow_mut(), |alloc| &mut alloc.alloc_inst)
    }
}

use std::cell::Cell;

use crate::mir::{
    inst::{MirInstCommon, opcode::MirOP},
    operand::{
        MirOperand,
        reg::{PReg, RegOP},
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressMode {
    BaseOnly,
    BaseOffset,
    PreIndex,
    PostIndex,
    Literal,
}

pub trait LoadStoreInst {
    fn common(&self) -> &MirInstCommon;
    fn get_addr_mode(&self) -> AddressMode;

    fn get_opcode(&self) -> MirOP {
        self.common().opcode
    }
}

/// Load/store instruction, with all operands being registers.
///
/// AArch64 + MIR assembly syntax:
///
/// - `<load-store-op> <Rt>, [<Rn>, <Rm>]`
/// - `<load-store-op> <Rt>, [<Rn>, <Rm>, <UXTW|SXTW|SXTX>]`
/// - `<load-store-op> <Rt>, [<Rn>, <Rm>, LSL #<shift>]`
///
/// Accepts opcode:
///
/// ```aarch64
/// ldr{b|h|sb|sh|sw}
/// str{b|h|sb|sh|sw}
/// ```
#[derive(Debug, Clone)]
pub struct LoadStoreRRR {
    pub(super) common: MirInstCommon,
    pub operands: [Cell<MirOperand>; 3],
    /// `None` if we keep the `Rm` unmodified,
    /// `Some(RegOP)` if we apply a register operation to `Rm`.
    pub rm_op: Option<RegOP>,
}

impl LoadStoreRRR {
    pub fn new(opcode: MirOP, rm_op: Option<RegOP>) -> Self {
        Self {
            common: MirInstCommon::new(opcode),
            operands: [
                Cell::new(MirOperand::None),                   // Rt
                Cell::new(MirOperand::None),                   // Rn
                Cell::new(MirOperand::PReg(PReg::zr())), // Rm
            ],
            rm_op,
        }
    }

    pub fn rt(&self) -> &Cell<MirOperand> {
        &self.operands[0]
    }
    pub fn rn(&self) -> &Cell<MirOperand> {
        &self.operands[1]
    }
    pub fn rm(&self) -> &Cell<MirOperand> {
        &self.operands[2]
    }
}

impl LoadStoreInst for LoadStoreRRR {
    fn common(&self) -> &MirInstCommon {
        &self.common
    }
    fn get_addr_mode(&self) -> AddressMode {
        match self.rm().get() {
            MirOperand::VReg(_) => AddressMode::BaseOffset,
            MirOperand::PReg(phys_reg) => match phys_reg {
                PReg::ZR(..) => AddressMode::BaseOnly,
                _ => AddressMode::BaseOffset,
            },
            _ => panic!("Invalid operand {:?} for address mode", self.rm().get()),
        }
    }
}

/// Load/store instruction, with its mem address made of a base register and an offset.
///
/// AArch64 + MIR assembly syntax:
///
/// - `<load-store-op> <Rt>, [<Rn>, #<imm>]` => AddressMode::BaseOffset
/// - `<load-store-op> <Rt>, [<Rn>], #<imm>` => AddressMode::PreIndex
/// - `<load-store-op> <Rt>, [<Rn>, #<imm>]!` => AddressMode::PostIndex
///
/// Accepts opcode:
///
/// ```aarch64
/// ldr{b|h|sb|sh|sw}
/// str{b|h|sb|sh|sw}
/// ```
#[derive(Debug, Clone)]
pub struct LoadStoreRRI {
    pub(super) common: MirInstCommon,
    pub operands: [Cell<MirOperand>; 3],
    addr_mode: AddressMode,
}

impl LoadStoreRRI {
    pub fn new(opcode: MirOP, addr_mode: AddressMode) -> Self {
        Self {
            common: MirInstCommon::new(opcode),
            addr_mode,
            operands: [
                Cell::new(MirOperand::None),        // Rt
                Cell::new(MirOperand::None),        // Rn
                Cell::new(MirOperand::Imm(0)), // Offset
            ],
        }
    }

    pub fn rt(&self) -> &Cell<MirOperand> {
        &self.operands[0]
    }
    pub fn rn(&self) -> &Cell<MirOperand> {
        &self.operands[1]
    }
    pub fn offset(&self) -> &Cell<MirOperand> {
        &self.operands[2]
    }
}

impl LoadStoreInst for LoadStoreRRI {
    fn common(&self) -> &MirInstCommon {
        &self.common
    }
    fn get_addr_mode(&self) -> AddressMode {
        self.addr_mode
    }
}

/// Load/store instruction, with a literal value as the address.
///
/// AArch64 + MIR assembly syntax: `<load-store-op> <Rt>, <label>`
///
/// Accepts opcode:
///
/// ```aarch64
/// ldr{b|h|sb|sh|sw}
/// str{b|h|sb|sh|sw}
/// ```
#[derive(Debug, Clone)]
pub struct LoadStoreLiteral {
    pub(super) common: MirInstCommon,
    pub operands: [Cell<MirOperand>; 2],
}

impl LoadStoreLiteral {
    pub fn new(opcode: MirOP) -> Self {
        Self {
            common: MirInstCommon::new(opcode),
            operands: [
                Cell::new(MirOperand::None), // Rt
                Cell::new(MirOperand::None), // Literal address
            ],
        }
    }

    pub fn rt(&self) -> &Cell<MirOperand> {
        &self.operands[0]
    }
    pub fn literal(&self) -> &Cell<MirOperand> {
        &self.operands[1]
    }
}

impl LoadStoreInst for LoadStoreLiteral {
    fn common(&self) -> &MirInstCommon {
        &self.common
    }
    fn get_addr_mode(&self) -> AddressMode {
        AddressMode::Literal
    }
}

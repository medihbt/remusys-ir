use std::cell::Cell;
use remusys_mir_instdef::impl_mir_inst;
use crate::mir::{
    inst::{IMirSubInst, MirInst, MirInstCommon, opcode::MirOP},
    operand::{
        MirOperand,
        reg::{RegOP, RegUseFlags},
        suboperand::*,
    },
};

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddressMode {
    BaseOnly,
    BaseOffset,
    PreIndex,
    PostIndex,
    Literal,
    PseudoImmMaker,
}

pub trait ILoadStoreInst: IMirSubInst {
    fn get_addr_mode(&self) -> AddressMode;
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
/// str{b|h}
/// ```
#[derive(Debug, Clone)]
pub struct LoadStoreRRR {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3],
    pub rm_op: Option<RegOP>,
}

impl_mir_inst! {
    LoadStoreRRR, LoadStoreRRR,
    operands: {
        rt: Reg { use_flags: [DEF] },
        rn: Reg, rm: Reg,
    },
    accept_opcode: [
        Ldr, LdrB, LdrH, LdrSB, LdrSH, LdrSW, Str, StrB, StrH
    ],
    field_inits: {
        pub rm_op: Option<RegOP> = None;
    }
}

impl ILoadStoreInst for LoadStoreRRR {
    fn get_addr_mode(&self) -> AddressMode {
        AddressMode::BaseOffset
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
/// str{b|h}
/// ```
#[derive(Debug, Clone)]
pub struct LoadStoreRRI {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3],
    _addr_mode: AddressMode,
}

impl_mir_inst! {
    LoadStoreRRI, LoadStoreRRI,
    operands: {
        rt: Reg { use_flags: [DEF] },
        rn: Reg, offset: Imm,
    },
    accept_opcode: [
        Ldr, LdrB, LdrH, LdrSB, LdrSH, LdrSW, Str, StrB, StrH
    ],
    field_inits: {
        _addr_mode: AddressMode = AddressMode::BaseOffset;
    }
}

impl LoadStoreRRI {
    pub fn set_addr_mode(&mut self, mode: AddressMode) {
        if self._addr_mode == mode {
            return;
        }
        let mut rn = self.get_rn();
        match mode {
            AddressMode::BaseOffset => {
                rn.use_flags_mut().remove(RegUseFlags::DEF);
            }
            AddressMode::PreIndex | AddressMode::PostIndex => {
                rn.use_flags_mut().insert(RegUseFlags::DEF);
            }
            _ => panic!("Invalid address mode for LoadStoreRRI: {:?}", mode),
        };
        self._addr_mode = mode;
    }
}

impl ILoadStoreInst for LoadStoreRRI {
    fn get_addr_mode(&self) -> AddressMode {
        self._addr_mode
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
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2],
}

impl_mir_inst! {
    LoadStoreLiteral, LoadStoreLiteral,
    operands: {
        rt: Reg { use_flags: [DEF] },
        literal: Symbol,
    },
    accept_opcode: [
        Ldr, LdrB, LdrH, LdrSB, LdrSH, LdrSW,
        Str, StrB, StrH
    ],
}

/// Pesudo Instruction: Load a constant value into a register.
///
/// AArch64 + MIR assembly syntax: `LDR <Rt>, =<imm>`
///
/// Accepts opcode:
///
/// ```aarch64
/// ldr
/// ```
#[derive(Debug, Clone)]
pub struct LoadConst {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2],
}

impl_mir_inst! {
    LoadConst, LoadConst,
    operands: {
        rt: Reg { use_flags: [DEF] },
        imm: Imm,
    },
    accept_opcode: [Ldr],
}

impl ILoadStoreInst for LoadConst {
    fn get_addr_mode(&self) -> AddressMode {
        AddressMode::PseudoImmMaker
    }
}

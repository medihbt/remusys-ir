use std::cell::Cell;

use remusys_mir_instdef::impl_mir_inst;

use crate::mir::{
    inst::{IMirSubInst, MirInst, MirInstCommon, cond::MirCondFlag, opcode::MirOP},
    module::block::MirBlockRef,
    operand::{
        MirOperand,
        reg::{PReg, RegUseFlags},
        suboperand::*,
    },
};

/// Conditional branch instruction in the MIR.
///
/// AArch64 Syntax: `<b.><cond> <label>`
///
/// Operand layout:
///
/// - `[0]`: Label to branch to.
/// - `[1]`: Implicit PSTATE register, used to hold the condition flags.
///   This is typically set to `PSTATE(IMPLICIT_DEF)` to indicate that the
///   condition flags are not explicitly defined in the instruction.
///
/// Accepts opcode:
///
/// ```aarch64
/// b.<cond> bc.<cond>
/// ```
#[derive(Debug, Clone)]
pub struct CondBr {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2],
    pub cond: MirCondFlag,
}

impl_mir_inst! {
    CondBr, CondBr,
    operands: {
        label: Label, csr: PState,
    },
    accept_opcode: [ BCond, BCCond, ],
    field_inits: {
        pub cond: MirCondFlag = MirCondFlag::AL;
    },
}

/// Unconditional branch instruction in the MIR.
///
/// AArch64 Syntax: `<br-opcode> <label>`
///
/// Operand layout:
/// - `[0]`: Label to branch to.
///
/// Accepts opcode:
///
/// ```aarch64
/// b
/// br
/// ```
#[derive(Debug, Clone)]
pub struct UncondBr {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 1],
}

impl_mir_inst! {
    UncondBr, UncondBr,
    operands: { label: Label, },
    accept_opcode: [ Branch, BReg ],
}

/// Branch link instruction in the MIR.
///
/// Operand layout: `<bl-opcode> <label>`
///
/// - `[0]`: Label to branch to.
/// - `[1]`: Implicit `%ra` register, used to hold the return address.
///
/// Accepts opcode:
///
/// ```aarch64
/// bl blr
/// ```
#[derive(Debug, Clone)]
pub struct BLink {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2],
}

impl_mir_inst! {
    BLink, BLink,
    operands: {
        label: Label,
        ra: PReg { use_flags: [IMPLICIT_DEF] },
    },
    accept_opcode: [ BLink, BLinkReg ],
}

/// Compare / Test and branch instruction in the MIR.
///
/// Operand layout:
///
/// - `[0]`: Register condition
/// - `[1]`: Branch target label
///
/// Aarch64 + MIR Assembly:
///
/// - `<opcode> <reg>, <label>`
///
/// Accepts opcode:
///
/// ```aarch64
/// cbz cbnz tbz tbnz
/// ```
#[derive(Debug, Clone)]
pub struct RegCondBr {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2],
}

impl_mir_inst! {
    RegCondBr, RegCondBr,
    operands: {
        reg: Reg,
        label: Label,
    },
    accept_opcode: [ CBZ, CBNZ, TBZ, TBNZ ],
}

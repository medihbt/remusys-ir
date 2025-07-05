use crate::mir::{
    inst::{IMirSubInst, MirInst, MirInstCommon, cond::MirCondFlag, opcode::MirOP},
    operand::{
        MirOperand,
        reg::{RegOP, RegUseFlags},
        suboperand::{IMirSubOperand, PStateSubOperand, RegOperand},
    },
};
use bitflags::bitflags;
use remusys_mir_instdef::impl_mir_inst;
use std::cell::Cell;

bitflags! {
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    pub struct NZCV: u8 {
        const N = 0b0001; // Negative flag
        const Z = 0b0010; // Zero flag
        const C = 0b0100; // Carry flag
        const V = 0b1000; // Overflow flag
    }
}

/// Compare or test instruction
///
/// AArch64 assembly syntax:
///
/// * `cmp-op rn, rm`
/// * `cmp-op rn, rm, <shift flag> #shift`
/// * `cmp-op rn, rm, <SXTX|SXTW|UXTW>`
/// * `cmp-op rn, #imm`
///
/// These syntaxes are mapped to the following Remusys-MIR syntaxes:
///
/// * `cmp-op %rn, %rm, implicit-def $PState`
/// * `cmp-op %rn, %rm, <shift flag> #shift, implicit-def $PState`
/// * `cmp-op %rn, %rm, <SXTX|SXTW|UXTW>, implicit-def $PState`
/// * `cmp-op %rn, #imm, implicit-def $PState`
///
/// Accepts the following opcodes:
///
/// ```aarch64
/// cmp cmn tst
/// fcmp fcmpe
/// ```
#[derive(Debug, Clone)]
pub struct CmpOP {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3],
    pub rhs_op: Option<RegOP>,
}

impl_mir_inst! {
    CmpOP, Cmp,
    operands: {
        rn: Reg { use_flags: [DEF] },
        rhs: MirOperand,
        csr: PState,
    },
    accept_opcode: [ Cmp, CmpN, Test, FCmp, FCmpE ],
    field_inits: {
        pub rhs_op: Option<RegOP> = None;
    },
}

/// Conditional comparation
///
/// AArch64 assembly syntax:
///
/// * `condcmp-op rn, rhs, #<nzcv>, cond`
///
/// These syntaxes are mapped to the following Remusys-MIR syntaxes:
///
/// * `condcmp-op %rn, %rm, #<nzcv>, <cond>, implicit-def $PState`
///
/// Accepts the following opcodes:
///
/// ```aarch64
/// fccmp fccmpe ccmn ccmp
/// ```
#[derive(Debug, Clone)]
pub struct CondCmpOP {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3],
    pub cond: MirCondFlag,
    pub nzcv: NZCV,
}

impl_mir_inst! {
    CondCmpOP, CondCmp,
    operands: {
        rn: Reg { use_flags: [DEF] },
        rhs: MirOperand,
        csr: PState,
    },
    accept_opcode: [ FCCmp, FCCmpE, CCmpN, CCmp ],
    field_inits: {
        pub cond: MirCondFlag = MirCondFlag::EQ;
        pub nzcv: NZCV = NZCV::empty();
    },
}

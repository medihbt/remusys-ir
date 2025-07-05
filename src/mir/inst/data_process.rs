use std::cell::Cell;

use remusys_mir_instdef::impl_mir_inst;

use crate::mir::{
    inst::{IMirSubInst, MirInst, MirInstCommon, cond::MirCondFlag, opcode::MirOP},
    operand::{
        MirOperand,
        reg::{PReg, RegOP, RegUseFlags, VReg},
        suboperand::*,
    },
};

/// Binary Operation Instruction
///
/// AArch64 assembly syntax (Do not show CSR operand if contains a CSR):
///
/// * `binop rd, rn, rm`
/// * `binop rd, rn, rm, <shift flag> #shift`
/// * `binop rd, rn, rm, <SXTX|SXTW|UXTW>`
/// * `binop rd, rn, #imm`
/// * `binop rd, rn, #imm, <shift flag> #shift`
/// * `binop rd, rn, rm, <SXTX|SXTW|UXTW>`
///
/// These syntaxes are mapped to the following Remusys-MIR syntaxes:
///
/// * `binop %rd, %rn, %rm`
/// * `binop %rd, %rn, #imm`
/// * `binop %rd, %rn, %rm,  implicit-def $PState`
/// * `binop %rd, %rn, #imm, implicit-def $PState`
///
/// 3-operand mode accepts opcode:
///
/// ```aarch64
/// add sub
/// smax smin umax umin
/// and bic eon eor orr orn
/// asr lsl lsr ror
/// asrv lslv lsrv rorv
/// mul mneg smnegl umnegl smull smulh umull umulh sdiv udiv
///
/// fadd fsub fmul fdiv fnmul fmax fmin fmaxnm fminnm
/// ```
///
/// 3-operand with CSR mode accepts opcode:
///
/// ```aarch64
/// adds subs
/// adc adcs sbc sbcs
/// ands bics
/// ```
#[derive(Debug, Clone)]
pub struct BinOp {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3],
    pub rhs_op: Option<RegOP>,
}

impl_mir_inst! {
    BinOp, Bin,
    operands: {
        rd: Reg { use_flags: [DEF] },
        rn: Reg,
        rhs: MirOperand,
    },
    accept_opcode: [
        Add, Sub, SMax, SMin, UMax, UMin,
        And, Bic, EON, EOr, Orr, OrN,
        ASR, LSL, LSR, ROR,
        ASRV, LSLV, LSRV, RORV,
        Mul, MNeg, SMNegL, UMNegL, SMulL, SMulH, UMulL, UMulH, SDiv, UDiv,
        FAdd, FSub, FMul, FDiv, FNMul, FMax, FMin, FMaxNM, FMinNM,
    ],
    field_inits: {
        pub rhs_op: Option<RegOP> = None;
    }
}

#[derive(Debug, Clone)]
pub struct BinCSROp {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 4],
    pub rhs_op: Option<RegOP>,
}

impl_mir_inst! {
    BinCSROp, BinCSR,
    operands: {
        rd: Reg { use_flags: [DEF] },
        rn: Reg,
        rhs: MirOperand,
        csr: PState,
    },
    accept_opcode: [ AddS, SubS, AddC, AddCS, SubC, SubCS, AndS, BicS ],
    field_inits: {
        pub rhs_op: Option<RegOP> = None;
    }
}

/// Move and unary operations
///
/// AArch64 assembly syntax:
///
/// * `unary-op rd, rn`
/// * `unary-op rd, rn, <shift flag> #shift`
/// * `unary-op rd, rn, <SXTX|SXTW|UXTW>`
/// * `unary-op rd, #imm`
/// * `adr-op rd, label`
///
/// These syntaxes are mapped to the following Remusys-MIR syntaxes:
///
/// * `unary-op %rd, %rn`
/// * `unary-op %rd, %rn, (<shift flag> #shift | SXTX|SXTW|UXTW)`
/// * `unary-op %rd, %rn, {<shift flag> #shift | SXTX|SXTW|UXTW,} implicit-def $PState`
///
/// * `unary-op %rd, #imm`
/// * `unary-op %rd, #imm, implicit-def $PState`
///
/// Accepts the following opcodes:
///
/// ```aarch64
/// movz movn movk mov
/// adrp adr
/// sxtb sxth sxtw uxtb uxth
/// abs neg
/// cls clz cnt ctz rbit rev{16, 32, 64}
///
/// fmov fcvt
/// fcvtas fcvtau fcvtms fcvtmu
/// fcvtns fcvtnu fcvtps fcvtpu
/// fcvtzs fcvtzu fjcvt.zs
/// scvtf  ucvtf
///
/// frint(a,i,m,n,p,x,z,32x,32z,64x,64z)
/// fabs fneg fsqrt
/// ```
///
/// Accepts the following opcodes with CSR:
///
/// ```aarch64
/// negs ngc ngcs
/// ```
#[derive(Debug, Clone)]
pub struct UnaryOp {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3],
    pub rhs_op: Option<RegOP>,
}

impl_mir_inst! {
    UnaryOp, Unary,
    operands: {
        rd: Reg { use_flags: [DEF] },
        rn: Reg,
        rhs: MirOperand,
    },
    accept_opcode: [
        MovZ, MovN, MovK, Mov,
        AdrP, Adr,
        SxtB, SxtH, SxtW, UxtB, UxtH,
        ABS, Neg,
        ClS, ClZ, Cnt, CntZ, RBit, Rev16, Rev32, Rev64,
        FMov, FCvt,
        FCvtAS, FCvtAU, FCvtMS, FCvtMU,
        FCvtNS, FCvtNU, FCvtPS, FCvtPU,
        FCvtZS, FCvtZU, FJCvtZS,
        SCvtF, UCvtF,
        FRIntA, FRIntI, FRIntN, FRIntP, FRIntX, FRIntZ,
        FRInt32X, FRInt32Z, FRInt64X, FRInt64Z,
        FAbs, FNeg, FSqrt,
    ],
    field_inits: {
        pub rhs_op: Option<RegOP> = None;
    }
}

#[derive(Debug, Clone)]
pub struct UnaCSROp {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3],
    pub rhs_op: Option<RegOP>,
}

impl_mir_inst! {
    UnaCSROp, UnaryCSR,
    operands: {
        rd: Reg { use_flags: [DEF] },
        rn: Reg,
        csr: PState,
    },
    accept_opcode: [ NegS, NegC, NegCS ],
    field_inits: {
        pub rhs_op: Option<RegOP> = None;
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum BFMMode {
    ImmrImms,
    LsbWidth,
}

/// BFM (Bit Field Move) operation
///
/// AArch64 & Remusys-IR assembly syntax:
///
/// * `bfn-op rd, rn, #immr, #imms`
/// * `bfn-op rd, rn, #lsb, #width`
///
/// Accepts the following opcodes:
///
/// ```aarch64
/// bfm sbfm ubfm
/// bfi sbfiz ubfiz
/// bfxil sbfx ubfx
/// ```
#[derive(Debug, Clone)]
pub struct BFMOp {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 4],
    pub mode: BFMMode,
}

impl_mir_inst! {
    BFMOp, BFM,
    operands: {
        rd: Reg { use_flags: [DEF] },
        rn: Reg,
        immr: MirOperand,
        imms: MirOperand,
    },
    accept_opcode: [
        BFM, SBFM, UBFM,
        BFI, SBFIZ, UBFIZ,
        BFXIL, SBFX, UBFX,
    ],
    field_inits: {
        pub mode: BFMMode = BFMMode::ImmrImms;
    }
}

/// Extended Register Operation (ExtROp)
///
/// AArch64 assembly syntax:
///
/// * `extr rd, rn, rm, #imm`
///
/// These syntaxes are mapped to the following Remusys-MIR syntaxes:
///
/// * `extr %rd, %rn, %rm, #imm`
#[derive(Debug, Clone)]
pub struct ExtROp {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 4],
}

impl_mir_inst! {
    ExtROp, ExtR,
    operands: {
        rd: Reg { use_flags: [DEF] },
        rn: Reg, rm: Reg, imm: Imm,
    },
    accept_opcode: [ ExtR ],
}

#[derive(Debug, Clone)]
pub struct TernaryOp {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 4],
}

impl_mir_inst! {
    TernaryOp, Tri,
    operands: {
        rd: Reg { use_flags: [DEF] },
        rn: Reg, rm: Reg, ra: Reg,
    },
    accept_opcode: [
        MAdd, MSub,
        SMAddL, SMSubL, UMAddL, UMSubL,
        FMAdd, FMSub, FNMAdd, FNMSub,
    ],
}

/// Conditional Select Instruction
///
/// AArch64 assembly syntax:
///
/// * `csel rd, rn, rm, <cond>`
///
/// These syntaxes are mapped to the following Remusys-MIR syntaxes:
///
/// * `csel %rd, %rn, %rm, <cond>, implicit-def $PState`
///
/// Accepts the following opcodes:
///
/// ```aarch64
/// csel csinc csinv csneg
/// fcsel
/// ```
#[derive(Debug, Clone)]
pub struct CondSelect {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 4],
    pub cond: MirCondFlag,
}

impl_mir_inst! {
    CondSelect, CondSelect,
    operands: {
        rd: Reg { use_flags: [DEF] },
        rn: Reg, rm: Reg,
        implicit_csr: PState,
    },
    accept_opcode: [
        CSel, CSInc, CSInv, CSNeg, FCSel,
    ],
    field_inits: {
        pub cond: MirCondFlag = MirCondFlag::AL;
    },
}

/// Conditional Unary Operation Instruction
///
/// AArch64 assembly syntax:
///
/// * `cunary-op rd, rn, <cond>`
///
/// These syntaxes are mapped to the following Remusys-MIR syntaxes:
///
/// * `cunary-op %rd, %rn, <cond>, implicit-def $PState`
///
/// Accepts the following opcodes:
///
/// ```aarch64
/// cinc cinv cneg
/// ```
#[derive(Debug, Clone)]
pub struct CondUnaryOp {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3],
    pub cond: MirCondFlag,
}

impl_mir_inst! {
    CondUnaryOp, CondUnary,
    operands: {
        rd: Reg { use_flags: [DEF] },
        rn: Reg,
        implicit_csr: PState,
    },
    accept_opcode: [
        CInc, CInv, CNeg,
    ],
    field_inits: {
        pub cond: MirCondFlag = MirCondFlag::AL;
    },
}

/// Conditional Set Instruction
///
/// AArch64 assembly syntax:
///
/// * `cset rd, <cond>`
///
/// These syntaxes are mapped to the following Remusys-MIR syntaxes:
///
/// * `cset %rd, <cond>, implicit-def $PState`
///
/// Accepts the following opcodes:
///
/// ```aarch64
/// cset csetm
/// ```
#[derive(Debug, Clone)]
pub struct CondSet {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 2],
    pub cond: MirCondFlag,
}

impl_mir_inst! {
    CondSet, CondSet,
    operands: {
        rd: Reg { use_flags: [DEF] },
        csr: PState,
    },
    accept_opcode: [
        CSet, CSetM,
    ],
    field_inits: {
        pub cond: MirCondFlag = MirCondFlag::AL;
    }
}
// #[derive(Debug, Clone)]
// pub struct CondSet {
//     pub(super) common: MirInstCommon,
//     /// [rd, implicit-def $PState]
//     pub operands: [Cell<MirOperand>; 2],
//     pub cond: MirCondFlag,
// }

// impl CondSet {
//     pub fn new(opcode: MirOP, cond: MirCondFlag) -> Self {
//         let operands = [
//             Cell::new(MirOperand::VReg(VReg::new_long(0))), // Rd
//             Cell::new(MirOperand::PReg(PReg::PState(RegUseFlags::IMPLICIT_DEF))), // Implicit CSR
//         ];
//         Self {
//             common: MirInstCommon::new(opcode),
//             operands,
//             cond,
//         }
//     }

//     pub fn get_opcode(&self) -> MirOP {
//         self.common.opcode
//     }

//     pub fn rd(&self) -> &Cell<MirOperand> {
//         &self.operands[0]
//     }
//     pub fn implicit_csr(&self) -> &Cell<MirOperand> {
//         &self.operands[1]
//     }
// }

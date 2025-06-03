#[rustfmt::skip]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum MachineOpcode {
    // AArch64 manual C3.1: Branch instructions
    BCond,  BCCond, CmpBranch,  TestBranch,
    Branch, BLink,  BLinkReg,   BranchReg,  Ret,

    MRS, MSR,

    // 3.1.8 Hints
    Nop, Yield,

    // 3.1.9 Barriers and CLREX instructions
    ClearExclusive,
    DataMemoryBarrier,
    DataSyncBarrier,
    InstSyncBarrier,
    CSDataBarrier,
    ErrorSyncBarrier,

    // AArch64 manual C3.2: Loads and Stores
    Ldr,    LdrB,   LdrSB,  LdrH,   LdrSH,  LdrSW,
    Str,    StrB,   StrH,   StrW,
    Ldp,    LdpSW,  Stp,

    // AArch64 manual C3.5: Data Processing
    Add,  AddS, Sub,  SubS, Cmp,  CmpN, // C3.5.1 Integer arithmetic
    SMax, SMin, UMax, UMin, // C3.5.2 Integer min/max
    And,  Bic,  EON,  EOr,  // C3.5.3 Logic
    Orr,  MVN,  OrN,  Test, // C3.5.3 Logic II
    MovZ, MovN, MovK, Mov,  // C3.5.4-5 Move (32 and 64 bit)
    AdrP, Adr,              // C3.5.6 Address generation
    BFM,  SBFM, UBFM,       // C3.5.7 Bitfield move
    // C3.5.8 Bitfield insert and extract
    BFC,    BFI,    BFXIL,
    SBFIZ,  SBFX,   UBFIZ, UBFX,

    ExtR,                           // C3.5.9 Extract Register
    AshR, LShL, LShR, ROR,          // C3.5.10 Shift and rotate
    SxtB, SxtH, SxtW, UxtB, UxtH,   // C3.5.11 Sign-extend and zero-extend

    AddC, SubC, NegC, NegCN,        // C3.7.3 Arithmetic with carry
    ABS,
    ASRV, LSLV, LSRV, RORV,         // C3.7.9 Shift and rotate with variable amount

    // C3.7.10 Multiply and multiply-accumulate
    MAdd, MSub, MNeg, Mul,
    SMAddL, SMSubL, SMNegL,
    UMAddL, UMSubL, UMNegL,
    UMulL,  UMulH,

    // C3.7.11 Divide
    SDiv, UDiv
}

impl MachineOpcode {
    pub fn to_str(self) -> &'static str {
        match self {
            Self::BCond => "b.cond",
            Self::BCCond => "bc.cond",
            Self::CmpBranch => "cb",
            Self::TestBranch => "tb",
            Self::Branch => "b",
            Self::BLink => "bl",
            Self::BLinkReg => "blr",
            Self::BranchReg => "br",
            Self::Ret => "ret",
            Self::MRS => "mrs",
            Self::MSR => "msr",
            Self::Nop => "nop",
            Self::Yield => "yield",
            Self::ClearExclusive => "clrex",
            Self::DataMemoryBarrier => "dmb",
            Self::DataSyncBarrier => "dsb",
            Self::InstSyncBarrier => "isb",
            Self::CSDataBarrier => "cbar",
            Self::ErrorSyncBarrier => "esb",
            Self::Ldr => "ldr",
            Self::LdrB => "ldrb",
            Self::LdrSB => "ldrsb",
            Self::LdrH => "ldrh",
            Self::LdrSH => "ldrsh",
            Self::LdrSW => "ldrsw",
            Self::Str => "str",
            Self::StrB => "strb",
            Self::StrH => "strh",
            Self::StrW => "strw",
            Self::Ldp => "ldp",
            Self::LdpSW => "ldpsw",
            Self::Stp => "stp",
            Self::Add => "add",
            Self::AddS => "adds",
            Self::Sub => "sub",
            Self::SubS => "subs",
            Self::Cmp => "cmp",
            Self::CmpN => "cmn",
            Self::SMax => "smax",
            Self::SMin => "smin",
            Self::UMax => "umax",
            Self::UMin => "umin",
            Self::And => "and",
            Self::Bic => "bic",
            Self::EON => "eon",
            Self::EOr => "eor",
            Self::Orr => "orr",
            Self::MVN => "mvn",
            Self::OrN => "orn",
            Self::Test => "tst",
            Self::MovZ => "movz",
            Self::MovN => "movn",
            Self::MovK => "movk",
            Self::Mov => "mov",
            Self::AdrP => "adrp",
            Self::Adr => "adr",
            Self::BFM => "bfm",
            Self::SBFM => "sbfm",
            Self::UBFM => "ubfm",
            Self::BFC => "bfc",
            Self::BFI => "bfi",
            Self::BFXIL => "bfxil",
            Self::SBFIZ => "sbfiz",
            Self::SBFX => "sbfx",
            Self::UBFIZ => "ubfiz",
            Self::UBFX => "ubfx",
            Self::ExtR => "extr",
            Self::AshR => "asr",
            Self::LShL => "lsl",
            Self::LShR => "lsr",
            Self::ROR => "ror",
            Self::SxtB => "sxtb",
            Self::SxtH => "sxth",
            Self::SxtW => "sxtw",
            Self::UxtB => "uxtb",
            Self::UxtH => "uxth",
            Self::AddC => "adc",
            Self::SubC => "sbc",
            Self::NegC => "ngc",
            Self::NegCN => "ngcn",
            Self::ABS => "abs",
            Self::ASRV => "asrv",
            Self::LSLV => "lslv",
            Self::LSRV => "lsrv",
            Self::RORV => "rorv",
            Self::MAdd => "madd",
            Self::MSub => "msub",
            Self::MNeg => "mneg",
            Self::Mul => "mul",
            Self::SMAddL => "smaddl",
            Self::SMSubL => "smsubl",
            Self::SMNegL => "smnegl",
            Self::UMAddL => "umaddl",
            Self::UMSubL => "umsubl",
            Self::UMNegL => "umnegl",
            Self::UMulL => "umulh",
            Self::UMulH => "umulh",
            Self::SDiv => "sdiv",
            Self::UDiv => "udiv",
        }
    }
}

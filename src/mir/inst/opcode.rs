#[rustfmt::skip]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
/// AArch64 machine opcodes and Remusys-MIR pesudo-opcodes.
pub enum AArch64OP {
    // AArch64 manual C3.1: Branch instructions
    BCond,  BCCond, CBNZ, CBZ, TBNZ, TBZ,
    Branch, BLink,  BLinkReg,  BReg, Ret,

    MRS, MSR,

    // 3.1.8 Hints
    Nop, Yield,

    // 3.1.9 Barriers and CLREX instructions
    ClrEX,
    DMB, DSB, ISB, CSDB, ESB,

    // AArch64 manual C3.2: Loads and Stores
    Ldr,    LdrB,   LdrSB,  LdrH,   LdrSH,  LdrSW,
    Str,    StrB,   StrH,
    Ldp,    LdpSW,  Stp,

    // AArch64 manual C3.5: Data Processing
    // and C3.7 Data Processing (register)
    Add,  AddS, Sub,  SubS, Cmp, CmpN,  // C3.5.1 Integer arithmetic
    SMax, SMin, UMax, UMin,             // C3.5.2 Integer min/max
    And,  AndS, Bic,  BicS, EON,  EOr,  // C3.5.3 Logic
    Orr,  MVN,  OrN,  Test,             // C3.5.3 Logic II
    MovZ, MovN, MovK, Mov,  // C3.5.4-5 Move (32 and 64 bit)
    AdrP, Adr,              // C3.5.6 Address generation
    BFM,  SBFM, UBFM,       // C3.5.7 Bitfield move
    // C3.5.8 Bitfield insert and extract
    BFC,    BFI,    BFXIL,
    SBFIZ,  SBFX,   UBFIZ, UBFX,

    ExtR,                           // C3.5.9 Extract Register
    ASR,  LSL,  LSR,  ROR,          // C3.5.10 Shift and rotate
    SxtB, SxtH, SxtW, UxtB, UxtH,   // C3.5.11 Sign-extend and zero-extend

    Neg,  NegS,                     // C3.7.1 Arithmetic (shifted register)
    AddC, AddCS, SubC, SubCS, NegC, NegCS, // C3.7.3 Arithmetic with carry
    ABS,
    ASRV, LSLV, LSRV, RORV,         // C3.7.9 Shift and rotate with variable amount

    // C3.7.10.1 Multiply and multiply-accumulate
    MAdd, MSub, MNeg, Mul,
    SMAddL, SMSubL, SMNegL, SMulL, SMulH,
    UMAddL, UMSubL, UMNegL, UMulL, UMulH,
    // C3.7.10.2 Divide
    SDiv, UDiv,
    // < C3.7.11 CRC32 is useless in code generation, ignore them. >
    // 3.7.12 Bit operations
    ClS, ClZ, Cnt, CntZ, RBit, Rev, Rev16, Rev32, Rev64,
    // 3.7.13 Conditional select
    CSel, CSInc, CSInv, CSNeg, CSet, CSetM, CInc, CInv,  CNeg,
    // C3.7.14 Conditional compare
    CCmp, CCmpN,

    // C3.8 Floating-point and SIMD instructions
    FMov, FMovG,
    // C3.8.4.1 Convert floating-point precision
    FCvt,
    // C3.8.4.3 Convert between floating-point and integer or fixed-point
    FCvtAS, FCvtAU, FCvtMS, FCvtMU, FCvtNS, FCvtNU, FCvtPS, FCvtPU,
    FCvtZS, FCvtZU, FJCvtZS,
    SCvtF, UCvtF,
    // C3.8.5.1 Floating-point round to integral value
    FRIntA, FRIntI, FRIntN, FRIntP, FRIntX, FRIntZ,
    // C3.8.5.2 Floating-point round to 32-bit or 64-bit integer
    FRInt32X, FRInt32Z, FRInt64X, FRInt64Z,
    // C3.8.6 Floating-point multiply-add
    FMAdd, FMSub, FNMAdd, FNMSub,
    // C3.8.7 Floating-point arithmetic (one source)
    FAbs, FNeg, FSqrt,
    // C3.8.8 Floating-point arithmetic (two sources)
    FAdd, FDiv, FMul, FNMul, FSub,
    // C3.8.9 Floating-point minimum and maximum
    FMax, FMaxNM, FMin, FMinNM,
    // C3.8.10 Floating-point compare
    FCmp, FCmpE, FCCmp, FCCmpE,
    // C3.8.11 Floating-point select
    FCSel,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum OpcodeNOperands {
    /// AArch64 文档规定了就这么多操作数
    Fix(u32),
    /// 操作数数量不定
    Dyn,
    /// 可能会用到 CSR 寄存器, 在不用到 CSR 时的操作数数量
    MayCSR(u32),
    /// 一定会用到 CSR 寄存器, 实际的寄存器数量是 `n + 1`, 其中 `n` 是这个枚举的值
    MustCSR(u32),

    /// LDR 操作数: 我不知道它是什么
    Ldr,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
// opcode categories, separated by ARM manual C3 sections
pub enum AArch64OPKind {
    Branch,           // C3.1
    System,
    Hint,             // C3.1.8
    Barrier,          // C3.1.9
    LoadStore,        // C3.2
    DataProcessing,   // C3.5, C3.7
    FpSimd,           // C3.8
}

impl AArch64OP {
    pub fn to_string_prob(&self) -> String {
        format!("{:?}", self)
    }

    pub const COUNT: usize = Self::FCSel as usize + 1;

    pub fn get_asm_name(self) -> &'static str {
        type O = AArch64OP;
        #[rustfmt::skip]
        return match self {
            O::BCond => "b.cond", O::BCCond => "bc.cond",
            O::CBNZ => "cbnz", O::CBZ  => "cbz", O::TBNZ    => "tbnz", O::TBZ => "tbz",
            O::Branch => "b",  O::BLink => "bl", O::BLinkReg => "blr", O::BReg => "br",
            O::Ret => "ret",

            O::MRS => "mrs", O::MSR => "msr",
            O::Nop => "nop", O::Yield => "yield",
            O::ClrEX => "clrex",
            O::DMB => "dmb", O::DSB => "dsb", O::ISB => "isb", O::CSDB => "csdb", O::ESB => "esb",

            O::Ldr => "ldr", O::LdrB => "ldrb", O::LdrSB => "ldrsb",
            O::LdrH => "ldrh", O::LdrSH => "ldrsh", O::LdrSW => "ldrsw",
            O::Str => "str", O::StrB => "strb", O::StrH => "strh",
            O::Ldp => "ldp", O::LdpSW => "ldpsw", O::Stp => "stp",
            O::Add => "add", O::AddS => "adds", O::Sub => "sub", O::SubS => "subs",
            O::Cmp => "cmp", O::CmpN => "cmn",
            O::SMax => "smax", O::SMin => "smin", O::UMax => "umax", O::UMin => "umin",
            O::And => "and", O::AndS => "ands", O::Bic => "bic", O::BicS => "bics",
            O::EON => "eon", O::EOr => "eor", O::Orr => "orr",
            O::MVN => "mvn", O::OrN => "orn", O::Test => "tst",

            O::MovZ => "movz", O::MovN => "movn", O::MovK => "movk", O::Mov => "mov",
            O::AdrP => "adrp", O::Adr => "adr",
            O::BFM => "bfm", O::SBFM => "sbfm", O::UBFM => "ubfm",
            O::BFC => "bfc", O::BFI => "bfi", O::BFXIL => "bfxil",
            O::SBFIZ => "sbfiz", O::SBFX => "sbfx", O::UBFIZ => "ubfiz", O::UBFX => "ubfx",
            O::ExtR => "extr",
            O::ASR => "asr", O::LSL => "lsl", O::LSR => "lsr", O::ROR => "ror",
            O::SxtB => "sxtb", O::SxtH => "sxth", O::SxtW => "sxtw", O::UxtB => "uxtb", O::UxtH => "uxth",
            O::Neg => "neg", O::NegS => "negs",
            O::AddC => "adc", O::AddCS => "adcs", O::SubC => "sbc", O::SubCS => "sbcs",
            O::NegC => "ngc", O::NegCS => "ngcs",
            O::ABS => "abs",
            O::ASRV => "asrv", O::LSLV => "lslv", O::LSRV => "lsrv", O::RORV => "rorv",
            O::MAdd => "madd", O::MSub => "msub", O::MNeg => "mneg", O::Mul => "mul",

            O::SMAddL => "smaddl", O::SMSubL => "smsubl", O::SMNegL => "smnegl",
            O::SMulL  => "smull",  O::SMulH  => "smulh",
            O::UMAddL => "umaddl", O::UMSubL => "umsubl", O::UMNegL => "umnegl",
            O::UMulL  => "umull",  O::UMulH  => "umulh",
            O::SDiv => "sdiv", O::UDiv => "udiv",

            O::ClS   => "cls",   O::ClZ   => "clz",   O::Cnt => "cnt", O::CntZ => "cntz",
            O::RBit  => "rbit",  O::Rev   => "rev",
            O::Rev16 => "rev16", O::Rev32 => "rev32", O::Rev64 => "rev64",

            O::CSel => "csel", O::CSInc => "csinc", O::CSInv => "csinv", O::CSNeg => "csneg",
            O::CSet => "cset", O::CSetM => "csetm", O::CInc  => "cinc",  O::CInv  => "cinv", O::CNeg => "cneg",
            O::CCmp => "ccmp", O::CCmpN => "ccmn",
            O::FMov  => "fmov", O::FMovG => "fmov", O::FCvt  => "fcvt",
            O::FCvtAS => "fcvtas", O::FCvtAU => "fcvtau", O::FCvtMS => "fcvtms",
            O::FCvtMU => "fcvtmu", O::FCvtNS => "fcvtns", O::FCvtNU => "fcvtnu",
            O::FCvtPS => "fcvtps", O::FCvtPU => "fcvtpu", O::FCvtZS => "fcvtzs",
            O::FCvtZU => "fcvtzu", O::FJCvtZS => "fjcvtzs",
            O::SCvtF => "scvtf", O::UCvtF => "ucvtf",

            O::FRIntA => "frinta", O::FRIntI => "frinti", O::FRIntN => "frintn",
            O::FRIntP => "frintp", O::FRIntX => "frintx", O::FRIntZ => "frintz",
            O::FRInt32X => "frint32x", O::FRInt32Z => "frint32z",
            O::FRInt64X => "frint64x", O::FRInt64Z => "frint64z",

            O::FMAdd => "fmadd", O::FMSub => "fmsub", O::FNMAdd => "fnmadd", O::FNMSub => "fnmsub",
            O::FAbs  => "fabs",  O::FNeg  => "fneg",  O::FSqrt => "fsqrt",
            O::FAdd  => "fadd",  O::FDiv  => "fdiv",  O::FMul  => "fmul", 
            O::FNMul => "fnmul", O::FSub  => "fsub",
            O::FMax  => "fmax", O::FMaxNM => "fmaxnm",
            O::FMin  => "fmin", O::FMinNM => "fminnm",
            O::FCmp  => "fcmp", O::FCmpE => "fcmpe", O::FCCmp => "fccmp", O::FCCmpE => "fccmpe",
            O::FCSel => "fcsel",
        };
    }

    pub const fn get_category(self) -> AArch64OPKind {
        type O = AArch64OP;
        type C = AArch64OPKind;
        #[rustfmt::skip]
        return match self {
            O::BCond | O::BCCond | O::CBNZ | O::CBZ | O::TBNZ | O::TBZ |
            O::Branch | O::BLink | O::BLinkReg | O::BReg | O::Ret => C::Branch,

            O::Nop | O::Yield => C::Hint,

            O::ClrEX | O::DMB | O::DSB | O::ISB | O::CSDB | O::ESB => C::Barrier,

            // Loads and Stores
            O::Ldr | O::LdrB | O::LdrSB | O::LdrH | O::LdrSH | O::LdrSW |
            O::Str | O::StrB | O::StrH |
            O::Ldp | O::LdpSW | O::Stp => C::LoadStore,

            O::MRS | O::MSR => C::System,

            // Data Processing
            O::Add | O::AddS | O::Sub | O::SubS | O::Cmp | O::CmpN |
            O::SMax | O::SMin | O::UMax | O::UMin |
            O::And | O::AndS | O::Bic | O::BicS | O::EON | O::EOr |
            O::Orr | O::MVN | O::OrN | O::Test |
            O::MovZ | O::MovN | O::MovK | O::Mov |
            O::AdrP | O::Adr |
            O::BFM | O::SBFM | O::UBFM |
            O::BFC | O::BFI | O::BFXIL |
            O::SBFIZ | O::SBFX | O::UBFIZ | O::UBFX |
            O::ExtR |
            O::ASR | O::LSL | O::LSR | O::ROR |
            O::SxtB | O::SxtH | O::SxtW | O::UxtB | O::UxtH |
            O::Neg | O::NegS |
            O::AddC | O::AddCS | O::SubC | O::SubCS | O::NegC | O::NegCS |
            O::ABS |
            O::ASRV | O::LSLV | O::LSRV | O::RORV |
            O::MAdd | O::MSub | O::MNeg | O::Mul |
            O::SMAddL | O::SMSubL | O::SMNegL | O::SMulL | O::SMulH |
            O::UMAddL | O::UMSubL | O::UMNegL | O::UMulL | O::UMulH |
            O::SDiv | O::UDiv |
            O::ClS | O::ClZ | O::Cnt | O::CntZ | O::RBit | O::Rev |
            O::Rev16 | O::Rev32 | O::Rev64 |
            O::CSel | O::CSInc | O::CSInv | O::CSNeg | O::CSet | O::CSetM |
            O::CInc | O::CInv | O::CNeg |
            O::CCmp | O::CCmpN => C::DataProcessing,

            // Floating-point and SIMD instructions
            O::FMov | O::FMovG |
            O::FCvt |
            O::FCvtAS | O::FCvtAU | O::FCvtMS | O::FCvtMU |
            O::FCvtNS | O::FCvtNU | O::FCvtPS | O::FCvtPU |
            O::FCvtZS | O::FCvtZU | O::FJCvtZS |
            O::SCvtF | O::UCvtF |
            O::FRIntA | O::FRIntI | O::FRIntN | O::FRIntP |
            O::FRIntX | O::FRIntZ |
            O::FRInt32X | O::FRInt32Z |
            O::FRInt64X | O::FRInt64Z |
            O::FMAdd | O::FMSub | O::FNMAdd | O::FNMSub |
            O::FAbs | O::FNeg | O::FSqrt |
            O::FAdd | O::FDiv | O::FMul | O::FNMul | O::FSub |
            O::FMax | O::FMaxNM | O::FMin | O::FMinNM |
            O::FCmp | O::FCmpE | O::FCCmp | O::FCCmpE |
            O::FCSel => C::FpSimd,
        };
    }

    /// 这个操作码代表的指令可能会有多少个操作数.
    /// 
    /// 对于会读取或修改 CSR 的指令, CSR 会被当成一个额外的操作数.
    /// 
    /// 例如 `Add` 指令有 3 个操作数（2个源操作数和 1 个目的操作数）, 而
    /// `AddS` `AddC` `AddCS` 这样的指令有 4 个操作数——
    /// AddS 会写、AddC 会读、AddCS 会读写 CSR.
    pub const fn get_n_operands(self) -> OpcodeNOperands {
        type O = AArch64OP;
        type N = OpcodeNOperands;
        #[rustfmt::skip]
        return match self {
            O::BCond | O::BCCond => N::MustCSR(1), // 会读取 CSR
            O::CBNZ  | O::CBZ | O::TBNZ | O::TBZ => N::MustCSR(2),
            O::Branch | O::BLink  => N::Fix(1),
            O::Ret | O::Nop | O::Yield => N::Fix(0),
            
            O::Ldr | O::LdrB | O::LdrSB | O::LdrH | O::LdrSH | O::LdrSW => N::Ldr,
            O::Str | O::StrB | O::StrH => N::Ldr,

            O::MRS | O::MSR => N::MustCSR(1),

            O::Add | O::Sub | O::SMax | O::SMin | O::UMax | O::UMin => N::Fix(3),
            O::Neg => N::Fix(2),
            O::AddS | O::SubS => N::MustCSR(3),
            O::Cmp  | O::CmpN | O::NegS => N::MustCSR(2),

            O::AddC | O::AddCS | O::SubC | O::SubCS => N::MustCSR(3),
            O::NegC | O::NegCS => N::MustCSR(2),

            O::And | O::Bic | O::EON | O::EOr | O::Orr | O::OrN | O::ASRV | O::LSLV | O::LSRV | O::RORV => N::Fix(3),
            O::AndS | O::BicS => N::MustCSR(3),
            O::Test => N::MustCSR(2),
            O::MovZ | O::MovN | O::MVN | O::MovK | O::Mov | O::ABS => N::Fix(2),

            O::AdrP | O::Adr => N::Fix(2),

            O::BFM | O::SBFM | O::UBFM | O::BFI | O::BFXIL | O::SBFIZ | O::SBFX | O::UBFIZ | O::UBFX => N::Fix(4),
            O::BFC => N::Fix(3),
            O::ASR | O::LSL | O::LSR | O::ROR | O::SxtB | O::SxtH | O::SxtW | O::UxtB | O::UxtH => N::Fix(2),

            O::MAdd | O::MSub | O::SMAddL | O::SMSubL | O::UMAddL | O::UMSubL => N::Fix(4),
            O::MNeg | O::Mul | O::SMNegL | O::SMulL | O::SMulH | O::UMNegL | O::UMulL | O::UMulH => N::Fix(3),

            O::SDiv | O::UDiv => N::Fix(3),

            O::ClS | O::ClZ | O::Cnt | O::CntZ | O::RBit | O::Rev | O::Rev16 | O::Rev32 | O::Rev64 => N::Fix(2),

            O::CSel | O::CSInc | O::CSInv | O::CSNeg => N::MustCSR(3),
            O::CSet | O::CSetM => N::MustCSR(1),
            O::CInc | O::CInv | O::CNeg | O::CCmpN | O::CCmp => N::MustCSR(2),
            
            _ => todo!("not implemented")
        };
    }
}

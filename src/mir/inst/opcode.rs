#[rustfmt::skip]
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
#[repr(u8)]
/// AArch64 machine opcodes and Remusys-MIR pesudo-opcodes.
pub enum MirOP {
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

    // Remusys-MIR pseudo-opcodes
    Call,
    TailCall,
    CallIndirect,
    /// return 指令: 根据上下文自动展开为合适的指令.
    MirReturn,

    /// switch 指令: 通过跳转表直接跳转到目标地址
    TabSwitch,
    /// switch 指令: 通过二进制搜索跳转到目标地址
    BinSwitch,
}

impl MirOP {
    /// 该 MirOP 是否是 call switch 这种 MIR 独有的伪指令.
    pub const fn is_mir_pseudo(self) -> bool {
        type O = MirOP;
        matches!(
            self,
            O::Call | O::TailCall | O::CallIndirect | O::TabSwitch | O::BinSwitch
        )
    }
    pub const fn is_load(self) -> bool {
        type O = MirOP;
        matches!(
            self,
            O::Ldr | O::LdrB | O::LdrSB | O::LdrH | O::LdrSH | O::LdrSW | O::Ldp | O::LdpSW
        )
    }
    pub fn lowercase_name(self) -> String {
        format!("{self:?}").to_lowercase()
    }

    pub fn asm_name(self) -> &'static str {
        type O = MirOP;
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

            O::Call => "call", O::TailCall => "tailcall", O::CallIndirect => "call_indirect",
            O::MirReturn => "mir_return",
            O::TabSwitch => "tabswitch", O::BinSwitch => "binswitch",
        };
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum OperandLayout {
    /// No operands.
    Nullary,

    /// Implicitly uses `$ra` (X30) register.
    BLink(u8),

    /// Operand layout collections for loads and stores.
    LoadStore,

    /// Operand number fixed and has no implicit register.
    NoImplicit(u8),

    /// Implicitly uses CSR ($PState) register.
    ImplicitCSR(u8),

    /// Implicitly uses PC register.
    ImplicitPC(u8),

    /// Pesudo `call`:
    ///
    /// * implicitly uses `$ra` (X30) register for return address.
    /// * implicitly uses `$x0-$x7` registers for integer arguments.
    /// * implicitly uses `$f0-$f7` registers for floating-point arguments.
    /// * operand numbers are dynamic.
    ///
    /// `bool` indicates whether the call is dynamic call which uses a register to hold the function address.
    Call(bool),

    /// Pesudo `MirReturn`
    MirReturn,

    Switch,

    Unsupported,
}

impl MirOP {
    pub const fn get_operand_layout(self) -> OperandLayout {
        type O = MirOP;
        type N = OperandLayout;

        // #[rustfmt::skip]
        return match self {
            O::BCond | O::BCCond => N::ImplicitCSR(1),
            O::CBNZ | O::CBZ | O::TBNZ | O::TBZ => N::ImplicitCSR(2),
            O::Branch => N::NoImplicit(1),
            O::BLink | O::BLinkReg => N::BLink(1),
            O::BReg => N::NoImplicit(2),
            O::Ret => N::BLink(0),

            O::MRS | O::MSR => N::NoImplicit(2),
            O::Nop | O::Yield => N::Nullary,

            O::ClrEX | O::DMB | O::DSB | O::ISB | O::CSDB | O::ESB => N::Unsupported,

            O::Ldr | O::LdrB | O::LdrSB | O::LdrH | O::LdrSH | O::LdrSW => N::LoadStore,
            O::Str | O::StrB | O::StrH => N::LoadStore,
            O::Ldp | O::LdpSW | O::Stp => N::LoadStore,

            O::Add | O::Sub | O::SMax | O::SMin | O::UMax | O::UMin | O::And => N::NoImplicit(3),
            O::ASRV | O::LSLV | O::LSRV | O::RORV => N::NoImplicit(3),
            O::EON | O::EOr | O::Bic | O::Orr | O::OrN => N::NoImplicit(3),

            O::Cmp | O::CmpN | O::Test => N::ImplicitCSR(2),

            O::MVN | O::MovZ | O::MovN | O::MovK | O::Mov | O::ABS => N::NoImplicit(2),
            O::AdrP | O::Adr => N::ImplicitPC(2),

            O::BFM | O::SBFM | O::UBFM | O::BFI | O::BFXIL | O::SBFX | O::UBFX | O::ExtR => {
                N::NoImplicit(4)
            }
            O::BFC | O::SBFIZ | O::UBFIZ => N::NoImplicit(3),

            O::ASR | O::LSL | O::LSR | O::ROR => N::NoImplicit(3),
            O::SxtB | O::SxtH | O::SxtW | O::UxtB | O::UxtH | O::Neg => N::NoImplicit(2),

            O::AddS | O::SubS | O::AndS | O::BicS => N::ImplicitCSR(3),
            O::AddC | O::AddCS | O::SubC | O::SubCS => N::ImplicitCSR(3),
            O::NegS | O::NegC | O::NegCS => N::ImplicitCSR(2),

            O::MNeg | O::Mul | O::SMNegL | O::SMulL | O::SMulH => N::NoImplicit(3),
            O::UMNegL | O::UMulL | O::UMulH | O::SDiv | O::UDiv => N::NoImplicit(3),

            O::MAdd | O::MSub | O::SMAddL | O::SMSubL | O::UMAddL | O::UMSubL => N::NoImplicit(4),

            O::ClS | O::ClZ | O::Cnt | O::CntZ => N::NoImplicit(2),
            O::RBit | O::Rev | O::Rev16 | O::Rev32 | O::Rev64 => N::NoImplicit(2),

            O::CSel | O::CSInc | O::CSInv | O::CSNeg => N::NoImplicit(3),
            O::CSet | O::CSetM => N::ImplicitCSR(1),
            O::CInc | O::CInv | O::CNeg => N::ImplicitCSR(2),
            O::CCmp | O::CCmpN => N::ImplicitCSR(2),

            O::FMov | O::FMovG | O::FCvt => N::NoImplicit(2),
            O::FCvtAS | O::FCvtAU | O::FCvtMS | O::FCvtMU => N::NoImplicit(2),
            O::FCvtNS | O::FCvtNU | O::FCvtPS | O::FCvtPU => N::NoImplicit(2),
            O::FCvtZS | O::FCvtZU | O::FJCvtZS => N::NoImplicit(2),
            O::SCvtF | O::UCvtF => N::NoImplicit(2),

            O::FRIntA | O::FRIntI | O::FRIntN | O::FRIntP | O::FRIntX | O::FRIntZ => {
                N::NoImplicit(2)
            }
            O::FRInt32X | O::FRInt32Z | O::FRInt64X | O::FRInt64Z => N::NoImplicit(2),

            O::FMAdd | O::FMSub | O::FNMAdd | O::FNMSub => N::NoImplicit(4),
            O::FAbs | O::FNeg | O::FSqrt => N::NoImplicit(2),
            O::FAdd | O::FDiv | O::FMul | O::FNMul | O::FSub => N::NoImplicit(3),
            O::FMax | O::FMaxNM | O::FMin | O::FMinNM => N::NoImplicit(3),

            O::FCmp | O::FCmpE | O::FCCmp | O::FCCmpE => N::ImplicitCSR(2),
            O::FCSel => N::ImplicitCSR(3),

            O::Call | O::TailCall => N::Call(false),
            O::CallIndirect => N::Call(true),
            O::MirReturn => N::MirReturn,

            O::TabSwitch | O::BinSwitch => N::Switch,
        };
    }
}

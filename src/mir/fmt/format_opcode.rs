use crate::mir::inst::opcode::MirOP;

/// 获取MIR操作码的字符串名称（静态字符串引用）
pub fn opcode_get_name_str(opcode: MirOP) -> &'static str {
    match opcode {
        // 条件分支指令
        MirOP::BCond => "b.cond",
        MirOP::BCCond => "bc.cond",

        // 寄存器条件分支指令
        MirOP::CBZ => "cbz",
        MirOP::CBNZ => "cbnz",
        MirOP::TBZ64 | MirOP::TBZ32 => "tbz",
        MirOP::TBNZ64 | MirOP::TBNZ32 => "tbnz",

        // 无条件分支指令
        MirOP::B => "b",
        MirOP::Br => "br",
        MirOP::Ret => "ret",
        MirOP::BLink | MirOP::BLinkGlobal => "bl",
        MirOP::BLinkReg => "blr",

        // 64位整数比较指令（寄存器操作数）
        MirOP::ICmp64R => "cmp",
        MirOP::ICmn64R => "cmn",

        // 32位整数比较指令（寄存器操作数）
        MirOP::ICmp32R => "cmp",
        MirOP::ICmn32R => "cmn",

        // 64位整数比较指令（立即数操作数）
        MirOP::ICmp64I => "cmp",
        MirOP::ICmn64I => "cmn",

        // 32位整数比较指令（立即数操作数）
        MirOP::ICmp32I => "cmp",
        MirOP::ICmn32I => "cmn",

        // 32位浮点比较指令
        MirOP::FCmp32 => "fcmp",
        MirOP::FCmpE32 => "fcmpe",

        // 64位浮点比较指令
        MirOP::FCmp64 => "fcmp",
        MirOP::FCmpE64 => "fcmpe",

        // 64位条件整数比较指令（寄存器操作数）
        MirOP::ICCmp64R => "ccmp",
        MirOP::ICCmn64R => "ccmn",

        // 32位条件整数比较指令（寄存器操作数）
        MirOP::ICCmp32R => "ccmp",
        MirOP::ICCmn32R => "ccmn",

        // 64位条件整数比较指令（立即数操作数）
        MirOP::ICCmp64I => "ccmp",
        MirOP::ICCmn64I => "ccmn",

        // 32位条件整数比较指令（立即数操作数）
        MirOP::ICCmp32I => "ccmp",
        MirOP::ICCmn32I => "ccmn",

        // 32位条件浮点比较指令
        MirOP::FCCmp32 => "fccmp",
        MirOP::FCCmpE32 => "fccmpe",

        // 64位条件浮点比较指令
        MirOP::FCCmp64 => "fccmp",
        MirOP::FCCmpE64 => "fccmpe",

        // 64位二元运算指令（寄存器操作数）
        MirOP::Add64R => "add",
        MirOP::Sub64R => "sub",
        MirOP::SMax64R => "smax",
        MirOP::SMin64R => "smin",
        MirOP::UMax64R => "umax",
        MirOP::UMin64R => "umin",
        MirOP::And64R => "and",
        MirOP::Bic64R => "bic",
        MirOP::EON64R => "eon",
        MirOP::EOR64R => "eor",
        MirOP::ORR64R => "orr",
        MirOP::ORN64R => "orn",
        MirOP::Asr64R => "asr",
        MirOP::Lsr64R => "lsr",
        MirOP::Lsl64R => "lsl",
        MirOP::Ror64R => "ror",
        MirOP::Mul64 => "mul",
        MirOP::MNeg64 => "mneg",
        MirOP::SDiv64 => "sdiv",
        MirOP::UDiv64 => "udiv",
        MirOP::SMulH => "smulh",
        MirOP::UMulH => "umulh",

        // 32位二元运算指令（寄存器操作数）
        MirOP::Add32R => "add",
        MirOP::Sub32R => "sub",
        MirOP::SMax32R => "smax",
        MirOP::SMin32R => "smin",
        MirOP::UMax32R => "umax",
        MirOP::UMin32R => "umin",
        MirOP::And32R => "and",
        MirOP::Bic32R => "bic",
        MirOP::EON32R => "eon",
        MirOP::EOR32R => "eor",
        MirOP::ORR32R => "orr",
        MirOP::ORN32R => "orn",
        MirOP::Asr32R => "asr",
        MirOP::Lsr32R => "lsr",
        MirOP::Lsl32R => "lsl",
        MirOP::Ror32R => "ror",
        MirOP::Mul32 => "mul",
        MirOP::MNeg32 => "mneg",
        MirOP::SDiv32 => "sdiv",
        MirOP::UDiv32 => "udiv",
        MirOP::SMULL => "smull",
        MirOP::UMULL => "umull",
        MirOP::SMNegL => "smnegl",
        MirOP::UMNegL => "umnegl",

        // 64位二元运算指令（立即数操作数）
        MirOP::Add64I => "add",
        MirOP::Sub64I => "sub",
        MirOP::And64I => "and",
        MirOP::Bic64I => "bic",
        MirOP::EON64I => "eon",
        MirOP::EOR64I => "eor",
        MirOP::ORR64I => "orr",
        MirOP::ORN64I => "orn",
        MirOP::SMax64I => "smax",
        MirOP::SMin64I => "smin",
        MirOP::UMax64I => "umax",
        MirOP::UMin64I => "umin",
        MirOP::Asr64I => "asr",
        MirOP::Lsr64I => "lsr",
        MirOP::Lsl64I => "lsl",
        MirOP::Ror64I => "ror",

        // 32位二元运算指令（立即数操作数）
        MirOP::Add32I => "add",
        MirOP::Sub32I => "sub",
        MirOP::And32I => "and",
        MirOP::Bic32I => "bic",
        MirOP::EON32I => "eon",
        MirOP::EOR32I => "eor",
        MirOP::ORR32I => "orr",
        MirOP::ORN32I => "orn",
        MirOP::SMax32I => "smax",
        MirOP::SMin32I => "smin",
        MirOP::UMax32I => "umax",
        MirOP::UMin32I => "umin",
        MirOP::Asr32I => "asr",
        MirOP::Lsr32I => "lsr",
        MirOP::Lsl32I => "lsl",
        MirOP::Ror32I => "ror",

        // 浮点二元运算指令
        MirOP::FAdd64 => "fadd",
        MirOP::FDiv64 => "fdiv",
        MirOP::FMul64 => "fmul",
        MirOP::FNMul64 => "fnmul",
        MirOP::FSub64 => "fsub",
        MirOP::FAdd32 => "fadd",
        MirOP::FDiv32 => "fdiv",
        MirOP::FMul32 => "fmul",
        MirOP::FNMul32 => "fnmul",
        MirOP::FSub32 => "fsub",

        // MIR伪拷贝指令
        MirOP::MirCopy64 => "mir.copy",
        MirOP::MirCopy32 => "mir.copy",
        MirOP::MirFCopy64 => "mir.fcopy",
        MirOP::MirFCopy32 => "mir.fcopy",
        MirOP::MirPCopy => "mir.pcopy",

        // 64位一元运算指令
        MirOP::Neg64R => "neg",
        MirOP::MVN64R => "mvn",
        MirOP::Mov64R => "mov",
        MirOP::Abs64R => "abs",
        MirOP::CLS64 => "cls",
        MirOP::CLZ64 => "clz",
        MirOP::CNT64 => "cnt",
        MirOP::CTZ64 => "ctz",
        MirOP::RBit64 => "rbit",

        // 32位一元运算指令
        MirOP::Neg32R => "neg",
        MirOP::MVN32R => "mvn",
        MirOP::Mov32R => "mov",
        MirOP::Abs32R => "abs",
        MirOP::CLS32 => "cls",
        MirOP::CLZ32 => "clz",
        MirOP::CNT32 => "cnt",
        MirOP::CTZ32 => "ctz",
        MirOP::RBit32 => "rbit",

        // 32位符号/零扩展指令
        MirOP::SXTB32 => "sxtb",
        MirOP::SXTH32 => "sxth",
        MirOP::SXTW32 => "sxtw",
        MirOP::UXTB32 => "uxtb",
        MirOP::UXTH32 => "uxth",

        // 64位符号/零扩展指令
        MirOP::SXTB64 => "sxtb",
        MirOP::SXTH64 => "sxth",
        MirOP::SXTW64 => "sxtw",
        MirOP::UXTB64 => "uxtb",
        MirOP::UXTH64 => "uxth",

        // 立即数移动指令
        MirOP::Mov64I => "mov",
        MirOP::MovZ64 => "movz",
        MirOP::MovN64 => "movn",
        MirOP::MovK64 => "movk",
        MirOP::Mov32I => "mov",
        MirOP::MovZ32 => "movz",
        MirOP::MovN32 => "movn",
        MirOP::MovK32 => "movk",

        // 地址计算指令
        MirOP::AdrP => "adrp",
        MirOP::Adr => "adr",

        // 浮点与通用寄存器转换指令（64位）
        MirOP::FMovFG64 => "fmov",
        MirOP::SCvtF64 => "scvtf",
        MirOP::UCvtF64 => "ucvtf",
        MirOP::FMovGF64 => "fmov",
        MirOP::FCvtAS64 => "fcvtas",
        MirOP::FCvtAU64 => "fcvtau",
        MirOP::FCvtMS64 => "fcvtms",
        MirOP::FCvtMU64 => "fcvtmu",
        MirOP::FCvtNS64 => "fcvtns",
        MirOP::FCvtNU64 => "fcvtnu",
        MirOP::FCvtPS64 => "fcvtps",
        MirOP::FCvtPU64 => "fcvtpu",
        MirOP::FCvtZS64 => "fcvtzs",
        MirOP::FCvtZU64 => "fcvtzu",
        MirOP::SCvtF64G32 => "scvtf",
        MirOP::UCvtF64G32 => "ucvtf",

        // 浮点与通用寄存器转换指令（32位）
        MirOP::FMovFG32 => "fmov",
        MirOP::SCvtF32 => "scvtf",
        MirOP::UCvtF32 => "ucvtf",
        MirOP::SCvtF32G64 => "scvtf",
        MirOP::UCvtF32G64 => "ucvtf",

        MirOP::FMovGF32 => "fmov",
        MirOP::FCvtAS32 => "fcvtas",
        MirOP::FCvtAU32 => "fcvtau",
        MirOP::FCvtMS32 => "fcvtms",
        MirOP::FCvtMU32 => "fcvtmu",
        MirOP::FCvtNS32 => "fcvtns",
        MirOP::FCvtNU32 => "fcvtnu",
        MirOP::FCvtPS32 => "fcvtps",
        MirOP::FCvtPU32 => "fcvtpu",
        MirOP::FCvtZS32 => "fcvtzs",
        MirOP::FCvtZU32 => "fcvtzu",

        // 64位到32位浮点转换指令
        MirOP::FCvtAS64F32 => "fcvtas",
        MirOP::FCvtAU64F32 => "fcvtau",
        MirOP::FCvtMS64F32 => "fcvtms",
        MirOP::FCvtMU64F32 => "fcvtmu",
        MirOP::FCvtNS64F32 => "fcvtns",
        MirOP::FCvtNU64F32 => "fcvtnu",
        MirOP::FCvtPS64F32 => "fcvtps",
        MirOP::FCvtPU64F32 => "fcvtpu",
        MirOP::FCvtZS64F32 => "fcvtzs",
        MirOP::FCvtZU64F32 => "fcvtzu",

        // 32位到64位浮点转换指令
        MirOP::FCvtAS32F64 => "fcvtas",
        MirOP::FCvtAU32F64 => "fcvtau",

        // 64位浮点一元运算指令
        MirOP::FMov64R => "fmov",
        MirOP::FRIntA64 => "frinta",
        MirOP::FRIntI64 => "frinti",
        MirOP::FRIntM64 => "frintm",
        MirOP::FRIntN64 => "frintn",
        MirOP::FRIntP64 => "frintp",
        MirOP::FRIntX64 => "frintx",
        MirOP::FRIntZ64 => "frintz",
        MirOP::FRInt32X64 => "frint32x",
        MirOP::FRIntZ32X64 => "frint32z",
        MirOP::FRInt64X64 => "frint64x",
        MirOP::FRIntZ64X64 => "frint64z",
        MirOP::FAbs64 => "fabs",
        MirOP::FNeg64 => "fneg",
        MirOP::FSqrt64 => "fsqrt",

        // 32位浮点一元运算指令
        MirOP::FMov32R => "fmov",
        MirOP::FRIntA32 => "frinta",
        MirOP::FRIntI32 => "frinti",
        MirOP::FRIntM32 => "frintm",
        MirOP::FRIntN32 => "frintn",
        MirOP::FRIntP32 => "frintp",
        MirOP::FRIntX32 => "frintx",
        MirOP::FRIntZ32 => "frintz",
        MirOP::FRInt32X32 => "frint32x",
        MirOP::FRIntZ32X32 => "frint32z",
        MirOP::FRInt64X32 => "frint64x",
        MirOP::FRIntZ64X32 => "frint64z",
        MirOP::FAbs32 => "fabs",
        MirOP::FNeg32 => "fneg",
        MirOP::FSqrt32 => "fsqrt",

        // 浮点精度转换指令
        MirOP::FCvt32F64 => "fcvt",
        MirOP::FCvt64F32 => "fcvt",

        // 浮点立即数移动指令
        MirOP::FMov64I => "fmov",
        MirOP::FMov32I => "fmov",

        // 三元运算指令
        MirOP::MAdd64 => "madd",
        MirOP::MSub64 => "msub",
        MirOP::SMAddL => "smaddl",
        MirOP::SMSubL => "smsubl",
        MirOP::UMAddL => "umaddl",
        MirOP::UMSubL => "umsubl",
        MirOP::MAdd32 => "madd",
        MirOP::MSub32 => "msub",

        // 浮点三元运算指令
        MirOP::FMAdd64 => "fmadd",
        MirOP::FMSub64 => "fmsub",
        MirOP::FNMAdd64 => "fnmadd",
        MirOP::FNMSub64 => "fnmsub",
        MirOP::FMAdd32 => "fmadd",
        MirOP::FMSub32 => "fmsub",
        MirOP::FNMAdd32 => "fnmadd",
        MirOP::FNMSub32 => "fnmsub",

        // 64位加载/存储指令（寄存器寻址）
        MirOP::LdrGr64 => "ldr",
        MirOP::LdrBGr64 => "ldrb",
        MirOP::LdrHGr64 => "ldrh",
        MirOP::LdrSBGr64 => "ldrsb",
        MirOP::LdrSHGr64 => "ldrsh",
        MirOP::StrGr64 => "str",
        MirOP::StrBGr64 => "strb",
        MirOP::StrHGr64 => "strh",

        // 32位加载/存储指令（寄存器寻址）
        MirOP::LdrGr32 => "ldr",
        MirOP::LdrBGr32 => "ldrb",
        MirOP::LdrHGr32 => "ldrh",
        MirOP::LdrSBGr32 => "ldrsb",
        MirOP::LdrSHGr32 => "ldrsh",
        MirOP::StrGr32 => "str",
        MirOP::StrBGr32 => "strb",
        MirOP::StrHGr32 => "strh",

        // 浮点加载/存储指令（寄存器寻址）
        MirOP::LdrF64 => "ldr",
        MirOP::StrF64 => "str",
        MirOP::LdrF32 => "ldr",
        MirOP::StrF32 => "str",

        // 64位加载/存储指令（基址偏移寻址）
        MirOP::LdrGr64Base => "ldr",
        MirOP::LdrBGr64Base => "ldrb",
        MirOP::LdrHGr64Base => "ldrh",
        MirOP::LdrSBGr64Base => "ldrsb",
        MirOP::LdrSHGr64Base => "ldrsh",
        MirOP::StrGr64Base => "str",
        MirOP::StrBGr64Base => "strb",
        MirOP::StrHGr64Base => "strh",

        // 32位加载/存储指令（基址偏移寻址）
        MirOP::LdrGr32Base => "ldr",
        MirOP::LdrBGr32Base => "ldrb",
        MirOP::LdrHGr32Base => "ldrh",
        MirOP::LdrSBGr32Base => "ldrsb",
        MirOP::LdrSHGr32Base => "ldrsh",
        MirOP::StrGr32Base => "str",
        MirOP::StrBGr32Base => "strb",
        MirOP::StrHGr32Base => "strh",

        // 浮点加载/存储指令（基址偏移寻址）
        MirOP::LdrF64Base => "ldr",
        MirOP::StrF64Base => "str",
        MirOP::LdrF32Base => "ldr",
        MirOP::StrF32Base => "str",

        // 64位加载/存储指令（索引寻址）
        MirOP::LdrGr64Indexed => "ldr",
        MirOP::LdrBGr64Indexed => "ldrb",
        MirOP::LdrHGr64Indexed => "ldrh",
        MirOP::LdrSBGr64Indexed => "ldrsb",
        MirOP::LdrSHGr64Indexed => "ldrsh",
        MirOP::StrGr64Indexed => "str",
        MirOP::StrBGr64Indexed => "strb",
        MirOP::StrHGr64Indexed => "strh",

        // 32位加载/存储指令（索引寻址）
        MirOP::LdrGr32Indexed => "ldr",
        MirOP::LdrBGr32Indexed => "ldrb",
        MirOP::LdrHGr32Indexed => "ldrh",
        MirOP::LdrSBGr32Indexed => "ldrsb",
        MirOP::LdrSHGr32Indexed => "ldrsh",
        MirOP::StrGr32Indexed => "str",
        MirOP::StrBGr32Indexed => "strb",
        MirOP::StrHGr32Indexed => "strh",

        // 浮点加载/存储指令（索引寻址）
        MirOP::LdrF64Indexed => "ldr",
        MirOP::StrF64Indexed => "str",
        MirOP::LdrF32Indexed => "ldr",
        MirOP::StrF32Indexed => "str",

        // 64位加载/存储指令（字面量寻址）
        MirOP::LdrGr64Literal => "ldr",
        MirOP::LdrBGr64Literal => "ldrb",
        MirOP::LdrHGr64Literal => "ldrh",
        MirOP::LdrSBGr64Literal => "ldrsb",
        MirOP::LdrSHGr64Literal => "ldrsh",
        MirOP::StrGr64Literal => "str",
        MirOP::StrBGr64Literal => "strb",
        MirOP::StrHGr64Literal => "strh",

        // 32位加载/存储指令（字面量寻址）
        MirOP::LdrGr32Literal => "ldr",
        MirOP::LdrBGr32Literal => "ldrb",
        MirOP::LdrHGr32Literal => "ldrh",
        MirOP::LdrSBGr32Literal => "ldrsb",
        MirOP::LdrSHGr32Literal => "ldrsh",
        MirOP::StrGr32Literal => "str",
        MirOP::StrBGr32Literal => "strb",
        MirOP::StrHGr32Literal => "strh",

        // 浮点加载/存储指令（字面量寻址）
        MirOP::LdrF64Literal => "ldr",
        MirOP::StrF64Literal => "str",
        MirOP::LdrF32Literal => "ldr",
        MirOP::StrF32Literal => "str",

        // 常量加载指令
        MirOP::LoadConst64 => "mir.loadconst",
        MirOP::LoadConstF64 => "mir.loadconst",
        MirOP::LoadConst64Symbol => "mir.loadconst",

        // 条件选择指令
        MirOP::CSel64 | MirOP::CSel32 => "csel",
        MirOP::CSInc64 | MirOP::CSInc32 => "csinc",
        MirOP::CSInv64 | MirOP::CSInv32 => "csinv",
        MirOP::CSNeg64 | MirOP::CSNeg32 => "csneg",
        MirOP::CSelF64 | MirOP::CSelF32 => "fcsel",
        MirOP::CSet64 | MirOP::CSet32 => "cset",

        // 特殊MIR伪指令（手动实现）
        MirOP::MirCall => "call",
        MirOP::MirReturn => "return",
        MirOP::MirSwitch => "switch",
    }
}

/// 获取MIR操作码的字符串名称（返回拥有的字符串）
pub fn opcode_to_name_string(opcode: MirOP) -> String {
    opcode_get_name_str(opcode).to_string()
}

use std::{collections::HashMap, fmt::Display, str::FromStr};

#[rustfmt::skip]
#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Enum representing the various opcodes in the Remusys IR.
pub enum Opcode {
    None,
    BitAnd, BitOr, BitXor, Shl, Lshr, Ashr,
    Add, Sub, Mul, Sdiv, Udiv, Srem, Urem,
    Fadd, Fsub, Fmul, Fdiv, Frem,
    Jmp, Br, Switch, Ret, Unreachable,
    Sitofp, Uitofp, Fptosi, Fptoui, Zext, Sext, Trunc, Fpext, Fptrunc,
    Bitcast, IntToPtr, PtrToInt,
    Select, IndexExtract, FieldExtract, IndexInsert, FieldInsert, IndexPtr, IndexOffsetOf,
    Load, Store, Alloca, DynAlloca,
    Call, DynCall, Phi,
    Icmp, Fcmp,
    AmoXchg, AmoAdd, AmoSub, AmoAnd, AmoNand, AmoOr, AmoXor,
    AmoSMax, AmoSMin, AmoUMax, AmoUMin,
    AmoFAdd, AmoFSub, AmoFMax, AmoFMin,
    AmoUIncWrap, AmoUDecWrap, AmoUSubCond, AmoUSubStat,
    ConstArray, ConstStruct, ConstVec, ConstPtrNull,
    Intrin, GuideNode, PhiEnd, ReservedCnt,
}

impl Opcode {
    pub fn is_shift_op(self) -> bool {
        matches!(self, Opcode::Shl | Opcode::Lshr | Opcode::Ashr)
    }
    pub fn is_logic_op(self) -> bool {
        matches!(self, Opcode::BitAnd | Opcode::BitOr | Opcode::BitXor)
    }
    pub fn is_int_op(self) -> bool {
        use Opcode::*;
        matches!(
            self,
            BitAnd | BitOr | BitXor | Add | Sub | Mul | Sdiv | Udiv | Srem | Urem
        )
    }
    pub fn is_float_op(self) -> bool {
        matches!(
            self,
            Opcode::Fadd | Opcode::Fsub | Opcode::Fmul | Opcode::Fdiv | Opcode::Frem
        )
    }
    pub fn is_binary_op(self) -> bool {
        use Opcode::*;
        #[rustfmt::skip]
        return matches!(
            self,
            BitAnd | BitOr | BitXor | Add | Sub | Mul | Sdiv | Udiv
            | Srem | Urem | Fadd | Fsub | Fmul | Fdiv | Frem
        );
    }
    pub fn is_divrem_op(self) -> bool {
        matches!(
            self,
            Opcode::Sdiv | Opcode::Udiv | Opcode::Srem | Opcode::Urem | Opcode::Frem | Opcode::Fdiv
        )
    }
    pub fn is_constexpr_op(self) -> bool {
        use Opcode::*;
        matches!(
            self,
            ConstArray | ConstStruct | ConstVec | ConstPtrNull | IndexOffsetOf
        )
    }
    pub fn is_inst_op(self) -> bool {
        use Opcode::*;
        !matches!(
            self,
            ConstArray | ConstStruct | ConstVec | ConstPtrNull | IndexOffsetOf
        )
    }
    pub fn is_cast_op(self) -> bool {
        use Opcode::*;
        matches!(
            self,
            Sitofp | Uitofp | Fptosi | Zext | Sext | Trunc | Fpext | Fptrunc
        )
    }

    pub fn get_name(self) -> &'static str {
        if self as usize >= Opcode::ReservedCnt as usize {
            "<undefined-opcode>"
        } else {
            OPCODE_NAMES[self as usize]
        }
    }

    fn init_name_map() -> HashMap<&'static str, Opcode> {
        let mut m = HashMap::new();
        for (i, &name) in OPCODE_NAMES.iter().enumerate() {
            m.insert(name, unsafe { std::mem::transmute::<u8, Opcode>(i as u8) });
        }
        m
    }

    pub fn get_kind(self) -> InstKind {
        use Opcode::*;
        match self {
            // Guide node
            Opcode::GuideNode => InstKind::ListGuideNode,
            Opcode::PhiEnd => InstKind::PhiInstEnd,

            // Phi instruction
            Opcode::Phi => InstKind::Phi,

            // Terminator instructions
            Opcode::Unreachable => InstKind::Unreachable,
            Opcode::Ret => InstKind::Ret,
            Opcode::Jmp => InstKind::Jump,
            Opcode::Br => InstKind::Br,
            Opcode::Switch => InstKind::Switch,

            // Memory operations
            Opcode::Alloca | Opcode::DynAlloca => InstKind::Alloca,
            Opcode::Load => InstKind::Load,
            Opcode::Store => InstKind::Store,

            // Selection and indexing
            Opcode::Select => InstKind::Select,
            Opcode::IndexPtr | Opcode::IndexExtract | Opcode::IndexInsert => InstKind::IndexPtr,
            Opcode::FieldExtract | Opcode::FieldInsert => InstKind::FieldOp,

            // Function calls
            Opcode::Call | Opcode::DynCall => InstKind::Call,
            Opcode::Intrin => InstKind::Intrin,

            // Binary operations (arithmetic and logical)
            BitAnd | BitOr | BitXor | Shl | Lshr | Ashr | Add | Sub | Mul | Sdiv | Udiv | Srem
            | Urem | Fadd | Fsub | Fmul | Fdiv | Frem => InstKind::BinOp,

            // Comparison operations
            Opcode::Icmp | Opcode::Fcmp => InstKind::Cmp,

            // Cast operations
            Sitofp | Uitofp | Fptosi | Fptoui | Zext | Sext | Trunc | Fpext | Fptrunc | Bitcast
            | IntToPtr | PtrToInt => InstKind::Cast,

            // Atomic read-modify-write operations
            AmoXchg | AmoAdd | AmoSub | AmoAnd | AmoNand | AmoOr | AmoXor | AmoSMax | AmoSMin
            | AmoUMax | AmoUMin | AmoFAdd | AmoFSub | AmoFMax | AmoFMin | AmoUIncWrap
            | AmoUDecWrap | AmoUSubCond | AmoUSubStat => InstKind::AmoRmw,

            // Special cases for undefined or reserved opcodes
            Opcode::None => panic!("Opcode::None does not have a kind"),
            Opcode::ReservedCnt => panic!("Opcode::ReservedCnt does not have a kind"),

            // Constant expressions - these might need special handling
            // For now, treating them as binary operations since they can appear in constant expressions
            Opcode::ConstArray | Opcode::ConstStruct | Opcode::ConstVec | Opcode::ConstPtrNull => {
                panic!("Constant opcodes should not be used in instructions")
            }
            Opcode::IndexOffsetOf => InstKind::IndexPtr,
        }
    }
}
impl Display for Opcode {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.get_name())
    }
}
impl FromStr for Opcode {
    type Err = ();
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        OPCODE_NAME_MAP.get(s).copied().ok_or(())
    }
}
impl From<u8> for Opcode {
    fn from(value: u8) -> Self {
        if (Self::None as usize..Self::ReservedCnt as usize).contains(&(value as usize)) {
            unsafe { std::mem::transmute::<u8, Opcode>(value) }
        } else {
            Self::None
        }
    }
}

#[rustfmt::skip]
static OPCODE_NAMES: [&str; Opcode::ReservedCnt as usize] = [
    "<undefined>",
    "and", "or", "xor", "shl",  "lshr", "ashr",
    "add", "sub", "mul", "sdiv", "udiv", "srem", "urem",
    "fadd", "fsub", "fmul", "fdiv", "frem",
    "jmp", "br", "switch", "ret", "unreachable",
    "sitofp", "uitofp", "fptosi", "fptoui", "zext", "sext", "trunc", "fpext", "fptrunc",
    "bitcast", "inttoptr", "ptrtoint",
    "select", "extractelement", "extractvalue", "insertelement", "insertvalue", "getelementptr", "offsetof",
    "load", "store", "alloca", "dyn-alloca",
    "call", "dyncall", "phi",
    "icmp", "fcmp",

    "amo.xchg", "amo.add", "amo.sub", "amo.and", "amo.nand", "amo.or", "amo.xor",
    "amo.max", "amo.min", "amo.umax", "amo.umin",
    "amo.fadd", "amo.fsub", "amo.fmax", "amo.fmin",
    "amo.uinc_wrap", "amo.udec_wrap", "amo.usub_cond", "amo.sub_stat",

    "constarray", "conststruct", "constvec",
    "constptrnull",
    "intrin", "guide-node", "phi-end",
];

lazy_static::lazy_static! {
    static ref OPCODE_NAME_MAP: HashMap<&'static str, Opcode> = Opcode::init_name_map();
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InstKind {
    /// Instruction list guide node containing a simple header and parent block.
    /// The guide node will be always attached to a block, so its parent block
    /// will be initialized when the block is allocated on `module.inner._alloc_block`.
    ListGuideNode,
    PhiInstEnd,

    // Terminator instructions. These instructions are put at the end of a block and
    // transfer control to another block or return from a function.
    Unreachable,
    Ret,
    Jump,
    Br,
    Switch,

    // Non-terminator instructions. These instructions are put in the middle of a block
    // and do not transfer control to another block or return from a function.
    Phi,
    Alloca,
    Load,
    Store,
    Select,
    BinOp,
    Cmp,
    Cast,
    IndexPtr,
    FieldOp,
    Call,
    AmoRmw,
    Intrin,
}

impl InstKind {
    pub fn is_terminator(self) -> bool {
        matches!(
            self,
            InstKind::Unreachable
                | InstKind::Ret
                | InstKind::Jump
                | InstKind::Br
                | InstKind::Switch
        )
    }
    pub fn is_guide_node(self) -> bool {
        self == InstKind::ListGuideNode || self == InstKind::PhiInstEnd
    }
    pub fn is_normal(self) -> bool {
        !self.is_terminator() && !self.is_guide_node()
    }
}

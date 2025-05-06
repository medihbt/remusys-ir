use std::{collections::HashMap, str::FromStr};

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
/// Enum representing the various opcodes in the Musys IR.
pub enum Opcode {
    None,
    And, Or, Xor, Shl, Lshr, Ashr,
    Add, Sub, Mul, Sdiv, Udiv, Srem, Urem,
    Fadd, Fsub, Fmul, Fdiv, Frem,
    Jmp, Br, Switch, Ret, Unreachable,
    Sitofp, Uitofp, Fptosi, Zext, Sext, Trunc, Fpext, Fptrunc,
    Bitcast, IntToPtr, PtrToInt,
    Select, IndexExtract, IndexInsert, IndexPtr, IndexOffsetOf,
    Load, Store, Alloca, DynAlloca,
    Call, DynCall, Phi,
    Icmp, Fcmp,
    ConstArray, ConstStruct, ConstVec, ConstPtrNull,
    Intrin, ReservedCnt,
}

impl Opcode {
    pub fn is_shift_op(self) -> bool {
        matches!(self, Opcode::Shl | Opcode::Lshr | Opcode::Ashr)
    }
    pub fn is_logic_op(self) -> bool {
        matches!(self, Opcode::And | Opcode::Or | Opcode::Xor)
    }
    pub fn is_int_op(self) -> bool {
        matches!(self, Opcode::And | Opcode::Or | Opcode::Xor | Opcode::Add | Opcode::Sub | Opcode::Mul | Opcode::Sdiv | Opcode::Udiv | Opcode::Srem | Opcode::Urem)
    }
    pub fn is_float_op(self) -> bool {
        matches!(self, Opcode::Fadd | Opcode::Fsub | Opcode::Fmul | Opcode::Fdiv | Opcode::Frem)
    }
    pub fn is_binary_op(self) -> bool {
        matches!(self, Opcode::And | Opcode::Or | Opcode::Xor | Opcode::Add | Opcode::Sub | Opcode::Mul | Opcode::Sdiv | Opcode::Udiv | Opcode::Srem | Opcode::Urem)
    }
    pub fn is_divrem_op(self) -> bool {
        matches!(self, Opcode::Sdiv | Opcode::Udiv | Opcode::Srem | Opcode::Urem | Opcode::Frem | Opcode::Fdiv)
    }
    pub fn is_constexpr_op(self) -> bool {
        matches!(self, Opcode::And | Opcode::Or | Opcode::Xor | Opcode::Add | Opcode::Sub | Opcode::Mul | Opcode::Sdiv | Opcode::Udiv | Opcode::Srem | Opcode::Urem | Opcode::IndexExtract | Opcode::IndexInsert | Opcode::IndexPtr | Opcode::IndexOffsetOf)
    }
    pub fn is_inst_op(self) -> bool {
        !matches!(self, Opcode::IndexOffsetOf | Opcode::ConstArray | Opcode::ConstStruct | Opcode::ConstVec)
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
            m.insert(name, unsafe { std::mem::transmute(i as u8) });
        }
        m
    }
}
impl ToString for Opcode {
    fn to_string(&self) -> String {
        self.get_name().to_string()
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
        if (Self::None as usize .. Self::ReservedCnt as usize).contains(&(value as usize)) {
            unsafe { std::mem::transmute(value) }
        } else {
            Self::None
        }
    }
}

static OPCODE_NAMES: [&str; Opcode::ReservedCnt as usize] = [
    "<undefined>",
    "and", "or", "xor", "shl",  "lshr", "ashr",
    "add", "sub", "mul", "sdiv", "udiv", "srem", "urem",
    "fadd", "fsub", "fmul", "fdiv", "frem",
    "jmp", "br", "switch", "ret", "unreachable",
    "sitofp", "uitofp", "fptosi", "zext", "sext", "trunc", "fpext", "fptrunc",
    "bitcast", "inttoptr", "ptrtoint",
    "select", "extractelement", "insertelement", "getelementptr", "offsetof",
    "load", "store", "alloca", "dyn-alloca",
    "call", "dyncall", "phi",
    "icmp", "fcmp",
    "constarray", "conststruct", "constvec",
    "constptrnull",
    "intrin"
];

lazy_static::lazy_static! {
    static ref OPCODE_NAME_MAP: HashMap<&'static str, Opcode> = Opcode::init_name_map();
}

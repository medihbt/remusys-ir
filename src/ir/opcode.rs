//< Transformed from Vala Opcode definition:
/*
public enum MusysIR.OpCode {
    NONE,
    AND,  ORR,  XOR,  SHL,  LSHR, ASHR, 
    ADD,  SUB,  MUL,  SDIV, UDIV, SREM, UREM,
    FADD, FSUB, FMUL, FDIV, FREM,
    JMP, BR, SWITCH, RET, UNREACHABLE,
    INEG, FNEG, NOT,
    SITOFP, UITOFP, FPTOSI, ZEXT, SEXT, TRUNC, FPEXT, FPTRUNC,
    BITCAST, INT_TO_PTR, PTR_TO_INT,
    SELECT, INDEX_EXTRACT, INDEX_INSERT, INDEX_PTR, INDEX_OFFSET_OF,
    LOAD, STORE, ALLOCA, DYN_ALLOCA,
    CALL, DYN_CALL, PHI,
    ICMP, FCMP,
    CONST_ARRAY, CONST_STRUCT, CONST_VEC, CONST_PTR_NULL,
    INTRIN, RESERVED_CNT;

    public bool is_shift_op()  { return SHL  <= this <= ASHR; }
    public bool is_logic_op()  { return AND  <= this <= ASHR || this == NOT; }
    public bool is_int_op()    { return AND  <= this <= UREM; }
    public bool is_float_op()  { return FADD <= this <= FREM; }
    public bool is_binary_op() { return AND  <= this <= FREM; }
    public bool is_divrem_op() {
        return SDIV <= this <= UREM || this == FREM || this == FDIV;
    }
    public bool is_constexpr_op() {
        return (AND <= this <= FREM) || (INDEX_EXTRACT <= this <= INDEX_OFFSET_OF);
    }
    public bool is_inst_op() {
        return this != INDEX_OFFSET_OF && !(CONST_ARRAY <= this <= CONST_VEC);
    }
    public unowned string get_name() {
        return this >= RESERVED_CNT?
                "<undefined-opcode>":
                _instruction_opcode_names[this];
    }
} // public enum OpCode

private unowned string _instruction_opcode_names[MusysIR.OpCode.RESERVED_CNT] = {
    "<undefined>",
    "and", "or", "xor", "shl",  "lshr", "ashr", 
    "add", "sub", "mul", "sdiv", "udiv", "srem", "urem",
    "fadd", "fsub", "fmul", "fdiv", "frem",
    "br", "br", "switch", "ret", "unreachable",
    "ineg", "fneg", "not",
    "sitofp", "uitofp", "fptosi", "zext", "sext", "trunc", "fpext", "fptrunc",
    "bitcast", "inttoptr", "ptrtoint",
    "select", "extractelement", "insertelement", "getelementptr", "offsetof",
    "load", "store", "alloca", "dyn-alloca", "call", "dyncall", "phi",
    "icmp", "fcmp",
    "constarray", "conststruct", "constvec",
    "intrin"
};

*/

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


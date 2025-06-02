pub enum MachineOpcode {
    // Data processing instructions
    Add,
    Sub,
    Mul,
    Div,
    And,
    Or,
    Xor,
    Lsl, // Logical shift left
    Lsr, // Logical shift right
    Asr, // Arithmetic shift right
    Ror, // Rotate right

    // Load/store instructions
    Ldr, // Load register
    Str, // Store register
    Ldp, // Load pair of registers
    Stp, // Store pair of registers

    // Branch instructions
    B,   // Branch
    Bl,  // Branch with link (function call)
    Br,  // Branch to register
    Cbz, // Compare and branch on zero
    Cbnz,// Compare and branch on non-zero

    // Comparison instructions
    Cmp, // Compare
    Tst, // Test bits

    // Miscellaneous instructions
    Nop, // No operation
}
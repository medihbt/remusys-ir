//! Since Remusys-lang frontend is not yet available, we build all our cases
//! in this module.

use crate::{
    base::APInt,
    ir::{
        CmpCond, IRBuilder, IRBuilderFocus, Module, Opcode, ValueSSA,
        inst::{ISubInstRef, RetRef},
        write_ir_module, write_ir_module_quiet,
    },
    typing::{ArchInfo, FuncTypeRef, TypeContext, ValTypeID},
};

/// Test case 1: CFG example with a lot of branches.
///
/// ```SysY
/// extern int getint();
///
/// int main() {
///     int a = getint();       // a = getint(), SSA
///     int b = getint();       // b = getint(), SSA
///
///     int c = a + b;          // c mutable
///     while (c < 75) {
///         int d = 42;         // d = 42, SSA
///         if (c < 100) {
///             c = c + d;
///             if (c > 99) {
///                 int e = d * 2; // e = 84, SSA
///                 if (getint() == 1) {
///                     c = e * 2;
///                 }
///             }
///         }
///     }
///     return c;
/// }
/// ```
///
///
/// ``` remusys-ir
/// declare i32 @getint()
///
/// define dso_local i32 @main() {
/// 0:
///     %1 = alloca i32, align 4    ; c = %1
///     %2 = call i32 @getint()     ; a = %2
///     %3 = call i32 @getint()     ; b = %3
///     %4 = add nsw i32 %2, %3
///     store i32 %4, ptr %1, align 4
///     br label %5
///
/// 5: ; while (c < 75)
///     %6 = load i32, ptr %1, align 4
///     %7 = icmp slt i32 %6, 75
///     br i1 %7, label %8, label %19
///
/// 8: ; if (c < 100)
///     %9 = load i32, ptr %1, align 4
///     %10 = icmp slt i32 %9, 100
///     br i1 %10, label %11, label %5
///
/// 11: ; c = c + d // c = c + 42
///     %12 = load i32, ptr %1, align 4
///     %13 = add nsw i32 %12, 42
///     store i32 %13, ptr %1, align 4
///
///     ; if (c > 99)
///     %14 = icmp sgt i32 %13, 99
///     br i1 %14, label %15, label %5
///
/// 15: ; if (getint() == 1)
///     %16 = call i32 @getint()
///     %17 = icmp eq i32 %16, 1
///     br i1 %17, label %18, label %5
///
/// 18: ; c = e * 2 // c = 84 * 2 = 168
///     store i32 168, ptr %1, align 4
///     br label %5
///
/// 19: ; return c
///     %20 = load i32, ptr %1, align 4
///     ret i32 %20
/// }
/// ```
#[allow(unused)]
pub fn test_case_cfg_deep_while_br() -> IRBuilder {
    let mut builder = create_module_builder("test_case_cfg_deep_while_br");
    let ri32fty = FuncTypeRef::new(builder.get_type_ctx(), ValTypeID::Int(32), false, []);

    let getint_func = builder.declare_function("getint", ri32fty).unwrap();
    let _main_func = builder
        .define_function_with_unreachable("main", ri32fty)
        .unwrap();

    // set builder current focus to: `Block(main() -> block %0)`
    let entry_block_0 = builder.get_focus_full().block;
    builder.set_focus(IRBuilderFocus::Block(entry_block_0));
    let (_, ret_inst) = builder.focus_set_return(APInt::new(0, 32).into()).unwrap();
    let ret_inst = RetRef::from_raw_nocheck(ret_inst);

    let alloca_c_1 = builder.add_alloca_inst(ValTypeID::Int(32), 2).unwrap();
    let call_a_2 = builder.add_call_inst(getint_func, [].into_iter()).unwrap();
    let call_b_3 = builder.add_call_inst(getint_func, [].into_iter()).unwrap();

    let add_4 = builder
        .add_binop_inst(
            Opcode::Add,
            ValueSSA::Inst(call_a_2),
            ValueSSA::Inst(call_b_3),
        )
        .unwrap();
    builder
        .add_store_inst(ValueSSA::Inst(alloca_c_1), ValueSSA::Inst(add_4), 4)
        .unwrap();

    let final_block_19 = builder.split_current_block_from_terminator().unwrap();
    let while_block_5 = builder.split_current_block_from_terminator().unwrap();

    // set up the final block
    builder.set_focus(IRBuilderFocus::Block(final_block_19));
    let load_20 = builder
        .add_load_inst(ValTypeID::Int(32), 4, ValueSSA::Inst(alloca_c_1))
        .unwrap();

    // set up the while block
    builder.set_focus(IRBuilderFocus::Block(while_block_5));
    // Create a loop
    builder.focus_set_jump_to(while_block_5).unwrap();
    // seperate the header and body
    let while_body_block_8 = builder.split_current_block_from_terminator().unwrap();

    let load_6 = builder
        .add_load_inst(ValTypeID::Int(32), 4, ValueSSA::Inst(alloca_c_1))
        .unwrap();
    let icmp_7 = builder
        .add_cmp_inst(
            CmpCond::LT | CmpCond::SIGNED_ORDERED,
            ValueSSA::Inst(load_6),
            APInt::new(75, 32).into(),
        )
        .unwrap();
    builder
        .focus_set_branch_to(ValueSSA::Inst(icmp_7), while_body_block_8, final_block_19)
        .unwrap();

    // set up the while body block
    builder.set_focus(IRBuilderFocus::Block(while_body_block_8));
    // if (c < 100)
    let if_block_11 = builder.split_current_block_from_terminator().unwrap();
    let load_9 = builder
        .add_load_inst(ValTypeID::Int(32), 4, ValueSSA::Inst(alloca_c_1))
        .unwrap();
    let icmp_10 = builder
        .add_cmp_inst(
            CmpCond::LT | CmpCond::SIGNED_ORDERED,
            ValueSSA::Inst(load_9),
            APInt::new(100, 32).into(),
        )
        .unwrap();
    builder
        .focus_set_branch_to(ValueSSA::Inst(icmp_10), if_block_11, while_block_5)
        .unwrap();

    // set up the if block
    builder.set_focus(IRBuilderFocus::Block(if_block_11));
    // c = c + d
    let load_12 = builder
        .add_load_inst(ValTypeID::Int(32), 4, ValueSSA::Inst(alloca_c_1))
        .unwrap();
    let add_13 = builder
        .add_binop_inst(
            Opcode::Add,
            ValueSSA::Inst(load_12),
            APInt::new(42, 32).into(),
        )
        .unwrap();
    builder
        .add_store_inst(ValueSSA::Inst(alloca_c_1), ValueSSA::Inst(add_13), 4)
        .unwrap();

    // if (c > 99)
    let if_block_15 = builder.split_current_block_from_terminator().unwrap();
    let icmp_14 = builder
        .add_cmp_inst(
            CmpCond::GT | CmpCond::SIGNED_ORDERED,
            ValueSSA::Inst(add_13),
            APInt::new(99, 32).into(),
        )
        .unwrap();
    builder
        .focus_set_branch_to(ValueSSA::Inst(icmp_14), if_block_15, while_block_5)
        .unwrap();

    // set up the if block
    builder.set_focus(IRBuilderFocus::Block(if_block_15));
    // if (getint() == 1)
    let if_block_18 = builder.split_current_block_from_terminator().unwrap();
    let call_16 = builder.add_call_inst(getint_func, [].into_iter()).unwrap();
    let icmp_17 = builder
        .add_cmp_inst(
            CmpCond::EQ | CmpCond::SIGNED_ORDERED,
            ValueSSA::Inst(call_16),
            APInt::new(1, 32).into(),
        )
        .unwrap();
    builder
        .focus_set_branch_to(ValueSSA::Inst(icmp_17), if_block_18, while_block_5)
        .unwrap();
    // set up the if block
    builder.set_focus(IRBuilderFocus::Block(if_block_18));
    // c = e * 2
    builder
        .add_store_inst(ValueSSA::Inst(alloca_c_1), APInt::new(168, 32).into(), 4)
        .unwrap();
    builder.focus_set_jump_to(while_block_5).unwrap();
    builder.module.gc_cleaner().compact([]);
    builder
}

pub fn create_module_builder(name: &str) -> IRBuilder {
    let host_platform = ArchInfo::new_host();
    let type_ctx = TypeContext::new_rc(host_platform);
    let builder = IRBuilder::new(Module::new(name.into(), type_ctx));
    builder
}

#[allow(unused)]
pub fn write_ir_to_file(module: &Module, filename: &str) {
    let filepath = format!("target/{}.ll", filename);
    let mut file = match std::fs::File::create(&filepath) {
        Ok(f) => f,
        Err(e) => {
            panic!("Failed to create file {filepath}: {e}")
        }
    };
    write_ir_module(module, &mut file);
}

pub fn write_ir_to_file_quiet(module: &Module, filename: &str) {
    let filepath = format!("target/{}.ll", filename);
    let mut file = match std::fs::File::create(&filepath) {
        Ok(f) => f,
        Err(e) => {
            panic!("Failed to create file {filepath}: {e}")
        }
    };
    write_ir_module_quiet(module, &mut file);
}

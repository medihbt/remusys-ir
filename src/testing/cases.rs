use crate::{
    base::APInt,
    ir::{inst::*, *},
    typing::*,
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
    let mut builder = IRBuilder::new_inlined(ArchInfo::new_host(), "test_case_cfg_deep_while_br");
    let ri32fty = FuncTypeID::new(builder.tctx(), ValTypeID::Int(32), false, []);
    // extern i32 @getint()
    let getint_func = FuncID::builder(builder.tctx(), "getint", ri32fty)
        .make_extern()
        .build_id(&builder.module)
        .unwrap();
    // main: define dso_local i32 @main() { ... }
    let main_func = FuncID::builder(builder.tctx(), "main", ri32fty)
        .make_defined()
        .terminate_mode(FuncTerminateMode::ReturnDefault)
        .add_attr(Attribute::NoUndef)
        .build_id(&builder.module)
        .unwrap();

    // Focus entry block of main
    let entry_block = main_func.get_entry(builder.allocs()).unwrap();
    builder.set_focus(IRFocus::Block(entry_block));

    // %1 = alloca i32, align 4    ; c
    let alloca_c = AllocaInstID::new(builder.allocs(), ValTypeID::Int(32), 2);
    builder.insert_inst(alloca_c).unwrap();

    // %2 = call i32 @getint()     ; a = %2
    let call_a = builder
        .build_inst(|allocs, tctx| {
            let mut cb = CallInst::builder(tctx, ri32fty);
            cb.callee(ValueSSA::Global(getint_func.raw_into()));
            let inst = cb.build_obj(allocs);
            CallInstID::allocate(allocs, inst).raw_into()
        })
        .unwrap();

    // %3 = call i32 @getint()     ; b = %3
    let call_b = builder
        .build_inst(|allocs, tctx| {
            let mut cb = CallInst::builder(tctx, ri32fty);
            cb.callee(ValueSSA::Global(getint_func.raw_into()));
            let inst = cb.build_obj(allocs);
            CallInstID::allocate(allocs, inst).raw_into()
        })
        .unwrap();

    // %4 = add i32 %2, %3
    let add_2_3 = BinOPInstID::new(
        builder.allocs(),
        Opcode::Add,
        ValueSSA::Inst(call_a),
        ValueSSA::Inst(call_b),
    );
    add_2_3.add_flags(builder.allocs(), BinOPFlags::NSW);
    builder.insert_inst(add_2_3).unwrap();

    // store i32 %4, ptr %1, align 4
    let store_init_c = StoreInstID::new(
        builder.allocs(),
        ValueSSA::Inst(add_2_3.raw_into()),
        ValueSSA::Inst(alloca_c.raw_into()),
        2,
    );
    builder.insert_inst(store_init_c).unwrap();

    // Split entry twice to create: entry -> while_header(%5) -> final(%19)
    let final_block = builder.split_block().unwrap();
    let while_header = builder.split_block().unwrap();

    // Final block: %20 = load i32, ptr %1, align 4; ret i32 %20
    builder.set_focus(IRFocus::Block(final_block));
    let load_ret = {
        let load = LoadInstID::new_uninit(builder.allocs(), ValTypeID::Int(32), 2);
        load.set_source(builder.allocs(), ValueSSA::Inst(alloca_c.raw_into()));
        builder.insert_inst(load).unwrap();
        load
    };
    builder
        .focus_set_terminator(RetInstID::with_retval(
            builder.allocs(),
            ValueSSA::Inst(load_ret.raw_into()),
        ))
        .unwrap();

    // While header (%5): create loop skeleton 5 -> 8 -> 5
    builder.set_focus(IRFocus::Block(while_header));
    builder.focus_set_jump_to(while_header).unwrap();
    let while_body = builder.split_block().unwrap();

    // %6 = load i32, ptr %1, align 4
    let load_c_6 = {
        let load = LoadInstID::new_uninit(builder.allocs(), ValTypeID::Int(32), 2);
        load.set_source(builder.allocs(), ValueSSA::Inst(alloca_c.raw_into()));
        builder.insert_inst(load).unwrap();
        load
    };
    // %7 = icmp slt i32 %6, 75
    let icmp_7 = {
        let cmp = CmpInstID::new_uninit(
            builder.allocs(),
            Opcode::Icmp,
            CmpCond::SLT,
            ValTypeID::Int(32),
        );
        cmp.set_lhs(builder.allocs(), ValueSSA::Inst(load_c_6.raw_into()));
        cmp.set_rhs(builder.allocs(), APInt::new(75u32, 32).into());
        builder.insert_inst(cmp).unwrap();
        cmp
    };
    // br i1 %7, label %8, label %19
    builder
        .focus_set_branch_to(ValueSSA::Inst(icmp_7.raw_into()), while_body, final_block)
        .unwrap();

    // While body (%8): if (c < 100) then goto %11 else %5
    builder.set_focus(IRFocus::Block(while_body));
    let if_block_11 = builder.split_block().unwrap();
    let load_c_9 = {
        let load = LoadInstID::new_uninit(builder.allocs(), ValTypeID::Int(32), 2);
        load.set_source(builder.allocs(), ValueSSA::Inst(alloca_c.raw_into()));
        builder.insert_inst(load).unwrap();
        load
    };
    let icmp_10 = {
        let cmp = CmpInstID::new_uninit(
            builder.allocs(),
            Opcode::Icmp,
            CmpCond::SLT,
            ValTypeID::Int(32),
        );
        cmp.set_lhs(builder.allocs(), ValueSSA::Inst(load_c_9.raw_into()));
        cmp.set_rhs(builder.allocs(), APInt::new(100u32, 32).into());
        builder.insert_inst(cmp).unwrap();
        cmp
    };
    builder
        .focus_set_branch_to(
            ValueSSA::Inst(icmp_10.raw_into()),
            if_block_11,
            while_header,
        )
        .unwrap();

    // If block (%11): c = c + 42
    builder.set_focus(IRFocus::Block(if_block_11));
    let load_c_12 = {
        let load = LoadInstID::new_uninit(builder.allocs(), ValTypeID::Int(32), 2);
        load.set_source(builder.allocs(), ValueSSA::Inst(alloca_c.raw_into()));
        builder.insert_inst(load).unwrap();
        load
    };
    let add_13 = BinOPInstID::new(
        builder.allocs(),
        Opcode::Add,
        ValueSSA::Inst(load_c_12.raw_into()),
        APInt::new(42u32, 32).into(),
    );
    add_13.add_flags(builder.allocs(), BinOPFlags::NSW);
    builder.insert_inst(add_13).unwrap();
    let store_c_13 = StoreInstID::new(
        builder.allocs(),
        ValueSSA::Inst(add_13.raw_into()),
        ValueSSA::Inst(alloca_c.raw_into()),
        2,
    );
    builder.insert_inst(store_c_13).unwrap();

    // if (c > 99) then %15 else %5
    let if_block_15 = builder.split_block().unwrap();
    let icmp_14 = {
        let cmp = CmpInstID::new_uninit(
            builder.allocs(),
            Opcode::Icmp,
            CmpCond::SGT,
            ValTypeID::Int(32),
        );
        cmp.set_lhs(builder.allocs(), ValueSSA::Inst(add_13.raw_into()));
        cmp.set_rhs(builder.allocs(), APInt::new(99u32, 32).into());
        builder.insert_inst(cmp).unwrap();
        cmp
    };
    builder
        .focus_set_branch_to(
            ValueSSA::Inst(icmp_14.raw_into()),
            if_block_15,
            while_header,
        )
        .unwrap();

    // If block (%15): if (getint() == 1) then %18 else %5
    builder.set_focus(IRFocus::Block(if_block_15));
    let if_block_18 = builder.split_block().unwrap();
    let call_16 = builder
        .build_inst(|allocs, tctx| {
            let mut cb = CallInst::builder(tctx, ri32fty);
            cb.callee(ValueSSA::Global(getint_func.raw_into()));
            let inst = cb.build_obj(allocs);
            CallInstID::allocate(allocs, inst).raw_into()
        })
        .unwrap();
    let icmp_17 = {
        let cmp = CmpInstID::new_uninit(
            builder.allocs(),
            Opcode::Icmp,
            CmpCond::EQ | CmpCond::SIGNED_ORDERED,
            ValTypeID::Int(32),
        );
        cmp.set_lhs(builder.allocs(), ValueSSA::Inst(call_16));
        cmp.set_rhs(builder.allocs(), APInt::new(1u32, 32).into());
        builder.insert_inst(cmp).unwrap();
        cmp
    };
    builder
        .focus_set_branch_to(
            ValueSSA::Inst(icmp_17.raw_into()),
            if_block_18,
            while_header,
        )
        .unwrap();

    // If block (%18): c = 168; br %5
    builder.set_focus(IRFocus::Block(if_block_18));
    let store_168 = StoreInstID::new(
        builder.allocs(),
        APInt::new(168u32, 32).into(),
        ValueSSA::Inst(alloca_c.raw_into()),
        2,
    );
    builder.insert_inst(store_168).unwrap();
    builder.focus_set_jump_to(while_header).unwrap();

    builder
}

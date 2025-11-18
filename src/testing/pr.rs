#![cfg(test)]
use crate::{
    base::*,
    ir::{inst::*, *},
    typing::*,
};

#[test]
fn test_builder_smoke_and_writer() {
    let mut builder = IRBuilder::new_inlined(ArchInfo::new_host(), "builder_smoke");
    let ri32fty = FuncTypeID::new(builder.tctx(), ValTypeID::Int(32), false, []);

    let main_func = FuncID::builder(builder.tctx(), "main", ri32fty)
        .make_defined()
        .terminate_mode(FuncTerminateMode::ReturnDefault)
        .build_id(&builder.module)
        .unwrap();

    let entry = main_func.get_entry(builder.allocs()).unwrap();
    builder.set_focus(IRFocus::Block(entry));

    let add = BinOPInstID::new(
        builder.allocs(),
        Opcode::Add,
        APInt::new(1u32, 32).into(),
        APInt::new(2u32, 32).into(),
    );
    builder.insert_inst(add).unwrap();
    builder
        .focus_set_terminator(RetInstID::with_retval(
            builder.allocs(),
            ValueSSA::Inst(add.raw_into()),
        ))
        .unwrap();

    // write to memory
    let mut buf = Vec::<u8>::new();
    let mut writer = IRWriter::from_module(&mut buf, &builder.module);
    writer.option = IRWriteOption::quiet();
    writer.write_module();
    let s = String::from_utf8(buf).unwrap();
    assert!(s.contains("define dso_local i32 @main"));
}

#[test]
fn test_gc_unreachable_expr_is_freed() {
    let builder = IRBuilder::new_inlined(ArchInfo::new_host(), "gc_case");

    // 构造一个未被引用的表达式（数组常量）
    let elem_ty = ValTypeID::Int(32);
    let arr_ty = ArrayTypeID::new(builder.tctx(), elem_ty, 2);
    let arr = ArrayExprID::new_uninit(builder.allocs(), builder.tctx(), arr_ty);
    let elems = arr.get_elems(builder.allocs());
    elems[0].set_operand(builder.allocs(), APInt::new(0u32, 32).into());
    elems[1].set_operand(builder.allocs(), APInt::new(1u32, 32).into());
    // 记录 GC 前的 expr 分配数
    let pre_expr_len = builder.module.allocs.exprs.len();

    // 同时构造一个活体函数放在符号表，避免整模块清空
    let ri32fty = FuncTypeID::new(builder.tctx(), ValTypeID::Int(32), false, []);
    let _ = FuncID::builder(builder.tctx(), "main", ri32fty)
        .make_defined()
        .terminate_mode(FuncTerminateMode::ReturnDefault)
        .build_id(&builder.module)
        .unwrap();
    let mut module = builder.module;
    module.begin_gc().finish();
    let post_expr_len = module.allocs.exprs.len();
    assert!(
        post_expr_len < pre_expr_len,
        "GC should free unreachable const exprs: before={}, after={}",
        pre_expr_len,
        post_expr_len
    );
}

#[test]
fn test_jump_target_invariants_via_sanity_check() {
    let mut builder = IRBuilder::new_inlined(ArchInfo::new_host(), "jt_case");
    let ri32fty = FuncTypeID::new(builder.tctx(), ValTypeID::Int(32), false, []);
    let main_func = FuncID::builder(builder.tctx(), "main", ri32fty)
        .make_defined()
        .terminate_mode(FuncTerminateMode::ReturnDefault)
        .build_id(&builder.module)
        .unwrap();

    // entry -> then/else -> ret
    let entry = main_func.get_entry(builder.allocs()).unwrap();
    builder.set_focus(IRFocus::Block(entry));

    let cond = CmpInstID::new_uninit(
        builder.allocs(),
        Opcode::Icmp,
        CmpCond::NE | CmpCond::SIGNED_ORDERED,
        ValTypeID::Int(32),
    );
    cond.set_lhs(builder.allocs(), APInt::new(1u32, 32).into());
    cond.set_rhs(builder.allocs(), APInt::new(2u32, 32).into());
    builder.insert_inst(cond).unwrap();

    let then_bb = builder.split_block().unwrap();
    let else_bb = builder.split_block().unwrap();
    builder
        .focus_set_branch_to(ValueSSA::Inst(cond.raw_into()), then_bb, else_bb)
        .unwrap();

    // then: ret 0
    builder.set_focus(IRFocus::Block(then_bb));
    builder
        .focus_set_terminator(RetInstID::with_retval(
            builder.allocs(),
            APInt::new(0u32, 32).into(),
        ))
        .unwrap();

    // else: ret 1
    builder.set_focus(IRFocus::Block(else_bb));
    builder
        .focus_set_terminator(RetInstID::with_retval(
            builder.allocs(),
            APInt::from(1u32).into(),
        ))
        .unwrap();

    let filepath = "target/ir_builder_test_jump_target_invariants.ll";
    let mut file = std::fs::File::create(filepath).unwrap();
    let mut writer = IRWriter::from_module(&mut file, &builder.module);
    writer.option = IRWriteOption::loud();
    writer.write_module();

    // 自检验证 JT/Preds/Users 等基本不变量
    crate::ir::checking::assert_module_sane(&builder.module);
}

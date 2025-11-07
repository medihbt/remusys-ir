//! Debug-only IR sanity check helpers.
//!
//! 提供基本不变量快速断言，避免重构后潜在环链/引用错误静默积累。
//! 当前实现是“最低保障级别”，后续可加强：
//! - Phi incoming 对称性
//! - Terminator JumpTarget 完整性
//! - 无 `DisposedUse` 残留在活体 users/preds 环
//! - 指令父块 / 块父函数双向关系全量巡检
use crate::ir::*;

/// 运行一组轻量的不变量检查。仅在 `debug_assertions` 有效。
#[cfg(debug_assertions)]
pub fn ir_sanity_check(module: &Module) {
    let allocs = &module.allocs;

    // 检查符号表中的 Global 仍处于 live 状态 & 函数入口块父关系正确。
    for (name, gid) in module.symbols.borrow().iter() {
        assert!(
            <GlobalObj as crate::ir::module::allocs::IPoolAllocated>::_id_is_live(*gid, allocs),
            "Global symbol `{}` disposed unexpectedly",
            name
        );
        let gobj = gid.deref_ir(allocs);
        if let GlobalObj::Func(f) = gobj {
            if let Some(body) = &f.body {
                let func_id = FuncID(*gid);
                assert_eq!(body.entry.get_parent_func(allocs), Some(func_id), "Function @{} entry block parent mismatch", name);
            }
        }
    }

    // Use 环链与 operand 对应关系：若 operand 可追踪，则该 Use 必出现在 operand.users 环中。
    for (_, uptr, use_obj) in allocs.uses.iter() {
        let kind = use_obj.get_kind();
        if kind == UseKind::DisposedUse { continue; }
        let operand = use_obj.operand.get();
        if operand == ValueSSA::None { continue; }
        if let Some(users) = operand.try_get_users(allocs) {
            // 查找该 use 是否在 users 环中。
            let mut found = false;
            for (uid, u) in users.iter(&allocs.uses) {
                assert_ne!(u.get_kind(), UseKind::DisposedUse, "DisposedUse discovered inside active users ring");
                if uid == uptr { found = true; break; }
            }
            assert!(found, "Use missing from operand's users ring: kind={:?}", kind);
        }
        if let Some(user_id) = use_obj.user.get() {
            // 用户种类与 UseKind 的预期一致性（粗略）：非数组/结构/vec元素即指令类；GlobalInit 属于 Global。
            assert_eq!(user_id.get_class(), kind.get_user_kind(), "UseKind {:?} attaches to mismatched user category", kind);
        }
    }

    // JumpTarget 处于块前驱环链中（若绑定 block）。
    for (_, jptr, jt) in allocs.jts.iter() {
        let jkind = jt.get_kind();
        if jkind == JumpTargetKind::Disposed { continue; }
        if let Some(block_id) = jt.block.get() {
            let Some(body) = block_id.deref_ir(allocs).try_get_body() else { continue; }; // 跳过哨兵
            let preds = &body.preds;
            let mut found = false;
            for (pid, _) in preds.iter(&allocs.jts) { if pid == jptr { found = true; break; } }
            assert!(found, "JumpTarget not found in its block's preds ring (kind={:?})", jkind);
        }
        if let Some(term) = jt.terminator.get() {
            // 粗略一致性：终结指令的 jt 集合包含当前 jt。
            if let Some(jts) = term.deref_ir(allocs).try_get_jts() {
                let exists = jts.iter().any(|&id| id.inner() == jptr);
                assert!(exists, "Terminator missing registered JumpTarget (kind={:?})", jkind);
            }
        }
    }
}

/// 空实现：在非 debug 构建中不执行任何检查。
#[cfg(not(debug_assertions))]
pub fn ir_sanity_check(_: &Module) {}

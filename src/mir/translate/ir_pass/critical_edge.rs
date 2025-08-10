use std::{
    collections::{BTreeMap, VecDeque},
    rc::Rc,
};

use crate::{
    base::SlabRef,
    ir::{BlockData, BlockRef, FuncRef, IRAllocs, JumpTarget, Module, UseKind, ValueSSA},
};

/// 消除 IR 模块中所有函数的关键边
///
/// # 算法说明
///
/// 关键边是控制流图中连接多后继基本块和多前驱基本块的边。
/// 这种边的存在会使某些编译器优化（如 PHI 节点消除）变得复杂。
///
/// ## 检测方法
/// 对于边 `A -> B`，如果同时满足：
/// 1. `A` 有多个后继基本块
/// 2. `B` 有多个前驱基本块且包含 Phi 节点
///
/// 则 `A -> B` 是关键边。
///
/// ## 消除方法
/// 在关键边中间插入新的基本块 `C`：
/// 1. 创建新基本块 `C`，包含一条跳转到 `B` 的指令
/// 2. 将 `A` 的相关跳转目标重定向到 `C`
/// 3. 更新 `B` 中所有 Phi 节点，将来自 `A` 的引用改为来自 `C`
/// 4. 将 `C` 加入函数的基本块列表
///
/// # 参数
/// - `ir_module`: 要处理的 IR 模块
pub fn break_critical_edges(ir_module: &Module) {
    let funcs = ir_module.dump_funcs(false);
    let mut allocs = ir_module.borrow_allocs_mut();

    let mut critical_edges = VecDeque::new();
    for &func in funcs.iter() {
        find_critical_edges_for_func(func, &mut allocs, &mut critical_edges);
    }
}

/// 关键边对象. jts 表示所有起止点相同的重边.
struct CriticalEdge {
    from: BlockRef,
    to: BlockRef,
    jts: Vec<Rc<JumpTarget>>,
}

fn find_critical_edges_for_func(
    func: FuncRef,
    allocs: &mut IRAllocs,
    critical_edges: &mut VecDeque<CriticalEdge>,
) {
    let func_data = func.to_data(&allocs.globals);
    let Some(body) = func_data.get_body() else {
        return;
    };
    for (bref, bb) in body.view(&allocs.blocks) {
        find_critical_edges_for_block(bref, bb, allocs, critical_edges);
    }

    while let Some(edge) = critical_edges.pop_front() {
        // 这里不用 split, 而是直接创建新的中间基本块
        // 并将所有相关的 JumpTarget 重定向到这个新块。
        let mid_bb = {
            let mid_bb = BlockRef::new_jump_to(allocs, edge.to);
            func.body_unwrap(&allocs.globals)
                .node_add_prev(&allocs.blocks, edge.to, mid_bb)
                .expect("Failed to add new block");
            mid_bb
        };
        for jt in edge.jts {
            jt.set_block(&allocs.blocks, mid_bb);
        }
        edge.from.users(allocs).move_to_if(
            mid_bb.users(allocs),
            |u| matches!(u.kind.get(), UseKind::PhiIncomingBlock(_)),
            |u| u.operand.set(ValueSSA::Block(mid_bb)),
        );
    }
}

fn find_critical_edges_for_block(
    block_ref: BlockRef,
    block: &BlockData,
    allocs: &IRAllocs,
    critical_edges: &mut VecDeque<CriticalEdge>,
) {
    if !block.has_multiple_succs(&allocs.insts) {
        return;
    }
    let mut succ_map: BTreeMap<BlockRef, Vec<Rc<JumpTarget>>> = BTreeMap::new();
    for jt in block.get_successors(&allocs.insts).iter() {
        let succ_bref = jt.get_block();
        let succ_block = succ_bref.to_data(&allocs.blocks);
        if succ_block.has_multiple_preds() && succ_block.has_phi(&allocs.insts) {
            succ_map.entry(succ_bref).or_default().push(Rc::clone(jt));
        }
    }
    for (succ_block, jump_targets) in succ_map {
        critical_edges.push_back(CriticalEdge {
            from: block_ref,
            to: succ_block,
            jts: jump_targets,
        });
    }
}

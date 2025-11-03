# Remusys-IR 架构与不变量（基于 MTB::Entity）

> 作者: Github Copilot (GPT-5)
>
> Prompt: 考虑到 Remusys-IR 全部采用 MTB::Entity 以后很多自动管理的东西都要手动管理了，请你通读 IR Module 自顶向下所有类型的结构，找到它们的引用关系、组织原则、与完整性密切相关的不变式，一边研究一边在 `项目目录/docs/`下输出分析与总结文档。

本文自顶向下梳理 Remusys-IR 的对象结构、引用关系、组织原则，以及与完整性密切相关的不变量与生命周期策略。代码参考自 `src/ir/` 目录，核心包括 `module.rs / module/allocs.rs / global.rs / global/{func, var}.rs / block.rs / inst.rs / jumping.rs / usedef.rs`。

## 总览

- 顶层容器：`Module`
  - 拥有 `IRAllocs`（多池分配器）与 `TypeContext`，并维护 `symbols: HashMap<Arc<str>, GlobalID>` 模块符号表。
  - 所有 IR 节点（Block/Inst/Expr/Global/Use/JumpTarget）均分配于对应池中，释放走统一队列 `disposed_queue`。
- 节点类别：
  - 顶点（Vertices）：`GlobalObj`, `BlockObj`, `InstObj`, `ExprObj`
  - 边（Edges，侵入式环链节点）：`Use`（use-def 边），`JumpTarget`（CFG 边）
- 统一 ID：`PtrID<T>` 封装的 `*ID`（如 `BlockID/InstID/...`），类型安全，支持池级别索引、释放检测。
- 环链容器：`EntityRingList<T>`（含 `sentinel` 哨兵锚点）；序链容器：`EntityList<T>`（基本块/指令双向链表，带首尾哨兵）。

## 内存与分配器（IRAllocs）

- 各池：`exprs / insts / globals / blocks / uses / jts`。
- 释放策略：
  - 不直接 free 对象（除特殊 sweep 场景），而是 `dispose(...)` → `push_disposed(id)` → 统一 `free_disposed()`。
  - GC 的 sweep 阶段对 `Use`/`JumpTarget` 先 `dispose()`（摘出环链），其余顶点依据 live bitset 直接 `free_if(...)`。
- `IPoolAllocated`/`PoolAllocatedID`：统一封装了不同池对象的 create/dispose/free 流程与类型判定。

## Value 与 Use-Def

- SSA 值联合：`ValueSSA = None | ConstData | ConstExpr(ExprID) | AggrZero(AggrType) | FuncArg(FuncID,u32) | Block(BlockID) | Inst(InstID) | Global(GlobalID)`
  - `ISubValueSSA` 提供类型查询、值类型、是否可 trace、users 列表访问等。
- 跟踪接口：
  - `ITraceableValue`：持有 `UserList`（`EntityRingList<Use>`）。核心方法：
    - `users() / user_iter()`：获取用户环链
    - `traceable_init_self_id(allocs, self_value)`：将环链上 Use 的 `operand` 回填为自己的 `ValueSSA`
    - `traceable_dispose(allocs)`：清空用户环链，并 `dispose` 其 `sentinel`
  - `IUser`：有操作数的“用户”（Inst/Expr/Global）。核心方法：
    - `get_operands()/operands_mut()` → `UseID` 列表
    - `user_init_self_id(allocs, user_id)`：为每个 `Use` 绑定 `user`
    - `user_dispose(allocs)`：依次 `dispose` 所有操作数 Use，并清理自身 traceable 状态
- Use（环链节点）
  - 字段：`list_head`, `kind: UseKind`, `user: Option<UserID>`, `operand: ValueSSA`
  - 语义不变量：
    - 若 `operand != None` 并且 `operand` 是可追踪的，该 Use 必须挂在 `operand.users` 环链上；否则，不在任意用户环上。注意, 不是所有类型的 `ValueSSA` 都可以被追踪: 像 `CosntData` 这类值语义的 Value 没有追踪链表, 不可追踪.
    - `UseKind::DisposedUse` 仅在 `dispose()` 后出现；其它场景不可直接设置该枚举。
  - 变更：`set_operand` 会先从旧 `operand` 的 users 环 `detach`，再将新 `operand` 的 `users` 环 `push_back_id`。`dispose` 会 `detach` 并清空 `user/operand`。

## 控制流 JumpTarget 与 Preds 环

- JumpTarget（环链节点，表达 CFG 一条边）：
  - 字段：`kind: JumpTargetKind`，`terminator: Option<InstID>`，`block: Option<BlockID>`
  - 挂载：附着于 `block.get_preds(): PredList = EntityRingList<JumpTarget>`；即以“目标块”为锚的前驱边表。
  - 语义不变量：
    - 当 `block.is_some()` 时，该 JumpTarget 必在该块的 `preds` 环链中；当 `block.is_none()` 时，不应在任一 `preds` 环中。
    - `terminator` 与持有该 JT 的终结指令数组形成回指（在 `inst_init_self_id` 中回填）。
  - 变更：`set_block` 先从旧 `preds` 环 `detach`，再加入新块的 `preds` 环；`dispose` 会 `detach`，清空 `terminator/block` 并标记 Disposed。

## 基本块（BlockObj）

- 结构：
  - `parent_func: Option<FuncID>`；`body: Option<BlockObjBody>` 表明是否哨兵
  - `BlockObjBody`：
    - `insts: EntityList<InstObj>`（带首尾哨兵）
    - `phi_end: InstID`（Phi 区段结束标记，作为链表中的特殊节点）
    - `users: UserList`（块作为 SSA 值被引用时的 users 环）
    - `preds: PredList`（前驱边环）
- 不变量：
  - 将 Inst 加入块时，`on_push_{next,prev}` 断言当前块存在 parent（必须先设置 parent 再链接）
  - `on_unplug` 断言非 body 哨兵并清除 parent；inst 的 `parent_bb` 设置/清除与入链/出链同步
  - `phi_end` 与 `insts.{head,tail}` 的 `parent_bb` 在 `init_self_id` 中回填为该块
- 终结指令：
  - `try_get_terminator()`/`get_terminator_inst()` 获取尾部是否为 terminator
  - `set_terminator_inst()` 支持替换并返回旧 terminator 的 `ManagedInst`（离作用域自动 dispose）
- 释放：
  - `dispose()`：
    - 遍历 `insts.forall_with_sentinel`，逐个 `InstID.dispose()`
    - `traceable_dispose()` 清理 users 环与哨兵
    - `JumpTargetID(body.preds.sentinel).dispose()` 释放前驱环的哨兵（其余 JT 在 GC 或指令处置时各自摘链）

## 指令（InstObj）

- 结构：`InstCommon { parent_bb, users: Option<UserList>, opcode, ret_type, disposed }` + 各具体指令体。
- 作为用户：
  - `IUser::get_operands()` 返回 Use 集合；`user_init_self_id` 将每个 Use 绑定到自身；
  - `_common_init_self_id` 还会为自身持有的 JumpTarget（若有）设置 `terminator = self_id`。
- 作为值：`ITraceableValue`（Inst 结果）持有 users 环，用于被其它用户引用。
- 终结指令：`ITerminatorInst` 提供 `get_jts()/jts_mut()` 等；`try_get_jts()` 在非终结指令返回 None。
- 释放：
  - `_common_dispose`：先 `user_dispose`（释放操作数 Use），再 `dispose` 所有 JumpTarget；
  - `dispose()` 针对各具体指令委派，哨兵/Unreachable 走公共路径。

## 全局（GlobalObj）与符号表

- `GlobalCommon { name, content_ty, align, users, back_linkage, dispose_mark }`
- 变体：
  - `GlobalVar`：`initval: [UseID;1]`（初始化表达式 Use），`readonly: bool`
  - `FuncObj`：`args: [FuncArg]`（每个参数是 ITraceableValue，具 users 环），`body: Option<FuncBody>`
    - `FuncBody { blocks: EntityList<BlockObj>, entry: BlockID }`
    - `_init_self_id`：
      - `user_init_self_id(UserID::Global)`，为 args 设置 `func` 与 `users` 环的 `operand = ValueSSA::FuncArg(func, index)`
      - 若有 body，遍历 blocks 回填 `parent_func`
- 符号表与释放：
  - `ISubGlobal::dispose(module)` 需要先 `common_dispose`（从 `module.symbols` 注销），再做 `user_dispose`/释放体。
  - `GlobalVar::dispose`：先注销符号，再释放 init Use 与自身 users；
  - `FuncObj::dispose`：先注销符号，再释放每个参数（清 func、清 users），释放 body 中的每个块，最后释放自身 users。

## 生命周期与销毁顺序（关键）

- 顶点（Global/Block/Inst/Expr）通常：
  1) `user_dispose()` → 释放所有操作数 Use；
  2)（对终结指令）释放其 JumpTargets；
  3) `traceable_dispose()` → 清空 users 环并释放哨兵；
  4) `push_disposed(id)` 等待统一 free。
- 边节点（Use/JumpTarget）必须先 `dispose()` 才能 `free`：
  - 因其位于活体环链（users/preds）上，若直接 free 会破坏环链不变量；
  - GC 中专门“先处置 Use/JT”，再清理其它对象。
- 符号表：Global 的 `dispose` 必须先注销符号，以避免 free 后仍被 `module.symbols` 引用。

## GC 标记-清扫算法（简述）

- 标记（`IRMarker`）：
  - 从根（如模块符号表中存活 Global）开始：
    - 顶点沿“强出边”推进：
      - Global → 其 operands（Use→ValueSSA）、users 哨兵；Func 还会标记每个参数的 users 哨兵，以及 body.blocks
      - Block → insts（含 phi_end 等哨兵的入队是安全的），preds/users 哨兵
      - Inst/Expr → operands（Use→ValueSSA）、users 哨兵；Inst 若为 terminator 再沿 JumpTarget → 目标 Block
      - Use → operand（ValueSSA）；JumpTarget → block（None 时忽略）
  - 每次入队前用 bitset 去重，避免重复扫描。
- 清扫（`IRLiveSet::sweep`）：
  1) 先扫描 `uses/jts`，对未标活的调用 `dispose()` 并入队 `disposed_queue`；
  2) `free_disposed()` 统一释放已处置的边节点；
  3) 其余顶点直接按 bitset `free_if(...)`。

## 组织原则与不变量清单（关键）

- 容器与父子关系：
  - Inst 必须有 `parent_bb` 才能入 `Block.insts`；出链时会清父指针；`Block` 自身必须有 `parent_func` 才能入 `Func.blocks`。
- 环链锚点：
  - 所有 `EntityRingList` 均以 `sentinel` 为锚；对活体值/块，哨兵必须存在且存活。
  - `traceable_dispose` 会清空环并 `dispose` 哨兵（随后活体不再需要该环）。
- Use 语义一致性：
  - `Use.operand == None` ⇒ 不在任何环；`Use.operand != None` ⇒ 一定在 `operand.users` 环上。
  - `set_operand/clean_operand/dispose` 必须通过 `detach/push_back_id` 维护环一致性。
- JumpTarget 一致性：
  - `block.is_some()` ⇒ 在该块的 `preds` 环；`block.is_none()` ⇒ 不在任一 `preds` 环；
  - 从指令角度持有 `JumpTargetID` 数组，并在 `_common_init_self_id` 里回填 `terminator`。
- Global 符号表：
  - `dispose` 顺序必须“先 `common_dispose`（注销符号），后 `user_dispose`/释放体”；
  - 注册时对重名以 `Entry::Vacant/Occupied` 判定，返回已存在的 `GlobalID` 错误。
- 释放队列与幂等：
  - `dispose()` 幂等（再次调用返回 false / 报已释放），避免二次处置；
  - `IRAllocs::free_disposed()` 统一按照 ID 类型分发到正确池释放。

## 常见失败模式与防御

- 直接 free 边节点（Use/JT）而未先 `dispose()`：破坏活体环链 → 遍历崩溃；
- 未维护 parent 指针与链表操作的配套：触发断言或悬挂；
- Global 未先从符号表注销即释放：`symbols` 悬挂引用；
- 未回填/清理 JumpTarget 的 `terminator/block`：导致 CFG 不一致（遍历或打印异常）。

## 调试建议与可选增强

- Debug-only 不变量检查（建议）：
  - 对存活顶点（Inst/Block/Expr/Global/FuncArg）遍历其 users 环，断言不存在已释放/未标活的 Use；
  - 对存活 Block 遍历 `preds` 环，断言不存在已释放/未标活的 JumpTarget；
  - 对 Inst 的 `try_get_jts()`，断言每个 JT 的 `terminator==self`。
- 明确根集策略：建议以模块符号表中的 `GlobalID`（及必要的 `Block`）为根进行标记，避免“孤立 Inst 作为根”导致容器未保活。
- 代码一致性：在需要的位置显式跳过哨兵的扩散（当前实现通过返回 None 已达同效）。

## 术语速览

- 顶点：Global/Block/Inst/Expr
- 边：Use（use-def 边）/JumpTarget（CFG 边）
- 用户环：UserList = EntityRingList<Use>
- 前驱环：PredList = EntityRingList<JumpTarget>
- 哨兵：sentinel（环链锚点）
- 处置：dispose（摘链、清指针，入释放队列）
- 释放：free（真正释放内存，一般由 `free_disposed()` 或 `free_if` 触发）

---

本文件旨在为后续 IRBuilder 重构与联调提供“结构与不变量”视图，便于验证生命周期与容器一致性。若结构演进，请同步更新本文件。
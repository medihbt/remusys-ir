# Remusys-IR 内存管理、引用关系与不变量（新版 allocate/dispose 设计）

> 适用分支：with-mtb-entity（新版 IPoolAllocated 统一生命周期）

本文在通读 `src/ir/` 全部代码后，完整描述 Remusys-IR 在 MTB::Entity 体系下的托管元素、ID 类型、引用组织、生命周期范式与完整性不变量，并指出潜在缺陷与改进建议。核心参考文件：
- `module.rs` / `module/{allocs.rs, managing.rs, gc.rs}`
- `global.rs` / `global/{func.rs, var.rs}`
- `block.rs` / `inst.rs`（含各子指令文件）/ `jumping.rs` / `usedef.rs`

## 顶层与内存池

- 顶层容器：`Module { allocs: IRAllocs, tctx: TypeContext, symbols: HashMap<Arc<str>, GlobalID> }`
- 多池分配器：`IRAllocs` 持有六类实体池与一个延迟释放队列：
  - 顶点池：`exprs / insts / globals / blocks`
  - 边池：`uses / jts`
  - 延迟释放：`disposed_queue: VecDeque<PoolAllocatedID>`，由 `push_disposed` 入队、`free_disposed` 统一出队释放

### 统一生命周期接口：IPoolAllocated

- 所有池对象实现 `IPoolAllocated`：
  - 分配：`allocate(allocs, obj) -> ID`，内部先放入池，再回调 `init_self_id(id, allocs)` 完成“自引用回填/挂链”
  - 释放：`dispose_id(id, pool)` → `dispose_obj(&self, id, pool)` 执行“摘链/标记/注销”等，再 `push_disposed(id)` 等待统一 `free`
  - 幂等：重复处置返回 `AlreadyDisposed`
  - 分类：`PoolAllocatedClass` 与一体化枚举 `PoolAllocatedID` 支持跨池统一处理（索引、标记、释放）

契约（简）：
- 分配成功后，`init_self_id` 必须将所有“反向边/持有者字段”与“环链哨兵”回填完整，保证对象立即处于一致状态；
- `dispose_*` 必须先恢复外部结构不变量（detach 环链、unplug 序链、清 parent/back-link、注销符号），再入延迟释放队列；
- 边节点（Use/JumpTarget）严禁直接 free，必须先 dispose 以维护环链完整性。

## SSA 值、用户与 Use-Def 边

- SSA 值联合：`ValueSSA = None | ConstData | ConstExpr(ExprID) | AggrZero(AggrType) | FuncArg(FuncID,u32) | Block(BlockID) | Inst(InstID) | Global(GlobalID)`
- 追踪接口：
  - `ITraceableValue` 提供 `UserList = EntityRingList<Use>` 与遍历/统计；具引用唯一性的值（如 Inst/Block/FuncArg/Global/Expr）拥有有效的 users 哨兵
  - `IUser` 提供操作数访问；用户的 `user_init_id(self_id)` 会：
    1) 通过 `traceable_init_id` 令自身 users 环上的所有 Use 的 `operand = self_id.into_ir()`（包括哨兵）
    2) 为每个操作数 Use 设置 `user = Some(self_id)`
- Use 节点：
  - 字段：`kind: UseKind, user: Option<UserID>, operand: ValueSSA`，为可入环的侵入式节点
  - 赋值：`set_operand` 先从旧 `operand.users` 环 `detach`，再尝试加入新 `operand.users`（若可追踪）；`clean_operand` 清空并脱环
  - 处置：`dispose` = `mark_disposed + detach + user=None + operand=None`
  - 不变量：`operand == None ⇒ 不在任何 users 环；operand != None 且可追踪 ⇒ 必在对应 users 环`

## 控制流边 JumpTarget 与前驱环

- `JumpTarget { kind, terminator: Option<InstID>, block: Option<BlockID> }`，以“目标块”为锚挂接到 `BlockObjBody.preds: EntityRingList<JumpTarget>`
- 变更：`set_block` 会从旧块环链脱离后再插入新块 `preds`；`clean_block` 清空并脱环
- 处置：`dispose` = `mark_disposed + detach + terminator=None + block=None`
- 不变量：
  - `block.is_some() ⇒` 节点存在于该块 `preds` 环；`block.is_none() ⇒` 不在任何 `preds` 环
  - 作为终结指令的出边，`init_self_id(inst)` 会为所有持有的 `JumpTarget` 回填 `terminator = inst`

## 基本块 Block 与指令 Inst

### BlockObj

- 结构：`parent_func: Option<FuncID>` 与 `body: Option<BlockObjBody>`（None 表示链表哨兵）
- `BlockObjBody`：`insts: EntityList<InstID>`、`phi_end: InstID`、`users: UserList`、`preds: PredList`
- 初始化：`init_self_id` 会为 `insts` 的所有节点（含哨兵/phi_end）设置 `parent_bb = Some(block)`，并回填自身 users 环的 Use.operands
- 处置：从父函数块表中 `node_unplug`，`dispose_entity_list(insts)` 逐个处置指令并处置 head/tail 哨兵；`traceable_dispose(self)` 清 users 环；清空 `preds` 并处置其哨兵

构建期 parent_bb 可为空（已在实现中处理）：当前 `InstObj::on_push_{next,prev,unplug}` 均允许 `parent_bb == None`，因此 `BlockObjBody::new` 在 `parent_bb` 尚未建立时插入 `phi_end` 是安全的；初始化完成后，再由 `BlockObj::init_self_id` 为全部节点（含哨兵/phi_end）补齐 `parent_bb`。

### InstObj

- 结构：`InstCommon { parent_bb, users: Option<UserList>, opcode, ret_type, disposed }` + 具体指令体
- 初始化：`init_self_id(inst)` 按下述顺序保证一致性：
  1) `user_init_id(self, UserID::Inst(inst))` 将自身 users 环 Use 的 `operand` 与全部操作数 Use 的 `user` 回填
  2) 若为终结指令，遍历 `try_get_jts()` 中的 `JumpTargetID`，为其设置 `terminator = inst`
  3) 特例：`PhiInst` 记录 `self_id` 以便后续动态增删 incoming 时设置 `Use.user`
- 处置：公共流程由 `inst_dispose` 执行：
  1) 标记 `disposed = true`
  2) 若有父块且非哨兵，则从 `Block.insts` 链表 `node_unplug`（同时清空自身 `parent_bb`）
  3) `user_dispose(self)` 逐个处置操作数 Use，并清理自身 users 环与哨兵
  4) 若为终结指令，遍历 `JumpTargetID` 并逐个 `dispose`
  5) 指令专属清尾：例如 `PhiInst` 在处置后清空其 `self_id`

动态结构：
- `PhiInst` 的 incoming 以二元组 `[UseID;2]` 存储（值、来源块），内部使用 `SmallVec`；增删时根据位置更新 `UseKind` 序号并维护 `Use.user`；删除使用 `swap_remove` 后仅在确有“末尾搬移”时更新被搬移元素索引（避免越界）
- `SwitchInst` 的跳转目标使用 `SmallVec<JumpTargetID>` 动态扩展：
  - 构造期先创建 `default` 目标；`push_case_jt` 如检测到默认目标已有 `terminator`，会为新 case 回填同一 `terminator`；
  - 无论构造期是否已设置，最终在 `InstObj::init_self_id` 会统一遍历并回填 `terminator`，保证一致性

## 全局 Global 与函数体

- `GlobalCommon { name, content_ty, content_align_log, users: Option<UserList>, back_linkage, dispose_mark }`
- 变体：
  - `GlobalVar { initval: [UseID;1], readonly }`
  - `FuncObj { args: Box<[FuncArg]>, ret_type, is_vararg, body: Option<FuncBody> }`
  - `FuncArg` 是可追踪值，持有 `users` 环与 `func: Cell<Option<FuncID>>`
- 初始化：
  - `GlobalObj::init_self_id` 首先 `user_init_id(UserID::Global(id))`
  - 函数体：为每个 `FuncArg` 设置 `func=Some(FuncID(id))`，并将 `arg.users` 环上的 Use 的 `operand` 回填为 `ValueSSA::FuncArg(func, index)`；若存在 `body`，遍历 `blocks` 的所有节点（含哨兵）设置 `parent_func`
- 处置：
  - `global_common_dispose` 统一完成：幂等检测、`symbols` 注销（借用冲突返回 `SymtabBorrowError`）、`user_dispose(global)` 清理该全局本身的 users 与操作数
  - `GlobalVar`：仅需公共处置
  - `FuncObj`：在公共处置后，逐个参数清 `func=None` 并 `traceable_dispose(arg)`；如有 `body`，`dispose_entity_list(body.blocks)` 处置所有基本块与哨兵

## GC：标记与清扫（IRMarker / IRLiveSet）

- 标记：
  - 根：来自 `Module.symbols` 的 `GlobalID`；也可追加外部根
  - 推进：
    - `Global` → `operands(Use)` 与自身 `users.sentinel`；`Func` 额外标记各 `FuncArg.users.sentinel` 与 `blocks`
    - `Block` → `insts`（含 `phi_end` 与哨兵安全入队）、`preds.sentinel`、`users.sentinel`
    - `Inst/Expr` → `operands(Use)` 与 `users.sentinel`；`Inst` 若为 terminator 还会标记每个 `JumpTarget`
    - `Use` → 其 `operand(ValueSSA)`；`JumpTarget` → 其 `block`
  - 去重：用 `FixBitSet` 按池索引避免重复
- 清扫：
  1) 先遍历 `uses` 和 `jts` 两个边池，对“未标活”的逐个调用 `dispose_obj` 并 `push_disposed`
  2) 调用 `free_disposed` 释放所有已处置边节点
  3) 其余顶点（`insts/blocks/exprs/globals`）基于 bitset 直接 `free_if`（无需显式 dispose，因所有外连边已在步骤 1 保守清理）

该顺序确保：任何指向死对象的环链节点在目标对象 free 前已被安全摘除，不破坏容器不变量。

## 关键不变量清单（运行期必须满足）

- 序链/父子：
  - Inst 入链前必须已设置 `parent_bb`；出链时会清空 `parent_bb`
  - Block 入 `Func.blocks` 前必须已设置 `parent_func`；出链时清空
- Use-Def：
  - `Use.operand == None ⇒` 不在任何 `users` 环；否则若 `operand` 可追踪 ⇒ 必在对应值的 `users` 环
  - `Use` 的 `user` 字段仅通过 `user_init_id` 或 `Phi`/构建期的专用逻辑设置/更新
- CFG：
  - `JumpTarget.block.is_some() ⇒` 节点出现在该块的 `preds` 环；`is_none() ⇒` 不在任何环
  - 任意活体 terminator 的所有 `JumpTarget.terminator` 均等于该 terminator 自身
- 符号表：任意已释放/处置的 `Global` 不应仍然存在于 `Module.symbols` 中
- 幂等性：任何 `dispose_*` 二次调用不会破坏状态（返回 `AlreadyDisposed` 或等价语义）

## 已发现/已修复的问题与建议

1) 构建期 `phi_end` 插入与 parent_bb 断言风险：已修复。
  - 现状：`InstObj::on_push_{next,prev,unplug}` 放宽为允许 `parent_bb == None`，并在 push 时将相邻节点的 `parent_bb` 设置为当前的（可能为 None）。
  - 结论：`BlockObjBody::new` 中提前插入 `phi_end` 不再有断言风险；`BlockObj::init_self_id` 会在初始化时为链上所有节点补齐 `parent_bb`。

2) 动态 JT 终结者回填的健壮性：`SwitchInst::push_case_jt` 试图从 `default_jt` 读取 `terminator` 以传播到新 case；若构建期尚未完成 `init_self_id`，该字段为 `None`，但稍后 `InstObj::init_self_id` 会统一回填，整体是安全的。建议在注释中明确这种“双路径保证”，避免未来修改引入遗漏。

3) Debug-only 自检建议：
   - 对活体 Inst/Expr/Global/Block/FuncArg 迭代 `users`，断言无 `DisposedUse` 节点且所有 Use 的 `operand` 反向指向该值
   - 对活体 Block 迭代 `preds`，断言无 `Disposed` 的 JT，且每个 JT 的 `block`/`terminator` 均一致
   - 对任意活体 terminator，断言其 `JumpTarget.terminator == self`

## 小结

新版的 allocate/dispose 通过 `IPoolAllocated` 统一生命周期，并在 GC 中“先处置边、后释放点”，配合 users/preds 哨兵标记，有效维护了容器不变量与引用一致性。除上文提到的 `phi_end` 插入时机可能触发断言的边角问题外，整体设计清晰、职责分离良好且具可扩展性。建议按“潜在缺陷与建议修复”落地一次小改，以进一步提升构建期鲁棒性，并考虑引入 Debug-only 自检以固化不变量。
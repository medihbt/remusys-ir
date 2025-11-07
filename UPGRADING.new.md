# 升级指南：0.1.x → 0.2.0 架构重写（临时新文件）

此文件用于替换已损坏的原始 `UPGRADING.md`，供人工比对。若内容确认无误，可将其重命名为 `UPGRADING.md` 覆盖旧文件。

## 核心变化一览
- 统一生命周期：所有池对象实现 `IPoolAllocated`，分配即回填自引用，处置幂等 + 延迟释放。
- Use-Def 改写：`Use` 环链 + `UserList` 取代旧散列/向量结构，支持快速 detach / move。
- 跳转目标抽象：`JumpTargetID` 独立管理终结指令到目标块的边，块维护前驱环 `PredList`。
- GC：`IRMarker` + `IRLiveSet`，扫描根（符号表）并安全清理死边后释放死顶点。

## 旧 → 新 主要符号映射
| 旧名称 (0.1.x)                | 新名称 (0.2.0)                     | 备注 |
|-------------------------------|------------------------------------|------|
| `InstRef`, `BlockRef`         | `InstID`, `BlockID`                | ID 统一，直接是池指针包装 |
| `GlobalRef`, `VarRef`, `FuncRef` | `GlobalID`, `FuncID`              | 函数即 `GlobalID` 的子类型 |
| `ExprRef` / 常量复合对象       | `ExprID` + 具体 `ArrayExprID` 等    | 表达式仍走统一池 |
| `ValueSSAClass`               | `ValueClass`                       | 简化分类枚举 |
| `UseDef` / `user.rs`          | `usedef.rs` 中 `UseID`, `UserList` | 环链表示，O(1) detach |
| Terminator + 目标混合结构      | `JumpTargetID` + `JumpTargets`     | 终结指令内聚合 `JumpTargetID` 列表 |
| MIR (`src/mir/*`)             | 已删除                             | 暂不再提供中间层 |
| 优化 pass (`opt/*`)           | 已删除                             | 以后将重建于新架构 |
| 属性集合 (`AttrList`, `AttrSet`)| 暂移除                             | 将来若需要重新设计 |
| `compact_ir/*`                | 已删除                             | 不再维护紧凑表示 |
| `checking/*`                  | 已删除                             | 以 Debug-only 自检取代 |

## 构建器迁移示例
旧：
```rust
// 伪示例 (0.1.x)
let f = FuncRef::new("foo", functy, &module);
let bb = f.append_block();
let inst = InstRef::new_binop(bb, Opcode::Add, a, b);
```
新：
```rust
use remusys_ir::ir::{IRBuilder, IRFocus, Opcode, BinOPInstID, ValueSSA};

let mut builder = IRBuilder::new_inlined(arch, "foo_mod");
let functy = FuncTypeID::new(builder.tctx(), ValTypeID::Int(32), false, []);
let foo = FuncID::builder(builder.tctx(), "foo", functy)
    .make_defined()
    .build_id(&builder.module)
    .unwrap();
let entry = foo.get_entry(builder.allocs()).unwrap();
builder.set_focus(IRFocus::Block(entry));
let add_id = BinOPInstID::new(
    builder.allocs(),
    Opcode::Add,
    APInt::new(1u32, 32).into(),
    APInt::new(2u32, 32).into(),
);
builder.insert_inst(add_id).unwrap();
```

## Use / Users 迁移
旧：手工维护 `Vec<Use>` 或散列表。
新：`UserList` 自动环链：
```rust
let use_id = UseID::new(builder.allocs(), UseKind::BinOpLhs);
use_id.set_operand(builder.allocs(), ValueSSA::Inst(add_id.into_instid()));
// 环链自动插入到被引用值的 users 列表
```

## 跳转与 CFG
旧：终结指令内嵌目标块指针，块不维护前驱。
新：
- `JumpTargetID` 对象持有 `terminator: Option<InstID>` 与 `block: Option<BlockID>`。
- 块维护 `preds: PredList`（环链哨兵）。
- 构建器辅助替换当前终结指令（原跳转关系会被丢弃）：
  - `focus_set_jump_to(target_bb)`
  - `focus_set_branch_to(cond, then_bb, else_bb)`
  - `focus_set_switch_to(discrim, default_bb, cases)`
注意：基本块必须满足“单 terminator 且位于块尾”的结构；如需在块中间插入指令，请先使用 `IRFocus` 与 `get_split_pos/split_block_at_*` 获取或切分插入点。

## GC 行为差异
| 项目            | 旧版                          | 新版 |
|----------------|-------------------------------|------|
| 边释放策略     | 即时释放，可能破坏引用结构    | 延迟处置 + 统一 `free_disposed` |
| 顶点扫描       | 遍历所有对象                  | 仅标活顶点，未标活直接 free |
| Use/JumpTarget | 普通对象                      | 作为“边”优先安全 detach |

## Debug 自检
启用 `debug_assertions` 时可调用：
```rust
#[cfg(debug_assertions)]
remusys_ir::ir::ir_sanity_check(&builder.module);
```
当前检查项（初版已覆盖以下关键点）：
1. Use/User 不变量：`Use` 必在被引用值的 users 环；`Use.user` 与 `User.operands[idx]` 一致；不允许 `DisposedUse` 混入活体环。
2. 函数/块拓扑：入口块在位置 0；块 `parent` 正确附着；函数参数索引与位置一致。
3. 基本块结构：`[Phi*] -> PhiEnd -> [普通指令...] -> Terminator`，且仅允许一个 terminator 且位于块尾。
4. 前驱环链：块 `preds` 环闭合；其中每个 `JumpTarget.block` 与该块一致。
5. 终结指令与 JumpTarget：terminator 持有的所有 `JumpTarget.terminator` 反向指向自身；目标块存在且已附着；禁止跨函数跳转。
6. 指令级类型/语义：
   - Ret/Br/Switch：返回值/条件/判别式类型校验；并校验其 JumpTargets 的基本不变量。
   - GEP：`base` 必为指针，使用 `GEPTypeIter::run_sanity_check` 验证索引与类型解包路径。
   - Load/Store/AmoRmw：指针/值类型匹配。
   - BinOP：整数/浮点/移位操作的元素与向量约束；移位量可为标量或等长向量。
   - Call：`callee` 必为函数指针；形参与实参与类型一致（变参放宽）。
   - Cast：各类整型/浮点/指针转换的方向性与位宽检查。
   - Cmp：结果为 `i1`，左右操作数同型，且类目匹配（int/float）。
   - Phi：每个前驱恰有一条 incoming；禁止重复/缺失；块操作数必须真为块值。
7. 常量表达式：数组/结构体/向量元素/字段类型逐一匹配。

后续计划增强：更细粒度的 Phi 与 JumpTarget 链一致性、无任何处置节点残留在活体环、跨模块引用的边界断言等。

## 迁移步骤建议
1. 升级版本依赖到 `0.2.0`。
2. 重写构建逻辑：替换旧 `Ref` API 为新 Builder + `*ID` 类型。
3. 若使用旧 MIR / 优化功能：暂时移除相关调用，待后续回归。
4. 启用 Debug 模式运行 `ir_sanity_check` 验证基本不变量。
5. 根据需要自行实现临时 Pass，基于 `usedef` 与 `jumping` 提供的结构。

## 依赖与缓存注意
- `mtb-entity` 依赖切换为上游 `master` 分支（`Cargo.toml` 使用 git+branch 配置）。
- 升级后建议执行一次依赖更新，必要时清理构建缓存：
  - `cargo update`
  - 如遇到实体分配/引用异常的历史缓存影响，可考虑 `cargo clean` 后重建。

## 回滚与标签
若需要在历史中访问旧实现：在合并前创建标签：
```bash
git tag pre-rewrite-mir
```

## FAQ
- 为什么移除 MIR？
  重构聚焦底层 IR 的引用/生命周期稳定；中间层将在确认新核心稳定后再决定是否回归。
- 为什么不保留旧属性系统？
  旧属性扩展性不足，等待重新设计与语义规范。
- GC 为什么“先边后点”？
  确保在释放顶点前，所有引用边已经安全摘链，避免悬挂环链。

## 后续关注
欢迎在后续版本中补充：优化 Pass、常量折叠、属性系统 v2、调试可视化。

---
如遇迁移问题，可直接在代码处打 `panic!` 并逐段迁移，因当前项目仅供个人使用，不需维持对外兼容矩阵。

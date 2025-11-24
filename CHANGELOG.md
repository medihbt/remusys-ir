# Changelog

本文件记录 `remusys-ir` 的可视语义变更。遵循 (MAJOR.MINOR.PATCH) 版本语义：
在 0.x 阶段，MINOR 代表潜在破坏性重构；PATCH 为向后兼容修补与小改。

## [0.2.1] - 2025-11-24
### 摘要 (相对 0.2.0)
在 0.2.0 基础上添加新 Value 类型，尝试添加一个实验性的属性系统; 同时适配下层依赖 `mtb-entity-slab` 的更新.

### Added
- 新操作数类型: `DataArrayExpr` `SplatArrayExpr` `KVArrayExpr` 等数组表达式, 用于不同场景下的数组数据表示.
- 实验性属性系统: `ir::attributes::Attribute` 枚举及相关 API, 支持为 IR 元素附加元数据.

### Changed
- 适配 `mtb-entity-slab` 更新: 迁移至最新版本, 更换引用模型.

## [0.2.0] - 2025-11-07
### 重构摘要 (相对 0.1.0)
主要完成“内存池替换 + API 打磨”：统一生命周期与引用管理；移除旧 MIR/优化/属性细分模块；将 use-def、JumpTarget 与 GC 迁移到基于 Entity 的实现并校准接口。

### Added
- `IPoolAllocated` 统一分配/处置接口与延迟释放队列 (`disposed_queue`).
- `usedef` 模块迁移/重构：`Use`/`UserList` 基于 Entity 环链，操作数迭代与 API 更趋一致。
- `jumping` 模块迁移/重构：终结指令与 `JumpTarget` 解耦，块前驱环链使用 Entity 实现。
- `Managed*` 系列包装：`ManagedInst/ManagedBlock/ManagedExpr/...`（更明确的托管语义）。
- 构建器 API 调整：`IRBuilder`、`FuncBuilder`、`IRFocus` 等接口更一致、更易用。
- 文档：`docs/ir-architecture-and-invariants.md` 描述生命周期与不变量。
- Debug 模式不变量检查入口保留并增强（`ir::checking::*`）。
- 迁移指南：`UPGRADING.md`。
- 基础测试与不变量断言：`src/testing/pr.rs` 覆盖构建器冒烟、GC 计数、JumpTarget 环与 GEP 类型检查。
- 依赖更新：对 `mtb-entity` 使用上游 `master` 分支（需注意本地构建缓存；偶发引用行为问题请先 `cargo update` 再复测）。

### Changed

- 内存池实现替换：除类型系统外，Slab → `mtb-entity`（非收缩池约定），`WeakList` → Entity 环链。
- 全部 ID/对象命名：`*Ref` / `*Data` → `*ID` / `*Obj`；指令/表达式/全局量都通过池 ID 管理。
- GC 策略简化：移除 mark-compact，保留 mark-sweep；扫描与释放顺序更明确。
- Writer/Numbering API 语义调整：`IRNumberValueMap`、`IRWriteOption` 等新命名。
- 模块间依赖重新梳理。

### Removed

- 多池 GC：删除了 mark-compact GC。
- 整个旧 MIR 子系统 (`src/mir/*`) 与相关翻译、寄存器分配、格式化、伪指令支持。
- 旧优化框架 (`src/opt/*`) 与 CFG/Dominator/DCE pass。
- 旧属性集合/列表模块 (`ir/attributes/attrlist.rs`, `attrset.rs`).
- `checking/` 与 `compact_ir/`（旧完整性检查与压缩表示）。
- 若干早期 Slab 引用实现 (`slablist`, `slabref`).

### Deprecated (计划后续处理)
- 无：重构已直接移除旧实现。

### Migration Notes
详见 `UPGRADING.md`，包括旧→新符号映射与示例。若需临时访问旧实现，请使用分支 `old-with-slab`。

### Internal Invariants（简要）
1. `Use.operand != None` 且可追踪 ⇒ 必出现在对应值的 `users` 环。
2. 活体 terminator 持有的所有 `JumpTarget.terminator` 与其一致；块前驱环中 `JumpTarget.block` 一致。
3. 处置顺序：边节点逻辑 detach → 入延迟释放队列 → 统一 `free_disposed`；顶点直接 free_if 未标活。
4. 基本块结构：`[Phi*] -> PhiEnd -> [普通指令...] -> Terminator`，且仅单 terminator；入口块在函数体数组位置 0。
5. 跳转目标与块/函数归属一致，不跨函数、不指向未附着块；前驱环链闭合无断点。
6. Use / User 双向引用：`Use.user` 与其在 `User.operands` 下标匹配；DisposedUse 不得出现在活体环。

### 后续计划
- 重新引入基础优化（DCE、CFG、Dominance）基于新 use-def。
- MIR 是否回归：待评估是否以 IR 直接面向后端。
- 增加 Debug 检查覆盖范围（对 Phi/JumpTarget 链的更细粒度断言）。
- 扩展算术 Opcode：计划增补 `SMax/SMin/UMax/UMin/FMax/FMin` 归类于 BinOP，Flags 初期为 `NONE`（后续可加饱和/无需变换标记）。

## [0.1.0] - 初始
初始发布：基本 IR 结构、指令与测试案例。

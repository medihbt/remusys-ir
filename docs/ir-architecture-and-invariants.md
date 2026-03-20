
# Remusys-IR：架构、内存管理与不变式

本文为对 Remusys-IR 源码的调研报告，按主题分章说明项目总体架构、内存与资源管理范式、引用关系与释放规则、数据流与控制流的表示方式，以及关键不变式与实现注意事项。文末保留扩展框架与下一步可执行工作建议，便于后续补充逐文件注解或自动化检查输出。


## 设计注记（教学优先）

Remusys‑IR 的首要目标为教学与可读性：代码应便于学生、研究人员和贡献者理解 IR 的基本概念与实现细节，而不是追求并发性能或工程级别的高并发吞吐。基于这一设计立场：

- 项目在历史上曾做过有限的并发友好重构（例如移除若干 `Rc` 以允许在受控场景下通过短暂的 `Mutex` 借出访问），以便在极其有限的场合支持短时并发访问；但并不计划为多线程场景继续牺牲单线程可读性或性能。
- 因此，不建议在核心 IR 实现中引入进一步的多线程优化或复杂并发原语；若需要并行化，请在上层工具链或独立服务中实现，而非修改 Remusys‑IR 的核心数据结构。
- `base/weak_list.rs` 当前作为历史/参考代码保留：IR 的核心模块已不再依赖 `WeakList`，其保留目的是为教学、对比与研究而非作为推荐的运行时数据结构。

我将在文档其余部分把 `WeakList` 视为“历史/备用工具”，并在必要处注记其非核心地位，避免误导读者以为它是当前推荐的实现路径。

## 目录

- **总体概览**
- **内存管理与池化**
- **引用关系与 Use/User 体系**
- **数据流与控制流表达**
- **弱引用与混合引用工具**
- **核心不变式与检查点**
- **GC/回收策略概览**
- **常见风险点与建议**
- **总结与后续工作**

## 总体概览

### 模块划分

源码按逻辑子系统组织：`base`（通用基础设施）、`ir`（核心 IR 表示与工具）、`opt`（变换/优化）、`typing`（类型系统）、`testing`（测试工具）等。`ir` 下进一步包含 `inst`、`module`、`jumping`、`usedef`、`utils` 等子模块。

### IR 对象模型

- 以 ID 为中心的对象模型：`InstID`, `BlockID`, `GlobalID`, `ExprID` 等。
- `IRAllocs`（位于 `module::allocs`）封装池/实体分配器，负责对象的分配与回收。
- `ValueSSA` 是统一的“值”表示：包含常量、表达式、指令、块、全局等变体，用于描述数据流边。

## 内存管理与池化

### 池化与 `mtb-entity-slab`

- 依赖 `mtb-entity-slab`，通过 `IPoolAllocated` 接口将不同类对象放入对应 pool。此设计利于集中管理内存、提高分配/释放效率，并在逻辑上将生命周期与 pool 绑定。

### 显式 Dispose 流程

- 释放以显式 `dispose_id` 为主。`ir::module::managing` 中包含一组辅助函数：
	- `traceable_dispose`：清理 `UserList` 等可追踪对象的用户信息。
	- `user_dispose`：释放 `User` 的 operands 并清理用户链表。
	- `inst_dispose`：断开指令与其父 block 链表，释放 operands 与 jump targets。
	- `global_common_dispose`：处理全局释放时的符号表和 pin-unpin。

这些 helper 会检查对象状态并返回 `PoolAllocatedDisposeErr` 以反映错误情况（如重复释放或借用冲突）。

### 自动 Drop 封装（RAII）

- `ir::managed` 提供 `Managed*` 类型（如 `ManagedInst`、`ManagedBlock`），它们在 `Drop` 中调用 `dispose_id`，便于在局部作用域自动释放资源。需要转移所有权时可用 `release()` 取消自动释放行为。

### 标记与幂等释放

- 对象通常带有布尔标志（如 `disposed`、`dispose_mark`），dispose helpers 会检查这些标志，避免重复 dispose。重复调用会返回 `AlreadyDisposed` 错误以保证幂等性和早期检测。

## 引用关系与 Use/User 体系

### Use / User / UserList 模型

- 采用经典 use-def 风格：`User`（指令/表达式/全局等）持有多个 `Use`（操作数）。
- `Use` 包含对某一 `ValueSSA` 的引用，并在目标的 `UserList` 中注册自己（链表结构，带哨兵）。

### 初始化与回填（ID 回填）

- 当创建某些可追踪实体时，需要把新生成的 ID 回填到已有的 `Use` 的 operand 字段，这里由 `traceable_init_id` 完成：遍历 `t.try_get_users()` 并将 `u.operand.set(self_id)`。

### 清理语义

- `traceable_dispose` 会清理 `users`（调用 `clean`）并释放哨兵；`user_dispose` 释放 `User` 的所有 `Use` 并调用 `traceable_dispose`。这确保释放顺序正确并尽量避免悬挂引用。

## 数据流与控制流表达

### `ValueSSA` 统一值表示

- 枚举变体包括 `ConstData`、`ConstExpr`、`AggrZero`、`FuncArg`、`Block`、`Inst`、`Global`。
- 提供工具方法：`get_valtype`, `can_trace`, `try_get_users`, `as_dyn_traceable` / `as_dyn_user` / `as_dyn_ptrvalue` 等，便于多态处理与检查。

### 指令、基本块与终结器

- 指令实现 `ISubInst` 接口（分散在 `ir/inst/*` 文件中），基本块由 `BlockObj` 表示，内部维护指令链表。
- 终结器（terminator）与跳转目标由 `ir/jumping` 模块建模，`JumpTargets`、`PredList` 等表示控制流边。

### Phi 与数据流一致性

- Phi 指令与类似需要在变换时维护 `Use` / `UserList` 的指令密切相关。所有变换必须保持 use-def 链表一致性，否则 `checking` 子模块（如 dominance / sanity）会检测到不正确状态。

## 弱引用与混合引用工具

### `MixRef` / `MixMutRef`

- 用于统一处理两类访问：固定引用（`&T` / `&mut T`）与运行时借用（`Ref` / `RefMut`）。方便在 pool/RefCell 混合场景下编写通用访问代码。

### `WeakList`

- 基于 `Rc/Weak` 的双向弱链表（带哨兵）。用于不希望强持有节点但仍需枚举/管理的场景。
- 支持 `push_front`/`push_back`、`move_all_to`、`move_to_if`、`clear` 等操作。析构时会遍历并调用节点的 `on_list_finalize`。

## 核心不变式与检查点

下面列出关键不变式、实现假设及对应的检查点：

### ID 与 pool 一致性

- 每个 `PrimaryID`/Slab ID（indexed backend）应在其对应 pool 中可解引用，或为约定的 null handle。`ISlabID`/`IPoolAllocated` 的 `deref`/`deref_mut`/`free` 都假定此不变式成立。

### 链表与哨兵完整性

- `UserList`, `EntityList`, `WeakList` 等依赖哨兵节点确保边界语义。实现假定不会出现已释放节点仍在链表中（`WeakList::drop` 会在异常情况下 panic）。

### Dispose 幂等性

- `disposed` / `dispose_mark` 标志保证释放幂等，重复 dispose 会返回错误以避免二次释放。

### Use/User 一致性

- 在更改某个 operand 或替换 value 时，必须同时维护目标的 `UserList`。例如 `traceable_init_id` 的回填机制正是为保证这类同步而设计的。

### 父子关系一致性

- 指令的父 block 与 block 中的指令链表位置必须一致；`inst_dispose` 在断开链表时依赖成功的 `node_unplug` 操作，失败会触发 `expect`。

### 符号表 pin/unpin 约束

- 释放全局对象前需检查并解除符号表 pin（`module.symbol_pinned(id)`），否则可能留下悬挂符号引用。

## 项目中的 GC / 回收策略概览

### 实现概述：完整的 Mark–Sweep

Remusys-IR 实际上实现了一个完整的 mark–sweep 垃圾回收流程，用于清理 IR 层面的死对象（以 pool/slab 为后端）。关键要点：

- **标记（Mark）**：由 `IRMarker` 驱动。回收周期由 `Module::begin_gc()` 发起（它会先调用 `allocs.free_disposed()` 清理已排队的 disposed），然后创建 `IRMarker` 并由各 root（例如符号表中的 pinned symbol）向 marker 推送初始标记。`IRMarker::mark_all()` 会通过 `mark_queue` 广度优先遍历活对象并把它们加入 `IRLiveSet`。

- **扫描/遍历策略**：扫描过程中 `IRMarker` 会根据对象类型消费 block/inst/expr/global/use/jump-target 等，并将它们的子对象（例如 instruction 的 operands，block 的指令列表、function 的 blocks/args）推入标记队列，以确保可达对象全部被标记。

- **清理（Sweep）**：`IRLiveSet::sweep` 对未标记的对象做两类操作：一是对部分对象（例如 `Use`、`JumpTarget`）调用 `dispose_obj` 并把它们放入 disposed 队列以触发 dispose helper 的清理逻辑；二是对其他对象（如 inst/blocks/exprs/globals）直接按位判断并调用实体池的 `fully_free_if` 来释放不在 live set 中的项。最终通过 `IRMarker::finish()` 合并完成并记录释放数量。

### GC 中的“修复”与错误处理策略

- **有限修复**：在 sweep 过程中，GC 会尽其所能通过调用 `dispose` 等 helper 去清理或断开被回收对象与其它对象的关联（例如把孤立的 `Use` 从 use-def 环上移除），这在一定程度上能修复轻微的不一致（例如残留的使用链条）。

- **断言与放弃**：实现中包含若干 debug 断言（例如 `IRMarker::consume_block` 与 `consume_global` 中对 parent/attachment 的断言）和 `debug_assert!` 检查；若出现严重的不变式破坏（例如 block/inst 的父子关系异常），GC 不会尝试作复杂修复，而是触发断言/panic，从而直接中止运行以引导开发者定位并修复根本问题。换言之：GC 会做可控的局部修复，但遇到结构性破坏会选择放弃并暴露错误而非隐式掩盖。

### 在实现中可观察到的要点

- `IRLiveSet` 负责维护不同 pool（insts/blocks/exprs/globals/uses/jts）的活性位集合；它的构造基于 `IRAllocs` 的当前容量以确保 index 映射一致。
- `IRMarker` 通过内部代理 `IRMarkerRef` 在标记遍历中避免重复可变借用问题，并在 `do_push_mark` 中保证对象只会被标记一次。
- `IRLiveSet::sweep` 在处理 `Use` 与 `JumpTarget` 时会优先调用 `dispose_obj` 并把它们加入 `allocs` 的 disposed 队列（这些 dispose 会进一步断开链表和引用），随后调用 `free_disposed()` 以保证 disposed 对象的完整清理，再对其它池使用 `fully_free_if` 做最终释放。

### 结论（关于 GC 的语义）

Remusys-IR 的回收并非仅靠手工 dispose 或仅依赖 RAII：它提供了一个协同的回收机制——显式 dispose + 局部 RAII + 系统级的 mark–sweep。GC 既会主动回收不可达对象，也会在必要时调用现有的 dispose 逻辑来修复引用关系，但在遇到严重不变式破坏时会选择中止以暴露问题。

## 常见风险点与建议

- 变换（如 `mem2reg`、死代码消除、指令/块重写）必须严格维护 `Use`/`UserList` 的一致性；建议在变换后运行 `checking::sanity`、`dominance` 等检查器以尽早发现错误。
- `WeakList::drop` 对已释放节点会 panic；在复杂移动或模块克隆场景中，应小心保证链表完整性或改进错误处理策略。
- 对 `Managed*` 的使用要注意所有权转移：当 ID 被转移到其他持有者时必须调用 `release()`。
- Dispose 过程中若出现借用冲突（例如 `symbols.try_borrow_mut()` 失败），会返回 `PoolAllocatedDisposeErr::SymtabBorrowError`；避免在同一作用域内多次可变借用同一 allocs。

## 总结与后续工作

Remusys-IR 采用 pool/slab 分配+显式 dispose 的混合资源管理模型，结合 Use/User 链表与哨兵数据结构保证变换时的一致性。核心挑战来自于在变换中稳定维护 use-def 链表、避免重复或错误释放、并保证链表/父子关系的一致性。

后续可做项（可选）：

- 为每个 `ir/inst/*.rs` 与 `ir/module/allocs.rs` 撰写逐文件注解，列出字段、关键不变式与典型使用示例（易于审阅与贡献者上手）。
- 在 CI 或本地运行 `checking` 子模块（`sanity` / `dominance`）并把输出附到本报告以便回归验证。
- 为常见变换添加单元测试模板与断言，自动在变换后运行不变式检查。


**核心不变式与检查点**
以下是不变式与实现层面需要关注的关键点（实现中有显式检查或依赖这些假设）：
- **ID 与 Pool 一致性**: 一个 `PrimaryID`/`*ID`（indexed backend）在对应 pool 中必须可被 deref（或为特定 null handle）。`ISlabID`/`IPoolAllocated` 的 `deref`/`deref_mut`/`free` 期望 handle 与 slab/pool 保持一致。
- **哨兵与链表完整性**: `UserList`、`EntityList`、`WeakList` 等采用哨兵节点。实现假定链表在正常运行时不会出现孤立的已释放节点；`WeakList::drop` 在遇到已释放节点会 panic（代码中有相应检测）。
- **dispose 的幂等性**: `inst_dispose` 与 `global_common_dispose` 会检查 `disposed` / `dispose_mark`，并在重复 dispose 时返回错误以避免双重释放的未定义行为。
- **Use/User 一致性**: 在初始化/改变某个 Value 的 ID 时，要保证 `Use.operand` 与目标 `UserList` 同步（见 `traceable_init_id`），否则可能出现 use 指向旧 ID 导致检查失败。变换中必须在更改 operand 前后正确维护 `UserList`。
- **父子关系一致性**: 指令与块的父关系（`inst.get_parent()` 返回的 block ID）必须与 block 中的 instruction 链表位置一致；`inst_dispose` 在断开链表时有断言（`expect("Failed to unplug...")`），说明函数调用者应保证在非 sentinel 情况下父链表是可修改的。
- **Symbol table pinning**: `global_common_dispose` 须知符号可能被 pin（`module.symbol_pinned(id)`），在释放全局前需要先从 symbol table 中解除 pin，以避免悬挂符号引用。

**项目中可见的 GC/回收策略**
- Remusys-IR 更像是“显式资源管理 + pool/dispose”的混合系统，而不是传统的 tracing GC。GC 行为体现在：
	- 使用池（entity slab）以批量/集中管理内存并提供高效的分配/回收。
	- 通过 `Managed*` 的 Drop 封装提供 RAII 式的自动释放（受作用域控制）。
	- 通过若干 dispose helper 在释放时递归清理关联结构（uses 列表、操作数、jump targets、entity lists），以保证不留悬挂引用。

**常见风险点与建议**
- 变换（如 mem2reg、DCE、重写指令/块）必须非常小心地维护 `Use` / `UserList` 与 `User` 内部的状态。推荐在关键变换后运行 `checking::sanity` / `dominance` 等检查器以早发现不一致。
- `WeakList::drop` 中遇到已释放节点会 panic；若存在并发或复杂移动场景（例如跨 module move/clone），请确保移动逻辑在所有路径上保持链表完好。
- 对 `Managed*` 的自动释放要谨慎使用：当需要将 ID 交由其它所有者管理时，必须 `release`，否则会导致提前释放。
- 在 dispose helpers 中，若 pool 的借用产生冲突（例如 `symbols.try_borrow_mut()` 失败），会返回 `PoolAllocatedDisposeErr::SymtabBorrowError`；在复杂场景中请避免在同一作用域中反复可变借用同一 allocs 结构。

**总结**
Remusys-IR 通过一套基于 slab/pool 的实体分配、显式 dispose 接口与少量 RAII 包装器来实现内存与资源管理。其引用体系基于 classic use-def（`Use`/`User`/`UserList`），并用哨兵链表、弱链表等数据结构在性能与语义上达成折衷。要保证变换正确性，最关键的是严格维护 Use/User 的一致性、在释放前解除所有引用并遵循已定义的 disposed 标志约束。建议在变换实现中广泛使用现有的检查器，并在需要的地方添加更严格的断言与测试用例。

---

如果你希望我把报告扩展为更详细的逐文件注解（例如逐个说明 `inst/*.rs`、`module/allocs.rs` 中的字段与不变式），或者自动运行现有检查器并收集输出以丰富报告，我可以继续执行这些步骤。

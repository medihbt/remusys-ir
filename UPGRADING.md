# 升级指南：0.1.x → 0.2.0 架构重写

本指南帮助从旧版 (`0.1.x`) 迁移到重写后的 `0.2.0`。若你仅使用最新版本，可将其视为设计说明补充。

## 核心变化一览

- 内存池替换: 除了类型系统外, 池分配的 IR 对象从 `Slab` 切换到具有内部可变性的 `MTB::Entity`.
- 统一生命周期：统一池对象的分配、释放与回收逻辑, 所有池对象实现 `IPoolAllocated`，分配即回填自引用，处置幂等 + 延迟释放。
- Use-Def 和 JumpTarget 引用系统改写：二者均从 `WeakList` 切换到基于 `MTB::Entity` 的 `EntityRingList`.
- GC：移除标记-压缩 GC, 仅仅保留标记-清除 GC. 同时新增 “内存池不收缩” 约定.

## 旧 → 新 主要符号映射
| 旧名称 (0.1.x)                | 新名称 (0.2.0)                     | 备注 |
|-------------------------------|------------------------------------|------|
| `InstRef`, `BlockRef`         | `InstID`, `BlockID`                | ID 统一，直接是池指针包装 |
| `GlobalRef`, `VarRef`, `FuncRef` | `GlobalID`, `FuncID`              | 函数即 `GlobalID` 的子类型 |
| `ExprRef` / 常量复合对象       | `ExprID` + 具体 `ArrayExprID` 等    | 表达式仍走统一池 |
| `ValueSSAClass`               | `ValueClass`                       | 简化分类枚举 |
| `UseDef` / `user.rs`          | `usedef.rs` 中 `UseID`, `UserList` | 环链表示, 替换为 Entity 实现 |
| Terminator + 目标混合结构      | `JumpTargetID` + `JumpTargets`     | 终结指令内聚合 `JumpTargetID` 列表 |
| MIR (`src/mir/*`)             | 已删除                             | 暂不再提供机器相关 IR. RIG 语言会被重新定义和实现 |
| 优化 pass (`opt/*`)           | 已删除                             | 以后将重建于新架构 |
| 属性集合 (`AttrList`, `AttrSet`)| 暂移除                             | 将来若需要重新设计 |
| `compact_ir/*`                | 已删除                             | 不再维护紧凑表示 |

## 构建器迁移示例
旧：

```rust
use remusys_ir::ir::{*, inst::*};

let mut builder = {
    let tctx = TypeContext::new_rc(ArchInfo::new_host());
    IRBuilder::new(Module::new("name", tctx))
};
let ri32fty = FuncTypeRef::new(builder.type_ctx(), ValTypeID::Int(32), false, []);
let main_func = builder
    .define_function_with_unreachable("main", ri32fty)
    .unwrap(); // automatically set focus to `main`

let entry_block_0 = builder.full_focus.block;
builder.set_focus(IRFocus::Block(entry_block_0));
let add_id = builder.add_binop_inst(
    Opcode::Add,
    APInt::new(1u32, 32).into(),
    APInt::new(2u32, 32).into(),
).unwrap();

builder.module.gc_cleaner().compact([]);
```
新：
```rust
// Remusys-IR 唯一推荐使用这种导入方式.
use remusys_ir::ir::{*, inst::*};

let mut builder = IRBuilder::new_inlined(arch, "foo_mod");
let functy = FuncTypeID::new(builder.tctx(), ValTypeID::Int(32), false, []);
let foo = FuncID::builder(builder.tctx(), "foo", functy)
    .make_defined()
    .terminate_mode(FuncTerminateMode::ReturnDefault)
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

旧：`UserList` 自动环链使用基于 `Rc` / `Weak` 的 `WeakList<T>`.
新：`UserList` 自动环链切换到基于 `MTB::Entity` 的环链 `EntityRingList`, 方便多线程访问. 同时, 为 UseID
    编写了新的 API.

新旧代码除了命名以外差异不大.

```rust
let use_id = UseID::new(builder.allocs(), UseKind::BinOpLhs);
use_id.set_operand(builder.allocs(), ValueSSA::Inst(add_id.into_instid()));
```

## 跳转与 CFG

旧：`PredList` 自动环链和 Use 一样基于 `WeakList`. 新版变更同样是内存管理模式变更, 具体逻辑区别不大.
新：
- `JumpTargetID` 对象持有 `terminator: Option<InstID>` 与 `block: Option<BlockID>`。
- 块维护 `preds: PredList`（环链哨兵）。

注意：基本块必须满足“单 terminator 且位于块尾”的结构, 因此 Remusys 几乎不允许在块的中间插入 terminator.
如果想做到类似的语义的话, 需要先调用 `builder.split_block()` 把当前基本块拆成两半 (此时焦点位于前一半),
然后直接调用 `builder.focus_set_***` 的终止子替换方法.

## IR 基本结构检查

在任意时刻均可以调用下面的方法检查 IR 的基本结构:

```rust
remusys_ir::ir::checking::assert_module_sane(&module);
```

如果 IR 损坏, 则会输出损坏位置和一些基本错误信息. 如果需要获取详细的报错信息, 需要使用下面的方法获取检查报告:

```rust
match remusys_ir::ir::checking::basic_sanity_check(&module) {
    Ok(()) => {},
    Err(e) => { /* Handle checking failures */ },
}
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

## 访问旧分支

原 master 分支的代码仓库将被切换到新分支 `old-with-slab`. 与 MIR 相关的内容均位于该旧代码库.

## FAQ
- 为什么移除 MIR？
  重构聚焦底层 IR 的引用/生命周期稳定；中间层将在确认新核心稳定后再决定是否回归。
- 为什么不保留旧属性系统？
  旧属性扩展性不足，等待重新设计与语义规范。
- GC 为什么“先边后点”？
  确保在释放顶点前，所有引用边已经安全摘链，避免悬挂环链。

## 后续关注
欢迎在后续版本中补充：优化 Pass、常量折叠、属性系统 v2。

---
如遇迁移问题，可直接在代码处打 `panic!` 并逐段迁移，因当前项目仅供个人使用，不需维持对外兼容矩阵。

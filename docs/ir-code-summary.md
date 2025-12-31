# Remusys-IR 重要代码总结

2025 年我花了大半年时间编写了 Remusys-IR，这是一个用于表示和优化中间表示（IR）的库，类似于 LLVM IR。项目代码总量 24,000 行, 非常巨大, 倘若交给人来调研至少要花费大半天时间. 下面的文档由 Copilot 辅助生成.

本节整理 Remusys-IR 核心源码中的“骨架”代码与关键调用关系，帮助 AI/工具在不通读全仓的情况下了解真实实现，避免想象不存在的 API。本总结按模块顺序列出代表性片段，并解释它们背后的语义、不变式与使用方式。

## 1. 顶层模块与符号表

- `src/lib.rs` 将代码分为 `base`、`ir`、`opt`、`testing`、`typing` 五大子系统；`ir` 模块再导出 attribute/block/inst/module/usedef/utils 等子模块，构成 IR 栈的主入口。
- `src/ir/module.rs` 定义真正的 `Module` 结构：聚合 `IRAllocs`、`TypeContext`、`SymbolPool`，并提供 `begin_gc()/free_disposed()`、`symbol_pinned()`、`get_global_by_name()` 等接口；`SymbolPool` 负责全局符号的 pin/export 语义，并在 GC 标记期调用 `gc_mark()` 把所有 pin 的函数/变量推入 root。

> 摘自 src/ir/module.rs：
````rust
pub struct Module {
	pub allocs: IRAllocs,
	pub tctx: TypeContext,
	pub symbols: RefCell<SymbolPool>,
	pub name: String,
}

impl Module {
	pub fn begin_gc(&mut self) -> IRMarker<'_> {
		self.allocs.free_disposed();
		let mut marker = IRMarker::new(&mut self.allocs);
		self.symbols.get_mut().gc_mark(&mut marker);
		marker
	}
}
````

## 2. 实体池 (`IRAllocs`) 与自动释放

### 2.1 集中分配、延迟回收

- `src/ir/module/allocs.rs` 中的 `IRAllocs` 统一管理表达式、指令、全局、块、`Use`、`JumpTarget` 六种实体，并维护一个 `disposed_queue`。显式 dispose 只把 ID 推入队列，统一调用 `free_disposed()` 才真正释放，实现“批量回收 + 缓冲池”策略。

> 摘自 src/ir/module/allocs.rs：
````rust
pub struct IRAllocs {
	pub exprs: ExprAlloc,
	pub insts: InstAlloc,
	pub globals: GlobalAlloc,
	pub blocks: BlockAlloc,
	pub uses: UseAlloc,
	pub jts: JumpTargetAlloc,
	pub disposed_queue: RefCell<VecDeque<PoolAllocatedID>>,
}

pub fn free_disposed(&mut self) {
	while let Some(id) = self.disposed_queue.get_mut().pop_front() {
		match id {
			PoolAllocatedID::Inst(i) => i.into_raw_ptr().free(&mut self.insts),
			// 省略 Block/Expr/Global/Use/JT 的分支
			_ => {}
		}
	}
	// 根据总实体量缩容 queue，避免长期占用大缓冲
}
````

### 2.2 `IPoolAllocated` 模式

- 每类实体实现 `IPoolAllocated`：提供分配、初始化、自清理（`dispose_obj`）、幂等检查逻辑。`dispose_id()` 会调用实体的 `dispose_obj()`，再把 ID 丢进 `disposed_queue`。

> 摘自 src/ir/module/allocs.rs：
````rust
pub(crate) trait IPoolAllocated: Sized {
	type PtrID: IPoliciedID<ObjectT = Self> + Into<PoolAllocatedID>;
	type MinRelatedPoolT: AsRef<IRAllocs>;

	fn allocate(allocs: &IRAllocs, obj: Self) -> Self::PtrID;
	fn dispose_obj(&self, id: Self::PtrID, pool: &Self::MinRelatedPoolT)
		-> PoolAllocatedDisposeRes;

	fn dispose_id(id: Self::PtrID, pool: &Self::MinRelatedPoolT)
		-> PoolAllocatedDisposeRes {
		let alloc = Self::get_alloc(pool.as_ref());
		let Some(obj) = id.into_backend().try_deref(alloc) else {
			return Err(PoolAllocatedDisposeErr::AlreadyDisposed);
		};
		obj.dispose_obj(id, pool)?;
		pool.as_ref().push_disposed(id);
		Ok(())
	}
}
````

### 2.3 `Managed*` RAII 包装器

- `src/ir/managed.rs` 把任意 `IPoolAllocated` 对象包成 `ManagedInst`、`ManagedBlock` 等 RAII 结构；`Drop` 自动调用 `dispose_id`，需要转移所有权时调用 `release()`。

````rust
pub struct ManagedInst<'ir> {
	inner: IRManagedImpl<'ir, InstObj>,
}

impl<'ir> ManagedInst<'ir> {
	pub fn new(pool: &'ir IRAllocs, id: InstID) -> Self {
		Self { inner: IRManagedImpl::new(pool, id) }
	}
	pub fn release(self) -> InstID { self.inner.release() }
}
````

## 3. Use/User 链与 `ValueSSA`

### 3.1 统一值枚举

- `src/ir/ir.rs` 把所有 SSA 值抽象成 `ValueSSA`，并提供 `get_valtype()`、`try_get_users()`、`as_dyn_traceable()` 等助手，便于在 pass 中统一处理常量/指令/块/全局。

````rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueSSA {
	None,
	ConstData(ConstData),
	ConstExpr(ExprID),
	AggrZero(AggrType),
	FuncArg(FuncID, u32),
	Block(BlockID),
	Inst(InstID),
	Global(GlobalID),
}

impl ISubValueSSA for ValueSSA {
	fn get_valtype(self, allocs: &IRAllocs) -> ValTypeID { /* 根据变体派发 */ }
	fn try_get_users(self, allocs: &IRAllocs) -> Option<&UserList> { /* 仅对可追踪值返回 */ }
}
````

### 3.2 `Use` 与 `UserList`

- `src/ir/usedef.rs` 定义 `Use` 节点（带 `UseKind`）、`UserID`、`ITraceableValue`/`IUser` 两个 trait。Use 以环形链表 + 哨兵形式连接 value 与 user，实现经典 use-def 体系。
- `Use::set_operand()` 先把节点从老 value 的 list 中 `detach`，再把新的 value 加入用户链；`Use::dispose()` 会标记 `UseKind::DisposedUse` 并清空 user/operand，保证 dispose 幂等。
- `ITraceableValue::replace_self_with()` 支持“用新值替换所有引用该 Value 的 Use”，变换 pass (mem2reg、DCE) 依赖它维持 use-def 一致性。

````rust
pub struct Use {
	list_head: Cell<EntityListNodeHead<UseID>>,
	kind: Cell<UseKind>,
	pub user: Cell<Option<UserID>>,
	pub operand: Cell<ValueSSA>,
}

impl ITraceableValue for InstCommon {
	fn users(&self) -> &UserList { self.users.as_ref().unwrap() }
	fn replace_self_with(&self, allocs: &IRAllocs, new_value: ValueSSA)
		-> Result<(), EntityListError<UseID>> { /* 见源码 */ }
}
````

## 4. IR 构建与编辑 (`IRBuilder`)

- `src/ir/utils/builder.rs` 提供高阶 API：管理“焦点”（函数/块/指令）、安全插入/拆分、批量构建终结器。`FocusDegradeConfig` 控制当焦点非法时的降级策略，减少崩溃。
- `split_block()` 组合 `get_split_pos()` 与链表操作，实现“保持 use-def + 更新焦点”的块拆分，phi 修复逻辑集中在 `split_block_at_end`。

> 摘自 builder：
````rust
#[derive(Debug, Clone, Copy)]
pub struct IRFullFocus {
	pub func: FuncID,
	pub block: Option<BlockID>,
	pub inst: Option<InstID>,
}

pub enum FocusDegradeOp { AsBlockOp, Strict, Ignore }

pub struct FocusDegradeConfig {
	pub add_phi_to_inst: FocusDegradeOp,
	pub add_inst_to_phi: FocusDegradeOp,
	pub add_terminator: FocusDegradeOp,
	// 其余配置项...
}
````

> 块拆分核心：
````rust
pub fn split_block(&mut self) -> IRBuildRes<BlockID> {
	match self.get_split_pos()? {
		IRFocus::Block(block) => self.split_block_at_end(block),
		IRFocus::Inst(inst) => self.split_block_at_inst(inst),
	}
}

fn split_block_at_end(&mut self, front_half: BlockID) -> IRBuildRes<BlockID> {
	let back_half = BlockID::new_uninit(self.allocs());
	self.focus_add_block(back_half)?;
	// 1. 用 Jump 接回后半块
	// 2. 修补原 terminator 的 use-def
	// 3. 若焦点在终结符上，重定位到新的 Jump
	Ok(back_half)
}
````

## 5. GC (`IRMarker` + `IRLiveSet`)

- `src/ir/module/gc.rs` 实现 mark-sweep：`IRMarker::push_mark()` 将 ID 入队，`mark_all()` BFS 扫描 block/inst/global/use/jt，并断言父子关系未破坏；`IRLiveSet::sweep()` 首先强制 dispose 所有 `Use`/`JumpTarget`，然后按位图批量 `fully_free_if`，最终产生日志。
- `Module::begin_gc()` 会在遍历前调用 `SymbolPool::gc_mark()`，确保 pin 的符号成为 GC root。

````rust
pub fn mark_all(&mut self) {
	while let Some(id) = self.mark_queue.pop_front() {
		match id {
			PoolAllocatedID::Block(b) => self.consume_block(b),
			PoolAllocatedID::Inst(i) => self.consume_inst(i),
			PoolAllocatedID::Expr(e) => self.consume_expr(e),
			PoolAllocatedID::Global(g) => self.consume_global(g),
			PoolAllocatedID::Use(u) => self.push_mark_value(u.get_operand(self.ir_allocs)),
			PoolAllocatedID::JumpTarget(jt) => {
				if let Some(bb) = jt.get_block(self.ir_allocs) { self.push_mark(bb); }
			}
		}
	}
}
````

## 6. 类型系统 (`typing`)

- `src/typing/mod.rs` 引入 `TypeContext`、`ValTypeID`、`IValType` trait，封装了 `get_size()/get_align()`、序列化（`TypeFormatter`）等操作，是构建 IR 时 `ValueSSA::new_zero()`、`FuncBuilder` 等函数的类型来源。
- `ValTypeID` 通过 `makes_instance()` 判断是否可构造值，`class_id()` 则映射到 `ValTypeClass`（Void/Ptr/Int/...）。

````rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum ValTypeID {
	#[default] Void,
	Ptr,
	Int(u8),
	Float(FPKind),
	FixVec(FixVecType),
	Array(ArrayTypeID),
	Struct(StructTypeID),
	StructAlias(StructAliasID),
	Func(FuncTypeID),
}
````

## 7. Pass/变换模块 (`opt`)

- `src/opt/mod.rs` 暴露 `analysis::{cfg, dfs, dominance}` 与 `transforms::{basic_dce, mem2reg}`。Pass 直接复用上述 use-def 与 builder 工具：先通过 `IRBuilder` 修改块/指令，再调用 `Use::replace_self_with()` 或 `ITraceableValue::clean_users()` 修复引用，最后可运行 `checking::sanity`/`dominance` 做验证。

## 8. 使用建议与常见坑

1. **优先使用 `Managed*` 或 `PoolAllocatedID::dispose()`**：忘记调用会让实体滞留在池里，`IRAllocs::free_disposed()` 不会自动触发。
2. **所有 IR 变换都要维护 use-def**：修改 operand 时务必使用 `Use::set_operand()` 或 `ITraceableValue::replace_self_with()`，否则 GC/检查器会在 `mark_all()` 触发断言。
3. **块/终结器更新走 `IRBuilder`**：它已经帮你处理 Phi 区段和焦点降级，直接操作链表容易破坏 `InstIDSummary` 的约束。
4. **释放全局前先处理符号表**：`global_common_dispose()` 会检查 `SymbolPool` 的 pin 状态；在遍历 symtab 时调用 dispose 会触发 `SymtabBorrowError`。
5. **运行 `Module::begin_gc().finish()` 前调用 pass**：GC 会把未标记实体全部释放，适合作为“变换后校验”步骤，也能暴露潜在的不变式破坏。

通过以上片段，AI/工具可以快速定位核心 API、理解内存/引用管理模型，从而在回答或生成补丁时引用真实代码而非臆测实现。

## 示例代码

一个最简单的函数构建示例：

```C
int max(int a, int b) {
    if (a > b) {
        return a;
    } else {
        return b;
    }
}
```

对应的 Remusys-IR 文本表示:

```remusys-ir
define dso_local i32 @max(i32 %0, i32 %1) {
2:
    %3 = icmp sgt i32 %0, %1
    br i1 %3, label %4, label %5
4:
    ret i32 %0
5:
    ret i32 %1
}
```

对应的 Rust 代码片段：

```rust
use crate::{ir::{*, inst::*}, typing::*};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut builder = IRBuilder::new_inlined(ArchInfo::new_host(), "max_example");
    let tctx = builder.tctx();
    
    let max_functy = FuncTypeID::new(
        tctx,
        /* return_type = */ ValTypeID::Int(32),
        /* is_vararg = */ false,
        /* arg_types = */ [ValTypeID::Int(32), ValTypeID::Int(32)],
    );
    let max_func = FuncID::builder(tctx, "max", max_functy)
        .make_defined()
        .terminate_mode(FuncTerminateMode::ReturnDefault)
        .build_id(&builder.module)
        .map_err(|existed_gid| format!("Function 'max' already exists with ID {existed_gid:?}"))?;
    let entry = max_func
        .get_entry(builder.allocs())
        .ok_or("Failed to get entry block")?;
    builder.set_focus(IRFocus::Block(entry));

    let arg_a = ValueSSA::FuncArg(max_func, 0);
    let arg_b = ValueSSA::FuncArg(max_func, 1);

    /* IR Builder 拆分基本块是逆序的 */
    let else_bb_5 = builder.split_block()?;
    let then_bb_4 = builder.split_block()?;

    let cmp_3 = builder.build_inst(|allocs, _tctx| {
        let cmp_inst_id = CmpInstID::new_uninit(
            allocs,
            Opcode::Icmp,
            CmpCond::SGT,
            /* operand type */ ValTypeID::Int(32),
        );
        cmp_inst_id.set_lhs(allocs, arg_a);
        cmp_inst_id.set_rhs(allocs, arg_b);
        cmp_inst_id
    })?;
    builder.focus_set_branch_to(cmp_3.raw_into(), then_bb_4, else_bb_5)?;

    // 演示 Builder 自动替换终止子的功能
    builder.set_focus(IRFocus::Block(then_bb_4));
    builder.build_inst(|allocs, _tctx| RetInstID::with_retval(allocs, arg_a))?;

    // 演示 Builder 自动替换终止子的功能
    builder.set_focus(IRFocus::Block(else_bb_5));
    builder.build_inst(|allocs, _tctx| RetInstID::with_retval(allocs, arg_b))?;

    let module = builder.module;
    write_ir_to_file("max.ll", &module, IRWriteOption::quiet());
    Ok(())
}
```

### 练习

修改逻辑：如果 return a 换成 return a + 1, 那 IR 构建器该怎么改？

下面提供 BinOP 相关的 API:

```rust
pub struct BinOPInst { /* 不可见字段 */ }

impl BinOPInst {
    pub const OP_LHS: usize = 0;
    pub const OP_RHS: usize = 1;

    pub fn new_uninit(allocs: &IRAllocs, opcode: Opcode, ty: ValTypeID) -> Self {
        ...
    }

    pub fn lhs_use(&self) -> UseID {
        self.operands[Self::OP_LHS]
    }
    pub fn get_lhs(&self, allocs: &IRAllocs) -> ValueSSA {
        self.lhs_use().get_operand(allocs)
    }
    pub fn set_lhs(&self, allocs: &IRAllocs, val: ValueSSA) {
        self.lhs_use().set_operand(allocs, val);
    }

    pub fn rhs_use(&self) -> UseID {
        self.operands[Self::OP_RHS]
    }
    pub fn get_rhs(&self, allocs: &IRAllocs) -> ValueSSA {
        self.rhs_use().get_operand(allocs)
    }
    pub fn set_rhs(&self, allocs: &IRAllocs, val: ValueSSA) {
        self.rhs_use().set_operand(allocs, val);
    }
}

// Remusys-IR 的所有指令 ID 都有 raw_from() raw_into() 方法和 InstID 互相转换,
// 有 into_value() -> ValueSSA 方法把指令装箱成 SSA 值.
_remusys_ir_subinst!(BinOPInstID, BinOPInst);
impl BinOPInstID {
    pub fn new_uninit(allocs: &IRAllocs, opcode: Opcode, ty: ValTypeID) -> Self {
        let inst = BinOPInst::new_uninit(allocs, opcode, ty);
        Self::allocate(allocs, inst)
    }
    pub fn new(allocs: &IRAllocs, opcode: Opcode, lhs: ValueSSA, rhs: ValueSSA) -> Self {
        let inst_id = Self::new_uninit(allocs, opcode, lhs.get_valtype(allocs));
        let inst = inst_id.deref_ir(allocs);
        inst.set_lhs(allocs, lhs);
        inst.set_rhs(allocs, rhs);
        inst_id
    }

    ...
}
```

还有 APInt 相关的 API:

```rust
#[derive(Copy)]
pub struct APInt { /* 不可见字段 */ }

impl From<i32> for APInt {
    /* 得到的 APInt 类型是 ValTypeID::Int(32). 实际上 APInt 可以从标准库的任何整数类型创建，并且会给该整数自己的类型 */
}

impl From<APInt> for ValueSSA { /* 自动装进 ValueSSA 里 */ }
```

完成这份练习.
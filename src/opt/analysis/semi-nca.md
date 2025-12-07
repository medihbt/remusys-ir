<!--
Semi-NCA (Semi-dominator / Semi-NCA) 算法设计与迁移说明
适配目标：在已重构的控制流表示（`CfgCache` / `CfgSnapshot` / `CfgDfsSeq` 等）上重实现 dominator tree 构建。
作者：自动生成（由开发者实施具体代码）
-->

# Semi-NCA 算法：设计报告与实现计划

**目标**：将原有基于旧 CFG 骨架的 Semi-NCA dominator 构建逻辑，迁移并适配到新的控制流层（`CfgCache` / `CfgSnapshot` / `CfgDfsSeq`）。文档包含算法回顾、与新 CFG API 的映射、实现要点、不变式、测试清单以及分步实现计划。

**背景**：Remusys-IR 之前使用 Semi-NCA（半支配 / 半最近公共祖先变体）算法来构建支配树（dominator tree）。该算法的核心思想是：
- 使用 DFS（通常是前序/访问/父亲关系）为基本块编号（dfn），并构造 DFS 树（parent 指向父 dfn）。
- 逆序遍历 DFS 编号（从后往前）来计算每个节点的 semidominator（半支配结点），依赖于对前驱节点的查询以及并查集（DSU）用于路径压缩与“最佳候选”维护。
- 在求出半支配信息后，通过逐步修正获得最终的直接支配（idom）。

本次迁移需注意：新的控制流实现将基础查询改为基于 `CfgCache` / `CfgSnapshot`（按需或一次性快照提供 pred/succ 列表），并且 Block 的标识为 `BlockRef`。另外存在虚根/虚出（virtual entry/exit）与不可达块的问题需要明确处理。

**假定的/建议的 API（来自代码库）**
- `CfgSnapshot`：对函数 CFG 的不可变、按需或一次性快照视图。
  - `block_iter()` / `body.blocks`：原始块顺序（可用于 stable snapshot）。
  - `block_get_prev(block: BlockRef) -> Option<&[(usize, BlockRef)]>`：按设计，返回前驱列表（若为 None 表示没有前驱或未记录）。
  - `block_get_next(block: BlockRef) -> Option<&[(usize, BlockRef)]>`：后继列表（若需要后序/后支配）。
- `CfgDfsSeq`：基于 snapshot 构建的 DFS 序列与查询：
  - `new_from_snapshot(snapshot, DfsOrder::Pre)`：构建 DFS（pre）序列
  - `block_get_dfn(block: BlockRef) -> Option<usize>`：Block -> dfn 映射
  - `dfn_get_parent_dfn(dfn: usize) -> Option<usize>`：DFS 树父 dfn
  - `dfn_get_block(dfn: usize) -> Option<BlockRef>`：dfn -> Block
  - `n_logical_nodes()` / `get_root()` 等

若当前 API 与上述不完全一致，在实现之前需补充或适配这些访问器（尤其是前驱/后继和 DFS 映射）。

算法实现要点（高层步骤）
1. 建立 DFS 序列与编号
   - 使用 `CfgDfsSeq::new_from_snapshot(snapshot, DfsOrder::Pre)`（或等价接口）从入口（root）构建 DFS，并得到每个块的 dfn 与父关系。
   - DFS 只应覆盖“可到达”节点；不可达块不会进入 dfs_seq，后续应当视为 unreachable（不参与 dominator 构建）。

2. 数据结构准备
   - `N = dfs_seq.n_logical_nodes()`。分配数组/Vec：
     - `semidom: Vec<usize>` 初始为 0..N-1
     - `best_candidate: Vec<usize>` 初始为 0..N-1
     - `dsu: DSU`（并查集，支持自定义 find/find_when 用于路径压缩与维护 best_candidate）
     - `idom: Vec<usize>`/`idom_block: Vec<BlockRef>` 最终输出

3. 计算 semidominators（逆序遍历）
   - 对 u 从 N-1 降到 1（通常根 dfn==0 跳过）：
     - 遍历 u 的所有前驱 v（注意：前驱可能不在 dfs_seq 中 → 忽略不可达前驱）。
     - 若 v 在 dfs_seq 中，用 `v_dfn = dfs_seq.block_get_dfn(v)` 获取编号。
     - 对于每个 v_dfn 调用 `dsu.find_when(v_dfn, |x, old_parent, _| { ... })`（或等效），并利用 `best_candidate` 与 `semidom` 比较，维护 `best_candidate`。
     - 计算 `res = min( v_dfn (if v_dfn < u), semidom[best_candidate[v_dfn]] (if v_dfn >= u) )`，最终 `semidom[u] = res`。
     - 将 u 插入 DSU 树：`dsu.set_direct_parent(u, parent_of_u)`（parent 由 DFS 树提供）。

4. 写回 semidom 到块映射
   - 遍历 semidom 数组，把 semidom 的 dfn 映射回 `BlockRef` 并存入节点数据结构中（如 `nodes[u].semidom_block`）。

5. 计算 idom
   - 从 1..N-1 顺序遍历每个 w：
     - 初始化 `idom = parent(w)`，当 `idom > semidom[w]` 时，循环设置 `idom = idom(idom)`（一直向上直到满足条件）
     - 最终 `idom[w]` 即直接支配结点的 dfn（映射回 BlockRef 写入节点）。

6. Postdom / 多出口情形处理
   - 对 post-dominator（后支配），应当对 CFG 取反向（或对退出节点做虚根合并）：
     - 构造 exit 集合；若函数有多个退出点，创建一个虚出口 `vexit` 并把所有真实退出连到它。
     - 在构建 semidom 时，`get_pred` 的语义需要改为“基于后向图的前驱”（即原图的后继）。

实现细节与不变式（必须断言/检查）
- DFS parent 在构建过程中只会被设置一次；对同一节点 parent 的多次设置应触发 panic/断言。
- 任何用到 `block_get_dfn(block)` 的地方，都必须处理 `Option`：仅对可达节点（Some）进行 semidom/idom 计算。
- `dsu` 的 `find_when` 回调要保证对 `best_candidate` 的更新是安全的（按原论文/代码实现保持顺序比较 semidom 值）。

复杂度 & 性能
- Semi-NCA 的经典复杂度为 O(E α(N))（几乎线性），其中 α 是反阿克曼函数。
- 在新 CFG 中，访问前驱/后继的开销取决于 `CfgSnapshot` 的实现：若 snapshot 已缓存好前驱数组则为 O(1)；如果是按需查询（`CfgCache`）则可能包含额外构建成本。建议在 dominator 构建时使用稳定的快照（一次性构建前驱表）以避免重复开销。

单元测试与验证用例
- 基础线：简单线性 CFG（A->B->C）→ idom 顺序验证。
- 分叉合并：A->B,C; B->D; C->D → 检查 D 的 semidom 与 idom。
- 环：带循环的 CFG（A->B->C->B, C->D）→ 验证算法在环中能正确停止与返回一致结果。
- 多出口：多个 return 点 → 使用虚出口测试 postdom 变体。
- 不可达块：函数体中插入不可达块，确保它们不进入 dfs_seq，且不影响其它节点。

迁移潜在风险与注意项
- 前驱/后继的顺序语义：原实现可能假定某种顺序（例如 terminator 中的 succ 顺序）；若新 snapshot 通过 BTreeSet 或类似结构去重并排序，可能改变顺序，但 Semi-NCA 不依赖 succ 的排列顺序，只依赖集合语义（一般安全）。
- 在大型函数上，若 snapshot 每次构建都做大量分配，可能导致显著开销。可考虑在 `CfgCache` 中提供一次性构建并复用的 API（例如 `to_indexed_snapshot()`）。

分步实现计划（按优先级）
1. 补齐/确认 API：确保 `CfgDfsSeq`、`CfgSnapshot`（或等价）能提供上述必需查询。若缺失，先增加最小访问器。
2. 在 `src/opt/analysis/` 新增/移植 `semi_nca.rs`（或在 `dominance.rs` 中重构）：实现核心算法，遵循文档中的步骤并保持接口清晰。
3. 编写单元测试：按上面的测试用例列表逐一实现。优先从小函数、人工构造 CFG 开始。
4. 性能/内存检视：在大函数上跑基准（或 rust `cargo test --release`）以观测内存与执行时间，确认快照策略是否需调整。
5. 防守式编程：加入断言以确保 dfns、parent 只被正确地设置一次，并在遇到不可达前驱时记录并跳过。

开发交付物（后续编码时参考）
- `src/opt/analysis/semi_nca.rs`（或整合到 `dominance.rs`）
- 单元测试文件：`src/opt/analysis/tests/dominance_tests.rs`
- 文档：本 `semi-nca.md`（实现完成后更新为包含 API 使用示例）

结语 / 下一步
1. 请确认是否接受本设计与计划（或指出希望优先的测试用例）。
2. 若确认，我会根据上述计划开始编码：先补齐必要的 snapshot/dfs API（若缺失），然后实现算法并提交单元测试。

-- End of Document

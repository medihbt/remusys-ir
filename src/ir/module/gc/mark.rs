use std::cell::RefCell;

use slab::Slab;

use crate::{
    base::{NullableValue, slablist::SlabRefListNodeRef, slabref::SlabRef},
    ir::{
        ValueSSA,
        block::jump_target::JumpTargetRef,
        constant::expr::{ConstExprData, ConstExprRef},
        global::{GlobalData, GlobalRef, func::FuncData},
        inst::{terminator::TerminatorInst, usedef::UseRef, *},
        module::{Module, ModuleError},
    },
};

use super::liveset::IRRefLiveSet;

pub(super) struct MarkVisitor<'a> {
    pub(super) inner: RefCell<MarkVisitorInner>,
    pub(super) module: &'a Module,
}

pub(super) struct MarkVisitorInner {
    pub(super) live_set: IRRefLiveSet,
    pub(super) mode: MarkMode,
}

#[derive(Debug, Clone)]
pub(super) enum MarkMode {
    /// Non-compact mode: keep the original order of references.
    NoCompact,

    /// ### 压缩模式
    ///
    /// 如果启用压缩选项并指定了空间预留函数, 该标记器将以函数体为单位为每组指令、
    /// 基本块等预留一定量的空间, 以优化内存局部性.
    ///
    /// #### 压缩后的对象排序规则与空间预留规则
    ///
    /// 该标记器在压缩模式下, 第一次查找活跃引用时就会为对象分配新位置. 由于 Module
    /// 中不同类型的 ValueSSA/Use/JumpTarget 对象都存储在不同的 `Slab` 分配器中,
    /// 因此压缩后这些对象的排序规则可能不同.
    ///
    /// * **全局量**: 全局量按照 `module.global_defs` 哈希表中的顺序排序. 由于
    ///   预留选项中没有针对全局量的选项, 因此全局量的排序规则与预留规则无关.
    /// * **基本块**: 保证同一个函数体中的基本块连续排布成组, 组之间的顺序与函数
    ///   定义顺序一致. 当预留空间的选项开启时, 基本块组之间会预留一定的空间.
    /// * **指令**: 保证同一个函数体中所有基本块的所有指令连续排布成组. 组内各基本块
    ///   的指令顺序与基本块内的指令顺序一致. 组之间的顺序与函数定义顺序一致.
    ///   当预留空间的选项开启时, 指令组之间会预留一定的空间.
    /// * **数据流边(`Use`)**: 保证同一个函数体中所有基本块的所有指令的所有数据流边
    ///   连续排布成组. 组内各基本块的指令的所有数据流边顺序与基本块内的指令的
    ///   所有数据流边顺序一致. 组之间的顺序与函数定义顺序一致. 当预留空间的选项
    ///   开启时, 数据流边组之间会预留一定的空间.
    /// * **跳转目标(`JumpTarget`)**: 保证同一个函数体中所有基本块的所有指令的所有
    ///   跳转目标连续排布成组. 组内各基本块的指令的所有跳转目标顺序与基本块内的
    ///   指令的所有跳转目标顺序一致. 组之间的顺序与函数定义顺序一致. 当预留空间的
    ///   选项开启时, 跳转目标组之间会预留一定的空间.
    /// * **表达式**: 由于表达式的标记是按照数据流图进行的, 表达式的排列可以视为无序
    ///   紧密排布. 预留空间的选项不会影响表达式的顺序.
    Compact(CompactItemTop),
}

#[derive(Debug, Clone)]
pub(super) struct CompactItemTop {
    pub(super) expr_top: usize,
    pub(super) global_top: usize,
    pub(super) inst_top: usize,
    pub(super) block_top: usize,
    pub(super) jt_top: usize,
    pub(super) use_top: usize,
}

impl<'a> MarkVisitor<'a> {
    pub fn from_module(module: &'a Module, should_compact: bool) -> Self {
        let live_set = IRRefLiveSet::from_module_empty(module);
        let mode = if should_compact {
            MarkMode::Compact(CompactItemTop {
                expr_top: 0,
                global_top: 0,
                inst_top: 0,
                block_top: 0,
                jt_top: 0,
                use_top: 0,
            })
        } else {
            MarkMode::NoCompact
        };

        Self {
            inner: RefCell::new(MarkVisitorInner { live_set, mode }),
            module,
        }
    }

    pub fn mark_value(&self, value: ValueSSA) -> Result<(), ModuleError> {
        let mode = self.inner.borrow().mode.clone();

        // discard non-reference value
        let old_pos = match value {
            ValueSSA::FuncArg(..) | ValueSSA::ConstData(..) | ValueSSA::None => {
                return Ok(());
            }
            ValueSSA::Block(b) => b.get_handle(),
            ValueSSA::Inst(i) => i.get_handle(),
            ValueSSA::ConstExpr(e) => e.get_handle(),
            ValueSSA::Global(g) => g.get_handle(),
        };

        // discard null reference
        if old_pos == usize::MAX {
            return Ok(());
        }

        let (new_pos, new_mode) = match mode {
            MarkMode::NoCompact => (old_pos, mode),
            MarkMode::Compact(mut c) => match value {
                ValueSSA::Block(_) => {
                    c.block_top += 1;
                    (c.block_top - 1, MarkMode::Compact(c))
                }
                ValueSSA::Inst(_) => {
                    c.inst_top += 1;
                    (c.inst_top - 1, MarkMode::Compact(c))
                }
                ValueSSA::ConstExpr(_) => {
                    c.expr_top += 1;
                    (c.expr_top - 1, MarkMode::Compact(c))
                }
                ValueSSA::Global(_) => {
                    c.global_top += 1;
                    (c.global_top - 1, MarkMode::Compact(c))
                }
                _ => unreachable!(),
            },
        };

        // update the live set
        let mut inner = self.inner.borrow_mut();
        inner.live_set.redirect_value(value, new_pos)?;
        inner.mode = new_mode;
        Ok(())
    }

    pub fn mark_use(&self, use_ref: UseRef) -> Result<(), ModuleError> {
        let mode = self.inner.borrow().mode.clone();

        // discard non-reference value
        let old_pos = use_ref.get_handle();
        // discard null reference
        if old_pos == usize::MAX {
            return Ok(());
        }

        let (new_pos, new_mode) = match mode {
            MarkMode::NoCompact => (old_pos, mode),
            MarkMode::Compact(mut c) => {
                c.use_top += 1;
                (c.use_top - 1, MarkMode::Compact(c))
            }
        };

        // update the live set
        let mut inner = self.inner.borrow_mut();
        inner.live_set.redirect_use(use_ref, new_pos)?;
        inner.mode = new_mode;
        Ok(())
    }
    pub fn mark_jt(&self, jt_ref: JumpTargetRef) -> Result<(), ModuleError> {
        let mode = self.inner.borrow().mode.clone();

        // discard non-reference value
        let old_pos = jt_ref.get_handle();
        // discard null reference
        if old_pos == usize::MAX {
            return Ok(());
        }

        let (new_pos, new_mode) = match mode {
            MarkMode::NoCompact => (old_pos, mode),
            MarkMode::Compact(mut c) => {
                c.jt_top += 1;
                (c.jt_top - 1, MarkMode::Compact(c))
            }
        };

        // update the live set
        let mut inner = self.inner.borrow_mut();
        inner.live_set.redirect_jt(jt_ref, new_pos)?;
        inner.mode = new_mode;
        Ok(())
    }

    pub fn value_is_live(&self, value: ValueSSA) -> bool {
        let inner = self.inner.borrow();
        inner.live_set.value_is_live(value).unwrap()
    }

    pub fn release_live_set(self) -> IRRefLiveSet {
        let inner = self.inner.into_inner();
        inner.live_set
    }
}

pub(super) struct MarkFuncTreeRes {
    live_operands: Vec<ValueSSA>,

    /// The number of live blocks. Used for reserveing space
    /// to compact the block list.
    _n_live_blocks: usize,

    /// The number of live instructions. Used for reserveing space
    /// to compact the instruction list.
    _n_live_insts: usize,

    /// The number of live jump targets. Used for reserveing space
    /// to compact the jump target list.
    _n_live_jts: usize,

    /// The number of live uses. Used for reserveing space
    /// to compact the use list.
    _n_live_uses: usize,
}

/// Mark all the values in the module
impl<'a> MarkVisitor<'a> {
    fn mark_operand(&self, op: ValueSSA) -> Result<(), ModuleError> {
        // discard non-reference value
        if !op.is_reference_semantics() {
            return Ok(());
        }
        // discard live reference.
        if self.value_is_live(op) {
            return Ok(());
        }

        match op {
            ValueSSA::None | ValueSSA::ConstData(_) | ValueSSA::FuncArg(..) => Ok(()),
            ValueSSA::Block(b) => self.mark_value(ValueSSA::Block(b)),
            ValueSSA::Inst(i) => self.mark_value(ValueSSA::Inst(i)),
            ValueSSA::Global(g) => self.mark_value(ValueSSA::Global(g)),
            ValueSSA::ConstExpr(e) => {
                self.mark_value(ValueSSA::ConstExpr(e))?;
                self.mark_expr_body(e)
            }
        }
    }

    fn mark_expr_body(&self, expr: ConstExprRef) -> Result<(), ModuleError> {
        let alloc_value = self.module.borrow_value_alloc();
        let alloc_expr = &alloc_value.alloc_expr;
        self._do_mark_expr_body(expr, alloc_expr)
    }
    fn _do_mark_expr_body(
        &self,
        expr: ConstExprRef,
        alloc: &Slab<ConstExprData>,
    ) -> Result<(), ModuleError> {
        if self.value_is_live(ValueSSA::ConstExpr(expr)) {
            return Ok(());
        }
        let expr_data = alloc.get(expr.get_handle()).unwrap();

        // **WARNING**: Only for this situation where the expr is whether a struct or an array.
        // If new ConstExprData is added, this code should be modified.
        let elems = match expr_data {
            ConstExprData::Array(a) => &a.elems,
            ConstExprData::Struct(s) => &s.elems,
        };

        for elem in elems {
            match elem {
                ValueSSA::None | ValueSSA::ConstData(_) | ValueSSA::FuncArg(..) => {}
                ValueSSA::Block(b) => self.mark_value(ValueSSA::Block(*b))?,
                ValueSSA::Inst(i) => self.mark_value(ValueSSA::Inst(*i))?,
                ValueSSA::Global(g) => self.mark_value(ValueSSA::Global(*g))?,
                ValueSSA::ConstExpr(e) => {
                    self.mark_value(ValueSSA::ConstExpr(*e))?;
                    self._do_mark_expr_body(*e, alloc)?;
                }
            }
        }
        Ok(())
    }

    /// Mark all the global objects.
    ///
    /// If encounter a function definition, push the function reference to a vector
    /// and return.
    ///
    /// ### Return
    ///
    /// * A vector of function references if `Ok`.
    /// * A `ModuleError` if failed.
    pub(super) fn mark_global(&self) -> Result<Vec<GlobalRef>, ModuleError> {
        let mut live_funcdef = Vec::new();
        let alloc_value = self.module.borrow_value_alloc();
        let alloc_global = &alloc_value.alloc_global;

        let global_map = self.module.global_defs.borrow();
        for (_, global_ref) in global_map.iter() {
            let global = global_ref.to_slabref_unwrap(alloc_global);
            match global {
                GlobalData::Func(f) => {
                    if !f.is_extern() {
                        live_funcdef.push(*global_ref);
                    }
                }
                GlobalData::Var(v) => match v.get_init() {
                    Some(init) => self.mark_value(init)?,
                    None => {}
                },
                _ => {}
            }
            // mark the global reference
            self.mark_value(ValueSSA::Global(*global_ref))?;
        }
        drop(global_map);

        Ok(live_funcdef)
    }

    pub(super) fn mark_func_tree(&self, func: GlobalRef) -> Result<MarkFuncTreeRes, ModuleError> {
        let alloc_value = self.module.borrow_value_alloc();
        let alloc_global = &alloc_value.alloc_global;
        let alloc_block = &alloc_value.alloc_block;
        let alloc_inst = &alloc_value.alloc_inst;

        let alloc_use = self.module.borrow_use_alloc();
        let alloc_jt = self.module.borrow_jt_alloc();
        let func = match func.to_slabref_unwrap(alloc_global) {
            GlobalData::Func(f) => f,
            _ => panic!("MarkVisitor::mark_func: not a function"),
        };

        // mark the function body
        let body = match func.get_blocks() {
            Some(body) => body,
            None => panic!("MarkVisitor::mark_func: no function body"),
        };

        // mark the blocks
        let mut n_inst_nodes = 0;
        let mut live_blocks = Vec::with_capacity(body.len());
        let mut block_node = body._head;
        while block_node.is_nonnull() {
            let block = block_node.to_slabref_unwrap(alloc_block);
            self.mark_value(ValueSSA::Block(block_node))?;
            let (inst_range, node_cnt) = block.instructions.load_range_and_full_node_count();
            n_inst_nodes += node_cnt;
            live_blocks.push((block_node, inst_range));
            block_node = match block_node.get_next_ref(alloc_block) {
                Some(next) => next,
                None => break,
            };
        }

        // mark the instructions per block
        let mut live_uses = Vec::with_capacity(n_inst_nodes);
        let mut live_jts = Vec::with_capacity(live_blocks.len());
        let mut n_use_nodes = 0;
        for (_, inst_view) in &live_blocks {
            let mut inst_node = inst_view.node_head;
            while inst_node.is_nonnull() {
                // mark the instruction reference
                self.mark_value(ValueSSA::Inst(inst_node))?;

                // mark the instruction operands
                let inst_data = inst_node.to_slabref_unwrap(alloc_inst);
                if let Some(c) = inst_data.get_common() {
                    let (use_range, n_nodes) =
                        c.operands.load_range_and_full_node_count();
                    if n_nodes > 0 {
                        n_use_nodes += n_nodes;
                        live_uses.push(use_range);
                    }
                }
                inst_node = match inst_node.get_next_ref(alloc_inst) {
                    Some(next) => next,
                    None => break,
                };

                let jts = match inst_data {
                    InstData::Jump(_, jmp) => jmp.get_jump_targets().unwrap(),
                    InstData::Br(_, br) => br.get_jump_targets().unwrap(),
                    InstData::Switch(_, s) => s.get_jump_targets().unwrap(),
                    _ => continue,
                };
                live_jts.push(jts.load_range());
            }
        }

        // Mark all live uses
        let mut live_operands = Vec::with_capacity(n_use_nodes);
        for useref in &live_uses {
            let mut use_node = useref.node_head;
            while use_node.is_nonnull() {
                // mark the use reference
                self.mark_use(use_node)?;
                let operand = use_node.to_slabref_unwrap(&alloc_use).get_operand();

                match operand {
                    ValueSSA::None | ValueSSA::FuncArg(..) | ValueSSA::ConstData(..) => {}
                    ValueSSA::Block(_) | ValueSSA::Inst(_) => { /* marked */ }
                    ValueSSA::ConstExpr(_) | ValueSSA::Global(_) => live_operands.push(operand),
                }

                use_node = match use_node.get_next_ref(&alloc_use) {
                    Some(next) => next,
                    None => break,
                };
            }
        }

        // Mark all live jump targets
        for jt_ref in &live_jts {
            let mut jt_node = jt_ref.node_head;
            while jt_node.is_nonnull() {
                // mark the jump target reference
                self.mark_jt(jt_node)?;
                jt_node = match jt_node.get_next_ref(&alloc_jt) {
                    Some(next) => next,
                    None => break,
                };
            }
        }

        Ok(MarkFuncTreeRes {
            live_operands,
            _n_live_blocks: live_blocks.len(),
            _n_live_insts: n_inst_nodes,
            _n_live_jts: live_jts.len(),
            _n_live_uses: n_use_nodes,
        })
    }

    pub(super) fn mark_module(&self) -> Result<(), ModuleError> {
        // mark all the global objects
        let live_funcdef = self.mark_global()?;

        // mark all the function bodies
        for func in live_funcdef {
            let res = self.mark_func_tree(func)?;
            let operands = &res.live_operands;

            // mark all the operands
            for op in operands {
                self.mark_operand(*op)?;
            }
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub(super) fn mark_module_reserve(
        &self,
        calc_reserve: impl Fn(&FuncData, &mut MarkFuncTreeRes),
    ) -> Result<(), ModuleError> {
        // mark all the global objects
        let live_funcdef = self.mark_global()?;

        // mark all the function bodies
        for func in live_funcdef {
            let mut res = self.mark_func_tree(func)?;
            let func_data = self.module.get_global(func);
            let func_data = match &*func_data {
                GlobalData::Func(f) => f,
                _ => panic!("MarkVisitor::mark_func: not a function"),
            };
            for op in &res.live_operands {
                self.mark_operand(*op)?;
            }
            calc_reserve(func_data, &mut res);
            if let MarkMode::Compact(ref mut c) = self.inner.borrow_mut().mode {
                c.expr_top += res._n_live_insts;
                c.global_top += res._n_live_blocks;
                c.inst_top += res._n_live_insts;
                c.block_top += res._n_live_blocks;
                c.jt_top += res._n_live_jts;
                c.use_top += res._n_live_uses;
            }
        }
        Ok(())
    }

    #[allow(dead_code)]
    pub(super) fn mark_module_reserve_double(&self) -> Result<(), ModuleError> {
        // Keep the live value count unchanged so that the
        // reserve space is twice the size of the live value.
        self.mark_module_reserve(|_, _| {})
    }
}

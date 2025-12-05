use crate::{
    ir::{
        AggrZero, ExprObj, IArrayExpr, IRAllocs, IRWriter, ISubExprID, ISubValueSSA,
        ITraceableValue, IUser, KVArrayExpr, KVArrayExprID, Module, SplatArrayExprID, StructExpr,
        StructExprID, UseKind, ValueSSA, module::allocs::IPoolAllocated,
    },
    typing::{AggrType, ArrayTypeID, IValType, StructTypeID, TypeContext, ValTypeID},
};
use smallvec::SmallVec;
use std::{ops::Range, path::Path};

pub fn dump_llvm_adapted<P: AsRef<Path>>(module: &Module, filepath: P) -> std::io::Result<()> {
    LLVMAdapt::new(module).run();
    let mut out = std::fs::File::create(filepath)?;
    IRWriter::from_module(&mut out, module).write_module();
    Ok(())
}

pub struct LLVMAdapt<'ir> {
    module: &'ir Module,
}

impl<'ir> LLVMAdapt<'ir> {
    pub fn new(module: &'ir Module) -> Self {
        Self { module }
    }

    pub fn run(&self) {
        let alloc_expr = &self.module.allocs.exprs;
        let mut kvarrays = Vec::new();
        for (_, expr_ptr, expr) in alloc_expr {
            if expr.obj_disposed() {
                continue;
            }
            match expr {
                ExprObj::KVArray(_) => kvarrays.push(KVArrayExprID(expr_ptr)),
                _ => continue,
            }
        }
        for kvid in kvarrays {
            // 不需要递归处理它的操作数。
            // - 遍历分配器的操作就已经把所有的 KVArrayExpr 包括在内了, 处理 KVArrayExpr
            //   时也不会产生新的 KVArrayExpr, 因此没有漏检. (Remusys-IR 不允许跨分配器的引用)
            // - KVArrayExpr 替换成其他表达式的操作是通过 def-use 链的反向传播进行的, 不需要
            //   通过遍历去主动替换.
            self.adapt_kvarray(kvid);
        }
    }

    /// LLVM 识别不了 KVArrayExpr, 所以需要转换成 LLVM 可以识别的模式. 具体来说:
    ///
    /// - 如果满足 splat 的条件就转换成 SplatArray. 虽然 SplatArray 这个类型本身
    ///   不是 LLVM 兼容的, 但它的文本格式和 ArrayExpr 完全一致, 是 LLVM 兼容的.
    /// - 如果是 C 语言那种 `{a, b, c, 0, ...}` 的模式, 就按照 LLVM 的习惯翻译成
    ///   `<{a, b, c, [aaa x bbb] zeroinitalizer}>` 的紧凑结构体模式
    /// - 哪儿都去不了的情况下就翻译成一般的数组表达式了.
    fn adapt_kvarray(&self, kvid: KVArrayExprID) {
        let allocs = self.allocs();
        // 先把全 0 的情况做了
        if let Some(z) = AggrZero::try_from_expr(kvid, allocs) {
            kvid.deref_ir(allocs)
                .replace_self_with(allocs, z.into_ir())
                .expect("Internal error");
            return;
        }
        let kvarray = kvid.deref_ir(allocs);

        // 总是返回一个 0..RIGHT 这样的范围.
        let nondefault_range = kvarray.nondefault_index_range(allocs);
        let default_val = kvarray.get_default(allocs);
        if nondefault_range.is_empty() {
            self.replace_kvarray_with_splat(kvarray, default_val);
            return;
        }
        if kvarray
            .operands_iter()
            .all(|u| u.get_operand(allocs) == default_val)
        {
            self.replace_kvarray_with_splat(kvarray, default_val);
            return;
        }

        // 在这个范围之外的元素都是 default_val —— 统计一下有多少个, 分配到紧凑结构体的后半部分
        // (当然, 如果数组不是 front_dense 的, 范围之内也有一些元素是 default_val)
        let right_default_len = kvarray.nelems - nondefault_range.end;
        if default_val.is_zero_const(allocs) && right_default_len > 4 {
            self.replace_kvarray_with_packed_struct(kvarray, nondefault_range, default_val);
            return;
        }

        // fallback: 直接展开
        let arrexp = kvarray.expand_to_array_id(allocs);
        kvarray
            .replace_self_with(allocs, arrexp.raw_into().into_ir())
            .unwrap();
    }

    fn replace_kvarray_with_splat(&self, kvarray: &KVArrayExpr, val: ValueSSA) {
        let allocs = self.allocs();
        let arrty = kvarray.arrty;
        let splat = SplatArrayExprID::new(allocs, self.tctx(), arrty, val);
        kvarray
            .replace_self_with(allocs, splat.raw_into().into_ir())
            .expect("Internal error");
    }
    fn replace_kvarray_with_packed_struct(
        &self,
        kvarray: &KVArrayExpr,
        nondefault_range: Range<usize>,
        default_val: ValueSSA,
    ) {
        assert_eq!(
            nondefault_range.start, 0,
            "Internal error: non-default range should begin with the leftmost index"
        );
        let allocs = self.allocs();
        let tctx = self.tctx();
        let mut field_types: SmallVec<[ValTypeID; 8]> =
            SmallVec::with_capacity(nondefault_range.end + 1);
        let elemty = kvarray.arrty.get_element_type(tctx);
        for _ in nondefault_range.clone() {
            field_types.push(elemty);
        }
        let right_default_len = kvarray.nelems - nondefault_range.end;
        let zarrty = ArrayTypeID::new(tctx, elemty, right_default_len);
        field_types.push(zarrty.into_ir());
        let emulated_structy = StructTypeID::new(tctx, true, field_types);
        let struc = StructExpr::new_uninit(allocs, tctx, emulated_structy);
        let mut virt_index = 0;

        // 这里每个 useid 对应的 elem_idx 都是单调递增的, 反应这个 use 在展开后数组的实际位置.
        for &useid in kvarray.elem_uses() {
            let UseKind::KVArrayElem(elem_idx) = useid.get_kind(allocs) else {
                panic!("Internal error: KVArray:{kvarray:p} has wrong use kind in elem region")
            };
            while virt_index < elem_idx {
                struc.fields[virt_index].set_operand(allocs, default_val);
                virt_index += 1;
            }
            struc.fields[elem_idx].set_operand(allocs, useid.get_operand(allocs));
        }

        // 然后把最后一个补上
        let zarray = ValueSSA::AggrZero(AggrType::Array(zarrty));
        struc.fields.last().unwrap().set_operand(allocs, zarray);
        let struc = StructExprID::allocate(allocs, struc).raw_into();
        kvarray.replace_self_with(allocs, struc.into_ir()).unwrap();
    }

    fn allocs(&self) -> &IRAllocs {
        &self.module.allocs
    }
    fn tctx(&self) -> &TypeContext {
        &self.module.tctx
    }
}

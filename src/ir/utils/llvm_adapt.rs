use crate::{
    ir::{
        AggrZero, IRAllocs, ISubExprID, ISubValueSSA, IUser, KVArrayExpr, KVArrayExprID, Module,
        SplatArrayExprID, StructExprID, UseKind, ValueSSA,
    },
    typing::{AggrType, ArrayTypeID, StructTypeID, ValTypeID},
};
use smallvec::SmallVec;
use std::collections::HashMap;

#[derive(Default)]
pub struct LLVMAdaptMapping {
    pub kvarr: HashMap<KVArrayExprID, ValueSSA>,
}

impl LLVMAdaptMapping {
    /// LLVM 识别不了 KVArrayExpr, 所以需要转换成 LLVM 可以识别的模式. 具体来说:
    ///
    /// - 如果满足 splat 的条件就转换成 SplatArray. 虽然 SplatArray 这个类型本身
    ///   不是 LLVM 兼容的, 但它的文本格式和 ArrayExpr 完全一致, 是 LLVM 兼容的.
    /// - 如果是 C 语言那种 `{a, b, c, 0, ...}` 的模式, 就按照 LLVM 的习惯翻译成
    ///   `<{a, b, c, [aaa x bbb] zeroinitalizer}>` 的紧凑结构体模式
    /// - 哪儿都去不了的情况下就翻译成一般的数组表达式了.
    pub fn map_kvarr(&mut self, module: &Module, from: KVArrayExprID) -> ValueSSA {
        if let Some(val) = self.kvarr.get(&from) {
            *val
        } else {
            let val = self.adapt_kvarr(module, from);
            self.kvarr.insert(from, val);
            val
        }
    }
    pub fn map_value(&mut self, module: &Module, val: ValueSSA) -> ValueSSA {
        let ValueSSA::ConstExpr(expr) = val else {
            return val;
        };
        let Some(kvid) = KVArrayExprID::try_from_expr(expr, &module.allocs) else {
            return val;
        };
        self.map_kvarr(module, kvid)
    }
    fn adapt_kvarr(&mut self, module: &Module, from: KVArrayExprID) -> ValueSSA {
        let (allocs, tctx) = (&module.allocs, &module.tctx);
        if let Some(z) = AggrZero::try_from_expr(from, allocs) {
            return z.into_ir();
        }
        let kvarray = from.deref_ir(allocs);
        let arrty = kvarray.arrty;
        if let Some(splat_val) = Self::kv_as_splat(kvarray, allocs) {
            let splat_arr = SplatArrayExprID::new(allocs, tctx, arrty, splat_val);
            return splat_arr.raw_into().into_ir();
        }

        let default_val = self.map_value(module, kvarray.get_default(allocs));
        let nondef_range = kvarray.nondefault_index_range(allocs);
        assert_eq!(
            nondef_range.start, 0,
            "KVArrayAdapt currently only supports trailing default values"
        );
        let struc = self.kv_as_packed_struct(kvarray, default_val, module);
        struc.raw_into().into_ir()
    }
    fn kv_as_splat(kv: &KVArrayExpr, allocs: &IRAllocs) -> Option<ValueSSA> {
        let default_val = kv.get_default(allocs);
        let mut elem_iter = kv.operands_iter().map(|u| u.get_operand(allocs));
        let Some(first) = elem_iter.next() else { return Some(default_val) };
        if !elem_iter.all(|v| v == first) {
            return None;
        }
        if first == default_val || kv.elem_uses().len() == kv.nelems { Some(first) } else { None }
    }
    fn kv_as_packed_struct(
        &mut self,
        kv: &KVArrayExpr,
        default_val: ValueSSA,
        module: &Module,
    ) -> StructExprID {
        let (allocs, tctx) = (&module.allocs, &module.tctx);
        let default_val_ty = default_val.get_valtype(allocs);
        let elemty = kv.arrty.get_element_type(tctx);

        let nondefault_range = kv.nondefault_index_range(allocs);
        assert_eq!(nondefault_range.start, 0);
        let nondefault_len = nondefault_range.end;

        let mut fields: SmallVec<[ValueSSA; 8]> = SmallVec::with_capacity(nondefault_len + 1);
        let mut field_tys: SmallVec<[ValTypeID; 8]> = SmallVec::with_capacity(nondefault_len + 1);

        let mut field_idx = 0;
        for &use_id in kv.elem_uses() {
            let UseKind::KVArrayElem(elem_idx) = use_id.get_kind(allocs) else {
                panic!("Internal error: Expected KVArrayElem use kind");
            };
            while field_idx < elem_idx {
                fields.push(default_val);
                field_tys.push(default_val_ty);
                field_idx += 1;
            }
            let val = self.map_value(module, use_id.get_operand(allocs));
            fields.push(val);
            // NOTE that val.valtype may differ from elemty due to LLVMAdaptMapping
            field_tys.push(val.get_valtype(allocs));
            field_idx += 1;
        }

        let mut back_len = kv.nelems - nondefault_range.end;
        loop {
            match fields.last() {
                Some(last_field) if last_field == &default_val => {
                    fields.pop();
                    field_tys.pop();
                    back_len += 1;
                }
                _ => break,
            }
        }

        if back_len > 0 {
            let back_field = if default_val.is_zero_const(allocs) {
                let back_arrty = ArrayTypeID::new(tctx, elemty, back_len);
                ValueSSA::AggrZero(AggrType::Array(back_arrty))
            } else if back_len == 1 {
                default_val
            } else {
                let back_arrty = ArrayTypeID::new(tctx, default_val_ty, back_len);
                let splat_arr = SplatArrayExprID::new(allocs, tctx, back_arrty, default_val);
                splat_arr.raw_into().into_ir()
            };
            fields.push(back_field);
            field_tys.push(back_field.get_valtype(allocs));
        }
        let structty = StructTypeID::new(tctx, true, field_tys);
        let struc = StructExprID::new_uninit(allocs, tctx, structty);
        for (idx, val) in fields.into_iter().enumerate() {
            struc.set_field(allocs, idx, val);
        }
        struc
    }
}

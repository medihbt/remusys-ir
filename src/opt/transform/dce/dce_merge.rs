use std::{
    collections::{HashMap, HashSet},
    hash::{DefaultHasher, Hash, Hasher},
};

use crate::{
    base::SlabRef,
    ir::{
        Array, ConstExprData, ExprRef, GlobalData, IRAllocs, ISubValueSSA, IUser, Module, Struct,
        Use, ValueSSA,
    },
    typing::{AggrType, AggrTypeIter, IValType, ScalarType, TypeContext},
};

struct ValueDataHasher<'a> {
    allocs: &'a IRAllocs,
    type_ctx: &'a TypeContext,
    key_map: HashMap<ValueSSA, u64>,
}

impl<'a> ValueDataHasher<'a> {
    fn new(allocs: &'a IRAllocs, type_ctx: &'a TypeContext) -> Self {
        Self { allocs, type_ctx, key_map: HashMap::new() }
    }

    fn get_hash(&mut self, value: ValueSSA) -> u64 {
        if let Some(&hash) = self.key_map.get(&value) {
            return hash;
        }
        let hash = self.make_hash(value);
        self.key_map.insert(value, hash);
        hash
    }

    fn make_hash(&mut self, value: ValueSSA) -> u64 {
        let mut hasher = DefaultHasher::new();
        match value {
            ValueSSA::ConstExpr(expr) => match expr.to_data(&self.allocs.exprs) {
                ConstExprData::Array(arr) => {
                    let arrty = AggrType::Array(arr.arrty);
                    self.hash_aggr(&mut hasher, arrty, arr.operands_iter());
                }
                ConstExprData::Struct(s) => {
                    let strty = AggrType::from_ir(s.structty);
                    self.hash_aggr(&mut hasher, strty, s.operands_iter());
                }
                ConstExprData::FixVec(v) => {
                    let vecty = AggrType::FixVec(v.vecty);
                    self.hash_aggr(&mut hasher, vecty, v.operands_iter());
                }
            },
            ValueSSA::AggrZero(zeroty) => {
                let aggr_iter = AggrTypeIter::new(zeroty, self.type_ctx);
                self.hash_aggr(
                    &mut hasher,
                    zeroty,
                    aggr_iter.map(|(_, ty)| ValueSSA::new_zero(ty)),
                )
            }
            _ => value.hash(&mut hasher),
        }
        hasher.finish()
    }

    fn hash_aggr(
        &mut self,
        hasher: &mut DefaultHasher,
        aggr_ty: AggrType,
        elems: impl Iterator<Item = ValueSSA>,
    ) {
        aggr_ty.hash(hasher);
        for elem in elems {
            let elem_hash = self.get_hash(elem);
            hasher.write_u64(elem_hash);
        }
    }

    fn value_equal(&mut self, lhs: ValueSSA, rhs: ValueSSA) -> bool {
        if lhs == rhs {
            return true;
        }
        if self.get_hash(lhs) != self.get_hash(rhs) {
            return false;
        }

        use ValueSSA::*;
        match (lhs, rhs) {
            (ConstExpr(l), ConstExpr(r)) => self.expr_equal(l, r),
            (ConstExpr(exp), AggrZero(ztype)) | (AggrZero(ztype), ConstExpr(exp)) => {
                exp.get_valtype(self.allocs) == ztype.into_ir() && exp.is_zero(self.allocs)
            }
            _ => false, // 其他相等分支都在上面的 lhs == rhs 里覆盖了, 这里返回 false
        }
    }
    fn use_equal(&mut self, l: &Use, r: &Use) -> bool {
        if std::ptr::eq(l, r) { true } else { self.value_equal(l.get_operand(), r.get_operand()) }
    }

    fn expr_equal(&mut self, l: ExprRef, r: ExprRef) -> bool {
        let alloc_expr = &self.allocs.exprs;
        let ldata = l.to_data(alloc_expr);
        let rdata = r.to_data(alloc_expr);

        match (ldata, rdata) {
            (ConstExprData::Array(ldata), ConstExprData::Array(rdata)) => {
                self.arr_equal(ldata, rdata)
            }
            (ConstExprData::Struct(ldata), ConstExprData::Struct(rdata)) => {
                self.struct_eq(ldata, rdata)
            }
            _ => false,
        }
    }

    fn arr_equal(&mut self, l: &Array, r: &Array) -> bool {
        if l.arrty != r.arrty || l.elems.len() != r.elems.len() {
            return false;
        }
        l.elems
            .iter()
            .zip(&r.elems)
            .all(|(l_elem, r_elem)| self.use_equal(l_elem, r_elem))
    }

    fn struct_eq(&mut self, l: &Struct, r: &Struct) -> bool {
        if l.structty != r.structty || l.elems.len() != r.elems.len() {
            return false;
        }
        l.elems
            .iter()
            .zip(&r.elems)
            .all(|(l_elem, r_elem)| self.use_equal(l_elem, r_elem))
    }
}

struct ExprEntry {
    candidate: ValueSSA,
    slots: HashSet<ValueSSA>,
}

impl ExprEntry {
    fn new(candidate: ValueSSA) -> Self {
        Self { candidate, slots: HashSet::new() }
    }

    fn add(&mut self, value: ValueSSA) {
        self.slots.insert(value);
    }
}

struct ExprMergeMap<'a> {
    data: HashMap<u64, Vec<ExprEntry>>,
    hasher: ValueDataHasher<'a>,
}

impl<'a> ExprMergeMap<'a> {
    fn new(type_ctx: &'a TypeContext, allocs: &'a IRAllocs) -> Self {
        Self {
            data: HashMap::new(),
            hasher: ValueDataHasher::new(allocs, type_ctx),
        }
    }

    fn insert_for_candidate(&mut self, value: ValueSSA) -> ValueSSA {
        match value {
            ValueSSA::ConstExpr(_) | ValueSSA::AggrZero(_) => {}
            // ConstData 和 None 是值语义的不用索引, 其他的有引用唯一性不用 Hash
            _ => return value,
        }
        let hash = self.hasher.get_hash(value);
        let entries = self.data.entry(hash).or_default();
        if entries.is_empty() {
            entries.push(ExprEntry::new(value));
            return value;
        }

        let entry = entries
            .iter_mut()
            .find(|e| self.hasher.value_equal(e.candidate, value));
        match entry {
            None => {
                entries.push(ExprEntry::new(value));
                value
            }
            Some(entry) => {
                entry.add(value);
                entry.candidate
            }
        }
    }

    fn dump_map(&self) -> HashMap<ValueSSA, ValueSSA> {
        let mut result = HashMap::new();
        for entry_list in self.data.values() {
            for entry in entry_list {
                for &slot in &entry.slots {
                    result.insert(slot, entry.candidate);
                }
            }
        }
        result
    }
}

pub(super) fn merge_exprs(module: &mut Module) {
    let allocs = &module.allocs;
    let value_map = {
        let mut merge_map = ExprMergeMap::new(&module.type_ctx, &allocs);
        for (expr_ref, _) in allocs.exprs.iter() {
            let expr_ref = ExprRef::from_handle(expr_ref);
            merge_map.insert_for_candidate(expr_ref.into_ir());
        }
        merge_map.dump_map()
    };
    for (_, gdata) in &allocs.globals {
        match gdata {
            GlobalData::Func(_) => continue,
            GlobalData::Var(var) => {
                let old_init = var.get_init();
                let new_init = value_map.get(&old_init).cloned().unwrap_or(old_init);
                var.set_init(&allocs, new_init);
            }
        }
    }
    for (_, exprs) in &allocs.exprs {
        let elems = match exprs {
            ConstExprData::Array(arr) => {
                if let Ok(_) = ScalarType::try_from_ir(arr.arrty.get_element_type(&module.type_ctx))
                {
                    continue;
                }
                &arr.elems
            }
            ConstExprData::Struct(s) => &s.elems,
            ConstExprData::FixVec(_) => continue,
        };
        for old_elem in elems {
            let oldval = old_elem.get_operand();
            let newval = value_map.get(&oldval).cloned().unwrap_or(oldval);
            old_elem.set_operand(&allocs, newval);
        }
    }

    for (_, insts) in &allocs.insts {
        for useref in &insts.get_operands() {
            let old_op = useref.get_operand();
            let new_op = value_map.get(&old_op).cloned().unwrap_or(old_op);
            useref.set_operand(&allocs, new_op);
        }
    }
    module.gc_mark_sweep([]);
}

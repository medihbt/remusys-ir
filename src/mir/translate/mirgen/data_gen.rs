use slab::Slab;

use crate::{
    base::slabref::SlabRef,
    ir::{
        ValueSSA,
        constant::{
            data::ConstData,
            expr::{ConstExprData, ConstExprRef},
        },
        module::Module,
    },
    mir::module::global::{MirGlobalData, Section},
    typing::{context::TypeContext, types::FloatTypeKind},
};

pub(super) fn translate_const_init(
    initval: ValueSSA,
    section: Section,
    ir_module: &Module,
) -> Vec<MirGlobalData> {
    let type_ctx = &ir_module.type_ctx;
    let alloc_value = ir_module.borrow_value_alloc();
    let alloc_expr = &alloc_value.alloc_expr;
    let mut builder = GlobalDataBuilder::new();
    builder.push_value(initval, alloc_expr, type_ctx);
    builder.collect_data(section)
}

pub(super) enum GlobalDataUnit {
    Bytes(Vec<u8>),
    Halfs(Vec<u16>),
    Words(Vec<u32>),
    Dwords(Vec<u64>),
    Byte(u8),
    Half(u16),
    Long(u32),
    Quad(u64),
}

impl GlobalDataUnit {
    fn box_single(&mut self) {
        match self {
            GlobalDataUnit::Bytes(_)
            | GlobalDataUnit::Halfs(_)
            | GlobalDataUnit::Words(_)
            | GlobalDataUnit::Dwords(_) => {}
            GlobalDataUnit::Byte(b) => *self = GlobalDataUnit::Bytes(vec![*b]),
            GlobalDataUnit::Half(s) => *self = GlobalDataUnit::Halfs(vec![*s]),
            GlobalDataUnit::Long(l) => *self = GlobalDataUnit::Words(vec![*l]),
            GlobalDataUnit::Quad(q) => *self = GlobalDataUnit::Dwords(vec![*q]),
        }
    }

    fn with_zeroes(unit_bytes_log2: u8, count: usize) -> Self {
        match unit_bytes_log2 {
            0 => GlobalDataUnit::Bytes(vec![0; count]),
            1 => GlobalDataUnit::Halfs(vec![0; count]),
            2 => GlobalDataUnit::Words(vec![0; count]),
            3 => GlobalDataUnit::Dwords(vec![0; count]),
            _ => panic!("Unsupported unit size"),
        }
    }

    pub(super) fn from_const_data(data: ConstData, type_ctx: &TypeContext) -> Self {
        match data {
            ConstData::Undef(ty) | ConstData::Zero(ty) => {
                let size_bytes = ty
                    .get_instance_size(type_ctx)
                    .expect("Type must have a defined size");
                let align = ty
                    .get_instance_align(type_ctx)
                    .expect("Type must have a defined alignment");
                let unit_bytes_log2 = if align.is_power_of_two() {
                    align.trailing_zeros() as u8
                } else {
                    panic!("Alignment {} is not a power of two", align);
                };
                GlobalDataUnit::with_zeroes(unit_bytes_log2, size_bytes >> unit_bytes_log2)
            }
            ConstData::PtrNull(_) => GlobalDataUnit::Quad(0),
            ConstData::Int(bits, value) => {
                let type_kind = bits as u8;
                match type_kind {
                    8 => GlobalDataUnit::Byte(value as u8),
                    16 => GlobalDataUnit::Half(value as u16),
                    32 => GlobalDataUnit::Long(value as u32),
                    64 => GlobalDataUnit::Quad(value as u64),
                    _ => panic!("Unsupported integer size: {}", type_kind),
                }
            }
            ConstData::Float(fp_kind, value) => match fp_kind {
                FloatTypeKind::Ieee32 => GlobalDataUnit::Long((value as f32).to_bits()),
                FloatTypeKind::Ieee64 => GlobalDataUnit::Quad((value as f64).to_bits()),
            },
        }
    }
}

struct GlobalDataBuilder {
    buffer: Vec<GlobalDataUnit>,
}

impl GlobalDataBuilder {
    fn new() -> Self {
        Self { buffer: Vec::new() }
    }

    fn push_unit(&mut self, mut unit: GlobalDataUnit) {
        type U = GlobalDataUnit;
        if self.buffer.is_empty() {
            unit.box_single();
            self.buffer.push(unit);
            return;
        }
        let last = self.buffer.last_mut().unwrap();
        match (last, &mut unit) {
            (U::Bytes(buf), U::Bytes(data)) => buf.extend(data.as_slice()),
            (U::Bytes(buf), U::Byte(b)) => buf.push(*b),
            (U::Halfs(buf), U::Halfs(data)) => buf.extend(data.as_slice()),
            (U::Halfs(buf), U::Half(s)) => buf.push(*s),
            (U::Words(buf), U::Words(data)) => buf.extend(data.as_slice()),
            (U::Words(buf), U::Long(l)) => buf.push(*l),
            (U::Dwords(buf), U::Dwords(data)) => buf.extend(data.as_slice()),
            (U::Dwords(buf), U::Quad(q)) => buf.push(*q),
            _ => {
                unit.box_single();
                self.buffer.push(unit);
            }
        }
    }

    fn push_const_data(&mut self, data: ConstData, type_ctx: &TypeContext) {
        self.push_unit(GlobalDataUnit::from_const_data(data, type_ctx));
    }
    fn push_const_expr(
        &mut self,
        expr: ConstExprRef,
        alloc_expr: &Slab<ConstExprData>,
        type_ctx: &TypeContext,
    ) {
        let expr_data = expr.to_slabref_unwrap(alloc_expr);
        match expr_data {
            ConstExprData::Array(a) => self.push_aggregate(&a.elems, alloc_expr, type_ctx),
            ConstExprData::Struct(s) => self.push_aggregate(&s.elems, alloc_expr, type_ctx),
        }
    }
    fn push_aggregate(
        &mut self,
        values: &[ValueSSA],
        alloc_expr: &Slab<ConstExprData>,
        type_ctx: &TypeContext,
    ) {
        for value in values {
            self.push_value(*value, alloc_expr, type_ctx);
        }
    }
    fn push_value(
        &mut self,
        value: ValueSSA,
        alloc_expr: &Slab<ConstExprData>,
        type_ctx: &TypeContext,
    ) {
        match value {
            ValueSSA::ConstData(data) => self.push_const_data(data, type_ctx),
            ValueSSA::ConstExpr(expr) => self.push_const_expr(expr, alloc_expr, type_ctx),
            ValueSSA::Block(_)
            | ValueSSA::FuncArg(..)
            | ValueSSA::Inst(_)
            | ValueSSA::Global(_)
            | ValueSSA::None => {
                panic!("Unexpected value type in global data builder: {:?}", value);
            }
        }
    }

    fn collect_data(self, section: Section) -> Vec<MirGlobalData> {
        let mut data = Vec::with_capacity(self.buffer.len());
        for unit in self.buffer {
            let gdata = match unit {
                GlobalDataUnit::Bytes(items) => MirGlobalData::new_bytes(section, &items),
                GlobalDataUnit::Halfs(items) => MirGlobalData::new_half(section, &items),
                GlobalDataUnit::Words(items) => MirGlobalData::new_word(section, &items),
                GlobalDataUnit::Dwords(items) => MirGlobalData::new_dword(section, &items),
                GlobalDataUnit::Byte(b) => MirGlobalData::new_bytes(section, &[b]),
                GlobalDataUnit::Half(h) => MirGlobalData::new_half(section, &[h]),
                GlobalDataUnit::Long(l) => MirGlobalData::new_word(section, &[l]),
                GlobalDataUnit::Quad(q) => MirGlobalData::new_dword(section, &[q]),
            };
            data.push(gdata);
        }
        data
    }
}

use slab::Slab;

use crate::{
    base::SlabRef,
    ir::{ConstData, ConstExprData, ConstExprRef, Module, ValueSSA},
    mir::module::global::{MirGlobalData, Section},
    typing::{FPKind, IValType, TypeContext, ValTypeID},
};

#[derive(Debug, Clone)]
pub enum DataUnit {
    Byte(u8),
    Half(u16),
    Word(u32),
    DWord(u64),
    Bytes(Vec<u8>),
    Halfs(Vec<u16>),
    Words(Vec<u32>),
    DWords(Vec<u64>),
}

impl DataUnit {
    pub fn size(&self) -> usize {
        match self {
            Self::Byte(_) => 1,
            Self::Half(_) => 2,
            Self::Word(_) => 4,
            Self::DWord(_) => 8,
            Self::Bytes(data) => data.len(),
            Self::Halfs(data) => data.len() * 2,
            Self::Words(data) => data.len() * 4,
            Self::DWords(data) => data.len() * 8,
        }
    }
    pub fn unit_size_log2(&self) -> u8 {
        match self {
            Self::Byte(_) | Self::Bytes(_) => 0,
            Self::Half(_) | Self::Halfs(_) => 1,
            Self::Word(_) | Self::Words(_) => 2,
            Self::DWord(_) | Self::DWords(_) => 3,
        }
    }
    pub fn unit_size(&self) -> usize {
        1 << self.unit_size_log2()
    }

    pub fn into_boxed(self) -> Self {
        match self {
            Self::Byte(b) => Self::Bytes(vec![b]),
            Self::Half(h) => Self::Halfs(vec![h]),
            Self::Word(w) => Self::Words(vec![w]),
            Self::DWord(d) => Self::DWords(vec![d]),
            Self::Bytes(data) => Self::Bytes(data),
            Self::Halfs(data) => Self::Halfs(data),
            Self::Words(data) => Self::Words(data),
            Self::DWords(data) => Self::DWords(data),
        }
    }
    pub fn box_self(&mut self) {
        match self {
            Self::Byte(b) => *self = Self::Bytes(vec![*b]),
            Self::Half(h) => *self = Self::Halfs(vec![*h]),
            Self::Word(w) => *self = Self::Words(vec![*w]),
            Self::DWord(d) => *self = Self::DWords(vec![*d]),
            Self::Bytes(_) | Self::Halfs(_) | Self::Words(_) | Self::DWords(_) => {}
        }
    }

    pub fn from_zeroes(unit_size_log2: u8, count: usize) -> Self {
        match (unit_size_log2, count) {
            (0, 1) => Self::Byte(0),
            (1, 1) => Self::Half(0),
            (2, 1) => Self::Word(0),
            (3, 1) => Self::DWord(0),
            (0, _) => Self::Bytes(vec![0; count]),
            (1, _) => Self::Halfs(vec![0; count]),
            (2, _) => Self::Words(vec![0; count]),
            (3, _) => Self::DWords(vec![0; count]),
            _ => panic!("Invalid unit size log2: {}", unit_size_log2),
        }
    }

    pub fn from_const_data(data: ConstData, type_ctx: &TypeContext) -> Self {
        match data {
            ConstData::Undef(ty) | ConstData::Zero(ty) => {
                let size_bytes = ty.get_size(type_ctx);
                let unit_bytes_log2 = ty.get_align_log2(type_ctx);
                let count = size_bytes >> unit_bytes_log2;
                Self::from_zeroes(unit_bytes_log2, count)
            }
            ConstData::PtrNull(_) => Self::DWord(0),
            ConstData::Int(apint) => match apint.bits() {
                8 => Self::Byte(apint.as_unsigned() as u8),
                16 => Self::Half(apint.as_unsigned() as u16),
                32 => Self::Word(apint.as_unsigned() as u32),
                64 => Self::DWord(apint.as_unsigned() as u64),
                _ => panic!("Unsupported integer bit width: {}", apint.bits()),
            },
            ConstData::Float(FPKind::Ieee32, x) => Self::Word((x as f32).to_bits()),
            ConstData::Float(FPKind::Ieee64, x) => Self::DWord(x.to_bits()),
        }
    }

    pub fn from_const_primitive_data(data: ConstData) -> Self {
        match data {
            ConstData::Undef(ty) | ConstData::Zero(ty) => {
                let bits_log2 = match ty {
                    ValTypeID::Ptr => 3, // Assuming 64-bit pointers
                    ValTypeID::Int(bits) => match bits {
                        8 => 0,
                        16 => 1,
                        32 => 2,
                        64 => 3,
                        _ => panic!("Unsupported integer bit width: {}", bits),
                    },
                    ValTypeID::Float(fpkind) => match fpkind {
                        FPKind::Ieee32 => 2,
                        FPKind::Ieee64 => 3,
                    },
                    ValTypeID::Void
                    | ValTypeID::Array(_)
                    | ValTypeID::Struct(_)
                    | ValTypeID::StructAlias(_)
                    | ValTypeID::Func(_) => panic!("Unsupported type for zero/undef data: {ty:?}"),
                };
                match bits_log2 {
                    0 => Self::Byte(0),
                    1 => Self::Half(0),
                    2 => Self::Word(0),
                    3 => Self::DWord(0),
                    _ => panic!("Unsupported bit width for zero/undef data: {}", bits_log2),
                }
            }
            ConstData::PtrNull(_) => Self::DWord(0),
            ConstData::Int(apint) => match apint.bits() {
                8 => Self::Byte(apint.as_unsigned() as u8),
                16 => Self::Half(apint.as_unsigned() as u16),
                32 => Self::Word(apint.as_unsigned() as u32),
                64 => Self::DWord(apint.as_unsigned() as u64),
                _ => panic!("Unsupported integer bit width: {}", apint.bits()),
            },
            ConstData::Float(FPKind::Ieee32, x) => Self::Word((x as f32).to_bits()),
            ConstData::Float(FPKind::Ieee64, x) => Self::DWord(x.to_bits()),
        }
    }

    pub fn try_connect(&mut self, other: &Self) -> bool {
        if self.unit_size_log2() != other.unit_size_log2() {
            return false; // Different unit sizes cannot be connected
        }
        self.box_self();
        match (self, other) {
            (Self::Bytes(data1), Self::Bytes(data2)) => {
                data1.extend(data2);
                true
            }
            (Self::Halfs(data1), Self::Halfs(data2)) => {
                data1.extend(data2);
                true
            }
            (Self::Words(data1), Self::Words(data2)) => {
                data1.extend(data2);
                true
            }
            (Self::DWords(data1), Self::DWords(data2)) => {
                data1.extend(data2);
                true
            }
            _ => false, // Types do not match for connection
        }
    }
}

pub struct DataGen {
    pub data: Vec<DataUnit>,
}

impl DataGen {
    pub fn new() -> Self {
        Self { data: Vec::new() }
    }

    pub fn add_data(&mut self, data: DataUnit) {
        if let Some(last) = self.data.last_mut() {
            if !last.try_connect(&data) {
                self.data.push(data.into_boxed());
            }
        } else {
            self.data.push(data.into_boxed());
        }
    }

    pub fn add_ir_data(&mut self, data: ConstData, type_ctx: &TypeContext) {
        self.add_data(DataUnit::from_const_data(data, type_ctx));
    }
    pub fn add_ir_expr(
        &mut self,
        expr: ConstExprRef,
        tctx: &TypeContext,
        alloc_expr: &Slab<ConstExprData>,
    ) {
        let expr_data = expr.to_data(alloc_expr);
        match expr_data {
            ConstExprData::Array(a) => self.add_aggr(&a.elems, tctx, alloc_expr),
            ConstExprData::Struct(s) => self.add_aggr(&s.elems, tctx, alloc_expr),
        }
    }
    pub fn add_ir_expr_from_module(&mut self, expr: ConstExprRef, module: &Module) {
        let allocs = module.borrow_allocs();
        let type_ctx = &*module.type_ctx;
        self.add_ir_expr(expr, type_ctx, &allocs.exprs);
    }
    fn add_aggr(
        &mut self,
        values: &[ValueSSA],
        type_ctx: &TypeContext,
        alloc_expr: &Slab<ConstExprData>,
    ) {
        for value in values {
            match value {
                ValueSSA::ConstData(data) => {
                    self.add_ir_data(data.clone(), type_ctx);
                }
                ValueSSA::ConstExpr(expr) => {
                    self.add_ir_expr(*expr, type_ctx, alloc_expr);
                }
                ValueSSA::None
                | ValueSSA::FuncArg(..)
                | ValueSSA::Block(_)
                | ValueSSA::Inst(_)
                | ValueSSA::Global(_) => panic!("Unsupported value type in DataGen: {value:?}"),
            }
        }
    }
    pub fn add_ir_value(
        &mut self,
        value: ValueSSA,
        type_ctx: &TypeContext,
        alloc_expr: &Slab<ConstExprData>,
    ) -> Result<(), String> {
        match value {
            ValueSSA::ConstData(data) => self.add_ir_data(data, type_ctx),
            ValueSSA::ConstExpr(expr) => self.add_ir_expr(expr, type_ctx, alloc_expr),
            ValueSSA::None
            | ValueSSA::FuncArg(..)
            | ValueSSA::Block(_)
            | ValueSSA::Inst(_)
            | ValueSSA::Global(_) => {
                return Err(format!("Unsupported value type in DataGen: {value:?}"));
            }
        }
        Ok(())
    }

    pub fn collect_data(&self, section: Section) -> Vec<MirGlobalData> {
        let mut data = Vec::with_capacity(self.data.len());
        for unit in &self.data {
            let gdata = match unit {
                DataUnit::Byte(b) => MirGlobalData::new_bytes(section, &[*b]),
                DataUnit::Half(h) => MirGlobalData::new_half(section, &[*h]),
                DataUnit::Word(w) => MirGlobalData::new_word(section, &[*w]),
                DataUnit::DWord(d) => MirGlobalData::new_dword(section, &[*d]),
                DataUnit::Bytes(items) => MirGlobalData::new_bytes(section, items),
                DataUnit::Halfs(items) => MirGlobalData::new_half(section, items),
                DataUnit::Words(items) => MirGlobalData::new_word(section, items),
                DataUnit::DWords(items) => MirGlobalData::new_dword(section, items),
            };
            data.push(gdata);
        }
        data
    }
}

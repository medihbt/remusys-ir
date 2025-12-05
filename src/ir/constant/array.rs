use crate::{
    base::APInt,
    impl_traceable_from_common,
    ir::{
        ConstData, ExprID, ExprObj, IRAllocs, ISubExprID, ISubValueSSA, IUser, OperandSet, UseID,
        UseKind, ValueSSA,
        constant::expr::{ExprCommon, ExprRawPtr, ISubExpr},
    },
    typing::{ArrayTypeID, FPKind, IValType, ScalarType, TypeContext, TypingRes, ValTypeID},
};
use smallvec::SmallVec;
use std::{
    collections::BTreeMap,
    ops::{Range, RangeFrom},
};

pub trait IArrayExpr: ISubExpr {
    fn get_array_type(&self) -> ArrayTypeID;
    fn get_elem_type(&self) -> ValTypeID;
    fn get_nelems(&self) -> usize;
    fn try_index_get(&self, allocs: &IRAllocs, index: usize) -> Option<ValueSSA>;
    fn index_get(&self, allocs: &IRAllocs, index: usize) -> ValueSSA {
        let nelems = self.get_nelems();
        match self.try_index_get(allocs, index) {
            Some(v) => v,
            None => panic!("IArrayExpr::index_get: index {index} out of bounds (nelems={nelems})"),
        }
    }

    fn foreach(&self, allocs: &IRAllocs, mut f: impl FnMut(&ValueSSA)) {
        let len = self.get_nelems();
        for i in 0..len {
            let val = self.index_get(allocs, i);
            f(&val);
        }
    }

    fn expand_to_array(&self, allocs: &IRAllocs) -> ArrayExpr;
    fn expand_to_array_id(&self, allocs: &IRAllocs) -> ArrayExprID {
        ArrayExprID::allocate(allocs, self.expand_to_array(allocs))
    }

    fn iter<'ir>(&'ir self, allocs: &'ir IRAllocs) -> ArrayExprIter<'ir>;
    fn value_iter<'ir>(&'ir self, allocs: &'ir IRAllocs) -> impl Iterator<Item = ValueSSA> + 'ir {
        ArrayExprIter(self.iter(allocs).0).map(|(val, _)| val)
    }
}
pub trait IArrayExprID: ISubExprID<ExprObjT: IArrayExpr> {
    fn get_array_type(self, allocs: &IRAllocs) -> ArrayTypeID {
        self.deref_ir(allocs).get_array_type()
    }
    fn get_elem_type(self, allocs: &IRAllocs) -> ValTypeID {
        self.deref_ir(allocs).get_elem_type()
    }
    fn get_nelems(self, allocs: &IRAllocs) -> usize {
        self.deref_ir(allocs).get_nelems()
    }

    fn try_index_get(self, allocs: &IRAllocs, index: usize) -> Option<ValueSSA> {
        self.deref_ir(allocs).try_index_get(allocs, index)
    }
    fn index_get(self, allocs: &IRAllocs, index: usize) -> ValueSSA {
        self.deref_ir(allocs).index_get(allocs, index)
    }

    fn iter(self, allocs: &IRAllocs) -> ArrayExprIter<'_> {
        self.deref_ir(allocs).iter(allocs)
    }
    fn value_iter(self, allocs: &IRAllocs) -> impl Iterator<Item = ValueSSA> + '_ {
        self.deref_ir(allocs).value_iter(allocs)
    }

    fn expand_to_array_id(self, allocs: &IRAllocs) -> ArrayExprID {
        self.deref_ir(allocs).expand_to_array_id(allocs)
    }
}

#[derive(Clone)]
pub struct ArrayExpr {
    pub common: ExprCommon,
    pub arrty: ArrayTypeID,
    pub elemty: ValTypeID,
    pub elems: SmallVec<[UseID; 4]>,
}
impl_traceable_from_common!(ArrayExpr, false);
impl IUser for ArrayExpr {
    fn get_operands(&self) -> OperandSet<'_> {
        OperandSet::Fixed(&self.elems)
    }
    fn operands_mut(&mut self) -> &mut [UseID] {
        &mut self.elems
    }
}
impl ISubExpr for ArrayExpr {
    fn get_common(&self) -> &ExprCommon {
        &self.common
    }
    fn common_mut(&mut self) -> &mut ExprCommon {
        &mut self.common
    }
    fn get_valtype(&self) -> ValTypeID {
        self.arrty.into_ir()
    }
    fn try_from_ir_ref(expr: &ExprObj) -> Option<&Self> {
        if let ExprObj::Array(arr) = expr { Some(arr) } else { None }
    }
    fn try_from_ir_mut(expr: &mut ExprObj) -> Option<&mut Self> {
        if let ExprObj::Array(arr) = expr { Some(arr) } else { None }
    }
    fn try_from_ir(expr: ExprObj) -> Option<Self> {
        if let ExprObj::Array(arr) = expr { Some(arr) } else { None }
    }
    fn into_ir(self) -> ExprObj {
        ExprObj::Array(self)
    }
    fn is_zero_const(&self, allocs: &IRAllocs) -> bool {
        if self.elems.is_empty() {
            return true;
        }
        self.elems
            .iter()
            .all(|&use_id| use_id.get_operand(allocs).is_zero_const(allocs))
    }
}
impl IArrayExpr for ArrayExpr {
    fn get_array_type(&self) -> ArrayTypeID {
        self.arrty
    }
    fn get_elem_type(&self) -> ValTypeID {
        self.elemty
    }
    fn get_nelems(&self) -> usize {
        self.elems.len()
    }
    fn try_index_get(&self, allocs: &IRAllocs, index: usize) -> Option<ValueSSA> {
        self.elems
            .get(index)
            .map(|&use_id| use_id.get_operand(allocs))
    }

    fn expand_to_array(&self, allocs: &IRAllocs) -> ArrayExpr {
        let arrty = self.arrty;
        let elemty = self.elemty;
        let nelems = self.elems.len();
        let mut elems = SmallVec::with_capacity(nelems);
        for i in 0..nelems {
            let use_id = UseID::new(allocs, UseKind::ArrayElem(i));
            let operand = self.index_get(allocs, i);
            use_id.set_operand(allocs, operand);
            elems.push(use_id);
        }
        ArrayExpr { common: ExprCommon::none(), arrty, elemty, elems }
    }

    fn iter<'ir>(&'ir self, allocs: &'ir IRAllocs) -> ArrayExprIter<'ir> {
        let inner = iter::ArrayExprIter { inner: self.elems.iter(), allocs };
        ArrayExprIter(iter::Impl::Array(inner))
    }
}
impl ArrayExpr {
    pub fn new_uninit(allocs: &IRAllocs, tctx: &TypeContext, arrty: ArrayTypeID) -> Self {
        let elemty = arrty.get_element_type(tctx);
        let nelems = arrty.get_num_elements(tctx);
        Self::new_full_uninit(allocs, arrty, elemty, nelems)
    }

    fn new_full_uninit(
        allocs: &IRAllocs,
        arrty: ArrayTypeID,
        elemty: ValTypeID,
        nelems: usize,
    ) -> Self {
        let elems = {
            let mut elems = SmallVec::with_capacity(nelems);
            for i in 0..nelems {
                let use_id = UseID::new(allocs, UseKind::ArrayElem(i));
                elems.push(use_id);
            }
            elems
        };
        Self { common: ExprCommon::none(), arrty, elemty, elems }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct ArrayExprID(pub ExprRawPtr);

impl ISubExprID for ArrayExprID {
    type ExprObjT = ArrayExpr;

    fn from_raw_ptr(id: ExprRawPtr) -> Self {
        ArrayExprID(id)
    }
    fn into_raw_ptr(self) -> ExprRawPtr {
        self.0
    }
}
impl IArrayExprID for ArrayExprID {
    fn expand_to_array_id(self, _: &IRAllocs) -> ArrayExprID {
        self
    }
}
impl ArrayExprID {
    pub fn new_uninit(allocs: &IRAllocs, tctx: &TypeContext, arrty: ArrayTypeID) -> Self {
        Self::allocate(allocs, ArrayExpr::new_uninit(allocs, tctx, arrty))
    }

    pub fn get_arrty(self, allocs: &IRAllocs) -> ArrayTypeID {
        self.deref_ir(allocs).arrty
    }
    pub fn get_elemty(self, allocs: &IRAllocs) -> ValTypeID {
        self.deref_ir(allocs).elemty
    }
    pub fn get_elems(self, allocs: &IRAllocs) -> &[UseID] {
        &self.deref_ir(allocs).elems
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ConstKind {
    I8,
    I16,
    I32,
    I64,
    I128,
    F32,
    F64,
    APInt,
    FreeStyle,
}
impl ConstKind {
    fn from_cdata(cdata: &ConstData) -> Self {
        match cdata {
            ConstData::Int(apint) => match apint.bits() {
                8 => ConstKind::I8,
                16 => ConstKind::I16,
                32 => ConstKind::I32,
                64 => ConstKind::I64,
                128 => ConstKind::I128,
                _ => ConstKind::APInt,
            },
            ConstData::Float(fp_kind, _) => match fp_kind {
                FPKind::Ieee32 => ConstKind::F32,
                FPKind::Ieee64 => ConstKind::F64,
            },
            _ => ConstKind::FreeStyle,
        }
    }
    fn from_value(val: &ValueSSA) -> Option<Self> {
        let ValueSSA::ConstData(cdata) = val else {
            return None;
        };
        Some(Self::from_cdata(cdata))
    }

    fn is_int(self) -> bool {
        use ConstKind::*;
        matches!(self, I8 | I16 | I32 | I64 | I128 | APInt)
    }
}
impl ArrayExprID {
    /// Try to zip the array expression into a more compact representation, if possible.
    pub fn zip(self, allocs: &IRAllocs) -> ExprID {
        let arrobj = self.deref_ir(allocs);
        let elemty = arrobj.get_elem_type();
        let nelems = arrobj.get_nelems();
        if nelems <= 4 {
            return self.raw_into();
        }

        let mut builder = ArrayBuilder::new_internal(arrobj.arrty, elemty, nelems);
        for (i, &u) in arrobj.elems.iter().enumerate() {
            let val = u.get_operand(allocs);
            let Ok(_) = builder.push(allocs, val) else {
                log::debug!("{self:?}::zip: failed to push value (#{i}) during zip");
                return self.raw_into();
            };
        }
        match builder.mode {
            ArrayBuildStat::Empty | ArrayBuildStat::Full => self.raw_into(),
            _ => builder.build_id(allocs).unwrap_or_else(|e| {
                log::debug!("{self:?}::zip: failed to build zipped array: {e}");
                self.raw_into()
            }),
        }
    }
}

#[derive(Clone)]
pub enum ConstArrayData {
    I8(SmallVec<[i8; 24]>),
    I16(SmallVec<[i16; 12]>),
    I32(SmallVec<[i32; 6]>),
    I64(SmallVec<[i64; 3]>),
    I128(Box<[i128]>),
    F32(SmallVec<[f32; 6]>),
    F64(SmallVec<[f64; 3]>),
    APInt(Box<[APInt]>),
    FreeStyle(Box<[ConstData]>),
}
impl ConstArrayData {
    pub fn len(&self) -> usize {
        match self {
            ConstArrayData::I8(v) => v.len(),
            ConstArrayData::I16(v) => v.len(),
            ConstArrayData::I32(v) => v.len(),
            ConstArrayData::I64(v) => v.len(),
            ConstArrayData::I128(v) => v.len(),
            ConstArrayData::F32(v) => v.len(),
            ConstArrayData::F64(v) => v.len(),
            ConstArrayData::APInt(v) => v.len(),
            ConstArrayData::FreeStyle(v) => v.len(),
        }
    }
    pub fn index_get(&self, index: usize) -> ValueSSA {
        match self {
            ConstArrayData::I8(v) => APInt::new(v[index] as i128, 8).into(),
            ConstArrayData::I16(v) => APInt::new(v[index] as i128, 16).into(),
            ConstArrayData::I32(v) => APInt::new(v[index] as i128, 32).into(),
            ConstArrayData::I64(v) => APInt::new(v[index] as i128, 64).into(),
            ConstArrayData::I128(v) => APInt::new(v[index], 128).into(),
            ConstArrayData::F32(v) => {
                ValueSSA::ConstData(ConstData::Float(FPKind::Ieee32, v[index] as f64))
            }
            ConstArrayData::F64(v) => {
                ValueSSA::ConstData(ConstData::Float(FPKind::Ieee64, v[index]))
            }
            ConstArrayData::APInt(v) => v[index].into(),
            ConstArrayData::FreeStyle(v) => v[index].into_ir(),
        }
    }
    fn index_set_unwrap(&mut self, index: usize, val: ValueSSA) {
        let val_int = val.as_apint();
        let val_cdata = ConstData::from_ir(val);
        let val_fp = if let ConstData::Float(_, v) = val_cdata { Some(v) } else { None };
        match self {
            ConstArrayData::I8(vec) => {
                vec[index] = val_int.unwrap().as_unsigned() as i8;
            }
            ConstArrayData::I16(vec) => {
                vec[index] = val_int.unwrap().as_unsigned() as i16;
            }
            ConstArrayData::I32(vec) => {
                vec[index] = val_int.unwrap().as_unsigned() as i32;
            }
            ConstArrayData::I64(vec) => {
                vec[index] = val_int.unwrap().as_unsigned() as i64;
            }
            ConstArrayData::I128(boxed) => {
                boxed[index] = val_int.unwrap().as_unsigned() as i128;
            }
            ConstArrayData::F32(vec) => {
                vec[index] = val_fp.unwrap() as f32;
            }
            ConstArrayData::F64(vec) => {
                vec[index] = val_fp.unwrap();
            }
            ConstArrayData::APInt(boxed) => {
                boxed[index] = val_int.unwrap();
            }
            ConstArrayData::FreeStyle(boxed) => {
                boxed[index] = val_cdata;
            }
        }
    }
    pub fn is_empty(&self) -> bool {
        self.len() == 0
    }

    pub fn is_zero_const(&self) -> bool {
        match self {
            ConstArrayData::I8(v) => v.iter().all(|&x| x == 0),
            ConstArrayData::I16(v) => v.iter().all(|&x| x == 0),
            ConstArrayData::I32(v) => v.iter().all(|&x| x == 0),
            ConstArrayData::I64(v) => v.iter().all(|&x| x == 0),
            ConstArrayData::I128(v) => v.iter().all(|&x| x == 0),
            ConstArrayData::F32(v) => v.iter().all(|&x| x == 0.0),
            ConstArrayData::F64(v) => v.iter().all(|&x| x == 0.0),
            ConstArrayData::APInt(v) => v.iter().all(|x| x.is_zero()),
            ConstArrayData::FreeStyle(v) => v.iter().all(|x| x.is_zero()),
        }
    }

    pub fn elem_type(&self) -> Option<ValTypeID> {
        let id = match self {
            ConstArrayData::I8(_) => ValTypeID::Int(8),
            ConstArrayData::I16(_) => ValTypeID::Int(16),
            ConstArrayData::I32(_) => ValTypeID::Int(32),
            ConstArrayData::I64(_) => ValTypeID::Int(64),
            ConstArrayData::I128(_) => ValTypeID::Int(128),
            ConstArrayData::F32(_) => ValTypeID::Float(FPKind::Ieee32),
            ConstArrayData::F64(_) => ValTypeID::Float(FPKind::Ieee64),
            ConstArrayData::APInt(v) if !v.is_empty() => ValTypeID::Int(v[0].bits()),
            ConstArrayData::FreeStyle(v) if !v.is_empty() => v[0].get_valtype_noalloc(),
            _ => return None,
        };
        Some(id)
    }

    fn new_zeroed_internal(kind: ConstKind, len: usize) -> Self {
        match kind {
            ConstKind::I8 => ConstArrayData::I8(SmallVec::from_elem(0i8, len)),
            ConstKind::I16 => ConstArrayData::I16(SmallVec::from_elem(0i16, len)),
            ConstKind::I32 => ConstArrayData::I32(SmallVec::from_elem(0i32, len)),
            ConstKind::I64 => ConstArrayData::I64(SmallVec::from_elem(0i64, len)),
            ConstKind::I128 => ConstArrayData::I128(vec![0i128; len].into_boxed_slice()),
            ConstKind::F32 => ConstArrayData::F32(SmallVec::from_elem(0.0f32, len)),
            ConstKind::F64 => ConstArrayData::F64(SmallVec::from_elem(0.0f64, len)),
            ConstKind::APInt => {
                ConstArrayData::APInt(vec![APInt::new(0, 0); len].into_boxed_slice())
            }
            ConstKind::FreeStyle => ConstArrayData::FreeStyle(
                // 占位仅供分配，后续每位覆盖
                vec![ConstData::new_zeroed(ScalarType::Int(8)); len].into_boxed_slice(),
            ),
        }
    }
}
#[derive(Clone)]
pub struct DataArrayExpr {
    pub common: ExprCommon,
    pub arrty: ArrayTypeID,
    pub elemty: ScalarType,
    pub data: ConstArrayData,
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct DataArrayExprID(pub ExprRawPtr);
impl_traceable_from_common!(DataArrayExpr, false);
impl IUser for DataArrayExpr {
    fn get_operands(&self) -> OperandSet<'_> {
        OperandSet::Fixed(&[])
    }
    fn operands_mut(&mut self) -> &mut [UseID] {
        &mut []
    }
}
impl ISubExpr for DataArrayExpr {
    fn get_common(&self) -> &ExprCommon {
        &self.common
    }
    fn common_mut(&mut self) -> &mut ExprCommon {
        &mut self.common
    }
    fn get_valtype(&self) -> ValTypeID {
        self.arrty.into_ir()
    }
    fn try_from_ir_ref(expr: &ExprObj) -> Option<&Self> {
        match expr {
            ExprObj::DataArray(da) => Some(da),
            _ => None,
        }
    }
    fn try_from_ir_mut(expr: &mut ExprObj) -> Option<&mut Self> {
        match expr {
            ExprObj::DataArray(da) => Some(da),
            _ => None,
        }
    }
    fn try_from_ir(expr: ExprObj) -> Option<Self> {
        match expr {
            ExprObj::DataArray(da) => Some(da),
            _ => None,
        }
    }
    fn into_ir(self) -> ExprObj {
        ExprObj::DataArray(self)
    }
    fn is_zero_const(&self, _: &IRAllocs) -> bool {
        self.data.is_zero_const()
    }
}
impl IArrayExpr for DataArrayExpr {
    fn get_array_type(&self) -> ArrayTypeID {
        self.arrty
    }
    fn get_elem_type(&self) -> ValTypeID {
        self.elemty.into_ir()
    }
    fn get_nelems(&self) -> usize {
        self.data.len()
    }
    fn try_index_get(&self, _: &IRAllocs, index: usize) -> Option<ValueSSA> {
        if index < self.get_nelems() { Some(self.data.index_get(index)) } else { None }
    }

    fn expand_to_array(&self, allocs: &IRAllocs) -> ArrayExpr {
        let arr_expr = ArrayExpr::new_full_uninit(
            allocs,
            self.arrty,
            self.elemty.into_ir(),
            self.get_nelems(),
        );
        for i in 0..self.get_nelems() {
            let val = self.index_get(allocs, i);
            let use_id = arr_expr.elems[i];
            use_id.set_operand(allocs, val);
        }
        arr_expr
    }
    fn iter<'ir>(&'ir self, _: &'ir IRAllocs) -> ArrayExprIter<'ir> {
        let inner = iter::DataArrayExprIter { data: &self.data, index: 0, len: self.data.len() };
        ArrayExprIter(iter::Impl::DataArray(inner))
    }
}
impl DataArrayExpr {
    pub fn new_zeroed(tctx: &TypeContext, arrty: ArrayTypeID) -> Option<Self> {
        let elemty = arrty.get_element_type(tctx);
        let Ok(elemty) = ScalarType::try_from_ir(elemty) else {
            return None;
        };
        let nelems = arrty.get_num_elements(tctx);
        let data = match elemty {
            ScalarType::Int(bits) => match bits {
                8 => ConstArrayData::I8(SmallVec::from_elem(0i8, nelems)),
                16 => ConstArrayData::I16(SmallVec::from_elem(0i16, nelems)),
                32 => ConstArrayData::I32(SmallVec::from_elem(0i32, nelems)),
                64 => ConstArrayData::I64(SmallVec::from_elem(0i64, nelems)),
                128 => ConstArrayData::I128(vec![0i128; nelems].into_boxed_slice()),
                _ => ConstArrayData::APInt(vec![APInt::new(0, bits); nelems].into_boxed_slice()),
            },
            ScalarType::Float(fk) => match fk {
                FPKind::Ieee32 => ConstArrayData::F32(SmallVec::from_elem(0.0f32, nelems)),
                FPKind::Ieee64 => ConstArrayData::F64(SmallVec::from_elem(0.0f64, nelems)),
            },
            _ => ConstArrayData::FreeStyle(
                vec![ConstData::new_zeroed(elemty); nelems].into_boxed_slice(),
            ),
        };
        Some(Self { common: ExprCommon::none(), arrty, elemty, data })
    }
}
impl ISubExprID for DataArrayExprID {
    type ExprObjT = DataArrayExpr;

    fn from_raw_ptr(id: ExprRawPtr) -> Self {
        Self(id)
    }
    fn into_raw_ptr(self) -> ExprRawPtr {
        self.0
    }
}
impl IArrayExprID for DataArrayExprID {}
impl DataArrayExprID {
    pub fn get_data(self, allocs: &IRAllocs) -> &ConstArrayData {
        &self.deref_ir(allocs).data
    }
    pub fn new_zeroed(allocs: &IRAllocs, tctx: &TypeContext, arrty: ArrayTypeID) -> Option<Self> {
        let data_array = DataArrayExpr::new_zeroed(tctx, arrty)?;
        Some(Self::allocate(allocs, data_array))
    }
}

#[derive(Clone)]
pub struct SplatArrayExpr {
    pub common: ExprCommon,
    pub arrty: ArrayTypeID,
    pub elemty: ValTypeID,
    pub nelems: usize,
    pub element: [UseID; 1],
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct SplatArrayExprID(pub ExprRawPtr);

impl_traceable_from_common!(SplatArrayExpr, false);
impl IUser for SplatArrayExpr {
    fn get_operands(&self) -> OperandSet<'_> {
        OperandSet::Fixed(&self.element)
    }
    fn operands_mut(&mut self) -> &mut [UseID] {
        &mut self.element
    }
}
impl ISubExpr for SplatArrayExpr {
    fn get_common(&self) -> &ExprCommon {
        &self.common
    }
    fn common_mut(&mut self) -> &mut ExprCommon {
        &mut self.common
    }
    fn get_valtype(&self) -> ValTypeID {
        self.arrty.into_ir()
    }
    fn try_from_ir_ref(expr: &ExprObj) -> Option<&Self> {
        match expr {
            ExprObj::SplatArray(sa) => Some(sa),
            _ => None,
        }
    }
    fn try_from_ir_mut(expr: &mut ExprObj) -> Option<&mut Self> {
        match expr {
            ExprObj::SplatArray(sa) => Some(sa),
            _ => None,
        }
    }
    fn try_from_ir(expr: ExprObj) -> Option<Self> {
        match expr {
            ExprObj::SplatArray(sa) => Some(sa),
            _ => None,
        }
    }
    fn into_ir(self) -> ExprObj {
        ExprObj::SplatArray(self)
    }
    fn is_zero_const(&self, allocs: &IRAllocs) -> bool {
        self.element[0].get_operand(allocs).is_zero_const(allocs)
    }
}
impl IArrayExpr for SplatArrayExpr {
    fn get_array_type(&self) -> ArrayTypeID {
        self.arrty
    }
    fn get_elem_type(&self) -> ValTypeID {
        self.elemty
    }
    fn get_nelems(&self) -> usize {
        self.nelems
    }
    fn try_index_get(&self, allocs: &IRAllocs, index: usize) -> Option<ValueSSA> {
        if index < self.nelems { Some(self.element[0].get_operand(allocs)) } else { None }
    }
    fn index_get(&self, allocs: &IRAllocs, _: usize) -> ValueSSA {
        self.element[0].get_operand(allocs)
    }
    fn expand_to_array(&self, allocs: &IRAllocs) -> ArrayExpr {
        let arr = ArrayExpr::new_full_uninit(allocs, self.arrty, self.elemty, self.nelems);
        let val = self.element[0].get_operand(allocs);
        for i in 0..self.nelems {
            let use_id = arr.elems[i];
            use_id.set_operand(allocs, val);
        }
        arr
    }
    fn iter<'ir>(&'ir self, allocs: &'ir IRAllocs) -> ArrayExprIter<'ir> {
        let inner = iter::SplatArrayExprIter {
            uid: self.element[0],
            allocs,
            nelems: self.nelems,
            index: 0,
        };
        ArrayExprIter(iter::Impl::SplatArray(inner))
    }
}
impl SplatArrayExpr {
    pub fn pattern_use(&self) -> UseID {
        self.element[0]
    }
    pub fn get_pattern(&self, allocs: &IRAllocs) -> ValueSSA {
        self.pattern_use().get_operand(allocs)
    }
    pub fn set_pattern(&self, allocs: &IRAllocs, val: ValueSSA) {
        let use_id = self.pattern_use();
        use_id.set_operand(allocs, val);
    }

    pub fn new(
        allocs: &IRAllocs,
        tctx: &TypeContext,
        arrty: ArrayTypeID,
        element: ValueSSA,
    ) -> Self {
        let elemty = arrty.get_element_type(tctx);
        let nelems = arrty.get_num_elements(tctx);
        Self::new_full(allocs, arrty, elemty, nelems, element)
    }
    fn new_full(
        allocs: &IRAllocs,
        arrty: ArrayTypeID,
        elemty: ValTypeID,
        nelems: usize,
        element: ValueSSA,
    ) -> Self {
        let use_id = UseID::new(allocs, UseKind::SplatArrayElem);
        use_id.set_operand(allocs, element);
        Self {
            common: ExprCommon::none(),
            arrty,
            elemty,
            nelems,
            element: [use_id],
        }
    }
}
impl ISubExprID for SplatArrayExprID {
    type ExprObjT = SplatArrayExpr;

    fn from_raw_ptr(id: ExprRawPtr) -> Self {
        Self(id)
    }
    fn into_raw_ptr(self) -> ExprRawPtr {
        self.0
    }
}
impl IArrayExprID for SplatArrayExprID {}
impl SplatArrayExprID {
    pub fn elem_use(self, allocs: &IRAllocs) -> UseID {
        self.deref_ir(allocs).element[0]
    }
    pub fn get_elem(self, allocs: &IRAllocs) -> ValueSSA {
        let use_id = self.deref_ir(allocs).element[0];
        use_id.deref_ir(allocs).operand.get()
    }
    pub fn set_elem(self, allocs: &IRAllocs, val: ValueSSA) {
        let use_id = self.deref_ir(allocs).element[0];
        use_id.set_operand(allocs, val);
    }
    pub fn new(allocs: &IRAllocs, tctx: &TypeContext, arrty: ArrayTypeID, elem: ValueSSA) -> Self {
        let splat_array = SplatArrayExpr::new(allocs, tctx, arrty, elem);
        Self::allocate(allocs, splat_array)
    }
}

#[derive(Debug, Clone, Copy, thiserror::Error)]
pub enum ArrayBuildErr {
    #[error("Array element type mismatch: expected {0:?}, got {1:?}")]
    TypeMismatch(ValTypeID, ValTypeID),

    #[error("Array builder is full with elements")]
    Full,

    #[error("Building process NOT finished: {0} elems remaining")]
    Unfinished(usize),

    #[error("KVArray index out of bounds: {1} >= {0}")]
    IndexOutOfRange(usize, usize),
}
pub type ArrayBuildRes<T = ()> = Result<T, ArrayBuildErr>;

pub struct ArrayBuilder {
    arrty: ArrayTypeID,
    elemty: ValTypeID,
    nelems: usize,
    elems: Vec<ValueSSA>,
    mode: ArrayBuildStat,
}

impl ArrayBuilder {
    pub fn new(tctx: &TypeContext, arrty: ArrayTypeID) -> Self {
        let elemty = arrty.get_element_type(tctx);
        let nelems = arrty.get_num_elements(tctx);
        Self {
            arrty,
            elemty,
            nelems,
            elems: Vec::with_capacity(nelems),
            mode: ArrayBuildStat::Empty,
        }
    }
    fn new_internal(arrty: ArrayTypeID, elemty: ValTypeID, nelems: usize) -> Self {
        Self {
            arrty,
            elemty,
            nelems,
            elems: Vec::with_capacity(nelems),
            mode: ArrayBuildStat::Empty,
        }
    }
    pub fn push(&mut self, allocs: &IRAllocs, val: ValueSSA) -> ArrayBuildRes {
        if self.elems.len() >= self.nelems {
            return Err(ArrayBuildErr::Full);
        }
        let valty = val.get_valtype(allocs);
        if valty != self.elemty {
            return Err(ArrayBuildErr::TypeMismatch(self.elemty, valty));
        }
        self.mode = self.mode.update(val);
        self.elems.push(val);
        Ok(())
    }
    pub fn set_elems(&mut self, allocs: &IRAllocs, vals: &[ValueSSA]) -> ArrayBuildRes {
        self.mode = ArrayBuildStat::Empty;
        self.elems.clear();
        for val in vals {
            self.push(allocs, *val)?;
        }
        Ok(())
    }

    pub fn build_id(&mut self, allocs: &IRAllocs) -> ArrayBuildRes<ExprID> {
        if self.elems.len() < self.nelems {
            return Err(ArrayBuildErr::Unfinished(self.nelems - self.elems.len()));
        }
        let id = match self.mode {
            ArrayBuildStat::Empty => self.build_empty_array(allocs),
            ArrayBuildStat::ConstUniform(_, val) | ArrayBuildStat::NonConstUniform(val) => {
                self.build_splat_array(allocs, val)
            }
            ArrayBuildStat::ConstNonUniform(ck) => self.build_data_array(allocs, ck),
            ArrayBuildStat::Full => self.build_full_array(allocs),
        };
        Ok(id)
    }
    fn build_empty_array(&mut self, allocs: &IRAllocs) -> ExprID {
        let data_array = DataArrayExpr {
            common: ExprCommon::none(),
            arrty: self.arrty,
            elemty: ScalarType::from_ir(self.elemty),
            data: ConstArrayData::FreeStyle(Box::new([])),
        };
        let data_array_id = DataArrayExprID::allocate(allocs, data_array);
        data_array_id.raw_into()
    }
    fn build_data_array(&mut self, allocs: &IRAllocs, ck: ConstKind) -> ExprID {
        let mut inner = ConstArrayData::new_zeroed_internal(ck, self.nelems);
        for (i, val) in self.elems.iter().cloned().enumerate() {
            inner.index_set_unwrap(i, val);
        }
        let data_array = DataArrayExpr {
            common: ExprCommon::none(),
            arrty: self.arrty,
            elemty: ScalarType::from_ir(self.elemty),
            data: inner,
        };
        let data_array_id = DataArrayExprID::allocate(allocs, data_array);
        data_array_id.raw_into()
    }
    fn build_splat_array(&mut self, allocs: &IRAllocs, val: ValueSSA) -> ExprID {
        let splat_array =
            SplatArrayExpr::new_full(allocs, self.arrty, self.elemty, self.nelems, val);
        let splat_array_id = SplatArrayExprID::allocate(allocs, splat_array);
        splat_array_id.raw_into()
    }
    fn build_full_array(&mut self, allocs: &IRAllocs) -> ExprID {
        let arr_expr = ArrayExpr::new_full_uninit(allocs, self.arrty, self.elemty, self.nelems);
        for (i, val) in self.elems.iter().cloned().enumerate() {
            let use_id = arr_expr.elems[i];
            use_id.set_operand(allocs, val);
        }
        let arr_expr_id = ArrayExprID::allocate(allocs, arr_expr);
        arr_expr_id.raw_into()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ArrayBuildStat {
    Empty,
    ConstUniform(ConstKind, ValueSSA),
    NonConstUniform(ValueSSA),
    ConstNonUniform(ConstKind),
    Full,
}
impl ArrayBuildStat {
    fn update(self, val: ValueSSA) -> Self {
        match self {
            ArrayBuildStat::Empty => match ConstKind::from_value(&val) {
                Some(ck) => ArrayBuildStat::ConstUniform(ck, val),
                None => ArrayBuildStat::NonConstUniform(val),
            },
            // 已经是常量且目前全同，遇到不同的值时根据常量种类收敛
            ArrayBuildStat::ConstUniform(ck, initial) if val != initial => {
                Self::classify_value_constant(val, ck)
            }
            ArrayBuildStat::NonConstUniform(initial) if val != initial => ArrayBuildStat::Full,
            ArrayBuildStat::ConstNonUniform(ck) => Self::classify_value_constant(val, ck),
            _ => self,
        }
    }
    fn classify_value_constant(val: ValueSSA, ck: ConstKind) -> ArrayBuildStat {
        let Some(val_ck) = ConstKind::from_value(&val) else {
            // 出现了非常量，无法保持 DataArray 形态
            return ArrayBuildStat::Full;
        };
        if val_ck == ck {
            ArrayBuildStat::ConstNonUniform(ck)
        } else if val_ck.is_int() && ck.is_int() {
            ArrayBuildStat::ConstNonUniform(ConstKind::APInt)
        } else {
            ArrayBuildStat::ConstNonUniform(ConstKind::FreeStyle)
        }
    }
}

/// 稀疏存储、按键值对初始化的数组表达式。规定未指定的元素均为默认值 (operands[0])，放在操作数列表的最开头；
/// 后续为按键值对形式存储的非默认值元素。Key 以 `UseKind::KVArrayElem(i)` 形式存储在 `UseID` 中，
/// 按照升序排列。若最后一个键等于元素个数减一且无间隙，则判定前缀致密（`is_front_dense = true`）。
///
/// Sparsely-stored array expression initialized by key-value pairs. The unspecified elements are all
/// set to the default value (operands[0]), which is placed at the very beginning of the operand list; the
/// subsequent elements are non-default elements stored in key-value pair form.
/// The keys are stored in `UseID` in the form of `UseKind::KVArrayElem(i)`, arranged in ascending order.
/// If the last key equals the number of elements minus one and there are no gaps, the prefix is considered dense
/// (`is_front_dense = true`).
#[derive(Clone)]
pub struct KVArrayExpr {
    pub common: ExprCommon,
    pub arrty: ArrayTypeID,
    pub nelems: usize,
    pub elemty: ValTypeID,
    operands: SmallVec<[UseID; 4]>,
    pub is_front_dense: bool,
}
impl_traceable_from_common!(KVArrayExpr, false);
impl IUser for KVArrayExpr {
    fn get_operands(&self) -> OperandSet<'_> {
        OperandSet::Fixed(&self.operands)
    }
    fn operands_mut(&mut self) -> &mut [UseID] {
        &mut self.operands
    }
}
impl ISubExpr for KVArrayExpr {
    fn get_common(&self) -> &ExprCommon {
        &self.common
    }
    fn common_mut(&mut self) -> &mut ExprCommon {
        &mut self.common
    }
    fn get_valtype(&self) -> ValTypeID {
        self.arrty.into_ir()
    }
    fn try_from_ir_ref(expr: &ExprObj) -> Option<&Self> {
        match expr {
            ExprObj::KVArray(kv) => Some(kv),
            _ => None,
        }
    }
    fn try_from_ir_mut(expr: &mut ExprObj) -> Option<&mut Self> {
        match expr {
            ExprObj::KVArray(kv) => Some(kv),
            _ => None,
        }
    }
    fn try_from_ir(expr: ExprObj) -> Option<Self> {
        match expr {
            ExprObj::KVArray(kv) => Some(kv),
            _ => None,
        }
    }
    fn into_ir(self) -> ExprObj {
        ExprObj::KVArray(self)
    }
    fn is_zero_const(&self, allocs: &IRAllocs) -> bool {
        self.operands
            .iter()
            .all(|&use_id| use_id.get_operand(allocs).is_zero_const(allocs))
    }
}
impl IArrayExpr for KVArrayExpr {
    fn get_array_type(&self) -> ArrayTypeID {
        self.arrty
    }
    fn get_elem_type(&self) -> ValTypeID {
        self.elemty
    }
    fn get_nelems(&self) -> usize {
        self.nelems
    }
    fn try_index_get(&self, allocs: &IRAllocs, index: usize) -> Option<ValueSSA> {
        if index >= self.nelems {
            return None;
        }
        let use_id = self
            .get_elem_use(allocs, index)
            .unwrap_or(self.default_use());
        Some(use_id.get_operand(allocs))
    }
    fn expand_to_array(&self, allocs: &IRAllocs) -> ArrayExpr {
        let arr = ArrayExpr::new_full_uninit(allocs, self.arrty, self.elemty, self.nelems);
        for i in 0..self.nelems {
            let val = self.index_get(allocs, i);
            let use_id = arr.elems[i];
            use_id.set_operand(allocs, val);
        }
        arr
    }
    fn iter<'ir>(&'ir self, allocs: &'ir IRAllocs) -> ArrayExprIter<'ir> {
        if self.is_front_dense && self.elem_uses().len() == self.nelems {
            let inner = iter::ArrayExprIter { inner: self.elem_uses().iter(), allocs };
            ArrayExprIter(iter::Impl::Array(inner))
        } else if self.is_front_dense {
            let inner = iter::FrontDenseKVIter {
                elem_uses: self.elem_uses(),
                default_use: self.default_use(),
                allocs,
                index: 0,
                nelems: self.nelems,
            };
            ArrayExprIter(iter::Impl::FrontDenseKV(inner))
        } else {
            let inner = iter::KVArrayExprIter {
                elem_uses: self.elem_uses(),
                default_use: self.default_use(),
                allocs,
                nelems: self.nelems,
                virt_index: 0,
                phys_index: 0,
            };
            ArrayExprIter(iter::Impl::KVArray(inner))
        }
    }
}
impl KVArrayExpr {
    pub const OP_DEFAULT: usize = 0;
    pub const OP_ELEMS: RangeFrom<usize> = 1..;
    pub const OP_ELEMS_BEGIN: usize = 1;

    pub fn builder<'ir>(
        tctx: &TypeContext,
        allocs: &'ir IRAllocs,
        arrty: ArrayTypeID,
    ) -> KVArrayBuilder<'ir> {
        KVArrayBuilder::new(tctx, allocs, arrty)
    }
    pub fn builder_zeroed<'ir>(
        tctx: &TypeContext,
        allocs: &'ir IRAllocs,
        arrty: ArrayTypeID,
    ) -> TypingRes<KVArrayBuilder<'ir>> {
        KVArrayBuilder::with_zero(tctx, allocs, arrty)
    }

    pub fn default_use(&self) -> UseID {
        self.operands[Self::OP_DEFAULT]
    }
    pub fn get_default(&self, allocs: &IRAllocs) -> ValueSSA {
        self.default_use().get_operand(allocs)
    }
    pub fn set_default(&self, allocs: &IRAllocs, val: ValueSSA) {
        self.default_use().set_operand(allocs, val);
    }

    pub fn elem_uses(&self) -> &[UseID] {
        &self.operands[Self::OP_ELEMS]
    }
    pub fn get_elem_use(&self, allocs: &IRAllocs, index: usize) -> Option<UseID> {
        if index >= self.nelems {
            return None;
        }
        if self.is_front_dense {
            self.elem_uses().get(index).copied()
        } else {
            let index = self
                .elem_uses()
                .binary_search_by_key(&index, |u| {
                    let UseKind::KVArrayElem(uk) = u.get_kind(allocs) else {
                        unreachable!("Internal error: invalid UseKind in KVArrayExpr");
                    };
                    uk
                })
                .ok();
            index.map(|index| self.elem_uses()[index])
        }
    }

    pub fn elem_iter<'kv>(&'kv self, allocs: &'kv IRAllocs) -> KVArrayElemIter<'kv> {
        KVArrayElemIter { inner: self.elem_uses().iter(), allocs }
    }

    pub fn nondefault_index_range(&self, allocs: &IRAllocs) -> Range<usize> {
        let Some(last) = self.elem_uses().last() else {
            return 0..0;
        };
        let back_idx = match last.get_kind(allocs) {
            UseKind::KVArrayElem(index) => index,
            kind => panic!("Internal error: expected UseKind::KVArrayElem but got {kind:?}"),
        };
        0..back_idx + 1
    }

    pub fn is_full(&self) -> bool {
        self.elem_uses().len() == self.nelems
    }
}
pub struct KVArrayElemIter<'kv> {
    inner: std::slice::Iter<'kv, UseID>,
    allocs: &'kv IRAllocs,
}
impl<'kv> Iterator for KVArrayElemIter<'kv> {
    type Item = (usize, ValueSSA, UseID);

    fn next(&mut self) -> Option<Self::Item> {
        let uid = *self.inner.next()?;
        let uobj = uid.deref_ir(self.allocs);
        let UseKind::KVArrayElem(index) = uobj.get_kind() else {
            unreachable!("Internal Error: Unexpected UseKind in KVArrayElem segment");
        };
        Some((index, uobj.operand.get(), uid))
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct KVArrayExprID(pub ExprRawPtr);

impl ISubExprID for KVArrayExprID {
    type ExprObjT = KVArrayExpr;
    fn from_raw_ptr(id: ExprRawPtr) -> Self {
        Self(id)
    }
    fn into_raw_ptr(self) -> ExprRawPtr {
        self.0
    }
}
impl IArrayExprID for KVArrayExprID {}
impl KVArrayExprID {
    pub fn builder<'ir>(
        tctx: &TypeContext,
        allocs: &'ir IRAllocs,
        arrty: ArrayTypeID,
    ) -> KVArrayBuilder<'ir> {
        KVArrayBuilder::new(tctx, allocs, arrty)
    }
    pub fn builder_zeroed<'ir>(
        tctx: &TypeContext,
        allocs: &'ir IRAllocs,
        arrty: ArrayTypeID,
    ) -> TypingRes<KVArrayBuilder<'ir>> {
        KVArrayBuilder::with_zero(tctx, allocs, arrty)
    }

    pub fn default_use(self, allocs: &IRAllocs) -> UseID {
        self.deref_ir(allocs).default_use()
    }
    pub fn get_default(self, allocs: &IRAllocs) -> ValueSSA {
        self.deref_ir(allocs).get_default(allocs)
    }
    pub fn set_default(self, allocs: &IRAllocs, val: ValueSSA) {
        self.deref_ir(allocs).set_default(allocs, val);
    }

    pub fn elem_uses(self, allocs: &IRAllocs) -> &[UseID] {
        self.deref_ir(allocs).elem_uses()
    }
    pub fn elem_iter(self, allocs: &IRAllocs) -> KVArrayElemIter<'_> {
        self.deref_ir(allocs).elem_iter(allocs)
    }

    pub fn is_front_dense(self, allocs: &IRAllocs) -> bool {
        self.deref_ir(allocs).is_front_dense
    }

    pub fn get_array_type(self, allocs: &IRAllocs) -> ArrayTypeID {
        self.deref_ir(allocs).arrty
    }

    pub fn nondefault_index_range(self, allocs: &IRAllocs) -> Range<usize> {
        self.deref_ir(allocs).nondefault_index_range(allocs)
    }
    pub fn is_full(self, allocs: &IRAllocs) -> bool {
        self.deref_ir(allocs).is_full()
    }
}

pub struct KVArrayBuilder<'ir> {
    pub arrty: ArrayTypeID,
    pub elemty: ValTypeID,
    pub nelems: usize,
    pub default_val: ValueSSA,
    pub elems: BTreeMap<usize, ValueSSA>,
    pub allocs: &'ir IRAllocs,
}

impl<'ir> KVArrayBuilder<'ir> {
    pub fn new(tctx: &TypeContext, allocs: &'ir IRAllocs, arrty: ArrayTypeID) -> Self {
        let elemty = arrty.get_element_type(tctx);
        Self {
            arrty,
            elemty,
            nelems: arrty.get_num_elements(tctx),
            default_val: ValueSSA::ConstData(ConstData::Undef(elemty)),
            elems: BTreeMap::new(),
            allocs,
        }
    }
    pub fn with_zero(
        tctx: &TypeContext,
        allocs: &'ir IRAllocs,
        arrty: ArrayTypeID,
    ) -> TypingRes<Self> {
        let mut res = Self::new(tctx, allocs, arrty);
        res.default_val = ValueSSA::new_zero(res.elemty)?;
        Ok(res)
    }

    pub fn try_default_val(&mut self, val: ValueSSA) -> ArrayBuildRes<&mut Self> {
        let valty = val.get_valtype(self.allocs);
        if valty != self.elemty {
            return Err(ArrayBuildErr::TypeMismatch(self.elemty, valty));
        }
        self.default_val = val;
        Ok(self)
    }
    pub fn default_val(&mut self, val: ValueSSA) -> &mut Self {
        match self.try_default_val(val) {
            Ok(slf) => slf,
            Err(e) => panic!("Cannot set default value: {e}"),
        }
    }

    pub fn add_elem(&mut self, index: usize, elem: ValueSSA) -> ArrayBuildRes<&mut Self> {
        if index >= self.nelems {
            return Err(ArrayBuildErr::IndexOutOfRange(self.nelems, index));
        }
        let elemty = elem.get_valtype(self.allocs);
        if elemty != self.elemty {
            return Err(ArrayBuildErr::TypeMismatch(self.elemty, elemty));
        }
        self.elems.insert(index, elem);
        Ok(self)
    }
    pub fn del_elem(&mut self, index: usize) -> Option<ValueSSA> {
        self.elems.remove(&index)
    }

    pub fn try_with_kv_elems(
        &mut self,
        elems: impl IntoIterator<Item = (usize, ValueSSA)>,
    ) -> Result<&mut Self, (usize, usize, ArrayBuildErr)> {
        for (cnt, (index, elem)) in elems.into_iter().enumerate() {
            match self.add_elem(index, elem) {
                Ok(_) => {}
                Err(e) => return Err((cnt, index, e)),
            }
        }
        Ok(self)
    }
    pub fn with_kv_elems(
        &mut self,
        elems: impl IntoIterator<Item = (usize, ValueSSA)>,
    ) -> &mut Self {
        match self.try_with_kv_elems(elems) {
            Ok(slf) => slf,
            Err((cnt, index, e)) => {
                panic!("Cannot add elem at count {cnt} index {index}: {e}")
            }
        }
    }
    pub fn try_with_arr_elems(
        &mut self,
        elems: &[ValueSSA],
    ) -> Result<&mut Self, (usize, ArrayBuildErr)> {
        self.try_with_kv_elems(elems.iter().copied().enumerate())
            .map_err(|(_, index, err)| (index, err))
    }
    pub fn with_arr_elems(&mut self, elems: &[ValueSSA]) -> &mut Self {
        self.with_kv_elems(elems.iter().copied().enumerate())
    }

    pub fn build_item(&mut self) -> KVArrayExpr {
        let allocs = self.allocs;
        let mut operands: SmallVec<[UseID; 4]> = SmallVec::with_capacity(self.elems.len() + 1);
        operands.push({
            let elem = UseID::new(allocs, UseKind::KVArrayDefaultElem);
            elem.set_operand(allocs, self.default_val);
            elem
        });
        for (&index, &val) in &self.elems {
            let elem = UseID::new(allocs, UseKind::KVArrayElem(index));
            elem.set_operand(allocs, val);
            operands.push(elem);
        }
        let is_front_dense = match self.elems.last_key_value() {
            None => true,
            Some((&idx, _)) => idx + 1 == self.elems.len(),
        };
        KVArrayExpr {
            common: ExprCommon::new(allocs),
            arrty: self.arrty,
            nelems: self.nelems,
            elemty: self.elemty,
            operands,
            is_front_dense,
        }
    }

    pub fn build_id(&mut self) -> KVArrayExprID {
        let allocs = self.allocs;
        KVArrayExprID::allocate(allocs, self.build_item())
    }
}

mod iter {
    use crate::ir::{ConstArrayData, IRAllocs, UseID, UseKind, ValueSSA};

    pub(super) struct ArrayExprIter<'ir> {
        pub inner: std::slice::Iter<'ir, UseID>,
        pub allocs: &'ir IRAllocs,
    }
    impl<'ir> Iterator for ArrayExprIter<'ir> {
        type Item = (ValueSSA, Option<UseID>);
        fn next(&mut self) -> Option<Self::Item> {
            let uid = *self.inner.next()?;
            Some((uid.get_operand(self.allocs), Some(uid)))
        }
    }

    pub(super) struct DataArrayExprIter<'ir> {
        pub data: &'ir ConstArrayData,
        pub index: usize,
        pub len: usize,
    }
    impl<'ir> Iterator for DataArrayExprIter<'ir> {
        type Item = (ValueSSA, Option<UseID>);
        fn next(&mut self) -> Option<Self::Item> {
            if self.index >= self.len {
                return None;
            }
            let val = self.data.index_get(self.index);
            self.index += 1;
            Some((val, None))
        }
    }

    pub(super) struct SplatArrayExprIter<'ir> {
        pub uid: UseID,
        pub allocs: &'ir IRAllocs,
        pub nelems: usize,
        pub index: usize,
    }
    impl<'ir> Iterator for SplatArrayExprIter<'ir> {
        type Item = (ValueSSA, Option<UseID>);
        fn next(&mut self) -> Option<Self::Item> {
            if self.index >= self.nelems {
                return None;
            }
            self.index += 1;
            Some((self.uid.get_operand(self.allocs), Some(self.uid)))
        }
    }

    pub(super) struct FrontDenseKVIter<'ir> {
        pub elem_uses: &'ir [UseID],
        pub default_use: UseID,
        pub allocs: &'ir IRAllocs,
        pub index: usize,
        pub nelems: usize,
    }
    impl<'ir> Iterator for FrontDenseKVIter<'ir> {
        type Item = (ValueSSA, Option<UseID>);

        fn next(&mut self) -> Option<Self::Item> {
            if self.index >= self.nelems {
                return None;
            }
            if self.index >= self.elem_uses.len() {
                let val = self.default_use.get_operand(self.allocs);
                self.index += 1;
                return Some((val, Some(self.default_use)));
            }
            let use_id = self.elem_uses[self.index];
            if use_id.get_kind(self.allocs) != UseKind::KVArrayElem(self.index) {
                unreachable!("Internal Error: Unexpected UseKind in KVArrayElem segment");
            }
            let val = use_id.get_operand(self.allocs);
            self.index += 1;
            Some((val, Some(use_id)))
        }
    }

    pub(super) struct KVArrayExprIter<'ir> {
        pub elem_uses: &'ir [UseID],
        pub default_use: UseID,
        pub allocs: &'ir IRAllocs,
        pub nelems: usize,
        pub virt_index: usize,
        pub phys_index: usize,
    }
    impl<'ir> Iterator for KVArrayExprIter<'ir> {
        type Item = (ValueSSA, Option<UseID>);

        fn next(&mut self) -> Option<Self::Item> {
            if self.virt_index >= self.nelems {
                return None;
            }
            if self.phys_index < self.elem_uses.len() {
                let use_id = self.elem_uses[self.phys_index];
                let UseKind::KVArrayElem(index) = use_id.get_kind(self.allocs) else {
                    unreachable!("Internal Error: Unexpected UseKind in KVArrayElem segment");
                };
                if index == self.virt_index {
                    let val = use_id.get_operand(self.allocs);
                    self.phys_index += 1;
                    self.virt_index += 1;
                    return Some((val, Some(use_id)));
                }
            }
            let val = self.default_use.get_operand(self.allocs);
            self.virt_index += 1;
            Some((val, Some(self.default_use)))
        }
    }

    pub(super) enum Impl<'ir> {
        Array(ArrayExprIter<'ir>),
        DataArray(DataArrayExprIter<'ir>),
        SplatArray(SplatArrayExprIter<'ir>),
        FrontDenseKV(FrontDenseKVIter<'ir>),
        KVArray(KVArrayExprIter<'ir>),
    }
}

pub struct ArrayExprIter<'ir>(iter::Impl<'ir>);

impl<'ir> Iterator for ArrayExprIter<'ir> {
    type Item = (ValueSSA, Option<UseID>);

    fn next(&mut self) -> Option<Self::Item> {
        match &mut self.0 {
            iter::Impl::Array(it) => it.next(),
            iter::Impl::DataArray(it) => it.next(),
            iter::Impl::SplatArray(it) => it.next(),
            iter::Impl::FrontDenseKV(it) => it.next(),
            iter::Impl::KVArray(it) => it.next(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MixArrayExprID {
    Array(ArrayExprID),
    Data(DataArrayExprID),
    Splat(SplatArrayExprID),
    KV(KVArrayExprID),
    Zero(ArrayTypeID),
}
impl From<MixArrayExprID> for ValueSSA {
    fn from(val: MixArrayExprID) -> Self {
        use crate::typing::AggrType;
        let expr = match val {
            MixArrayExprID::Array(expr) => expr.raw_into(),
            MixArrayExprID::Data(expr) => expr.raw_into(),
            MixArrayExprID::Splat(expr) => expr.raw_into(),
            MixArrayExprID::KV(expr) => expr.raw_into(),
            MixArrayExprID::Zero(zat) => return ValueSSA::AggrZero(AggrType::Array(zat)),
        };
        ValueSSA::ConstExpr(expr)
    }
}
impl MixArrayExprID {
    pub fn try_index_get(
        self,
        allocs: &IRAllocs,
        tctx: &TypeContext,
        index: usize,
    ) -> Option<ValueSSA> {
        match self {
            MixArrayExprID::Array(arr) => arr.try_index_get(allocs, index),
            MixArrayExprID::Data(arr) => arr.try_index_get(allocs, index),
            MixArrayExprID::Splat(arr) => arr.try_index_get(allocs, index),
            MixArrayExprID::KV(arr) => arr.try_index_get(allocs, index),
            MixArrayExprID::Zero(aty) => ValueSSA::new_zero(aty.get_element_type(tctx)).ok(),
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{
        base::APInt,
        ir::{
            GlobalVarBuilder, IArrayExprID, IGlobalVarBuildable, IRBuilder, IRWriter, ISubExprID,
            ISubValueSSA, KVArrayBuilder, ValueSSA, global::Linkage,
        },
        typing::{ArchInfo, ArrayTypeID, IValType, ValTypeID},
    };

    #[test]
    fn kvarray() {
        let builder = IRBuilder::new_inlined(ArchInfo::new_host(), "KVArrayTesting");
        let tctx = builder.tctx();
        let ai32n10 = ArrayTypeID::new(tctx, ValTypeID::Int(32), 10);
        let kv_array_id = {
            let mut kv_builder = KVArrayBuilder::with_zero(tctx, builder.allocs(), ai32n10)
                .expect("Failed to create KVArrayBuilder with zeroed default value");
            kv_builder
                .add_elem(0, ValueSSA::from(APInt::from(0i32)))
                .unwrap()
                .add_elem(1, ValueSSA::from(APInt::from(1i32)))
                .unwrap()
                .add_elem(3, ValueSSA::from(APInt::from(3i32)))
                .unwrap()
                .add_elem(4, ValueSSA::from(APInt::from(4i32)))
                .unwrap()
                .add_elem(9, ValueSSA::from(APInt::from(9i32)))
                .unwrap()
                .build_id()
        };
        GlobalVarBuilder::new("arr", ai32n10.into_ir())
            .linkage(Linkage::DSOLocal)
            .initval(kv_array_id.raw_into().into_ir())
            .build_id(builder.module())
            .expect("Failed to build global variable `arr`");
        let module = builder.module;
        let mut out = std::io::stdout();
        let writer = IRWriter::from_module(&mut out, &module);
        writer.write_module();

        for (cnt, (val, u)) in kv_array_id.iter(&module.allocs).enumerate() {
            println!("[{cnt}]: Value: {val:?}, UseID: {u:?}");
        }
    }
}

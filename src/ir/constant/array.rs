use crate::{
    base::APInt,
    impl_traceable_from_common,
    ir::{
        ConstData, ExprID, ExprObj, IRAllocs, ISubExprID, ISubValueSSA, IUser, OperandSet, UseID,
        UseKind, ValueSSA,
        constant::expr::{ExprCommon, ExprRawPtr, ISubExpr},
    },
    typing::{ArrayTypeID, FPKind, IValType, ScalarType, TypeContext, ValTypeID},
};
use smallvec::SmallVec;

pub trait IArrayExpr: ISubExpr {
    fn get_array_type(&self) -> ArrayTypeID;
    fn get_elem_type(&self) -> ValTypeID;
    fn get_nelems(&self) -> usize;
    fn index_get(&self, allocs: &IRAllocs, index: usize) -> ValueSSA;

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

    fn index_get(self, allocs: &IRAllocs, index: usize) -> ValueSSA {
        self.deref_ir(allocs).index_get(allocs, index)
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
    fn index_get(&self, allocs: &IRAllocs, index: usize) -> ValueSSA {
        let use_id = self.elems[index];
        use_id.deref_ir(allocs).operand.get()
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
                ValueSSA::ConstData(ConstData::Float(FPKind::Ieee64, v[index] as f64))
            }
            ConstArrayData::APInt(v) => v[index].clone().into(),
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
        return Some(id);
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
    fn index_get(&self, _: &IRAllocs, index: usize) -> ValueSSA {
        self.data.index_get(index)
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
    fn index_get(&self, allocs: &IRAllocs, _index: usize) -> ValueSSA {
        let use_id = self.element[0];
        use_id.deref_ir(allocs).operand.get()
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
            self.push(allocs, val.clone())?;
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
                Some(ck) => ArrayBuildStat::ConstUniform(ck, val.clone()),
                None => ArrayBuildStat::NonConstUniform(val.clone()),
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

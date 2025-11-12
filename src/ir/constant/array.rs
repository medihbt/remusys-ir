use crate::{
    base::APInt,
    impl_traceable_from_common,
    ir::{
        ConstData, ExprObj, IRAllocs, ISubExprID, ISubValueSSA, IUser, OperandSet, UseID, UseKind,
        ValueSSA,
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
    ) -> Option<Self> {
        let elemty = arrty.get_element_type(tctx);
        let nelems = arrty.get_num_elements(tctx);
        let use_id = UseID::new(allocs, UseKind::SplatArrayElem);
        use_id.set_operand(allocs, element);
        Some(Self {
            common: ExprCommon::none(),
            arrty,
            elemty,
            nelems,
            element: [use_id],
        })
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
    pub fn pattern_use(self, allocs: &IRAllocs) -> UseID {
        self.deref_ir(allocs).element[0]
    }
    pub fn get_pattern(self, allocs: &IRAllocs) -> ValueSSA {
        let use_id = self.deref_ir(allocs).element[0];
        use_id.deref_ir(allocs).operand.get()
    }

    pub fn new(
        allocs: &IRAllocs,
        tctx: &TypeContext,
        arrty: ArrayTypeID,
        element: ValueSSA,
    ) -> Option<Self> {
        let splat_array = SplatArrayExpr::new(allocs, tctx, arrty, element)?;
        Some(Self::allocate(allocs, splat_array))
    }
}

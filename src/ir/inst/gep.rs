use crate::{
    base::INullableValue,
    impl_debug_for_subinst_id, impl_traceable_from_common,
    ir::{
        IPtrUniqueUser, IPtrValue, IRAllocs, ISubInst, ISubInstID, ISubValueSSA, IUser, InstCommon,
        InstID, InstObj, JumpTargets, Module, Opcode, OperandSet, UseID, UseKind, ValueSSA,
    },
    typing::{IValType, StructTypeID, TypeContext, ValTypeID},
};
use smallvec::SmallVec;
use std::{cell::Cell, ops::RangeFrom};
use thiserror::Error;

/// 索引指针 (GEP) 指令
///
/// 该指令用于计算指针偏移，支持多级索引和复杂类型的内存访问。
///
/// ### 语法
///
/// ```llvm
/// getelementptr inbounds <1st unpacked ty>, ptr %<ptr>, <intty0> <sindex0>, <intty1> <sindex1>, ...
/// getelementptr <1st unpacked ty>, ptr %<ptr>, <intty0> <sindex0>, <intty1> <sindex1>, ...
/// ```
///
/// ### 操作数布局
///
/// - `operands[0]`: 基础指针 (Base Pointer)
/// - `operands[1..]`: 索引操作数 (Index Operands). 索引操作数必须是整数类型的, 而且不论什么
///    类型的索引都统一视为有符号整数索引.
pub struct GEPInst {
    pub common: InstCommon,
    operands: SmallVec<[UseID; 3]>,

    /// 是否为 inbounds GEP 指令.
    ///
    /// inbounds GEP 指令保证所有索引操作数在访问内存时不会越界, 否则行为未定义.
    pub inbounds_mark: Cell<bool>,

    /// 第一次取索引后被解包出来的类型.
    ///
    /// 例如: `getelementptr inbounds [4 x i32], ptr %ptr, i64 0, i64 %1` 中,
    /// 字段 `first_unpacked_ty` 是 `[4 x i32]` 类型.
    pub initial_ty: ValTypeID,

    /// 最后一次取索引后被解包出来的类型.
    ///
    /// 例如: `getelementptr inbounds [4 x i32], ptr %ptr, i64 0, i64 %1` 中,
    /// 字段 `final_unpacked_ty` 是 `i32` 类型.
    pub final_ty: ValTypeID,

    /// 对齐对数.
    pub align_log2: u8,

    /// 指针所指向的类型的对齐对数.
    pub pointee_align_log2: u8,
}
impl_traceable_from_common!(GEPInst, true);
impl IUser for GEPInst {
    fn get_operands(&self) -> OperandSet<'_> {
        OperandSet::Fixed(&self.operands)
    }
    fn operands_mut(&mut self) -> &mut [UseID] {
        &mut self.operands
    }
}
impl IPtrValue for GEPInst {
    fn get_ptr_pointee_type(&self) -> ValTypeID {
        self.final_ty
    }
    fn get_ptr_pointee_align(&self) -> u32 {
        1 << self.pointee_align_log2
    }
}
impl IPtrUniqueUser for GEPInst {
    fn get_operand_pointee_type(&self) -> ValTypeID {
        self.initial_ty
    }
    fn get_operand_pointee_align(&self) -> u32 {
        1 << self.pointee_align_log2
    }
}
impl ISubInst for GEPInst {
    fn get_common(&self) -> &InstCommon {
        &self.common
    }
    fn common_mut(&mut self) -> &mut InstCommon {
        &mut self.common
    }
    fn try_from_ir_ref(inst: &InstObj) -> Option<&Self> {
        match inst {
            InstObj::GEP(g) => Some(g),
            _ => None,
        }
    }
    fn try_from_ir_mut(inst: &mut InstObj) -> Option<&mut Self> {
        match inst {
            InstObj::GEP(g) => Some(g),
            _ => None,
        }
    }
    fn try_from_ir(inst: InstObj) -> Option<Self> {
        match inst {
            InstObj::GEP(g) => Some(g),
            _ => None,
        }
    }
    fn into_ir(self) -> InstObj {
        InstObj::GEP(self)
    }
    fn try_get_jts(&self) -> Option<JumpTargets<'_>> {
        None
    }
}
impl GEPInst {
    pub const OP_BASE_PTR: usize = 0;
    pub const OP_INDICES: RangeFrom<usize> = 1..;
    pub const OP_INDICES_BEGIN: usize = 1;

    pub fn new_uninit(
        allocs: &IRAllocs,
        initial_ty: ValTypeID,
        final_ty: ValTypeID,
        nindices: usize,
        align_log2: u8,
        pointee_align_log2: u8,
    ) -> Self {
        let operands = {
            let mut ops = SmallVec::with_capacity(1 + nindices);
            ops.push(UseID::new(allocs, UseKind::GepBase));
            for i in 0..nindices {
                ops.push(UseID::new(allocs, UseKind::GepIndex(i as u32)));
            }
            ops
        };
        Self {
            common: InstCommon::new(Opcode::IndexPtr, ValTypeID::Ptr),
            operands,
            inbounds_mark: Cell::new(false),
            initial_ty,
            final_ty,
            align_log2,
            pointee_align_log2,
        }
    }
    pub fn builder<'ir>(
        tctx: &'ir TypeContext,
        allocs: &'ir IRAllocs,
        initial_ty: ValTypeID,
    ) -> GEPInstBuilder<'ir> {
        GEPInstBuilder::new(tctx, allocs, initial_ty)
    }
    pub fn builder_from_module(module: &Module, initial_ty: ValTypeID) -> GEPInstBuilder<'_> {
        GEPInstBuilder::from_module(module, initial_ty)
    }

    pub fn get_inbounds(&self) -> bool {
        self.inbounds_mark.get()
    }
    pub fn set_inbounds(&self, inbounds: bool) {
        self.inbounds_mark.set(inbounds);
    }

    pub fn base_use(&self) -> UseID {
        self.operands[Self::OP_BASE_PTR]
    }
    pub fn get_base(&self, allocs: &IRAllocs) -> ValueSSA {
        self.base_use().get_operand(allocs)
    }
    pub fn set_base(&self, allocs: &IRAllocs, val: ValueSSA) {
        self.base_use().set_operand(allocs, val);
    }

    pub fn index_uses(&self) -> &[UseID] {
        &self.operands[Self::OP_INDICES]
    }
    pub fn index_use(&self, index: usize) -> UseID {
        self.operands[Self::OP_INDICES_BEGIN + index]
    }
    pub fn get_index(&self, allocs: &IRAllocs, index: usize) -> ValueSSA {
        self.index_use(index).get_operand(allocs)
    }
    pub fn set_index(&self, allocs: &IRAllocs, index: usize, val: ValueSSA) {
        self.index_use(index).set_operand(allocs, val);
    }
}

#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct GEPInstID(pub InstID);
impl_debug_for_subinst_id!(GEPInstID);
impl ISubInstID for GEPInstID {
    type InstObjT = GEPInst;

    fn raw_from_ir(id: InstID) -> Self {
        Self(id)
    }
    fn into_ir(self) -> InstID {
        self.0
    }
}
impl GEPInstID {
    pub fn new_uninit(
        allocs: &IRAllocs,
        initial_ty: ValTypeID,
        final_ty: ValTypeID,
        nindices: usize,
        align_log2: u8,
        pointee_align_log2: u8,
    ) -> Self {
        let inst = GEPInst::new_uninit(
            allocs,
            initial_ty,
            final_ty,
            nindices,
            align_log2,
            pointee_align_log2,
        );
        Self::allocate(allocs, inst)
    }
    pub fn builder<'ir>(
        tctx: &'ir TypeContext,
        allocs: &'ir IRAllocs,
        initial_ty: ValTypeID,
    ) -> GEPInstBuilder<'ir> {
        GEPInstBuilder::new(tctx, allocs, initial_ty)
    }
    pub fn builder_from_module(module: &Module, initial_ty: ValTypeID) -> GEPInstBuilder<'_> {
        GEPInstBuilder::from_module(module, initial_ty)
    }
    pub fn assert_indices_complete(self, allocs: &IRAllocs, tctx: &TypeContext) {
        let type_iter = GEPTypeIter::new(tctx, allocs, self);
        type_iter.run_check_assertion();
    }

    pub fn get_inbounds(self, allocs: &IRAllocs) -> bool {
        self.deref_ir(allocs).inbounds_mark.get()
    }
    pub fn set_inbounds(self, allocs: &IRAllocs, inbounds: bool) {
        self.deref_ir(allocs).inbounds_mark.set(inbounds);
    }
    pub fn get_initial_ty(self, allocs: &IRAllocs) -> ValTypeID {
        self.deref_ir(allocs).initial_ty
    }
    pub fn get_final_ty(self, allocs: &IRAllocs) -> ValTypeID {
        self.deref_ir(allocs).final_ty
    }
    pub fn get_align_log2(self, allocs: &IRAllocs) -> u8 {
        self.deref_ir(allocs).align_log2
    }
    pub fn get_pointee_align_log2(self, allocs: &IRAllocs) -> u8 {
        self.deref_ir(allocs).pointee_align_log2
    }
    pub fn get_align(self, allocs: &IRAllocs) -> u32 {
        1 << self.get_align_log2(allocs)
    }
    pub fn get_pointee_align(self, allocs: &IRAllocs) -> u32 {
        1 << self.get_pointee_align_log2(allocs)
    }

    pub fn base_use(self, allocs: &IRAllocs) -> UseID {
        self.deref_ir(allocs).base_use()
    }
    pub fn get_base(self, allocs: &IRAllocs) -> ValueSSA {
        self.deref_ir(allocs).get_base(allocs)
    }
    pub fn set_base(self, allocs: &IRAllocs, val: ValueSSA) {
        self.deref_ir(allocs).set_base(allocs, val);
    }

    pub fn index_uses(self, allocs: &IRAllocs) -> &[UseID] {
        self.deref_ir(allocs).index_uses()
    }
    pub fn index_use(self, allocs: &IRAllocs, index: usize) -> UseID {
        self.deref_ir(allocs).index_use(index)
    }
    pub fn get_index(self, allocs: &IRAllocs, index: usize) -> ValueSSA {
        self.deref_ir(allocs).get_index(allocs, index)
    }
    pub fn set_index(self, allocs: &IRAllocs, index: usize, val: ValueSSA) {
        self.deref_ir(allocs).set_index(allocs, index, val);
    }
}

/// GEP 指令的类型状态, 用于跟踪索引操作的类型变化.
///
/// ### LLVM GEP 语义
///
/// 以指令 `getelementptr inbounds [4 x i32], ptr %ptr, i64 0, i64 %1` 为例:
///
/// - 初始状态为 `InfLenArray([4 x i32])`，表示将指针指向的内存看作 `[∞ x [4 x i32]]`
/// - 第一次索引: `unpack(0)` -> 转换为 `ItSelf([4 x i32])`，从无限数组中选择第0个元素
/// - 第二次索引: `unpack(%1)` -> 转换为 `ItSelf(i32)`，对选中的数组进行索引
///
/// 如果对基础类型（如 i32）继续索引，则转换为 `Ends` 状态
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum GEPTypeState {
    /// 无限数组状态 - 将指针指向的内存看作 [∞ x ElementType]
    InfLenArray(ValTypeID),
    /// 正常类型状态 - 对具体类型进行索引操作
    BeforeUnpack(ValTypeID),
    /// 结束状态 - 不能再索引
    AfterUnpack,
}

#[derive(Debug, Clone, Copy, Error)]
pub enum GEPUnpackErr {
    #[error("index type {0:?} is not integer type")]
    IndexNotInt(ValTypeID),
    #[error("index out of range")]
    IndexOutOfRange,
    #[error("unpacking struct {0:?} with variable index {1:?}")]
    UnpackStructWithVariable(StructTypeID, ValueSSA),
    #[error("type {0:?} cannot unpack (expecting array, struct or vector)")]
    TypeCannotUnpack(ValTypeID),
}
pub type GEPTypeUnpackRes<T = ()> = Result<T, GEPUnpackErr>;

/// 用于遍历 GEP 指令的类型状态迭代器
///
/// 这个迭代器跟踪 GEP 指令每一步索引操作的类型变化，确保类型转换的正确性并计算最终结果类型。
pub struct GEPTypeUnpack<'ir> {
    allocs: &'ir IRAllocs,
    tctx: &'ir TypeContext,
    stat: GEPTypeState,
    id: u32,
}
impl<'ir> GEPTypeUnpack<'ir> {
    pub fn new_initial(tctx: &'ir TypeContext, allocs: &'ir IRAllocs, initty: ValTypeID) -> Self {
        Self { allocs, tctx, stat: GEPTypeState::InfLenArray(initty), id: 0 }
    }
    pub fn with_inst(tctx: &'ir TypeContext, allocs: &'ir IRAllocs, inst: GEPInstID) -> Self {
        let initial_ty = inst.get_initial_ty(allocs);
        Self {
            allocs,
            tctx,
            stat: GEPTypeState::InfLenArray(initial_ty),
            id: 0,
        }
    }

    pub fn current_state(&self) -> GEPTypeState {
        self.stat
    }
    pub fn ends(&self) -> bool {
        matches!(self.stat, GEPTypeState::AfterUnpack)
    }
    pub fn current_id(&self) -> u32 {
        self.id
    }

    pub fn try_unpack(&mut self, idx: ValueSSA) -> GEPTypeUnpackRes<GEPTypeState> {
        let idx_type = idx.get_valtype(self.allocs);
        let ValTypeID::Int(_) = idx_type else {
            return Err(GEPUnpackErr::IndexNotInt(idx_type));
        };

        let stat = match self.stat {
            GEPTypeState::AfterUnpack => GEPTypeState::AfterUnpack,
            GEPTypeState::InfLenArray(elemty) => GEPTypeState::BeforeUnpack(elemty),
            GEPTypeState::BeforeUnpack(ty) => self.do_unpack(ty, idx)?,
        };
        self.stat = stat;
        self.id += 1;
        Ok(stat)
    }
    pub fn unpack(&mut self, idx: ValueSSA) -> GEPTypeState {
        self.try_unpack(idx).expect("GEP unpack failed")
    }
    fn do_unpack(&self, aggrty: ValTypeID, idx: ValueSSA) -> GEPTypeUnpackRes<GEPTypeState> {
        use ValTypeID::*;
        match aggrty {
            Ptr | Int(_) | Float(_) => Ok(GEPTypeState::AfterUnpack),
            FixVec(v) => {
                let elemty = v.get_elem().into_ir();
                Ok(GEPTypeState::BeforeUnpack(elemty))
            }
            Array(a) => {
                let elemty = a.get_element_type(self.tctx);
                Ok(GEPTypeState::BeforeUnpack(elemty))
            }
            Struct(s) => {
                let elemty = Self::unpack_struct(self.tctx, s, idx)?;
                Ok(GEPTypeState::BeforeUnpack(elemty))
            }
            StructAlias(sa) => {
                let struc = sa.get_aliasee(self.tctx);
                let elemty = Self::unpack_struct(self.tctx, struc, idx)?;
                Ok(GEPTypeState::BeforeUnpack(elemty))
            }
            _ => Err(GEPUnpackErr::TypeCannotUnpack(aggrty)),
        }
    }
    fn unpack_struct(
        tctx: &TypeContext,
        struc: StructTypeID,
        idx: ValueSSA,
    ) -> GEPTypeUnpackRes<ValTypeID> {
        let Some(index) = idx.as_apint() else {
            return Err(GEPUnpackErr::UnpackStructWithVariable(struc, idx));
        };
        let index = index.as_signed() as usize;
        let fields = struc.get_fields(tctx);
        if let Some(&field) = fields.get(index) {
            Ok(field)
        } else {
            Err(GEPUnpackErr::IndexOutOfRange)
        }
    }
}

pub struct GEPTypeIter<'ir> {
    unpacker: GEPTypeUnpack<'ir>,
    indices: &'ir [UseID],
    index: usize,
}

impl<'ir> Iterator for GEPTypeIter<'ir> {
    type Item = (ValueSSA, ValTypeID);

    fn next(&mut self) -> Option<Self::Item> {
        use GEPTypeState::*;
        if self.index >= self.indices.len() {
            return None;
        }
        let index_op = self.indices[self.index].get_operand(self.allocs());
        let next_stat = self.unpacker.unpack(index_op);
        let next_ty = match next_stat {
            AfterUnpack => return None,
            InfLenArray(_) => {
                panic!("GEPTypeIter should not yield InfLenArray state")
            }
            BeforeUnpack(ty) => ty,
        };
        self.index += 1;
        Some((index_op, next_ty))
    }
    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.indices.len() - self.index;
        (0, Some(remaining))
    }
}
impl<'ir> GEPTypeIter<'ir> {
    pub fn allocs(&self) -> &'ir IRAllocs {
        self.unpacker.allocs
    }
    pub fn tctx(&self) -> &'ir TypeContext {
        self.unpacker.tctx
    }

    pub fn new(tctx: &'ir TypeContext, allocs: &'ir IRAllocs, inst: GEPInstID) -> GEPTypeIter<'ir> {
        let unpacker = GEPTypeUnpack::with_inst(tctx, allocs, inst);
        let indices = inst.index_uses(allocs);
        GEPTypeIter { unpacker, indices, index: 0 }
    }

    pub fn run_check_assertion(self) {
        for (..) in self {
            // Just iterate to trigger type unpacking and checks.
        }
    }
}

pub struct GEPInstBuilder<'ir> {
    initial_ty: ValTypeID,
    base_ptr: ValueSSA,
    indices: SmallVec<[(ValueSSA, ValTypeID); 4]>,
    inbounds: bool,
    unpacker: GEPTypeUnpack<'ir>,
    pointee_align_log2: u8,
    align_log2: Option<u8>,
}
impl<'ir> GEPInstBuilder<'ir> {
    pub fn new(tctx: &'ir TypeContext, allocs: &'ir IRAllocs, initial_ty: ValTypeID) -> Self {
        Self {
            initial_ty,
            base_ptr: ValueSSA::None,
            indices: SmallVec::new(),
            inbounds: false,
            align_log2: None,
            pointee_align_log2: initial_ty.get_align_log2(tctx),
            unpacker: GEPTypeUnpack::new_initial(tctx, allocs, initial_ty),
        }
    }

    pub fn from_module(module: &'ir Module, initial_ty: ValTypeID) -> Self {
        let Module { allocs, tctx, .. } = module;
        Self::new(tctx, allocs, initial_ty)
    }

    pub fn try_add_index(&mut self, idx: ValueSSA) -> GEPTypeUnpackRes<GEPTypeState> {
        let new_state = self.unpacker.try_unpack(idx)?;
        match new_state {
            GEPTypeState::AfterUnpack => {}
            GEPTypeState::BeforeUnpack(ty) => self.indices.push((idx, ty)),
            GEPTypeState::InfLenArray(_) => {
                panic!("GEPInstBuilder should not yield InfLenArray state")
            }
        }
        Ok(new_state)
    }
    pub fn add_index(&mut self, idx: ValueSSA) -> GEPTypeState {
        self.try_add_index(idx).expect("GEPInstBuilder failed")
    }
    pub fn add_indices(&mut self, indices: &[ValueSSA]) -> &mut Self {
        for idx in indices {
            self.add_index(*idx);
        }
        self
    }
    pub fn base_ptr(&mut self, ptr: ValueSSA) -> &mut Self {
        self.base_ptr = ptr;
        self
    }
    pub fn inbounds(&mut self, inbounds: bool) -> &mut Self {
        self.inbounds = inbounds;
        self
    }
    pub fn align_log2(&mut self, align_log2: u8) -> &mut Self {
        self.align_log2 = Some(align_log2);
        self
    }
    pub fn pointee_align_log2(&mut self, align_log2: u8) -> &mut Self {
        self.pointee_align_log2 = align_log2;
        self
    }

    fn allocs(&self) -> &'ir IRAllocs {
        self.unpacker.allocs
    }
    fn tctx(&self) -> &'ir TypeContext {
        self.unpacker.tctx
    }
    fn get_final_ty(&self) -> ValTypeID {
        if let Some((_, ty)) = self.indices.last() { *ty } else { self.initial_ty }
    }
    fn get_align_log2(&self) -> u8 {
        if let Some(a) = self.align_log2 {
            a
        } else {
            let final_ty = self.get_final_ty();
            final_ty.get_align_log2(self.tctx())
        }
    }
    fn dump_operands(&self) -> SmallVec<[UseID; 3]> {
        let mut operands = SmallVec::with_capacity(self.indices.len() + 1);
        operands.push({
            let u = UseID::new(self.allocs(), UseKind::GepBase);
            if self.base_ptr.is_nonnull() {
                u.set_operand(self.allocs(), self.base_ptr);
            }
            u
        });
        for (i, &(idx, _)) in self.indices.iter().enumerate() {
            let u = UseID::new(self.allocs(), UseKind::GepIndex(i as u32));
            u.set_operand(self.allocs(), idx);
            operands.push(u);
        }
        operands
    }

    pub fn build_inst(&self) -> GEPInst {
        GEPInst {
            common: InstCommon::new(Opcode::IndexPtr, ValTypeID::Ptr),
            operands: self.dump_operands(),
            inbounds_mark: Cell::new(self.inbounds),
            initial_ty: self.initial_ty,
            final_ty: self.get_final_ty(),
            align_log2: self.get_align_log2(),
            pointee_align_log2: self.pointee_align_log2,
        }
    }
    pub fn build_id(&self) -> GEPInstID {
        GEPInstID::allocate(self.allocs(), self.build_inst())
    }
}

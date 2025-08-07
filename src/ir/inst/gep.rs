use crate::{
    base::APInt,
    ir::{
        ConstData, IRAllocs, IRWriter, ISubInst, ISubValueSSA, InstCommon, InstData, InstRef,
        Opcode, PtrStorage, PtrUser, Use, UseKind, ValueSSA,
        inst::{ISubInstRef, InstOperands},
    },
    typing::{context::TypeContext, id::ValTypeID, types::StructTypeRef},
};
use std::{num::NonZero, panic, rc::Rc};

/// 索引指针（GEP）指令
///
/// 该指令用于计算指针偏移，支持多级索引和复杂类型的内存访问。
///
/// ### 语法
///
/// ```llvm
/// getelementptr inbounds <1st unpacked ty>, ptr %<ptr>, <intty0> <sindex0>, <intty1> <sindex1>, ...
/// ```
///
/// ### 操作数布局
///
/// - `operands[0]`: 基础指针 (Base Pointer)
/// - `operands[1..]`: 索引操作数 (Index Operands). 索引操作数必须是整数类型的, 而且不论什么
///    类型的索引都统一视为有符号整数索引.
#[derive(Debug)]
pub struct IndexPtr {
    common: InstCommon,
    operands: Box<[Rc<Use>]>,

    /// 第一次取索引后被解包出来的类型.
    ///
    /// 例如: `getelementptr inbounds [4 x i32], ptr %ptr, i64 0, i64 %1` 中,
    /// 字段 `first_unpacked_ty` 是 `[4 x i32]` 类型.
    pub first_unpacked_ty: ValTypeID,

    /// 最后一次取索引后被解包出来的类型.
    ///
    /// 例如: `getelementptr inbounds [4 x i32], ptr %ptr, i64 0, i64 %1` 中,
    /// 字段 `last_unpacked_ty` 是 `i32` 类型.
    pub last_unpacked_ty: ValTypeID,

    pub storage_align_log2: u8,
    pub ret_align_log2: u8,
}

impl ISubInst for IndexPtr {
    fn new_empty(opcode: Opcode) -> Self {
        if opcode != Opcode::IndexPtr {
            panic!("Tried to create an IndexPtr with non-Gep opcode");
        }
        Self {
            common: InstCommon::new(opcode, ValTypeID::Ptr),
            operands: Box::new([Use::new(UseKind::GepBase)]),
            first_unpacked_ty: ValTypeID::Void,
            last_unpacked_ty: ValTypeID::Void,
            storage_align_log2: 0,
            ret_align_log2: 0,
        }
    }

    fn try_from_ir(inst: &InstData) -> Option<&Self> {
        if let InstData::GEP(gep) = inst { Some(gep) } else { None }
    }
    fn try_from_ir_mut(inst: &mut InstData) -> Option<&mut Self> {
        if let InstData::GEP(gep) = inst { Some(gep) } else { None }
    }

    fn into_ir(self) -> InstData {
        InstData::GEP(self)
    }
    fn get_common(&self) -> &InstCommon {
        &self.common
    }
    fn common_mut(&mut self) -> &mut InstCommon {
        &mut self.common
    }
    fn is_terminator(&self) -> bool {
        false
    }
    fn get_operands(&self) -> InstOperands {
        InstOperands::Fixed(&self.operands)
    }
    fn operands_mut(&mut self) -> &mut [Rc<Use>] {
        &mut self.operands
    }

    fn fmt_ir(&self, id: Option<usize>, writer: &IRWriter) -> std::io::Result<()> {
        let Some(id) = id else {
            use std::io::{Error, ErrorKind::InvalidInput};
            return Err(Error::new(InvalidInput, "ID must be provided for GEPop"));
        };
        let opcode = self.get_opcode().get_name();
        write!(writer, "%{id} = getelementptr inbounds {opcode} ")?;
        writer.write_type(self.first_unpacked_ty)?;
        writer.write_str(", ptr")?;
        writer.write_operand(self.get_base())?;
        for u in self.index_uses() {
            let index = u.get_operand();
            let index_ty = index.get_valtype(&writer.allocs);
            writer.write_str(", ")?;
            writer.write_type(index_ty)?;
            writer.write_str(" ")?;
            writer.write_operand(index)?;
        }
        Ok(())
    }
}

impl PtrStorage for IndexPtr {
    fn get_stored_pointee_type(&self) -> ValTypeID {
        self.last_unpacked_ty
    }

    fn get_stored_pointee_align(&self) -> Option<NonZero<usize>> {
        NonZero::new(1usize << self.ret_align_log2)
    }
}

impl PtrUser for IndexPtr {
    fn get_operand_pointee_type(&self) -> ValTypeID {
        self.first_unpacked_ty
    }

    fn get_operand_align(&self) -> Option<NonZero<usize>> {
        NonZero::new(1usize << self.ret_align_log2)
    }
}

impl IndexPtr {
    /// 创建一个未初始化操作数的 GEP 指令
    pub fn new_raw(
        base_ty: ValTypeID,
        last_ty: ValTypeID,
        nindices: usize,
        storage_align_log2: u8,
        ret_align_log2: u8,
    ) -> Self {
        let operands = {
            let mut ops = Vec::with_capacity(nindices + 1);
            ops.push(Use::new(UseKind::GepBase));
            for i in 0..nindices {
                ops.push(Use::new(UseKind::GepIndex(i as u32)));
            }
            ops.into_boxed_slice()
        };
        Self {
            common: InstCommon::new(Opcode::IndexPtr, ValTypeID::Ptr),
            operands,
            first_unpacked_ty: base_ty,
            last_unpacked_ty: last_ty,
            storage_align_log2,
            ret_align_log2,
        }
    }

    /// 创建一个新的 GEP 指令, 并初始化操作数.
    pub fn new<'a, T>(
        type_ctx: &TypeContext,
        allocs: &IRAllocs,
        base_ptr: ValueSSA,
        base_ty: ValTypeID,
        indices: T,
    ) -> Self
    where
        T: IntoIterator<Item = &'a ValueSSA> + 'a,
        T::IntoIter: Clone,
    {
        let indices_iter = indices.into_iter();
        let indices_vec: Vec<&ValueSSA> = indices_iter.clone().collect(); // 收集到Vec以便重用

        let (last_ty, nindices) = {
            let mut indexer = GEPTypeIndexer::new_initial(type_ctx, allocs, base_ty);
            let mut nindices = 0;
            for idx in indices_iter {
                let new_state = indexer.unpack(*idx);
                if let GEPTypeState::Ends = new_state {
                    break;
                }
                nindices += 1;
            }
            match indexer.current_state() {
                GEPTypeState::ItSelf(ty) => (ty, nindices),
                GEPTypeState::Ends => {
                    panic!("GEP indexing ended prematurely, cannot determine final type")
                }
                GEPTypeState::InfLenArray(_) => {
                    panic!("GEP has no indices, cannot determine final type")
                }
            }
        };

        let gep = Self::new_raw(
            base_ty,
            last_ty,
            nindices,
            strict_ilog2(base_ty.try_get_instance_align(type_ctx).unwrap()),
            strict_ilog2(last_ty.try_get_instance_align(type_ctx).unwrap()),
        );

        // 设置索引操作数
        for (i, &idx) in indices_vec.iter().enumerate().take(nindices) {
            gep.operands[i + 1].set_operand(allocs, *idx);
        }
        gep.set_base(allocs, base_ptr);
        gep
    }

    /// 计算GEP指令的最终类型
    ///
    /// 根据索引序列计算最终的类型，并验证每一步的类型转换是否合法
    pub fn compute_result_type(
        &self,
        type_ctx: &TypeContext,
        allocs: &IRAllocs,
    ) -> Result<ValTypeID, String> {
        let mut indexer = GEPTypeIndexer {
            type_ctx,
            allocs,
            type_state: GEPTypeState::InfLenArray(self.first_unpacked_ty),
        };

        for use_ref in self.operands.iter().skip(1) {
            let idx = use_ref.get_operand();
            let new_state = indexer.unpack(idx);
            if matches!(new_state, GEPTypeState::Ends) {
                break;
            }
        }

        match indexer.type_state {
            GEPTypeState::ItSelf(ty) => Ok(ty),
            GEPTypeState::Ends => Err("GEP indexing ended prematurely".to_string()),
            GEPTypeState::InfLenArray(_) => Err("GEP has no indices".to_string()),
        }
    }

    pub fn check(&self, type_ctx: &TypeContext, allocs: &IRAllocs) -> Result<(), String> {
        let result_type = self.compute_result_type(type_ctx, allocs)?;
        if result_type != self.last_unpacked_ty {
            return Err(format!(
                "GEP result type mismatch: expected {:?}, got {:?}",
                self.last_unpacked_ty, result_type
            ));
        }
        Ok(())
    }

    pub fn base_use(&self) -> &Rc<Use> {
        &self.operands[0]
    }
    pub fn get_base(&self) -> ValueSSA {
        self.base_use().get_operand()
    }
    pub fn set_base(&self, allocs: &IRAllocs, value: ValueSSA) {
        self.base_use().set_operand(allocs, value);
    }

    pub fn index_uses(&self) -> &[Rc<Use>] {
        &self.operands[1..]
    }
    pub fn index_use_at(&self, index: usize) -> Option<&Rc<Use>> {
        self.operands.get(index + 1)
    }
    pub fn try_get_index(&self, index: usize) -> Option<ValueSSA> {
        self.index_use_at(index)
            .map(|use_ref| use_ref.get_operand())
    }
    pub fn get_index(&self, index: usize) -> ValueSSA {
        self.index_use_at(index)
            .expect("Index out of bounds")
            .get_operand()
    }
    pub fn set_index(&self, allocs: &IRAllocs, index: usize, value: ValueSSA) {
        self.index_use_at(index)
            .expect("Index out of bounds")
            .set_operand(allocs, value);
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
    ItSelf(ValTypeID),

    /// 结束状态 - 不能再索引
    Ends,
}

/// 用于遍历 GEP 指令的类型状态迭代器
///
/// 这个迭代器跟踪 GEP 指令每一步索引操作的类型变化，确保类型转换的正确性并计算最终结果类型。
pub struct GEPTypeIndexer<'a> {
    type_ctx: &'a TypeContext,
    allocs: &'a IRAllocs,
    type_state: GEPTypeState,
}

impl<'a> GEPTypeIndexer<'a> {
    pub fn new_initial(
        type_ctx: &'a TypeContext,
        allocs: &'a IRAllocs,
        base_ty: ValTypeID,
    ) -> Self {
        Self {
            type_ctx,
            allocs,
            type_state: GEPTypeState::InfLenArray(base_ty),
        }
    }

    pub fn current_state(&self) -> GEPTypeState {
        self.type_state
    }
    pub fn ends(&self) -> bool {
        matches!(self.type_state, GEPTypeState::Ends)
    }

    pub fn unpack(&mut self, idx: ValueSSA) -> GEPTypeState {
        if !matches!(idx.get_valtype(self.allocs), ValTypeID::Int(_)) {
            panic!(
                "Expected an integer index for GEP unpacking but got {:?}",
                idx.get_valtype(self.allocs).get_display_name(self.type_ctx)
            );
        }

        fn unpack_struct(type_ctx: &TypeContext, sty: StructTypeRef, idx: ValueSSA) -> ValTypeID {
            let Some(cdata) = ConstData::try_from_ir(&idx) else {
                panic!("Struct index must be a constant value but got {idx:?}");
            };
            let index = match cdata {
                ConstData::PtrNull(_) | ConstData::Zero(_) => 0,
                ConstData::Int(bits, value) => APInt::new(*value, *bits).as_signed() as isize,
                _ => {
                    panic!("Expected an integer constant for struct index but got {cdata:?}");
                }
            };
            let nfields = sty.get_nelements(type_ctx);
            if index < 0 || index >= nfields as isize {
                panic!("Struct index out of bounds: {index} for struct with {nfields} fields");
            }
            sty.get_element_type(type_ctx, index as usize).unwrap()
        }

        match self.type_state {
            GEPTypeState::Ends => GEPTypeState::Ends,
            GEPTypeState::InfLenArray(elemty) => {
                // 第一个索引是指针偏移，通常为0，完成指针解引用
                self.type_state = GEPTypeState::ItSelf(elemty);
                GEPTypeState::ItSelf(elemty)
            }
            GEPTypeState::ItSelf(to_unpack) => match to_unpack {
                ValTypeID::Array(a) => {
                    // 对数组类型的索引可以越界 -- 毕竟像 C 这样的语言是有越界处理
                    // 数组的需求的, 我们要检查也不应该在这里检查.
                    let elemty = a.get_element_type(self.type_ctx);
                    self.type_state = GEPTypeState::ItSelf(elemty);
                    GEPTypeState::ItSelf(elemty)
                }
                ValTypeID::Struct(s) => {
                    let unpacked_ty = unpack_struct(self.type_ctx, s, idx);
                    self.type_state = GEPTypeState::ItSelf(unpacked_ty);
                    GEPTypeState::ItSelf(unpacked_ty)
                }
                ValTypeID::StructAlias(sa) => {
                    let sty = sa.get_aliasee(self.type_ctx);
                    let unpacked_ty = unpack_struct(self.type_ctx, sty, idx);
                    self.type_state = GEPTypeState::ItSelf(unpacked_ty);
                    GEPTypeState::ItSelf(unpacked_ty)
                }
                ValTypeID::Int(_) | ValTypeID::Float(_) | ValTypeID::Ptr => {
                    self.type_state = GEPTypeState::Ends;
                    GEPTypeState::Ends
                }
                _ => panic!(
                    "Cannot unpack GEP type {} with index {idx:?}",
                    to_unpack.get_display_name(self.type_ctx)
                ),
            },
        }
    }
}

/// 从 GEP 指令中提取索引的迭代器
///
/// 该迭代器与 MIR 模块对接，返回每个索引值及其对应的结果类型。
///
/// ### 返回值
///
/// 每次迭代返回 `(ValueSSA, ValTypeID)` 元组：
/// - `ValueSSA`: 当前索引值
/// - `ValTypeID`: 索引操作后的结果类型（即 unpack 后的类型）
///
/// ### 示例
///
/// 对于 `getelementptr [4 x i32], ptr %p, i64 0, i64 %i`：
///
/// - 第1次迭代: `(0, [4 x i32])` - 对 `[∞ x [4 x i32]]` 索引后得到 `[4 x i32]`
/// - 第2次迭代: `(%i, i32)` - 对 `[4 x i32]` 索引后得到 `i32`
pub struct GEPIndexIter<'a> {
    indexer: GEPTypeIndexer<'a>,
    indices: &'a [Rc<Use>],
    index: usize,
}

impl<'a> GEPIndexIter<'a> {
    pub fn new(indexer: GEPTypeIndexer<'a>, indices: &'a [Rc<Use>]) -> Self {
        Self { indexer, indices, index: 0 }
    }
    pub fn current_state(&self) -> GEPTypeState {
        self.indexer.current_state()
    }
    pub fn ends(&self) -> bool {
        self.indexer.ends()
    }
}

impl<'a> Iterator for GEPIndexIter<'a> {
    type Item = (ValueSSA, ValTypeID);

    fn next(&mut self) -> Option<Self::Item> {
        if self.index >= self.indices.len() || self.indexer.ends() {
            return None;
        }
        let idx_use = &self.indices[self.index];
        let idx = idx_use.get_operand();

        // 执行索引操作并获取结果类型（unpack 后的类型）
        let result_state = self.indexer.unpack(idx);
        let result_ty = match result_state {
            GEPTypeState::ItSelf(ty) => ty,
            GEPTypeState::Ends => {
                // 即使到达 Ends 状态，也应该返回最后一个有效的类型
                // 这种情况通常发生在对基础类型进行索引时
                match self.indexer.current_state() {
                    GEPTypeState::Ends => {
                        // 如果当前状态也是 Ends，说明之前已经处理过了
                        self.index += 1;
                        return None;
                    }
                    _ => {
                        // 这里需要返回一个合适的类型，但通常不应该到达这里
                        panic!("Unexpected state transition to Ends")
                    }
                }
            }
            GEPTypeState::InfLenArray(_) => {
                panic!("GEP indexer should not return InfLenArray state after unpacking")
            }
        };

        self.index += 1;
        Some((idx, result_ty))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.indices.len() - self.index;
        (0, Some(remaining))
    }
}

fn strict_ilog2(x: usize) -> u8 {
    if x == 0 {
        panic!("Cannot compute log2 of zero");
    } else if x.is_power_of_two() {
        x.trailing_zeros() as u8
    } else {
        panic!("Value {x} is not a power of two");
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct GEPRef(InstRef);

impl ISubInstRef for GEPRef {
    type InstDataT = IndexPtr;

    fn from_raw_nocheck(inst_ref: InstRef) -> Self {
        Self(inst_ref)
    }
    fn into_raw(self) -> InstRef {
        self.0
    }
}

//! GerElemPtr Instruction.

use std::num::NonZero;

use slab::Slab;

use crate::{
    base::NullableValue,
    ir::{
        PtrStorage, PtrUser, ValueSSA, constant::data::ConstData, global::GlobalRef, inst::InstRef,
        module::Module, opcode::Opcode,
    },
    typing::{TypeMismatchError, context::TypeContext, id::ValTypeID, types::StructTypeRef},
};

use super::{
    InstDataCommon, InstDataUnique, InstError,
    checking::{
        check_operand_integral_const, check_operand_type_kind_match, check_operand_type_match,
    },
    usedef::{UseData, UseKind, UseRef},
};

#[derive(Debug)]
pub struct IndexChainNode {
    pub index: ValueSSA,
    pub unpacked_ty: ValTypeID,
}

#[derive(Debug)]
pub struct IndexPtrOp {
    pub base_ptr: UseRef,
    pub base_pointee_ty: ValTypeID,
    pub ret_pointee_ty: ValTypeID,

    pub base_pointee_align: usize,
    pub ret_pointee_align: usize,

    pub indices: Box<[UseRef]>,
}

impl PtrStorage for IndexPtrOp {
    fn get_stored_pointee_type(&self) -> ValTypeID {
        self.ret_pointee_ty
    }

    fn get_stored_pointee_align(&self) -> Option<NonZero<usize>> {
        NonZero::new(self.ret_pointee_align)
    }
}

impl PtrUser for IndexPtrOp {
    fn get_operand_pointee_type(&self) -> ValTypeID {
        self.base_pointee_ty
    }

    fn get_operand_align(&self) -> Option<NonZero<usize>> {
        NonZero::new(self.base_pointee_align)
    }
}

impl InstDataUnique for IndexPtrOp {
    /// Assume that we've known the actual levels of indexing.
    /// Without this assumption, it is impossible for us to allocate
    /// use edges for this instruction.
    fn build_operands(&mut self, common: &mut InstDataCommon, alloc_use: &mut Slab<UseData>) {
        // Set the base pointer.
        self.base_ptr = common.alloc_use(alloc_use, UseKind::GepBase);

        // Set the indices.
        for (i, index) in self.indices.iter_mut().enumerate() {
            *index = common.alloc_use(alloc_use, UseKind::GepIndex(i));
        }
    }

    /// This function traverses offset type chain to check the base pointer and indices.
    fn check_operands(&self, _: &InstDataCommon, module: &Module) -> Result<(), InstError> {
        // Check the base pointer.
        let base_ptr = self.base_ptr.get_operand(&module.borrow_use_alloc());
        check_operand_type_match(ValTypeID::Ptr, base_ptr, module)?;

        // Go throuth the offset type chain to check indices.
        let base_ty = self.base_pointee_ty;
        let final_ty = self.ret_pointee_ty;

        // Check the layer 0..N indices (0 and i..N).
        let layer_n_ty = Self::unpack_indices_get_final_type(
            module,
            &module.type_ctx,
            base_ty,
            self.indices
                .iter()
                .map(|u| u.get_operand(&module.borrow_use_alloc())),
        )?;

        // Check the return type.
        if layer_n_ty == final_ty {
            Ok(())
        } else {
            Err(InstError::OperandTypeMismatch(
                TypeMismatchError::IDNotEqual(layer_n_ty, final_ty),
                ValueSSA::None,
            ))
        }
    }
}

impl IndexPtrOp {
    fn check_struct_layer(
        type_ctx: &TypeContext,
        layer_n_ty: StructTypeRef,
        idx_value: ValueSSA,
    ) -> Result<ValTypeID, InstError> {
        let (binbits, value) = check_operand_integral_const(idx_value)?;
        let value = (value as usize) & ((1 << binbits) - 1);
        match layer_n_ty.get_element_type(type_ctx, value) {
            Some(ty) => Ok(ty),
            None => Err(InstError::OperandOverflow),
        }
    }

    /// Unpack the aggregate layers of the given type.
    /// If the type is an array, the function returns the element type regardless of the index.
    /// If the type is a struct or a struct alias, the function checks the index and returns the element type.
    fn unpack_aggregate_layers_iter(
        module: &Module,
        type_ctx: &TypeContext,
        before_unpack_ty: ValTypeID,
        n_layer_index: ValueSSA,
    ) -> Result<ValTypeID, InstError> {
        match before_unpack_ty {
            ValTypeID::Array(arrref) => {
                check_operand_type_kind_match(ValTypeID::Int(0), n_layer_index, module)?;
                Ok(arrref.get_element_type(type_ctx))
            }
            ValTypeID::Struct(sref) => Self::check_struct_layer(type_ctx, sref, n_layer_index),
            ValTypeID::StructAlias(sa) => {
                // Check struct alias: prohibits overflow and return the element type at the right
                // constant integral index.
                let sref = sa.get_aliasee(type_ctx);
                Self::check_struct_layer(type_ctx, sref, n_layer_index)
            }
            _ => {
                return Err(InstError::OperandTypeMismatch(
                    TypeMismatchError::NotAggregate(before_unpack_ty),
                    n_layer_index,
                ));
            }
        }
    }

    /// Unpack the indices and go to the final type.
    /// The first index is the base pointer regarded as an array with unknown size.
    fn unpack_indices_get_final_type(
        module: &Module,
        type_ctx: &TypeContext,
        base_ty: ValTypeID,
        mut indices_with_layer0: impl Iterator<Item = ValueSSA>,
    ) -> Result<ValTypeID, InstError> {
        // Layer 0: check the base pointer.
        match indices_with_layer0.next() {
            Some(layer0) => check_operand_type_kind_match(ValTypeID::Int(0), layer0, module)?,
            None => return Err(InstError::OperandUninit),
        };

        // Layer 1..N: check the indices.
        let mut layer_n_ty = base_ty;
        for idx_value in indices_with_layer0 {
            layer_n_ty =
                Self::unpack_aggregate_layers_iter(module, type_ctx, layer_n_ty, idx_value)?;
        }
        Ok(layer_n_ty)
    }

    pub fn dump_index_chain(&self, module: &Module) -> Vec<IndexChainNode> {
        let mut ret = Vec::with_capacity(self.indices.len());
        let alloc_use = module.borrow_use_alloc();

        assert!(
            !self.indices.is_empty(),
            "IndexPtrOp must have at least one index"
        );

        ret.push(IndexChainNode {
            index: self.indices[0].get_operand(&alloc_use),
            unpacked_ty: self.base_pointee_ty,
        });
        let mut layer_n_ty = self.base_pointee_ty;
        for index_use in self.indices[1..].iter() {
            let idx_value = index_use.get_operand(&alloc_use);
            layer_n_ty = match Self::unpack_aggregate_layers_iter(
                module,
                &module.type_ctx,
                layer_n_ty,
                idx_value,
            ) {
                Ok(ty) => ty,
                Err(_) => continue, // Skip invalid indices.
            };
            ret.push(IndexChainNode { index: idx_value, unpacked_ty: layer_n_ty });
        }
        ret
    }

    pub fn new_raw(
        mut_module: &Module,
        base_pointee_ty: ValTypeID,
        ret_pointee_ty: ValTypeID,
        base_pointee_align: usize,
        ret_pointee_align: usize,
        n_indices: usize,
    ) -> Result<(InstDataCommon, Self), InstError> {
        let mut common = InstDataCommon::new(
            Opcode::IndexPtr,
            ValTypeID::Ptr,
            &mut mut_module.borrow_use_alloc_mut(),
        );
        let mut ret = Self {
            base_ptr: UseRef::new_null(),
            base_pointee_ty,
            ret_pointee_ty,
            base_pointee_align,
            ret_pointee_align,
            indices: vec![UseRef::new_null(); n_indices].into_boxed_slice(),
        };
        ret.build_operands(&mut common, &mut mut_module.borrow_use_alloc_mut());
        Ok((common, ret))
    }

    pub fn new_from_indices(
        mut_module: &Module,
        base_pointee_ty: ValTypeID,
        base_pointee_align: usize,
        ret_pointee_align: usize,
        base_ptr: ValueSSA,
        indices: impl Iterator<Item = ValueSSA> + Clone,
    ) -> Result<(InstDataCommon, Self), InstError> {
        let ret_type = Self::unpack_indices_get_final_type(
            mut_module,
            &mut_module.type_ctx,
            base_pointee_ty,
            indices.clone(),
        )?;
        let (common, ret) = Self::new_raw(
            mut_module,
            base_pointee_ty,
            ret_type,
            base_pointee_align,
            ret_pointee_align,
            indices.clone().count(),
        )?;
        let alloc_use = mut_module.borrow_use_alloc();

        // Set the base pointer.
        ret.base_ptr.set_operand_nordfg(&alloc_use, base_ptr);

        // Set the indices.
        for (useref, idxvalue) in ret.indices.iter().zip(indices) {
            useref.set_operand_nordfg(&alloc_use, idxvalue);
        }

        Ok((common, ret))
    }
}

/// GEP 操作的偏移量类型.
#[derive(Debug, Clone)]
pub enum IrGEPOffset {
    /// 一个常量偏移量, 以字节为单位.
    Imm(i64),
    /// 函数参数 * 单位长度
    Arg(GlobalRef, u32, u64),
    /// 指令返回值 * 单位长度
    Inst(InstRef, u64),
}

/// GEP 偏移量迭代器.
pub struct IrGEPOffsetIter<'a> {
    pub indices: &'a [UseRef],
    pub index: isize,
    pub module: &'a Module,
    pub before_unpack: ValTypeID,
    pub after_unpack: ValTypeID,
    pub curr_offset: Option<IrGEPOffset>,
}

impl<'a> IrGEPOffsetIter<'a> {
    pub fn from_module(inst: &'a IndexPtrOp, module: &'a Module) -> Self {
        Self {
            indices: &inst.indices,
            index: -1,
            module,
            before_unpack: ValTypeID::Void,
            after_unpack: inst.base_pointee_ty,
            curr_offset: None,
        }
    }

    pub fn get_offset(&self) -> Option<IrGEPOffset> {
        self.curr_offset.clone()
    }

    pub fn to_vec(&mut self) -> Vec<IrGEPOffset> {
        self.collect()
    }

    /// 从当前索引值和类型中获取下一个偏移量, 并更新解包后的类型. 返回偏移量和解包后的类型.
    ///
    /// #### 基本规则
    ///
    /// - 如果索引值是函数参数或指令结果，必须是整数类型。
    /// - 如果索引值是常量数据，必须是整数类型。
    /// - 如果索引值是数组类型，返回数组元素类型。
    /// - 如果索引值是结构体类型，返回结构体元素类型。
    /// - 如果不符合上述规则，抛出 panic。
    fn head_to_next_gep_offset(
        &mut self,
        idx_value: ValueSSA,
        to_unpack: ValTypeID,
    ) -> (IrGEPOffset, ValTypeID) {
        type V = ValueSSA;
        type T = ValTypeID;
        let vallocs = self.module.borrow_value_alloc();
        let type_ctx = &self.module.type_ctx;
        match (idx_value, to_unpack) {
            (V::FuncArg(f, idx), T::Array(aty)) => {
                if !matches!(idx_value.get_value_type(&self.module), T::Int(_)) {
                    panic!("Expected an Int type for GEP index");
                }
                let weight = aty.get_elem_aligned_size(type_ctx) as u64;
                (
                    IrGEPOffset::Arg(f, idx, weight),
                    aty.get_element_type(type_ctx),
                )
            }
            (V::Inst(inst), T::Array(aty)) => {
                if !matches!(inst.get_valtype(&vallocs.alloc_inst), T::Int(_)) {
                    panic!(
                        "Expected an Int type for GEP index, found: {:?}",
                        inst.get_valtype(&vallocs.alloc_inst)
                    );
                }
                let weight = aty.get_elem_aligned_size(type_ctx) as u64;
                (
                    IrGEPOffset::Inst(inst, weight),
                    aty.get_element_type(type_ctx),
                )
            }
            (V::ConstData(data), T::Array(aty)) => {
                let index = Self::const_data_get_sint(&data);
                let weight = aty.get_elem_aligned_size(type_ctx);
                (
                    IrGEPOffset::Imm(index * weight as i64),
                    aty.get_element_type(type_ctx),
                )
            }
            (V::ConstData(data), T::Struct(sty)) => {
                let index = Self::const_data_get_sint(&data);
                let index = if index < 0 {
                    panic!("GEP index cannot be negative: {index}");
                } else {
                    index as usize
                };
                let offset = sty.offset_unwrap(type_ctx, index);
                (
                    IrGEPOffset::Imm(offset as i64),
                    sty.get_element_type(type_ctx, index).unwrap(),
                )
            }
            _ => panic!("Unsupported GEP index type: {idx_value:?} for {to_unpack:?}"),
        }
    }

    /// 解包层级 0 的索引值, 并返回偏移量和解包后的类型.
    fn unpack_level0(&mut self, idx_value: ValueSSA, unpacked: ValTypeID) -> IrGEPOffset {
        let weight = {
            let type_ctx = &self.module.type_ctx;
            let size = unpacked.get_instance_size_unwrap(type_ctx);
            let align = unpacked.get_instance_align(type_ctx).unwrap();
            size.next_multiple_of(align) as u64
        };
        match idx_value {
            ValueSSA::ConstData(data) => {
                let simm = Self::const_data_get_sint(&data);
                IrGEPOffset::Imm(simm * weight as i64)
            }
            ValueSSA::FuncArg(f, arg_id) => IrGEPOffset::Arg(f, arg_id, weight),
            ValueSSA::Inst(inst) => IrGEPOffset::Inst(inst, weight),
            _ => panic!("Expected a valid GEP index value"),
        }
    }

    fn const_data_get_sint(data: &ConstData) -> i64 {
        match data {
            ConstData::Int(64, value) => *value as i64,
            ConstData::Int(32, value) => *value as i32 as i64,
            ConstData::Zero(_) | ConstData::PtrNull(_) => 0,
            _ => panic!("Expected an integer constant data"),
        }
    }
}

impl<'a> Iterator for IrGEPOffsetIter<'a> {
    type Item = IrGEPOffset;

    fn next(&mut self) -> Option<IrGEPOffset> {
        if self.index >= self.indices.len() as isize {
            return None;
        }
        self.index += 1;
        if self.index as usize >= self.indices.len() {
            self.curr_offset = None;
            return None;
        }

        let idx_use = &self.indices[self.index as usize];
        let idx_value = idx_use.get_operand(&self.module.borrow_use_alloc());

        if self.index == 0 {
            let unpacked = self.after_unpack;
            let offset = self.unpack_level0(idx_value, unpacked);
            Some(offset)
        } else {
            let to_unpack = self.after_unpack;
            let (offset, unpacked) = self.head_to_next_gep_offset(idx_value, to_unpack);
            self.before_unpack = self.after_unpack;
            self.after_unpack = unpacked;
            self.curr_offset = Some(offset.clone());
            Some(offset)
        }
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.indices.len() - self.index as usize;
        (remaining, Some(remaining))
    }
}

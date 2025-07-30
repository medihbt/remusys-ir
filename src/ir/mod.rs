use core::panic;
use std::num::NonZero;

use block::{BlockData, BlockRef};
use constant::{
    data::{ConstData, IConstDataVisitor},
    expr::{ConstExprRef, IConstExprVisitor},
};
use global::{GlobalData, GlobalRef, IGlobalObjectVisitor, func::FuncStorage};
use inst::{InstRef, visitor::IInstVisitor};
use module::{Module, ModuleAllocatorInner};
use slab::Slab;

use crate::{
    base::{NullableValue, slabref::SlabRef},
    typing::id::ValTypeID,
};

pub mod block;
pub mod cmp_cond;
pub mod constant;
pub mod global;
pub mod graph_traits;
pub mod inst;
pub mod module;
pub mod opcode;
pub mod util;

/// Represents a value in the intermediate representation (IR).
///
/// A value can be a constant data, constant expression, function argument,
/// block, instruction, global variable, or none (representing absence of a value).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValueSSA {
    // Variants with value semantics
    // These variants are used by at most one instruction, so there is no need to track users.
    /// Represents no value or absence of a value.
    None,
    /// A constant data value with a specific type.
    ConstData(ConstData),

    // Variants with reference semantics
    // These variants may be used by multiple instructions, so their users need to be tracked.
    /// A constant expression that can be evaluated at compile time.
    ConstExpr(ConstExprRef),
    /// A function argument identified by the function reference and argument index.
    FuncArg(GlobalRef, u32),
    /// A basic block in the control flow graph.
    Block(BlockRef),
    /// An instruction that produces a value.
    Inst(InstRef),
    /// A reference to a global variable or function.
    Global(GlobalRef),
}

#[derive(Debug, Clone, Copy)]
pub enum ValueSSAError {
    IDNotEqual(ValueSSA, ValueSSA),
    KindNotMatch(ValueSSA, ValueSSA),

    NotFunction(ValueSSA),
}

impl ValueSSA {
    pub fn is_none(&self) -> bool {
        matches!(self, ValueSSA::None)
    }
    pub fn is_block(&self) -> bool {
        matches!(self, ValueSSA::Block(_))
    }
    pub fn is_const_data(&self) -> bool {
        matches!(self, ValueSSA::ConstData(_))
    }
    pub fn is_const_expr(&self) -> bool {
        matches!(self, ValueSSA::ConstExpr(_))
    }
    pub fn is_func_arg(&self) -> bool {
        matches!(self, ValueSSA::FuncArg(_, _))
    }
    pub fn is_inst(&self) -> bool {
        matches!(self, ValueSSA::Inst(_))
    }
    pub fn is_global(&self) -> bool {
        matches!(self, ValueSSA::Global(_))
    }

    pub fn is_reference_semantics(&self) -> bool {
        matches!(
            self,
            ValueSSA::ConstExpr(_)
                | ValueSSA::FuncArg(_, _)
                | ValueSSA::Block(_)
                | ValueSSA::Inst(_)
                | ValueSSA::Global(_)
        )
    }
    pub fn is_value_semantics(&self) -> bool {
        matches!(self, ValueSSA::None | ValueSSA::ConstData(_))
    }

    pub fn get_value_type(&self, module: &Module) -> ValTypeID {
        match self {
            ValueSSA::None | ValueSSA::Block(_) => ValTypeID::Void,
            ValueSSA::ConstData(data) => data.get_value_type(),
            ValueSSA::ConstExpr(expr) => expr.get_value_type(module),
            ValueSSA::Inst(inst) => module.get_inst(inst.clone()).get_value_type(),
            ValueSSA::Global(_) => ValTypeID::Ptr,
            ValueSSA::FuncArg(func_id, index) => {
                let func = module.get_global(func_id.clone());
                match &*func {
                    GlobalData::Func(func) => func
                        .get_stored_func_type()
                        .get_arg(&module.type_ctx, *index as usize)
                        .expect("Index overflow"),
                    _ => panic!("Invalid function reference"),
                }
            }
        }
    }

    pub fn binary_is_zero(&self, module: &Module) -> bool {
        match self {
            ValueSSA::ConstData(data) => data.binary_is_zero(),
            ValueSSA::ConstExpr(expr) => expr.binary_is_zero(module),
            _ => false,
        }
    }

    pub fn binary_is_zero_from_alloc(
        &self,
        alloc_expr: &Slab<constant::expr::ConstExprData>,
    ) -> bool {
        match self {
            ValueSSA::ConstData(data) => data.binary_is_zero(),
            ValueSSA::ConstExpr(expr) => expr.binary_is_zero_from_alloc(alloc_expr),
            _ => false,
        }
    }
}

/// Implementation of `NullableValue` trait for `Value` type.
/// This allows `Value` to be treated as a nullable value where `Value::None` represents null.
impl NullableValue for ValueSSA {
    /// Checks if the value is null (i.e., `Value::None`).
    ///
    /// ### Returns
    /// `true` if the value is `Value::None`, otherwise `false`.
    fn is_null(&self) -> bool {
        self.is_none()
    }

    /// Creates a new null value.
    ///
    /// ### Returns
    /// A `Value::None` representing a null value.
    fn new_null() -> Self {
        ValueSSA::None
    }
}

/// Trait for types that store pointer information.
/// Implementors of this trait can provide information about the type pointed to.
pub trait PtrStorage {
    /// Gets the type of the value being pointed to.
    ///
    /// # Returns
    /// The value type ID of the pointee type.
    fn get_stored_pointee_type(&self) -> ValTypeID;

    /// Gets the align of the value begin pointed to.
    fn get_stored_pointee_align(&self) -> Option<NonZero<usize>>;
}

/// Trait for types that use pointers as operands.
/// Implementors of this trait can retrieve type information about the pointee.
pub trait PtrUser {
    /// Gets the type of the value pointed to by an operand.
    ///
    /// # Returns
    /// The value type ID of the pointee.
    fn get_operand_pointee_type(&self) -> ValTypeID;

    /// Gets the align of this value user.
    fn get_operand_align(&self) -> Option<NonZero<usize>>;
}

pub trait IValueVisitor:
    IConstDataVisitor + IConstExprVisitor + IGlobalObjectVisitor + IInstVisitor
{
    fn read_block(&self, block: BlockRef, block_data: &BlockData);
    fn read_func_arg(&self, func: GlobalRef, index: u32);

    fn value_visitor_diapatch(&self, value: ValueSSA, alloc_value: &ModuleAllocatorInner) {
        let alloc_block = &alloc_value.alloc_block;
        let alloc_global = &alloc_value.alloc_global;
        let alloc_inst = &alloc_value.alloc_inst;
        let alloc_expr = &alloc_value.alloc_expr;
        match value {
            ValueSSA::None => {}
            ValueSSA::ConstData(data) => self.const_data_visitor_dispatch(&data),
            ValueSSA::FuncArg(func, index) => self.read_func_arg(func, index),
            ValueSSA::Block(bb) => self.read_block(bb, bb.to_data(alloc_block)),
            ValueSSA::ConstExpr(expr) => self.expr_visitor_dispatch(expr, alloc_expr),
            ValueSSA::Inst(inst_ref) => {
                self.inst_visitor_dispatch(inst_ref, inst_ref.to_data(alloc_inst))
            }
            ValueSSA::Global(global_ref) => {
                self.global_object_visitor_dispatch(global_ref, alloc_global)
            }
        }
    }
}

pub trait ISubValueSSA {
    fn try_from_ir(ir: &ValueSSA) -> Option<&Self>;

    fn from_ir(ir: &ValueSSA) -> &Self {
        match Self::try_from_ir(ir) {
            Some(x) => x,
            None => panic!("cannot cast {ir:?} to self"),
        }
    }

    fn into_ir(self) -> ValueSSA;
}

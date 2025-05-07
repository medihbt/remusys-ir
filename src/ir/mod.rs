use std::num::NonZero;

use block::BlockRef;
use constant::{data::ConstData, expr::ConstExprRef};
use global::{GlobalData, GlobalRef, func::FuncStorage};
use inst::InstRef;
use module::Module;

use crate::{base::NullableValue, typing::id::ValTypeID};

pub mod block;
pub mod constant;
pub mod global;
pub mod inst;
pub mod module;
pub mod opcode;
pub mod util;

/// Represents a value in the intermediate representation (IR).
/// 
/// A value can be a constant data, constant expression, function argument,
/// block, instruction, global variable, or none (representing absence of a value).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ValueSSA {
    /// Represents no value or absence of a value.
    None,
    /// A constant data value with a specific type.
    ConstData(ConstData),
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

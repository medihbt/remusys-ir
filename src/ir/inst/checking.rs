use crate::{
    base::NullableValue,
    ir::{ValueSSA, module::Module},
    typing::{TypeMismatchError, id::ValTypeID},
};

use super::InstError;

pub(super) fn check_type_match(
    required: ValTypeID,
    current: ValTypeID,
) -> Result<(), TypeMismatchError> {
    if required != current {
        Err(TypeMismatchError::IDNotEqual(required, current))
    } else {
        Ok(())
    }
}

pub(super) fn check_type_kind_match(
    required: ValTypeID,
    current: ValTypeID,
) -> Result<(), TypeMismatchError> {
    match (required, current) {
        (ValTypeID::Void, ValTypeID::Void)
        | (ValTypeID::Ptr, ValTypeID::Ptr)
        | (ValTypeID::Int(..), ValTypeID::Int(..))
        | (ValTypeID::Float(..), ValTypeID::Float(..))
        | (ValTypeID::Array(..), ValTypeID::Array(..))
        | (ValTypeID::Struct(..), ValTypeID::Struct(..))
        | (ValTypeID::StructAlias(..), ValTypeID::StructAlias(..))
        | (ValTypeID::Func(..), ValTypeID::Func(..)) => Ok(()),
        _ => Err(TypeMismatchError::KindNotMatch(required, current)),
    }
}

pub(super) fn check_operand_type_match(
    required: ValTypeID,
    current: ValueSSA,
    module: &Module,
) -> Result<(), InstError> {
    if current.is_nonnull() {
        check_type_match(required, current.get_value_type(module))
            .map_err(|e| InstError::OperandTypeMismatch(e, current))
    } else {
        Ok(())
    }
}

pub(super) fn check_operand_type_kind_match(
    required: ValTypeID,
    current: ValueSSA,
    module: &Module,
) -> Result<(), InstError> {
    if current.is_nonnull() {
        check_type_kind_match(required, current.get_value_type(module))
            .map_err(|e| InstError::OperandTypeMismatch(e, current))
    } else {
        Ok(())
    }
}

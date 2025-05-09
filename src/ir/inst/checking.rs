use crate::{
    base::NullableValue,
    ir::{ValueSSA, constant::data::ConstData, module::Module},
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
    if std::mem::discriminant(&required) == std::mem::discriminant(&current) {
        Ok(())
    } else {
        Err(TypeMismatchError::KindNotMatch(required, current))
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

pub(super) fn check_operand_integral_const(current: ValueSSA) -> Result<(u8, i128), InstError> {
    match current {
        ValueSSA::ConstData(ConstData::Int(binbits, value)) => Ok((binbits, value)),
        ValueSSA::ConstData(ConstData::Zero(ValTypeID::Int(binbits))) => Ok((binbits, 0)),
        ValueSSA::ConstData(x) => {
            Err(InstError::OperandTypeMismatch(
                TypeMismatchError::KindNotMatch(ValTypeID::Int(0), x.get_value_type()),
                current,
            ))
        },
        _ => Err(InstError::OperandNotComptimeConst(current)),
    }
}

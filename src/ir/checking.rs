use crate::{
    ir::{IRAllocs, ISubValueSSA, ValueSSA},
    typing::{ValTypeClass, ValTypeID},
};

#[derive(Debug, Clone)]
pub enum ValueCheckError {
    TypeMismatch(ValTypeID, ValTypeID),
    TypeNotClass(ValTypeID, ValTypeClass),
    InvalidValue(ValueSSA),
}

pub(super) fn type_matches(
    ty: ValTypeID,
    val: ValueSSA,
    allocs: &IRAllocs,
) -> Result<(), ValueCheckError> {
    let valty: ValTypeID = val.get_valtype(allocs);
    if valty != ty { Err(ValueCheckError::TypeMismatch(ty, valty)) } else { Ok(()) }
}

use super::id::ValTypeID;

#[derive(Debug, Clone, Copy, PartialEq)]
pub struct IntType {
    pub bin_bits: u8,
}

#[derive(Debug, Clone, Copy, PartialEq)]
pub enum FloatTypeKind {
    Ieee32,
    Ieee64,
}

impl ToString for FloatTypeKind {
    fn to_string(&self) -> String {
        match self {
            FloatTypeKind::Ieee32 => "float",
            FloatTypeKind::Ieee64 => "double",
        }.to_string()
    }
}
impl FloatTypeKind {
    pub fn size(&self) -> usize {
        match self {
            FloatTypeKind::Ieee32 => 4,
            FloatTypeKind::Ieee64 => 8,
        }
    }

    pub const NELEMS: usize = 2;
}

#[derive(Debug, Clone)]
pub struct ArrayType {
    pub elem_ty: ValTypeID,
    pub length:  usize,
}

pub type StructType = Vec<ValTypeID>;

#[derive(Debug, Clone)]
pub struct StructAliasType {
    pub name:    String,
    pub aliasee: ValTypeID,
}

#[derive(Debug, Clone)]
pub struct FuncType {
    pub args:   Vec<ValTypeID>,
    pub ret_ty: ValTypeID
}
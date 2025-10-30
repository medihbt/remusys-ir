use std::io::Write;

mod alias;
mod array;
mod compound;
mod context;
mod fmt;
mod func;
mod prim;
mod structty;
mod vec;

pub use self::{
    alias::{StructAliasID, StructAliasObj},
    array::{ArrayTypeID, ArrayTypeObj},
    compound::{AggrType, ScalarType},
    context::{ArchInfo, TypeAllocs, TypeContext},
    fmt::TypeFormatter,
    func::{FuncTypeID, FuncTypeObj},
    prim::{FPKind, IntType, PtrType},
    structty::{StructTypeID, StructTypeObj},
    vec::FixVecType,
};

#[derive(Debug, Clone, Copy)]
pub enum TypeMismatchError {
    IDNotEqual(ValTypeID, ValTypeID),
    LayoutNotEqual(ValTypeID, ValTypeID),
    KindNotMatch(ValTypeID, ValTypeID),
    NotClass(ValTypeID, ValTypeClass),

    NotAggregate(ValTypeID),
    NotPrimitive(ValTypeID),
}

pub type TypingRes<T = ()> = Result<T, TypeMismatchError>;

pub trait IValType: Copy {
    fn try_from_ir(ty: ValTypeID) -> TypingRes<Self>;

    fn from_ir(ty: ValTypeID) -> Self {
        match Self::try_from_ir(ty) {
            Ok(val) => val,
            Err(err) => {
                let thisname = std::any::type_name::<Self>();
                panic!("Failed to convert {ty:?} to {thisname:?}: {err:?}")
            }
        }
    }

    fn into_ir(self) -> ValTypeID;

    fn makes_instance(self) -> bool;

    /// 这个类型的 class ID
    fn class_id(self) -> ValTypeClass;

    /// 序列化
    fn serialize<T: Write>(self, f: &TypeFormatter<T>) -> std::io::Result<()>;
    fn get_display_name(self, tctx: &TypeContext) -> String {
        let mut buffer = Vec::new();
        let formatter = TypeFormatter::new(&mut buffer, tctx);
        self.serialize(&formatter)
            .expect("Serialization to Vec<u8> should not fail");
        drop(formatter);
        String::from_utf8(buffer).expect("Type names should be valid UTF-8")
    }
    fn println(&self, tctx: &TypeContext) {
        let name = self.get_display_name(tctx);
        println!("{name}");
    }

    fn try_get_size_full(self, alloc: &TypeAllocs, tctx: &TypeContext) -> Option<usize>;
    fn try_get_size(self, tctx: &TypeContext) -> Option<usize> {
        self.try_get_size_full(&tctx.allocs.borrow(), tctx)
    }
    fn get_size(self, tctx: &TypeContext) -> usize {
        self.try_get_size(tctx).expect("Failed to get size of type")
    }

    fn try_get_align_full(self, alloc: &TypeAllocs, tctx: &TypeContext) -> Option<usize>;
    fn try_get_align(self, tctx: &TypeContext) -> Option<usize> {
        self.try_get_align_full(&tctx.allocs.borrow(), tctx)
    }
    fn get_align(self, tctx: &TypeContext) -> usize {
        self.try_get_align(tctx)
            .expect("Failed to get align of type")
    }
    fn get_align_log2(self, tctx: &TypeContext) -> u8 {
        self.get_align(tctx).ilog2() as u8
    }

    fn try_get_aligned_size_full(self, alloc: &TypeAllocs, tctx: &TypeContext) -> Option<usize> {
        let size = self.try_get_size_full(alloc, tctx)?;
        let align = self.try_get_align_full(alloc, tctx)?;
        Some(size.next_multiple_of(align))
    }
    fn try_get_aligned_size(self, tctx: &TypeContext) -> Option<usize> {
        self.try_get_aligned_size_full(&tctx.allocs.borrow(), tctx)
    }
    fn get_aligned_size(self, tctx: &TypeContext) -> usize {
        self.try_get_aligned_size(tctx)
            .expect("Failed to get aligned size of type")
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValTypeClass {
    Void,
    Ptr,
    Int,
    Float,
    FixVec,
    Array,
    Struct,
    StructAlias,
    Func,
    Compound,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ValTypeID {
    /// Uninhabited type
    Void,

    /// Opaque pointer type (without pointee type)
    Ptr,

    /// Binary Bits: 1..128
    Int(u8),

    /// Floating Type
    Float(FPKind),

    /// Fixed Vector Type
    FixVec(FixVecType),

    /// Array Type
    Array(ArrayTypeID),

    /// Unnamed Structure Type
    Struct(StructTypeID),

    /// Struct Alias Type
    StructAlias(StructAliasID),

    /// Function type
    Func(FuncTypeID),
}

impl Default for ValTypeID {
    fn default() -> Self {
        ValTypeID::Void
    }
}

impl IValType for ValTypeID {
    fn try_from_ir(ty: ValTypeID) -> TypingRes<Self> {
        Ok(ty)
    }

    fn into_ir(self) -> ValTypeID {
        self
    }

    fn makes_instance(self) -> bool {
        !matches!(self, ValTypeID::Void | ValTypeID::Func(_))
    }

    fn class_id(self) -> ValTypeClass {
        match self {
            ValTypeID::Void => ValTypeClass::Void,
            ValTypeID::Ptr => ValTypeClass::Ptr,
            ValTypeID::Int(_) => ValTypeClass::Int,
            ValTypeID::Float(_) => ValTypeClass::Float,
            ValTypeID::FixVec(_) => ValTypeClass::FixVec,
            ValTypeID::Array(_) => ValTypeClass::Array,
            ValTypeID::Struct(_) => ValTypeClass::Struct,
            ValTypeID::StructAlias(_) => ValTypeClass::StructAlias,
            ValTypeID::Func(_) => ValTypeClass::Func,
        }
    }

    fn try_get_size_full(self, alloc: &TypeAllocs, tctx: &TypeContext) -> Option<usize> {
        match self {
            ValTypeID::Void | ValTypeID::Func(_) => None,
            ValTypeID::Ptr => PtrType.try_get_size_full(alloc, tctx),
            ValTypeID::Int(bits) => IntType(bits).try_get_size_full(alloc, tctx),
            ValTypeID::Float(fpkind) => fpkind.try_get_size_full(alloc, tctx),
            ValTypeID::FixVec(fixvec) => fixvec.try_get_size_full(alloc, tctx),
            ValTypeID::Array(arr) => arr.try_get_size_full(alloc, tctx),
            ValTypeID::Struct(s) => s.try_get_size_full(alloc, tctx),
            ValTypeID::StructAlias(a) => a.try_get_size_full(alloc, tctx),
        }
    }

    fn try_get_align_full(self, alloc: &TypeAllocs, tctx: &TypeContext) -> Option<usize> {
        match self {
            ValTypeID::Void | ValTypeID::Func(_) => None,
            ValTypeID::Ptr => PtrType.try_get_align_full(alloc, tctx),
            ValTypeID::Int(bits) => IntType(bits).try_get_align_full(alloc, tctx),
            ValTypeID::Float(fpkind) => fpkind.try_get_align_full(alloc, tctx),
            ValTypeID::FixVec(fixvec) => fixvec.try_get_align_full(alloc, tctx),
            ValTypeID::Array(arr) => arr.try_get_align_full(alloc, tctx),
            ValTypeID::Struct(s) => s.try_get_align_full(alloc, tctx),
            ValTypeID::StructAlias(a) => a.try_get_align_full(alloc, tctx),
        }
    }

    fn serialize<T: std::io::Write>(self, f: &TypeFormatter<T>) -> std::io::Result<()> {
        match self {
            ValTypeID::Void => f.write_str("void"),
            ValTypeID::Ptr => f.write_str("ptr"),
            ValTypeID::Int(bits) => write!(f, "i{}", bits),
            ValTypeID::Float(fpkind) => fpkind.serialize(f),
            ValTypeID::FixVec(fixvec) => fixvec.serialize(f),
            ValTypeID::Array(a) => a.serialize(f),
            ValTypeID::Struct(s) => s.serialize(f),
            ValTypeID::StructAlias(sa) => sa.serialize(f),
            ValTypeID::Func(func) => func.serialize(f),
        }
    }
}

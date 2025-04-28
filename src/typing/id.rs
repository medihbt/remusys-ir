use std::rc::{Rc, Weak};

use super::{context::{TypeContext, TypeContextInner}, subtypes::*};

#[derive(Debug, Clone)]
pub enum ValTypeUnion {
    None, Void, Ptr,
    Int   (IntType),
    Float (FloatTypeKind),
    Array (ArrayType),
    Struct(StructType),
    StructAlias(StructAliasType),
    Func  (FuncType),
}

impl ValTypeUnion {
    pub fn to_nonnull<'a>(&'a self) -> Option<&'a ValTypeUnion>
    {
        match self {
            ValTypeUnion::None => None,
            _ => Some(self),
        }
    }
    pub fn from_option(opt: Option<ValTypeUnion>) -> ValTypeUnion
    {
        match opt {
            Some(vty) => vty.clone(),
            None => ValTypeUnion::None,
        }
    }

    pub fn is_void(&self) -> bool { matches!(self, ValTypeUnion::Void) }
    pub fn is_ptr (&self) -> bool { matches!(self, ValTypeUnion::Ptr)  }
    pub fn is_int (&self) -> bool { matches!(self, ValTypeUnion::Int(_)) }
    pub fn is_float(&self)  -> bool { matches!(self, ValTypeUnion::Float(_)) }
    pub fn is_array(&self)  -> bool { matches!(self, ValTypeUnion::Array(_)) }
    pub fn is_struct(&self) -> bool { matches!(self, ValTypeUnion::Struct(_)) }
    pub fn is_func(&self)   -> bool { matches!(self, ValTypeUnion::Func(_)) }
    pub fn is_struct_alias(&self) -> bool { matches!(self, ValTypeUnion::StructAlias(_)) }
    pub fn is_primitive(&self) -> bool {
        matches!(self, ValTypeUnion::Int(_) | ValTypeUnion::Float(_))
    }

    pub fn deep_eq(&self, other: &ValTypeUnion) -> bool
    {
        if std::ptr::eq(self, other) {
            return true;
        }
        // For types with "element" type ID, we use '==' instead of 'eq_with_typectx'
        // to avoid infinite recursion.
        match (self, other) {
            (ValTypeUnion::Int(i1),         ValTypeUnion::Int(i2)) => i1.bin_bits == i2.bin_bits,
            (ValTypeUnion::Float(f1), ValTypeUnion::Float(f2)) => f1 == f2,
            (ValTypeUnion::Array(a1),     ValTypeUnion::Array(a2)) => {
                a1.length == a2.length && a1.elem_ty == a2.elem_ty
            },
            (ValTypeUnion::Struct(s1), ValTypeUnion::Struct(s2)) => {
                if s1.len() != s2.len() { return false; }
                for (l, r) in s1.iter().zip(s2.iter()) {
                    if l != r {
                        return false;
                    }
                }
                true
            },
            (ValTypeUnion::StructAlias(sa1), ValTypeUnion::StructAlias(sa2)) => {
                sa1.name == sa2.name && sa1.aliasee == sa2.aliasee
            },
            _ => false,
        }
    }
}

#[derive(Debug)]
pub enum ValTypeErr {
    NullException, TypeIDNotValid (usize), InvalidContext,
    Unmatch { requried: ValTypeID, curr: ValTypeID },
    NotInstanciated (ValTypeID),
}

impl Default for ValTypeUnion {
    fn default() -> Self { Self::None }
}

#[derive(Debug, Clone)]
pub struct ValTypeID (
    pub(super) usize,
    pub(super) Weak<TypeContext>
);

impl ValTypeID {
    pub fn handle (&self)     -> usize { self.0 }
    pub fn typectx(&self)     -> Weak<TypeContext> { self.1.clone() }
    pub fn own_typectx(&self) -> Option<Rc<TypeContext>> { self.1.upgrade() }

    pub fn get_instance_size(&self) -> Result<usize, ValTypeErr>
    {
        let type_ctx_ref = self.own_typectx().ok_or(ValTypeErr::InvalidContext)?;
        let type_ctx = type_ctx_ref.inner.borrow();
        self.get_instance_size_with_tctx(&type_ctx)
    }
    fn get_instance_size_with_tctx(&self, type_ctx: &TypeContextInner) -> Result<usize, ValTypeErr>
    {
        let vty = type_ctx.find_type(self).ok_or(ValTypeErr::NullException)?;
        
        match vty {
            ValTypeUnion::None => Err(ValTypeErr::NullException),
            ValTypeUnion::Void => Err(ValTypeErr::NotInstanciated(self.clone())),
            ValTypeUnion::Ptr  => Ok(type_ctx.intptr_size()),
            ValTypeUnion::Int   (i)       => Ok(i.bin_bits as usize / 8),
            ValTypeUnion::Float (f) => Ok(f.size()),
            ValTypeUnion::Array (a)     => {
                let elem_size = a.elem_ty.get_instance_size_with_tctx(type_ctx)?;
                Ok(elem_size * a.length)
            },
            ValTypeUnion::Struct(s) => {
                let mut size = 0;
                for elem in s {
                    size += elem.get_instance_size_with_tctx(type_ctx)?;
                }
                Ok(size)
            },
            ValTypeUnion::StructAlias(sa) => {
                let aliasee = type_ctx.find_type(&sa.aliasee).ok_or(ValTypeErr::NullException)?;
                match aliasee {
                    ValTypeUnion::Struct(s) => {
                        let mut size = 0;
                        for elem in s {
                            size += elem.get_instance_size_with_tctx(type_ctx)?;
                        }
                        Ok(size)
                    },
                    _ => Err(ValTypeErr::Unmatch { requried: self.clone(), curr: sa.aliasee.clone() }),
                }
            },
            ValTypeUnion::Func(_) => Err(ValTypeErr::NotInstanciated(self.clone())),
        }
    }

    pub fn to_string_with_type_ctx(&self, type_ctx: &TypeContextInner) -> String
    {
        let vty = type_ctx.find_type(self).unwrap();
        match vty {
            ValTypeUnion::None => "none".to_string(),
            ValTypeUnion::Void => "void".to_string(),
            ValTypeUnion::Ptr  => "ptr".to_string(),
            ValTypeUnion::Int   (i)       => format!("i{}", i.bin_bits),
            ValTypeUnion::Float (f) => f.to_string(),
            ValTypeUnion::Array (a)     => {
                format!("[{} x {}]",
                    a.elem_ty.to_string_with_type_ctx(type_ctx),
                    a.length)
            },
            ValTypeUnion::Struct(s) => {
                let mut str = String::new();
                for elem in s {
                    str.push_str(&elem.to_string_with_type_ctx(type_ctx));
                    str.push(',');
                }
                format!("{{{}}}", str)
            },
            ValTypeUnion::StructAlias(sa) => format!("%{}", sa.name),
            ValTypeUnion::Func(f) => {
                let mut s = f.ret_ty.to_string_with_type_ctx(type_ctx);
                s.push('(');
                let mut idx = 0;
                for arg in &f.args {
                    if idx > 0 {
                        s.push(',');
                    }
                    s.push_str(&arg.to_string_with_type_ctx(type_ctx));
                    idx += 1;
                }
                s.push(')');
                s
            },
        }
    }

    /// Compare two ValTypeID with the type context.
    /// This is a deep comparison, which means that it will compare the types of the
    /// ValTypeID, not just the IDs.
    pub fn deep_eq(&self, other: &ValTypeID) -> bool
    {
        if self == other {
            return true;
        }
        let type_ctx = self.own_typectx().expect("Type context is not valid");
        let type_ctx = type_ctx.borrow();
        self.deep_eq_with_typectx(other, &type_ctx)
    }
    /// Compare two ValTypeID with the type context.
    pub fn deep_eq_with_typectx(&self, other: &ValTypeID, type_ctx: &TypeContextInner) -> bool
    {
        if std::ptr::eq(self, other) {
            return true;
        }
        if self.0 != other.0 {
            return false;
        }
        let vty1 = type_ctx.find_type(self).unwrap();
        let vty2 = type_ctx.find_type(other).unwrap();
        vty1.deep_eq(vty2)
    }
}

impl ToString for ValTypeID {
    fn to_string(&self) -> String {
        let type_ctx = self.own_typectx().expect("Type context is not valid");
        let type_ctx = type_ctx.borrow();
        self.to_string_with_type_ctx(&type_ctx)
    }
}

impl PartialEq for ValTypeID {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && Weak::ptr_eq(&self.1, &other.1)
    }
}

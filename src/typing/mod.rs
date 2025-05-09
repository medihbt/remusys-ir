use context::TypeContext;
use id::ValTypeID;

pub mod context;
pub mod id;
pub mod types;

pub trait IValType {
    fn get_instance_size(&self, type_ctx: &TypeContext) -> Option<usize>;
    fn makes_instance(&self) -> bool;

    fn get_display_name(&self, type_ctx: &TypeContext) -> String;

    fn deep_eq(&self, rhs: &Self) -> bool;

    fn gc_trace(&self, gather_func: impl Fn(ValTypeID));
}

#[derive(Debug)]
pub enum TypeMismatchError {
    IDNotEqual(ValTypeID, ValTypeID),
    LayoutNotEqual(ValTypeID, ValTypeID),
    KindNotMatch(ValTypeID, ValTypeID),
}

#[cfg(test)]
mod testing {
    use super::{context::{binary_bits_to_bytes, PlatformPolicy, TypeContext}, id::ValTypeID, types::FloatTypeKind};

    
    #[test]
    fn test_void_ptr() {
        let type_ctx = TypeContext::new(PlatformPolicy::new_host());
        let voidty = ValTypeID::Void;
        let ptrty  = ValTypeID::Ptr;

        assert_eq!(voidty.get_display_name(&type_ctx), "void");
        assert_eq!(ptrty.get_display_name(&type_ctx),  "ptr");
    }

    #[test]
    fn test_primitive_types() {
        let type_ctx = TypeContext::new(PlatformPolicy::new_host());
        for i in 1..u8::MAX {
            let inty = ValTypeID::Int(i);

            assert_eq!(inty.get_instance_size(&type_ctx), Some(binary_bits_to_bytes(i as usize)));
            print!("{}, ", inty.get_display_name(&type_ctx));
        }

        assert_eq!(ValTypeID::Int(1), ValTypeID::new_boolean());

        let f32ty = ValTypeID::Float(FloatTypeKind::Ieee32);
        let f64ty = ValTypeID::Float(FloatTypeKind::Ieee64);

        assert_eq!(f32ty.get_display_name(&type_ctx), "float");
        assert_eq!(f64ty.get_display_name(&type_ctx), "double");
    }
}
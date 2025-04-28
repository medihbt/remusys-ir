pub mod subtypes;
pub mod context;
pub mod id;

mod testing {
    #[allow(unused_imports)]
    use super::{context::TypeContext, id::ValTypeUnion, subtypes::ArrayType};

    #[test]
    fn test_type_context() {
        let context = TypeContext::new();
        let i32ty = context.get_int_type(32);
        let i32ty2 = context.get_int_type(32);
        let ai32n4ty = context.reg_get_type(
            ValTypeUnion::Array(ArrayType {
                elem_ty: i32ty.clone(),
                length:  4,
            })
        );
        let ai32n4ty2 = context.reg_get_type(
            ValTypeUnion::Array(ArrayType {
                elem_ty: i32ty.clone(),
                length:  4,
            })
        );
        assert_eq!(i32ty, i32ty2);
        assert_eq!(ai32n4ty.to_string(), "[i32 x 4]");
        assert_eq!(ai32n4ty, ai32n4ty2);
    }

    #[test]
    fn test_int_types() {
        let context = TypeContext::new();
        let boolty = context.get_int_type(1);
        let i8ty  = context.get_int_type(8);
        let i16ty = context.get_int_type(16);
        let i32ty = context.get_int_type(32);
        let i64ty = context.get_int_type(64);
        assert_eq!(boolty.to_string(), "i1");
        assert_eq!(i8ty.to_string(),   "i8");
        assert_eq!(i16ty.to_string(), "i16");
        assert_eq!(i32ty.to_string(), "i32");
        assert_eq!(i64ty.to_string(), "i64");
    }
}

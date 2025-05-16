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

#[derive(Debug, Clone, Copy)]
pub enum TypeMismatchError {
    IDNotEqual(ValTypeID, ValTypeID),
    LayoutNotEqual(ValTypeID, ValTypeID),
    KindNotMatch(ValTypeID, ValTypeID),

    NotAggregate(ValTypeID),
    NotPrimitive(ValTypeID),
}

#[cfg(test)]
mod testing {
    use super::{
        context::{PlatformPolicy, TypeContext, binary_bits_to_bytes},
        id::ValTypeID,
        types::FloatTypeKind,
    };

    #[test]
    fn test_void_ptr() {
        let type_ctx = TypeContext::new(PlatformPolicy::new_host());
        let voidty = ValTypeID::Void;
        let ptrty = ValTypeID::Ptr;

        assert_eq!(voidty.get_display_name(&type_ctx), "void");
        assert_eq!(ptrty.get_display_name(&type_ctx), "ptr");
    }

    #[test]
    fn test_primitive_types() {
        let type_ctx = TypeContext::new(PlatformPolicy::new_host());
        for i in 1..u8::MAX {
            let inty = ValTypeID::Int(i);

            assert_eq!(
                inty.get_instance_size(&type_ctx),
                Some(binary_bits_to_bytes(i as usize))
            );
            print!("{}, ", inty.get_display_name(&type_ctx));
        }

        assert_eq!(ValTypeID::Int(1), ValTypeID::new_boolean());

        let f32ty = ValTypeID::Float(FloatTypeKind::Ieee32);
        let f64ty = ValTypeID::Float(FloatTypeKind::Ieee64);

        assert_eq!(f32ty.get_display_name(&type_ctx), "float");
        assert_eq!(f64ty.get_display_name(&type_ctx), "double");
    }

    #[test]
    fn test_array_type() {
        let type_ctx = TypeContext::new(PlatformPolicy::new_host());
        // Array type `[8 x i32]`
        let arrty = type_ctx.make_array_type(8, ValTypeID::Int(32));
        // Array type `[8 x i32]`
        let arrty2 = type_ctx.make_array_type(8, ValTypeID::Int(32));
        // Array type `[8 x i64]`
        let arrty3 = type_ctx.make_array_type(8, ValTypeID::Int(64));

        assert_eq!(
            ValTypeID::Array(arrty).get_display_name(&type_ctx),
            "[8 x i32]"
        );
        assert_eq!(arrty, arrty2);
        assert_eq!(
            ValTypeID::Array(arrty3).get_display_name(&type_ctx),
            "[8 x i64]"
        );
    }

    #[test]
    fn test_struct_and_alias() {
        /* Source code:
           public struct Student {
               public age: int;
               public name: byte[16];
           }
        */
        let type_ctx = TypeContext::new(PlatformPolicy::new_host());
        let student_struct = {
            let name_type = type_ctx.make_array_type(16, ValTypeID::Int(8));
            let age_type = ValTypeID::Int(32);
            type_ctx.make_struct_type(&[age_type, ValTypeID::Array(name_type)])
        };
        let student_struct_alias =
            type_ctx.make_struct_alias_lazy("Student".into(), student_struct);

        assert_eq!(student_struct_alias.get_name(&type_ctx), "Student");
        assert_eq!(
            ValTypeID::StructAlias(student_struct_alias).get_display_name(&type_ctx),
            "%Student"
        );
        assert_eq!(
            ValTypeID::Struct(student_struct).get_display_name(&type_ctx),
            "{i32, [16 x i8]}"
        );
        assert_eq!(student_struct_alias.get_aliasee(&type_ctx), student_struct);
    }

    #[test]
    fn test_func_type() {
        /* source code:
           public extern func strlen(string: byte*): int;
           public extern func foo();
           public extern func printf(format: byte*, ...);
        */
        let type_ctx = TypeContext::new(PlatformPolicy::new_host());
        let strlen_functype = type_ctx.make_func_type(&[ValTypeID::Ptr], ValTypeID::Int(32), false);
        let foo_functype = type_ctx.make_func_type(&[], ValTypeID::Void, false);
        let printf_functype = type_ctx.make_func_type(
            &[ValTypeID::Ptr],
            ValTypeID::Int(32),
            true,
        );

        assert_eq!(
            ValTypeID::Func(strlen_functype).get_display_name(&type_ctx),
            "fn<(ptr):i32>"
        );
        assert_eq!(
            ValTypeID::Func(foo_functype).get_display_name(&type_ctx),
            "fn<():void>"
        );
        assert_eq!(
            ValTypeID::Func(printf_functype).get_display_name(&type_ctx),
            "fn<(ptr, ...):i32>"
        );
    }
}

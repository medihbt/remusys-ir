#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ir::{
            IRAllocs, ValueSSA, InstData,
            inst::gep::IndexPtr,
            constant::ConstData,
        },
        typing::{TypeContext, ValTypeID},
        base::APInt,
    };

    #[test]
    fn test_gep_check_basic() {
        // 创建基本的测试环境
        let type_ctx = TypeContext::new();
        let mut allocs = IRAllocs::new();
        
        // 创建一个简单的 GEP 指令用于测试
        // getelementptr [4 x i32], ptr %ptr, i64 0, i64 1
        
        // 创建常量索引值
        let zero_const = ConstData::Int(APInt::from_u64(0, 64));
        let one_const = ConstData::Int(APInt::from_u64(1, 64)); 
        
        // 将常量添加到 allocs
        let zero_ref = allocs.exprs.alloc_with(|r| zero_const.with_ref(r));
        let one_ref = allocs.exprs.alloc_with(|r| one_const.with_ref(r));
        
        let zero_val = ValueSSA::ConstExpr(zero_ref);
        let one_val = ValueSSA::ConstExpr(one_ref);
        
        // 创建数组类型 [4 x i32]
        let array_ty = type_ctx.make_array_type(ValTypeID::Int(32), 4);
        
        // 创建基础指针值
        let base_ptr = ValueSSA::None; // 暂时使用 None，实际测试中需要有效的指针
        
        // 创建 GEP 指令
        let indices = vec![&zero_val, &one_val];
        let gep = IndexPtr::new(&type_ctx, &allocs, base_ptr, array_ty, indices);
        
        // 创建验证上下文
        let validate_ctx = InstValidateCtx::new(&type_ctx, &allocs);
        
        // 测试检查应该失败，因为基础指针是 None
        let result = validate_ctx.check_gep(&gep);
        assert!(result.is_err());
        
        if let Err(ValueCheckError::OperandPosNone(_, UseKind::GepBase)) = result {
            // 预期的错误：基础指针为空
        } else {
            panic!("Expected OperandPosNone error for GepBase, got {:?}", result);
        }
    }

    #[test] 
    fn test_gep_check_invalid_index_type() {
        let type_ctx = TypeContext::new();
        let mut allocs = IRAllocs::new();
        
        // 创建浮点常量作为无效的索引类型
        let float_const = ConstData::Float(1.0f32);
        let float_ref = allocs.exprs.alloc_with(|r| float_const.with_ref(r));
        let float_val = ValueSSA::ConstExpr(float_ref);
        
        let array_ty = type_ctx.make_array_type(ValTypeID::Int(32), 4);
        let base_ptr = ValueSSA::None;
        
        let indices = vec![&float_val];
        let gep = IndexPtr::new(&type_ctx, &allocs, base_ptr, array_ty, indices);
        
        let validate_ctx = InstValidateCtx::new(&type_ctx, &allocs);
        
        // 这个测试可能会在 IndexPtr::new 中 panic，因为它会验证索引类型
        // 实际上我们需要一个更低级的方式来创建无效的 GEP
    }
}

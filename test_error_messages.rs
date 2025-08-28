// 测试错误消息格式的临时文件
use remusys_ir::ir::checking::ValueCheckError;
use remusys_ir::typing::{ValTypeID, ValTypeClass};

fn main() {
    // 测试基本的类型错误
    let type_error = ValueCheckError::TypeMismatch(ValTypeID::Int(32), ValTypeID::Float(remusys_ir::typing::FPKind::Ieee32));
    println!("TypeMismatch: {}", type_error);

    let class_error = ValueCheckError::TypeNotClass(ValTypeID::Int(32), ValTypeClass::Float);
    println!("TypeNotClass: {}", class_error);

    let sized_error = ValueCheckError::TypeNotSized(ValTypeID::Void);
    println!("TypeNotSized: {}", sized_error);
}

use crate::{
    base::SlabRef,
    ir::{IRAllocs, IRWriter, ISubValueSSA, ITraceableValue, UserList, ValueSSA},
    typing::{ArrayTypeRef, TypeContext, ValTypeID},
};
use slab::Slab;

#[derive(Debug, Clone)]
pub enum ConstExprData {
    Array(Array),
    Struct(Struct),
}

impl ITraceableValue for ConstExprData {
    fn users(&self) -> &UserList {
        match self {
            ConstExprData::Array(data) => &data.users,
            ConstExprData::Struct(data) => &data.users,
        }
    }

    fn has_single_reference_semantics(&self) -> bool {
        false
    }
}

impl ConstExprData {
    pub fn get_value_type(&self) -> ValTypeID {
        match self {
            ConstExprData::Array(data) => ValTypeID::Array(data.arrty),
            ConstExprData::Struct(data) => data.structty,
        }
    }

    pub fn is_aggregate(&self) -> bool {
        matches!(self, ConstExprData::Array(_) | ConstExprData::Struct(_))
    }
}

#[derive(Debug)]
pub struct Array {
    pub arrty: ArrayTypeRef,
    pub elems: Vec<ValueSSA>,

    /// 有哪些指令使用了该常量表达式引用
    ///
    /// **重要限制**: ConstExpr 不是引用唯一的，相同值的常量表达式可能
    /// 有多个不同的 ConstExprRef。因此这个 UserList 只能反映使用了
    /// **当前这个引用** 的指令，而不是使用了相同**值**的所有指令。
    ///
    /// 要获得完整的使用信息，需要先运行 Const Expression Compression Pass
    /// 将所有值相同的表达式合并，之后 UserList 才能准确反映该值的所有使用者。
    ///
    /// ### 示例
    ///
    /// ```ignore
    /// // 编译前：两个相同值的不同引用
    /// let ref1 = ConstExpr([1, 2, 3]);  // users: [inst1, inst3]  
    /// let ref2 = ConstExpr([1, 2, 3]);  // users: [inst2, inst4]
    ///
    /// // 压缩后：合并为同一个引用
    /// let merged = ConstExpr([1, 2, 3]); // users: [inst1, inst2, inst3, inst4]
    /// ```
    pub users: UserList,
}

impl Clone for Array {
    /// 克隆时不克隆 users 列表，保持为空。因为 UserList 在设计上就不支持深拷贝.
    fn clone(&self) -> Self {
        Self {
            arrty: self.arrty,
            elems: self.elems.clone(),
            users: UserList::new_empty(),
        }
    }
}

impl Array {
    pub fn len(&self) -> usize {
        self.elems.len()
    }

    pub fn is_zero(&self, allocs: &IRAllocs) -> bool {
        self.elems.iter().all(|elem| elem.is_zero(allocs))
    }

    pub fn fmt_ir(&self, writer: &IRWriter) -> std::io::Result<()> {
        if self.is_zero(&writer.allocs) {
            return write!(writer.output.borrow_mut(), "zeroinitializer");
        }
        let elemty = self.arrty.get_element_type(&writer.type_ctx);
        writer.write_str("[")?;
        for (i, &elem) in self.elems.iter().enumerate() {
            if i > 0 {
                writer.write_str(", ")?;
            }
            debug_assert_eq!(
                elemty,
                elem.get_valtype(&writer.allocs),
                "Element type mismatch",
            );
            // 写入元素类型和操作数
            writer.write_type(elemty)?;
            writer.write_str(" ")?;
            writer.write_operand(elem)?;
        }
        writer.write_str("]")
    }

    pub fn new<'a>(arrty: ArrayTypeRef, elems: impl IntoIterator<Item = &'a ValueSSA>) -> Self {
        Self {
            arrty,
            elems: elems.into_iter().cloned().collect(),
            users: UserList::new_empty(),
        }
    }
    pub fn from_vec(arrty: ArrayTypeRef, elems: Vec<ValueSSA>) -> Self {
        Self { arrty, elems, users: UserList::new_empty() }
    }
    pub fn from_slice(arrty: ArrayTypeRef, elems: &[ValueSSA]) -> Self {
        Self { arrty, elems: elems.to_vec(), users: UserList::new_empty() }
    }
}

#[derive(Debug)]
pub struct Struct {
    pub structty: ValTypeID,
    pub elems: Vec<ValueSSA>,

    /// 有哪些指令使用了该常量表达式引用
    ///
    /// **重要限制**: ConstExpr 不是引用唯一的，相同值的常量表达式可能
    /// 有多个不同的 ConstExprRef。因此这个 UserList 只能反映使用了
    /// **当前这个引用** 的指令，而不是使用了相同**值**的所有指令。
    ///
    /// 要获得完整的使用信息，需要先运行 Const Expression Compression Pass
    /// 将所有值相同的表达式合并，之后 UserList 才能准确反映该值的所有使用者。
    ///
    /// ### 示例
    ///
    /// ```ignore
    /// // 编译前：两个相同值的不同引用
    /// let ref1 = ConstExpr([1, 2, 3]);  // users: [inst1, inst3]  
    /// let ref2 = ConstExpr([1, 2, 3]);  // users: [inst2, inst4]
    ///
    /// // 压缩后：合并为同一个引用
    /// let merged = ConstExpr([1, 2, 3]); // users: [inst1, inst2, inst3, inst4]
    /// ```
    pub users: UserList,
}

impl Clone for Struct {
    /// 克隆时不克隆 users 列表，保持为空。因为 UserList 在设计上就不支持深拷贝.
    fn clone(&self) -> Self {
        Self {
            structty: self.structty,
            elems: self.elems.clone(),
            users: UserList::new_empty(),
        }
    }
}

impl Struct {
    pub fn is_zero(&self, allocs: &IRAllocs) -> bool {
        self.elems.iter().all(|elem| elem.is_zero(allocs))
    }

    pub fn is_packed(&self, type_ctx: &TypeContext) -> bool {
        match self.structty {
            ValTypeID::Struct(structty) => structty.is_packed(type_ctx),
            ValTypeID::StructAlias(sa) => {
                let str = sa.get_aliasee(type_ctx);
                str.is_packed(type_ctx)
            }
            _ => panic!("Expected struct type but got {:?}", self.structty),
        }
    }

    pub fn fmt_ir(&self, writer: &IRWriter) -> std::io::Result<()> {
        if self.is_zero(&writer.allocs) {
            return write!(writer.output.borrow_mut(), "zeroinitializer");
        }
        let is_packed = self.is_packed(&writer.type_ctx);
        if is_packed {
            writer.write_str("<{")?;
        } else {
            writer.write_str("{")?;
        }

        let sty = match self.structty {
            ValTypeID::Struct(structty) => structty,
            ValTypeID::StructAlias(sa) => sa.get_aliasee(&writer.type_ctx),
            _ => panic!("Expected struct type but got {:?}", self.structty),
        };
        for (i, &elem) in self.elems.iter().enumerate() {
            if i > 0 {
                writer.write_str(", ")?;
            }
            let elemty = sty.get_field(writer.type_ctx, i);
            debug_assert_eq!(
                elemty,
                elem.get_valtype(&writer.allocs),
                "Element type mismatch",
            );
            writer.write_type(elemty)?;
            writer.write_str(" ")?;
            writer.write_operand(elem)?;
        }
        if is_packed { writer.write_str("}>") } else { writer.write_str("}") }
    }

    pub fn new<'a>(structty: ValTypeID, elems: impl IntoIterator<Item = &'a ValueSSA>) -> Self {
        Self {
            structty,
            elems: elems.into_iter().cloned().collect(),
            users: UserList::new_empty(),
        }
    }
    pub fn from_vec(structty: ValTypeID, elems: Vec<ValueSSA>) -> Self {
        Self { structty, elems, users: UserList::new_empty() }
    }
    pub fn from_slice(structty: ValTypeID, elems: &[ValueSSA]) -> Self {
        Self {
            structty,
            elems: elems.to_vec(),
            users: UserList::new_empty(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct ConstExprRef(usize);

impl SlabRef for ConstExprRef {
    type RefObject = ConstExprData;
    fn from_handle(handle: usize) -> Self {
        ConstExprRef(handle)
    }
    fn get_handle(&self) -> usize {
        self.0
    }
}

impl ConstExprRef {
    pub fn from_alloc(alloc: &mut Slab<ConstExprData>, data: ConstExprData) -> Self {
        ConstExprRef(alloc.insert(data))
    }
}

impl ISubValueSSA for ConstExprRef {
    fn try_from_ir(value: &ValueSSA) -> Option<&Self> {
        match value {
            ValueSSA::ConstExpr(x) => Some(x),
            _ => None,
        }
    }

    fn into_ir(self) -> ValueSSA {
        ValueSSA::ConstExpr(self)
    }

    fn get_valtype(self, allocs: &IRAllocs) -> ValTypeID {
        match self.to_data(&allocs.exprs) {
            ConstExprData::Array(data) => ValTypeID::Array(data.arrty),
            ConstExprData::Struct(data) => data.structty,
        }
    }

    fn try_gettype_noalloc(self) -> Option<ValTypeID> {
        None
    }

    fn is_zero(&self, allocs: &IRAllocs) -> bool {
        match self.to_data(&allocs.exprs) {
            ConstExprData::Array(data) => data.elems.iter().all(|elem| elem.is_zero(allocs)),
            ConstExprData::Struct(data) => data.elems.iter().all(|elem| elem.is_zero(allocs)),
        }
    }

    fn fmt_ir(&self, writer: &IRWriter) -> std::io::Result<()> {
        if self.is_zero(&writer.allocs) {
            return write!(writer.output.borrow_mut(), "zeroinitializer");
        }
        match self.to_data(&writer.allocs.exprs) {
            ConstExprData::Array(arr) => arr.fmt_ir(writer),
            ConstExprData::Struct(str) => str.fmt_ir(writer),
        }
    }
}

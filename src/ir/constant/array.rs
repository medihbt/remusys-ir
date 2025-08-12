use crate::{
    ir::{ExprCommon, IRAllocs, IRWriter, ISubExpr, ISubValueSSA, ValueSSA},
    typing::{ArrayTypeRef, TypeContext},
};

#[derive(Debug, Clone)]
pub struct Array {
    pub arrty: ArrayTypeRef,
    pub elems: Vec<ValueSSA>,
    pub common: ExprCommon,
}

impl ISubExpr for Array {
    fn get_common(&self) -> &ExprCommon {
        &self.common
    }

    fn is_aggregate(&self) -> bool {
        true
    }

    fn fmt_ir(&self, writer: &IRWriter) -> std::io::Result<()> {
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
}

impl Array {
    pub fn len(&self) -> usize {
        self.elems.len()
    }
    pub fn is_zero(&self, allocs: &IRAllocs) -> bool {
        self.elems.iter().all(|elem| elem.is_zero(allocs))
    }

    pub fn new<'a>(arrty: ArrayTypeRef, elems: impl IntoIterator<Item = &'a ValueSSA>) -> Self {
        Self {
            arrty,
            elems: elems.into_iter().cloned().collect(),
            common: ExprCommon::new(),
        }
    }
    pub fn from_vec(arrty: ArrayTypeRef, elems: Vec<ValueSSA>) -> Self {
        Self { arrty, elems, common: ExprCommon::new() }
    }
    pub fn from_slice(arrty: ArrayTypeRef, elems: &[ValueSSA]) -> Self {
        Self { arrty, elems: elems.to_vec(), common: ExprCommon::new() }
    }

    pub fn new_zero(type_ctx: &TypeContext, arrty: ArrayTypeRef) -> Self {
        let elemty = arrty.get_element_type(type_ctx);
        let nelems = arrty.get_num_elements(type_ctx);
        let elem0 = ValueSSA::new_zero(elemty);
        let elems = vec![elem0; nelems];
        Self { arrty, elems, common: ExprCommon::new() }
    }
}

use std::rc::Rc;

use crate::{
    ir::{
        ExprCommon, IRAllocs, IRWriter, ISubExpr, ISubValueSSA, IUser, OperandSet, Use, UseKind,
        ValueSSA,
    },
    typing::{ArrayTypeRef, TypeContext},
};

#[derive(Debug, Clone)]
pub struct Array {
    pub arrty: ArrayTypeRef,
    pub elems: Box<[Rc<Use>]>,
    pub common: ExprCommon,
}

impl IUser for Array {
    fn get_operands(&self) -> OperandSet {
        OperandSet::Fixed(&self.elems)
    }

    fn operands_mut(&mut self) -> &mut [Rc<Use>] {
        &mut self.elems
    }
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
        for (i, elem) in self.elems.iter().enumerate() {
            if i > 0 {
                writer.write_str(", ")?;
            }
            debug_assert_eq!(
                elemty,
                elem.get_operand().get_valtype(&writer.allocs),
                "Element type mismatch",
            );
            // 写入元素类型和操作数
            writer.write_type(elemty)?;
            writer.write_str(" ")?;
            writer.write_operand(elem.get_operand())?;
        }
        writer.write_str("]")
    }
}

impl Array {
    pub fn len(&self) -> usize {
        self.elems.len()
    }
    pub fn is_zero(&self, allocs: &IRAllocs) -> bool {
        self.elems
            .iter()
            .all(|elem| elem.get_operand().is_zero(allocs))
    }

    pub fn new(
        arrty: ArrayTypeRef,
        allocs: &IRAllocs,
        elems: impl IntoIterator<Item = ValueSSA>,
    ) -> Self {
        let elems = {
            let elems_iter = elems.into_iter();
            let mut elems = Vec::with_capacity(elems_iter.size_hint().0);
            for (id, elem) in elems_iter.enumerate() {
                let u = Use::new(UseKind::ArrayElem(id));
                u.set_operand(allocs, elem);
                elems.push(u);
            }
            elems.into_boxed_slice()
        };
        Self { arrty, elems, common: ExprCommon::new() }
    }
    pub fn from_slice(arrty: ArrayTypeRef, allocs: &IRAllocs, elems: &[ValueSSA]) -> Self {
        Self::new(arrty, allocs, elems.iter().cloned())
    }

    pub fn new_zero(type_ctx: &TypeContext, arrty: ArrayTypeRef, allocs: &IRAllocs) -> Self {
        let elemty = arrty.get_element_type(type_ctx);
        let nelems = arrty.get_num_elements(type_ctx);
        let elem0 = ValueSSA::new_zero(elemty);
        Self::new(arrty, allocs, std::iter::repeat(elem0).take(nelems))
    }
}

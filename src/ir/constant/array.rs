use std::rc::Rc;

use crate::{
    ir::{
        ConstData, ExprCommon, IRAllocs, IRWriter, ISubExpr, ISubValueSSA, IUser, OperandSet, Use,
        UseKind, ValueSSA,
    },
    typing::{ArrayTypeRef, TypeContext, ValTypeID},
};

#[derive(Debug, Clone)]
pub struct Array {
    pub arrty: ArrayTypeRef,
    pub elems: Box<[Rc<Use>]>,
    pub common: ExprCommon,
}

impl IUser for Array {
    fn get_operands(&self) -> OperandSet<'_> {
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
    fn common_mut(&mut self) -> &mut ExprCommon {
        &mut self.common
    }

    fn is_aggregate(&self) -> bool {
        true
    }

    fn fmt_ir(&self, writer: &IRWriter) -> std::io::Result<()> {
        if self.is_zero(&writer.allocs) {
            return write!(writer.output.borrow_mut(), "zeroinitializer");
        }
        if self.try_fmt_str_literal(writer)? {
            return Ok(());
        }
        self.fmt_array_literal(writer)
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

    fn fmt_array_literal(&self, writer: &IRWriter) -> std::io::Result<()> {
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
    fn try_fmt_str_literal(&self, writer: &IRWriter) -> std::io::Result<bool> {
        // 仅当元素类型为 i8 且所有元素均为常量时，才尝试格式化为字符串字面量
        // 其他情况均格式化为数组字面量
        let ValTypeID::Int(8) = self.arrty.get_element_type(&writer.type_ctx) else {
            return Ok(false);
        };
        let bytes = {
            use std::fmt::Write;
            let mut bytes = String::with_capacity(self.elems.len() + 4);
            bytes.push_str("c\"");
            for elem in &self.elems {
                let val = elem.get_operand();
                // 仅当元素为常量时，才尝试格式化为字符串字面量
                // 非常量元素会导致整个数组无法格式化为字符串字面量
                let ValueSSA::ConstData(cx) = val else {
                    return Ok(false);
                };
                let ch = match cx {
                    ConstData::Zero(_) => 0u8,
                    ConstData::Int(x) => x.as_signed() as u8,
                    // 非整数常量，无法格式化为字符串
                    _ => return Ok(false),
                };
                match ch {
                    x if x.is_ascii_graphic() => bytes.push(x as char),
                    _ => write!(bytes, "\\x{:02x}", ch).unwrap(),
                }
            }
            bytes.push('"');
            bytes
        };
        writer.write_str(&bytes)?;
        Ok(true)
    }
}

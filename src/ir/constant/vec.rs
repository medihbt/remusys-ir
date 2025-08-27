use crate::{
    ir::{ExprCommon, IRAllocs, IRWriter, ISubExpr, ISubValueSSA, IUser, OperandSet, Use},
    typing::{FixVecType, IValType, ScalarType},
};
use std::rc::Rc;

/// ## 向量表达式
///
/// ### LLVM-IR 语法
///
/// ```llvm-ir
/// <ty1 elem1, ty1 elem2, ...>
/// ```
#[derive(Debug, Clone)]
pub struct FixVec {
    pub(super) common: ExprCommon,
    pub elems: Box<[Rc<Use>]>,
    pub vecty: FixVecType,
}

impl IUser for FixVec {
    fn get_operands(&self) -> OperandSet {
        OperandSet::Fixed(&self.elems)
    }

    fn operands_mut(&mut self) -> &mut [Rc<Use>] {
        &mut self.elems
    }
}

impl ISubExpr for FixVec {
    fn get_common(&self) -> &ExprCommon {
        &self.common
    }

    fn is_aggregate(&self) -> bool {
        true
    }

    fn fmt_ir(&self, writer: &IRWriter) -> std::io::Result<()> {
        let elemty = self.vecty.get_elemty();
        writer.write_str("<")?;
        for (i, elem) in self.elems.iter().enumerate() {
            writer.write_type(elemty.into_ir())?;
            writer.write_str(" ")?;
            writer.write_operand(elem.get_operand())?;
            if i < self.elems.len() - 1 {
                writer.write_str(", ")?;
            }
        }
        writer.write_str(">")?;
        Ok(())
    }
}

impl FixVec {
    pub fn is_zero(&self, allocs: &IRAllocs) -> bool {
        self.elems.iter().all(|e| e.get_operand().is_zero(allocs))
    }

    pub fn get_elemty(&self) -> ScalarType {
        self.vecty.get_elemty()
    }
}

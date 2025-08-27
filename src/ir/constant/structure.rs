use crate::{
    ir::{
        ExprCommon, IRAllocs, IRWriter, ISubExpr, ISubValueSSA, IUser, OperandSet, Use, UseKind,
        ValueSSA,
    },
    typing::{IValType, StructTypeRef, TypeContext, ValTypeID},
};
use std::{cell::Ref, rc::Rc};

#[derive(Debug, Clone)]
pub struct Struct {
    pub structty: ValTypeID,
    pub elems: Box<[Rc<Use>]>,
    pub common: ExprCommon,
}

impl IUser for Struct {
    fn get_operands(&self) -> OperandSet {
        OperandSet::Fixed(&self.elems)
    }

    fn operands_mut(&mut self) -> &mut [Rc<Use>] {
        &mut self.elems
    }
}

impl ISubExpr for Struct {
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
        for (i, elem) in self.elems.iter().enumerate() {
            if i > 0 {
                writer.write_str(", ")?;
            }
            let elemty = sty.get_field(writer.type_ctx, i);
            debug_assert_eq!(
                elemty,
                elem.get_operand().get_valtype(&writer.allocs),
                "Element type mismatch",
            );
            writer.write_type(elemty)?;
            writer.write_str(" ")?;
            writer.write_operand(elem.get_operand())?;
        }
        if is_packed { writer.write_str("}>") } else { writer.write_str("}") }
    }
}

impl Struct {
    pub fn is_zero(&self, allocs: &IRAllocs) -> bool {
        self.elems
            .iter()
            .all(|elem| elem.get_operand().is_zero(allocs))
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

    pub fn new(
        structty: ValTypeID,
        allocs: &IRAllocs,
        elems: impl IntoIterator<Item = ValueSSA>,
    ) -> Self {
        let fields = {
            let iter = elems.into_iter();
            let mut fields = Vec::with_capacity(iter.size_hint().0);
            for (id, elem) in iter.enumerate() {
                let u = Use::new(UseKind::StructField(id));
                u.set_operand(allocs, elem);
                fields.push(u);
            }
            fields.into_boxed_slice()
        };
        Self { structty, elems: fields, common: ExprCommon::new() }
    }
    pub fn from_slice(structty: ValTypeID, allocs: &IRAllocs, elems: &[ValueSSA]) -> Self {
        Self::new(structty, allocs, elems.iter().cloned())
    }

    pub fn new_zero(type_ctx: &TypeContext, allocs: &IRAllocs, structty: StructTypeRef) -> Self {
        Self::new(
            structty.into_ir(),
            allocs,
            FieldBuilderIter::new(type_ctx, structty),
        )
    }
}

struct FieldBuilderIter<'a> {
    field_tys: Ref<'a, [ValTypeID]>,
    index: usize,
}

impl<'a> FieldBuilderIter<'a> {
    fn new(type_ctx: &'a TypeContext, sty: StructTypeRef) -> Self {
        let field_tys = sty.fields(type_ctx);
        Self { field_tys, index: 0 }
    }
    fn get(&self) -> Option<ValueSSA> {
        if self.index >= self.field_tys.len() {
            return None;
        }
        let field_ty = self.field_tys[self.index];
        Some(ValueSSA::new_zero(field_ty))
    }
}

impl<'a> Iterator for FieldBuilderIter<'a> {
    type Item = ValueSSA;

    fn next(&mut self) -> Option<Self::Item> {
        let val = self.get();
        self.index += 1;
        val
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.field_tys.len() - self.index;
        (remaining, Some(remaining))
    }
}

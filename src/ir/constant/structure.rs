use crate::{
    ir::{ExprCommon, IRAllocs, IRWriter, ISubValueSSA, ValueSSA},
    typing::{IValType, StructTypeRef, TypeContext, ValTypeID},
};

#[derive(Debug, Clone)]
pub struct Struct {
    pub structty: ValTypeID,
    pub elems: Vec<ValueSSA>,
    pub common: ExprCommon,
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
            common: ExprCommon::new(),
        }
    }
    pub fn from_vec(structty: ValTypeID, elems: Vec<ValueSSA>) -> Self {
        Self { structty, elems, common: ExprCommon::new() }
    }
    pub fn from_slice(structty: ValTypeID, elems: &[ValueSSA]) -> Self {
        Self { structty, elems: elems.to_vec(), common: ExprCommon::new() }
    }

    pub fn new_zero(type_ctx: &TypeContext, structty: StructTypeRef) -> Self {
        let field_types = structty.fields(type_ctx);
        let elems = {
            let mut elems = Vec::with_capacity(field_types.len());
            for &fieldty in field_types.iter() {
                elems.push(ValueSSA::new_zero(fieldty));
            }
            elems
        };
        Self {
            structty: structty.into_ir(),
            elems,
            common: ExprCommon::new(),
        }
    }
}

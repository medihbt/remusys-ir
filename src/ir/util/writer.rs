//! IR Writer implementation.

use std::io::{Result as IoResult, Write as IoWrite};

use crate::{
    base::slabref::SlabRef,
    ir::{
        ValueSSA,
        constant::{data::ConstData, expr::ConstExprRef},
        global::{GlobalData, GlobalRef, func::FuncStorage},
        module::Module,
    },
    typing::{context::TypeContext, id::ValTypeID, types::FuncTypeRef},
};

pub fn write_ir_module(module: &Module, writer: &mut dyn IoWrite) -> IoResult<()> {
    _write_struct_alias(&module.type_ctx, writer)?;

    let ret_funcdefs = _write_global_defs(module, writer)?;
    Ok(())
}

fn _write_struct_alias(type_ctx: &TypeContext, writer: &mut dyn IoWrite) -> IoResult<()> {
    let mut struct_aliases = Vec::new();
    type_ctx.read_struct_aliases(|name, aliasee| {
        let line = format!(
            "type %{} = {}\n",
            name,
            ValTypeID::Struct(aliasee).get_display_name(type_ctx)
        );
        struct_aliases.push(line);
    });
    for line in struct_aliases {
        writer.write_all(line.as_bytes())?;
    }
    Ok(())
}

/// Write global variables, aliases, and external functions to writer
/// and return function definitions.
fn _write_global_defs(module: &Module, writer: &mut dyn IoWrite) -> IoResult<Vec<GlobalRef>> {
    let mut ret_funcdefs = Vec::new();
    let type_ctx = module.type_ctx.as_ref();
    let alloc_value = module.borrow_value_alloc();
    for (id, global) in alloc_value._alloc_global.iter() {
        match global {
            GlobalData::Var(v) => {
                let line = match v.init.get() {
                    ValueSSA::None => format!(
                        "@{} = external global {}\n",
                        v.common.name,
                        v.common.content_ty.get_display_name(type_ctx)
                    ),
                    _ => {
                        format!(
                            "@{} = dso_local global {} {}",
                            v.common.name,
                            v.common.content_ty.get_display_name(type_ctx),
                            _format_value_by_semantic(module, v.init.get())
                        )
                    }
                };
                writer.write_all(line.as_bytes())?;
            }
            // LLVM global alias syntax: `@<name> = alias <type> <target>`
            GlobalData::Alias(a) => {
                let line = format!(
                    "@{} = alias {} @{}",
                    a.common.name,
                    a.common.content_ty.get_display_name(type_ctx),
                    a.target
                        .get()
                        .to_slabref_unwrap(&alloc_value._alloc_global)
                        .get_common()
                        .name
                );
                writer.write_all(line.as_bytes())?;
            }

            // LLVM extern function syntax: `declare <type> @<name>(<args>)`
            GlobalData::Func(f) => {
                if !f.is_extern() {
                    ret_funcdefs.push(GlobalRef::from_handle(id));
                    continue;
                }
                writer.write_all(
                    _format_func_header(f.get_name(), f.get_stored_func_type(), type_ctx, true)
                        .as_bytes(),
                )?;
                writer.write_all(b"\n")?;
            }
        }
    }

    Ok(ret_funcdefs)
}

/// Format value by semantic.
fn _format_value_by_semantic(module: &Module, value: ValueSSA) -> String {
    match value {
        ValueSSA::None => "<ValueSSA::None>".to_string(),
        ValueSSA::ConstData(d) => _format_const_data(&d),
        ValueSSA::ConstExpr(expr) => _fromat_const_expr(expr, module),
        ValueSSA::Global(g) => "@".to_string() + &*g.get_name_with_module(module),
        ValueSSA::Inst(_i) => todo!("instruction name not implemented"),
        ValueSSA::Block(_b) => todo!("block name not implemented"),
        ValueSSA::FuncArg(_, idx) => "%".to_string() + idx.to_string().as_str(),
    }
}

fn _format_const_data(const_data: &ConstData) -> String {
    match const_data {
        ConstData::Int(bits, value) => {
            let real_value = value & ((1 << bits) - 1);
            if *bits == 1 {
                if real_value == 0 {
                    "false".to_string()
                } else {
                    "true".to_string()
                }
            } else {
                real_value.to_string()
            }
        }
        // 格式化保留精度, 不要做任何舍入
        ConstData::Float(_, value) => value.to_string(),
        ConstData::Zero(ty) => match ty {
            ValTypeID::Int(..) | ValTypeID::Float(..) => "0".to_string(),
            ValTypeID::Ptr => "null".to_string(),
            ValTypeID::Array(..) | ValTypeID::Struct(..) | ValTypeID::StructAlias(..) => {
                "zeroinitializer".to_string()
            }
            _ => "0".to_string(),
        },
        ConstData::Undef(..) => "undef".to_string(),
        ConstData::PtrNull(..) => "null".to_string(),
    }
}

fn _fromat_const_expr(expr: ConstExprRef, module: &Module) -> String {
    todo!("format const expr")
}

/// LLVM function header syntax:
///
/// `declare|define <type> @<name>(<args>)`
fn _format_func_header(
    name: &str,
    functy: FuncTypeRef,
    type_ctx: &TypeContext,
    is_extern: bool,
) -> String {
    let declare_define = if is_extern { "declare" } else { "define" };
    let ret_ty = functy.get_return_type(type_ctx).get_display_name(type_ctx);
    let args = functy
        .get_args(type_ctx)
        .iter()
        .map(|arg| arg.get_display_name(type_ctx))
        .collect::<Vec<_>>()
        .join(", ");
    format!("{} {} @{}({})", declare_define, ret_ty, name, args)
}

fn _write_func_defs(module: &Module, func_ref: &GlobalRef, writer: &mut dyn IoWrite) -> IoResult<()> {
    todo!("write function body");
}

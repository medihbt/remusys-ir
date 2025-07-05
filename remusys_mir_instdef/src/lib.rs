use std::fmt::Debug;

use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::{ToTokens, quote};
use syn::{
    Expr, Ident, LitInt, Type,
    parse::{Parse, ParseStream},
    parse_macro_input,
};

/// Syntax:
///
/// ```bnf
/// ImplMirInst: "{"
///     $struct_ty:ident ","
///     $mir_variant:ident ","
///     OperandDecls ","
///     AcceptOpcodeDecl ","
///     FieldInits (",")?
/// "}"
/// ;
/// ```
#[derive(Debug)]
struct ImplMirInst {
    struct_name: String,
    mir_variant: String,
    operand_decls: Vec<OperandDecl>,
    accept_opcodes: IdentList,
    field_inits: Vec<FieldInit>,
}

/// Syntax:
///
/// ```bnf
/// OperandDecls: "operands" ":" "{" OperandDecl ("," OperandDecl)* (",")? "}"
/// ```
#[derive(Debug)]
struct OperandDecls(Vec<OperandDecl>);

/// Syntax:
///
/// ```bnf
/// OperandDecl: (OperandAttrs)? $name:ident ":" OperandInfo
/// OperandAttrs: #IdentList
/// ```
#[derive(Debug)]
struct OperandDecl {
    attrs: Vec<OperandAttr>,
    name: String,
    info: OperandInfo,
}
#[derive(Debug)]
struct OperandAttrs(Vec<OperandAttr>);

#[derive(Debug, PartialEq, Eq)]
enum OperandAttr {
    Readonly,
}

enum RegKind {
    VReg,
    PReg,
    Reg,
}

impl OperandDecl {
    fn write_default_init_to_token_stream(&self) -> proc_macro2::TokenStream {
        let name = self.name.as_str();
        let info = &self.info;

        match info {
            OperandInfo::VReg(Some(rsub)) => Self::write_default_init_reg_to_token_stream(
                RegKind::VReg,
                rsub.use_flags.as_ref().map(|f| &f.0),
                rsub.subreg_index,
            ),
            OperandInfo::PReg(Some(rsub)) => Self::write_default_init_reg_to_token_stream(
                RegKind::PReg,
                rsub.use_flags.as_ref().map(|f| &f.0),
                rsub.subreg_index,
            ),
            OperandInfo::Reg(Some(rsub)) => Self::write_default_init_reg_to_token_stream(
                RegKind::Reg,
                rsub.use_flags.as_ref().map(|f| &f.0),
                rsub.subreg_index,
            ),
            _ => {
                // For other operand types, we just return a new instance of the type.
                let operand_type = info.get_typename_token();
                quote! {
                    #operand_type::new_empty_mirsubop()
                }
            }
        }
    }

    fn write_default_init_reg_to_token_stream(
        kind: RegKind,
        use_flags: Option<&Vec<String>>,
        subreg_index: Option<SubRegIndexExpr>,
    ) -> proc_macro2::TokenStream {
        let reg_type = match kind {
            RegKind::VReg => quote! { VReg },
            RegKind::PReg => quote! { PReg },
            RegKind::Reg => quote! { RegOperand },
        };
        let after_init_subreg = if let Some(subreg) = subreg_index {
            let bits = subreg.bits;
            let index = subreg.index;
            quote! {
                #reg_type::new_empty_mirsubop()
                    .insert_subreg_index(SubRegIndex::new(#bits, #index))
            }
        } else {
            quote! { #reg_type::new_empty_mirsubop() }
        };

        let after_init_use_flags = if let Some(flags) = use_flags {
            let flags_tokens: Vec<proc_macro2::TokenStream> = flags
                .into_iter()
                .map(|flag| {
                    let flag_ident = Ident::new(&flag, Span::call_site());
                    quote! { RegUseFlags::#flag_ident }
                })
                .collect();
            quote! {
                #after_init_subreg.insert_use_flags(#(#flags_tokens)|*)
            }
        } else {
            after_init_subreg
        };
        after_init_use_flags
    }

    fn write_accessors(&self, self_index: usize) -> proc_macro2::TokenStream {
        let name = self.name.as_str();
        let access_name = Ident::new(name, Span::call_site());
        let getter_name = Ident::new(&format!("get_{name}"), Span::call_site());
        let setter_name = Ident::new(&format!("set_{name}"), Span::call_site());
        let self_type = self.info.get_typename_token();

        let getter_doc = format!("Getter for operand {name} at index {self_index}");
        let setter_doc = format!("Setter for operand {name} at index {self_index}");
        let access_doc = format!(
            "Accessor for operand `{name}` at index `{self_index}` in the instruction.\n \
            requires type: `{self_type}`.\n \
            getter: `{getter_name}`.",
        );

        let mut accessors = quote! {
            #[doc = #access_doc]
            pub fn #access_name(&self) -> &Cell<MirOperand> {
                &self._operands[#self_index]
            }
            #[doc = #getter_doc]
            pub fn #getter_name(&self) -> #self_type {
                let operand_ref = &self._operands[#self_index];
                #self_type::from_mirop(operand_ref.get())
            }
        };
        if !self.attrs.contains(&OperandAttr::Readonly) {
            accessors.extend(quote! {
                // #[doc = #setter_doc]
                pub fn #setter_name(&mut self, operand: #self_type) {
                    let operand_ref = &self._operands[#self_index];
                    operand_ref.set(operand.insert_to_mirop(operand_ref.get()));
                }
            });
        }
        accessors
    }
}

/// Syntax:
///
/// ```bnf
/// OperandInfo: "VReg" (RegSubInfo)?
///            | "PReg" (RegSubInfo)?
///            | "Reg"  (RegSubInfo)?
///            | "Imm"
///            | "PState"
///            | "Label"
///            | "Global"
///            | "CompactSwitchTab"
///            | "SparseSwitchTab"
///            | "Symbol"
///            | "MirOperand"
///            ;
/// ```
#[derive(Debug)]
enum OperandInfo {
    VReg(Option<RegSubInfo>),
    PReg(Option<RegSubInfo>),
    Reg(Option<RegSubInfo>),
    Imm,
    PState,
    Label,
    Global,
    CompactSwitchTab,
    SparseSwitchTab,
    Symbol,
    MirOperand,
}

impl OperandInfo {
    fn get_typename_token(&self) -> proc_macro2::TokenStream {
        match self {
            OperandInfo::VReg(_) => quote::quote! { VReg },
            OperandInfo::PReg(_) => quote::quote! { PReg },
            OperandInfo::Reg(_) => quote::quote! { RegOperand },
            OperandInfo::Imm => quote::quote! { i64 },
            OperandInfo::PState => quote::quote! { PStateSubOperand },
            OperandInfo::Label => quote::quote! { MirBlockRef },
            OperandInfo::Global => quote::quote! { MirGlobalRef },
            OperandInfo::CompactSwitchTab => quote::quote! { VecSwitchTabPos },
            OperandInfo::SparseSwitchTab => quote::quote! { BinSwitchTabPos },
            OperandInfo::Symbol => quote::quote! { ImmSymbolOperand },
            OperandInfo::MirOperand => quote::quote! { MirOperand },
        }
    }
}

/// Syntax:
///
/// ```bnf
/// RegSubInfo: "{"
///     ("use_flags" ":" UseFlags ",")?
///     ("subreg_index" ":" SubRegIndexExpr ",")?
/// "}";
///
/// UseFlags: IdentList;
/// ```
#[derive(Debug, Clone)]
struct RegSubInfo {
    use_flags: Option<IdentList>,
    subreg_index: Option<SubRegIndexExpr>,
}

/// Syntax:
///
/// ```bnf
/// SubRegIndexExpr: "(" $bits:number "," $index:number ")" ;
/// ```
#[derive(Debug, Clone, Copy)]
struct SubRegIndexExpr {
    bits: u8,
    index: u8,
}

/// Syntax:
///
/// ```bnf
/// AcceptOpcodeDecl: "accept_opcode" ":" IdentList ;
/// ```
#[derive(Debug)]
struct AcceptOpcodeDecl(IdentList);

/// Syntax:
///
/// ```bnf
/// FieldInits: "field_inits" ":" "{" (FieldInit ";")* "}"
/// ```
#[derive(Debug)]
struct FieldInits(Vec<FieldInit>);

/// Syntax:
///
/// ```bnf
/// FieldInit: ("pub")? $name:ident ":" $ty:type "=" $value:expr
/// ```
struct FieldInit {
    pub name: String,
    pub ty: Type,
    pub value: Expr,
    pub is_pub: bool, // 是否为 public 字段
}

impl Debug for FieldInit {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FieldInit")
            .field("name", &self.name)
            .field("is_pub", &self.is_pub)
            .finish()
    }
}

/// Syntax:
///
/// ```bnf
/// IdentList: "[" $item:ident ("," $item:ident)* (",")? "]"
/// ```
#[derive(Debug, Clone)]
struct IdentList(Vec<String>);

impl Parse for ImplMirInst {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content = input;
        // syn::braced!(content in input);

        let struct_ty: Ident = content.parse().expect("RemusysError");
        content.parse::<syn::Token![,]>().expect("RemusysError");

        let mir_variant: Ident = content.parse().expect("RemusysError");
        content.parse::<syn::Token![,]>().expect("RemusysError");

        let operand_decls: OperandDecls = content.parse().expect("RemusysError");
        content.parse::<syn::Token![,]>().expect("RemusysError");

        let accept_opcodes: AcceptOpcodeDecl = content.parse().expect("RemusysError");
        content.parse::<syn::Token![,]>().expect("RemusysError");

        let field_inits = if content.peek(Ident) && content.peek2(syn::Token![:]) {
            content.parse::<FieldInits>().expect("RemusysError")
        } else {
            FieldInits(Vec::new())
        };

        if content.peek(syn::Token![,]) {
            content.parse::<syn::Token![,]>().expect("RemusysError");
        }
        
        Ok(ImplMirInst {
            struct_name: struct_ty.to_string(),
            mir_variant: mir_variant.to_string(),
            operand_decls: operand_decls.0,
            accept_opcodes: accept_opcodes.0,
            field_inits: field_inits.0,
        })
    }
}

impl Parse for OperandDecls {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let operand_leading = input.parse::<Ident>().expect("RemusysError");
        if operand_leading != "operands" {
            return Err(syn::Error::new(
                operand_leading.span(),
                "expected 'operands' keyword",
            ));
        }
        input.parse::<syn::Token![:]>().expect("RemusysError");

        let content;
        syn::braced!(content in input);

        let mut decls = Vec::new();
        while !content.is_empty() {
            let decl: OperandDecl = content.parse().expect("RemusysError");
            decls.push(decl);
            if content.peek(syn::Token![,]) {
                content.parse::<syn::Token![,]>().expect("RemusysError");
            }
        }

        Ok(OperandDecls(decls))
    }
}

impl Parse for OperandDecl {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let mut attrs = Vec::new();
        if input.peek(syn::Token![#]) {
            let attr: OperandAttrs = input.parse().expect("RemusysError");
            attrs = attr.0;
        }

        let name: Ident = input.parse().expect("RemusysError");
        input.parse::<syn::Token![:]>().expect("RemusysError");

        let info: OperandInfo = input.parse().expect("RemusysError");

        Ok(OperandDecl {
            attrs,
            name: name.to_string(),
            info,
        })
    }
}

impl Parse for OperandAttrs {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        input.parse::<syn::Token![#]>().expect("RemusysError");
        let ident_list: IdentList = input.parse().expect("RemusysError");
        let mut attrs = Vec::new();
        for ident in ident_list.0 {
            match ident.as_str() {
                "readonly" => attrs.push(OperandAttr::Readonly),
                _ => {
                    return Err(syn::Error::new(
                        input.span(),
                        format!("Unknown operand attribute: {}", ident),
                    ));
                }
            }
        }
        Ok(OperandAttrs(attrs))
    }
}

impl Parse for OperandInfo {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ident = input.parse::<Ident>().expect("RemusysError");
        let ident_str = ident.to_string();
        match ident_str.as_str() {
            "VReg" | "PReg" | "Reg" => {
                let sub_info = if input.peek(syn::token::Brace) {
                    Some(input.parse::<RegSubInfo>()?)
                } else {
                    None
                };
                match ident_str.as_str() {
                    "VReg" => Ok(OperandInfo::VReg(sub_info)),
                    "PReg" => Ok(OperandInfo::PReg(sub_info)),
                    "Reg" => Ok(OperandInfo::Reg(sub_info)),
                    _ => unreachable!(),
                }
            }
            "Imm" => Ok(OperandInfo::Imm),
            "PState" => Ok(OperandInfo::PState),
            "Label" => Ok(OperandInfo::Label),
            "Global" => Ok(OperandInfo::Global),
            "CompactSwitchTab" => Ok(OperandInfo::CompactSwitchTab),
            "SparseSwitchTab" => Ok(OperandInfo::SparseSwitchTab),
            "Symbol" => Ok(OperandInfo::Symbol),
            "MirOperand" => Ok(OperandInfo::MirOperand),
            _ => Err(syn::Error::new(
                ident.span(),
                format!("Unknown operand info type: {}", ident_str),
            )),
        }
    }
}

impl Parse for RegSubInfo {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        syn::braced!(content in input);

        let mut use_flags = None;
        let mut subreg_index = None;

        while !content.is_empty() {
            let key: Ident = content.parse().expect("RemusysError");
            content.parse::<syn::Token![:]>().expect("RemusysError");
            match key.to_string().as_str() {
                "use_flags" => {
                    use_flags = Some(content.parse()?);
                }
                "subreg_index" => {
                    subreg_index = Some(content.parse()?);
                }
                _ => {
                    return Err(syn::Error::new(
                        key.span(),
                        format!("Unknown register sub-info field: {}", key),
                    ));
                }
            }
            if content.peek(syn::Token![,]) {
                content.parse::<syn::Token![,]>().expect("RemusysError");
            }
        }
        Ok(RegSubInfo {
            use_flags,
            subreg_index,
        })
    }
}

impl Parse for SubRegIndexExpr {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        syn::parenthesized!(content in input);

        let bits: LitInt = content.parse().expect("RemusysError");
        let bits: u8 = bits.base10_parse().expect("RemusysError");
        content.parse::<syn::Token![,]>().expect("RemusysError");
        let index: LitInt = content.parse().expect("RemusysError");
        let index: u8 = index.base10_parse().expect("RemusysError");

        Ok(SubRegIndexExpr { bits, index })
    }
}

impl Parse for AcceptOpcodeDecl {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let accept_opcode: Ident = input.parse().expect("RemusysError");
        if accept_opcode != "accept_opcode" {
            return Err(syn::Error::new(
                accept_opcode.span(),
                "expected 'accept_opcode' keyword",
            ));
        }
        input.parse::<syn::Token![:]>().expect("RemusysError");
        let ident_list: IdentList = input.parse().expect("RemusysError");
        Ok(AcceptOpcodeDecl(ident_list))
    }
}

impl Parse for FieldInits {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let field_inits: Ident = input.parse().expect("RemusysError");
        if field_inits != "field_inits" {
            return Err(syn::Error::new(
                field_inits.span(),
                "expected 'field_inits' keyword",
            ));
        }
        input.parse::<syn::Token![:]>().expect("RemusysError");

        let content;
        syn::braced!(content in input);

        let mut inits = Vec::new();
        while !content.is_empty() {
            let init: FieldInit = content.parse().expect("RemusysError");
            inits.push(init);
            if content.peek(syn::Token![;]) {
                content.parse::<syn::Token![;]>().expect("RemusysError");
            }
        }
        Ok(FieldInits(inits))
    }
}

impl Parse for FieldInit {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let is_pub = input.peek(syn::Token![pub]);
        if is_pub {
            input.parse::<syn::Token![pub]>().expect("RemusysError");
        }

        let name: Ident = input.parse().expect("RemusysError");
        input.parse::<syn::Token![:]>().expect("RemusysError");
        let ty: Type = input.parse().expect("RemusysError");
        input.parse::<syn::Token![=]>().expect("RemusysError");
        let value: Expr = input.parse().expect("RemusysError");

        Ok(FieldInit {
            name: name.to_string(),
            ty,
            value,
            is_pub,
        })
    }
}

impl Parse for IdentList {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        syn::bracketed!(content in input);

        let mut items = Vec::new();
        while !content.is_empty() {
            let item: Ident = content.parse().expect("RemusysError");
            items.push(item.to_string());
            if content.peek(syn::Token![,]) {
                content.parse::<syn::Token![,]>().expect("RemusysError");
            }
        }

        Ok(IdentList(items))
    }
}

impl quote::ToTokens for ImplMirInst {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        tokens.extend(self.write_impl_mir_subinst_for_inst_block());
        tokens.extend(self.write_impl_inst_block());
    }
}

impl ImplMirInst {
    fn write_impl_mir_subinst_for_inst_block(&self) -> proc_macro2::TokenStream {
        let struct_name = Ident::new(&self.struct_name, Span::call_site());
        let mir_variant = Ident::new(&self.mir_variant, Span::call_site());
        let noperands = self.operand_decls.len();

        let accepts_opcode_function = self.write_accepts_opcode_function();
        let new_empty_function = self.write_new_empty_function();

        // Generate the implementation block
        quote! {
            impl IMirSubInst for #struct_name {
                fn from_mir_inst(inst: &MirInst) -> Option<&Self> {
                    if let MirInst::#mir_variant(inst) = inst {
                        Some(inst)
                    } else {
                        None
                    }
                }
                fn into_mir_inst(self) -> MirInst {
                    MirInst::#mir_variant(self)
                }

                fn common(&self) -> &MirInstCommon {
                    &self._common
                }
                fn operands(&self) -> &[Cell<MirOperand>] {
                    &self._operands[..#noperands]
                }

                #accepts_opcode_function
                #new_empty_function
            }
        }
    }

    fn write_accepts_opcode_function(&self) -> proc_macro2::TokenStream {
        let mut opcode_matches = Vec::new();
        for opcode in &self.accept_opcodes.0 {
            let opcode_ident = Ident::new(opcode, Span::call_site());
            opcode_matches.push(quote! { MirOP::#opcode_ident });
        }
        quote! {
            fn accepts_opcode(opcode: MirOP) -> bool {
                matches!(opcode, #(#opcode_matches)|*)
            }
        }
    }

    fn write_new_empty_function(&self) -> proc_macro2::TokenStream {
        let mut operands_init = Vec::new();
        for decl in self.operand_decls.iter() {
            let operand_init = decl.write_default_init_to_token_stream();
            operands_init.push(quote! { Cell::new(#operand_init.into_mirop()) });
        }
        let mut field_inits = Vec::with_capacity(self.field_inits.len());
        for field in &self.field_inits {
            let field_name = Ident::new(&field.name, Span::call_site());
            let field_value = &field.value;

            let field_init = quote! { #field_name: #field_value, };
            field_inits.push(field_init);
        }
        quote! {
            fn new_empty(opcode: MirOP) -> Self {
                Self {
                    _common: MirInstCommon::new(opcode),
                    _operands: [#(#operands_init),*],
                    #(#field_inits)*
                }
            }
        }
    }

    fn write_impl_inst_block(&self) -> proc_macro2::TokenStream {
        let struct_name = Ident::new(&self.struct_name, Span::call_site());
        let mut operand_accessors = Vec::new();
        for (i, decl) in self.operand_decls.iter().enumerate() {
            let accessors = decl.write_accessors(i);
            operand_accessors.push(accessors);
        }
        quote! {
            impl #struct_name {
                #(#operand_accessors)*
            }
        }
    }
}

/// Generate code to implement an instruction in MIR.
///
/// **Usage:** `impl_mir_inst! { ... }`
///
/// The user should write the instruction structure itself like this:
///
/// ```rust
/// // Should implement `Debug` trait.
/// #[derive(Debug)]
/// pub struct MyMIRInst {
///     /// Required field for MIR instruction: Common metadata.
///     /// The common data is pre-defined inside `remusys-ir` crate.
///     _common: MirInstCommon,
///
///     /// Required field for MIR instruction: Operand Array.
///     /// Manually set the length of the array.
///     /// The length of the array should be equal to the number of operands.
///     /// The array should be initialized with `Cell<MirOperand>` type.
///     _operands: [Cell<MirOperand>; 2],
///
///     /// self-defined fields, which should be consistent with the `field_inits` in the macro.
///     pub my_field: u32,
/// }
///
/// impl_mir_inst! {
///     MyMIRInst, // The name of the instruction structure.
///     MyMIRInst, // The variant inside `MirInst` enum.
///     operands: {
///        #[readonly] my_dest_reg: Reg {
///            use_flags: [DEF],
///            subreg_index: (64, 0)
///        },
///        my_src_reg: Reg,
///     },
///     // The opcodes that this instruction accepts.
///     accept_opcode: [Opcode1, Opcode2],
///     field_inits: {
///         // The field should be initialized with a value.
///         // The field definition should be consistent with the instruction structure.
///         my_field: u32 = 42;
///     }
/// }
/// ```
///
/// instruction template generated from macro `impl_mir_instdef` is like this:
///
/// ```rust
/// // hand-written structure definition
/// // ...
///
/// // generated from macro `impl_mir_instdef`
/// // At position *a: the "MyMIRInst" is the name of the instruction structure.
/// impl IMirSubInst for MyMIRInst /* *a */ {
///     fn from_mir_inst(inst: &MirInst) -> Option<&Self> {
///         // At positon *b: the "MIRInst" is the enum variant defined in `remusys-ir` crate.
///         if let MirInst::MyMIRInst/* *b */(inst) = inst {
///             Some(inst)
///         } else {
///             None
///         }
///     }
///     fn into_mir_inst(self) -> MirInst {
///         MirInst::MyMIRInst(self)
///     }
///
///     fn common(&self) -> &MirInstCommon {
///         &self._common
///     }
///     fn operands(&self) -> &[Cell<MirOperand>] {
///         // the "2" comes from the number of "operands" field defined in the macro.
///         &self._operands[..2]
///     }
///
///     fn accepts_opcode(opcode: &Opcode) -> bool {
///         // NOTE: the opcode enum name is `MirOP` in `remusys-ir` crate.
///         matches!(opcode, MirOP::Opcode1 | MirOP::Opcode2)
///     }
///
///     fn new_empty(opcode: MirOP) -> Self {
///         Self {
///             _common: MirInstCommon::new(opcode),
///             _operands: [
///                 // operand 0: `my_dest_reg`
///                 // kind: `Reg` => mapped to `RegOperand` type
///                 Cell::new(RegOperand::new()
///                     // has field "subreg_index" => set the sub-register index
///                     .insert_subreg_index(SubRegIndex::new(6, 0)) // 6 = log2(64)
///                     // has field "use_flags" => set the use flags
///                     .insert_use_flags(UseFlags::DEF)
///                     .to_mir_operand()
///                 ),
///                 // operand 1: `my_src_reg`
///                 // kind: `Reg` => mapped to `RegOperand` type
///                 Cell::new(RegOperand::new()
///                     // no sub-register index => not set
///                     // no use flags => not set
///                     .to_mir_operand()
///                 ),
///             ],
///             // The fields defined in the macro `field_inits` should be initialized here.
///             my_field: 42, // initialized with the value defined in the macro
///         }
///     }
/// }
///
/// impl MyMIRInst {
///     pub fn new(
///         opcode: MirOP,
///         my_dest_reg: RegOperand, // operand 0
///         my_src_reg: RegOperand,  // operand 1
///         my_field: u32, // field defined in the macro
///     ) -> Self {
///         let mut inst = Self::new_empty(opcode);
///         // set the operands
///         inst._operands[0].set(
///             my_dest_reg.insert_to_mir_operand(inst._operands[0].get())
///         );
///         inst._operands[1].set(
///             my_src_reg.insert_to_mir_operand(inst._operands[1].get())
///         );
///         inst.my_field = my_field;
///         inst
///     }
///     
///     // Operand getter & setter section
///     // part 1: trivial referece accessor
///     pub fn my_dest_reg(&self) -> &Cell<MirOperand> {
///         &self._operands[0]
///     }
///     // part 2: getter for the operand in limited type
///     pub fn get_my_dest_reg(&self) -> RegOperand {
///         RegOperand::from_mir_operand(self._operands[0].get())
///     }
///     // part 3: setter for the operand in limited type
///     // for this operand marked with `readonly` attribute,
///     // the setter should not be provided.
///
///     // Operand 1: `my_src_reg`
///     pub fn my_src_reg(&self) -> &Cell<MirOperand> {
///         &self._operands[1]
///     }
///     pub fn get_my_src_reg(&self) -> RegOperand {
///         RegOperand::from_mir_operand(self._operands[1].get())
///     }
///     pub fn set_my_src_reg(&mut self, operand: RegOperand) {
///         self._operands[1].set(operand.insert_to_mir_operand(self._operands[1].get()));
///     }
/// }
/// ```
#[doc = "Usage: impl_mir_inst! { ... }"]
#[proc_macro]
pub fn impl_mir_inst(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ImplMirInst);
    let mut output = proc_macro2::TokenStream::new();
    input.to_tokens(&mut output);
    output.into()
}

#[cfg(test)]
mod testing {
    use super::*;

    #[test]
    fn test_impl_mir_inst_parse() {
        let input = r#"
            MyStruct,
            MyMirVariant,
            operands: {
                #[readonly] my_operand: VReg {
                    use_flags: [DEF],
                    subreg_index: (32, 0)
                },
                my_other_operand: PReg {
                    use_flags: [KILL, IMPLICIT_DEF],
                    subreg_index: (64, 1)
                },
                my_imm_operand: Imm,
                my_label_operand: Label,
                my_global_operand: Global,
                #[readonly] my_pstate_operand: PState,
                my_switch_tab_operand: CompactSwitchTab,
            },
            accept_opcode: [Opcode1, Opcode2],
            field_inits: {
                my_field: u32 = 42;
                my_other_field: String = "Hello, world!".into();
                my_third_field: bool = true;
            },
        "#;
        let parsed: syn::Result<ImplMirInst> = syn::parse_str(input);
        let inst = match parsed {
            Ok(inst) => {
                println!("Parsed ImplMirInst: {inst:#?}");
                inst
            }
            Err(e) => panic!("Failed to parse ImplMirInst: {}", e),
        };

        let mut output = proc_macro2::TokenStream::new();
        inst.to_tokens(&mut output);
        println!("Generated code:\n{output}");
    }
}

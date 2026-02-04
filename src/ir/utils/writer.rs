use crate::{
    SymbolStr,
    ir::{inst::*, module::allocs::IPoolAllocated, *},
    typing::*,
};
use mtb_entity_slab::IEntityAllocID;
use smol_str::format_smolstr;
use std::{
    cell::{Cell, RefCell, RefMut},
    collections::HashMap,
    path::Path,
    rc::Rc,
};

pub fn write_ir_to_file(path: impl AsRef<Path>, module: &Module, option: IRWriteOption) {
    let mut file = std::fs::File::create(path).unwrap();
    let mut writer = IRWriter::from_module(&mut file, module);
    writer.set_option(option);
    writer.fmt_module().unwrap();
}
#[derive(Debug, thiserror::Error)]
pub enum IRWriteErr {
    #[error("IO Error: {0}")]
    IO(#[from] std::io::Error),

    #[error("Function '{0}' is external and cannot be written.")]
    FuncIsExtern(SymbolStr),

    #[error("JumpTarget CFG edge has no target block.")]
    JumpToNoTarget,

    #[error("Phi instruction has incoming value that is not a block. (real: {0:?})")]
    PhiIncomeNotBlock(ValueSSA),
}
pub type IRWriteRes<T = ()> = Result<T, IRWriteErr>;

#[derive(Debug, Clone, Copy, Default)]
pub struct IRWriteOption {
    pub show_ptrid: bool,
    pub show_users: bool,
    pub show_preds: bool,
    pub llvm_compatible: bool,
}
impl IRWriteOption {
    pub fn loud() -> Self {
        Self {
            show_ptrid: true,
            show_users: true,
            show_preds: true,
            llvm_compatible: false,
        }
    }
    pub fn quiet() -> Self {
        Self::default()
    }

    pub fn show_ptrid(self, val: bool) -> Self {
        Self { show_ptrid: val, ..self }
    }
    pub fn show_users(self, val: bool) -> Self {
        Self { show_users: val, ..self }
    }
    pub fn show_preds(self, val: bool) -> Self {
        Self { show_preds: val, ..self }
    }
    pub fn llvm_compatible(self, val: bool) -> Self {
        Self { llvm_compatible: val, ..self }
    }
}

/// Status that is kept during writing an IR module.
#[derive(Default)]
pub struct IRWriteModuleStat {
    pub indent: Cell<usize>,
    pub option: IRWriteOption,
    inner: RefCell<IRStatInner>,
}

#[derive(Default)]
struct IRStatInner {
    type_names: HashMap<ValTypeID, Rc<str>>,
    str_literals: HashMap<ExprID, Option<Rc<str>>>,
    llvm_mapping: LLVMAdaptMapping,
}

impl IRWriteModuleStat {
    pub fn insert_option(&mut self, option: IRWriteOption) -> &mut Self {
        self.option = option;
        self
    }

    pub fn map_value(&self, value: ValueSSA, module: &Module) -> ValueSSA {
        let mut inner = self.inner.borrow_mut();
        if self.option.llvm_compatible {
            inner.llvm_mapping.map_value(module, value)
        } else {
            value
        }
    }

    pub fn get_typename(&self, type_id: ValTypeID, module: &Module) -> Rc<str> {
        let mut inner = self.inner.borrow_mut();
        let type_names = &mut inner.type_names;
        if let Some(name) = type_names.get(&type_id) {
            name.clone()
        } else {
            let tyname = type_id.get_display_name(&module.tctx);
            let tyname: Rc<str> = Rc::from(tyname);
            type_names.insert(type_id, tyname.clone());
            tyname
        }
    }

    pub fn get_str_literal(&self, expr_id: ExprID, module: &Module) -> Option<Rc<str>> {
        let mut inner = self.inner.borrow_mut();
        let str_literals = &mut inner.str_literals;
        if let Some(lit) = str_literals.get(&expr_id) {
            return lit.clone();
        }
        let maybe_str = match expr_id.deref_ir(&module.allocs) {
            ExprObj::Array(arr) => Self::write_arrexp_as_string(arr, &module.allocs),
            ExprObj::DataArray(darr) => Self::write_darray_as_string(darr),
            ExprObj::SplatArray(splat) => Self::write_splat_as_string(splat, &module.allocs),
            _ => None,
        };
        let lit: Option<Rc<str>> = maybe_str.map(Rc::from);
        str_literals.insert(expr_id, lit.clone());
        lit
    }

    fn write_arrexp_as_string(arrexp: &ArrayExpr, allocs: &IRAllocs) -> Option<String> {
        if arrexp.elemty != ValTypeID::Int(8) {
            return None;
        }
        let mut res = String::with_capacity(arrexp.elems.len() + 4);
        let bytes = {
            arrexp.operands_iter().map(|useid| {
                useid
                    .get_operand(allocs)
                    .as_apint()
                    .map(|x| x.as_unsigned() as u8)
            })
        };
        Self::bytes_as_string(bytes, &mut res)?;
        Some(res)
    }
    fn write_darray_as_string(darray: &DataArrayExpr) -> Option<String> {
        let ConstArrayData::I8(i8arr) = &darray.data else {
            return None;
        };
        let bytes = i8arr.iter().map(|&b| Some(b as u8));
        let mut res = String::with_capacity(i8arr.len() + 4);
        Self::bytes_as_string(bytes, &mut res)?;
        Some(res)
    }
    fn write_splat_as_string(splat: &SplatArrayExpr, allocs: &IRAllocs) -> Option<String> {
        if splat.elemty != ValTypeID::Int(8) {
            return None;
        }
        let apint = splat.get_elem(allocs).as_apint()?;
        let byte = apint.as_unsigned() as u8;
        let nelems = splat.get_nelems();
        let bytes = std::iter::repeat_n(Some(byte), nelems);
        let mut res = String::with_capacity(nelems + 4);
        Self::bytes_as_string(bytes, &mut res)?;
        Some(res)
    }
    fn bytes_as_string(
        bytes: impl IntoIterator<Item = Option<u8>>,
        buff: &mut String,
    ) -> Option<()> {
        use std::fmt::Write;
        buff.push_str("c\"");
        for ch in bytes {
            let ch = ch?;
            match ch {
                b'"' => buff.push_str("\\22"),
                b'\\' => buff.push_str("\\5c"),
                0x20..=0x7e if ch.is_ascii_graphic() => buff.push(ch as char),
                b' ' => buff.push(' '),
                _ => write!(buff, "\\{ch:02x}").unwrap(),
            }
        }
        buff.push('"');
        Some(())
    }
}

pub struct IRWriteFuncStat {
    pub func_id: FuncID,
    pub numbers: IRNumberValueMap,
}

impl IRWriteFuncStat {
    pub fn new(allocs: &IRAllocs, func_id: FuncID) -> Result<Self, IRWriteErr> {
        let option = NumberOption::ignore_all();
        let Some(numbers) = IRNumberValueMap::new(allocs, func_id, option) else {
            let name = func_id.deref_ir(allocs).clone_name();
            return Err(IRWriteErr::FuncIsExtern(name));
        };
        Ok(Self { func_id, numbers })
    }
}

pub trait WriteIR<'a>: 'a {
    fn get_module(&self) -> &'a Module;
    fn module_stat(&self) -> &IRWriteModuleStat;
    fn writer(&self) -> RefMut<'_, dyn std::io::Write + 'a>;
    fn get_numbers(&self) -> Option<&IRNumberValueMap>;

    fn get_allocs(&self) -> &'a IRAllocs {
        &self.get_module().allocs
    }
    fn get_tctx(&self) -> &'a TypeContext {
        &self.get_module().tctx
    }

    fn inc_indent(&self) {
        let stat = self.module_stat();
        stat.indent.set(stat.indent.get() + 1);
    }
    fn dec_indent(&self) {
        let stat = self.module_stat();
        stat.indent.set(stat.indent.get() - 1);
    }
    fn wrap_indent(&self) -> IRWriteRes {
        let indent = self.module_stat().indent.get();
        self.write_str("\n")?;
        for _ in 0..indent {
            self.write_str("    ")?;
        }
        Ok(())
    }

    fn write_str(&self, s: &str) -> IRWriteRes {
        write!(self.writer(), "{}", s).map_err(IRWriteErr::IO)
    }
    fn write_all(&self, buf: &[u8]) -> IRWriteRes {
        self.writer().write_all(buf).map_err(IRWriteErr::IO)
    }
    fn write_fmt(&self, args: std::fmt::Arguments<'_>) -> IRWriteRes {
        self.writer().write_fmt(args).map_err(IRWriteErr::IO)
    }

    fn fmt_type(&self, ty: ValTypeID) -> IRWriteRes {
        let module = self.get_module();
        let module_stat = self.module_stat();
        let tyname = module_stat.get_typename(ty, module);
        self.write_str(&tyname)
    }
    fn fmt_attr(&self, attr: &Attribute) -> IRWriteRes {
        match attr {
            Attribute::NoUndef => self.write_str("noundef"),
            Attribute::IntExt(iext) => self.write_str(iext.as_str()),
            Attribute::PtrReadOnly => self.write_str("readonly"),
            Attribute::PtrNoCapture => self.write_str("nocapture"),
            Attribute::FuncNoReturn => self.write_str("noreturn"),
            Attribute::FuncInline(inline) => self.write_str(inline.as_str()),
            Attribute::FuncAlignStack(log2) => write!(self, "alignstack({})", 1 << log2),
            Attribute::FuncPure => self.write_str("pure"),
            Attribute::ArgPtrTarget(target) => {
                let (name, ty) = match *target {
                    PtrArgTargetAttr::ByRef(ty) => ("byref", ty),
                    PtrArgTargetAttr::ByVal(ty) => ("byval", ty),
                    PtrArgTargetAttr::DynArray(ty) => ("elementtype", ty),
                };
                write!(self, "{}(", name)?;
                self.fmt_type(ty)?;
                write!(self, ")")
            }
            Attribute::ArgPtrDerefBytes(nbytes) => write!(self, "dereferenceable({})", nbytes),
        }
    }
    fn fmt_operand(&self, operand: ValueSSA) -> IRWriteRes {
        let operand = self.module_stat().map_value(operand, self.get_module());
        match operand {
            ValueSSA::None => self.write_str("none"),
            ValueSSA::ConstData(cdata) => write_impl::fmt_cdata(self, &cdata),
            ValueSSA::ConstExpr(expr_id) => self.fmt_expr(expr_id),
            ValueSSA::AggrZero(_) => self.write_str("zeroinitializer"),
            ValueSSA::FuncArg(func, arg) => {
                if self.get_numbers().is_some() {
                    write!(self, "%{arg}")
                } else {
                    let func_name = func.get_name(self.get_allocs());
                    write!(self, "%arg[{}]:{}", arg, func_name)
                }
            }
            ValueSSA::Global(global) => {
                let name = global.get_name(self.get_allocs());
                let registered = self.get_module().get_global_by_name(name);
                if registered != Some(global) {
                    write!(self, "@{}:unpinned", name)
                } else {
                    write!(self, "@{}", name)
                }
            }
            ValueSSA::Block(block_id) => {
                if let Some(numbering) = self.get_numbers()
                    && let Some(n) = numbering.block_get_number(block_id)
                {
                    write!(self, "%{n}")
                } else {
                    let indexed = block_id.get_entity_index(self.get_allocs());
                    write!(self, "%block:{indexed:x}")
                }
            }
            ValueSSA::Inst(inst_id) => {
                if let Some(numbering) = self.get_numbers()
                    && let Some(n) = numbering.inst_get_number(inst_id)
                {
                    write!(self, "%{n}")
                } else {
                    let indexed = inst_id.get_entity_index(self.get_allocs());
                    write!(self, "%inst:{indexed:x}")
                }
            }
        }
    }
    fn fmt_block_target(&self, block: Option<BlockID>) -> IRWriteRes {
        match block {
            Some(bb) => self.fmt_operand(ValueSSA::Block(bb)),
            None => self.write_str("%block:(nil)"),
        }
    }
    fn fmt_mapped_type_and_operand(&self, original: ValueSSA) -> IRWriteRes {
        let mapped = self.module_stat().map_value(original, self.get_module());
        let ty = mapped.get_valtype(self.get_allocs());
        self.fmt_type(ty)?;
        self.write_str(" ")?;
        self.fmt_operand(mapped)
    }

    fn fmt_expr(&self, expr_id: ExprID) -> IRWriteRes {
        let allocs = self.get_allocs();
        let tctx = self.get_tctx();
        if expr_id.is_zero_const(allocs) {
            return self.write_str("zeroinitializer");
        }
        if let Some(lit) = self
            .module_stat()
            .get_str_literal(expr_id, self.get_module())
        {
            return self.write_str(&lit);
        }

        let expr = expr_id.deref_ir(allocs);
        match expr {
            ExprObj::Array(arr) => write_impl::fmt_aggr_uses(self, &arr.elems, "[", "]"),
            ExprObj::DataArray(da) => write_impl::fmt_darr_aggr(self, &da.data),
            ExprObj::SplatArray(splat) => {
                write_impl::fmt_splat(self, splat.get_elem(allocs), splat.get_nelems())
            }
            ExprObj::KVArray(kv) => write_impl::fmt_kvarr_aggr(self, kv),
            ExprObj::Struct(struc) => {
                let structy = struc.structty;
                let (begin_s, end_s) =
                    if structy.is_packed(tctx) { ("<{", "}>") } else { ("{", "}") };
                write_impl::fmt_aggr_uses(self, &struc.fields, begin_s, end_s)
            }
            ExprObj::FixVec(fixvec) => write_impl::fmt_aggr_uses(self, &fixvec.elems, "<", ">"),
        }
    }

    fn fmtln_users(&self, traceable: &dyn ITraceableValue) -> IRWriteRes {
        self.write_str("; users = [")?;
        for (_, uobj) in traceable.user_iter(self.get_allocs()) {
            let kind = uobj.get_kind();
            let user = match uobj.user.get() {
                Some(u) => u.into_ir(),
                None => ValueSSA::None,
            };
            write!(self, "({kind:?}, user=")?;
            self.fmt_operand(user)?;
            self.write_str("), ")?;
        }
        self.write_str("]")?;
        self.wrap_indent()
    }
    fn fmtln_preds(&self, block: &BlockObj) -> IRWriteRes {
        self.write_str("; preds = [")?;
        for (_, pred) in block.get_preds().iter(&self.get_allocs().jts) {
            let kind = pred.get_kind();
            let Some(termi) = pred.terminator.get() else {
                write!(self, "(kind={kind:?} with no terminator), ")?;
                continue;
            };
            let block = termi.get_parent(self.get_allocs());
            write!(self, "(kind={kind:?}, from=")?;
            self.fmt_block_target(block)?;
            self.write_str("), ")?;
        }
        self.write_str("]")?;
        self.wrap_indent()
    }
}

mod write_impl {
    use super::*;
    use crate::typing::{FPKind, ScalarType};

    pub(super) fn fmt_cdata<'a>(
        write: &(impl WriteIR<'a> + ?Sized),
        data: &ConstData,
    ) -> IRWriteRes {
        match data {
            ConstData::Undef(_) => write.write_str("undef")?,
            ConstData::Zero(ty) => match ty {
                ScalarType::Ptr => write.write_str("null")?,
                ScalarType::Int(_) => write.write_str("0")?,
                ScalarType::Float(_) => write.write_str("0.0")?,
            },
            ConstData::PtrNull => write.write_str("null")?,
            ConstData::Int(apint) => {
                if apint.bits() == 1 {
                    write!(write, "{}", !apint.is_zero())?
                } else {
                    write!(write, "{}", apint.as_signed())?
                }
            }
            ConstData::Float(FPKind::Ieee32, fp) => write!(write, "{:.20e}", (*fp) as f32)?,
            ConstData::Float(FPKind::Ieee64, fp) => write!(write, "{:.20e}", fp)?,
        };
        Ok(())
    }
    fn fmt_aggr<'a>(
        write: &(impl WriteIR<'a> + ?Sized),
        elems: impl IntoIterator<Item = ValueSSA>,
        begin_s: &str,
        end_s: &str,
    ) -> IRWriteRes {
        write!(write, "{begin_s} ")?;
        let allocs = &write.get_module().allocs;
        for (i, elem) in elems.into_iter().enumerate() {
            if i > 0 {
                write.write_str(", ")?;
            }
            write.fmt_type(elem.get_valtype(allocs))?;
            write.write_str(" ")?;
            write.fmt_operand(elem)?;
        }
        write!(write, " {end_s}")?;
        Ok(())
    }
    pub(super) fn fmt_aggr_uses<'a>(
        write: &(impl WriteIR<'a> + ?Sized),
        uses: &[UseID],
        begin_s: &str,
        end_s: &str,
    ) -> IRWriteRes {
        let allocs = &write.get_module().allocs;
        fmt_aggr(
            write,
            uses.iter().map(|u| u.get_operand(allocs)),
            begin_s,
            end_s,
        )
    }
    pub(super) fn fmt_darr_aggr<'a>(
        write: &(impl WriteIR<'a> + ?Sized),
        data: &ConstArrayData,
    ) -> IRWriteRes {
        let data_iter = (0..data.len()).map(|n| data.index_get(n));
        fmt_aggr(write, data_iter, "[", "]")
    }
    pub(super) fn fmt_splat<'a>(
        write: &(impl WriteIR<'a> + ?Sized),
        elem: ValueSSA,
        nelems: usize,
    ) -> IRWriteRes {
        let data_iter = std::iter::repeat_n(elem, nelems);
        fmt_aggr(write, data_iter, "[", "]")
    }
    pub(super) fn fmt_kvarr_aggr<'a>(
        write: &(impl WriteIR<'a> + ?Sized),
        kv: &KVArrayExpr,
    ) -> IRWriteRes {
        let allocs = &write.get_module().allocs;
        let default_val = kv.get_default(allocs);
        write!(write, "sparse [ ")?;
        let mut first = true;
        for (idx, val, _) in kv.elem_iter(allocs) {
            if first {
                first = false;
            } else {
                write.write_str(", ")?;
            }
            write!(write, "[{}] = ", idx)?;
            write.fmt_type(kv.elemty)?;
            write.write_str(" ")?;
            write.fmt_operand(val)?;
        }

        if !first {
            write.write_str(", ")?;
        }
        write.write_str("..= ")?;
        write.fmt_type(kv.elemty)?;
        write.write_str(" ")?;
        write.fmt_operand(default_val)?;
        write!(write, " ]")?;
        Ok(())
    }
}

pub struct IRFuncWriter<'a> {
    pub writer: RefCell<&'a mut (dyn std::io::Write + 'a)>,
    pub module_stat: &'a IRWriteModuleStat,
    pub func_stat: Rc<IRWriteFuncStat>,
    pub module: &'a Module,
}
impl<'a> WriteIR<'a> for IRFuncWriter<'a> {
    fn get_module(&self) -> &'a Module {
        self.module
    }
    fn module_stat(&self) -> &IRWriteModuleStat {
        self.module_stat
    }
    fn writer(&self) -> RefMut<'_, dyn std::io::Write + 'a> {
        RefMut::map(self.writer.borrow_mut(), |x| *x)
    }
    fn get_numbers(&self) -> Option<&IRNumberValueMap> {
        Some(&self.func_stat.numbers)
    }
}
impl<'a> IRFuncWriter<'a> {
    pub fn new_full(
        writer: &'a mut (dyn std::io::Write + 'a),
        module_stat: &'a mut IRWriteModuleStat,
        module: &'a Module,
        func_id: FuncID,
    ) -> IRWriteRes<Self> {
        let ret = Self {
            writer: RefCell::new(writer),
            module_stat,
            module,
            func_stat: Rc::new(IRWriteFuncStat::new(&module.allocs, func_id)?),
        };
        Ok(ret)
    }
    pub fn from_stat(
        writer: &'a mut (dyn std::io::Write + 'a),
        module_stat: &'a mut IRWriteModuleStat,
        func_stat: Rc<IRWriteFuncStat>,
        module: &'a Module,
    ) -> Self {
        Self { writer: RefCell::new(writer), module_stat, func_stat, module }
    }

    pub fn func_id(&self) -> FuncID {
        self.func_stat.func_id
    }

    pub fn fmt_instid(&self, id: impl ISubInstID) -> IRWriteRes {
        id.raw_into().deref_ir(self.get_allocs()).format_ir(self)
    }
    pub fn fmt_block_body(&self, id: BlockID, obj: &BlockObj) -> IRWriteRes {
        let allocs = self.get_allocs();
        let option = self.module_stat().option;

        self.wrap_indent()?;
        if option.show_ptrid {
            let (index, gene) = {
                let indexed = id.to_indexed(allocs);
                (indexed.0.get_order(), indexed.0.get_generation())
            };
            let addr = id.0;
            write!(
                self,
                "; block addr={addr:p}, id=%block:{index:x}, gen={gene:x}"
            )?;
            self.wrap_indent()?;
        }
        if option.show_preds {
            self.fmtln_preds(obj)?;
        }
        if option.show_users {
            self.fmtln_users(obj)?;
        }
        let Some(number) = self.func_stat.numbers.block_get_number(id) else {
            return Err(IRWriteErr::JumpToNoTarget);
        };
        write!(self, "{number}:")?;

        self.inc_indent();
        for (inst_id, inst) in obj.insts_iter(allocs) {
            self.wrap_indent()?;
            if option.show_ptrid {
                let (index, gene) = {
                    let indexed = inst_id.to_indexed(allocs);
                    (indexed.0.get_order(), indexed.0.get_generation())
                };
                let addr = inst_id.0;
                write!(
                    self,
                    "; inst addr={addr:p}, id=%inst:{index:x}, gen={gene:x}"
                )?;
                self.wrap_indent()?;
            }
            if option.show_users {
                self.fmtln_users(inst)?;
            }
            if let Some(num) = self.func_stat.numbers.inst_get_number(inst_id)
                && inst.format_has_number()
            {
                write!(self, "%{num} = ")?;
            }
            inst.format_ir(self)?;
        }
        self.dec_indent();
        Ok(())
    }
}

pub trait IRFormatInst: ISubInst {
    fn format_has_number(&self) -> bool;

    fn format_ir(&self, write: &IRFuncWriter<'_>) -> IRWriteRes;
}

impl IRFormatInst for InstObj {
    fn format_has_number(&self) -> bool {
        !self.is_terminator() && self.get_valtype() != ValTypeID::Void
    }

    fn format_ir(&self, write: &IRFuncWriter<'_>) -> IRWriteRes {
        match self {
            InstObj::Unreachable(inst) => inst.format_ir(write),
            InstObj::GuideNode(_) => Ok(()),
            InstObj::PhiInstEnd(_) => {
                write!(write, "; ==== Phi Section End ====")
            }
            InstObj::Ret(ret_inst) => ret_inst.format_ir(write),
            InstObj::Jump(jump_inst) => jump_inst.format_ir(write),
            InstObj::Br(br_inst) => br_inst.format_ir(write),
            InstObj::Switch(switch_inst) => switch_inst.format_ir(write),
            InstObj::Alloca(alloca_inst) => alloca_inst.format_ir(write),
            InstObj::GEP(gepinst) => gepinst.format_ir(write),
            InstObj::Load(load_inst) => load_inst.format_ir(write),
            InstObj::Store(store_inst) => store_inst.format_ir(write),
            InstObj::AmoRmw(amo_rmw_inst) => amo_rmw_inst.format_ir(write),
            InstObj::BinOP(bin_opinst) => bin_opinst.format_ir(write),
            InstObj::Call(call_inst) => call_inst.format_ir(write),
            InstObj::Cast(cast_inst) => cast_inst.format_ir(write),
            InstObj::Cmp(cmp_inst) => cmp_inst.format_ir(write),
            InstObj::IndexExtract(index_extract_inst) => index_extract_inst.format_ir(write),
            InstObj::FieldExtract(field_extract_inst) => field_extract_inst.format_ir(write),
            InstObj::IndexInsert(index_insert_inst) => index_insert_inst.format_ir(write),
            InstObj::FieldInsert(field_insert_inst) => field_insert_inst.format_ir(write),
            InstObj::Phi(phi_inst) => phi_inst.format_ir(write),
            InstObj::Select(select_inst) => select_inst.format_ir(write),
        }
    }
}

impl IRFormatInst for UnreachableInst {
    fn format_has_number(&self) -> bool {
        false
    }
    fn format_ir(&self, write: &IRFuncWriter<'_>) -> IRWriteRes {
        write.write_str("unreachable")
    }
}

impl IRFormatInst for RetInst {
    fn format_has_number(&self) -> bool {
        false
    }

    fn format_ir(&self, write: &IRFuncWriter<'_>) -> IRWriteRes {
        write.write_str("ret ")?;
        let ret_type = self.get_valtype();
        if ret_type == ValTypeID::Void {
            write.write_str("void")?;
        } else {
            write.fmt_type(ret_type)?;
            write.write_str(" ")?;
            let allocs = write.get_allocs();
            write.fmt_operand(self.get_retval(allocs))?;
        }
        Ok(())
    }
}

impl IRFormatInst for JumpInst {
    fn format_has_number(&self) -> bool {
        false
    }

    fn format_ir(&self, write: &IRFuncWriter<'_>) -> IRWriteRes {
        write.write_str("br label ")?;
        write.fmt_block_target(self.get_target(write.get_allocs()))?;
        Ok(())
    }
}

impl IRFormatInst for BrInst {
    fn format_has_number(&self) -> bool {
        false
    }

    fn format_ir(&self, write: &IRFuncWriter<'_>) -> IRWriteRes {
        write.write_str("br i1 ")?;
        let allocs = write.get_allocs();
        write.fmt_operand(self.get_cond(allocs))?;
        write.write_str(", label ")?;
        write.fmt_block_target(self.get_then(allocs))?;
        write.write_str(", label ")?;
        write.fmt_block_target(self.get_else(allocs))?;
        Ok(())
    }
}

impl IRFormatInst for SwitchInst {
    fn format_has_number(&self) -> bool {
        false
    }

    fn format_ir(&self, write: &IRFuncWriter<'_>) -> IRWriteRes {
        let allocs = write.get_allocs();
        write.write_str("switch ")?;
        let cond = self.get_discrim(allocs);
        let cond_type = cond.get_valtype(allocs);
        write.fmt_type(cond_type)?;
        write.write_str(" ")?;
        write.fmt_operand(cond)?;
        write.write_str(", label ")?;
        write.fmt_block_target(self.get_default_bb(allocs))?;
        write.write_str(" [")?;
        write.inc_indent();
        for case in &*self.case_jts() {
            write.wrap_indent()?;
            let JumpTargetKind::SwitchCase(case_val) = case.get_kind(allocs) else {
                panic!("Invalid JumpTargetKind in Switch instruction");
            };
            write.fmt_type(cond_type)?;
            write.write_str(" ")?;
            write!(write, "{case_val}")?;
            write.write_str(", label ")?;
            write.fmt_block_target(case.get_block(allocs))?;
        }
        write.dec_indent();
        write.write_str(" ]")?;
        Ok(())
    }
}

impl IRFormatInst for AllocaInst {
    fn format_has_number(&self) -> bool {
        true
    }
    fn format_ir(&self, write: &IRFuncWriter<'_>) -> IRWriteRes {
        write.write_str("alloca ")?;
        write.fmt_type(self.pointee_ty)?;
        write.write_str(" ")?;
        write!(write, ", align {}", self.get_ptr_pointee_align())?;
        Ok(())
    }
}

impl IRFormatInst for GEPInst {
    fn format_has_number(&self) -> bool {
        true
    }
    fn format_ir(&self, write: &IRFuncWriter<'_>) -> IRWriteRes {
        write.write_str("getelementptr ")?;
        if self.get_inbounds() {
            write.write_str("inbounds ")?;
        }
        write.fmt_type(self.initial_ty)?;
        write.write_str(", ptr ")?;
        write.fmt_operand(self.get_base(write.get_allocs()))?;
        for &index_use in self.index_uses() {
            let index = index_use.get_operand(write.get_allocs());
            let index_ty = index.get_valtype(write.get_allocs());
            write.write_str(", ")?;
            write.fmt_type(index_ty)?;
            write.write_str(" ")?;
            write.fmt_operand(index)?;
        }
        Ok(())
    }
}

impl IRFormatInst for LoadInst {
    fn format_has_number(&self) -> bool {
        true
    }
    fn format_ir(&self, write: &IRFuncWriter<'_>) -> IRWriteRes {
        write.write_str("load ")?;
        let pointee_ty = self.get_valtype();
        write.fmt_type(pointee_ty)?;
        write.write_str(", ptr ")?;
        write.fmt_operand(self.get_source(write.get_allocs()))?;
        write!(write, ", align {}", self.get_operand_pointee_align())?;
        Ok(())
    }
}

impl IRFormatInst for StoreInst {
    fn format_has_number(&self) -> bool {
        false
    }
    fn format_ir(&self, write: &IRFuncWriter<'_>) -> IRWriteRes {
        write.write_str("store ")?;
        write.fmt_mapped_type_and_operand(self.get_source(write.get_allocs()))?;
        write.write_str(", ptr ")?;
        write.fmt_operand(self.get_target(write.get_allocs()))?;
        write!(write, ", align {}", self.get_operand_pointee_align())?;
        Ok(())
    }
}

impl IRFormatInst for AmoRmwInst {
    fn format_has_number(&self) -> bool {
        true
    }
    fn format_ir(&self, write: &IRFuncWriter<'_>) -> IRWriteRes {
        write.write_str("atomicrmw ")?;
        if self.is_volatile {
            write.write_str("volatile ")?;
        }
        write!(write, "{} ptr ", self.subop_name())?;
        write.fmt_operand(self.get_pointer(write.get_allocs()))?;
        write.write_str(", ")?;
        write.fmt_type(self.value_ty)?;
        write.write_str(" ")?;
        write.fmt_operand(self.get_value(write.get_allocs()))?;
        if self.scope != SyncScope::System {
            write!(write, " syncscope(\"{}\")", self.scope.as_str())?;
        }
        write!(write, " {}", self.ordering.as_str())?;
        if self.align_log2 > 0 {
            write!(write, ", align {}", 1 << self.align_log2)?;
        }
        Ok(())
    }
}

impl IRFormatInst for BinOPInst {
    fn format_has_number(&self) -> bool {
        true
    }
    fn format_ir(&self, write: &IRFuncWriter<'_>) -> IRWriteRes {
        let opcode = self.get_opcode().get_name();
        let flags = self.get_flags();
        if flags.is_empty() {
            write!(write, "{opcode} ")?;
        } else {
            write!(write, "{opcode} {flags} ")?;
        }
        write.fmt_type(self.get_valtype())?;
        write.write_str(" ")?;
        write.fmt_operand(self.get_lhs(write.get_allocs()))?;
        write.write_str(", ")?;
        write.fmt_operand(self.get_rhs(write.get_allocs()))?;
        Ok(())
    }
}

impl IRFormatInst for CallInst {
    fn format_has_number(&self) -> bool {
        true
    }
    fn format_ir(&self, write: &IRFuncWriter<'_>) -> IRWriteRes {
        let allocs = write.get_allocs();
        write.write_str("call ")?;
        write.fmt_type(self.get_valtype())?;
        write.write_str(if self.is_vararg { " (...) " } else { " " })?;
        write.fmt_operand(self.get_callee(allocs))?;
        write.write_str("(")?;
        for (i, arg_use) in self.arg_uses().iter().enumerate() {
            if i > 0 {
                write.write_str(", ")?;
            }
            let arg = arg_use.get_operand(allocs);
            let arg_ty = self
                .callee_ty
                .get_args(write.get_tctx())
                .get(i)
                .copied()
                .unwrap_or(arg.get_valtype(allocs));
            write.fmt_type(arg_ty)?;
            write.write_str(" ")?;
            write.fmt_operand(arg)?;
        }
        write.write_str(")")?;
        Ok(())
    }
}

impl IRFormatInst for CastInst {
    fn format_has_number(&self) -> bool {
        true
    }
    fn format_ir(&self, write: &IRFuncWriter<'_>) -> IRWriteRes {
        write!(write, "{} ", self.get_opcode().get_name())?;
        write.fmt_type(self.from_ty)?;
        write.write_str(" ")?;
        write.fmt_operand(self.get_from(write.get_allocs()))?;
        write.write_str(" to ")?;
        write.fmt_type(self.get_valtype())?;
        Ok(())
    }
}

impl IRFormatInst for CmpInst {
    fn format_has_number(&self) -> bool {
        true
    }
    fn format_ir(&self, write: &IRFuncWriter<'_>) -> IRWriteRes {
        let opcode = self.get_opcode().get_name();
        let cond = self.cond;
        write!(write, "{} {} ", opcode, cond)?;
        write.fmt_type(self.operand_ty)?;
        write.write_str(" ")?;
        write.fmt_operand(self.get_lhs(write.get_allocs()))?;
        write.write_str(", ")?;
        write.fmt_operand(self.get_rhs(write.get_allocs()))?;
        Ok(())
    }
}

impl IRFormatInst for IndexExtractInst {
    fn format_has_number(&self) -> bool {
        true
    }
    fn format_ir(&self, write: &IRFuncWriter<'_>) -> IRWriteRes {
        write.write_str("extractelement ")?;
        write.fmt_type(self.aggr_type.into_ir())?;
        write.write_str(" ")?;
        write.fmt_operand(self.get_aggr(write.get_allocs()))?;
        write.write_str(", ")?;
        let index = self.get_index(write.get_allocs());
        let index_ty = index.get_valtype(write.get_allocs());
        write.fmt_type(index_ty)?;
        write.write_str(" ")?;
        write.fmt_operand(index)?;
        Ok(())
    }
}

impl IRFormatInst for FieldExtractInst {
    fn format_has_number(&self) -> bool {
        true
    }
    fn format_ir(&self, write: &IRFuncWriter<'_>) -> IRWriteRes {
        write.write_str("extractvalue ")?;
        write.fmt_type(self.aggr_type.into_ir())?;
        write.write_str(" ")?;
        write.fmt_operand(self.get_aggr(write.get_allocs()))?;
        for &idx in self.get_field_indices() {
            write!(write, ", {idx}")?;
        }
        Ok(())
    }
}

impl IRFormatInst for IndexInsertInst {
    fn format_has_number(&self) -> bool {
        true
    }
    fn format_ir(&self, write: &IRFuncWriter<'_>) -> IRWriteRes {
        write.write_str("insertelement ")?;
        write.fmt_type(self.get_valtype())?;
        write.write_str(" ")?;
        write.fmt_operand(self.get_aggr(write.get_allocs()))?;
        write.write_str(", ")?;
        write.fmt_type(self.get_elem_type())?;
        write.write_str(" ")?;
        write.fmt_operand(self.get_elem(write.get_allocs()))?;
        write.write_str(", ")?;
        let index = self.get_index(write.get_allocs());
        let index_ty = index.get_valtype(write.get_allocs());
        write.fmt_type(index_ty)?;
        write.write_str(" ")?;
        write.fmt_operand(index)?;
        Ok(())
    }
}

impl IRFormatInst for FieldInsertInst {
    fn format_has_number(&self) -> bool {
        true
    }
    fn format_ir(&self, write: &IRFuncWriter<'_>) -> IRWriteRes {
        write.write_str("insertvalue ")?;
        write.fmt_type(self.get_valtype())?;
        write.write_str(" ")?;
        write.fmt_operand(self.get_aggr(write.get_allocs()))?;
        write.write_str(", ")?;
        write.fmt_type(self.get_elem_type())?;
        write.write_str(" ")?;
        write.fmt_operand(self.get_elem(write.get_allocs()))?;
        for &idx in self.get_field_indices() {
            write!(write, ", {}", idx)?;
        }
        Ok(())
    }
}

impl IRFormatInst for PhiInst {
    fn format_has_number(&self) -> bool {
        true
    }
    fn format_ir(&self, write: &IRFuncWriter<'_>) -> IRWriteRes {
        write.write_str("phi ")?;
        write.fmt_type(self.get_valtype())?;
        let mut first = true;
        let allocs = write.get_allocs();
        for &[uval, ublk] in self.incoming_uses().iter() {
            write.write_str(if first { " " } else { ", " })?;
            first = false;

            write.write_str("[")?;
            write.fmt_operand(uval.get_operand(allocs))?;
            write.write_str(", ")?;

            let ValueSSA::Block(bb) = ublk.get_operand(allocs) else {
                return Err(IRWriteErr::PhiIncomeNotBlock(ublk.get_operand(allocs)));
            };
            write.fmt_operand(ValueSSA::Block(bb))?;
            write.write_str("]")?;
        }
        Ok(())
    }
}

impl IRFormatInst for SelectInst {
    fn format_has_number(&self) -> bool {
        true
    }
    fn format_ir(&self, write: &IRFuncWriter<'_>) -> IRWriteRes {
        write.write_str("select ")?;
        write.fmt_type(self.get_valtype())?;
        write.write_str(", i1 ")?;
        write.fmt_operand(self.get_cond(write.get_allocs()))?;
        write.write_str(", ")?;
        write.fmt_operand(self.get_then(write.get_allocs()))?;
        write.write_str(", ")?;
        write.fmt_operand(self.get_else(write.get_allocs()))?;
        Ok(())
    }
}

pub struct IRWriter<'a> {
    pub writer: RefCell<&'a mut (dyn std::io::Write + 'a)>,
    pub module: &'a Module,
    pub module_stat: IRWriteModuleStat,
}

impl<'a> WriteIR<'a> for IRWriter<'a> {
    fn get_module(&self) -> &'a Module {
        self.module
    }
    fn module_stat(&self) -> &IRWriteModuleStat {
        &self.module_stat
    }
    fn writer(&self) -> RefMut<'_, dyn std::io::Write + 'a> {
        RefMut::map(self.writer.borrow_mut(), |x| *x)
    }
    fn get_numbers(&self) -> Option<&IRNumberValueMap> {
        None
    }
}

impl<'a> IRWriter<'a> {
    pub fn from_module(output: &'a mut dyn std::io::Write, module: &'a Module) -> Self {
        Self {
            writer: RefCell::new(output),
            module,
            module_stat: IRWriteModuleStat::default(),
        }
    }

    pub fn set_option(&mut self, option: IRWriteOption) {
        self.module_stat.option = option;
    }
    pub fn switch_to_func(&mut self, func: FuncID) -> IRWriteRes<IRFuncWriter<'_>> {
        let ret = IRFuncWriter {
            writer: RefCell::new(self.writer.get_mut()),
            module_stat: &self.module_stat,
            func_stat: Rc::new(IRWriteFuncStat::new(&self.module.allocs, func)?),
            module: self.module,
        };
        Ok(ret)
    }

    pub fn fmt_attr_set(&self, attrs: &AttrSet) -> IRWriteRes {
        for attr in attrs.iter() {
            self.write_str(" ")?;
            self.fmt_attr(&attr)?;
        }
        Ok(())
    }

    /// Syntax:
    ///
    /// ```llvm
    /// ; for external function
    /// declare [linkage] [ret_type] @function_name(<arg_ty0>, <arg_ty1>, [...]) [ret_type_attributes]
    ///
    /// ; for defined function
    /// define [linkage] [ret_type] @function_name(<arg_ty0> %0, <arg_ty1> %1, [...]) [ret_type_attributes] {
    ///     ; function body
    /// }
    /// ```
    pub fn fmt_func(&mut self, func_id: FuncID) -> IRWriteRes {
        let allocs = self.get_allocs();
        let func = func_id.deref_ir(allocs);
        self.fmt_func_header(func_id, func)?;
        if func.is_extern(allocs) {
            self.write_str(" ; extern")?;
            self.wrap_indent()?;
            return Ok(());
        }
        let entry = func.entry_unwrap();

        self.write_str(" {")?;
        let func_writer = self.switch_to_func(func_id)?;
        func_writer.fmt_block_body(entry, entry.deref_ir(allocs))?;
        for (bb_id, bb_obj) in func.block_iter(allocs) {
            if bb_id == entry {
                continue;
            }
            func_writer.fmt_block_body(bb_id, bb_obj)?;
        }
        drop(func_writer);

        self.wrap_indent()?;
        self.write_str("}")?;
        Ok(())
    }

    /// Syntax:
    ///
    /// ```llvm
    /// @global_name = [linkage] [type init_value | type], align <alignment>
    /// ```
    pub fn fmt_global_var(&mut self, gvar_id: GlobalVarID) -> IRWriteRes {
        let allocs = self.get_allocs();

        let gvar = gvar_id.deref_ir(allocs);
        let gid = gvar_id.raw_into();

        self.fmt_global_header_prefix(gid, "gvar")?;
        let name = if self.module.symbol_is_exported(gid) {
            gvar.clone_name()
        } else {
            format_smolstr!("{}:unpinned", gvar.get_name())
        };
        let prefix = gvar.get_linkage_prefix(allocs);

        write!(self, "@{name} = {prefix} ")?;
        if let Some(tls) = gvar.tls_model.get() {
            write!(self, "thread_local({}) ", tls.get_ir_text())?;
        }

        match gvar.get_init(allocs) {
            ValueSSA::None => self.fmt_type(gvar.common.content_ty)?,
            initval => self.fmt_mapped_type_and_operand(initval)?,
        };
        write!(self, ", align {}", gvar.get_ptr_pointee_align())
    }
    fn fmt_func_header(&self, func_id: FuncID, func: &FuncObj) -> IRWriteRes {
        let allocs = self.get_allocs();
        let gid = func_id.raw_into();

        self.fmt_global_header_prefix(gid, "func")?;
        self.write_str(func.get_linkage_prefix(allocs))?;
        self.fmt_attr_set(&func.attrs())?;
        self.write_str(" ")?;
        self.fmt_type(func.ret_type)?;
        self.write_str(" @")?;
        if self.module.symbol_is_exported(gid) {
            self.write_str(func.get_name())?;
        } else {
            write!(self, "{}:unpinned", func.get_name())?;
        }
        self.write_str("(")?;
        for arg in &func.args {
            if arg.index > 0 {
                self.write_str(", ")?;
            }
            let arg_ty = arg.ty;
            self.fmt_type(arg_ty)?;
            self.fmt_attr_set(&arg.attrs())?;
            if !func.is_extern(allocs) {
                write!(self, " %{}", arg.index)?;
            }
        }
        if func.is_vararg {
            let prompt = if func.args.is_empty() { "..." } else { ", ..." };
            self.write_str(prompt)?;
        }
        self.write_str(")")
    }
    fn fmt_global_header_prefix(&self, gid: GlobalID, kind: &str) -> IRWriteRes {
        let allocs = self.get_allocs();
        let options = self.module_stat().option;
        if options.show_ptrid {
            let addr = gid.0;
            let (index, gene) = {
                let indexed = gid.to_indexed(allocs);
                (indexed.0.get_order(), indexed.0.get_generation())
            };
            write!(
                self,
                "; {kind} addr={addr:p}, index={index:x}, gen={gene:x}"
            )?;
            self.wrap_indent()?;
        }
        if options.show_users {
            self.fmtln_users(gid.deref_ir(allocs))?;
        }
        Ok(())
    }

    pub fn fmt_module(&mut self) -> IRWriteRes {
        let mut aliases = Vec::new();
        self.get_tctx().foreach_aliases(|name, _, sid| {
            aliases.push((name.clone(), sid.into_ir()));
        });
        for (name, sid) in aliases {
            write!(self, "%{name} = type ").unwrap();
            self.fmt_type(sid)?;
            self.wrap_indent()?;
        }

        let allocs = self.get_allocs();
        let symbols = self.module.symbols.borrow();
        let globals = {
            let mut globs = Vec::from_iter(symbols.var_pool().iter().copied());
            globs.sort_by_key(|g| (g.get_kind(allocs), g.get_name(allocs)));
            globs
        };
        let funcs = {
            let mut funcs = Vec::from_iter(symbols.func_pool().iter().copied());
            funcs.sort_by_key(|f| (f.get_kind(allocs), f.get_name(allocs)));
            funcs
        };
        drop(symbols);

        for gid in globals {
            self.fmt_global_var(gid)?;
            self.wrap_indent()?;
        }
        for fid in funcs {
            self.fmt_func(fid)?;
            self.wrap_indent()?;
        }

        self.writer().flush()?;

        // 清理 LLVM 兼容模式下的临时表达式对象.
        // 这些临时分配在内存池中的表达式对象是垃圾对象, 需要手动释放掉.
        // 但 KVArrayExpr 不是, 因为它们还实际挂在 User 上当操作数.
        let mut inner = self.module_stat.inner.borrow_mut();
        for (_, value) in inner.llvm_mapping.kvarr.drain() {
            let ValueSSA::ConstExpr(expr) = value else {
                continue;
            };
            let Some(expr_obj) = expr.0.try_deref(&allocs.exprs) else {
                continue;
            };
            if expr_obj.obj_disposed() || expr_obj.has_users(allocs) {
                continue;
            }
            let _ = expr.dispose(allocs);
        }
        Ok(())
    }
}

use slab::Slab;

use crate::{
    base::{INullableValue, SlabListError, SlabListRes, SlabRef, SlabRefList},
    ir::{
        Attr, AttrList, AttrSet, BlockRef, GlobalData, GlobalDataCommon, GlobalKind, GlobalRef,
        IRAllocs, IRAllocsReadable, IRValueNumberMap, IRWriter, IReferenceValue, ISubValueSSA,
        ITraceableValue, Module, NumberOption, PtrStorage, PtrUser, UserList, ValueSSA,
        global::{ISubGlobal, Linkage},
    },
    typing::{FuncTypeRef, TypeContext, ValTypeID},
};
use std::cell::{Cell, RefCell, RefMut};

/// 函数存储接口，为存储函数值的对象提供类型信息访问能力
pub trait FuncStorage: PtrStorage {
    /// 获取存储的函数类型引用
    fn get_stored_func_type(&self) -> FuncTypeRef {
        match self.get_stored_pointee_type() {
            ValTypeID::Func(func_type) => func_type,
            _ => panic!("Expected a function type"),
        }
    }

    /// 获取函数的返回值类型
    fn get_return_type(&self, type_ctx: &TypeContext) -> ValTypeID {
        self.get_stored_func_type().ret_type(type_ctx)
    }

    /// 获取函数参数个数
    fn get_nargs(&self, type_ctx: &TypeContext) -> usize {
        self.get_stored_func_type().nargs(type_ctx)
    }

    /// 获取指定索引的参数类型
    fn try_get_arg_type(&self, type_ctx: &TypeContext, index: usize) -> Option<ValTypeID> {
        self.get_stored_func_type().try_get_arg(type_ctx, index)
    }
    fn get_arg_type(&self, type_ctx: &TypeContext, index: usize) -> ValTypeID {
        self.try_get_arg_type(type_ctx, index)
            .expect("Failed to get argument type")
    }

    /// 检查函数是否为可变参数函数
    fn is_vararg(&self, type_ctx: &TypeContext) -> bool {
        self.get_stored_func_type().is_vararg(type_ctx)
    }
}
/// 函数使用接口，为使用函数作为操作数的对象提供类型信息访问能力
pub trait FuncUser: PtrUser {
    /// 获取操作数的函数类型引用
    fn get_operand_func_type(&self) -> FuncTypeRef {
        match self.get_operand_pointee_type() {
            ValTypeID::Func(func_type) => func_type,
            _ => panic!("Expected a function type"),
        }
    }

    /// 获取操作数函数的返回值类型
    fn get_return_type(&self, type_ctx: &TypeContext) -> ValTypeID {
        self.get_operand_func_type().ret_type(type_ctx)
    }

    /// 获取操作数函数的参数个数
    fn get_nargs(&self, type_ctx: &TypeContext) -> usize {
        self.get_operand_func_type().nargs(type_ctx)
    }

    /// 获取操作数函数指定索引的参数类型
    fn try_get_arg_type(&self, type_ctx: &TypeContext, index: usize) -> Option<ValTypeID> {
        self.get_operand_func_type().try_get_arg(type_ctx, index)
    }
    fn get_arg_type(&self, type_ctx: &TypeContext, index: usize) -> ValTypeID {
        self.try_get_arg_type(type_ctx, index).unwrap()
    }
}

/// 函数对象，表示 IR 中的一个函数
///
/// 函数可以处于两种状态之一：
/// - 外部函数 (extern): 只有函数签名，没有函数体
/// - 已定义函数: 包含完整的基本块序列
#[derive(Debug)]
pub struct Func {
    /// 全局对象的通用数据 (名称、类型、可见性等)
    pub common: GlobalDataCommon,
    /// 函数参数列表，每个参数包含类型和使用者追踪信息
    pub args: Box<[FuncArg]>,
    /// 返回类型
    pub return_type: ValTypeID,
    /// 函数属性
    pub attrs: RefCell<AttrList>,
    /// 函数体的基本块列表 (空列表表示外部函数)
    pub(crate) body: SlabRefList<BlockRef>,
    /// 函数入口基本块的引用
    pub(crate) entry: Cell<BlockRef>,
}

impl ISubGlobal for Func {
    fn from_ir(data: &GlobalData) -> Option<&Self> {
        match data {
            GlobalData::Func(func) => Some(func),
            _ => None,
        }
    }
    fn into_ir(self) -> GlobalData {
        GlobalData::Func(self)
    }
    fn get_common(&self) -> &GlobalDataCommon {
        &self.common
    }
    fn common_mut(&mut self) -> &mut GlobalDataCommon {
        &mut self.common
    }
    fn is_readonly(&self) -> bool {
        true
    }
    fn is_extern(&self) -> bool {
        if !self.body.is_valid() {
            true
        } else if self.common.linkage.get() == Linkage::Extern {
            self.common.linkage.set(Linkage::DSOLocal);
            false
        } else {
            false
        }
    }
    fn set_linkage(&self, linkage: Linkage) {
        if linkage == Linkage::Extern && !self.is_extern() {
            panic!("Cannot set linkage to Extern for a defined function");
        }
        self.common.linkage.set(linkage);
    }
    fn get_kind(&self) -> GlobalKind {
        if self.is_extern() { GlobalKind::ExternFunc } else { GlobalKind::Func }
    }
    fn get_linkage(&self) -> Linkage {
        if self.is_extern() { Linkage::Extern } else { self.common.linkage.get() }
    }

    fn fmt_ir(&self, self_ref: GlobalRef, writer: &IRWriter) -> std::io::Result<()> {
        let _guard = writer.stat.hold_curr_func(FuncRef(self_ref));
        self.fmt_header(writer)?;
        if self.is_extern() {
            write!(writer, "; external function")?;
            return Ok(());
        }

        writer.numbering.replace(IRValueNumberMap::new(
            &writer.allocs,
            self_ref,
            NumberOption::ignore_all(),
        ));

        write!(writer, " {{")?;
        for (block_ref, _) in self.body.view(&writer.allocs.blocks) {
            block_ref.fmt_ir(writer)?;
        }
        write!(writer, "}}")?;
        Ok(())
    }
}

impl Func {
    fn fmt_header(&self, writer: &IRWriter) -> std::io::Result<()> {
        write!(
            writer,
            "{} ",
            self.get_kind().get_ir_prefix(self.get_linkage())
        )?;
        writer.write_type(self.return_type)?;
        write!(writer, " @{}", self.common.name)?;
        self.fmt_args(!self.is_extern(), writer)?;
        self.with_attrs(|attrs| {
            if attrs.is_empty() {
                Ok(())
            } else {
                writer.write_str(" ")?;
                attrs.fmt_ir(writer)
            }
        })
    }

    fn fmt_args(&self, writes_id: bool, writer: &IRWriter) -> std::io::Result<()> {
        writer.write_str("(")?;
        for (i, arg) in self.args.iter().enumerate() {
            if i > 0 {
                writer.write_str(", ")?;
            }
            writer.write_type(arg.ty)?;
            arg.with_attrs(|attrs| {
                if attrs.is_empty() {
                    Ok(())
                } else {
                    writer.write_str(" ")?;
                    attrs.fmt_ir(writer)
                }
            })?;
            if writes_id {
                write!(writer, " %{}", arg.index)?;
            }
        }
        if self.is_vararg(&writer.type_ctx) {
            if !self.args.is_empty() {
                writer.write_str(", ")?;
            }
            writer.write_str("...")?;
        }
        writer.write_str(")")?;
        Ok(())
    }
}

impl FuncStorage for Func {
    /* auto-implemented */
}

impl Func {
    fn create_extern(name: String, functy: FuncTypeRef, type_ctx: &TypeContext) -> Self {
        let content_ty = ValTypeID::Func(functy);
        let common = GlobalDataCommon::new(name, content_ty, 8);

        let args = {
            let mut args = Vec::with_capacity(functy.nargs(type_ctx));
            for (index, arg_ty) in functy.args(type_ctx).iter().enumerate() {
                args.push(FuncArg::new(index, *arg_ty));
            }
            args.into_boxed_slice()
        };
        let return_type = functy.ret_type(type_ctx);
        Func {
            common,
            args,
            attrs: RefCell::new(AttrList::default()),
            body: SlabRefList::new_guide(),
            entry: Cell::new(BlockRef::new_null()),
            return_type,
        }
    }

    /// 创建一个外部函数 (函数声明)
    ///
    /// # 参数
    /// - `functy`: 函数类型引用
    /// - `name`: 函数名称
    /// - `type_ctx`: 类型上下文，用于解析函数类型信息
    ///
    /// # 返回
    /// 返回一个只有声明没有定义的外部函数
    pub fn new_extern(
        functy: FuncTypeRef,
        name: impl Into<String>,
        type_ctx: &TypeContext,
    ) -> Self {
        Self::create_extern(name.into(), functy, type_ctx)
    }

    fn make_defined_with_unreachable(&mut self, allocs: &mut IRAllocs) {
        debug_assert!(
            self.is_extern(),
            "Function must be extern to define with unreachable"
        );
        self.common.linkage.set(Linkage::DSOLocal);

        // 将函数体设置为一个空的基本块列表
        self.body = SlabRefList::from_slab(&mut allocs.blocks);

        let unreachable_bb = BlockRef::new_unreachable(allocs);
        self.body
            .push_back_ref(&allocs.blocks, unreachable_bb)
            .expect("Failed to push unreachable block");
        assert!(
            unreachable_bb.has_terminator(allocs),
            "Failed to push unreachable block"
        );
        self.entry.set(unreachable_bb);
    }
    /// 创建一个包含单个 unreachable 基本块的函数
    ///
    /// 这是一个便利函数，用于创建一个"已定义"但永远不会被执行到的函数。
    /// 常用于占位符或测试场景。
    ///
    /// # 参数
    /// - `module`: IR 模块引用 (不可变借用)
    /// - `functy`: 函数类型引用
    /// - `name`: 函数名称
    ///
    /// # 返回
    /// 返回一个包含单个 unreachable 基本块的已定义函数
    pub fn new_with_unreachable(
        module: &mut Module,
        functy: FuncTypeRef,
        name: impl Into<String>,
    ) -> Self {
        let mut func = Self::create_extern(name.into(), functy, &module.type_ctx);
        let mut allocs = &mut module.allocs;
        func.make_defined_with_unreachable(&mut allocs);
        func
    }
}

impl Func {
    /// 向函数添加基本块引用 (使用分配器)
    ///
    /// 将一个已存在的基本块添加到函数的基本块列表中。
    /// 只有外部函数可以添加基本块 (用于将其转换为已定义函数)。
    ///
    /// # 参数
    /// - `allocs`: IR 分配器的可变引用
    /// - `block`: 要添加的基本块引用
    ///
    /// # 返回
    /// - `Ok(())`: 成功添加基本块
    /// - `Err(SlabListError)`: 添加失败 (通常是因为函数已经有定义)
    pub fn add_block_ref(&self, allocs: &impl IRAllocsReadable, block: BlockRef) -> SlabListRes {
        if !self.is_extern() {
            return Err(SlabListError::InvalidList);
        }
        self.body
            .push_back_ref(&allocs.get_allocs_ref().blocks, block)
    }

    /// 获取函数的入口基本块引用
    ///
    /// # 返回
    /// 函数入口基本块的引用。对于外部函数，返回空引用。
    pub fn get_entry(&self) -> BlockRef {
        self.entry.get()
    }

    /// 获取函数体的基本块列表
    ///
    /// # 返回
    /// - `Some(&SlabRefList<BlockRef>)`: 对于已定义函数，返回基本块列表的引用
    /// - `None`: 对于外部函数，返回 None
    pub fn get_body(&self) -> Option<&SlabRefList<BlockRef>> {
        if self.is_extern() { None } else { Some(&self.body) }
    }

    pub fn get_nargs(&self) -> usize {
        self.args.len()
    }

    /// 向函数添加属性
    pub fn add_attr(&self, attr: Attr) -> RefMut<'_, AttrList> {
        RefMut::map(self.attrs.borrow_mut(), |attrs| {
            if attr.is_func_attr() {
                attrs.add_attr(attr)
            } else {
                panic!("Attribute {attr:?} is not a function attribute");
            }
        })
    }

    /// 检查函数是否具有特定属性（包括继承）
    pub fn has_attr(&self, attr: &Attr, alloc: &Slab<AttrList>) -> bool {
        self.attrs.borrow().has_attr(attr, alloc)
    }

    /// 获取完整的合并属性集（包括继承）
    pub fn get_merged_attrs(&self, alloc: &Slab<AttrList>) -> AttrSet {
        self.attrs.borrow().get_merged_attrs(alloc)
    }

    /// 访问属性列表（只读）
    pub fn with_attrs<R>(&self, f: impl FnOnce(&AttrList) -> R) -> R {
        f(&self.attrs.borrow())
    }

    /// 访问属性列表（可变）
    pub fn with_attrs_mut<R>(&self, f: impl FnOnce(&mut AttrList) -> R) -> R {
        f(&mut self.attrs.borrow_mut())
    }
}

/// 函数参数表示
///
/// 封装了函数参数的类型信息、位置索引和使用者追踪信息，
/// 支持 Use-Def 链分析和优化。
#[derive(Debug)]
pub struct FuncArg {
    /// 参数的类型
    pub ty: ValTypeID,
    /// 参数在参数列表中的索引位置
    pub index: usize,
    /// 追踪使用此参数的指令列表 (Use-Def 链)
    pub users: UserList,
    /// 属性集合
    pub attrs: RefCell<AttrList>,
}

impl ITraceableValue for FuncArg {
    fn users(&self) -> &UserList {
        &self.users
    }

    fn has_single_reference_semantics(&self) -> bool {
        true
    }
}

impl FuncArg {
    pub fn new(index: usize, ty: ValTypeID) -> Self {
        FuncArg {
            index,
            ty,
            users: UserList::new_empty(),
            attrs: RefCell::new(AttrList::default()),
        }
    }

    // ================================
    // 参数属性操作便利方法
    // ================================

    /// 向参数添加属性
    pub fn add_attr(&self, attr: Attr) -> RefMut<'_, AttrList> {
        RefMut::map(self.attrs.borrow_mut(), |attrs| attrs.add_attr(attr))
    }

    /// 检查参数是否具有特定属性（包括继承）
    pub fn has_attr(&self, attr: &Attr, alloc: &Slab<AttrList>) -> bool {
        self.attrs.borrow().has_attr(attr, alloc)
    }

    /// 获取完整的合并属性集（包括继承）
    pub fn get_merged_attrs(&self, alloc: &Slab<AttrList>) -> AttrSet {
        self.attrs.borrow().get_merged_attrs(alloc)
    }

    /// 访问属性列表（只读）
    pub fn with_attrs<R>(&self, f: impl FnOnce(&AttrList) -> R) -> R {
        f(&self.attrs.borrow())
    }

    /// 访问属性列表（可变）
    pub fn with_attrs_mut<R>(&self, f: impl FnOnce(&mut AttrList) -> R) -> R {
        f(&mut self.attrs.borrow_mut())
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FuncArgRef(pub GlobalRef, pub usize);

impl IReferenceValue for FuncArgRef {
    type ValueDataT = FuncArg;

    fn to_value_data<'a>(self, allocs: &'a IRAllocs) -> &'a Self::ValueDataT
    where
        Self::ValueDataT: 'a,
    {
        self.to_data(&allocs.globals)
    }

    fn to_value_data_mut<'a>(self, allocs: &'a mut IRAllocs) -> &'a mut Self::ValueDataT
    where
        Self::ValueDataT: 'a,
    {
        self.to_data_mut(&mut allocs.globals)
    }
}

impl FuncArgRef {
    pub fn get_func(self) -> GlobalRef {
        self.0
    }
    pub fn get_index(self) -> usize {
        self.1
    }

    pub fn from_ir(value: ValueSSA) -> Option<Self> {
        match value {
            ValueSSA::FuncArg(func, index) => Some(Self(func, index as usize)),
            _ => None,
        }
    }
    pub fn into_ir(self) -> ValueSSA {
        ValueSSA::FuncArg(self.0, self.1 as u32)
    }

    pub fn to_data(self, alloc: &Slab<GlobalData>) -> &FuncArg {
        let func_data = self.0.to_data(alloc);
        match func_data {
            GlobalData::Func(func) => &func.args[self.1],
            _ => panic!("Expected a function data"),
        }
    }
    pub fn as_data(self, alloc: &Slab<GlobalData>) -> Option<&FuncArg> {
        let func_data = self.0.to_data(alloc);
        match func_data {
            GlobalData::Func(func) => func.args.get(self.1),
            _ => None,
        }
    }

    pub fn to_data_mut(self, alloc: &mut Slab<GlobalData>) -> &mut FuncArg {
        let func_data = self.0.to_data_mut(alloc);
        match func_data {
            GlobalData::Func(func) => &mut func.args[self.1],
            _ => panic!("Expected a function data"),
        }
    }
    pub fn as_data_mut(self, alloc: &mut Slab<GlobalData>) -> Option<&mut FuncArg> {
        let func_data = self.0.to_data_mut(alloc);
        match func_data {
            GlobalData::Func(func) => func.args.get_mut(self.1),
            _ => None,
        }
    }

    pub fn get_valtype(self, alloc: &Slab<GlobalData>) -> ValTypeID {
        self.to_data(alloc).ty
    }
    pub fn get_users(self, alloc: &Slab<GlobalData>) -> &UserList {
        self.to_data(alloc).users()
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct FuncRef(pub GlobalRef);

impl FuncRef {
    pub fn as_data<'a>(&self, alloc: &'a Slab<GlobalData>) -> Option<&'a Func> {
        let func_data = self.0.to_data(alloc);
        match func_data {
            GlobalData::Func(func) => Some(func),
            _ => None,
        }
    }
    pub fn as_data_mut(self, alloc: &mut Slab<GlobalData>) -> Option<&mut Func> {
        let func_data = self.0.to_data_mut(alloc);
        match func_data {
            GlobalData::Func(func) => Some(func),
            _ => None,
        }
    }
    pub fn to_data<'a>(&self, alloc: &'a Slab<GlobalData>) -> &'a Func {
        self.as_data(alloc).expect("Expected a function data")
    }
    pub fn to_data_mut(self, alloc: &mut Slab<GlobalData>) -> &mut Func {
        self.as_data_mut(alloc).expect("Expected a function data")
    }

    pub fn try_from_real(real: GlobalRef, alloc: &Slab<GlobalData>) -> Option<Self> {
        let func_data = real.to_data(alloc);
        match func_data {
            GlobalData::Func(_) => Some(Self(real)),
            _ => None,
        }
    }
    pub fn from_real(real: GlobalRef, alloc: &Slab<GlobalData>) -> Self {
        Self::try_from_real(real, alloc).expect("Expected a function data")
    }

    pub fn try_from_ir(value: ValueSSA, alloc: &Slab<GlobalData>) -> Option<Self> {
        if let ValueSSA::Global(gref) = value { Self::try_from_real(gref, alloc) } else { None }
    }
    pub fn from_ir(value: ValueSSA, alloc: &Slab<GlobalData>) -> Self {
        Self::try_from_ir(value, alloc).expect("Expected a function refrence")
    }
    pub fn into_ir(self) -> GlobalRef {
        self.0
    }

    pub fn args_from_alloc(self, alloc: &Slab<GlobalData>) -> &[FuncArg] {
        self.to_data(alloc).args.as_ref()
    }
    pub fn args(self, allocs: &impl IRAllocsReadable) -> &[FuncArg] {
        self.args_from_alloc(&allocs.get_allocs_ref().globals)
    }

    pub fn try_get_body_from_alloc(
        self,
        alloc: &Slab<GlobalData>,
    ) -> Option<&SlabRefList<BlockRef>> {
        self.to_data(alloc).get_body()
    }
    pub fn get_body_from_alloc(self, alloc: &Slab<GlobalData>) -> &SlabRefList<BlockRef> {
        self.try_get_body_from_alloc(alloc)
            .expect("Expected a function body")
    }
    pub fn try_get_body(self, allocs: &impl IRAllocsReadable) -> Option<&SlabRefList<BlockRef>> {
        self.try_get_body_from_alloc(&allocs.get_allocs_ref().globals)
    }
    pub fn get_body(self, allocs: &impl IRAllocsReadable) -> &SlabRefList<BlockRef> {
        self.get_body_from_alloc(&allocs.get_allocs_ref().globals)
    }

    pub fn try_get_entry_from_alloc(self, alloc: &Slab<GlobalData>) -> Option<BlockRef> {
        let func = self.to_data(alloc);
        if func.is_extern() {
            return None;
        }
        func.entry.get().to_option()
    }
    pub fn get_entry_from_alloc(self, alloc: &Slab<GlobalData>) -> BlockRef {
        self.try_get_entry_from_alloc(alloc)
            .expect("trying to get entry block from extern or broken function")
    }
    pub fn try_get_entry(self, allocs: &impl IRAllocsReadable) -> Option<BlockRef> {
        self.try_get_entry_from_alloc(&allocs.get_allocs_ref().globals)
    }
    pub fn get_entry(self, allocs: &impl IRAllocsReadable) -> BlockRef {
        self.get_entry_from_alloc(&allocs.get_allocs_ref().globals)
    }
}

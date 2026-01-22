use crate::{
    ir::{checking::basic_sanity_check, inst::*, *},
    typing::*,
};
use mtb_entity_slab::EntityListIter;
use std::{
    cell::RefCell,
    collections::{HashMap, VecDeque},
};

pub struct ModuleClone<'old> {
    pub old_module: &'old Module,
    pub new_module: Box<Module>,
    typing_mapping: RefCell<TypeMapping>,
    value_mapping: RefCell<ValueMapping>,
    process_queue: RefCell<ProcQueue>,
}

impl<'old> ModuleClone<'old> {
    pub fn new(old_module: &'old Module) -> Self {
        basic_sanity_check(old_module)
            .expect("ModuleClone::new: old_module failed basic sanity check");
        let arch = old_module.tctx.arch.clone();
        Self {
            old_module,
            new_module: Box::new(Module::new(arch, old_module.name.clone())),
            typing_mapping: RefCell::new(TypeMapping::new(&old_module.tctx)),
            value_mapping: RefCell::new(ValueMapping::default()),
            process_queue: RefCell::new(ProcQueue::default()),
        }
    }

    pub fn set_name(&mut self, name: impl Into<String>) -> &mut Self {
        self.new_module.name = name.into();
        self
    }

    pub fn clone(&mut self) {
        let symtab = self.old_module.symbols.borrow();
        let mut funcs = Vec::with_capacity(symtab.func_pool.len());
        let mut vars = Vec::with_capacity(symtab.var_pool.len());
        let old_allocs = &self.old_module.allocs;
        for &funcid in &symtab.func_pool {
            let is_exported = {
                let val = symtab.exported.get(funcid.get_name(old_allocs));
                val.cloned() == Some(funcid.raw_into())
            };
            funcs.push((funcid, is_exported));
        }

        for &varid in &symtab.var_pool {
            let is_exported = {
                let val = symtab.exported.get(varid.get_name(old_allocs));
                val.cloned() == Some(varid.raw_into())
            };
            vars.push((varid, is_exported));
        }

        for (varid, is_exported) in vars {
            self.clone_frame_of_global_var(varid, is_exported);
        }
        for (funcid, is_exported) in funcs {
            self.clone_frame_of_function(funcid, is_exported);
        }
        self.map_all_uses();
    }

    pub fn clone_and_release(mut self) -> Module {
        self.clone();
        *self.new_module
    }
    pub fn clone_and_release_box(mut self) -> Box<Module> {
        self.clone();
        self.new_module
    }

    pub fn clone_type<T: IValType>(&self, old_ty: T) -> T {
        let mut mapping = self.typing_mapping.borrow_mut();
        let mut helper = TypeMappingHelper {
            old_tctx: &self.old_module.tctx,
            new_tctx: &self.new_module.tctx,
            mapping: &mut mapping,
        };
        let ret = helper.map_type(old_ty.into_ir());
        T::from_ir(ret)
    }
}

struct TypeMapping {
    arrays: Box<[u32]>,
    structs: Box<[u32]>,
    aliases: Box<[u32]>,
    funcs: Box<[u32]>,
}

impl TypeMapping {
    fn new(old_tctx: &TypeContext) -> Self {
        let tyallocs = old_tctx.allocs.borrow();
        Self {
            arrays: Self::nulled_box(tyallocs.arrays.capacity()),
            structs: Self::nulled_box(tyallocs.structs.capacity()),
            aliases: Self::nulled_box(tyallocs.aliases.capacity()),
            funcs: Self::nulled_box(tyallocs.funcs.capacity()),
        }
    }
    fn nulled_box(size: usize) -> Box<[u32]> {
        let mut vec = Vec::with_capacity(size);
        vec.resize(size, u32::MAX);
        vec.into_boxed_slice()
    }
}

struct TypeMappingHelper<'ir> {
    old_tctx: &'ir TypeContext,
    new_tctx: &'ir TypeContext,
    mapping: &'ir mut TypeMapping,
}
impl<'ir> TypeMappingHelper<'ir> {
    const UNMAPPED: u32 = u32::MAX;

    fn map_type(&mut self, old_ty: ValTypeID) -> ValTypeID {
        match old_ty {
            ValTypeID::Array(a) => self.map_array_type(a).into_ir(),
            ValTypeID::Struct(s) => self.map_struct_type(s).into_ir(),
            ValTypeID::StructAlias(sa) => self.map_struct_alias(sa).into_ir(),
            ValTypeID::Func(f) => self.map_func_type(f).into_ir(),
            ty => ty,
        }
    }
    fn map_array_type(&mut self, old_arr: ArrayTypeID) -> ArrayTypeID {
        let idx = old_arr.0 as usize;
        if self.mapping.arrays[idx] != Self::UNMAPPED {
            return ArrayTypeID(self.mapping.arrays[idx]);
        }
        let arr = old_arr.deref_ir(self.old_tctx);
        let ArrayTypeObj { elemty, nelems, .. } = &*arr;
        let elemty = self.map_type(*elemty);
        let new_arrty = ArrayTypeID::new(self.new_tctx, elemty, *nelems);
        self.mapping.arrays[idx] = new_arrty.0;
        new_arrty
    }
    fn map_struct_type(&mut self, old_s: StructTypeID) -> StructTypeID {
        let idx = old_s.0 as usize;
        if self.mapping.structs[idx] != Self::UNMAPPED {
            return StructTypeID(self.mapping.structs[idx]);
        }
        let struc = old_s.deref_ir(self.old_tctx);
        let fields = {
            let mut fields = struc.fields.clone();
            for field in &mut fields {
                *field = self.map_type(*field);
            }
            fields
        };
        let new_struc_ty = StructTypeID::new(self.new_tctx, struc.packed, fields);
        self.mapping.structs[idx] = new_struc_ty.0;
        new_struc_ty
    }
    fn map_struct_alias(&mut self, old_sa: StructAliasID) -> StructAliasID {
        let idx = old_sa.0 as usize;
        if self.mapping.aliases[idx] != Self::UNMAPPED {
            return StructAliasID(self.mapping.aliases[idx]);
        }
        let alias = old_sa.deref_ir(self.old_tctx);
        let StructAliasObj { name, aliasee } = &*alias;
        let aliasee = self.map_struct_type(*aliasee);
        let new_aliases = self.new_tctx.set_alias(name.clone(), aliasee);
        self.mapping.aliases[idx] = new_aliases.0;
        new_aliases
    }
    fn map_func_type(&mut self, old_f: FuncTypeID) -> FuncTypeID {
        let idx = old_f.0 as usize;
        if self.mapping.funcs[idx] != Self::UNMAPPED {
            return FuncTypeID(self.mapping.funcs[idx]);
        }
        let func = old_f.deref_ir(self.old_tctx);
        let args = {
            let mut args = func.args.clone();
            for arg in &mut args {
                *arg = self.map_type(*arg);
            }
            args
        };
        let ret_ty = self.map_type(func.ret_type);
        let new_func_ty = FuncTypeID::new(self.new_tctx, ret_ty, func.is_vararg, args);
        self.mapping.funcs[idx] = new_func_ty.0;
        new_func_ty
    }
}

#[derive(Default)]
struct ValueMapping {
    exprs: HashMap<ExprID, ExprID>,
    insts: HashMap<InstID, InstID>,
    blocks: HashMap<BlockID, BlockID>,
    globals: HashMap<GlobalID, GlobalID>,
}

struct UseProc {
    new_use: UseID,
    old_val: ValueSSA,
}
struct BlockProc {
    old_block: BlockID,
    new_block: BlockID,
}

#[derive(Default)]
struct ProcQueue {
    uses: VecDeque<UseProc>,
}

impl<'old> ModuleClone<'old> {
    fn push_use(&self, new_use: UseID, old_val: ValueSSA) {
        let mut queues = self.process_queue.borrow_mut();
        queues.uses.push_back(UseProc { new_use, old_val });
    }
    fn map_get_block(&self, old_bb: BlockID) -> BlockID {
        self.value_mapping.borrow().blocks[&old_bb]
    }
    fn map_value_to_const(&self, value: ValueSSA) -> ValueSSA {
        let cdata = match value {
            ValueSSA::None => return ValueSSA::None,
            ValueSSA::ConstData(d) => d,
            val => {
                let old_ty = val.get_valtype(&self.old_module.allocs);
                let new_ty = self.clone_type(old_ty);
                return ValueSSA::new_zero(new_ty).unwrap_or(ValueSSA::None);
            }
        };
        self.map_get_constdata(cdata).into_ir()
    }
    fn map_get_constdata(&self, cdata: ConstData) -> ConstData {
        match cdata {
            ConstData::Undef(ty) => ConstData::Undef(self.clone_type(ty)),
            ConstData::Zero(scal) => ConstData::Zero(scal),
            ConstData::PtrNull(ty) => ConstData::PtrNull(self.clone_type(ty)),
            ConstData::Int(apint) => ConstData::Int(apint),
            ConstData::Float(fk, fv) => ConstData::Float(fk, fv),
        }
    }
    fn insert_global(&self, old_global: impl ISubGlobalID, new_global: impl ISubGlobalID) {
        self.value_mapping
            .borrow_mut()
            .globals
            .insert(old_global.raw_into(), new_global.raw_into());
    }
    fn insert_block(&self, old_bb: BlockID, new_bb: BlockID) {
        self.value_mapping
            .borrow_mut()
            .blocks
            .insert(old_bb, new_bb);
    }
    fn insert_inst(&self, old_inst: InstID, new_inst: InstID) {
        self.value_mapping
            .borrow_mut()
            .insts
            .insert(old_inst, new_inst);
    }
    fn insert_expr(&self, old_expr: ExprID, new_expr: impl ISubExprID) {
        self.value_mapping
            .borrow_mut()
            .exprs
            .insert(old_expr, new_expr.raw_into());
    }

    fn clone_frame_of_global_var(&self, old_varid: GlobalVarID, is_exported: bool) {
        let old_allocs = &self.old_module.allocs;
        let old_var = old_varid.deref_ir(old_allocs);
        let old_common = &old_var.common;
        let new_ty = self.clone_type(old_common.content_ty);
        let mut builder = GlobalVar::builder(old_var.get_name(), new_ty);
        builder
            .align_log(old_common.content_align_log)
            .linkage(old_common.back_linkage.get())
            .readonly(old_var.is_readonly());

        if old_var.is_extern(old_allocs) {
            builder.make_extern();
        } else {
            builder.initval(ValueSSA::new_zero(new_ty).unwrap());
        }
        let newvar_id = if is_exported {
            builder.build_id(&self.new_module).unwrap()
        } else {
            builder.build_pinned(&self.new_module)
        };

        self.insert_global(old_varid, newvar_id);
        let new_allocs = &self.new_module.allocs;
        self.push_use(newvar_id.init_use(new_allocs), old_var.get_init(old_allocs));
    }

    fn clone_frame_of_function(&self, old_funcid: FuncID, is_exported: bool) {
        let old_allocs = &self.old_module.allocs;
        let old_func = old_funcid.deref_ir(old_allocs);
        let old_common = &old_func.common;
        let ValTypeID::Func(new_ty) = self.clone_type(old_common.content_ty) else {
            panic!("Internal error: Expected FuncTypeID when cloning function");
        };
        let func_attr = self.clone_attr(&old_func.attrs());
        let arg_attrs = {
            let mut arg_attrs = Vec::with_capacity(old_func.args.len());
            for arg in &old_func.args {
                arg_attrs.push(self.clone_attr(&arg.attrs()));
            }
            arg_attrs.into_boxed_slice()
        };
        let mut builder = FuncObj::builder(&self.new_module.tctx, old_func.get_name(), new_ty);
        builder.linkage(old_common.back_linkage.get());
        builder.attrs = func_attr;
        builder.arg_attrs = arg_attrs;

        if old_func.is_extern(old_allocs) {
            builder.make_extern();
        } else {
            // 占位, 之后翻译指令的时候会替换掉 Unreachable 指令
            builder.terminate_mode(FuncTerminateMode::Unreachable);
        }
        let new_funcid = if is_exported {
            builder.build_id(&self.new_module).unwrap()
        } else {
            builder.build_pinned(&self.new_module)
        };

        self.insert_global(old_funcid, new_funcid);
        if old_func.is_extern(old_allocs) {
            return;
        }
        self.clone_frame_of_func_body(old_funcid, new_funcid);
    }

    fn clone_attr(&self, attrs: &AttrSet) -> AttrSet {
        use crate::ir::{Attribute, PtrArgTargetAttr};
        let mut new_attrs = AttrSet::default();
        for attr in attrs.iter() {
            let new_attr = match attr {
                Attribute::ArgPtrTarget(parg) => {
                    let parg = match parg {
                        PtrArgTargetAttr::ByRef(t) => PtrArgTargetAttr::ByRef(self.clone_type(t)),
                        PtrArgTargetAttr::ByVal(t) => PtrArgTargetAttr::ByVal(self.clone_type(t)),
                        PtrArgTargetAttr::DynArray(t) => {
                            PtrArgTargetAttr::DynArray(self.clone_type(t))
                        }
                    };
                    Attribute::ArgPtrTarget(parg)
                }
                attr => attr,
            };
            new_attrs.set_attr(new_attr);
        }
        new_attrs
    }

    fn clone_frame_of_func_body(&self, old_func: FuncID, new_func: FuncID) {
        let old_func = old_func.deref_ir(&self.old_module.allocs);
        let new_func = new_func.deref_ir(&self.new_module.allocs);
        let (old_body, old_entry, len) = match &old_func.body {
            None => panic!("Internal error: Expected function body when cloning function"),
            Some(body) => {
                let old_allocs = &self.old_module.allocs;
                let range = body.blocks.get_range(&old_allocs.blocks);
                (range, body.entry, body.blocks.len())
            }
        };
        let mut blocks = Vec::with_capacity(len);

        let Some(new_body) = &new_func.body else {
            let name = new_func.get_name();
            panic!("Internal error: Expected function body when cloning function {name}")
        };
        blocks.push(BlockProc { old_block: old_entry, new_block: new_body.entry });
        let old_alloc_bb = &self.old_module.allocs.blocks;
        for (old_block, _) in EntityListIter::new(old_body, old_alloc_bb) {
            if old_block == old_entry {
                continue;
            }
            let new_allocs = &self.new_module.allocs;
            let new_unreach = UnreachableInstID::new(new_allocs);
            let new_block = BlockID::new_with_terminator(new_allocs, new_unreach);
            blocks.push(BlockProc { old_block, new_block });

            new_body
                .blocks
                .push_back_id(new_block, &new_allocs.blocks)
                .expect("Internal error: list broken");
            self.insert_block(old_block, new_block);
        }
        for BlockProc { old_block, new_block } in blocks {
            self.clone_frame_of_block(old_block, new_block);
        }
    }

    fn clone_frame_of_block(&self, old_bb: BlockID, new_bb: BlockID) {
        let old_allocs = &self.old_module.allocs;
        let new_allocs = &self.new_module.allocs;
        let mut builder = IRBuilder::new(self.new_module.as_ref());
        builder.set_focus(IRFocus::Block(new_bb));
        for (old_inst, old_iobj) in old_bb.insts_iter(old_allocs) {
            let new_inst: InstID = match old_iobj {
                InstObj::GuideNode(_) | InstObj::PhiInstEnd(_) => continue,
                InstObj::Unreachable(_) => UnreachableInstID::new(new_allocs).raw_into(),
                InstObj::Ret(inst) => self.clone_ret(inst),
                InstObj::Jump(inst) => self.clone_jump(inst),
                InstObj::Br(inst) => self.clone_br(inst),
                InstObj::Switch(inst) => self.clone_switch(inst),
                InstObj::Alloca(inst) => {
                    let pointee_ty = self.clone_type(inst.pointee_ty);
                    AllocaInstID::new(new_allocs, pointee_ty, inst.align_log2).raw_into()
                }
                InstObj::GEP(inst) => self.clone_gep_inst(inst),
                InstObj::Load(inst) => self.clone_load_inst(inst),
                InstObj::Store(inst) => self.clone_store_inst(inst),
                InstObj::AmoRmw(inst) => self.clone_amormw_inst(inst),
                InstObj::BinOP(inst) => self.clone_binop_inst(inst),
                InstObj::Call(inst) => self.clone_call_inst(inst),
                InstObj::Cast(inst) => self.clone_cast_inst(inst),
                InstObj::Cmp(inst) => self.clone_cmp_inst(inst),
                InstObj::IndexExtract(inst) => self.clone_index_extract_inst(inst),
                InstObj::FieldExtract(inst) => self.clone_field_extract_inst(inst),
                InstObj::IndexInsert(inst) => self.clone_index_insert_inst(inst),
                InstObj::FieldInsert(inst) => self.clone_field_insert_inst(inst),
                InstObj::Phi(inst) => self.clone_phi_inst(inst),
                InstObj::Select(inst) => self.clone_select_inst(inst),
            };

            self.insert_inst(old_inst, new_inst);
            builder
                .insert_inst(new_inst)
                .expect("Inst insertion error in module cloning");
        }
    }

    fn clone_ret(&self, inst: &RetInst) -> InstID {
        let old_allocs = &self.old_module.allocs;
        let new_allocs = &self.new_module.allocs;
        let retty = inst.get_valtype();
        let ret_inst = RetInstID::new_uninit(new_allocs, inst.get_valtype());
        if retty != ValTypeID::Void {
            let old_retval = inst.get_retval(old_allocs);
            self.push_use(ret_inst.retval_use(new_allocs), old_retval);
        }
        ret_inst.raw_into()
    }
    fn clone_jump(&self, inst: &JumpInst) -> InstID {
        let new_allocs = &self.new_module.allocs;
        let old_allocs = &self.old_module.allocs;
        let jump = JumpInstID::new_uninit(new_allocs);
        let new_bb = inst
            .get_target(old_allocs)
            .map(|old_bb| self.map_get_block(old_bb));
        if let Some(new_bb) = new_bb {
            jump.set_target(new_allocs, new_bb);
        }
        jump.raw_into()
    }
    fn clone_br(&self, inst: &BrInst) -> InstID {
        let old_allocs = &self.old_module.allocs;
        let new_allocs = &self.new_module.allocs;
        let old_cond = inst.get_cond(old_allocs);
        let then_bb = inst
            .get_then(old_allocs)
            .map(|old_bb| self.map_get_block(old_bb));
        let else_bb = inst
            .get_else(old_allocs)
            .map(|old_bb| self.map_get_block(old_bb));
        let br_inst = BrInstID::new_uninit(new_allocs);
        if let Some(then_bb) = then_bb {
            br_inst.set_then(new_allocs, then_bb);
        }
        if let Some(else_bb) = else_bb {
            br_inst.set_else(new_allocs, else_bb);
        }
        self.push_use(br_inst.cond_use(new_allocs), old_cond);
        br_inst.raw_into()
    }
    fn clone_switch(&self, old_inst: &SwitchInst) -> InstID {
        let old_allocs = &self.old_module.allocs;
        let new_allocs = &self.new_module.allocs;
        let default_old = old_inst
            .get_default_bb(old_allocs)
            .expect("empty bb for switch");

        let mut builder = SwitchInst::builder(old_inst.discrim_ty);
        let placeholder = ValueSSA::new_zero(old_inst.discrim_ty.into_ir()).unwrap();

        let default_new = self.map_get_block(default_old);
        builder.discrim(placeholder).default_bb(default_new).cases(
            old_inst.cases_iter(old_allocs).map(|(_, val, old_bb)| {
                let old_bb = old_bb.expect("switch case without block while cloning");
                let new_bb = self.map_get_block(old_bb);
                (val, new_bb)
            }),
        );
        let switch_id = builder.build_id(new_allocs);
        self.push_use(
            switch_id.discrim_use(new_allocs),
            old_inst.get_discrim(old_allocs),
        );
        switch_id.raw_into()
    }

    fn clone_gep_inst(&self, old_inst: &GEPInst) -> InstID {
        // 这里要特别注意, GEP 的类型层级和索引层级是关联在一起的,
        // 如果索引像其他指令类一样不初始化的话会直接引发索引错误, 导致 clone 失败.
        let old_allocs = &self.old_module.allocs;
        let new_allocs = &self.new_module.allocs;

        let final_ty = self.clone_type(old_inst.final_ty);
        let initial_ty = self.clone_type(old_inst.initial_ty);

        let new_tctx = &self.new_module.tctx;
        let mut gep_builder = GEPInst::builder(new_tctx, new_allocs, initial_ty);
        for &uindex in old_inst.index_uses() {
            // 索引重置: 把常量索引透传过去, 非常量索引则设为 0.
            // 前者保证不会引发索引错误, 后者防止跨分配器传递引用、引发数据失效
            let old_index = uindex.get_operand(old_allocs);
            let new_index = self.map_value_to_const(old_index);
            gep_builder.add_index(new_index);
        }
        let gep = gep_builder
            .align_log2(old_inst.align_log2)
            .inbounds(old_inst.inbounds_mark.get())
            .build_id();
        assert_eq!(final_ty, gep.get_final_ty(new_allocs));

        let old_uses = old_inst.operands_iter();
        let new_uses = gep.deref_ir(new_allocs).operands_iter();
        for (new_use, old_use) in new_uses.zip(old_uses) {
            let old_val = old_use.get_operand(old_allocs);
            self.push_use(new_use, old_val);
        }
        gep.raw_into()
    }

    fn clone_load_inst(&self, old_inst: &LoadInst) -> InstID {
        let old_allocs = &self.old_module.allocs;
        let new_allocs = &self.new_module.allocs;

        let pointee_ty = self.clone_type(old_inst.get_valtype());
        let load_inst = LoadInstID::new_uninit(new_allocs, pointee_ty, old_inst.align_log2);

        let old_source = old_inst.get_source(old_allocs);
        self.push_use(load_inst.source_use(new_allocs), old_source);
        load_inst.raw_into()
    }
    fn clone_store_inst(&self, old_inst: &StoreInst) -> InstID {
        let old_allocs = &self.old_module.allocs;
        let new_allocs = &self.new_module.allocs;

        let source_ty = self.clone_type(old_inst.source_ty);
        let store_inst = StoreInstID::new_uninit(new_allocs, source_ty, old_inst.align_log2);

        let old_src = old_inst.get_source(old_allocs);
        let old_dst = old_inst.get_target(old_allocs);
        self.push_use(store_inst.source_use(new_allocs), old_src);
        self.push_use(store_inst.target_use(new_allocs), old_dst);
        store_inst.raw_into()
    }
    fn clone_amormw_inst(&self, old_inst: &AmoRmwInst) -> InstID {
        let old_allocs = &self.old_module.allocs;
        let new_allocs = &self.new_module.allocs;

        let pointee_ty = self.clone_type(old_inst.value_ty);
        let amormw = AmoRmwInst::builder(old_inst.get_opcode(), pointee_ty)
            .align_log2(old_inst.align_log2)
            .is_volatile(old_inst.is_volatile)
            .ordering(old_inst.ordering)
            .scope(old_inst.scope)
            .build_id(new_allocs);

        let old_ptr_use = old_inst.pointer_use();
        let old_val_use = old_inst.value_use();
        let new_ptr_use = amormw.pointer_use(new_allocs);
        let new_val_use = amormw.value_use(new_allocs);

        self.push_use(new_ptr_use, old_ptr_use.get_operand(old_allocs));
        self.push_use(new_val_use, old_val_use.get_operand(old_allocs));
        amormw.raw_into()
    }

    fn clone_binop_inst(&self, old_inst: &BinOPInst) -> InstID {
        let old_allocs = &self.old_module.allocs;
        let new_allocs = &self.new_module.allocs;

        let new_ty = self.clone_type(old_inst.get_valtype());
        let flag = old_inst.get_flags();
        let opcode = old_inst.get_opcode();

        let new_binop = BinOPInstID::new_uninit(new_allocs, opcode, new_ty);
        new_binop.set_flags(new_allocs, flag);

        let old_lhs = old_inst.get_lhs(old_allocs);
        let old_rhs = old_inst.get_rhs(old_allocs);

        self.push_use(new_binop.lhs_use(new_allocs), old_lhs);
        self.push_use(new_binop.rhs_use(new_allocs), old_rhs);
        new_binop.raw_into()
    }

    fn clone_call_inst(&self, old_inst: &CallInst) -> InstID {
        let old_allocs = &self.old_module.allocs;
        let new_allocs = &self.new_module.allocs;

        let callee_ty = self.clone_type(old_inst.callee_ty.into_ir());
        let callee_ty = FuncTypeID::from_ir(callee_ty);

        let new_tctx = &self.new_module.tctx;
        let mut call_builder = CallInst::builder(new_tctx, callee_ty);
        call_builder.resize_nargs(old_inst.arg_uses().len() as u32);
        let call_inst = call_builder
            .is_tail_call(old_inst.is_tail_call.get())
            .builder_uninit(true)
            .build_id(new_allocs);

        let old_callee = old_inst.get_callee(old_allocs);
        self.push_use(call_inst.callee_use(new_allocs), old_callee);
        for i in 0..old_inst.arg_uses().len() {
            let old_arg = old_inst.arg_uses()[i].get_operand(old_allocs);
            let new_arg_use = call_inst.arg_uses(new_allocs)[i];
            self.push_use(new_arg_use, old_arg);
        }
        call_inst.raw_into()
    }

    fn clone_cast_inst(&self, old_inst: &CastInst) -> InstID {
        let old_allocs = &self.old_module.allocs;
        let new_allocs = &self.new_module.allocs;

        let from_ty = self.clone_type(old_inst.from_ty);
        let into_ty = self.clone_type(old_inst.get_valtype());
        let opcode = old_inst.get_opcode();

        let cast_inst = CastInstID::new_uninit(new_allocs, opcode, from_ty, into_ty);

        let old_from = old_inst.get_from(old_allocs);
        self.push_use(cast_inst.from_use(new_allocs), old_from);
        cast_inst.raw_into()
    }

    fn clone_cmp_inst(&self, old_inst: &CmpInst) -> InstID {
        let old_allocs = &self.old_module.allocs;
        let new_allocs = &self.new_module.allocs;

        let operand_ty = self.clone_type(old_inst.operand_ty);
        let opcode = old_inst.get_opcode();
        let cond = old_inst.cond;

        let cmp_inst = CmpInstID::new_uninit(new_allocs, opcode, cond, operand_ty);

        let old_lhs = old_inst.get_lhs(old_allocs);
        let old_rhs = old_inst.get_rhs(old_allocs);

        self.push_use(cmp_inst.lhs_use(new_allocs), old_lhs);
        self.push_use(cmp_inst.rhs_use(new_allocs), old_rhs);
        cmp_inst.raw_into()
    }

    fn clone_index_extract_inst(&self, old_inst: &IndexExtractInst) -> InstID {
        let old_allocs = &self.old_module.allocs;
        let new_allocs = &self.new_module.allocs;

        let aggr_ty = self.clone_type(old_inst.aggr_type.into_ir());
        let aggr_ty = AggrType::from_ir(aggr_ty);

        let new_tctx = &self.new_module.tctx;
        let extract_inst = IndexExtractInstID::new_uninit(new_allocs, new_tctx, aggr_ty);

        let old_aggr = old_inst.get_aggr(old_allocs);
        let old_index = old_inst.get_index(old_allocs);

        self.push_use(extract_inst.aggr_use(new_allocs), old_aggr);
        self.push_use(extract_inst.index_use(new_allocs), old_index);
        extract_inst.raw_into()
    }
    /// Field extract 的索引是 u32 整数列表而不是操作数, 因此不需要 push_use
    fn clone_field_extract_inst(&self, old_inst: &FieldExtractInst) -> InstID {
        let old_allocs = &self.old_module.allocs;
        let new_allocs = &self.new_module.allocs;

        let aggr_ty = self.clone_type(old_inst.aggr_type.into_ir());
        let aggr_ty = AggrType::from_ir(aggr_ty);

        let new_tctx = &self.new_module.tctx;
        let extract_inst = FieldExtractInstID::builder(aggr_ty)
            .reserve_steps(old_inst.fields.len())
            .add_steps(new_tctx, old_inst.fields.iter().cloned())
            .build_id(new_allocs);

        let old_aggr = old_inst.get_aggr(old_allocs);
        self.push_use(extract_inst.aggr_use(new_allocs), old_aggr);
        extract_inst.raw_into()
    }
    fn clone_index_insert_inst(&self, old_inst: &IndexInsertInst) -> InstID {
        let old_allocs = &self.old_module.allocs;
        let new_allocs = &self.new_module.allocs;

        let aggr_ty = self.clone_type(old_inst.get_valtype());
        let aggr_type = AggrType::from_ir(aggr_ty);

        let new_tctx = &self.new_module.tctx;
        let insert_inst = IndexInsertInstID::new_uninit(new_allocs, new_tctx, aggr_type);

        let old_aggr = old_inst.get_aggr(old_allocs);
        let old_index = old_inst.get_index(old_allocs);
        let old_elem = old_inst.get_elem(old_allocs);

        self.push_use(insert_inst.aggr_use(new_allocs), old_aggr);
        self.push_use(insert_inst.index_use(new_allocs), old_index);
        self.push_use(insert_inst.elem_use(new_allocs), old_elem);
        insert_inst.raw_into()
    }
    fn clone_field_insert_inst(&self, old_inst: &FieldInsertInst) -> InstID {
        let old_allocs = &self.old_module.allocs;
        let new_allocs = &self.new_module.allocs;

        let aggr_ty = self.clone_type(old_inst.get_valtype());
        let aggr_type = AggrType::from_ir(aggr_ty);

        let new_tctx = &self.new_module.tctx;
        let insert_inst = FieldInsertInstID::builder(aggr_type)
            .reserve_steps(old_inst.fields.len())
            .add_steps(new_tctx, old_inst.fields.iter().cloned())
            .build_id(new_allocs);

        let old_aggr = old_inst.get_aggr(old_allocs);
        let old_elem = old_inst.get_elem(old_allocs);

        self.push_use(insert_inst.aggr_use(new_allocs), old_aggr);
        self.push_use(insert_inst.elem_use(new_allocs), old_elem);
        insert_inst.raw_into()
    }

    fn clone_phi_inst(&self, old_inst: &PhiInst) -> InstID {
        let old_allocs = &self.old_module.allocs;
        let new_allocs = &self.new_module.allocs;

        let new_ty = self.clone_type(old_inst.get_valtype());
        let phi_inst = {
            let mut builder = PhiInst::builder(new_allocs, new_ty);
            builder.allow_uninit(true);
            for &[_, old_blk] in old_inst.incoming_uses().iter() {
                let old_block = BlockID::from_ir(old_blk.get_operand(old_allocs));
                let new_block = self.map_get_block(old_block);
                builder.add_uninit_incoming(new_block);
            }
            builder.build_id()
        };

        let mut mapping = HashMap::new();
        for &[uval, ublk] in &*phi_inst.incoming_uses(new_allocs) {
            let block = BlockID::from_ir(ublk.get_operand(new_allocs));
            mapping.insert(block, uval);
        }

        for &[old_uval, old_ubb] in &*old_inst.incoming_uses() {
            let old_block = BlockID::from_ir(old_ubb.get_operand(old_allocs));
            let new_block = self.map_get_block(old_block);
            let new_use = mapping[&new_block];
            let old_val = old_uval.get_operand(old_allocs);
            self.push_use(new_use, old_val);
        }
        phi_inst.raw_into()
    }
    fn clone_select_inst(&self, old_inst: &SelectInst) -> InstID {
        let old_allocs = &self.old_module.allocs;
        let new_allocs = &self.new_module.allocs;

        let new_ty = self.clone_type(old_inst.get_valtype());
        let select_inst = SelectInstID::new_uninit(new_allocs, new_ty);

        let old_cond = old_inst.get_cond(old_allocs);
        let old_then = old_inst.get_then(old_allocs);
        let old_else = old_inst.get_else(old_allocs);

        self.push_use(select_inst.cond_use(new_allocs), old_cond);
        self.push_use(select_inst.then_use(new_allocs), old_then);
        self.push_use(select_inst.else_use(new_allocs), old_else);
        select_inst.raw_into()
    }

    fn map_all_uses(&self) {
        let new_allocs = &self.new_module.allocs;
        while let Some(uproc) = self.pop_use_proc() {
            let UseProc { new_use, old_val } = uproc;
            let mapped_val = self.map_get_value(old_val);
            new_use.set_operand(new_allocs, mapped_val);
        }
    }
    fn pop_use_proc(&self) -> Option<UseProc> {
        self.process_queue.borrow_mut().uses.pop_front()
    }
    fn map_get_value(&self, old_value: ValueSSA) -> ValueSSA {
        match old_value {
            ValueSSA::None => ValueSSA::None,
            ValueSSA::ConstData(data) => {
                let data = match data {
                    ConstData::Undef(ty) => ConstData::Undef(self.clone_type(ty)),
                    ConstData::PtrNull(ty) => ConstData::PtrNull(self.clone_type(ty)),
                    data => data,
                };
                ValueSSA::ConstData(data)
            }
            ValueSSA::ConstExpr(old_expr) => ValueSSA::ConstExpr(self.map_expr(old_expr)),
            ValueSSA::AggrZero(old_aggr) => {
                let new_aggr = self.clone_type(old_aggr.into_ir());
                ValueSSA::AggrZero(AggrType::from_ir(new_aggr))
            }
            ValueSSA::FuncArg(old_func, idx) => {
                let mapping = self.value_mapping.borrow();
                let new_func = FuncID::raw_from(mapping.globals[&old_func.raw_into()]);
                ValueSSA::FuncArg(new_func, idx)
            }
            ValueSSA::Block(old_bb) => ValueSSA::Block(self.map_get_block(old_bb)),
            ValueSSA::Inst(old_inst) => {
                let mapping = self.value_mapping.borrow();
                let Some(new_inst) = mapping.insts.get(&old_inst) else {
                    panic!(
                        r#"Internal error: cannot find instruction mapping for {old_inst:?}.
                        NOTE that users can only take instruction operands in the same function."#
                    )
                };
                ValueSSA::Inst(*new_inst)
            }
            ValueSSA::Global(old_global) => {
                let mapping = self.value_mapping.borrow();
                let new_global = mapping.globals[&old_global];
                ValueSSA::Global(new_global)
            }
        }
    }
    fn map_expr(&self, old_expr: ExprID) -> ExprID {
        use crate::ir::ExprObj;
        if let Some(new_expr) = self.value_mapping.borrow().exprs.get(&old_expr) {
            return *new_expr;
        }

        match old_expr.deref_ir(&self.old_module.allocs) {
            ExprObj::Array(old_arr) => self.clone_array_expr(old_expr, old_arr),
            ExprObj::DataArray(old_darr) => self.clone_data_array(old_expr, old_darr),
            ExprObj::SplatArray(old_sarr) => self.clone_splat_array(old_expr, old_sarr),
            ExprObj::KVArray(old_kvarr) => self.clone_kvarray(old_expr, old_kvarr),
            ExprObj::Struct(old_struc) => self.clone_struct_expr(old_expr, old_struc),
            ExprObj::FixVec(old_fvec) => self.clone_fixvec_expr(old_expr, old_fvec),
        }
    }
    fn clone_array_expr(&self, old_expr: ExprID, old_arr: &ArrayExpr) -> ExprID {
        let new_allocs = &self.new_module.allocs;
        let new_tctx = &self.new_module.tctx;
        let arrty = self.clone_type(old_arr.arrty);
        let arr = ArrayExprID::new_uninit(new_allocs, new_tctx, arrty);
        // 为了应对可能出现的循环引用问题, 这里提前把 new 好的数组引用放上去.
        self.insert_expr(old_expr, arr);

        let old_elems = old_arr.elems.iter().copied();
        let new_elems = arr.get_elems(new_allocs).iter().copied();
        for (new_use, old_use) in new_elems.zip(old_elems) {
            let old_val = old_use.get_operand(&self.old_module.allocs);
            self.push_use(new_use, old_val);
        }
        arr.raw_into()
    }
    fn clone_data_array(&self, old_expr: ExprID, old_arr: &DataArrayExpr) -> ExprID {
        let new_allocs = &self.new_module.allocs;
        // DataArrayExpr 没有操作数, 可以直接 clone. 不过需要做一些修正.
        let mut new_arr = old_arr.clone();
        new_arr.common.dispose_mark.set(false);
        new_arr.common.users = None;
        new_arr.arrty = self.clone_type(old_arr.arrty);
        // 这里需要做一些类型修正.
        if let ConstArrayData::FreeStyle(cdatas) = &mut new_arr.data {
            for data in cdatas {
                *data = self.map_get_constdata(*data);
            }
        }

        let new_arr = DataArrayExprID::allocate(new_allocs, new_arr);
        self.insert_expr(old_expr, new_arr);
        new_arr.raw_into()
    }
    fn clone_splat_array(&self, old_expr: ExprID, old_arr: &SplatArrayExpr) -> ExprID {
        let new_allocs = &self.new_module.allocs;
        let new_tctx = &self.new_module.tctx;

        let arrty = self.clone_type(old_arr.arrty);
        let new_arr = SplatArrayExprID::new_uninit(new_allocs, new_tctx, arrty);
        self.insert_expr(old_expr, new_arr);

        let old_allocs = &self.old_module.allocs;
        let old_elem = old_arr.get_elem(old_allocs);
        let new_elem_use = new_arr.elem_use(new_allocs);
        self.push_use(new_elem_use, old_elem);
        new_arr.raw_into()
    }
    fn clone_kvarray(&self, old_expr: ExprID, old_arr: &KVArrayExpr) -> ExprID {
        let new_allocs = &self.new_module.allocs;
        let new_tctx = &self.new_module.tctx;

        let elemty = self.clone_type(old_arr.elemty);
        let arrty = self.clone_type(old_arr.arrty);

        let elem0 = ValueSSA::new_zero(elemty).unwrap();
        let mut arr_builder = KVArrayExpr::builder(new_tctx, new_allocs, arrty);
        for (k, ..) in old_arr.elem_iter(&self.old_module.allocs) {
            arr_builder.add_elem(k, elem0).unwrap();
        }
        arr_builder.default_val(elem0);
        let new_arr = arr_builder.build_id();
        self.insert_expr(old_expr, new_arr);

        let old_elems = old_arr.elem_iter(&self.old_module.allocs);
        let new_elems = new_arr.elem_iter(new_allocs);
        for ((_, old_val, _), (.., new_use)) in old_elems.zip(new_elems) {
            self.push_use(new_use, old_val);
        }

        let old_default = old_arr.get_default(&self.old_module.allocs);
        let new_default_use = new_arr.default_use(new_allocs);
        self.push_use(new_default_use, old_default);
        new_arr.raw_into()
    }
    fn clone_struct_expr(&self, old_expr: ExprID, struc: &StructExpr) -> ExprID {
        let new_allocs = &self.new_module.allocs;
        let new_tctx = &self.new_module.tctx;

        let struct_ty = self.clone_type(struc.structty);
        let new_struc = StructExprID::new_uninit(new_allocs, new_tctx, struct_ty);
        self.insert_expr(old_expr, new_struc);

        let old_allocs = &self.old_module.allocs;
        let old_elems = struc.fields.iter().copied();
        let new_elems = new_struc.field_uses(new_allocs).iter().copied();
        for (new_use, old_use) in new_elems.zip(old_elems) {
            let old_val = old_use.get_operand(old_allocs);
            self.push_use(new_use, old_val);
        }
        new_struc.raw_into()
    }
    fn clone_fixvec_expr(&self, old_expr: ExprID, old_fvec: &FixVec) -> ExprID {
        let new_allocs = &self.new_module.allocs;
        let vecty = self.clone_type(old_fvec.vecty); // actually does nothing
        let new_vec = FixVecID::new_uninit(new_allocs, vecty);
        self.insert_expr(old_expr, new_vec);

        let old_allocs = &self.old_module.allocs;
        let old_elems = old_fvec.elems.iter().copied();
        let new_elems = new_vec.elem_uses(new_allocs).iter().copied();
        for (new_use, old_use) in new_elems.zip(old_elems) {
            let old_val = old_use.get_operand(old_allocs);
            self.push_use(new_use, old_val);
        }
        new_vec.raw_into()
    }
}

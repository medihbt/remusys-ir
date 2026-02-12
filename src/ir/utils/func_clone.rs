use smallvec::SmallVec;
use smol_str::SmolStr;

use crate::{
    SymbolStr,
    ir::{inst::*, *},
};
use std::{collections::HashMap, rc::Rc};

#[derive(Debug, Clone, thiserror::Error)]
pub enum FuncCloneErr {
    #[error("failed to clone instruction: {0}")]
    IRBuild(#[from] IRBuildError),

    #[error("function cloning does not support extern linkage")]
    NoExternLinkage,

    #[error("function @{name:?} (addr: {addr:?}) is extern and cannot be cloned")]
    FuncIsExtern { name: SymbolStr, addr: FuncID },

    #[error("block {0:?} not found in mapping")]
    BlockNotFound(BlockID),

    #[error("empty jump target {id:?} from instruction {from:?} is not found in mapping")]
    EmptyJT { from: InstID, id: JumpTargetID },

    #[error("instruction {inst:?} is in section {actual:?}, but expected {expect:?}")]
    InstInWrongSection { inst: InstID, expect: BlockSection, actual: BlockSection },

    #[error("duplicate incoming block {block:?} in phi {inst:?} when cloning function")]
    DuplicatePhiIncoming { inst: PhiInstID, block: BlockID },

    #[error("some uses cannot be resolved in mapping: {0:?}")]
    UnresolvedUses(Rc<[UseID]>),
}

type FuncCloneRes<T = ()> = Result<T, FuncCloneErr>;

pub struct FuncCloneMapping {
    pub insts: HashMap<InstID, InstID>,
    pub blocks: HashMap<BlockID, BlockID>,
    pub old_func: FuncID,
    pub new_func: FuncID,
    pub keep_recurse: bool,
}
impl FuncCloneMapping {
    /// Map a value from the old function to the new function.
    /// Unmapped instructions and blocks will return `None`, while other unmapped values will return themselves.
    /// For global values, if `keep_recurse` is true and the value is the old function,
    /// it will be mapped to the new function. Otherwise, global values are not mapped
    /// and returned as is.
    pub fn map_get(&self, old: ValueSSA) -> Option<ValueSSA> {
        match old {
            ValueSSA::FuncArg(func_id, index) => {
                assert_eq!(func_id, self.old_func, "arguments are function local");
                Some(ValueSSA::FuncArg(self.new_func, index))
            }
            ValueSSA::Block(block_id) => self
                .blocks
                .get(&block_id)
                .map(|&new_block_id| ValueSSA::Block(new_block_id)),
            ValueSSA::Inst(inst_id) => self
                .insts
                .get(&inst_id)
                .map(|&new_inst_id| ValueSSA::Inst(new_inst_id)),
            ValueSSA::Global(global_id) => {
                if self.keep_recurse && global_id == self.old_func.raw_into() {
                    Some(ValueSSA::Global(self.new_func.raw_into()))
                } else {
                    Some(ValueSSA::Global(global_id))
                }
            }
            old => Some(old),
        }
    }
}

pub struct FuncClone<'ir> {
    pub module: &'ir mut Module,
    pub old_func: FuncID,
    pub builder: FuncBuilder,
    pub keep_recurse: bool,
    pub name: SymbolStr,
    exports: bool,
}

impl<'ir> FuncClone<'ir> {
    pub fn new(module: &'ir mut Module, func: FuncID) -> FuncCloneRes<Self> {
        let (builder, name) = {
            let tctx = &module.tctx;
            let allocs = &module.allocs;

            let obj = func.deref_ir(allocs);
            if obj.is_extern(allocs) {
                return Err(FuncCloneErr::FuncIsExtern { name: obj.clone_name(), addr: func });
            }
            let functype = func.get_functype(allocs);
            let name = obj.clone_name();
            let mut builder = FuncBuilder::new(tctx, String::new(), functype);
            builder
                .linkage(obj.get_linkage(allocs))
                .terminate_mode(FuncTerminateMode::Unreachable);
            builder.attrs = obj.attrs().clone();
            for (i, arg) in obj.args.iter().enumerate() {
                builder.arg_attrs[i] = arg.attrs().clone();
                builder.arg_attrs[i].set_pos(AttributePos::FUNCARG);
            }
            (builder, name)
        };

        Ok(Self {
            module,
            old_func: func,
            builder,
            exports: false,
            keep_recurse: true,
            name,
        })
    }

    /// Change the name of the cloned function, then hide it from symbols.
    pub fn change_name(&mut self, name: impl Into<SmolStr>) -> &mut Self {
        self.name = name.into();
        self.exports = false;
        self
    }
    pub fn try_export(&mut self) -> Result<&mut Self, GlobalID> {
        let symtab = self.module.symbols.get_mut();
        let exported = symtab.exported();
        if let Some(exported) = exported.get(&self.name) {
            Err(*exported)
        } else {
            self.exports = true;
            Ok(self)
        }
    }
    pub fn hide(&mut self) -> &mut Self {
        self.exports = false;
        self
    }
    pub fn keep_recurse(&mut self, keep: bool) -> &mut Self {
        self.keep_recurse = keep;
        self
    }
    pub fn linkage(&mut self, linkage: Linkage) -> FuncCloneRes<&mut Self> {
        if linkage == Linkage::External {
            return Err(FuncCloneErr::NoExternLinkage);
        }
        self.builder.linkage(linkage);
        Ok(self)
    }

    /// Finish cloning the function and return the mapping.
    pub fn finish(self) -> FuncCloneRes<FuncCloneMapping> {
        let Self { module, old_func, builder, keep_recurse, name, exports } = self;
        let new_func = builder.build_pinned(module);
        new_func.deref_ir_mut(&mut module.allocs).common.name = name;
        if exports {
            new_func
                .export(module)
                .expect("internal error: should return error in build section");
        }

        let mut inner = Self::clone_block_infra(module, old_func, new_func, keep_recurse)?;
        inner.clone_terminator()?;
        inner.clone_insts()?;

        Ok(inner.into_mapping())
    }

    fn clone_block_infra(
        module: &'ir Module,
        old_func: FuncID,
        new_func: FuncID,
        keep_recurse: bool,
    ) -> FuncCloneRes<Inner<'ir>> {
        let allocs = &module.allocs;

        let old_func_obj = old_func.deref_ir(allocs);
        let new_func_obj = new_func.deref_ir(allocs);

        let Some(old_body) = &old_func_obj.body else {
            // this should be prevented by the constructor, but we check again just in case
            let name = old_func_obj.get_name();
            unreachable!("internal error: function @{name:?} has no body, cannot clone");
        };
        let Some(new_body) = &new_func_obj.body else {
            // this should never happen, but we check just in case
            let name = new_func_obj.get_name();
            unreachable!("internal error: new function @{name:?} has no body, cannot clone");
        };

        let mut blocks = HashMap::with_capacity(old_body.blocks.len());
        let mut block_list = Vec::with_capacity(old_body.blocks.len());
        let mut builder = IRBuilder::new(module);

        builder.set_focus(IRFocus::Block(new_body.entry));
        blocks.insert(old_body.entry, new_body.entry);
        block_list.push((old_body.entry, new_body.entry));

        for (old_bbid, _) in old_body.blocks.iter(&allocs.blocks) {
            if old_bbid == old_body.entry {
                continue; // entry block is already created
            }
            let new_bbid = builder
                .split_block()
                .expect("internal error: failed to create block during function cloning");
            blocks.insert(old_bbid, new_bbid);
            block_list.push((old_bbid, new_bbid));
            builder.set_focus(IRFocus::Block(new_bbid));
        }

        Ok(Inner {
            module,
            insts: HashMap::new(),
            blocks: Rc::new(blocks),
            old_func,
            new_func,
            keep_recurse,
            block_list: Rc::from(block_list.as_slice()),
            use_queue: SmallVec::new(),
        })
    }
}

struct Inner<'ir> {
    module: &'ir Module,
    insts: HashMap<InstID, InstID>,
    blocks: Rc<HashMap<BlockID, BlockID>>,
    old_func: FuncID,
    new_func: FuncID,
    keep_recurse: bool,
    block_list: Rc<[(BlockID, BlockID)]>,
    use_queue: SmallVec<[UseID; 16]>,
}
impl<'ir> Inner<'ir> {
    fn into_mapping(self) -> FuncCloneMapping {
        let Self { insts, mut blocks, old_func, new_func, keep_recurse, .. } = self;
        let blocks = std::mem::take(Rc::make_mut(&mut blocks));
        FuncCloneMapping { insts, blocks, old_func, new_func, keep_recurse }
    }

    /// Map a value from the old function to the new function.
    /// If the value is not found in the mapping, it will be returned as is.
    /// For global values, if `keep_recurse` is true and the value is the old function,
    /// it will be mapped to the new function. Otherwise, global values are not mapped
    /// and returned as is.
    fn map_val(&self, old: ValueSSA) -> Option<ValueSSA> {
        match old {
            ValueSSA::FuncArg(func_id, index) => {
                assert_eq!(func_id, self.old_func, "arguments are function local");
                Some(ValueSSA::FuncArg(self.new_func, index))
            }
            ValueSSA::Block(block_id) => self
                .blocks
                .get(&block_id)
                .map(|&new_block_id| ValueSSA::Block(new_block_id)),
            ValueSSA::Inst(inst_id) => self
                .insts
                .get(&inst_id)
                .map(|&new_inst_id| ValueSSA::Inst(new_inst_id)),
            ValueSSA::Global(global_id) => {
                if self.keep_recurse && global_id == self.old_func.raw_into() {
                    Some(ValueSSA::Global(self.new_func.raw_into()))
                } else {
                    Some(ValueSSA::Global(global_id))
                }
            }
            old => Some(old),
        }
    }

    fn map_jt(&self, old_jt: JumpTargetID) -> FuncCloneRes<BlockID> {
        let allocs = &self.module.allocs;
        let Some(bb) = old_jt.get_block(allocs) else {
            let inst = old_jt
                .get_terminator(allocs)
                .expect("broken IR structure: jump target should be linked with a terminator");
            return Err(FuncCloneErr::EmptyJT { from: inst, id: old_jt });
        };
        match self.blocks.get(&bb) {
            Some(&new_bb) => Ok(new_bb),
            None => Err(FuncCloneErr::BlockNotFound(bb)),
        }
    }
    fn use_setval(&mut self, newuse: UseID, old_val: ValueSSA) {
        let allocs = &self.module.allocs;
        match self.map_val(old_val) {
            Some(v) => {
                newuse.set_operand(allocs, v);
            }
            None => {
                newuse.set_operand(allocs, old_val);
                self.use_queue.push(newuse);
            }
        }
    }

    fn clone_terminator(&mut self) -> FuncCloneRes {
        let block_list = self.block_list.clone();
        let allocs = &self.module.allocs;

        let mut builder = IRBuilder::new(self.module);
        for &(old_bb, new_bb) in block_list.iter() {
            use crate::ir::TerminatorID as T;
            builder.set_focus(IRFocus::Block(new_bb));
            let old_terminator = old_bb.get_terminator(allocs);

            let new_termi: InstID = match old_terminator {
                T::Unreachable(_) => builder.focus_set_unreachable()?.1.raw_into(),
                T::Ret(ret) => {
                    let retval = ret.get_retval(allocs);
                    let new_ret = RetInstID::new_uninit(allocs, ret.get_rettype(allocs));
                    builder.insert_inst(new_ret)?;
                    self.use_setval(new_ret.retval_use(allocs), retval);
                    new_ret.raw_into()
                }
                T::Jump(jump) => {
                    let new_jt = self.map_jt(jump.target_jt(allocs))?;
                    builder.focus_set_jump_to(new_jt)?.1.raw_into()
                }
                T::Br(br) => {
                    let new_then = self.map_jt(br.then_jt(allocs))?;
                    let new_else = self.map_jt(br.else_jt(allocs))?;
                    let cond = br.get_cond(allocs);
                    let (_, new_br) = builder.focus_set_branch_to(cond, new_then, new_else)?;
                    self.use_setval(new_br.cond_use(allocs), cond);
                    new_br.raw_into()
                }
                T::Switch(switch) => {
                    let discrim = switch.get_discrim(allocs);
                    let default_bb = self.map_jt(switch.default_jt(allocs))?;
                    let cases = {
                        let cases_len = switch.borrow_cases(allocs).len();
                        let mut cases: SmallVec<[(i64, BlockID); 8]> =
                            SmallVec::with_capacity(cases_len);
                        for (jt, val, _) in switch.cases_iter(allocs) {
                            let new_bb = self.map_jt(jt)?;
                            cases.push((val, new_bb));
                        }
                        cases
                    };
                    let (_, new_switch) =
                        builder.focus_set_switch_to(discrim, default_bb, cases)?;
                    self.use_setval(new_switch.discrim_use(allocs), discrim);
                    new_switch.raw_into()
                }
            };

            self.insts.insert(old_terminator.into_ir(), new_termi);
        }
        Ok(())
    }

    fn clone_insts(&mut self) -> FuncCloneRes {
        let block_list = self.block_list.clone();
        let allocs = &self.module.allocs;

        let mut builder = IRBuilder::new(self.module);
        // `len - 2` because:
        // - the terminator is not cloned in this step.
        // - the phi-end should not be mapped.
        // use `saturating_sub` to avoid underflow when the block has no instructions (only phi-end and terminator).
        let insts_len: usize = block_list
            .iter()
            .map(|&(old_bb, _)| old_bb.get_insts(allocs).len().saturating_sub(2))
            .sum();
        self.insts.reserve(insts_len);

        for &(old_bb, new_bb) in block_list.iter() {
            let mut phi_section = true;
            builder.set_focus(IRFocus::Block(new_bb));
            for (oinst_id, oinst) in old_bb.insts_iter(allocs) {
                if oinst_id == old_bb.get_terminator_inst(allocs) {
                    break; // terminator is handled in `clone_terminator`
                }
                let Some(inst) = self.make_inst(oinst_id, oinst, &mut phi_section)? else {
                    continue;
                };
                builder.insert_inst(inst)?;
                self.insts.insert(oinst_id, inst);
            }
        }

        let use_queue = std::mem::take(&mut self.use_queue);
        for use_id in use_queue {
            self.use_setval(use_id, use_id.get_operand(allocs));
        }
        if self.use_queue.is_empty() {
            Ok(())
        } else {
            let uses = self.use_queue.as_slice();
            Err(FuncCloneErr::UnresolvedUses(Rc::from(uses)))
        }
    }

    fn make_inst(
        &mut self,
        oinst_id: InstID,
        oinst: &InstObj,
        phi_section: &mut bool,
    ) -> FuncCloneRes<Option<InstID>> {
        let (allocs, tctx) = (&self.module.allocs, &self.module.tctx);

        let section_not_body = |actual| {
            Err(FuncCloneErr::InstInWrongSection {
                inst: oinst_id,
                expect: BlockSection::Body,
                actual,
            })
        };

        match (*phi_section, oinst.get_block_section()) {
            (true, BlockSection::Phi) | (false, BlockSection::Body) => {}
            (true, BlockSection::PhiEnd) => {
                *phi_section = false;
                return Ok(None); // phi-end is not cloned and not mapped, so we skip it.
            }
            (_, BlockSection::Terminator) => return section_not_body(BlockSection::Terminator),
            (false, BlockSection::Phi) => return section_not_body(BlockSection::Phi),
            (false, BlockSection::PhiEnd) => return section_not_body(BlockSection::PhiEnd),
            (true, BlockSection::Body) => {
                return Err(FuncCloneErr::InstInWrongSection {
                    inst: oinst_id,
                    expect: BlockSection::Phi,
                    actual: BlockSection::Body,
                });
            }
        }

        let inst: InstID = match oinst {
            InstObj::PhiInstEnd(_)
            | InstObj::Unreachable(_)
            | InstObj::Ret(_)
            | InstObj::Jump(_)
            | InstObj::Br(_)
            | InstObj::Switch(_)
            | InstObj::GuideNode(_) => {
                let opcode = oinst.get_opcode();
                unreachable!("internal error: inst {opcode:?} mentioned obove should be handled")
            }
            InstObj::Alloca(alloca) => {
                AllocaInstID::new(allocs, alloca.pointee_ty, alloca.align_log2).raw_into()
            }
            InstObj::GEP(gepinst) => {
                let id = GEPInstID::new_uninit(
                    allocs,
                    gepinst.initial_ty,
                    gepinst.final_ty,
                    gepinst.index_uses().len(),
                    gepinst.align_log2,
                    gepinst.pointee_align_log2,
                );
                let oldops = gepinst.operands_iter();
                let newops = id.deref_ir(allocs).operands_iter();
                for (olduid, newuid) in oldops.zip(newops) {
                    self.use_setval(newuid, olduid.get_operand(allocs));
                }
                id.raw_into()
            }
            InstObj::Load(load) => {
                let load_inst = LoadInstID::new_uninit(allocs, load.get_valtype(), load.align_log2);
                let old_source = load.get_source(allocs);
                self.use_setval(load_inst.source_use(allocs), old_source);
                load_inst.raw_into()
            }
            InstObj::Store(store) => {
                let store_inst = StoreInstID::new_uninit(allocs, store.source_ty, store.align_log2);
                let old_src = store.get_source(allocs);
                let old_dst = store.get_target(allocs);
                self.use_setval(store_inst.source_use(allocs), old_src);
                self.use_setval(store_inst.target_use(allocs), old_dst);
                store_inst.raw_into()
            }
            InstObj::AmoRmw(amormw) => {
                let amormw_inst = AmoRmwInst::builder(amormw.get_opcode(), amormw.value_ty)
                    .align_log2(amormw.align_log2)
                    .is_volatile(amormw.is_volatile)
                    .ordering(amormw.ordering)
                    .scope(amormw.scope)
                    .build_id(allocs);
                let old_ptr = amormw.pointer_use().get_operand(allocs);
                let old_val = amormw.value_use().get_operand(allocs);
                self.use_setval(amormw_inst.pointer_use(allocs), old_ptr);
                self.use_setval(amormw_inst.value_use(allocs), old_val);
                amormw_inst.raw_into()
            }
            InstObj::BinOP(binop) => {
                let opcode = binop.get_opcode();
                let flags = binop.get_flags();
                let new_binop = BinOPInstID::new_uninit(allocs, opcode, binop.get_valtype());
                new_binop.set_flags(allocs, flags);
                let old_lhs = binop.get_lhs(allocs);
                let old_rhs = binop.get_rhs(allocs);
                self.use_setval(new_binop.lhs_use(allocs), old_lhs);
                self.use_setval(new_binop.rhs_use(allocs), old_rhs);
                new_binop.raw_into()
            }
            InstObj::Call(call) => {
                let mut call_builder = CallInst::builder(tctx, call.callee_ty);
                call_builder
                    .resize_nargs(call.arg_uses().len() as u32)
                    .expect(
                        "internal error: failed to resize call instruction when cloning function",
                    );
                let call_inst = call_builder
                    .is_tail_call(call.is_tail_call.get())
                    .builder_uninit(true)
                    .build_id(allocs);
                let old_callee = call.get_callee(allocs);
                self.use_setval(call_inst.callee_use(allocs), old_callee);
                for (idx, old_arg_use) in call.arg_uses().iter().enumerate() {
                    let old_arg = old_arg_use.get_operand(allocs);
                    let new_arg_use = call_inst.arg_uses(allocs)[idx];
                    self.use_setval(new_arg_use, old_arg);
                }
                call_inst.raw_into()
            }
            InstObj::Cast(cast) => {
                let cast_inst = CastInstID::new_uninit(
                    allocs,
                    cast.get_opcode(),
                    cast.from_ty,
                    cast.get_valtype(),
                );
                let old_from = cast.get_from(allocs);
                self.use_setval(cast_inst.from_use(allocs), old_from);
                cast_inst.raw_into()
            }
            InstObj::Cmp(cmp) => {
                let cmp_inst =
                    CmpInstID::new_uninit(allocs, cmp.get_opcode(), cmp.cond, cmp.operand_ty);
                let old_lhs = cmp.get_lhs(allocs);
                let old_rhs = cmp.get_rhs(allocs);
                self.use_setval(cmp_inst.lhs_use(allocs), old_lhs);
                self.use_setval(cmp_inst.rhs_use(allocs), old_rhs);
                cmp_inst.raw_into()
            }
            InstObj::IndexExtract(index_extract) => {
                let extract_inst =
                    IndexExtractInstID::new_uninit(allocs, tctx, index_extract.aggr_type);
                let old_aggr = index_extract.get_aggr(allocs);
                let old_index = index_extract.get_index(allocs);
                self.use_setval(extract_inst.aggr_use(allocs), old_aggr);
                self.use_setval(extract_inst.index_use(allocs), old_index);
                extract_inst.raw_into()
            }
            InstObj::FieldExtract(field_extract) => {
                let extract_inst = FieldExtractInstID::builder(field_extract.aggr_type)
                    .reserve_steps(field_extract.fields.len())
                    .add_steps(tctx, field_extract.fields.iter().cloned())
                    .build_id(allocs);
                let old_aggr = field_extract.get_aggr(allocs);
                self.use_setval(extract_inst.aggr_use(allocs), old_aggr);
                extract_inst.raw_into()
            }
            InstObj::IndexInsert(index_insert) => {
                let aggr_type = index_insert.get_aggr_operand_type();
                let insert_inst = IndexInsertInstID::new_uninit(allocs, tctx, aggr_type);
                let old_aggr = index_insert.get_aggr(allocs);
                let old_index = index_insert.get_index(allocs);
                let old_elem = index_insert.get_elem(allocs);
                self.use_setval(insert_inst.aggr_use(allocs), old_aggr);
                self.use_setval(insert_inst.index_use(allocs), old_index);
                self.use_setval(insert_inst.elem_use(allocs), old_elem);
                insert_inst.raw_into()
            }
            InstObj::FieldInsert(field_insert) => {
                let aggr_type = field_insert.get_aggr_operand_type();
                let insert_inst = FieldInsertInstID::builder(aggr_type)
                    .reserve_steps(field_insert.fields.len())
                    .add_steps(tctx, field_insert.fields.iter().cloned())
                    .build_id(allocs);
                let old_aggr = field_insert.get_aggr(allocs);
                let old_elem = field_insert.get_elem(allocs);
                self.use_setval(insert_inst.aggr_use(allocs), old_aggr);
                self.use_setval(insert_inst.elem_use(allocs), old_elem);
                insert_inst.raw_into()
            }
            InstObj::Phi(phi) => {
                let mut builder = PhiInst::builder(allocs, phi.get_valtype());
                builder.allow_uninit(true);
                for &[old_val, old_blk] in &*phi.incoming_uses() {
                    let old_block = old_blk.get_operand(allocs);
                    let ValueSSA::Block(old_block) = old_block else {
                        panic!("internal error: expect phi incoming block, found {old_block:?}");
                    };
                    let Some(&new_block) = self.blocks.get(&old_block) else {
                        return Err(FuncCloneErr::BlockNotFound(old_block));
                    };
                    let old_val = old_val.get_operand(allocs);
                    if builder.incomings.contains_key(&new_block) {
                        let inst = PhiInstID::raw_from(oinst_id);
                        let block = old_block;
                        return Err(FuncCloneErr::DuplicatePhiIncoming { inst, block });
                    }
                    builder.add_incoming(new_block, old_val);
                }

                let new_phi = builder.build_id();
                for [newval, _] in &*new_phi.incoming_uses(allocs) {
                    let old_val = newval.get_operand(allocs);
                    self.use_setval(*newval, old_val);
                }
                new_phi.raw_into()
            }
            InstObj::Select(select) => {
                let select_inst = SelectInstID::new_uninit(allocs, select.get_valtype());
                let old_cond = select.get_cond(allocs);
                let old_then = select.get_then(allocs);
                let old_else = select.get_else(allocs);
                self.use_setval(select_inst.cond_use(allocs), old_cond);
                self.use_setval(select_inst.then_use(allocs), old_then);
                self.use_setval(select_inst.else_use(allocs), old_else);
                select_inst.raw_into()
            }
        };
        Ok(Some(inst))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::testing::cases::*;

    #[test]
    fn test_func_clone() {
        let mut module = test_case_cfg_deep_while_br().module;
        let old_func = module
            .get_global_by_name("main")
            .map(FuncID::raw_from)
            .expect("test case should have a function named 'main'");
        let mut func_clone = FuncClone::new(&mut module, old_func).unwrap();
        func_clone.change_name("main_clone").try_export().unwrap();
        func_clone.finish().unwrap();
        module.begin_gc().finish();
        write_ir_to_file(
            "../target/test-func-clone.ll",
            &module,
            IRWriteOption::loud(),
        );
    }
}

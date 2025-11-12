//! IR Basic sanity check helpers. Returns an error if any invariant is violated.
//!
//! 提供基本不变量快速断言，避免重构后潜在环链/引用错误静默积累。
//! 当前实现是“最低保障级别”，后续可加强：
//! - Phi incoming 对称性
//! - Terminator JumpTarget 完整性
//! - 无 `DisposedUse` 残留在活体 users/preds 环
//! - 指令父块 / 块父函数双向关系全量巡检

use crate::{
    base::FixBitSet,
    ir::{checking::IRLocation, inst::*, module::allocs::IPoolAllocated, *},
    typing::{FPKind, FixVecType, IntType, PtrType, TypeContext},
};
use mtb_entity_slab::{EntityAlloc, IEntityAllocID};
use std::{
    cell::{Cell, Ref, RefCell},
    collections::{HashMap, HashSet, VecDeque},
    sync::Arc,
};

#[derive(Debug, Clone, thiserror::Error)]
pub enum IRSanityErr {
    #[error("Global `{name:?}` (ID {id:p}) is unexpectedly dead")]
    DeadGlobal { name: Option<Arc<str>>, id: GlobalID },
    #[error("Basic block ID {0:?} is unexpectedly dead")]
    DeadBlock(BlockID),
    #[error("Instruction ID {0:?} is unexpectedly dead")]
    DeadInst(InstID),
    #[error("Constant expression ID {0:p} is unexpectedly dead")]
    DeadConstExpr(ExprID),
    #[error("Use ID {0:?} missing from operand's users ring")]
    TraceableMissingUse(UseID),
    #[error("Use ID {0:?} of kind {1:?} missing its operand")]
    UseMissingOperand(UseID, UseKind),
    #[error("User ring broken for ValueSSA {0:?}")]
    BrokenUserRing(ValueSSA),

    #[error("User {0:?} is dead")]
    DeadUser(UserID),
    #[error("UseID {2:?} of User {0:?} index {1} is dead")]
    DeadUserUse(UserID, u32, UseID),
    #[error("Inconsistent operand-use relationship for User {0:?} index {1} UseID {2:?}")]
    InConsistentOperandUse(UserID, u32, UseID),

    #[error("JumpTarget ID {0:?} is unexpectedly dead")]
    DeadJT(JumpTargetID),
    #[error("JumpTarget ID {0:?} (kind {1:?}) preds should be {2:?}, found {3:?}")]
    JTPredMismatch(JumpTargetID, JumpTargetKind, BlockID, Option<BlockID>),
    #[error("JumpTarget ID {0:?} (kind {1:?}) has inconsistent state")]
    InConsistentJT(JumpTargetID, JumpTargetKind),
    #[error("JumpTarget {0:?} (kind {1:?}) target block missing")]
    JTBlockMissing(JumpTargetID, JumpTargetKind),
    #[error("JumpTarget {0:?} (kind {1:?}) is jumping into an unattached block {2:?}")]
    JTBlockUnattached(JumpTargetID, JumpTargetKind, BlockID),
    #[error(
        "JumpTarget {0:?} (kind {1:?}) is jumping into a block {2:?} inside other funcion {3:?}"
    )]
    JumpToNonLocalBlock(JumpTargetID, JumpTargetKind, BlockID, FuncID),
    #[error("Broken JumpTarget ring for block {0:?}")]
    BrokenJTRing(BlockID),

    #[error("Function argument index mismatch for {0:?}")]
    FuncArgIndexMismatch(FuncArgID),
    #[error("Function {0:?} entry block {1:?} position {2} invalid: expected 0")]
    InvalidFuncEntryPos(FuncID, BlockID, usize),
    #[error("Body block {1:?} not attached to parent function {0:?}")]
    BlockNotAttached(FuncID, BlockID),

    #[error("Basic block {0:?} has multiple terminators")]
    BlockHasMultipleTerminators(BlockID),
    #[error("Basic block {0:?} terminator position incorrect")]
    BlockTerminatorPosIncorrect(BlockID),
    #[error("Basic block {0:?} missing PhiEnd marker")]
    BlockPhiEndMissing(BlockID),
    #[error("Basic block {0:?} Phi position incorrect")]
    BlockPhiSegmentErr(BlockID, InstID),

    #[error("Type mismatch: {0}")]
    TypeErr(#[from] TypeMismatchErr),
    #[error("Value type mismatch: expected {1:?}, found {2:?} for ValueSSA {0:?}")]
    ValueNotType(ValueSSA, ValTypeID, ValTypeID),
    #[error("Instruction ID {0:?} type mismatch: expected {1:?}, found {2:?}")]
    InstNotType(InstID, ValTypeID, ValTypeID),
    #[error("Instruction ID {0:?} type class mismatch: expected {1:?}, found {2:?}")]
    InstTypeNotClass(InstID, ValTypeClass, ValTypeID),
    #[error("Use ID {0:?} (kind {1:?}) operand type mismatch: expected {2:?}, found {3:?}")]
    OperandNotType(UseID, UseKind, ValTypeID, ValTypeID),
    #[error(
        "Use ID {0:?} (kind {1:?}) operand type mismatch: expected {2:?} or its element, found {3:?}"
    )]
    OperandTypeNotVecOrElem(UseID, UseKind, FixVecType, ValTypeID),
    #[error("Use ID {0:?} (kind {1:?}) operand type class mismatch: expected {2:?}, found {3:?}")]
    OperandTypeNotClass(UseID, UseKind, ValTypeClass, ValTypeID),
    #[error("Use ID {0:?} (kind {1:?}) operand class mismatch: expected {2:?}, found {3:?}")]
    OperandNotClass(UseID, UseKind, ValueClass, ValueSSA),

    #[error("Instruction {0:?} not attached to any basic block")]
    InstNotAttached(InstID),
    #[error("GEP unpack error for {0:?}: {1}")]
    GEPUnpackErr(GEPInstID, GEPUnpackErr),
    #[error("Cast error for instruction ID {0:?}: {1}")]
    CastErr(CastInstID, CastErr),
    #[error("Phi instruction ID {0:?} error: {1}")]
    PhiErr(PhiInstID, PhiInstErr),
}
pub type IRSanityRes<T = ()> = Result<T, IRSanityErr>;

impl IRSanityErr {
    pub fn explain(&self, module: &Module, out: &mut dyn std::io::Write) -> std::io::Result<()> {
        writeln!(out, "IR Sanity Error: {}", self)?;
        write!(out, "Location: ")?;
        let loc = self.get_location(module);
        loc.describe(module, out);
        Ok(())
    }

    pub fn get_location(&self, module: &Module) -> IRLocation {
        let allocs = &module.allocs;
        match self {
            IRSanityErr::DeadGlobal { id, .. } => match id.deref_ir(allocs) {
                GlobalObj::Var(_) => IRLocation::GlobalVar(GlobalVarID::raw_from(*id)),
                GlobalObj::Func(_) => IRLocation::Func(FuncID::raw_from(*id)),
            },
            IRSanityErr::DeadBlock(block_id) => IRLocation::Block(*block_id),
            IRSanityErr::DeadInst(inst) => IRLocation::Inst(*inst),
            IRSanityErr::DeadConstExpr(expr) => IRLocation::Operand(expr.into_ir()),
            IRSanityErr::TraceableMissingUse(use_id) => IRLocation::Use(*use_id),
            IRSanityErr::UseMissingOperand(use_id, _) => IRLocation::Use(*use_id),
            IRSanityErr::BrokenUserRing(op) => IRLocation::Operand(*op),
            IRSanityErr::DeadUser(user) => IRLocation::Operand(user.into_ir()),
            IRSanityErr::DeadUserUse(user_id, ..)
            | IRSanityErr::InConsistentOperandUse(user_id, ..) => match user_id {
                UserID::Expr(expr) => IRLocation::Operand(expr.into_ir()),
                UserID::Inst(inst) => IRLocation::Inst(*inst),
                UserID::Global(glob) => match glob.deref_ir(allocs) {
                    GlobalObj::Var(_) => IRLocation::GlobalVar(GlobalVarID::raw_from(*glob)),
                    GlobalObj::Func(_) => IRLocation::Func(FuncID::raw_from(*glob)),
                },
            },
            IRSanityErr::DeadJT(jt)
            | IRSanityErr::JTPredMismatch(jt, ..)
            | IRSanityErr::InConsistentJT(jt, ..)
            | IRSanityErr::JTBlockMissing(jt, ..)
            | IRSanityErr::JumpToNonLocalBlock(jt, ..)
            | IRSanityErr::JTBlockUnattached(jt, ..) => IRLocation::JumpTarget(*jt),
            IRSanityErr::BrokenJTRing(block_id) => IRLocation::Block(*block_id),
            IRSanityErr::FuncArgIndexMismatch(arg) => IRLocation::Operand(arg.into_ir()),
            IRSanityErr::InvalidFuncEntryPos(func_id, ..)
            | IRSanityErr::BlockNotAttached(func_id, ..) => IRLocation::Func(*func_id),
            IRSanityErr::BlockHasMultipleTerminators(block_id)
            | IRSanityErr::BlockTerminatorPosIncorrect(block_id)
            | IRSanityErr::BlockPhiEndMissing(block_id)
            | IRSanityErr::BlockPhiSegmentErr(block_id, _) => IRLocation::Block(*block_id),
            IRSanityErr::TypeErr(_) => IRLocation::Module,
            IRSanityErr::ValueNotType(val, ..) => IRLocation::Operand(*val),
            IRSanityErr::InstNotType(inst, ..) | IRSanityErr::InstTypeNotClass(inst, ..) => {
                IRLocation::Inst(*inst)
            }
            IRSanityErr::OperandNotType(use_id, ..)
            | IRSanityErr::OperandTypeNotVecOrElem(use_id, ..)
            | IRSanityErr::OperandTypeNotClass(use_id, ..)
            | IRSanityErr::OperandNotClass(use_id, ..) => IRLocation::Use(*use_id),
            IRSanityErr::InstNotAttached(inst_id) => IRLocation::Inst(*inst_id),
            IRSanityErr::GEPUnpackErr(gepinst_id, _) => IRLocation::Inst(gepinst_id.raw_into()),
            IRSanityErr::CastErr(castinst_id, _) => IRLocation::Inst(castinst_id.raw_into()),
            IRSanityErr::PhiErr(phiinst_id, _) => IRLocation::Inst(phiinst_id.raw_into()),
        }
    }
}

pub fn assert_module_sane(module: &Module) {
    let Err(err) = basic_sanity_check(module) else {
        return;
    };
    err.explain(module, &mut std::io::stdout()).unwrap();
    panic!("IR Sanity Check Failed");
}
pub fn basic_sanity_check(module: &Module) -> IRSanityRes {
    let ctx = SanityCheckCtx::new(module);
    ctx.module_sane()
}

enum ExprSet {
    Dense(FixBitSet<3>),
    Sparse(HashSet<ExprID>),
}
impl ExprSet {
    fn new(alloc: &EntityAlloc<ExprObj>) -> Self {
        let alloc_cap = alloc.capacity();
        let alloc_len = alloc.len();
        if alloc_cap / 4 <= alloc_len {
            Self::Dense(FixBitSet::with_len(alloc_cap))
        } else {
            Self::Sparse(HashSet::with_capacity(alloc_len))
        }
    }

    fn insert(&mut self, allocs: &IRAllocs, eid: ExprID) {
        match self {
            ExprSet::Dense(bs) => {
                bs.enable(eid.into_raw_ptr().get_index(&allocs.exprs).unwrap());
            }
            ExprSet::Sparse(set) => {
                set.insert(eid);
            }
        }
    }
    fn contains(&self, allocs: &IRAllocs, eid: ExprID) -> bool {
        match self {
            ExprSet::Dense(bs) => {
                let Some(idx) = eid.into_raw_ptr().get_index(&allocs.exprs) else {
                    return false;
                };
                bs.get(idx)
            }
            ExprSet::Sparse(set) => set.contains(&eid),
        }
    }
}

struct SanityCheckCtx<'ir> {
    module: &'ir Module,
    curr_func: Cell<Option<FuncID>>,
    curr_block: Cell<Option<BlockID>>,
    exprs: RefCell<(ExprSet, VecDeque<ExprID>)>,
}

impl<'ir> SanityCheckCtx<'ir> {
    fn new(module: &'ir Module) -> Self {
        let exprs = ExprSet::new(&module.allocs.exprs);
        let expr_queue = VecDeque::new();
        Self {
            module,
            curr_func: Cell::new(None),
            curr_block: Cell::new(None),
            exprs: RefCell::new((exprs, expr_queue)),
        }
    }
    fn begin_func(&self, fid: FuncID) -> impl Drop + '_ {
        struct Guard<'a>(&'a SanityCheckCtx<'a>, Option<FuncID>);
        impl<'a> Drop for Guard<'a> {
            fn drop(&mut self) {
                self.0.curr_func.set(self.1);
            }
        }
        let prev = self.curr_func.get();
        self.curr_func.set(Some(fid));
        Guard(self, prev)
    }
    fn begin_block(&self, bid: BlockID) -> impl Drop + '_ {
        struct Guard<'a>(&'a SanityCheckCtx<'a>, Option<BlockID>);
        impl<'a> Drop for Guard<'a> {
            fn drop(&mut self) {
                self.0.curr_block.set(self.1);
            }
        }
        let prev = self.curr_block.get();
        self.curr_block.set(Some(bid));
        Guard(self, prev)
    }

    fn allocs(&self) -> &IRAllocs {
        &self.module.allocs
    }
    fn tctx(&self) -> &TypeContext {
        &self.module.tctx
    }
    fn symbols(&self) -> Ref<'_, HashMap<Arc<str>, GlobalID>> {
        self.module.symbols.borrow()
    }
    fn push_mark_expr(&self, eid: ExprID) {
        let mut borrow = self.exprs.borrow_mut();
        let (expr_set, expr_queue) = &mut *borrow;
        if !expr_set.contains(self.allocs(), eid) {
            expr_set.insert(self.allocs(), eid);
            expr_queue.push_back(eid);
        }
    }
    fn curr_func_retty(&self) -> ValTypeID {
        let Some(fid) = self.curr_func.get() else {
            panic!("No current function in sanity check context");
        };
        fid.deref_ir(self.allocs()).ret_type
    }

    fn module_sane(&self) -> IRSanityRes {
        let allocs = self.allocs();

        for (name, &gid) in self.symbols().iter() {
            if !GlobalObj::id_is_live(gid, allocs) {
                let name = Some(Arc::clone(name));
                return Err(IRSanityErr::DeadGlobal { name, id: gid });
            }
            match gid.deref_ir(allocs) {
                GlobalObj::Var(gvar) => self.global_var_sane(GlobalVarID::raw_from(gid), gvar),
                GlobalObj::Func(func) => self.func_sane(FuncID::raw_from(gid), func),
            }?;
        }
        self.all_operands_sane()
    }

    fn traceable_sane(&self, val: ValueSSA, obj: &impl ITraceableValue) -> IRSanityRes {
        let Some(users) = obj.try_get_users() else {
            return Ok(());
        };
        let sentinel = users.sentinel;
        let mut curr = sentinel;
        let allocs = self.allocs();
        loop {
            let Some(use_obj) = curr.inner().try_deref(&allocs.uses) else {
                return Err(IRSanityErr::TraceableMissingUse(curr));
            };
            if use_obj.get_kind() == UseKind::DisposedUse {
                return Err(IRSanityErr::TraceableMissingUse(curr));
            }
            let operand = use_obj.operand.get();
            if operand != val {
                return Err(IRSanityErr::UseMissingOperand(curr, use_obj.get_kind()));
            }
            let Some(next_ptr) = use_obj.get_next_id() else {
                return Err(IRSanityErr::TraceableMissingUse(curr));
            };
            if next_ptr == sentinel {
                break Ok(());
            }
            curr = next_ptr;
        }
    }

    fn user_sane(&self, user_id: UserID, obj: &impl IUser) -> IRSanityRes {
        self.traceable_sane(user_id.into_ir(), obj)?;
        let allocs = self.allocs();
        for (idx, &use_id) in obj.get_operands().iter().enumerate() {
            let idx = idx as u32;
            let Some(use_obj) = use_id.inner().try_deref(&allocs.uses) else {
                return Err(IRSanityErr::DeadUserUse(user_id, idx, use_id));
            };
            if use_obj.get_kind() == UseKind::DisposedUse {
                return Err(IRSanityErr::DeadUserUse(user_id, idx, use_id));
            }
            if use_obj.user.get() != Some(user_id) {
                return Err(IRSanityErr::InConsistentOperandUse(user_id, idx, use_id));
            }
            if let ValueSSA::ConstExpr(exp) = use_obj.operand.get() {
                self.push_mark_expr(exp);
            }
        }
        Ok(())
    }

    fn global_var_sane(&self, gid: GlobalVarID, gvar: &GlobalVar) -> IRSanityRes {
        self.user_sane(UserID::Global(gid.raw_into()), gvar)
    }

    fn func_sane(&self, fid: FuncID, func: &FuncObj) -> IRSanityRes {
        self.user_sane(UserID::Global(fid.raw_into()), func)?;
        for (idx, arg) in func.args.iter().enumerate() {
            let arg_id = FuncArgID(fid, arg.index);
            if arg.index as usize != idx {
                return Err(IRSanityErr::FuncArgIndexMismatch(arg_id));
            }
            self.traceable_sane(arg_id.into_ir(), arg)?;
        }
        let Some(body) = &func.body else {
            return Ok(());
        };
        let mut entry_pos = None;
        let allocs = self.allocs();
        let func_guard = self.begin_func(fid);
        for (idx, (bid, bb)) in body.blocks.iter(&allocs.blocks).enumerate() {
            if !BlockObj::id_is_live(bid, allocs) {
                return Err(IRSanityErr::DeadBlock(bid));
            }
            if bid == body.entry {
                entry_pos = Some(idx);
            }
            if bb.get_parent_func() != Some(fid) {
                return Err(IRSanityErr::BlockNotAttached(fid, bid));
            }
            self.block_sane(bid, bb)?;
        }
        drop(func_guard);
        if entry_pos != Some(0) {
            let pos = entry_pos.map_or(usize::MAX, |p| p);
            return Err(IRSanityErr::InvalidFuncEntryPos(fid, body.entry, pos));
        }
        Ok(())
    }

    fn block_sane(&self, bid: BlockID, block: &BlockObj) -> IRSanityRes {
        self.traceable_sane(ValueSSA::Block(bid), block)?;
        let body = block.get_body();
        let phiend = body.phi_end;
        let preds = &body.preds;

        let allocs = self.allocs();
        let mut has_phiend = false;
        let mut terminator = None;
        let block_guard = self.begin_block(bid);
        for (inst_id, inst) in body.insts.iter(&allocs.insts) {
            if !InstObj::id_is_live(inst_id, allocs) {
                return Err(IRSanityErr::DeadInst(inst_id));
            }
            if inst_id == phiend {
                has_phiend = true;
            }
            if matches!(inst, InstObj::Phi(_)) == has_phiend {
                return Err(IRSanityErr::BlockPhiSegmentErr(bid, inst_id));
            }
            if let Some(termi) = TerminatorID::try_from_ir(allocs, inst_id) {
                if let Some(_) = terminator {
                    return Err(IRSanityErr::BlockHasMultipleTerminators(bid));
                }
                terminator = Some(termi);
            }
            self.inst_sane(inst_id, inst)?;
        }
        drop(block_guard);
        if !has_phiend {
            return Err(IRSanityErr::BlockPhiEndMissing(bid));
        }
        let terminator = terminator.map(TerminatorID::into_ir);
        if body.insts.get_back_id(&allocs.insts) != terminator {
            return Err(IRSanityErr::BlockTerminatorPosIncorrect(bid));
        }

        let sentinel = preds.sentinel;
        let mut curr = sentinel;
        loop {
            let Some(jt_obj) = curr.inner().try_deref(&allocs.jts) else {
                return Err(IRSanityErr::DeadJT(curr));
            };
            let jt_kind = jt_obj.get_kind();
            if jt_kind == JumpTargetKind::Disposed {
                return Err(IRSanityErr::DeadJT(curr));
            }
            if Some(bid) != jt_obj.block.get() {
                return Err(IRSanityErr::JTPredMismatch(
                    curr,
                    jt_kind,
                    bid,
                    jt_obj.block.get(),
                ));
            }
            let Some(next_ptr) = jt_obj.get_next_id() else {
                return Err(IRSanityErr::BrokenJTRing(bid));
            };
            if next_ptr == sentinel {
                break;
            }
            curr = next_ptr;
        }
        Ok(())
    }

    fn terminator_sane(&self, term_id: TerminatorID) -> IRSanityRes {
        let allocs = self.allocs();
        for jt in term_id.get_jts(allocs) {
            let kind = jt.get_kind(allocs);
            if Some(term_id.into_ir()) != jt.get_terminator(allocs) {
                return Err(IRSanityErr::InConsistentJT(jt, kind));
            }
            let Some(bb) = jt.get_block(allocs) else {
                return Err(IRSanityErr::JTBlockMissing(jt, kind));
            };
            let Some(bb_parent) = bb.get_parent_func(allocs) else {
                return Err(IRSanityErr::JTBlockUnattached(jt, kind, bb));
            };
            let curr_func = self.curr_func.get().unwrap();
            if bb_parent != curr_func {
                return Err(IRSanityErr::JumpToNonLocalBlock(jt, kind, bb, bb_parent));
            }
        }
        Ok(())
    }

    fn inst_sane(&self, inst_id: InstID, inst: &InstObj) -> IRSanityRes {
        self.user_sane(UserID::Inst(inst_id), inst)?;
        use InstObj::*;
        let allocs = self.allocs();
        match inst {
            GuideNode(_) | PhiInstEnd(_) | Unreachable(_) => Ok(()),
            InstObj::Ret(ret) => {
                let retty = self.curr_func_retty();
                self.use_type_match(ret.retval_use(), retty)
            }
            InstObj::Jump(_) => {
                self.terminator_sane(TerminatorID::Jump(JumpInstID::raw_from(inst_id)))
            }
            InstObj::Br(br) => {
                self.use_type_match(br.cond_use(), ValTypeID::Int(1))?;
                self.terminator_sane(TerminatorID::Br(BrInstID::raw_from(inst_id)))
            }
            InstObj::Switch(sw) => {
                self.use_typeclass_match(sw.discrim_use(), ValTypeClass::Int)?;
                self.terminator_sane(TerminatorID::Switch(SwitchInstID::raw_from(inst_id)))
            }

            // Memory Operation
            InstObj::Alloca(_) => Ok(()),
            InstObj::GEP(gep) => {
                let inst_id = GEPInstID::raw_from(inst_id);
                self.use_type_match(gep.base_use(), ValTypeID::Ptr)?;
                GEPTypeIter::new(self.tctx(), allocs, inst_id)
                    .run_sanity_check()
                    .map_err(|e| IRSanityErr::GEPUnpackErr(inst_id, e))
            }
            InstObj::Load(load) => self.use_type_match(load.source_use(), ValTypeID::Ptr),
            InstObj::Store(store) => {
                self.inst_type_match(inst_id, ValTypeID::Void)?;
                self.use_type_match(store.source_use(), store.source_ty)?;
                self.use_type_match(store.target_use(), ValTypeID::Ptr)
            }
            InstObj::AmoRmw(amormw) => {
                let valty = amormw.get_valtype();
                self.use_type_match(amormw.pointer_use(), ValTypeID::Ptr)?;
                self.use_type_match(amormw.value_use(), valty)
            }
            InstObj::BinOP(binop) => self.inst_sane_binop(inst_id, binop),
            InstObj::Call(call) => self.inst_sane_callop(call),
            InstObj::Cast(cast) => self.inst_sane_cast(inst_id, cast),
            InstObj::Cmp(cmp) => self.inst_sane_cmp(inst_id, cmp),
            InstObj::IndexExtract(_)
            | InstObj::FieldExtract(_)
            | InstObj::IndexInsert(_)
            | InstObj::FieldInsert(_) => todo!("How to implement?"),
            InstObj::Phi(phi) => self.inst_sane_phi(inst_id, phi),
            InstObj::Select(select) => {
                let valty = select.get_valtype();
                self.use_type_match(select.then_use(), valty)?;
                self.use_type_match(select.else_use(), valty)?;
                self.use_type_match(select.cond_use(), ValTypeID::Int(1))
            }
        }
    }

    fn inst_sane_binop(&self, inst_id: InstID, binop: &BinOPInst) -> IRSanityRes {
        let opcode = binop.get_opcode();
        let allocs = self.allocs();
        assert!(opcode.is_binary_op());
        self.use_type_match(binop.lhs_use(), binop.get_valtype())?;
        use crate::ir::Opcode::*;
        match opcode {
            Add | Sub | Mul | Sdiv | Udiv | Srem | Urem | BitAnd | BitOr | BitXor => {
                self.inst_typeclass_match_or_vec(inst_id, ValTypeClass::Int)?;
                self.use_type_match(binop.rhs_use(), binop.get_valtype())
            }
            Fadd | Fsub | Fmul | Fdiv | Frem => {
                self.inst_typeclass_match_or_vec(inst_id, ValTypeClass::Float)?;
                self.use_type_match(binop.rhs_use(), binop.get_valtype())
            }
            Shl | Lshr | Ashr => {
                self.inst_typeclass_match_or_vec(inst_id, ValTypeClass::Int)?;
                let ValTypeID::FixVec(FixVecType(s, n)) = binop.get_valtype() else {
                    return self.use_typeclass_match(binop.rhs_use(), ValTypeClass::Int);
                };
                let ScalarType::Int(_) = s else {
                    unreachable!("Shift amount must be integer type");
                };
                let rhsty = binop.get_rhs(allocs).get_valtype(allocs);
                match rhsty {
                    ValTypeID::Int(_) => Ok(()),
                    ValTypeID::FixVec(FixVecType(ScalarType::Int(_), m)) if m == n => Ok(()),
                    _ => Err(IRSanityErr::OperandTypeNotVecOrElem(
                        binop.rhs_use(),
                        UseKind::BinOpRhs,
                        FixVecType(s, n),
                        rhsty,
                    )),
                }
            }
            _ => panic!("Unhandled BinOP opcode in sanity check"),
        }
    }
    fn inst_sane_callop(&self, call: &CallInst) -> Result<(), IRSanityErr> {
        let allocs = self.allocs();
        self.use_type_match(call.callee_use(), ValTypeID::Ptr)?;
        let callee = call.get_callee(allocs);
        let calleety = call.callee_ty;
        if let Some(func) = callee.as_dyn_ptrvalue(allocs) {
            let functy = func.get_ptr_pointee_type();
            let calleety = calleety.into_ir();
            if functy != calleety {
                return Err(TypeMismatchErr::IDNotEqual(calleety, functy).into());
            }
        }
        for (idx, &uarg) in call.arg_uses().iter().enumerate() {
            let Some(&func_argty) = calleety.get_args(self.tctx()).get(idx) else {
                // encountered variadic args, accept remaining args as-is
                break;
            };
            self.use_type_match(uarg, func_argty)?;
        }
        Ok(())
    }
    fn inst_sane_cast(&self, inst_id: InstID, cast: &CastInst) -> IRSanityRes {
        let from_ty = cast.from_ty;
        let into_ty = cast.get_valtype();
        let cast_id = CastInstID::raw_from(inst_id);
        self.use_type_match(cast.from_use(), from_ty)?;
        let opcode = cast.get_opcode();
        match opcode {
            Opcode::Zext | Opcode::Sext => {
                let from_ty = self.useid_as::<IntType>(cast.from_use(), from_ty)?;
                let into_ty = Self::inst_as::<IntType>(inst_id, into_ty)?;
                if from_ty.0 <= into_ty.0 {
                    Ok(())
                } else {
                    Err(IRSanityErr::CastErr(
                        cast_id,
                        CastErr::IntExtToSmaller(from_ty, into_ty),
                    ))
                }
            }
            Opcode::Trunc => {
                let IntType(from_bits) = self.useid_as::<IntType>(cast.from_use(), from_ty)?;
                let IntType(into_bits) = Self::inst_as::<IntType>(inst_id, into_ty)?;
                if from_bits >= into_bits {
                    Ok(())
                } else {
                    Err(IRSanityErr::CastErr(
                        cast_id,
                        CastErr::IntTruncToLarger(IntType(from_bits), IntType(into_bits)),
                    ))
                }
            }
            Opcode::Bitcast => Ok(()),
            Opcode::PtrToInt => {
                self.useid_as::<PtrType>(cast.from_use(), from_ty)?;
                Self::inst_as::<IntType>(inst_id, into_ty).map(drop)
            }
            Opcode::IntToPtr => {
                self.useid_as::<IntType>(cast.from_use(), from_ty)?;
                Self::inst_as::<PtrType>(inst_id, into_ty).map(drop)
            }

            Opcode::Sitofp | Opcode::Uitofp => {
                self.useid_as::<IntType>(cast.from_use(), from_ty)?;
                Self::inst_as::<FPKind>(inst_id, into_ty).map(drop)
            }
            Opcode::Fptosi | Opcode::Fptoui => {
                self.useid_as::<FPKind>(cast.from_use(), from_ty)?;
                Self::inst_as::<IntType>(inst_id, into_ty).map(drop)
            }
            Opcode::Fpext | Opcode::Fptrunc => {
                let from_fk = self.useid_as::<FPKind>(cast.from_use(), from_ty)?;
                let into_fk = Self::inst_as::<FPKind>(inst_id, into_ty)?;
                match (opcode, from_fk, into_fk) {
                    (Opcode::Fpext, FPKind::Ieee64, FPKind::Ieee32) => Err(IRSanityErr::CastErr(
                        cast_id,
                        CastErr::FPExtToSmaller(FPKind::Ieee64, FPKind::Ieee32),
                    )),
                    (Opcode::Fptrunc, FPKind::Ieee32, FPKind::Ieee64) => Err(IRSanityErr::CastErr(
                        cast_id,
                        CastErr::FPTruncToLarger(FPKind::Ieee32, FPKind::Ieee64),
                    )),
                    _ => Ok(()),
                }
            }
            _ => panic!("Unhandled Cast opcode in sanity check"),
        }
    }
    fn inst_sane_cmp(&self, inst_id: InstID, cmp: &CmpInst) -> IRSanityRes {
        let allocs = self.allocs();
        self.inst_type_match(inst_id, ValTypeID::Int(1))?;
        let lhs_ty = cmp.get_lhs(allocs).get_valtype(allocs);
        let rhs_ty = cmp.get_rhs(allocs).get_valtype(allocs);
        if lhs_ty != rhs_ty {
            return Err(IRSanityErr::OperandNotType(
                cmp.rhs_use(),
                UseKind::CmpRhs,
                lhs_ty,
                rhs_ty,
            ));
        }
        match cmp.get_opcode() {
            Opcode::Icmp => self.use_typeclass_match(cmp.lhs_use(), ValTypeClass::Int),
            Opcode::Fcmp => self.use_typeclass_match(cmp.lhs_use(), ValTypeClass::Float),
            _ => panic!("Unhandled Cmp opcode in sanity check"),
        }
    }
    fn inst_sane_phi(&self, inst_id: InstID, phi: &PhiInst) -> IRSanityRes {
        let allocs = self.allocs();
        let mut phi_has_incoming: HashMap<BlockID, bool> = HashMap::new();
        let Some(curr_block) = self.curr_block.get() else {
            panic!("No current block in sanity check context");
        };
        let preds = curr_block.get_preds(allocs).iter(&allocs.jts);
        for (_, jt) in preds {
            let termi = jt.terminator.get().unwrap();
            let Some(from_bb) = termi.get_parent(allocs) else {
                return Err(IRSanityErr::InstNotAttached(termi));
            };
            phi_has_incoming.insert(from_bb, false);
        }
        for [ublk, uval] in &*phi.incoming_uses() {
            let ublk_op = ublk.get_operand(allocs);
            let ValueSSA::Block(bb) = ublk_op else {
                let ublk_kind = ublk.get_kind(allocs);
                return Err(IRSanityErr::OperandNotClass(
                    *ublk,
                    ublk_kind,
                    ValueClass::Block,
                    ublk_op,
                ));
            };
            self.use_type_match(*uval, phi.get_valtype())?;
            let Some(already_has) = phi_has_incoming.get_mut(&bb) else {
                return Err(IRSanityErr::PhiErr(
                    PhiInstID::raw_from(inst_id),
                    PhiInstErr::IncomingNotInPreds(*ublk, bb),
                ));
            };
            if *already_has {
                return Err(IRSanityErr::PhiErr(
                    PhiInstID::raw_from(inst_id),
                    PhiInstErr::DuplicateIncoming(bb),
                ));
            }
            *already_has = true;
        }
        for (pred_bb, has_incoming) in phi_has_incoming {
            if !has_incoming {
                return Err(IRSanityErr::PhiErr(
                    PhiInstID::raw_from(inst_id),
                    PhiInstErr::MissingIncoming(pred_bb),
                ));
            }
        }
        Ok(())
    }

    fn use_type_match(&self, uid: UseID, ty: ValTypeID) -> IRSanityRes {
        let val = uid.get_operand(self.allocs());
        let valty = val.get_valtype(self.allocs());
        if valty == ty {
            Ok(())
        } else {
            let use_kind = uid.get_kind(self.allocs());
            Err(IRSanityErr::OperandNotType(uid, use_kind, ty, valty))
        }
    }
    fn inst_type_match(&self, inst_id: InstID, ty: ValTypeID) -> IRSanityRes {
        let valty = inst_id.get_valtype(self.allocs());
        if valty == ty {
            Ok(())
        } else {
            Err(IRSanityErr::ValueNotType(
                ValueSSA::Inst(inst_id),
                ty,
                valty,
            ))
        }
    }
    fn use_typeclass_match(&self, uid: UseID, rclass: ValTypeClass) -> IRSanityRes {
        let val = uid.get_operand(self.allocs());
        let valty = val.get_valtype(self.allocs());
        if valty.class_id() == rclass {
            Ok(())
        } else {
            let kind = uid.get_kind(self.allocs());
            Err(IRSanityErr::OperandTypeNotClass(uid, kind, rclass, valty))
        }
    }
    fn inst_typeclass_match_or_vec(&self, inst_id: InstID, rclass: ValTypeClass) -> IRSanityRes {
        let valty = inst_id.get_valtype(self.allocs());
        if valty.class_id() == rclass {
            Ok(())
        } else if let ValTypeID::FixVec(FixVecType(s, _)) = valty {
            let elemty = s.into_ir();
            if elemty.class_id() == rclass {
                Ok(())
            } else {
                Err(IRSanityErr::InstTypeNotClass(inst_id, rclass, valty))
            }
        } else {
            Err(IRSanityErr::InstTypeNotClass(inst_id, rclass, valty))
        }
    }
    fn useid_as<T: IValType>(&self, useid: UseID, from_ty: ValTypeID) -> IRSanityRes<T> {
        match T::try_from_ir(from_ty) {
            Ok(r) => Ok(r),
            Err(TypeMismatchErr::NotClass(_, klass)) => Err(IRSanityErr::OperandTypeNotClass(
                useid,
                useid.get_kind(self.allocs()),
                klass,
                from_ty,
            )),
            Err(e) => Err(e.into()),
        }
    }
    fn inst_as<T: IValType>(inst: InstID, into_ty: ValTypeID) -> IRSanityRes<T> {
        match T::try_from_ir(into_ty) {
            Ok(r) => Ok(r),
            Err(TypeMismatchErr::NotClass(t, c)) => Err(IRSanityErr::InstTypeNotClass(inst, c, t)),
            Err(e) => Err(e.into()),
        }
    }

    fn all_operands_sane(&self) -> IRSanityRes {
        let mut queue = std::mem::take(&mut self.exprs.borrow_mut().1);
        while !queue.is_empty() {
            while let Some(eid) = queue.pop_front() {
                self.expr_sane(eid)?;
            }
            let mut borrow = self.exprs.borrow_mut();
            std::mem::swap(&mut queue, &mut borrow.1);
        }
        Ok(())
    }
    fn expr_sane(&self, expr: ExprID) -> IRSanityRes {
        let allocs = self.allocs();
        let expr_obj = expr.deref_ir(allocs);
        self.user_sane(UserID::Expr(expr), expr_obj)?;

        match expr_obj {
            ExprObj::Array(arr) => {
                let elemty = arr.elemty;
                for &uelem in arr.elems.iter() {
                    self.use_type_match(uelem, elemty)?;
                }
                Ok(())
            }
            ExprObj::Struct(struc) => {
                let structy = struc.structty;
                let fields = structy.get_fields(self.tctx());
                for (idx, &ufield) in struc.fields.iter().enumerate() {
                    self.use_type_match(ufield, fields[idx])?;
                }
                Ok(())
            }
            ExprObj::FixVec(vec) => {
                let elemty = vec.vecty.get_elem().into_ir();
                for &uelem in vec.elems.iter() {
                    self.use_type_match(uelem, elemty)?;
                }
                Ok(())
            }
        }
    }
}

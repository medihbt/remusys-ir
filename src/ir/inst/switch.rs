use crate::{
    _remusys_ir_subinst,
    ir::{
        BlockID, IRAllocs, ISubInst, ISubInstID, ISubValueSSA, ITerminatorInst, IUser, InstCommon,
        InstObj, JumpTargetID, JumpTargetKind, JumpTargets, Opcode, OperandSet, UseID, UseKind,
        ValueSSA,
    },
    typing::{IntType, ValTypeID},
};
use smallvec::{SmallVec, smallvec};
use std::{
    cell::{Ref, RefCell, RefMut},
    collections::HashMap,
    ops::RangeFrom,
};

/// Switch 指令：根据条件值跳转到不同的基本块
///
/// ### LLVM IR 语法
///
/// ```llvm
/// switch <intty> <value>, label <defaultdest> [
///     <intty> <val0>, label <dest0>
///     <intty> <val1>, label <dest1>
///     <intty> <val2>, label <dest2>
///     ...
/// ]
/// ```
///
/// ### 操作数布局
///
/// - **条件操作数**: 一个整数类型的值，用于匹配各个 case
///
/// ### 跳转目标布局
///
/// - `targets[0]`: 默认跳转目标 (`JumpTargetKind::SwitchDefault`)
/// - `targets[1..]`: 各个 case 跳转目标 (`JumpTargetKind::SwitchCase(value)`)
///
/// ### 语义
///
/// 1. 计算条件操作数的值
/// 2. 按顺序查找匹配的 case 值
/// 3. 如果找到匹配的 case，跳转到对应的基本块
/// 4. 如果没有匹配的 case，跳转到默认基本块
///
/// ## 约束
///
/// - 条件操作数必须是整数类型
/// - 每个 case 值必须唯一
/// - 必须有且仅有一个默认跳转目标
pub struct SwitchInst {
    pub common: InstCommon,
    pub discrim_ty: IntType,
    discrim: [UseID; 1],
    targets: RefCell<SmallVec<[JumpTargetID; 4]>>,
}

impl IUser for SwitchInst {
    fn get_operands(&self) -> OperandSet<'_> {
        OperandSet::Fixed(&self.discrim)
    }
    fn operands_mut(&mut self) -> &mut [UseID] {
        &mut self.discrim
    }
}
impl ISubInst for SwitchInst {
    fn get_common(&self) -> &InstCommon {
        &self.common
    }
    fn common_mut(&mut self) -> &mut InstCommon {
        &mut self.common
    }
    fn try_from_ir_ref(inst: &InstObj) -> Option<&Self> {
        match inst {
            InstObj::Switch(s) => Some(s),
            _ => None,
        }
    }
    fn try_from_ir_mut(inst: &mut InstObj) -> Option<&mut Self> {
        match inst {
            InstObj::Switch(s) => Some(s),
            _ => None,
        }
    }
    fn try_from_ir(inst: InstObj) -> Option<Self> {
        match inst {
            InstObj::Switch(s) => Some(s),
            _ => None,
        }
    }
    fn into_ir(self) -> InstObj {
        InstObj::Switch(self)
    }

    fn is_terminator(&self) -> bool {
        true
    }
    fn try_get_jts(&self) -> Option<JumpTargets<'_>> {
        let targets = Ref::map(self.targets.borrow(), SmallVec::as_slice);
        Some(JumpTargets::Dyn(targets))
    }
}
impl ITerminatorInst for SwitchInst {
    fn get_jts(&self) -> JumpTargets<'_> {
        let targets = Ref::map(self.targets.borrow(), SmallVec::as_slice);
        JumpTargets::Dyn(targets)
    }
    fn jts_mut(&mut self) -> &mut [JumpTargetID] {
        self.targets.get_mut().as_mut_slice()
    }
    fn terminates_function(&self) -> bool {
        false
    }
}
impl SwitchInst {
    pub const OP_DISCRIM: usize = 0;
    pub const JT_DEFAULT: usize = 0;
    pub const JT_CASES: RangeFrom<usize> = 1..;
    pub const JT_CASE_START: usize = 1;

    pub fn new_uninit(allocs: &IRAllocs, discrim_ty: IntType) -> Self {
        let default = JumpTargetID::new(allocs, JumpTargetKind::SwitchDefault);
        Self {
            common: InstCommon::new(Opcode::Switch, ValTypeID::Void),
            discrim_ty,
            discrim: [UseID::new(allocs, UseKind::SwitchCond)],
            targets: RefCell::new(smallvec![default]),
        }
    }
    pub fn builder(discrim_ty: IntType) -> SwitchInstBuilder {
        SwitchInstBuilder::new(discrim_ty)
    }

    pub fn discrim_use(&self) -> UseID {
        self.discrim[Self::OP_DISCRIM]
    }
    pub fn get_discrim(&self, allocs: &IRAllocs) -> ValueSSA {
        self.discrim_use().get_operand(allocs)
    }
    pub fn set_discrim(&self, allocs: &IRAllocs, val: ValueSSA) {
        self.discrim_use().set_operand(allocs, val);
    }

    pub fn default_jt(&self) -> JumpTargetID {
        self.targets.borrow()[Self::JT_DEFAULT]
    }
    pub fn get_default_bb(&self, allocs: &IRAllocs) -> Option<BlockID> {
        self.default_jt().get_block(allocs)
    }
    pub fn set_default_bb(&self, allocs: &IRAllocs, bb: BlockID) {
        self.default_jt().set_block(allocs, bb);
    }

    pub fn case_jts(&self) -> Ref<'_, [JumpTargetID]> {
        Ref::map(self.targets.borrow(), |targets| &targets[Self::JT_CASES])
    }
    pub fn case_jts_mut(&self) -> RefMut<'_, [JumpTargetID]> {
        RefMut::map(self.targets.borrow_mut(), |targets| {
            &mut targets[Self::JT_CASES]
        })
    }
    pub fn find_case_pos(&self, allocs: &IRAllocs, case_val: i64) -> Option<usize> {
        self.case_jts()
            .iter()
            .position(|jt| jt.get_kind(allocs) == JumpTargetKind::SwitchCase(case_val))
    }
    pub fn find_case_jt(&self, allocs: &IRAllocs, case_val: i64) -> Option<JumpTargetID> {
        self.case_jts()
            .iter()
            .find(|jt| jt.get_kind(allocs) == JumpTargetKind::SwitchCase(case_val))
            .copied()
    }
    pub fn get_case_by_index(&self, allocs: &IRAllocs, case_index: usize) -> Option<BlockID> {
        self.case_jts()
            .get(case_index)
            .and_then(|jt| jt.get_block(allocs))
    }
    pub fn find_case(&self, allocs: &IRAllocs, case_val: i64) -> Option<BlockID> {
        self.find_case_jt(allocs, case_val)
            .and_then(|jt| jt.get_block(allocs))
    }
    pub fn find_or_insert_case(&self, allocs: &IRAllocs, case_val: i64) -> JumpTargetID {
        if let Some(jt) = self.find_case_jt(allocs, case_val) {
            jt
        } else {
            self.push_case_jt(allocs, case_val)
        }
    }
    pub fn find_set_case(&self, allocs: &IRAllocs, case_val: i64, bb: BlockID) -> JumpTargetID {
        let jt = self.find_or_insert_case(allocs, case_val);
        jt.set_block(allocs, bb);
        jt
    }
    pub fn remove_case(&self, allocs: &IRAllocs, case_val: i64) -> bool {
        let Some(pos) = self.find_case_pos(allocs, case_val) else {
            return false;
        };
        let jt = self.remove_case_jt(pos);
        jt.dispose(allocs)
            .expect("Broken IR invariant in SwitchInst::remove_case");
        true
    }
    pub fn sort_cases(&self, allocs: &IRAllocs) {
        let mut targets = self.targets.borrow_mut();
        let cases = &mut targets[Self::JT_CASES];
        cases.sort_by_key(|jt| {
            let kind = jt.get_kind(allocs);
            let JumpTargetKind::SwitchCase(case_val) = kind else {
                unreachable!("Found non-case jump target {kind:?} in case targets");
            };
            case_val
        });
    }

    fn push_case_jt(&self, allocs: &IRAllocs, case_val: i64) -> JumpTargetID {
        let jt = JumpTargetID::new(allocs, JumpTargetKind::SwitchCase(case_val));
        // Self-reference is stored in every case jump target. SwitchInst has at least one target (the default),
        // so it's safe to get its ID here.
        let self_id = self.default_jt().get_terminator(allocs);
        if let Some(inst_id) = self_id {
            jt.set_terminator(allocs, inst_id);
        }
        self.targets.borrow_mut().push(jt);
        jt
    }
    fn remove_case_jt(&self, case_index: usize) -> JumpTargetID {
        self.targets
            .borrow_mut()
            .remove(Self::JT_CASE_START + case_index)
    }
    fn reserve_cases(&self, case_count: usize) {
        // Reserve exactly the additional number of case entries we plan to push.
        // Current length already includes the default entry at index 0, so we
        // only need to reserve `case_count` more slots for cases.
        self.targets.borrow_mut().reserve(case_count);
    }

    pub fn cases_iter<'ir>(&'ir self, allocs: &'ir IRAllocs) -> SwitchCaseIter<'ir> {
        SwitchCaseIter::new(self, allocs)
    }
}

_remusys_ir_subinst!(SwitchInstID, SwitchInst, terminator);
impl SwitchInstID {
    pub fn new_uninit(allocs: &IRAllocs, discrim_ty: IntType) -> Self {
        Self::allocate(allocs, SwitchInst::new_uninit(allocs, discrim_ty))
    }
    pub fn builder(discrim_ty: IntType) -> SwitchInstBuilder {
        SwitchInstBuilder::new(discrim_ty)
    }
    pub fn from_cases(
        allocs: &IRAllocs,
        discrim: ValueSSA,
        cases: impl IntoIterator<Item = (i64, BlockID)>,
        default_bb: BlockID,
    ) -> Self {
        let ValTypeID::Int(discrim_bits) = discrim.get_valtype(allocs) else {
            panic!("SwitchInstID::from_cases_iter: discrim must be of integer type");
        };
        SwitchInstBuilder::new(IntType(discrim_bits))
            .cases(cases)
            .discrim(discrim)
            .default_bb(default_bb)
            .build_id(allocs)
    }

    pub fn discrim_ty(self, allocs: &IRAllocs) -> IntType {
        self.deref_ir(allocs).discrim_ty
    }
    pub fn discrim_use(self, allocs: &IRAllocs) -> UseID {
        self.deref_ir(allocs).discrim_use()
    }
    pub fn get_discrim(self, allocs: &IRAllocs) -> ValueSSA {
        self.discrim_use(allocs).get_operand(allocs)
    }
    pub fn set_discrim(self, allocs: &IRAllocs, val: ValueSSA) {
        self.discrim_use(allocs).set_operand(allocs, val);
    }

    pub fn default_jt(self, allocs: &IRAllocs) -> JumpTargetID {
        self.deref_ir(allocs).default_jt()
    }
    pub fn get_default_bb(self, allocs: &IRAllocs) -> Option<BlockID> {
        self.default_jt(allocs).get_block(allocs)
    }
    pub fn set_default_bb(self, allocs: &IRAllocs, bb: BlockID) {
        self.default_jt(allocs).set_block(allocs, bb);
    }

    pub fn borrow_cases(self, allocs: &IRAllocs) -> Ref<'_, [JumpTargetID]> {
        self.deref_ir(allocs).case_jts()
    }
    pub fn find_case_jt(self, allocs: &IRAllocs, case_val: i64) -> Option<JumpTargetID> {
        self.deref_ir(allocs).find_case_jt(allocs, case_val)
    }
    pub fn find_case(self, allocs: &IRAllocs, case_val: i64) -> Option<BlockID> {
        self.deref_ir(allocs).find_case(allocs, case_val)
    }
    pub fn find_case_or_default(self, allocs: &IRAllocs, case_val: i64) -> Option<BlockID> {
        self.find_case(allocs, case_val)
            .or_else(|| self.get_default_bb(allocs))
    }
    pub fn find_set_case(self, allocs: &IRAllocs, case_val: i64, bb: BlockID) -> JumpTargetID {
        self.deref_ir(allocs).find_set_case(allocs, case_val, bb)
    }
    pub fn find_remove_case(self, allocs: &IRAllocs, case_val: i64) -> bool {
        self.deref_ir(allocs).remove_case(allocs, case_val)
    }

    pub fn cases_iter(self, allocs: &IRAllocs) -> SwitchCaseIter<'_> {
        SwitchCaseIter::new(self.deref_ir(allocs), allocs)
    }
}

pub struct SwitchInstBuilder {
    discrim_ty: IntType,
    discrim: ValueSSA,
    cases: HashMap<i64, BlockID>,
    default_bb: Option<BlockID>,
}

impl SwitchInstBuilder {
    pub fn new(discrim_ty: IntType) -> Self {
        Self {
            discrim_ty,
            discrim: ValueSSA::None,
            cases: HashMap::new(),
            default_bb: None,
        }
    }

    pub fn discrim(&mut self, val: ValueSSA) -> &mut Self {
        self.discrim = val;
        self
    }
    pub fn case(&mut self, case_val: i64, bb: BlockID) -> &mut Self {
        self.cases.insert(case_val, bb);
        self
    }
    pub fn default_bb(&mut self, bb: BlockID) -> &mut Self {
        self.default_bb = Some(bb);
        self
    }
    pub fn del_case(&mut self, case_val: i64) -> &mut Self {
        self.cases.remove(&case_val);
        self
    }
    pub fn cases(&mut self, cases: impl IntoIterator<Item = (i64, BlockID)>) -> &mut Self {
        for (case_val, bb) in cases {
            self.cases.insert(case_val, bb);
        }
        self
    }

    pub fn build_obj(&self, allocs: &IRAllocs) -> SwitchInst {
        let default_bb = self
            .default_bb
            .expect("SwitchInstBuilder::build: default_bb must be set");
        let switch = SwitchInst::new_uninit(allocs, self.discrim_ty);
        switch.set_discrim(allocs, self.discrim);
        switch.set_default_bb(allocs, default_bb);

        switch.reserve_cases(self.cases.len());
        for (&case_val, &bb) in &self.cases {
            let jt = switch.push_case_jt(allocs, case_val);
            jt.set_block(allocs, bb);
        }
        switch
    }
    pub fn build_id(&self, allocs: &IRAllocs) -> SwitchInstID {
        let switch = self.build_obj(allocs);
        SwitchInstID::allocate(allocs, switch)
    }
}

pub struct SwitchCaseIter<'ir> {
    cases: Ref<'ir, [JumpTargetID]>,
    allocs: &'ir IRAllocs,
    pos: usize,
}

impl<'ir> Iterator for SwitchCaseIter<'ir> {
    type Item = (JumpTargetID, i64, Option<BlockID>);

    fn next(&mut self) -> Option<Self::Item> {
        if self.pos >= self.cases.len() {
            return None;
        }
        let jt = self.cases[self.pos];
        self.pos += 1;
        let kind = jt.get_kind(self.allocs);
        let JumpTargetKind::SwitchCase(case_val) = kind else {
            panic!("Found non-case jump target {kind:?} in SwitchCaseIter");
        };
        let bb = jt.get_block(self.allocs);
        Some((jt, case_val, bb))
    }

    fn size_hint(&self) -> (usize, Option<usize>) {
        let remaining = self.cases.len() - self.pos;
        (remaining, Some(remaining))
    }
}
impl<'ir> ExactSizeIterator for SwitchCaseIter<'ir> {
    fn len(&self) -> usize {
        self.cases.len() - self.pos
    }
}

impl<'ir> SwitchCaseIter<'ir> {
    pub fn new(switch: &'ir SwitchInst, allocs: &'ir IRAllocs) -> Self {
        Self { cases: switch.case_jts(), allocs, pos: 0 }
    }
}

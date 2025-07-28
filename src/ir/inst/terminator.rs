use std::{
    cell::{Ref, RefCell},
    collections::BTreeSet,
};

use slab::Slab;

use crate::{
    base::{
        NullableValue,
        slablist::{SlabRefList, SlabRefListNodeRef},
        slabref::SlabRef,
    },
    ir::{
        ValueSSA,
        block::{
            BlockRef,
            jump_target::{JumpTargetData, JumpTargetKind, JumpTargetRef},
        },
        module::{Module, rcfg::RcfgAlloc},
        opcode::Opcode,
    },
    typing::id::ValTypeID,
};

use super::{
    InstData, InstDataCommon, InstDataUnique, InstError, InstRef,
    checking::{check_operand_type_kind_match, check_operand_type_match},
    usedef::{UseData, UseKind, UseRef},
};

pub trait TerminatorInst {
    fn get_jump_targets(&self) -> Option<&SlabRefList<JumpTargetRef>>;

    fn init_jump_targets(&mut self, alloc_jt: &mut Slab<JumpTargetData>);

    fn collect_jump_blocks_dedup(&self, alloc_jt: &Slab<JumpTargetData>) -> Vec<BlockRef>;
    fn collect_jump_blocks_nodedup(&self, alloc_jt: &Slab<JumpTargetData>) -> Vec<BlockRef>;

    fn get_n_jump_targets(&self) -> usize {
        self.get_jump_targets().map_or(0, |targets| targets.len())
    }

    /// Whether this terminator terminates the function control flow.
    /// True value means whether this instruction will return from the function
    /// or makes the control flow unreachable.
    fn terminates_function(&self) -> bool {
        self.get_jump_targets().is_none()
    }

    fn _jt_init_set_self_reference(&self, self_ref: InstRef, alloc_jt: &Slab<JumpTargetData>) {
        if let Some(targets) = self.get_jump_targets() {
            let mut curr_node = targets._head;
            while curr_node.is_nonnull() {
                curr_node
                    .to_slabref_unwrap(alloc_jt)
                    ._terminator
                    .set(self_ref);
                curr_node = match curr_node.get_next_ref(alloc_jt) {
                    Some(x) => x,
                    None => break,
                };
            }
        }
    }
}

pub struct Ret {
    pub retval: UseRef,
}

pub struct JumpCommon {
    pub(crate) _targets: SlabRefList<JumpTargetRef>,
    pub(crate) _condition: UseRef,
}

pub struct Jump(pub(crate) JumpCommon);
pub struct Br {
    pub(crate) _common: JumpCommon,
    pub if_true: JumpTargetRef,
    pub if_false: JumpTargetRef,
}

pub struct Switch {
    pub(crate) _common: JumpCommon,
    pub(crate) _default: JumpTargetRef,
    pub(crate) _cases: RefCell<Vec<(i128, JumpTargetRef)>>,
}

impl TerminatorInst for Ret {
    fn get_jump_targets(&self) -> Option<&SlabRefList<JumpTargetRef>> {
        None
    }
    fn init_jump_targets(&mut self, _: &mut Slab<JumpTargetData>) {}

    fn collect_jump_blocks_dedup(&self, _: &Slab<JumpTargetData>) -> Vec<BlockRef> {
        Vec::new()
    }
    fn collect_jump_blocks_nodedup(&self, _: &Slab<JumpTargetData>) -> Vec<BlockRef> {
        Vec::new()
    }
}
impl TerminatorInst for Jump {
    fn get_jump_targets(&self) -> Option<&SlabRefList<JumpTargetRef>> {
        Some(&self.0._targets)
    }
    fn init_jump_targets(&mut self, alloc_jt: &mut Slab<JumpTargetData>) {
        let list = SlabRefList::from_slab(alloc_jt);
        list.push_back_value(
            alloc_jt,
            JumpTargetData::new_with_kind(JumpTargetKind::Jump),
        )
        .unwrap();
        self.0._targets = list;
    }
    fn collect_jump_blocks_dedup(&self, alloc_jt: &Slab<JumpTargetData>) -> Vec<BlockRef> {
        vec![self.get_block(alloc_jt)]
    }
    fn collect_jump_blocks_nodedup(&self, alloc_jt: &Slab<JumpTargetData>) -> Vec<BlockRef> {
        vec![self.get_block(alloc_jt)]
    }
}
impl TerminatorInst for Br {
    fn get_jump_targets(&self) -> Option<&SlabRefList<JumpTargetRef>> {
        Some(&self._common._targets)
    }
    fn init_jump_targets(&mut self, alloc_jt: &mut Slab<JumpTargetData>) {
        let list = SlabRefList::from_slab(alloc_jt);
        let if_true = list
            .push_back_value(
                alloc_jt,
                JumpTargetData::new_with_kind(JumpTargetKind::BrFalse),
            )
            .unwrap();
        let if_false = list
            .push_back_value(
                alloc_jt,
                JumpTargetData::new_with_kind(JumpTargetKind::BrTrue),
            )
            .unwrap();
        self._common._targets = list;
        self.if_true = if_true;
        self.if_false = if_false;
    }
    fn collect_jump_blocks_dedup(&self, alloc_jt: &Slab<JumpTargetData>) -> Vec<BlockRef> {
        let if_true = self.if_true.get_block(alloc_jt);
        let if_false = self.if_false.get_block(alloc_jt);
        if if_true == if_false { vec![if_true] } else { vec![if_true, if_false] }
    }
    fn collect_jump_blocks_nodedup(&self, alloc_jt: &Slab<JumpTargetData>) -> Vec<BlockRef> {
        vec![self.if_true.get_block(alloc_jt), self.if_false.get_block(alloc_jt)]
    }
}
impl TerminatorInst for Switch {
    fn get_jump_targets(&self) -> Option<&SlabRefList<JumpTargetRef>> {
        Some(&self._common._targets)
    }

    fn init_jump_targets(&mut self, alloc_jt: &mut Slab<JumpTargetData>) {
        let list = SlabRefList::from_slab(alloc_jt);
        self._default = list
            .push_back_value(
                alloc_jt,
                JumpTargetData::new_with_kind(JumpTargetKind::SwitchDefault),
            )
            .unwrap();
        self._common._targets = list;
    }

    fn collect_jump_blocks_dedup(&self, alloc_jt: &Slab<JumpTargetData>) -> Vec<BlockRef> {
        let cases = self._cases.borrow();
        let mut blocks = Vec::with_capacity(1 + cases.len());
        let mut block_set = BTreeSet::new();

        blocks.push(self.get_default(alloc_jt));
        block_set.insert(blocks[0]);

        for (_, j) in &*cases {
            let block = j.get_block(alloc_jt);
            if block_set.insert(block) {
                blocks.push(block);
            }
        }
        blocks
    }
    fn collect_jump_blocks_nodedup(&self, alloc_jt: &Slab<JumpTargetData>) -> Vec<BlockRef> {
        let mut blocks = Vec::with_capacity(1 + self._cases.borrow().len());
        blocks.push(self.get_default(alloc_jt));
        for (_, j) in self._cases.borrow().iter() {
            blocks.push(j.get_block(alloc_jt));
        }
        blocks
    }
}

impl InstDataUnique for Ret {
    fn build_operands(&mut self, common: &mut InstDataCommon, alloc_use: &mut Slab<UseData>) {
        self.retval = common.alloc_use(alloc_use, UseKind::RetValue);
    }

    fn check_operands(&self, common: &InstDataCommon, module: &Module) -> Result<(), InstError> {
        let retval = self.retval.get_operand(&module.borrow_use_alloc());
        check_operand_type_match(common.ret_type, retval, module)
    }
}
impl InstDataUnique for Jump {
    fn build_operands(&mut self, _: &mut InstDataCommon, _: &mut Slab<UseData>) {}

    fn check_operands(&self, _: &InstDataCommon, _: &Module) -> Result<(), InstError> {
        Ok(())
    }
}
impl InstDataUnique for Br {
    fn build_operands(&mut self, common: &mut InstDataCommon, alloc_use: &mut Slab<UseData>) {
        self._common._condition = common.alloc_use(alloc_use, UseKind::BranchCond)
    }

    fn check_operands(&self, _: &InstDataCommon, module: &Module) -> Result<(), InstError> {
        let cond = self
            ._common
            ._condition
            .get_operand(&module.borrow_use_alloc());
        check_operand_type_match(ValTypeID::new_boolean(), cond, module)
    }
}
impl InstDataUnique for Switch {
    fn build_operands(&mut self, common: &mut InstDataCommon, alloc_use: &mut Slab<UseData>) {
        self._common._condition = common.alloc_use(alloc_use, UseKind::SwitchCond);
    }

    fn check_operands(&self, _: &InstDataCommon, module: &Module) -> Result<(), InstError> {
        let cond = self
            ._common
            ._condition
            .get_operand(&module.borrow_use_alloc());
        check_operand_type_kind_match(ValTypeID::Int(0), cond, module)
    }
}

impl Ret {
    pub fn new_raw(module: &Module, ret_ty: ValTypeID) -> (InstDataCommon, Self) {
        let mut common =
            InstDataCommon::new(Opcode::Ret, ret_ty, &mut module.borrow_use_alloc_mut());
        let mut ret = Self { retval: UseRef::new_null() };
        ret.build_operands(&mut common, &mut module.borrow_use_alloc_mut());
        (common, ret)
    }

    pub fn new(module: &Module, retval: ValueSSA) -> (InstDataCommon, Self) {
        let (common, ret) = Self::new_raw(module, retval.get_value_type(module));
        ret.retval
            .set_operand_nordfg(&module.borrow_use_alloc(), retval);
        (common, ret)
    }
}

impl Jump {
    pub fn new_raw(module: &Module) -> (InstDataCommon, Self) {
        let common = InstDataCommon::new(
            Opcode::Jmp,
            ValTypeID::Void,
            &mut module.borrow_use_alloc_mut(),
        );
        let mut jump = Self(JumpCommon {
            _targets: SlabRefList::new_guide(),
            _condition: UseRef::new_null(),
        });
        jump.init_jump_targets(&mut module.borrow_jt_alloc_mut());
        (common, jump)
    }

    pub fn new(module: &Module, block: BlockRef) -> (InstDataCommon, Self) {
        let (common, jump) = Self::new_raw(module);
        jump.set_block_norcfg(&module.borrow_jt_alloc(), block);
        (common, jump)
    }

    pub fn get_jt(&self, alloc_jt: &Slab<JumpTargetData>) -> JumpTargetRef {
        self.0._targets.get_back_ref(alloc_jt).unwrap()
    }
    pub fn get_block(&self, alloc_jt: &Slab<JumpTargetData>) -> BlockRef {
        self.get_jt(alloc_jt).get_block(alloc_jt)
    }
    pub fn set_block_norcfg(&self, alloc_jt: &Slab<JumpTargetData>, block: BlockRef) {
        self.get_jt(alloc_jt).set_block_norcfg(alloc_jt, block);
    }
    pub fn set_block(&self, module: &Module, block: BlockRef) {
        let jt = {
            let alloc_jt = module.borrow_jt_alloc();
            self.get_jt(&alloc_jt)
        };
        jt.set_block(module, block);
    }
}

impl Br {
    pub fn new_raw(module: &Module) -> (InstDataCommon, Self) {
        let mut common = InstDataCommon::new(
            Opcode::Br,
            ValTypeID::Void,
            &mut module.borrow_use_alloc_mut(),
        );
        let mut br = Self {
            _common: JumpCommon {
                _targets: SlabRefList::new_guide(),
                _condition: UseRef::new_null(),
            },
            if_true: JumpTargetRef::new_null(),
            if_false: JumpTargetRef::new_null(),
        };
        br.init_jump_targets(&mut module.borrow_jt_alloc_mut());
        br.build_operands(&mut common, &mut module.borrow_use_alloc_mut());
        (common, br)
    }

    pub fn new(
        module: &Module,
        cond: ValueSSA,
        if_true: BlockRef,
        if_false: BlockRef,
    ) -> (InstDataCommon, Self) {
        let (common, br) = Self::new_raw(module);
        br._common
            ._condition
            .set_operand_nordfg(&module.borrow_use_alloc(), cond);
        br.if_true
            .set_block_norcfg(&module.borrow_jt_alloc(), if_true);
        br.if_false
            .set_block_norcfg(&module.borrow_jt_alloc(), if_false);
        (common, br)
    }

    pub fn get_cond_use(&self) -> UseRef {
        self._common._condition
    }
    pub fn get_cond(&self, alloc: &Slab<UseData>) -> ValueSSA {
        self._common._condition.get_operand(alloc)
    }
    pub fn set_cond_nordfg(&self, alloc: &Slab<UseData>, cond: ValueSSA) {
        self._common._condition.set_operand_nordfg(alloc, cond);
    }
    pub fn set_cond(&self, module: &Module, cond: ValueSSA) {
        self._common._condition.set_operand(module, cond)
    }
}

impl Switch {
    pub fn new_raw(module: &Module) -> (InstDataCommon, Self) {
        let mut common = InstDataCommon::new(
            Opcode::Switch,
            ValTypeID::Void,
            &mut module.borrow_use_alloc_mut(),
        );
        let mut switch = Self {
            _common: JumpCommon {
                _targets: SlabRefList::new_guide(),
                _condition: UseRef::new_null(),
            },
            _default: JumpTargetRef::new_null(),
            _cases: RefCell::new(Vec::new()),
        };
        switch.init_jump_targets(&mut module.borrow_jt_alloc_mut());
        switch.build_operands(&mut common, &mut module.borrow_use_alloc_mut());
        (common, switch)
    }

    pub fn new(module: &Module, cond: ValueSSA, default: BlockRef) -> (InstDataCommon, Self) {
        let (common, switch) = Self::new_raw(module);
        switch
            ._common
            ._condition
            .set_operand_nordfg(&module.borrow_use_alloc(), cond);
        switch
            ._default
            .set_block_norcfg(&module.borrow_jt_alloc(), default);
        (common, switch)
    }

    pub fn get_cond_use(&self) -> UseRef {
        self._common._condition
    }
    pub fn get_cond(&self, alloc: &Slab<UseData>) -> ValueSSA {
        self._common._condition.get_operand(alloc)
    }
    pub fn set_cond_nordfg(&self, alloc: &Slab<UseData>, cond: ValueSSA) {
        self._common._condition.set_operand_nordfg(alloc, cond);
    }
    pub fn set_cond(&self, module: &Module, cond: ValueSSA) {
        self._common._condition.set_operand(module, cond)
    }

    pub fn get_default(&self, alloc_jt: &Slab<JumpTargetData>) -> BlockRef {
        self._default.get_block(alloc_jt)
    }
    pub fn set_default_norcfg(&self, alloc_jt: &Slab<JumpTargetData>, block: BlockRef) {
        self._default.set_block_norcfg(alloc_jt, block);
    }
    pub fn set_default(&self, module: &Module, block: BlockRef) {
        self._default.set_block(module, block);
    }

    pub fn get_cast_target(&self, case: i128) -> Option<JumpTargetRef> {
        self._cases
            .borrow()
            .iter()
            .find(|(k, _)| *k == case)
            .map(|(_, v)| *v)
    }
    pub fn get_case(&self, alloc_jt: &Slab<JumpTargetData>, case: i128) -> Option<BlockRef> {
        self.get_cast_target(case).map(|jt| jt.get_block(alloc_jt))
    }
    pub fn set_existing_case_norcfg(
        &self,
        alloc_jt: &Slab<JumpTargetData>,
        case: i128,
        block: BlockRef,
    ) -> Option<BlockRef> {
        self.get_cast_target(case).map(|jt| {
            let ret = jt.get_block(alloc_jt);
            jt.set_block_norcfg(alloc_jt, block);
            ret
        })
    }
    pub fn set_case_norcfg(
        &self,
        alloc_jt: &mut Slab<JumpTargetData>,
        case: i128,
        block: BlockRef,
    ) -> (JumpTargetRef, BlockRef) {
        if let Some(bb) = self.set_existing_case_norcfg(alloc_jt, case, block) {
            return (self.get_cast_target(case).unwrap(), bb);
        }
        let new_jt = JumpTargetData::new_with_kind(JumpTargetKind::SwitchCase(case));
        new_jt.set_block_norcfg(block);

        let new_jt = self
            ._common
            ._targets
            .push_back_value(alloc_jt, new_jt)
            .unwrap();

        self._cases.borrow_mut().push((case, new_jt));
        self.sort_cases();
        (new_jt, BlockRef::new_null())
    }
    /// Set a case for the switch instruction.
    /// This may change the block of the case and CFG.
    pub fn set_case(&self, module: &Module, case: i128, block: BlockRef) {
        let mut alloc_jt = module.borrow_jt_alloc_mut();
        if let Some(mut rcfg) = module.borrow_rcfg_alloc_mut() {
            self.set_case_with_rcfg(&mut rcfg, &mut alloc_jt, case, block);
        } else {
            self.set_case_norcfg(&mut alloc_jt, case, block);
        }
    }
    pub fn set_case_with_rcfg(
        &self,
        rcfg: &mut RcfgAlloc,
        alloc_jt: &mut Slab<JumpTargetData>,
        case: i128,
        block: BlockRef,
    ) {
        let (jt, old_block) = self.set_case_norcfg(alloc_jt, case, block);

        if old_block == block {
            return;
        }
        if old_block.is_nonnull() {
            rcfg.get_node(old_block).remove_predecessor(jt);
        }
        if block.is_nonnull() {
            rcfg.get_node(block).add_predecessor(jt);
        }
    }

    pub fn sort_cases(&self) {
        let mut cases = self._cases.borrow_mut();
        cases.sort_by(|(a, _), (b, _)| a.cmp(b));
    }
    pub fn borrow_cases(&self) -> Ref<[(i128, JumpTargetRef)]> {
        Ref::map(self._cases.borrow(), Vec::as_slice)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
pub struct TerminatorInstRef(pub InstRef);

impl TerminatorInstRef {
    pub fn from_inst(inst: InstRef, alloc_inst: &Slab<InstData>) -> Self {
        let inst_data = inst.to_slabref_unwrap(alloc_inst);
        match inst_data.get_opcode() {
            Opcode::Ret | Opcode::Jmp | Opcode::Br | Opcode::Switch | Opcode::Unreachable => {
                Self(inst)
            }
            _ => panic!(
                "InstRef {:?} is not a terminator instruction",
                inst_data.get_opcode()
            ),
        }
    }

    pub fn get_inst(&self) -> InstRef {
        self.0
    }

    pub fn get_n_jump_targets(self, alloc_inst: &Slab<InstData>) -> usize {
        let inst_data = self.0.to_slabref_unwrap(alloc_inst);
        match inst_data {
            InstData::Jump(..) => 1,
            InstData::Br(..) => 2,
            InstData::Switch(_, sw) => sw.get_n_jump_targets(),
            InstData::Ret(..) | InstData::Unreachable(_) => 0,
            _ => panic!(
                "InstRef {:?} is not a terminator instruction",
                inst_data.get_opcode()
            ),
        }
    }

    pub fn get_jump_targets_from_alloc_inst<'a>(
        &self,
        alloc_inst: &'a Slab<InstData>,
    ) -> Option<&'a SlabRefList<JumpTargetRef>> {
        let inst_data = self.0.to_slabref_unwrap(alloc_inst);
        match inst_data {
            InstData::Ret(_, r) => r.get_jump_targets(),
            InstData::Jump(_, j) => j.get_jump_targets(),
            InstData::Br(_, br) => br.get_jump_targets(),
            InstData::Switch(_, sw) => sw.get_jump_targets(),
            InstData::Unreachable(_) => None,
            _ => panic!(
                "InstRef {:?} is not a terminator instruction",
                inst_data.get_opcode()
            ),
        }
    }
    pub fn get_jump_targets<'a>(
        &self,
        module: &'a Module,
    ) -> Option<Ref<'a, SlabRefList<JumpTargetRef>>> {
        let inst_data = module.get_inst(self.0);
        match &*inst_data {
            InstData::Ret(..) | InstData::Unreachable(_) => return None,
            InstData::Jump(..) | InstData::Br(..) | InstData::Switch(..) => {}
            _ => panic!(
                "InstRef {:?} is not a terminator instruction",
                inst_data.get_opcode()
            ),
        };

        Some(Ref::map(inst_data, |inst_data| {
            match inst_data {
                InstData::Ret(_, r) => r.get_jump_targets(),
                InstData::Jump(_, j) => j.get_jump_targets(),
                InstData::Br(_, br) => br.get_jump_targets(),
                InstData::Switch(_, sw) => sw.get_jump_targets(),
                _ => unreachable!(),
            }
            .unwrap()
        }))
    }
    pub fn collect_jump_blocks(
        &self,
        alloc_inst: &Slab<InstData>,
        alloc_jt: &Slab<JumpTargetData>,
    ) -> Vec<BlockRef> {
        let inst_data = self.0.to_slabref_unwrap(alloc_inst);
        match inst_data {
            InstData::Ret(_, r) => r.collect_jump_blocks_dedup(alloc_jt),
            InstData::Jump(_, j) => j.collect_jump_blocks_dedup(alloc_jt),
            InstData::Br(_, br) => br.collect_jump_blocks_dedup(alloc_jt),
            InstData::Switch(_, sw) => sw.collect_jump_blocks_dedup(alloc_jt),
            InstData::Unreachable(_) => Vec::new(),
            _ => panic!(
                "InstRef {:?} is not a terminator instruction",
                inst_data.get_opcode()
            ),
        }
    }
    pub fn collect_jump_blocks_from_module(&self, module: &Module) -> Vec<BlockRef> {
        let alloc_value = module.borrow_value_alloc();
        let alloc_jt = &module.borrow_jt_alloc();
        let alloc_inst = &alloc_value.alloc_inst;
        self.collect_jump_blocks(alloc_inst, alloc_jt)
    }
    pub fn collect_jump_blocks_nodedup(
        &self,
        alloc_inst: &Slab<InstData>,
        alloc_jt: &Slab<JumpTargetData>,
    ) -> Vec<BlockRef> {
        let inst_data = self.0.to_slabref_unwrap(alloc_inst);
        match inst_data {
            InstData::Ret(_, r) => r.collect_jump_blocks_nodedup(alloc_jt),
            InstData::Jump(_, j) => j.collect_jump_blocks_nodedup(alloc_jt),
            InstData::Br(_, br) => br.collect_jump_blocks_nodedup(alloc_jt),
            InstData::Switch(_, sw) => sw.collect_jump_blocks_nodedup(alloc_jt),
            InstData::Unreachable(_) => Vec::new(),
            _ => panic!(
                "InstRef {:?} is not a terminator instruction",
                inst_data.get_opcode()
            ),
        }
    }
    pub fn collect_jump_blocks_from_module_nodedup(&self, module: &Module) -> Vec<BlockRef> {
        let alloc_value = module.borrow_value_alloc();
        let alloc_jt = &module.borrow_jt_alloc();
        let alloc_inst = &alloc_value.alloc_inst;
        self.collect_jump_blocks_nodedup(alloc_inst, alloc_jt)
    }
    pub fn terminates_function(&self, alloc_inst: &Slab<InstData>) -> bool {
        let inst_data = self.0.to_slabref_unwrap(alloc_inst);
        match inst_data {
            InstData::Ret(_, r) => r.terminates_function(),
            InstData::Jump(_, j) => j.terminates_function(),
            InstData::Br(_, br) => br.terminates_function(),
            InstData::Switch(_, sw) => sw.terminates_function(),
            InstData::Unreachable(_) => true,
            _ => panic!(
                "InstRef {:?} is not a terminator instruction",
                inst_data.get_opcode()
            ),
        }
    }
}

impl Into<InstRef> for TerminatorInstRef {
    fn into(self) -> InstRef {
        self.0
    }
}

impl From<InstRef> for TerminatorInstRef {
    fn from(inst: InstRef) -> Self {
        Self(inst)
    }
}

impl SlabRef for TerminatorInstRef {
    type RefObject = InstData;
    fn from_handle(handle: usize) -> Self {
        Self(InstRef::from_handle(handle))
    }
    fn get_handle(&self) -> usize {
        self.0.get_handle()
    }
}

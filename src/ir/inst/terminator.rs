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
        module::Module,
        opcode::Opcode,
    },
    typing::id::ValTypeID,
};

use super::{
    InstDataCommon, InstDataUnique, InstError, InstRef,
    checking::{check_operand_type_kind_match, check_operand_type_match},
    usedef::{UseData, UseRef},
};

pub trait TerminatorInst {
    fn get_jump_targets(&self) -> Option<&SlabRefList<JumpTargetRef>>;

    fn init_jump_targets(&mut self, jt_alloc: &mut Slab<JumpTargetData>);

    fn get_n_jump_targets(&self) -> usize {
        self.get_jump_targets().map_or(0, |targets| targets.len())
    }

    /// Whether this terminator terminates the function control flow.
    /// True value means whether this instruction will return from the function
    /// or makes the control flow unreachable.
    fn terminates_function(&self) -> bool {
        self.get_jump_targets().is_none()
    }

    fn _jt_init_set_self_reference(&self, self_ref: InstRef, jt_alloc: &Slab<JumpTargetData>) {
        if let Some(targets) = self.get_jump_targets() {
            let mut curr_node = targets._head;
            while curr_node.is_nonnull() {
                curr_node.to_slabref_unwrap(jt_alloc)._terminator.set(self_ref);
                curr_node = match curr_node.get_next_ref(jt_alloc) {
                    Some(x) => x,
                    None => break,
                };
            }
        }
    }
}

pub struct Ret {
    _retval: UseRef,
}

pub struct JumpCommon {
    _targets: SlabRefList<JumpTargetRef>,
    _condition: UseRef,
}

pub struct Jump(JumpCommon);
pub struct Br {
    _common: JumpCommon,
    pub if_true: JumpTargetRef,
    pub if_false: JumpTargetRef,
}

pub struct Switch {
    _common: JumpCommon,
    _default: JumpTargetRef,
    _cases: Vec<(i128, JumpTargetRef)>,
}

impl TerminatorInst for Ret {
    fn get_jump_targets(&self) -> Option<&SlabRefList<JumpTargetRef>> {
        None
    }
    fn init_jump_targets(&mut self, _: &mut Slab<JumpTargetData>) {}
}
impl TerminatorInst for Jump {
    fn get_jump_targets(&self) -> Option<&SlabRefList<JumpTargetRef>> {
        Some(&self.0._targets)
    }
    fn init_jump_targets(&mut self, jt_alloc: &mut Slab<JumpTargetData>) {
        let list = SlabRefList::from_slab(jt_alloc);
        list.push_back_value(
            jt_alloc,
            JumpTargetData::new_with_kind(JumpTargetKind::Jump),
        )
        .unwrap();
        self.0._targets = list;
    }
}
impl TerminatorInst for Br {
    fn get_jump_targets(&self) -> Option<&SlabRefList<JumpTargetRef>> {
        Some(&self._common._targets)
    }
    fn init_jump_targets(&mut self, jt_alloc: &mut Slab<JumpTargetData>) {
        let list = SlabRefList::from_slab(jt_alloc);
        let if_true = list
            .push_back_value(
                jt_alloc,
                JumpTargetData::new_with_kind(JumpTargetKind::BrFalse),
            )
            .unwrap();
        let if_false = list
            .push_back_value(
                jt_alloc,
                JumpTargetData::new_with_kind(JumpTargetKind::BrTrue),
            )
            .unwrap();
        self._common._targets = list;
        self.if_true = if_true;
        self.if_false = if_false;
    }
}
impl TerminatorInst for Switch {
    fn get_jump_targets(&self) -> Option<&SlabRefList<JumpTargetRef>> {
        Some(&self._common._targets)
    }

    fn init_jump_targets(&mut self, jt_alloc: &mut Slab<JumpTargetData>) {
        let list = SlabRefList::from_slab(jt_alloc);
        self._default = list
            .push_back_value(
                jt_alloc,
                JumpTargetData::new_with_kind(JumpTargetKind::SwitchDefault),
            )
            .unwrap();
        self._common._targets = list;
    }
}

impl InstDataUnique for Ret {
    fn build_operands(&mut self, common: &mut InstDataCommon, alloc_use: &mut Slab<UseData>) {
        self._retval = common.alloc_use(alloc_use);
    }

    fn check_operands(&self, common: &InstDataCommon, module: &Module) -> Result<(), InstError> {
        let retval = self._retval.get_operand(&module.borrow_use_alloc());
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
        self._common._condition = common.alloc_use(alloc_use)
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
        self._common._condition = common.alloc_use(alloc_use)
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
        let mut commmon =
            InstDataCommon::new(Opcode::Ret, ret_ty, &mut module.borrow_use_alloc_mut());
        let mut ret = Self {
            _retval: UseRef::new_null(),
        };
        ret.build_operands(&mut commmon, &mut module.borrow_use_alloc_mut());
        (commmon, ret)
    }

    pub fn new(module: &Module, retval: ValueSSA) -> (InstDataCommon, Self) {
        let (common, ret) = Self::new_raw(module, retval.get_value_type(module));
        ret._retval
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
        jump.set_block(&module.borrow_jt_alloc(), block);
        (common, jump)
    }

    pub fn get_jt(&self, jt_alloc: &Slab<JumpTargetData>) -> JumpTargetRef {
        self.0._targets.get_back_ref(jt_alloc).unwrap()
    }
    pub fn get_block(&self, jt_alloc: &Slab<JumpTargetData>) -> BlockRef {
        self.get_jt(jt_alloc).get_block(jt_alloc)
    }
    pub fn set_block(&self, jt_alloc: &Slab<JumpTargetData>, block: BlockRef) {
        self.get_jt(jt_alloc).set_block(jt_alloc, block);
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
        br.if_true.set_block(&module.borrow_jt_alloc(), if_true);
        br.if_false.set_block(&module.borrow_jt_alloc(), if_false);
        (common, br)
    }

    pub fn get_cond_use(&self) -> UseRef {
        self._common._condition
    }
    pub fn get_cond(&self, alloc: &Slab<UseData>) -> ValueSSA {
        self._common._condition.get_operand(alloc)
    }
    pub fn set_cond(&self, alloc: &Slab<UseData>, cond: ValueSSA) {
        self._common._condition.set_operand_nordfg(alloc, cond);
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
            _cases: Vec::new(),
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
            .set_block(&module.borrow_jt_alloc(), default);
        (common, switch)
    }

    pub fn get_cond_use(&self) -> UseRef {
        self._common._condition
    }
    pub fn get_cond(&self, alloc: &Slab<UseData>) -> ValueSSA {
        self._common._condition.get_operand(alloc)
    }
    pub fn set_cond(&self, alloc: &Slab<UseData>, cond: ValueSSA) {
        self._common._condition.set_operand_nordfg(alloc, cond);
    }

    pub fn get_default(&self, jt_alloc: &Slab<JumpTargetData>) -> BlockRef {
        self._default.get_block(jt_alloc)
    }
    pub fn set_default(&self, jt_alloc: &Slab<JumpTargetData>, block: BlockRef) {
        self._default.set_block(jt_alloc, block);
    }

    pub fn get_case(&self, jt_alloc: &Slab<JumpTargetData>, case: i128) -> Option<BlockRef> {
        self._cases
            .iter()
            .find(|(k, _)| *k == case)
            .map(|(_, v)| v.get_block(jt_alloc))
    }
    pub fn set_existing_case(&self, jt_alloc: &Slab<JumpTargetData>, case: i128, block: BlockRef) {
        self._cases
            .iter()
            .find(|(k, _)| *k == case)
            .map(|(_, v)| v.set_block(jt_alloc, block));
    }
}

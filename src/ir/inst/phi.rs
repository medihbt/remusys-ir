use std::cell::{Ref, RefCell, RefMut};

use slab::Slab;

use crate::{
    base::{NullableValue, slabref::SlabRef},
    ir::{ValueSSA, block::BlockRef, module::Module, opcode::Opcode},
    typing::id::ValTypeID,
};

use super::{
    InstData, InstDataCommon, InstDataUnique, InstError, InstRef,
    checking::check_operand_type_match,
    usedef::{UseData, UseKind, UseRef},
};

pub struct PhiOperand {
    pub from_bb: BlockRef,
    pub from_bb_use: UseRef,
    pub from_value_use: UseRef,
}

pub struct PhiOp {
    from: RefCell<Vec<PhiOperand>>,
}

#[derive(Debug, Clone, Copy)]
pub enum PhiErr {
    FromBBShouldInsert(BlockRef),
}

impl PhiOp {
    pub fn get_from_use(&self, from_bb: BlockRef) -> Option<UseRef> {
        self.from
            .borrow()
            .iter()
            .find(|op| op.from_bb == from_bb)
            .map(|op| op.from_bb_use)
    }

    pub fn get_from_value(&self, from_bb: BlockRef, alloc_use: &Slab<UseData>) -> Option<ValueSSA> {
        self.get_from_use(from_bb)
            .map(|u| u.to_slabref_unwrap(alloc_use).get_operand())
    }

    pub fn get_from_all(&self) -> Ref<[PhiOperand]> {
        Ref::map(self.from.borrow(), Vec::as_slice)
    }
    pub fn get_from_all_mut(&self) -> RefMut<Vec<PhiOperand>> {
        self.from.borrow_mut()
    }

    pub fn set_from_value_noinsert_nordfg(
        &self,
        from_bb: BlockRef,
        alloc_use: &Slab<UseData>,
        value: ValueSSA,
    ) -> Result<(), PhiErr> {
        let x = self
            .get_from_use(from_bb)
            .map(|u| u.set_operand_nordfg(alloc_use, value));
        match x {
            Some(_) => Ok(()),
            None => Err(PhiErr::FromBBShouldInsert(from_bb)),
        }
    }
    pub fn set_from_value_noinsert(
        &self,
        from_bb: BlockRef,
        module: &Module,
        value: ValueSSA,
    ) -> Result<(), PhiErr> {
        let x = self
            .get_from_use(from_bb)
            .map(|u| u.set_operand(module, value));
        match x {
            Some(_) => Ok(()),
            None => Err(PhiErr::FromBBShouldInsert(from_bb)),
        }
    }

    pub fn insert_from_value_nordfg(
        instref: InstRef,
        module: &Module,
        from_bb: BlockRef,
        value: ValueSSA,
    ) -> Result<UseRef, PhiErr> {
        let new_useref = if let InstData::Phi(_, phi) = &*module.get_inst(instref) {
            phi.get_from_use(from_bb)
        } else {
            panic!("Requries PHI but got {:?}", instref);
        };

        match new_useref {
            Some(u) => {
                u.set_operand_nordfg(&module.borrow_use_alloc(), value);
                Ok(u)
            }
            None => {
                let (block_useref, value_useref) = Self::insert_value_build_uses(module, from_bb);
                if let InstData::Phi(common, phi) = &*module.get_inst(instref) {
                    common
                        .operands
                        .push_back_ref(&*module.borrow_use_alloc(), value_useref)
                        .unwrap();
                    phi.from.borrow_mut().push(PhiOperand {
                        from_bb,
                        from_bb_use: block_useref,
                        from_value_use: value_useref,
                    });
                } else {
                    panic!();
                }
                Ok(value_useref)
            }
        }
    }
    pub fn insert_from_value(
        instref: InstRef,
        module: &Module,
        from_bb: BlockRef,
        value: ValueSSA,
    ) -> Result<UseRef, PhiErr> {
        let new_useref = if let InstData::Phi(_, phi) = &*module.get_inst(instref) {
            phi.get_from_use(from_bb)
        } else {
            panic!("Requries PHI but got {:?}", instref);
        };

        match new_useref {
            Some(u) => {
                u.set_operand(&module, value);
                Ok(u)
            }
            None => {
                let (block_useref, value_useref) = Self::insert_value_build_uses(module, from_bb);
                if let InstData::Phi(common, phi) = &*module.get_inst(instref) {
                    common
                        .operands
                        .push_back_ref(&*module.borrow_use_alloc(), value_useref)
                        .unwrap();
                    phi.from.borrow_mut().push(PhiOperand {
                        from_bb,
                        from_bb_use: block_useref,
                        from_value_use: value_useref,
                    });
                } else {
                    unreachable!()
                }
                value_useref.set_operand(&module, value);
                Ok(value_useref)
            }
        }
    }

    fn insert_value_build_uses(module: &Module, from_bb: BlockRef) -> (UseRef, UseRef) {
        let block_useref = module.insert_use(UseData::new(
            UseKind::PhiIncomingBlock(UseRef::new_null()),
            InstRef::new_null(),
            ValueSSA::None,
        ));
        let value_useref = module.insert_use(UseData::new(
            UseKind::PhiIncomingValue {
                from_bb,
                from_bb_use: block_useref,
            },
            InstRef::new_null(),
            ValueSSA::None,
        ));
        match module.mut_use(block_useref).kind.get_mut() {
            UseKind::PhiIncomingBlock(related_value_use) => *related_value_use = value_useref,
            _ => unreachable!("Expected PhiIncomingBlock use kind"),
        };
        (block_useref, value_useref)
    }

    pub fn unset_from(
        &self,
        common: &InstDataCommon,
        from_bb: BlockRef,
        module: &Module,
    ) -> Option<(UseRef, UseRef)> {
        let mut from = self.from.borrow_mut();
        let index = from.iter().position(|op| op.from_bb == from_bb);
        let u = match index {
            Some(i) => {
                let op = from.swap_remove(i);
                op.from_value_use.set_operand(module, ValueSSA::None);
                common.remove_use(&module.borrow_use_alloc(), op.from_bb_use);
                Some((op.from_bb_use, op.from_value_use))
            }
            None => None,
        };
        u
    }

    pub fn replace_from_bb_with_new(
        &self,
        from_bb: BlockRef,
        new_from_bb: BlockRef,
        module: &Module,
    ) -> bool {
        if from_bb == new_from_bb {
            return true; // No change
        }
        let mut from = self.from.borrow_mut();
        let from = &mut *from;
        let from_ref = if let Some(i) = from.iter().position(|op| op.from_bb == from_bb) {
            &mut from[i]
        } else {
            return false; // No such from_bb
        };
        from_ref.from_bb = new_from_bb;
        from_ref
            .from_bb_use
            .set_operand(module, ValueSSA::Block(new_from_bb));
        from_ref
            .from_value_use
            .kind_by_module(module)
            .set(UseKind::PhiIncomingValue {
                from_bb: new_from_bb,
                from_bb_use: from_ref.from_bb_use,
            });
        true
    }

    pub fn new(ret_type: ValTypeID, module: &Module) -> (InstDataCommon, Self) {
        (
            InstDataCommon::new(Opcode::Phi, ret_type, &mut module.borrow_use_alloc_mut()),
            Self {
                from: RefCell::new(Vec::new()),
            },
        )
    }
}

impl InstDataUnique for PhiOp {
    fn build_operands(&mut self, _: &mut InstDataCommon, _: &mut Slab<UseData>) {}

    fn check_operands(&self, common: &InstDataCommon, module: &Module) -> Result<(), InstError> {
        let self_type = common.ret_type;
        for op in self.from.borrow().iter() {
            let from = op.from_value_use.get_operand(&module.borrow_use_alloc());
            check_operand_type_match(self_type, from, module)?;
        }
        Ok(())
    }
}

pub struct PhiOpRef(InstRef);

impl PhiOpRef {
    pub fn new(module: &Module, ret_type: ValTypeID) -> Self {
        let (common, phi) = PhiOp::new(ret_type, module);
        let instref = module.insert_inst(InstData::Phi(common, phi));
        Self(instref)
    }

    pub fn from_inst_raw(instref: InstRef) -> Self {
        Self(instref)
    }
    pub fn from_inst_checked(instref: InstRef, module: &Module) -> Option<Self> {
        if let InstData::Phi(_, _) = &*module.get_inst(instref) {
            Some(Self(instref))
        } else {
            None
        }
    }

    pub fn get_data<'a>(&self, module: &'a Module) -> Ref<'a, PhiOp> {
        Ref::map(module.get_inst(self.0), |data| {
            if let InstData::Phi(_, phi) = data {
                phi
            } else {
                panic!("Requries PHI but got {:?}", self.0);
            }
        })
    }
}

use std::cell::RefCell;

use slab::Slab;

use crate::{
    base::slabref::SlabRef,
    ir::{ValueSSA, block::BlockRef, module::Module},
};

use super::{
    InstData, InstRef,
    usedef::{UseData, UseRef},
};

pub struct PhiOp {
    from: RefCell<Vec<(BlockRef, UseRef)>>,
}

pub enum PhiErr {
    FromBBShouldInsert(BlockRef),
}

impl PhiOp {
    pub fn get_from_use(&self, from_bb: BlockRef) -> Option<UseRef> {
        self.from
            .borrow()
            .iter()
            .find(|(b, _)| *b == from_bb)
            .map(|(_, u)| u.clone())
    }

    pub fn get_from_value(&self, from_bb: BlockRef, alloc_use: &Slab<UseData>) -> Option<ValueSSA> {
        self.get_from_use(from_bb)
            .map(|u| u.to_slabref_unwrap(alloc_use).get_operand())
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
                let useref = module.insert_use(UseData::new(instref, value));
                if let InstData::Phi(common, phi) = &*module.get_inst(instref) {
                    common
                        .operands
                        .push_back_ref(&*module.borrow_use_alloc(), useref)
                        .unwrap();
                    phi.from.borrow_mut().push((from_bb, useref));
                } else {
                    panic!();
                }
                Ok(useref)
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
                let useref = module.insert_use(UseData::new(instref, value));
                if let InstData::Phi(common, phi) = &*module.get_inst(instref) {
                    common
                        .operands
                        .push_back_ref(&*module.borrow_use_alloc(), useref)
                        .unwrap();
                    phi.from.borrow_mut().push((from_bb, useref));
                } else {
                    panic!();
                }
                Ok(useref)
            }
        }
    }
}

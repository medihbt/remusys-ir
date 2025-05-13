use std::cell::{Ref, RefCell};

use slab::Slab;

use crate::{
    base::slabref::SlabRef,
    ir::{ValueSSA, block::BlockRef, module::Module, opcode::Opcode},
    typing::id::ValTypeID,
};

use super::{
    InstData, InstDataCommon, InstDataUnique, InstError, InstRef,
    checking::check_operand_type_match,
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

    pub fn get_from_all(&self) -> Ref<[(BlockRef, UseRef)]> {
        Ref::map(self.from.borrow(), Vec::as_slice)
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

    pub fn unset_from(
        &self,
        common: &InstDataCommon,
        from_bb: BlockRef,
        module: &Module,
    ) -> Option<UseRef> {
        let mut from = self.from.borrow_mut();
        let index = from.iter().position(|(b, _)| *b == from_bb);
        let u = match index {
            Some(i) => {
                let (_, u) = from.swap_remove(i);
                u.set_operand(module, ValueSSA::None);
                common.remove_use(&module.borrow_use_alloc(), u);
                Some(u)
            }
            None => None,
        };
        u
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
        for (_, from) in self.from.borrow().iter() {
            let from = from.get_operand(&module.borrow_use_alloc());
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

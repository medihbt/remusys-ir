use crate::{
    base::NullableValue,
    ir::{block::BlockRef, opcode::Opcode, Module},
    typing::id::ValTypeID,
};

use super::{
    InstCommon, InstDataTrait, InstRef,
    usedef::{UseData, UseRef},
};

pub struct BinSelect {
    pub cond: UseRef,
    pub true_value:  UseRef,
    pub false_value: UseRef,
}

pub struct BinOp {
    pub lhs: UseRef,
    pub rhs: UseRef,
}

pub struct CastOp {
    pub src: UseRef,
}

pub struct IndexPtrOp {
    pub aggr_ty:    ValTypeID,
    pub index_ty:   ValTypeID,
    pub indexed:    UseRef,
    pub indices:    Vec<UseRef>,
}

pub struct LoadOp {
    pub source_ty: ValTypeID,
    pub source:    UseRef,
}

pub struct StoreOp {
    pub source_ty: ValTypeID,
    pub source:    UseRef,
    pub target:    UseRef,
}


impl InstDataTrait for BinSelect {
    fn init_common(
        &mut self,
        opcode: Opcode,
        ty: ValTypeID,
        parent: BlockRef,
        module: &mut Module,
    ) -> InstCommon {
        let ret = InstCommon::new(opcode, ty, parent, module);
        self.cond = ret.add_use(UseData::new(InstRef::new_null()), &mut module._alloc_use);
        self.true_value = ret.add_use(UseData::new(InstRef::new_null()), &mut module._alloc_use);
        self.false_value = ret.add_use(UseData::new(InstRef::new_null()), &mut module._alloc_use);
        ret
    }
}

impl InstDataTrait for BinOp {
    fn init_common(
        &mut self,
        opcode: Opcode,
        ty: ValTypeID,
        parent: BlockRef,
        module: &mut Module,
    ) -> InstCommon {
        let ret = InstCommon::new(opcode, ty, parent, module);
        self.lhs = ret.add_use(UseData::new(InstRef::new_null()), &mut module._alloc_use);
        self.rhs = ret.add_use(UseData::new(InstRef::new_null()), &mut module._alloc_use);
        ret
    }
}

impl InstDataTrait for CastOp {
    fn init_common(
        &mut self,
        opcode: Opcode,
        ty: ValTypeID,
        parent: BlockRef,
        module: &mut Module,
    ) -> InstCommon {
        let ret = InstCommon::new(opcode, ty, parent, module);
        self.src = ret.add_use(UseData::new(InstRef::new_null()), &mut module._alloc_use);
        ret
    }
}

impl InstDataTrait for IndexPtrOp {
    fn init_common(
        &mut self,
        opcode: Opcode,
        ty: ValTypeID,
        parent: BlockRef,
        module: &mut Module,
    ) -> InstCommon {
        let ret = InstCommon::new(opcode, ty, parent, module);
        self.indexed = ret.add_use(UseData::new(InstRef::new_null()), &mut module._alloc_use);
        // `self.indices` cannot be initialized because lacking of actual container type.
        ret
    }
}

use crate::{base::slabref::SlabRef, ir::{block::BlockRef, Module}, typing::id::ValTypeID};

use super::{Global, GlobalDataCommon, GlobalRef};

pub struct Func {
    pub global: GlobalDataCommon,
    pub body:   Option<FuncBody>,
}

pub struct FuncBody {
    pub parent: GlobalRef,
    pub blocks: slab::Slab<BlockRef>,
    pub entry:  BlockRef,
}

pub struct FuncBodyIter<'a> {
    pub func_body: &'a FuncBody,
    pub iter:      slab::Iter<'a, BlockRef>,
    pub at_entry:  bool,
}

impl FuncBody {
    pub fn get_parent<'a>(&'a self, module: &'a Module) -> &'a Func {
        match self.parent
                  .to_slabref(&module._alloc_global)
                  .expect("FuncBody::get_parent: parent not found") {
            Global::Func(func) => func,
            _ => panic!("FuncBody::get_parent: not a Func"),
        }
    }
    pub fn get_parent_mut<'a>(&'a mut self, module: &'a mut Module) -> &'a mut Func {
        match self.parent
                  .to_slabref_mut(&mut module._alloc_global)
                  .expect("FuncBody::get_parent: parent not found") {
            Global::Func(func) => func,
            _ => panic!("FuncBody::get_parent: not a Func"),
        }
    }

    pub fn iter(&self) -> FuncBodyIter {
        FuncBodyIter {
            func_body: self,
            iter:      self.blocks.iter(),
            at_entry:  true,
        }
    }
}

impl Iterator for FuncBodyIter<'_> {
    type Item = BlockRef;

    fn next(&mut self) -> Option<Self::Item> {
        if self.at_entry {
            self.at_entry = false;
            Some(self.func_body.entry)
        } else {
            self.iter.next().map(|(_,b)| b.clone())
        }
    }
}

pub struct FuncArg {
    pub arg_ty:      ValTypeID,
    pub parent_func: GlobalRef,
    pub arg_idx:     usize,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct FuncArgRef(pub(crate) usize);

impl SlabRef for FuncArgRef {
    type Item = FuncArg;
    fn from_handle(handle: usize) -> Self { Self(handle) }
    fn get_handle (&self) -> usize { self.0 }
}
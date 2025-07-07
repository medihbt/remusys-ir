use std::{
    cell::{Cell, Ref, RefCell},
    num::NonZero,
};

use crate::{
    base::{
        NullableValue,
        slablist::{SlabRefList, SlabRefListError},
    },
    ir::{
        PtrStorage, PtrUser,
        block::{BlockData, BlockRef},
        module::Module,
    },
    typing::{context::TypeContext, id::ValTypeID, types::FuncTypeRef},
};

use super::{GlobalDataCommon, GlobalRef};

pub trait FuncStorage: PtrStorage {
    fn get_stored_func_type(&self) -> FuncTypeRef {
        match self.get_stored_pointee_type() {
            ValTypeID::Func(func_type) => func_type,
            _ => panic!("Expected a function type"),
        }
    }

    fn get_return_type(&self, type_ctx: &TypeContext) -> ValTypeID {
        self.get_stored_func_type().get_return_type(type_ctx)
    }
    fn get_nargs(&self, type_ctx: &TypeContext) -> usize {
        self.get_stored_func_type().get_nargs(type_ctx)
    }
    fn get_arg_type(&self, type_ctx: &TypeContext, index: usize) -> Option<ValTypeID> {
        self.get_stored_func_type().get_arg(type_ctx, index)
    }
    fn is_vararg(&self, type_ctx: &TypeContext) -> bool {
        self.get_stored_func_type().is_vararg(type_ctx)
    }
}
pub trait FuncUser: PtrUser {
    fn get_operand_func_type(&self) -> FuncTypeRef {
        match self.get_operand_pointee_type() {
            ValTypeID::Func(func_type) => func_type,
            _ => panic!("Expected a function type"),
        }
    }

    fn get_return_type(&self, type_ctx: &TypeContext) -> ValTypeID {
        self.get_operand_func_type().get_return_type(type_ctx)
    }
    fn get_nargs(&self, type_ctx: &TypeContext) -> usize {
        self.get_operand_func_type().get_nargs(type_ctx)
    }
    fn get_arg_type(&self, type_ctx: &TypeContext, index: usize) -> Option<ValTypeID> {
        self.get_operand_func_type().get_arg(type_ctx, index)
    }
}

#[derive(Debug)]
pub struct FuncData {
    pub(crate) _common: GlobalDataCommon,
    pub(crate) _body: RefCell<Option<FuncBody>>,
}

#[derive(Debug)]
pub struct FuncBody {
    pub func: GlobalRef,
    pub body: SlabRefList<BlockRef>,
    pub entry: BlockRef,
}

impl PtrStorage for FuncData {
    fn get_stored_pointee_type(&self) -> ValTypeID {
        self._common.content_ty.clone()
    }

    fn get_stored_pointee_align(&self) -> Option<NonZero<usize>> {
        None
    }
}
impl FuncStorage for FuncData {}

impl FuncData {
    pub fn new_extern(functy: FuncTypeRef, name: String) -> Self {
        Self {
            _common: GlobalDataCommon {
                name,
                content_ty: ValTypeID::Func(functy),
                self_ref: Cell::new(GlobalRef::new_null()),
            },
            _body: RefCell::new(None),
        }
    }
    pub fn new_with_unreachable(
        module: &Module,
        functy: FuncTypeRef,
        name: String,
    ) -> Result<Self, SlabRefListError> {
        let unreachable_bb = BlockData::new_unreachable(module)?;
        let unreachable_bb_ref = module.insert_block(unreachable_bb);

        let blocks = {
            let mut alloc_value = module.borrow_value_alloc_mut();
            let alloc_block = &mut alloc_value.alloc_block;
            let blocks = SlabRefList::from_slab(alloc_block);
            blocks.push_back_ref(alloc_block, unreachable_bb_ref)?;
            blocks
        };

        Ok(Self {
            _common: GlobalDataCommon {
                name,
                content_ty: ValTypeID::Func(functy),
                self_ref: Cell::new(GlobalRef::new_null()),
            },
            _body: RefCell::new(Some(FuncBody {
                func: GlobalRef::new_null(),
                body: blocks,
                entry: unreachable_bb_ref,
            })),
        })
    }

    pub fn is_extern(&self) -> bool {
        self._body.borrow().is_none()
    }

    pub fn add_block_data(
        &self,
        mut_module: &Module,
        block_data: BlockData,
    ) -> Result<BlockRef, SlabRefListError> {
        let block_ref = mut_module.insert_block(block_data);
        self.add_block_ref(mut_module, block_ref)?;
        Ok(block_ref)
    }
    pub fn add_block_ref(
        &self,
        module: &Module,
        block_ref: BlockRef,
    ) -> Result<(), SlabRefListError> {
        self._body
            .borrow_mut()
            .as_mut()
            .unwrap()
            .body
            .push_back_ref(&module.borrow_value_alloc().alloc_block, block_ref)?;
        Ok(())
    }

    pub fn get_blocks(&self) -> Option<Ref<SlabRefList<BlockRef>>> {
        let body = self._body.borrow();
        if body.is_none() {
            None
        } else {
            Some(Ref::map(body, |body| &body.as_ref().unwrap().body))
        }
    }
    pub fn get_entry(&self) -> BlockRef {
        self._body.borrow().as_ref().unwrap().entry
    }
    pub fn get_name(&self) -> &str {
        &self._common.name
    }
}

#[cfg(test)]
mod testing {
    use crate::{
        ir::{
            ValueSSA,
            constant::data::ConstData,
            global::GlobalData,
            inst::{InstData, terminator::Ret},
            module::Module,
        },
        typing::{
            context::{PlatformPolicy, TypeContext},
            id::ValTypeID,
        },
    };

    use super::FuncData;

    #[test]
    fn test_new_func_data() {
        let platform = PlatformPolicy::new_host();
        let type_ctx = TypeContext::new_rc(platform);
        let module = Module::new("io.medihbt.RemusysIRTes".into(), type_ctx.clone());

        let main_functy = type_ctx.make_func_type(
            &[ValTypeID::Int(32), ValTypeID::Ptr],
            ValTypeID::Int(32),
            false,
        );
        let main_func_data =
            FuncData::new_with_unreachable(&module, main_functy, "main".into()).unwrap();
        assert_eq!(main_func_data.is_extern(), false);

        let main_func_ref = module.insert_global(GlobalData::Func(main_func_data));

        // Add `return 0` to the function body.
        let (c, r) = Ret::new(
            &module,
            ValueSSA::ConstData(ConstData::Zero(ValTypeID::Int(32))),
        );
        let ret_inst = module.insert_inst(InstData::Ret(c, r));

        match &*module.get_global(main_func_ref) {
            GlobalData::Func(func_data) => {
                let entry = func_data._body.borrow().as_ref().unwrap().entry;
                module
                    .get_block(entry)
                    .set_terminator(&module, ret_inst)
                    .unwrap();
            }
            _ => panic!("Expected a function data"),
        }

        module.perform_basic_check();
    }
}

use crate::{
    base::{NullableValue, slabref::SlabRef},
    ir::{
        Module,
        block::BlockRef,
        global::{
            Global, GlobalDataCommon, GlobalRef,
            func::{Func, FuncBody},
        },
        inst::InstRef,
    },
    typing::id::ValTypeID,
};

pub struct Builder {
    pub module: Module,

    pub current_func: GlobalRef,
    pub current_block: BlockRef,
    pub current_inst: InstRef,
}

impl Builder {
    pub fn new(module: Module) -> Self {
        Self {
            module,
            current_func: GlobalRef::new_null(),
            current_block: BlockRef::new_null(),
            current_inst: InstRef::new_null(),
        }
    }

    pub fn add_func(&mut self, name: String, func_ty: ValTypeID, is_extern: bool) -> GlobalRef {
        let func_data = Global::Func(Func {
            global: GlobalDataCommon {
                name: name.clone(),
                pointee_ty: func_ty,
            },
            body: if is_extern {
                    None
                } else {
                    let void_ty = self.module._type_ctx.get_void_type();
                    Some(FuncBody::new(
                        GlobalRef::new_null(),
                        BlockRef::new_unreachable(&mut self.module, void_ty),
                    ))
                },
        });

        let func_ref = GlobalRef::from_handle(self.module._alloc_global.insert(func_data));
        self.module._global_map.insert(name, func_ref.clone());

        func_ref.modify_slabref(&mut self.module._alloc_global, |func| match func {
            Global::Func(func) => match &mut func.body {
                Some(body) => body.parent = func_ref.clone(),
                None => {}
            },
            _ => panic!("Invalid global reference"),
        });

        func_ref
    }
}

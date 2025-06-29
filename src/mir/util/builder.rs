use std::rc::Rc;

use crate::{
    base::NullableValue,
    mir::{
        inst::{MirInst, MirInstRef},
        module::{
            MirModule, ModuleItem, ModuleItemRef,
            block::MirBlockRef,
            func::MirFunc,
            global::{MirGlobalData, MirGlobalVariable, Section},
        },
    },
    typing::{context::TypeContext, id::ValTypeID, types::FuncTypeRef},
};

#[derive(Debug, Clone)]
pub struct MirFocusData {
    func: Option<Rc<MirFunc>>,
    block: MirBlockRef,
}

#[derive(Debug, Clone)]
pub enum MirFocus {
    Global,
    Func(Rc<MirFunc>),
    Block(Rc<MirFunc>, MirBlockRef),
}

impl MirFocus {
    pub fn from_data(data: &MirFocusData) -> Self {
        let func = if let Some(func) = &data.func {
            Rc::clone(func)
        } else {
            return MirFocus::Global;
        };
        if let Some(bb) = data.block.to_option() {
            MirFocus::Block(func, bb)
        } else {
            MirFocus::Func(func)
        }
    }
    pub fn to_data(self) -> MirFocusData {
        match self {
            MirFocus::Global => MirFocusData {
                func: None,
                block: MirBlockRef::new_null(),
            },
            MirFocus::Func(func) => MirFocusData {
                func: Some(func),
                block: MirBlockRef::new_null(),
            },
            MirFocus::Block(func, block) => MirFocusData {
                func: Some(func),
                block: block,
            },
        }
    }
}

#[derive(Debug)]
pub struct MirBuilder<'a> {
    pub mir_module: &'a mut MirModule,
    focus: MirFocusData,
}

impl<'a> MirBuilder<'a> {
    pub fn new(mir_module: &'a mut MirModule) -> Self {
        MirBuilder {
            mir_module,
            focus: MirFocus::Global.to_data(),
        }
    }

    pub fn get_focus(&self) -> MirFocus {
        MirFocus::from_data(&self.focus)
    }
    pub fn set_focus(&mut self, focus: MirFocus) {
        self.focus = focus.to_data();
    }

    pub fn push_variable(
        &mut self,
        var: MirGlobalVariable,
    ) -> (ModuleItemRef, Rc<MirGlobalVariable>) {
        let rc_var = Rc::new(var);
        let item_ref = self
            .mir_module
            .add_item(ModuleItem::Variable(Rc::clone(&rc_var)));
        (item_ref, rc_var)
    }
    pub fn extern_variable(
        &mut self,
        name: String,
        section: Section,
        ty: ValTypeID,
        type_ctx: &TypeContext,
    ) -> (ModuleItemRef, Rc<MirGlobalVariable>) {
        self.push_variable(MirGlobalVariable::new_extern(name, section, ty, type_ctx))
    }
    pub fn push_unnamed_data(&mut self, data: MirGlobalData) -> ModuleItemRef {
        self.mir_module.add_item(ModuleItem::UnnamedData(data))
    }
    pub fn push_func(&mut self, func: MirFunc, switch_focus: bool) -> (ModuleItemRef, Rc<MirFunc>) {
        let rc_func = Rc::new(func);
        let item_ref = self
            .mir_module
            .add_item(ModuleItem::Function(Rc::clone(&rc_func)));
        if switch_focus && rc_func.is_define() {
            self.set_focus(MirFocus::Func(Rc::clone(&rc_func)));
        }
        (item_ref, rc_func)
    }
    pub fn extern_func(
        &mut self,
        name: String,
        func_ty: FuncTypeRef,
        type_ctx: &TypeContext,
    ) -> (ModuleItemRef, Rc<MirFunc>) {
        let func = MirFunc::new_extern(name, func_ty, type_ctx);
        self.push_func(func, true)
    }

    pub fn add_block(&mut self, mir_block: MirBlockRef, switch_focus: bool) {
        let curr_func = match self.get_focus() {
            MirFocus::Func(mir_func) => mir_func,
            MirFocus::Block(mir_func, _) => mir_func,
            _ => panic!("Cannot add block when focus is not on a function or block"),
        };
        let mut alloc_block = self.mir_module.borrow_alloc_block_mut();
        curr_func
            .blocks
            .push_back_ref(&mut alloc_block, mir_block)
            .expect("Failed to add block to function");
        drop(alloc_block);
        if switch_focus {
            self.set_focus(MirFocus::Block(curr_func, mir_block));
        }
    }

    pub fn add_inst_ref(&self, inst: MirInstRef) {
        let (_, block) = if let MirFocus::Block(func, block) = self.get_focus() {
            (func, block)
        } else {
            panic!("Cannot add instruction when focus is not on a block");
        };
        let mut alloc_inst = self.mir_module.borrow_alloc_inst_mut();
        let block_data = block.data_from_module(&self.mir_module);
        block_data.push_inst_ref(inst, &mut alloc_inst);
    }
    pub fn add_inst(&self, inst: MirInst) -> MirInstRef {
        let inst_ref = MirInstRef::from_module(&self.mir_module, inst);
        self.add_inst_ref(inst_ref);
        inst_ref
    }
}

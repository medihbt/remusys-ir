use std::{fmt::Debug, rc::Rc};

use crate::{
    base::slabref::SlabRef,
    mir::module::{
        MirGlobal, MirGlobalRef, MirModule,
        block::{MirBlock, MirBlockRef},
        func::MirFunc,
        global::{MirGlobalVariable, Section},
    },
};

pub struct FormatMir<'a>(pub &'a MirModule);

impl<'a> Debug for FormatMir<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let items: Vec<_> = self
            .0
            .items
            .iter()
            .map(|i| FormatMirGlobal(self.0, *i))
            .collect();
        f.debug_struct("MirModule")
            .field("name", &self.0.name)
            .field("globals", &items);
        Ok(())
    }
}

#[derive(Clone)]
struct FormatMirGlobal<'a>(pub &'a MirModule, pub MirGlobalRef);

impl<'a> Debug for FormatMirGlobal<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self(module, global) = self.clone();
        let global_data = global.data_from_module(module).clone();
        match global_data {
            MirGlobal::Variable(gvar) => FormatMirGVar(gvar).fmt(f),
            MirGlobal::UnnamedData(data) => data.fmt(f),
            MirGlobal::Function(func) => FormatMirFunc(func, module).fmt(f),
            _ => f.write_str("Useless"),
        }
        .expect("Failed to format MirGlobal");
        Ok(())
    }
}

struct FormatMirGVar(pub Rc<MirGlobalVariable>);

impl Debug for FormatMirGVar {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let v = &self.0;
        let variant = match v.common.section {
            Section::Text | Section::RoData => "Constant",
            Section::Data | Section::Bss | Section::None => "Variable",
        };
        let mut k = f.debug_struct(variant);
        k.field("name", &v.get_name())
            .field("section", &v.common.section)
            .field("linkage", &v.common.linkage)
            .field("align_log2", &v.common.align_log2)
            .field("size", &v.common.size);
        if v.common.section == Section::Bss {
            k.field("initval", &"[zeroinitializer]")
        } else {
            k.field("initval", &v.initval)
        }
        .finish()
    }
}

struct FormatMirFunc<'a>(pub Rc<MirFunc>, pub &'a MirModule);

impl<'a> Debug for FormatMirFunc<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let func = &self.0;
        let module = self.1;
        let alloc_block = module.borrow_alloc_block();
        let blocks: Vec<_> = if func.blocks.is_empty() {
            vec![]
        } else {
            func.blocks
                .view(&alloc_block)
                .into_iter()
                .map(|(bref, bb)| FormatMirBlock(bref, bb, module))
                .collect()
        };
        f.debug_struct("Function")
            .field("name", &func.get_name())
            .field("section", &func.common.section)
            .field("linkage", &func.common.linkage)
            .field("align_log2", &func.common.align_log2)
            .field("blocks", &blocks)
            .field("switch_tabs", &func.borrow_vec_switch_tabs())
            .field("args_regs", &func.arg_regs)
            .field("stack_layout", &func.borrow_inner().stack_layout)
            .finish()
    }
}

#[derive(Clone)]
struct FormatMirBlock<'a>(MirBlockRef, &'a MirBlock, &'a MirModule);

impl<'a> Debug for FormatMirBlock<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let FormatMirBlock(bref, bb, module) = self.clone();
        let alloc_inst = module.borrow_alloc_inst();
        let insts: Vec<_> = bb
            .insts
            .view(&alloc_inst)
            .into_iter()
            .map(|(_, inst)| inst)
            .collect();
        f.debug_struct("Block")
            .field("mir_ref", &bref.get_handle())
            .field("name", &bb.name)
            .field("insts", &insts)
            .finish()
    }
}

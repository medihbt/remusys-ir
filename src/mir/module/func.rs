use crate::{
    base::SlabRefList,
    mir::{
        inst::{MirInstRef, inst::MirInst, switch::VecSwitchTab},
        module::{
            MirModule,
            block::{MirBlock, MirBlockRef},
            global::{Linkage, MirGlobalCommon, Section},
            stack::{MirStackItem, MirStackLayout, StackItemKind},
            vreg_alloc::VirtRegAlloc,
        },
        operand::{physreg_set::MirPhysRegSet, reg::*},
        translate::mirgen::{
            operandgen::DispatchedReg,
            paramgen::{ArgPos, MirArgBuilder, MirArgInfo},
        },
    },
    typing::{FuncTypeRef, IValType, PrimType, TypeContext, ValTypeID},
};
use slab::Slab;
use std::{
    cell::{Cell, Ref, RefCell, RefMut},
    rc::Rc,
};

#[derive(Debug)]
pub struct MirFunc {
    pub common: MirGlobalCommon,

    pub arg_ir_types: Vec<ValTypeID>,
    pub ret_ir_type: ValTypeID,

    pub arg_info: MirArgInfo,
    pub blocks: SlabRefList<MirBlockRef>,

    pub has_call: Cell<bool>,
    inner: RefCell<MirFuncInner>,
}

#[derive(Debug)]
pub struct MirFuncInner {
    pub stack_layout: MirStackLayout,
    pub vreg_alloc: VirtRegAlloc,
    pub vec_switch_tabs: Vec<Rc<VecSwitchTab>>,
}

impl MirFunc {
    pub fn new_extern(name: String, func_ty: FuncTypeRef, type_ctx: &TypeContext) -> Self {
        let arg_ir_types = func_ty.args(type_ctx).to_vec();
        let ret_ir_type = func_ty.ret_type(type_ctx);
        let mut vreg_alloc = VirtRegAlloc::new();
        let arg_info = MirArgBuilder::new().build_func(func_ty, type_ctx, &mut vreg_alloc);
        let stack_layout = Self::init_stack(&arg_info);
        Self {
            common: MirGlobalCommon::new(name, Section::Text, 2, 0, Linkage::Extern),
            arg_ir_types,
            ret_ir_type,
            arg_info,
            blocks: SlabRefList::new_guide(),
            has_call: Cell::new(false),
            inner: RefCell::new(MirFuncInner {
                stack_layout,
                vreg_alloc,
                vec_switch_tabs: Vec::new(),
            }),
        }
    }
    pub fn new_define(
        name: String,
        func_ty: FuncTypeRef,
        type_ctx: &TypeContext,
        alloc_block: &mut Slab<MirBlock>,
    ) -> Self {
        let mut ret = Self::new_extern(name, func_ty, type_ctx);
        let inner_mut = ret.inner.get_mut();
        inner_mut.stack_layout.init_saved_regs_as_aapcs_callee();
        ret.blocks = SlabRefList::from_slab(alloc_block);
        ret.common.linkage = Linkage::Global;
        ret
    }

    fn init_stack(arg_info: &MirArgInfo) -> MirStackLayout {
        let mut stack = MirStackLayout::new();
        let mut count = 0;
        for &(ty, pos) in &arg_info.pos {
            let ArgPos::Stack(offset, vreg, stackpos) = pos else { continue };
            stack.args.push(MirStackItem {
                irtype: ty.into_ir(),
                index: count,
                stackpos_reg: stackpos,
                offset: offset as i64,
                size: vreg.get_size() as u64,
                size_with_padding: vreg.get_size().max(8) as u64,
                align_log2: vreg.get_align().ilog2() as u8,
                kind: StackItemKind::SpilledArg,
            });
            count += 1;
        }
        stack
    }

    /// 在虚拟寄存器 / 虚拟栈空间中添加一个变量，并返回指向这块栈空间的位置虚拟寄存器。
    pub fn add_spilled_variable(&self, irtype: ValTypeID, type_ctx: &TypeContext) -> GPR64 {
        let mut inner = self.inner.borrow_mut();
        let inner = &mut *inner;
        inner
            .stack_layout
            .add_variable(irtype, type_ctx, &mut inner.vreg_alloc)
            .stackpos_reg
    }

    pub fn is_extern(&self) -> bool {
        !self.blocks.is_valid()
    }
    pub fn is_define(&self) -> bool {
        self.blocks.is_valid()
    }
    pub fn get_name(&self) -> &str {
        self.common.name.as_str()
    }

    pub fn borrow_inner(&self) -> Ref<MirFuncInner> {
        self.inner.borrow()
    }
    pub fn borrow_inner_mut(&self) -> RefMut<MirFuncInner> {
        self.inner.borrow_mut()
    }

    pub fn borrow_spilled_args(&self) -> Ref<Vec<MirStackItem>> {
        Ref::map(self.inner.borrow(), |inner| &inner.stack_layout.args)
    }

    pub fn borrow_vec_switch_tabs(&self) -> Ref<Vec<Rc<VecSwitchTab>>> {
        Ref::map(self.inner.borrow(), |inner| &inner.vec_switch_tabs)
    }
    pub fn add_vec_switch_tab(&self, tab: Rc<VecSwitchTab>) -> usize {
        let mut inner = self.inner.borrow_mut();
        let ret = inner.vec_switch_tabs.len();
        inner.vec_switch_tabs.push(tab);
        ret
    }
    pub fn get_vec_switch_tab(&self, index: usize) -> Option<Rc<VecSwitchTab>> {
        self.inner.borrow().vec_switch_tabs.get(index).cloned()
    }

    pub fn dump_all_insts(
        &self,
        alloc_block: &Slab<MirBlock>,
        alloc_inst: &Slab<MirInst>,
    ) -> Vec<(MirBlockRef, MirInstRef)> {
        if self.is_extern() {
            return Vec::new();
        }
        let mut ret = Vec::new();
        for (block_ref, block) in self.blocks.view(alloc_block) {
            for (inst_ref, _) in block.insts.view(alloc_inst) {
                ret.push((block_ref.clone(), inst_ref.clone()));
            }
        }
        ret
    }
    pub fn dump_insts_when(
        &self,
        alloc_block: &Slab<MirBlock>,
        alloc_inst: &Slab<MirInst>,
        condition: impl Fn(&MirInst) -> bool,
    ) -> Vec<(MirBlockRef, MirInstRef)> {
        if self.is_extern() {
            return Vec::new();
        }
        let mut ret = Vec::new();
        for (block_ref, block) in self.blocks.view(alloc_block) {
            for (inst_ref, inst) in block.insts.view(alloc_inst) {
                if condition(inst) {
                    ret.push((block_ref.clone(), inst_ref.clone()));
                }
            }
        }
        ret
    }
    pub fn dump_insts_with_module(&self, module: &MirModule) -> Vec<(MirBlockRef, MirInstRef)> {
        let allocs = module.allocs.borrow();
        self.dump_all_insts(&allocs.block, &allocs.inst)
    }
    pub fn dump_insts_with_module_when(
        &self,
        module: &MirModule,
        condition: impl Fn(&MirInst) -> bool,
    ) -> Vec<(MirBlockRef, MirInstRef)> {
        let allocs = module.allocs.borrow();
        self.dump_insts_when(&allocs.block, &allocs.inst, condition)
    }

    pub fn reinit_saved_regs(&self, saved_regs: MirPhysRegSet) {
        self.inner
            .borrow_mut()
            .stack_layout
            .reinit_saved_regs(saved_regs);
    }
    pub fn arg_regs(&self) -> &[(u32, PrimType, DispatchedReg)] {
        self.arg_info.arg_regs.as_slice()
    }
}

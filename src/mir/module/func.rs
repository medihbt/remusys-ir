use crate::{
    base::slablist::SlabRefList,
    mir::{
        inst::{MirInstRef, inst::MirInst, switch::VecSwitchTab},
        module::{
            MirModule,
            block::{MirBlock, MirBlockRef},
            global::{Linkage, MirGlobalCommon, Section},
            stack::{MirStackItem, MirStackLayout, VirtRegAlloc},
        },
        operand::{IMirSubOperand, reg::*},
    },
    typing::{context::TypeContext, id::ValTypeID, types::FuncTypeRef},
};
use slab::Slab;
use std::{
    cell::{Ref, RefCell, RefMut},
    rc::Rc,
};

#[derive(Debug)]
pub struct MirFunc {
    pub common: MirGlobalCommon,

    pub arg_ir_types: Vec<ValTypeID>,
    pub ret_ir_type: ValTypeID,

    pub arg_regs: Vec<RegOperand>,
    pub blocks: SlabRefList<MirBlockRef>,

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
        let arg_ir_types = func_ty.get_args(type_ctx).to_vec();
        let ret_ir_type = func_ty.get_return_type(type_ctx);
        let mut vreg_alloc = VirtRegAlloc::new();
        let mut arg_regs = Vec::with_capacity(16.min(arg_ir_types.len()));
        let mut stack_layout = MirStackLayout::new();
        Self::init_args(
            &mut arg_regs,
            &mut stack_layout,
            &mut vreg_alloc,
            arg_ir_types.as_slice(),
        );
        Self {
            common: MirGlobalCommon::new(name, Section::Text, 2, Linkage::Extern),
            arg_ir_types,
            ret_ir_type,
            arg_regs,
            blocks: SlabRefList::new_guide(),
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
        MirStackLayout::new_aapcs_callee(&mut inner_mut.vreg_alloc);
        ret.blocks = SlabRefList::from_slab(alloc_block);
        ret.common.linkage = Linkage::Global;
        ret
    }

    fn init_args(
        arg_regs: &mut Vec<RegOperand>,
        stack: &mut MirStackLayout,
        vreg_alloc: &mut VirtRegAlloc,
        arg_tys: &[ValTypeID],
    ) {
        // 第 0-7 个整型/指针参数使用 GP 寄存器传递。这里记录已经使用的 GP 寄存器数量, 超过则转为栈分配.
        let mut gpreg_top: u32 = 0;
        // 第 0-7 个浮点参数使用 FP 寄存器传递。这里记录已经使用的 FP 寄存器数量, 超过则转为栈分配.
        let mut fpreg_top: u32 = 0;

        for &arg_ty in arg_tys {
            let (reg_top, mut reg) = match arg_ty {
                ValTypeID::Ptr | ValTypeID::Int(_) => (
                    &mut gpreg_top,
                    RegOperand::from(GPR64::new_empty().into_real()),
                ),
                ValTypeID::Float(_) => (
                    &mut fpreg_top,
                    RegOperand::from(FPR64::new_empty().into_real()),
                ),
                _ => panic!("Invalid argument type for MIR function: {arg_ty:?}"),
            };
            if *reg_top < 8 {
                // 使用寄存器传递参数
                reg.set_id(RegID::Phys(*reg_top));
                arg_regs.push(reg);
                *reg_top += 1;
            } else {
                // 使用栈传递参数
                stack.add_spilled_arg(arg_ty, vreg_alloc);
            }
        }
        stack.finish_arg_building();
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
}

use std::{cell::{Ref, RefCell, RefMut}, rc::Rc};

use slab::Slab;

use crate::{
    base::slablist::SlabRefList,
    mir::{
        inst::switch::{BinSwitchTab, VecSwitchTab},
        module::{
            block::{MirBlock, MirBlockRef},
            global::{Linkage, MirGlobalCommon, Section},
            stack::{MirStackLayout, VirtRegAlloc},
        },
        operand::reg::{PhysReg, VirtReg},
    },
    typing::{
        context::TypeContext,
        id::ValTypeID,
        types::{FloatTypeKind, FuncTypeRef},
    },
};

#[derive(Debug)]
pub struct MirFunc {
    pub common: MirGlobalCommon,

    pub arg_ir_types: Vec<ValTypeID>,
    pub ret_ir_type: ValTypeID,

    pub arg_regs: Vec<PhysReg>,
    pub blocks: SlabRefList<MirBlockRef>,

    inner: RefCell<MirFuncInner>,
}

#[derive(Debug)]
pub struct MirFuncInner {
    pub stack_layout: MirStackLayout,
    pub vreg_alloc: VirtRegAlloc,
    pub vec_switch_tabs: Vec<Rc<VecSwitchTab>>,
    pub bin_switch_tabs: Vec<Rc<BinSwitchTab>>,
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
                bin_switch_tabs: Vec::new(),
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
        ret.blocks = SlabRefList::from_slab(alloc_block);
        ret.common.linkage = Linkage::Global;
        ret
    }

    fn init_args(
        arg_regs: &mut Vec<PhysReg>,
        stack: &mut MirStackLayout,
        vreg_alloc: &mut VirtRegAlloc,
        arg_tys: &[ValTypeID],
    ) {
        // 第 0-7 个整型/指针参数使用 GP 寄存器传递。这里记录已经使用的 GP 寄存器数量, 超过则转为栈分配.
        let mut gpreg_top = 0;
        // 第 0-7 个浮点参数使用 FP 寄存器传递。这里记录已经使用的 FP 寄存器数量, 超过则转为栈分配.
        let mut fpreg_top = 0;

        for &arg_ty in arg_tys {
            let (reg_top, mut preg) = match arg_ty {
                ValTypeID::Ptr | ValTypeID::Int(_) => (&mut gpreg_top, PhysReg::x(0)),
                ValTypeID::Float(_) => (&mut fpreg_top, PhysReg::fp_s(0)),
                _ => panic!("Invalid argument type for MIR function: {arg_ty:?}"),
            };
            if *reg_top < 8 {
                // 使用寄存器传递参数
                *preg.id_mut().unwrap() = (*reg_top) as u8;
                arg_regs.push(preg);
                *reg_top += 1;
            } else {
                // 使用栈传递参数
                stack.add_spilled_arg(arg_ty, vreg_alloc);
            }
        }
        stack.finish_arg_building();
    }

    /// 在虚拟寄存器 / 虚拟栈空间中添加一个变量，并返回其虚拟寄存器。
    ///
    /// 如果变量是整数、浮点或者指针这类一个寄存器就能装下并且没有什么内部结构的类型, 直接分配寄存器返回即可
    /// 除此之外, 结构体和数组类型的变量需要在栈上分配空间, 返回一个指向栈上分配的虚拟寄存器。
    pub fn add_variable(&mut self, irtype: ValTypeID, type_ctx: &TypeContext) -> VirtReg {
        let mut inner = self.inner.borrow_mut();
        let inner = &mut *inner;
        match irtype {
            ValTypeID::Ptr => {
                // 指针类型的变量需要分配一个虚拟寄存器。
                let vreg = inner.vreg_alloc.alloc_gp();
                vreg.subreg_index_mut().insert_bits_log2(6); // aarch64
                vreg.clone()
            }
            ValTypeID::Int(bits) => {
                // 整数类型的变量需要分配一个虚拟寄存器。
                // 不支持大于 64 位的整数类型。
                if bits > 64 || !bits.is_power_of_two() {
                    panic!("Unsupported integer type for MIR function: {irtype:?}");
                }
                let vreg = inner.vreg_alloc.alloc_gp();
                vreg.subreg_index_mut()
                    .insert_bits_log2(bits.trailing_zeros() as u8);
                vreg.clone()
            }
            ValTypeID::Float(fpkind) => {
                // 浮点类型的变量需要分配一个虚拟寄存器。
                let vreg = inner.vreg_alloc.alloc_float();
                vreg.subreg_index_mut().insert_bits_log2(match fpkind {
                    FloatTypeKind::Ieee32 => 5, // 32 bits
                    FloatTypeKind::Ieee64 => 6, // 64 bits
                });
                vreg.clone()
            }
            ValTypeID::Array(_) | ValTypeID::Struct(_) | ValTypeID::StructAlias(_) => {
                // 结构体和数组类型的变量需要在栈上分配空间。
                inner.stack_layout
                    .add_variable(irtype, type_ctx, &mut inner.vreg_alloc)
                    .virtreg
            }
            _ => panic!(
                "Invalid variable type for MIR function: {}",
                irtype.get_display_name(type_ctx)
            ),
        }
    }

    pub fn is_extern(&self) -> bool {
        !self.blocks.is_valid()
    }
    pub fn is_define(&self) -> bool {
        self.blocks.is_valid()
    }

    pub fn borrow_inner(&self) -> Ref<MirFuncInner> {
        self.inner.borrow()
    }
    pub fn borrow_inner_mut(&self) -> RefMut<MirFuncInner> {
        self.inner.borrow_mut()
    }

    pub fn borrow_vec_switch_tabs(&self) -> Ref<Vec<Rc<VecSwitchTab>>> {
        Ref::map(self.inner.borrow(), |inner| &inner.vec_switch_tabs)
    }
    pub fn borrow_bin_switch_tabs(&self) -> Ref<Vec<Rc<BinSwitchTab>>> {
        Ref::map(self.inner.borrow(), |inner| &inner.bin_switch_tabs)
    }
    pub fn add_vec_switch_tab(&self, tab: Rc<VecSwitchTab>) -> usize {
        let mut inner = self.inner.borrow_mut();
        let ret = inner.vec_switch_tabs.len();
        inner.vec_switch_tabs.push(tab);
        ret
    }
    pub fn add_bin_switch_tab(&self, tab: Rc<BinSwitchTab>) -> usize {
        let mut inner = self.inner.borrow_mut();
        let ret = inner.bin_switch_tabs.len();
        inner.bin_switch_tabs.push(tab);
        ret
    }
    pub fn get_vec_switch_tab(&self, index: usize) -> Option<Rc<VecSwitchTab>> {
        self.inner.borrow().vec_switch_tabs.get(index).cloned()
    }
    pub fn get_bin_switch_tab(&self, index: usize) -> Option<Rc<BinSwitchTab>> {
        self.inner.borrow().bin_switch_tabs.get(index).cloned()
    }
}

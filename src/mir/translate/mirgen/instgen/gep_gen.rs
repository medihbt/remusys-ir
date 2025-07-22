use crate::{
    base::slabref::SlabRef,
    ir::{
        ValueSSA,
        constant::data::ConstData,
        inst::{InstData, InstRef, gep::IndexChainNode, usedef::UseData},
        module::Module,
    },
    mir::{
        inst::{IMirSubInst, impls::*, inst::MirInst, opcode::MirOP},
        module::stack::VirtRegAlloc,
        operand::{
            IMirSubOperand, MirOperand,
            compound::MirSymbolOp,
            imm::*,
            imm_traits,
            reg::{GPR64, GPReg, RegOP, RegUseFlags},
        },
        translate::mirgen::operandgen::OperandMap,
    },
    typing::{context::TypeContext, id::ValTypeID, types::StructTypeRef},
};
use slab::Slab;
use std::{cell::Ref, collections::VecDeque};

pub(crate) fn dispatch_gep(
    ir_module: &Module,
    operand_map: &OperandMap,
    vreg_alloc: &mut VirtRegAlloc,
    out_insts: &mut VecDeque<MirInst>,
    ir_ref: InstRef,
    alloc_inst: &Slab<InstData>,
    alloc_use: Ref<Slab<UseData>>,
) {
    let (index_chain, base_ptr) = match ir_ref.to_slabref_unwrap(alloc_inst) {
        InstData::IndexPtr(_, gep) => (
            gep.dump_index_chain(ir_module),
            gep.base_ptr.get_operand(&alloc_use),
        ),
        _ => panic!("Expected IndexPtr instruction"),
    };

    let base_ptr_mir = operand_map.find_operand_no_constdata(&base_ptr).unwrap();

    let mut curr_ptr = {
        use MirOperand::{F32, F64, Global, Imm32, Imm64, Label, PState, SwitchTab, VFReg};

        match base_ptr_mir {
            MirOperand::GPReg(GPReg(id, ..)) => {
                // 该 GEP 生成的指令会直接修改 curr_ptr 的值, 如果后面的指令还要用到这个寄存器的话
                // 就完蛋了. 所以仍然需要分配一个虚拟寄存器, 然后找个 mov 指令把它的值复制过去.
                let curr_ptr = vreg_alloc.insert_gp(GPR64::new_empty().into_real());
                let curr_ptr = GPR64::from_real(curr_ptr);
                let mov_inst = Una64R::new(
                    MirOP::Mov64R,
                    curr_ptr,
                    GPR64(id, RegUseFlags::empty()),
                    None,
                );
                out_insts.push_back(mov_inst.into_mir());
                curr_ptr
            }
            Label(bb) => symbol_to_gpreg_ptr(MirSymbolOp::Label(bb), vreg_alloc, out_insts),
            Global(g) => symbol_to_gpreg_ptr(MirSymbolOp::Global(g), vreg_alloc, out_insts),
            SwitchTab(index) => {
                symbol_to_gpreg_ptr(MirSymbolOp::SwitchTab(index), vreg_alloc, out_insts)
            }
            VFReg(_) | PState(_) | Imm64(_) | Imm32(_) | F32(_) | F64(_) => {
                panic!(
                    "Expected GEP base pointer to be a valid operand (GPReg or symbol), found {base_ptr_mir:?}"
                );
            }
            MirOperand::None => panic!("Expected GEP base pointer to be a valid operand"),
        }
    };

    let mut integral_offset: Option<usize> = None;
    let mut type_before_unpack = ValTypeID::Void;
    let type_ctx = &*ir_module.type_ctx;
    for IndexChainNode { index, unpacked_ty } in index_chain {
        let ty_size = unpacked_ty
            .get_instance_size(type_ctx)
            .expect("Failed to get type size");
        if let Some(index) = value_as_int_index(index) {
            let this_level_offset = match type_before_unpack {
                ValTypeID::Void | ValTypeID::Array(_) => {
                    // 对于数组或未定义类型，使用 index * size 计算偏移
                    index as usize * ty_size
                }
                ValTypeID::Struct(s) => {
                    // 对于结构体，使用结构体元素的偏移
                    struct_get_offset(type_ctx, s, index as usize)
                }
                ValTypeID::StructAlias(a) => {
                    let s = a.get_aliasee(type_ctx);
                    struct_get_offset(type_ctx, s, index as usize)
                }
                _ => panic!("Unsupported type for index pointer: {type_before_unpack:?}"),
            };
            integral_offset = Some(integral_offset.unwrap_or(0) + this_level_offset);
        } else {
            handle_nonconst_index(
                operand_map,
                vreg_alloc,
                out_insts,
                &mut curr_ptr,
                &mut integral_offset,
                index,
                type_before_unpack,
                ty_size,
            );
        }
        // 更新当前指针和类型
        type_before_unpack = unpacked_ty;
    }
}

fn symbol_to_gpreg_ptr(
    symbol: MirSymbolOp,
    vreg_alloc: &mut VirtRegAlloc,
    out_insts: &mut VecDeque<MirInst>,
) -> GPR64 {
    let curr_ptr = vreg_alloc.insert_gp(GPR64::new_empty().into_real());
    let curr_ptr = GPR64::from_real(curr_ptr);
    let loadaddr_inst = LoadConst64Symbol::new(MirOP::LoadConst64Symbol, curr_ptr, symbol);
    out_insts.push_back(loadaddr_inst.into_mir());
    curr_ptr
}

fn handle_nonconst_index(
    operand_map: &OperandMap<'_>,
    vreg_alloc: &mut VirtRegAlloc,
    out_insts: &mut VecDeque<MirInst>,
    curr_ptr: &mut GPR64,
    integral_offset: &mut Option<usize>,
    index: ValueSSA,
    type_before_unpack: ValTypeID,
    ty_size: usize,
) {
    // 遇到一个不是整数的索引. 现在要把前面的账结了.
    if let Some(offset) = *integral_offset {
        *curr_ptr = make_ptr_add(vreg_alloc, *curr_ptr, offset as i64, out_insts);
        *integral_offset = None;
    }

    // 现在处理这个非整数索引.
    if !matches!(type_before_unpack, ValTypeID::Void | ValTypeID::Array(_)) {
        panic!(
            "Unsupported type for index pointer with non-constant index: {type_before_unpack:?}"
        );
    }

    let index = operand_map.find_operand_no_constdata(&index).unwrap();
    let index = match index {
        MirOperand::GPReg(GPReg(id, _, uf)) => GPR64(id, uf),
        _ => panic!("Unsupported index type for index pointer: {index:?}"),
    };

    if ty_size.is_power_of_two() {
        // 如果是 2 的幂次方，直接使用位移操作
        let shift_amount = ty_size.trailing_zeros() as u8;
        let add_inst = Bin64R::new(
            MirOP::Add64R,
            *curr_ptr,
            *curr_ptr,
            index,
            Some(RegOP::LSL(shift_amount)),
        );
        out_insts.push_back(add_inst.into_mir());
    } else {
        // 否则使用乘法
        let tysize = vreg_alloc.insert_gp(GPR64::new_empty().into_real());
        let tysize = GPR64::from_real(tysize);
        let ldr64_inst = LoadConst64::new(
            MirOP::LoadConst64,
            tysize,
            Imm64(ty_size as u64, ImmKind::Full),
        );
        // current_ptr = index * ty_size + current_ptr
        let mla_inst = TenaryG64::new(MirOP::MAdd64, *curr_ptr, tysize, index, *curr_ptr);
        out_insts.push_back(ldr64_inst.into_mir());
        out_insts.push_back(mla_inst.into_mir());
    }
}

/// SysY 中的数组下标都是 i32 的, 因此这里用 i64 似乎也可以.
fn value_as_int_index(value: ValueSSA) -> Option<i64> {
    let ValueSSA::ConstData(cdata) = value else {
        return None;
    };
    match cdata {
        ConstData::Zero(_) | ConstData::PtrNull(_) => Some(0),
        ConstData::Int(bits, value) => {
            Some(ConstData::iconst_value_get_real_signed(bits, value) as i64)
        }
        ConstData::Undef(_) => None,
        ConstData::Float(..) => panic!("Unexpected float constant in index pointer"),
    }
}

fn struct_get_offset(type_ctx: &TypeContext, struct_ty: StructTypeRef, index: usize) -> usize {
    let mut offset = 0;
    for i in 0..index {
        offset += struct_ty
            .get_element_type(type_ctx, i)
            .unwrap()
            .get_instance_size(type_ctx)
            .unwrap();
    }
    offset
}
fn make_ptr_add(
    vreg_alloc: &mut VirtRegAlloc,
    base_ptr_mir: GPR64,
    integral_offset: i64,
    out_insts: &mut VecDeque<MirInst>,
) -> GPR64 {
    if integral_offset == 0 {
        return base_ptr_mir;
    }
    if imm_traits::is_calc_imm(integral_offset as u64) {
        let add_inst = Bin64RC::new(
            MirOP::Add64I,
            base_ptr_mir,
            base_ptr_mir,
            ImmCalc::new(integral_offset as u32),
        );
        out_insts.push_back(add_inst.into_mir());
        base_ptr_mir
    } else if imm_traits::is_calc_imm((-integral_offset) as u64) {
        let add_inst = Bin64RC::new(
            MirOP::Add64I,
            base_ptr_mir,
            base_ptr_mir,
            ImmCalc::new((-integral_offset) as u32),
        );
        out_insts.push_back(add_inst.into_mir());
        base_ptr_mir
    } else {
        let offset_reg = vreg_alloc.insert_gp(GPR64::new_empty().into_real());
        let offset_reg = GPR64::from_real(offset_reg);
        let load_const = LoadConst64::new(
            MirOP::LoadConst64,
            offset_reg,
            Imm64(integral_offset as u64, ImmKind::Full),
        );
        out_insts.push_back(load_const.into_mir());
        let add_inst = Bin64R::new(MirOP::Add64R, base_ptr_mir, base_ptr_mir, offset_reg, None);
        out_insts.push_back(add_inst.into_mir());
        base_ptr_mir
    }
}

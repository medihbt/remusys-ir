use crate::{
    base::INullableValue,
    ir::{
        BlockData, BlockRef, FuncRef, IRAllocs, IRAllocsReadable, ISubValueSSA, InstKind,
        JumpTargetKind, Module, Opcode, UseKind, ValueSSA, ValueSSAClass,
        checking::{ValueCheckError, optype_isclass, optype_matches, type_matches},
        inst::*,
    },
    typing::{IValType, TypeContext, ValTypeClass, ValTypeID},
};
use std::collections::BTreeSet;

pub struct InstCheckCtx<'a> {
    pub(crate) type_ctx: &'a TypeContext,
    pub(crate) allocs: &'a IRAllocs,
}

impl<'a> InstCheckCtx<'a> {
    pub fn new(type_ctx: &'a TypeContext, allocs: &'a impl IRAllocsReadable) -> Self {
        Self { type_ctx, allocs: allocs.get_allocs_ref() }
    }
    pub fn from_module(module: &'a Module) -> Self {
        Self { type_ctx: &module.type_ctx, allocs: &module.allocs }
    }

    pub fn allocs(&self) -> &IRAllocs {
        self.allocs
    }

    pub fn check_module(&self, module: &Module) -> Result<(), ValueCheckError> {
        for (name, global) in &*module.globals.borrow() {
            log::debug!("checking global {name}");
            let Some(fref) = FuncRef::try_from_real(*global, &self.allocs.globals) else {
                continue;
            };
            self.check_func(fref)?;
        }
        Ok(())
    }

    pub fn check_func(&self, fref: FuncRef) -> Result<(), ValueCheckError> {
        let func = fref.to_data(&self.allocs.globals);
        let Some(body) = func.get_body() else {
            // extern functions detected, skip checking
            return Ok(());
        };
        let body = body.view(&self.allocs.blocks).into_iter();
        for (id, (bref, block)) in body.enumerate() {
            use super::FuncLayoutError as F;
            use super::ValueCheckError as V;
            if bref == func.entry.get() && id != 0 {
                return Err(V::FuncLayoutError(F::EntryNotInFront(fref)));
            }
            self.check_block(block)?;
        }
        Ok(())
    }

    pub fn check_block(&self, block: &BlockData) -> Result<(), ValueCheckError> {
        let mut has_terminator = false;
        let mut phi_ends = false;
        for (iref, inst) in block.insts.view(&self.allocs.insts) {
            use super::BlockLayoutError as L;
            use ValueCheckError::*;
            match (phi_ends, inst) {
                (false, InstData::Phi(_)) => {}
                (false, InstData::PhiInstEnd(_)) => phi_ends = true,
                (false, _) => return Err(BlockLayoutError(L::DirtyPhiSection(iref))),
                (true, InstData::Phi(_)) => {
                    return Err(BlockLayoutError(L::PhiNotInHead(PhiRef(iref))));
                }
                _ => {}
            }
            if has_terminator && inst.is_terminator() {
                return Err(BlockLayoutError(L::MultipleTerminator(iref)));
            }
            if inst.is_terminator() {
                has_terminator = true;
            }
            self.check_inst(inst)?;
        }
        Ok(())
    }

    pub fn check_inst(&self, inst: &InstData) -> Result<(), ValueCheckError> {
        match inst {
            InstData::Ret(ret) => self.check_ret(ret),
            InstData::Jump(jump) => self.check_jump(jump),
            InstData::Br(br) => self.check_br(br),
            InstData::Switch(switch) => self.check_switch(switch),
            InstData::Alloca(alloca) => self.check_alloca(alloca),
            InstData::BinOp(bin_op) => bin_op.validate(&self.allocs),
            InstData::Call(call_op) => self.check_callop(call_op),
            InstData::Cast(cast_op) => self.check_castop(cast_op),
            InstData::Phi(phi_node) => self.check_phi(phi_node),
            InstData::Cmp(cmp_op) => self.check_cmpop(cmp_op),
            InstData::GEP(index_ptr) => self.check_gep(index_ptr),
            InstData::Select(select_op) => self.check_select(select_op),
            InstData::Load(load_op) => self.check_load(load_op),
            InstData::Store(store_op) => self.check_store(store_op),
            InstData::AmoRmw(amo_rmw) => self.check_amo_rmw(amo_rmw),
            _ => Ok(()),
        }
    }

    pub fn check_ret(&self, ret: &Ret) -> Result<(), ValueCheckError> {
        use super::ValueCheckError::*;
        let func = ret.get_parent_func(self.allocs());
        let func_retty = func.to_data(&self.allocs.globals).return_type;

        if ret.get_valtype() != func_retty {
            return Err(OpTypeMismatch(
                ret.get_self_ref(),
                UseKind::RetValue,
                func_retty,
                ret.get_valtype(),
            ));
        }
        if !ret.has_retval() {
            return Ok(());
        }
        Self::operand_nonnull(ret.get_retval(), ret.get_self_ref(), UseKind::RetValue)?;
        type_matches(func_retty, ret.get_retval(), self.allocs())
    }

    pub fn check_jump(&self, jump: &Jump) -> Result<(), ValueCheckError> {
        Self::jump_target_nonnull(jump.get_target(), jump.get_self_ref(), JumpTargetKind::Jump)
    }

    pub fn check_br(&self, br: &Br) -> Result<(), ValueCheckError> {
        let brref = br.get_self_ref();
        Self::jump_target_nonnull(br.get_if_true(), brref, JumpTargetKind::BrTrue)?;
        Self::jump_target_nonnull(br.get_if_false(), brref, JumpTargetKind::BrFalse)?;

        let cond = Self::operand_nonnull(br.get_cond(), brref, UseKind::BranchCond)?;
        type_matches(ValTypeID::Int(1), cond, self.allocs())
    }

    /// 检查 Switch 指令的操作数与跳转目标规则
    ///
    /// ## Switch 指令规则总结：
    ///
    /// ### 操作数规则：
    /// 1. **条件操作数**：必须非空且为整数类型（任意位宽的 `ValTypeID::Int(_)`）
    ///    - 用于与各个 case 值进行匹配比较
    ///    - 支持有符号和无符号整数
    ///
    /// ### 跳转目标规则：
    /// 1. **默认跳转目标**：必须非空且为有效的基本块引用
    ///    - 当条件值不匹配任何 case 时的跳转目标
    ///    - 对应 `JumpTargetKind::SwitchDefault`
    ///
    /// 2. **Case 跳转目标**：每个 case 都必须有非空且有效的基本块引用
    ///    - 当条件值匹配对应 case 值时的跳转目标
    ///    - 对应 `JumpTargetKind::SwitchCase(value)`
    ///
    /// ### 唯一性约束：
    /// 1. **Case 值唯一性**：所有 case 值必须互不相同
    ///    - 不允许重复的 case 值
    ///    - 违反时返回 `ValueCheckError::DuplicatedSwitchCase`
    ///
    /// ### 完整性约束：
    /// 1. **必须有默认目标**：即使所有可能的值都被 case 覆盖，也必须提供默认目标
    /// 2. **所有跳转目标有效**：默认目标和所有 case 目标都必须指向有效的基本块
    ///
    /// ## 检查流程：
    /// 1. 验证默认跳转目标非空
    /// 2. 验证条件操作数非空且为整数类型
    /// 3. 遍历所有 case，检查：
    ///    - case 值的唯一性
    ///    - case 跳转目标的有效性
    pub fn check_switch(&self, switch: &Switch) -> Result<(), ValueCheckError> {
        let sref = switch.get_self_ref();
        Self::jump_target_nonnull(switch.get_default(), sref, JumpTargetKind::SwitchDefault)?;
        let cond = Self::operand_nonnull(switch.get_cond(), sref, UseKind::SwitchCond)?;
        optype_isclass(
            sref,
            UseKind::SwitchCond,
            ValTypeClass::Int,
            cond,
            self.allocs(),
        )?;

        let mut cases = BTreeSet::new();
        for c in &*switch.cases() {
            let kind = c.kind;
            if cases.contains(&kind) {
                return Err(ValueCheckError::DuplicatedSwitchCase(sref, kind));
            }
            cases.insert(kind);
            let block = c.get_block();
            Self::jump_target_nonnull(block, sref, kind)?;
        }
        Ok(())
    }

    pub fn check_alloca(&self, alloca: &Alloca) -> Result<(), ValueCheckError> {
        if alloca.pointee_ty.makes_instance() {
            Ok(())
        } else {
            Err(ValueCheckError::InstTypeNotSized(
                alloca.get_self_ref(),
                alloca.pointee_ty,
            ))
        }
    }

    pub fn check_binop(&self, bin_op: &BinOp) -> Result<(), ValueCheckError> {
        bin_op.validate(&self.allocs)
    }

    /// 检查 Call 指令的操作数规则
    ///
    /// ## Call 指令规则总结：
    ///
    /// ### 操作数规则：
    /// 1. **被调用函数操作数**：必须非空且为指针类型（`ValTypeID::Ptr`）
    ///    - 通常为全局函数引用 `ValueSSA::Global`
    ///    - 全局引用在 IR 中统一按指针类型处理
    ///    - 具体函数类型信息存储在 `CallOp.callee_ty` 字段中
    ///
    /// 2. **参数操作数**：每个参数都必须非空且类型匹配
    ///    - 固定参数：类型必须与函数签名中对应位置的参数类型完全匹配
    ///    - 可变参数：额外参数可以是任意类型（对于 vararg 函数）
    ///
    /// ### 参数数量规则：
    /// 1. **固定参数函数**：实际参数数量必须等于函数签名中定义的参数数量
    /// 2. **可变参数函数**：实际参数数量必须 >= 函数签名中固定参数的数量
    ///    - 可以传递超过固定参数数量的额外参数
    ///    - 额外参数的类型检查由运行时处理
    ///
    /// ### 类型匹配规则：
    /// 1. **固定参数类型检查**：每个固定参数的类型必须与函数签名匹配
    /// 2. **返回值类型**：指令的返回值类型必须与函数签名的返回值类型匹配
    /// 3. **可变参数类型**：超出固定参数范围的参数类型不做严格检查
    ///
    /// ## 检查流程：
    /// 1. 验证被调用函数操作数非空且为指针类型
    /// 2. 检查参数数量是否符合函数签名要求
    /// 3. 逐一检查固定参数的类型匹配
    /// 4. 验证所有参数操作数非空
    pub fn check_callop(&self, callop: &CallOp) -> Result<(), ValueCheckError> {
        let callref = callop.get_self_ref();

        // 1. 检查被调用函数操作数. 被调用者的类型应该是指针
        let callee = Self::operand_nonnull(callop.get_callee(), callref, UseKind::CallOpCallee)?;
        optype_matches(
            callref,
            UseKind::CallOpCallee,
            ValTypeID::Ptr,
            callee,
            self.allocs(),
        )?;

        // 2. 检查参数数量
        let actual_nargs = callop.args().len();
        let expected_fixed_nargs = callop.fixed_nargs;

        let argcount_correct = if callop.is_vararg {
            actual_nargs >= expected_fixed_nargs
        } else {
            actual_nargs == expected_fixed_nargs
        };
        if !argcount_correct {
            return Err(ValueCheckError::CallArgCountUnmatch(
                callref,
                expected_fixed_nargs as u32,
                actual_nargs as u32,
            ));
        }

        // 3. 检查每个参数
        for (index, arg_use) in callop.args().iter().enumerate() {
            // 检查参数操作数非空
            let arg_val = Self::operand_nonnull(
                arg_use.get_operand(),
                callref,
                UseKind::CallOpArg(index as u32),
            )?;

            // 检查固定参数的类型匹配
            if index < expected_fixed_nargs {
                let expected_arg_type = callop.callee_ty.get_arg(self.type_ctx, index);
                optype_matches(
                    callref,
                    UseKind::CallOpArg(index as u32),
                    expected_arg_type,
                    arg_val,
                    self.allocs(),
                )?;
            }
            // 可变参数部分不做类型检查，由运行时处理
        }

        Ok(())
    }

    /// 检查 Cast 指令的操作数与类型转换规则
    ///
    /// ## Cast 指令规则总结：
    ///
    /// ### 操作数规则：
    /// 1. **源操作数**：必须非空且类型与声明的源类型匹配
    ///    - 操作数的实际类型必须与 `cast.fromty` 字段匹配
    ///    - 确保类型转换的源类型信息准确
    ///
    /// ### 类型转换规则：
    ///
    /// #### 整数类型转换：
    /// 1. **零扩展 (Zext)**：从较小位宽整数扩展到较大位宽，高位补零
    /// 2. **符号扩展 (Sext)**：从较小位宽整数扩展到较大位宽，按符号位扩展
    /// 3. **截断 (Trunc)**：从较大位宽整数截断到较小位宽，丢弃高位
    ///
    /// #### 浮点类型转换：
    /// 4. **浮点扩展 (Fpext)**：f32 -> f64，提高精度
    /// 5. **浮点截断 (Fptrunc)**：f64 -> f32，降低精度
    ///
    /// #### 整数与浮点转换：
    /// 6. **有符号整数到浮点 (Sitofp)**：任意位宽有符号整数 -> 任意浮点类型
    /// 7. **无符号整数到浮点 (Uitofp)**：任意位宽无符号整数 -> 任意浮点类型
    /// 8. **浮点到有符号整数 (Fptosi)**：任意浮点类型 -> 任意位宽整数（带符号截断）
    ///
    /// #### 位级转换：
    /// 9. **位转换 (Bitcast)**：不改变位模式的类型重解释
    /// 10. **整数到指针 (IntToPtr)**：整数值转换为指针
    /// 11. **指针到整数 (PtrToInt)**：指针转换为整数值
    ///
    /// ## 检查流程：
    /// 1. 验证源操作数非空
    /// 2. 检查操作数类型与声明的源类型匹配
    /// 3. 根据操作码验证类型转换的合法性
    pub fn check_castop(&self, cast: &CastOp) -> Result<(), ValueCheckError> {
        let castref = cast.get_self_ref();
        let fromty = cast.fromty;
        let intoty = cast.get_valtype();

        // 1. 检查源操作数非空
        let from_operand = Self::operand_nonnull(cast.get_from(), castref, UseKind::CastOpFrom)?;

        // 2. 检查操作数类型与声明的源类型匹配
        type_matches(fromty, from_operand, self.allocs())?;

        // 3. 验证类型转换的合法性
        self.cast_matches(castref, cast.get_opcode(), fromty, intoty)?;

        Ok(())
    }

    pub(crate) fn cast_matches(
        &self,
        inst: InstRef,
        opcode: Opcode,
        fromty: ValTypeID,
        intoty: ValTypeID,
    ) -> Result<(), ValueCheckError> {
        use crate::typing::FPKind;

        match opcode {
            // 整数位宽转换：零扩展、符号扩展、截断
            Opcode::Zext | Opcode::Sext | Opcode::Trunc => {
                let ValTypeID::Int(from_bits) = fromty else {
                    return Err(ValueCheckError::OpTypeNotClass(
                        inst,
                        UseKind::CastOpFrom,
                        fromty,
                        ValTypeClass::Int,
                    ));
                };
                let ValTypeID::Int(into_bits) = intoty else {
                    return Err(ValueCheckError::InstTypeNotClass(
                        inst,
                        intoty,
                        ValTypeClass::Int,
                    ));
                };
                let matches = if matches!(opcode, Opcode::Zext | Opcode::Sext) {
                    from_bits <= into_bits // 扩展：源位宽必须 <= 目标位宽
                } else {
                    from_bits >= into_bits // 截断：源位宽必须 >= 目标位宽
                };
                if matches {
                    Ok(())
                } else {
                    Err(ValueCheckError::CastUnmatch(inst, opcode, fromty, intoty))
                }
            }

            // 浮点扩展：从低精度浮点到高精度浮点
            Opcode::Fpext => {
                let ValTypeID::Float(from_fp) = fromty else {
                    return Err(ValueCheckError::OpTypeNotClass(
                        inst,
                        UseKind::CastOpFrom,
                        fromty,
                        ValTypeClass::Float,
                    ));
                };
                let ValTypeID::Float(into_fp) = intoty else {
                    return Err(ValueCheckError::InstTypeNotClass(
                        inst,
                        intoty,
                        ValTypeClass::Float,
                    ));
                };
                // 只支持 f32 -> f64 的扩展
                if matches!((from_fp, into_fp), (FPKind::Ieee32, FPKind::Ieee64)) {
                    Ok(())
                } else {
                    Err(ValueCheckError::CastUnmatch(inst, opcode, fromty, intoty))
                }
            }

            // 浮点截断：从高精度浮点到低精度浮点
            Opcode::Fptrunc => {
                let ValTypeID::Float(from_fp) = fromty else {
                    return Err(ValueCheckError::OpTypeNotClass(
                        inst,
                        UseKind::CastOpFrom,
                        fromty,
                        ValTypeClass::Float,
                    ));
                };
                let ValTypeID::Float(into_fp) = intoty else {
                    return Err(ValueCheckError::InstTypeNotClass(
                        inst,
                        intoty,
                        ValTypeClass::Float,
                    ));
                };
                // 只支持 f64 -> f32 的截断
                if matches!((from_fp, into_fp), (FPKind::Ieee64, FPKind::Ieee32)) {
                    Ok(())
                } else {
                    Err(ValueCheckError::CastUnmatch(inst, opcode, fromty, intoty))
                }
            }

            // 有符号整数到浮点数转换
            Opcode::Sitofp => {
                let ValTypeID::Int(_) = fromty else {
                    return Err(ValueCheckError::OpTypeNotClass(
                        inst,
                        UseKind::CastOpFrom,
                        fromty,
                        ValTypeClass::Int,
                    ));
                };
                let ValTypeID::Float(_) = intoty else {
                    return Err(ValueCheckError::InstTypeNotClass(
                        inst,
                        intoty,
                        ValTypeClass::Float,
                    ));
                };
                Ok(()) // 任意整数位宽到任意浮点类型都支持
            }

            // 无符号整数到浮点数转换
            Opcode::Uitofp => {
                let ValTypeID::Int(_) = fromty else {
                    return Err(ValueCheckError::OpTypeNotClass(
                        inst,
                        UseKind::CastOpFrom,
                        fromty,
                        ValTypeClass::Int,
                    ));
                };
                let ValTypeID::Float(_) = intoty else {
                    return Err(ValueCheckError::InstTypeNotClass(
                        inst,
                        intoty,
                        ValTypeClass::Float,
                    ));
                };
                Ok(()) // 任意整数位宽到任意浮点类型都支持
            }

            // 浮点数到有符号整数转换
            Opcode::Fptosi => {
                let ValTypeID::Float(_) = fromty else {
                    return Err(ValueCheckError::OpTypeNotClass(
                        inst,
                        UseKind::CastOpFrom,
                        fromty,
                        ValTypeClass::Float,
                    ));
                };
                let ValTypeID::Int(_) = intoty else {
                    return Err(ValueCheckError::InstTypeNotClass(
                        inst,
                        intoty,
                        ValTypeClass::Int,
                    ));
                };
                Ok(()) // 任意浮点类型到任意整数位宽都支持
            }

            // 位转换：不改变位模式的类型转换
            Opcode::Bitcast => {
                let tctx = self.type_ctx;
                let from_bits =
                    fromty
                        .try_get_bits(tctx)
                        .ok_or(ValueCheckError::OpTypeNotSized(
                            inst,
                            UseKind::CastOpFrom,
                            fromty,
                        ))?;
                let into_bits = intoty
                    .try_get_bits(tctx)
                    .ok_or(ValueCheckError::InstTypeNotSized(inst, intoty))?;
                if from_bits == into_bits {
                    Ok(()) // 其他情况由具体实现验证大小匹配
                } else {
                    Err(ValueCheckError::CastUnmatch(inst, opcode, fromty, intoty))
                }
            }

            // 整数到指针转换
            Opcode::IntToPtr => {
                let ValTypeID::Int(_) = fromty else {
                    return Err(ValueCheckError::OpTypeNotClass(
                        inst,
                        UseKind::CastOpFrom,
                        fromty,
                        ValTypeClass::Int,
                    ));
                };
                let ValTypeID::Ptr = intoty else {
                    return Err(ValueCheckError::InstTypeNotClass(
                        inst,
                        intoty,
                        ValTypeClass::Ptr,
                    ));
                };
                Ok(()) // 任意整数位宽到指针都支持
            }

            // 指针到整数转换
            Opcode::PtrToInt => {
                let ValTypeID::Ptr = fromty else {
                    return Err(ValueCheckError::OpTypeNotClass(
                        inst,
                        UseKind::CastOpFrom,
                        fromty,
                        ValTypeClass::Ptr,
                    ));
                };
                let ValTypeID::Int(_) = intoty else {
                    return Err(ValueCheckError::InstTypeNotClass(
                        inst,
                        intoty,
                        ValTypeClass::Int,
                    ));
                };
                Ok(()) // 指针到任意整数位宽都支持
            }

            _ => Err(ValueCheckError::FalseOpcodeKind(InstKind::Cast, opcode)),
        }
    }

    /// 检查 Phi 指令的操作数与前驱块匹配规则
    ///
    /// ## Phi 指令规则总结：
    ///
    /// ### SSA 形式约束：
    /// 1. **前驱块完整性**：Phi 指令的输入块集合必须与父基本块的前驱块集合完全匹配
    ///    - 每个前驱块必须提供恰好一个输入值
    ///    - 不能有来自非前驱块的输入
    ///    - 不能遗漏任何前驱块的输入
    ///
    /// 2. **类型一致性**：所有输入值的类型必须与 Phi 指令的返回值类型匹配
    ///    - 确保 Phi 节点在所有执行路径上都产生相同类型的值
    ///    - 符合 SSA 形式的类型安全要求
    ///
    /// ### 操作数规则：
    /// 1. **输入值操作数**：每个输入值都必须非空且类型正确
    ///    - 对应各个前驱块在该执行路径上的值
    ///    - 类型必须与 Phi 指令返回值类型匹配
    ///
    /// 2. **输入块操作数**：每个输入块都必须是有效的基本块引用
    ///    - 必须是 `ValueSSA::Block` 类型
    ///    - 必须是父基本块的实际前驱块
    ///
    /// ### 完整性约束：
    /// 1. **一对一映射**：前驱块与 Phi 输入之间必须是一对一的完整映射
    /// 2. **无遗漏**：所有前驱块都必须在 Phi 指令中有对应的输入
    /// 3. **无冗余**：Phi 指令不能包含来自非前驱块的输入
    ///
    /// ## 检查流程：
    /// 1. 获取父基本块的所有前驱块集合
    /// 2. 遍历 Phi 指令的所有输入，验证：
    ///    - 输入值非空且类型匹配
    ///    - 输入块非空且为有效块引用
    /// 3. 构建 Phi 输入块集合
    /// 4. 验证前驱块集合与 Phi 输入块集合完全相等
    ///
    /// ## 设计说明：
    /// 此实现采用严格的集合相等性检查，确保 SSA 形式的完整性。
    /// 在某些高级优化场景下，可能需要考虑死代码和不可达块的特殊处理。
    pub fn check_phi(&self, phi: &PhiNode) -> Result<(), ValueCheckError> {
        let phiref = phi.get_self_ref();
        let phity = phi.get_valtype();
        let parent_bb = phiref.get_parent(self.allocs());
        let allocs = self.allocs();
        let block_preds = parent_bb
            .preds(self.allocs())
            .iter()
            .map(|jt| jt.get_terminator_inst().get_parent(allocs));
        let block_preds = BTreeSet::from_iter(block_preds);
        let phi_preds = {
            let incomes = phi.incoming_uses();
            let mut phi_preds = BTreeSet::new();
            for [val, blk] in incomes.iter() {
                let val = Self::operand_nonnull(val.get_operand(), phiref, val.kind.get())?;
                type_matches(phity, val, allocs)?;
                let blk = Self::operand_nonnull(blk.get_operand(), phiref, blk.kind.get())?;
                let ValueSSA::Block(blk) = blk else {
                    return Err(ValueCheckError::ValueNotClass(blk, ValueSSAClass::Block));
                };
                phi_preds.insert(blk);
            }
            phi_preds
        };
        if block_preds == phi_preds {
            Ok(())
        } else {
            Err(ValueCheckError::PhiIncomeSetUnmatch(
                phiref,
                block_preds,
                phi_preds,
            ))
        }
    }

    pub fn check_cmpop(&self, cmp: &CmpOp) -> Result<(), ValueCheckError> {
        let cmpref = cmp.get_self_ref();

        let lhs = Self::operand_nonnull(cmp.get_lhs(), cmpref, UseKind::CmpLhs)?;
        let rhs = Self::operand_nonnull(cmp.get_rhs(), cmpref, UseKind::CmpRhs)?;

        let operandty = {
            let lhsty = lhs.get_valtype(self.allocs());
            let rhsty = rhs.get_valtype(self.allocs());
            if lhsty != rhsty {
                return Err(ValueCheckError::OpTypeMismatch(
                    cmpref,
                    UseKind::CmpRhs,
                    lhsty,
                    rhsty,
                ));
            }
            lhsty
        };
        match (operandty, cmp.get_opcode()) {
            (ValTypeID::Int(_), Opcode::Icmp) => Ok(()),
            (ValTypeID::Float(_), Opcode::Fcmp) => Ok(()),
            _ => Err(ValueCheckError::CmpOpcodeErr(
                cmpref,
                cmp.get_opcode(),
                operandty,
            )),
        }
    }

    /// 检查 GEP (GetElementPtr) 指令的操作数与索引类型规则
    ///
    /// ## GEP 指令规则总结：
    ///
    /// ### 基础指针规则：
    /// 1. **基础操作数**：必须非空且为指针类型 (`ValTypeID::Ptr`)
    ///    - GEP 指令总是对指针进行操作
    ///    - 基础指针指向的类型必须与 `first_unpacked_ty` 匹配
    ///
    /// ### 索引操作数规则：
    /// 1. **索引类型约束**：所有索引操作数都必须是整数类型
    ///    - 任意位宽的整数类型 `ValTypeID::Int(_)` 都支持
    ///    - 不允许浮点、指针或其他类型作为索引
    ///    - 索引统一按有符号整数处理
    ///
    /// 2. **索引非空性**：每个索引操作数都必须非空
    ///    - 对应 `UseKind::GepIndex(index)`
    ///    - 确保所有索引位置都有有效的操作数
    ///
    /// ### 索引链规则：
    /// 1. **类型链一致性**：索引操作序列必须能产生有效的类型转换链
    ///    - 从 `first_unpacked_ty` 开始，通过索引序列到达 `last_unpacked_ty`
    ///    - 每一步索引操作都必须在当前类型上合法
    ///
    /// 2. **结构体索引约束**：
    ///    - 结构体索引必须是编译时常量
    ///    - 索引值必须在字段范围内（0 <= index < fields_count）
    ///    - 对应 struct 字段的正确访问
    ///
    /// 3. **数组索引约束**：
    ///    - 数组索引可以是运行时值（不要求常量）
    ///    - 允许越界索引（运行时行为，符合 C 语义）
    ///    - 无限长数组的第一个索引通常为 0（指针解引用）
    ///
    /// ### 完整性约束：
    /// 1. **类型计算一致性**：通过 `compute_result_type` 计算的最终类型必须与声明的 `last_unpacked_ty` 匹配
    /// 2. **索引序列完整性**：必须有足够的索引来完成从基础类型到目标类型的转换
    /// 3. **无过度索引**：不允许对基础类型（如 i32、f64）继续索引
    ///
    /// ## 检查流程：
    /// 1. 验证基础指针非空且为指针类型
    /// 2. 遍历所有索引操作数，检查：
    ///    - 索引非空性
    ///    - 索引类型为整数
    /// 3. 验证索引链的类型转换正确性
    /// 4. 确保最终类型与声明类型匹配
    ///
    /// ## 错误处理：
    /// - 操作数为空：`ValueCheckError::OperandPosNone`
    /// - 类型不匹配：`ValueCheckError::TypeMismatch` 或 `ValueCheckError::TypeNotClass`
    /// - 无效值：`ValueCheckError::InvalidValue`
    pub fn check_gep(&self, gep: &IndexPtr) -> Result<(), ValueCheckError> {
        let gepref = gep.get_self_ref();
        let allocs = self.allocs();

        // 1. 检查基础指针操作数
        let base_ptr = Self::operand_nonnull(gep.get_base(), gepref, UseKind::GepBase)?;
        type_matches(ValTypeID::Ptr, base_ptr, allocs)?;

        // 2. 检查所有索引操作数
        for (index, index_use) in gep.index_uses().iter().enumerate() {
            // 检查索引操作数非空
            let index_val = Self::operand_nonnull(
                index_use.get_operand(),
                gepref,
                UseKind::GepIndex(index as u32),
            )?;

            // 检查索引类型为整数
            optype_isclass(
                gepref,
                UseKind::GepIndex(index as u32),
                ValTypeClass::Int,
                index_val,
                allocs,
            )?;
        }

        // 3. 验证索引链的类型转换正确性
        // 使用 IndexPtr 的内置检查方法来验证索引链和最终类型
        gep.check(self.type_ctx, allocs).map_err(|err_msg| {
            // 将字符串错误转换为 ValueCheckError
            // 这里我们使用一个通用的错误类型来表示 GEP 特定的错误
            ValueCheckError::InvalidValue(ValueSSA::Inst(gepref), err_msg)
        })?;

        Ok(())
    }

    pub fn check_select(&self, select: &SelectOp) -> Result<(), ValueCheckError> {
        let selref = select.get_self_ref();

        let cond = Self::operand_nonnull(select.get_cond(), selref, UseKind::SelectCond)?;
        type_matches(ValTypeID::Int(1), cond, self.allocs())?;

        let lhs = Self::operand_nonnull(select.get_true_val(), selref, UseKind::SelectTrue)?;
        let rhs = Self::operand_nonnull(select.get_false_val(), selref, UseKind::SelectFalse)?;

        let selectty = select.get_valtype();
        type_matches(selectty, lhs, self.allocs())?;
        type_matches(selectty, rhs, self.allocs())?;

        Ok(())
    }

    pub fn check_load(&self, load: &LoadOp) -> Result<(), ValueCheckError> {
        let loadref = load.get_self_ref();
        let allocs = self.allocs();
        let ptr = Self::operand_nonnull(load.get_source(), loadref, UseKind::LoadSource)?;
        type_matches(ValTypeID::Ptr, ptr, allocs)
    }

    pub fn check_store(&self, store: &StoreOp) -> Result<(), ValueCheckError> {
        let storeref = store.get_self_ref();
        let allocs = self.allocs();
        let ptr = Self::operand_nonnull(store.get_target(), storeref, UseKind::StoreTarget)?;
        type_matches(ValTypeID::Ptr, ptr, allocs)?;

        let val = Self::operand_nonnull(store.get_source(), storeref, UseKind::StoreSource)?;
        let source_ty = store.source_ty;
        type_matches(source_ty, val, allocs)
    }

    pub fn check_amo_rmw(&self, amo: &AmoRmw) -> Result<(), ValueCheckError> {
        let amoref = amo.get_self_ref();
        let allocs = self.allocs();
        let ptr = Self::operand_nonnull(amo.get_pointer(), amoref, UseKind::AmoRmwPtr)?;
        type_matches(ValTypeID::Ptr, ptr, allocs)?;

        let val = Self::operand_nonnull(amo.get_value(), amoref, UseKind::AmoRmwVal)?;
        let val_ty = amo.get_valtype();
        type_matches(val_ty, val, allocs)
    }

    pub(crate) fn operand_nonnull(
        operand: ValueSSA,
        inst_ref: InstRef,
        use_kind: UseKind,
    ) -> Result<ValueSSA, ValueCheckError> {
        if operand.is_null() {
            Err(ValueCheckError::OperandPosNone(inst_ref, use_kind))
        } else {
            Ok(operand)
        }
    }
    pub(crate) fn jump_target_nonnull(
        target: BlockRef,
        inst_ref: InstRef,
        jt_kind: JumpTargetKind,
    ) -> Result<(), ValueCheckError> {
        if target.is_null() {
            Err(ValueCheckError::JumpTargetNone(inst_ref, jt_kind))
        } else {
            Ok(())
        }
    }
}

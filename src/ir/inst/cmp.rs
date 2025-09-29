use crate::{
    ir::{
        CmpCond, IRAllocs, IRAllocsReadable, IRWriter, ISubInst, ISubValueSSA, IUser, InstCommon,
        InstData, InstKind, InstRef, Opcode, OperandSet, Use, UseKind, ValueSSA, inst::ISubInstRef,
    },
    typing::ValTypeID,
};
use std::rc::Rc;

/// 比较指令
///
/// 执行两个操作数的比较运算，根据比较条件返回布尔值结果。
/// 支持整数、浮点数等类型的各种比较操作（相等、大于、小于等）。
///
/// ### LLVM IR 语法
///
/// ```llvm
/// %<result> = <op> <cond> <type> <lhs>, <rhs>
/// ```
///
/// ### 操作数布局
/// - `operands[0]`: 左操作数 (LHS)
/// - `operands[1]`: 右操作数 (RHS)
///
/// ### 返回类型
/// 固定返回布尔类型 (`ValTypeID::new_boolean()`)
#[derive(Debug)]
pub struct CmpOp {
    common: InstCommon,
    operands: [Rc<Use>; 2],
    /// 比较条件
    pub cond: CmpCond,
    /// 比较对象类型
    pub operand_ty: ValTypeID,
}

impl IUser for CmpOp {
    fn get_operands(&self) -> OperandSet<'_> {
        OperandSet::Fixed(&self.operands)
    }
    fn operands_mut(&mut self) -> &mut [Rc<Use>] {
        &mut self.operands
    }
}

impl ISubInst for CmpOp {
    fn new_empty(opcode: Opcode) -> Self {
        if opcode.get_kind() != InstKind::Cmp {
            panic!("Tried to create a CmpOp with non-Cmp opcode");
        }
        Self {
            common: InstCommon::new(opcode, ValTypeID::new_boolean()),
            operands: [Use::new(UseKind::CmpLhs), Use::new(UseKind::CmpRhs)],
            cond: CmpCond::NEVER,
            operand_ty: ValTypeID::Void, // 初始类型为 Void
        }
    }
    fn try_from_ir(inst: &InstData) -> Option<&Self> {
        if let InstData::Cmp(cmp) = inst { Some(cmp) } else { None }
    }
    fn try_from_ir_mut(inst: &mut InstData) -> Option<&mut Self> {
        if let InstData::Cmp(cmp) = inst { Some(cmp) } else { None }
    }
    fn into_ir(self) -> InstData {
        InstData::Cmp(self)
    }
    fn get_common(&self) -> &InstCommon {
        &self.common
    }
    fn common_mut(&mut self) -> &mut InstCommon {
        &mut self.common
    }
    fn is_terminator(&self) -> bool {
        false
    }

    fn fmt_ir(&self, id: Option<usize>, writer: &IRWriter) -> std::io::Result<()> {
        let Some(id) = id else {
            use std::io::{Error, ErrorKind::InvalidInput};
            return Err(Error::new(InvalidInput, "ID must be provided for CmpOp"));
        };
        let opcode = self.get_opcode().get_name();
        let cond = self.cond;
        write!(writer, "%{id} = {opcode} {cond} ")?;
        writer.write_type(self.operand_ty)?;
        writer.write_str(" ")?;
        writer.write_operand(self.get_lhs())?;
        writer.write_str(", ")?;
        writer.write_operand(self.get_rhs())
    }
}

impl CmpOp {
    pub const OP_LHS: usize = 0;
    pub const OP_RHS: usize = 1;

    /// 创建一个未初始化操作数的比较指令
    ///
    /// # 参数
    /// - `opcode`: 指令操作码，必须是比较类型
    /// - `cond`: 比较条件
    ///
    /// # Panics
    /// 如果 opcode 不是比较指令类型则 panic
    pub fn new_raw(opcode: Opcode, cond: CmpCond, ty: ValTypeID) -> Self {
        if opcode.get_kind() != InstKind::Cmp {
            panic!("Tried to create a CmpOp with non-Cmp opcode");
        }
        let cond = match opcode {
            Opcode::Icmp => cond.switch_to_int(),
            Opcode::Fcmp => cond.switch_to_float(),
            _ => panic!("Unsupported opcode for comparison: {opcode:?}"),
        };
        Self {
            common: InstCommon::new(opcode, ValTypeID::new_boolean()),
            operands: [Use::new(UseKind::CmpLhs), Use::new(UseKind::CmpRhs)],
            cond,
            operand_ty: ty,
        }
    }

    /// 创建一个完全初始化的比较指令
    ///
    /// # 参数
    /// - `allocs`: IR 分配器，用于建立 Use-Def 关系
    /// - `opcode`: 指令操作码，必须是比较类型
    /// - `cond`: 比较条件
    /// - `lhs`: 左操作数
    /// - `rhs`: 右操作数
    ///
    /// # Panics
    /// 如果 opcode 不是比较指令类型则 panic
    pub fn new(allocs: &IRAllocs, cond: CmpCond, lhs: ValueSSA, rhs: ValueSSA) -> Self {
        let ty: ValTypeID = {
            let lty = lhs.get_valtype(allocs);
            let rty = rhs.get_valtype(allocs);
            assert_eq!(
                lty, rty,
                "Left and right operands must have the same type for comparison"
            );
            lty
        };
        let opcode = match ty {
            ValTypeID::Ptr | ValTypeID::Int(_) => Opcode::Icmp,
            ValTypeID::Float(_) => Opcode::Fcmp,
            _ => panic!("Unsupported type for comparison: {ty:?}"),
        };
        let cmp = Self::new_raw(opcode, cond, ty);
        cmp.set_lhs(allocs, lhs);
        cmp.set_rhs(allocs, rhs);
        cmp
    }

    /// 获取左操作数的 Use 引用
    ///
    /// # 返回
    /// 左操作数的 Use 对象引用，用于 Use-Def 链分析
    pub fn lhs_use(&self) -> &Rc<Use> {
        &self.operands[0]
    }

    /// 获取右操作数的 Use 引用
    ///
    /// # 返回
    /// 右操作数的 Use 对象引用，用于 Use-Def 链分析
    pub fn rhs_use(&self) -> &Rc<Use> {
        &self.operands[1]
    }

    /// 获取左操作数的值
    ///
    /// # 返回
    /// 左操作数的 SSA 值
    pub fn get_lhs(&self) -> ValueSSA {
        self.lhs_use().get_operand()
    }

    /// 设置左操作数的值
    ///
    /// # 参数
    /// - `allocs`: IR 分配器，用于维护 Use-Def 关系
    /// - `value`: 新的左操作数值
    pub fn set_lhs(&self, allocs: &IRAllocs, value: ValueSSA) {
        self.lhs_use().set_operand(allocs, value);
    }

    /// 获取右操作数的值
    ///
    /// # 返回
    /// 右操作数的 SSA 值
    pub fn get_rhs(&self) -> ValueSSA {
        self.rhs_use().get_operand()
    }

    /// 设置右操作数的值
    ///
    /// # 参数
    /// - `allocs`: IR 分配器，用于维护 Use-Def 关系
    /// - `value`: 新的右操作数值
    pub fn set_rhs(&self, allocs: &IRAllocs, value: ValueSSA) {
        self.rhs_use().set_operand(allocs, value);
    }
}

/// 比较指令的强类型引用
///
/// 包装 `InstRef` 提供类型安全的比较指令引用，
/// 确保引用指向的确实是比较指令而非其他类型的指令。
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct CmpOpRef(InstRef);

impl ISubInstRef for CmpOpRef {
    type InstDataT = CmpOp;

    fn from_raw_nocheck(inst_ref: InstRef) -> Self {
        CmpOpRef(inst_ref)
    }
    fn into_raw(self) -> InstRef {
        self.0
    }
}

impl CmpOpRef {
    pub fn get_cond(self, allocs: &impl IRAllocsReadable) -> CmpCond {
        self.to_inst(&allocs.get_allocs_ref().insts).cond
    }

    pub fn get_lhs(self, allocs: &impl IRAllocsReadable) -> ValueSSA {
        self.to_inst(&allocs.get_allocs_ref().insts).get_lhs()
    }
    pub fn set_lhs(self, allocs: &impl IRAllocsReadable, lhs: ValueSSA) {
        self.to_inst(&allocs.get_allocs_ref().insts)
            .set_lhs(allocs.get_allocs_ref(), lhs);
    }

    pub fn get_rhs(self, allocs: &impl IRAllocsReadable) -> ValueSSA {
        self.to_inst(&allocs.get_allocs_ref().insts).get_rhs()
    }
    pub fn set_rhs(self, allocs: &impl IRAllocsReadable, rhs: ValueSSA) {
        self.to_inst(&allocs.get_allocs_ref().insts)
            .set_rhs(allocs.get_allocs_ref(), rhs);
    }
}

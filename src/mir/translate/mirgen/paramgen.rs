use std::fmt::{Debug, Formatter};

use crate::{
    mir::{
        module::vreg_alloc::VirtRegAlloc,
        operand::{
            IMirSubOperand,
            reg::{FPR32, FPR64, GPR32, GPR64},
        },
        translate::mirgen::operandgen::DispatchedReg,
    },
    typing::{FPKind, FuncTypeRef, IValType, PrimType, TypeContext},
};

#[derive(Clone, Copy)]
pub enum ArgPos {
    Reg(DispatchedReg),
    Stack(u32, DispatchedReg, GPR64),
}

impl Debug for ArgPos {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        match self {
            ArgPos::Reg(dreg) => write!(f, "Ref:{dreg:?}"),
            ArgPos::Stack(offset, vreg, gpr64) => {
                write!(f, "Stack:{gpr64:?} = sp + {offset} -> &{vreg:?}")
            }
        }
    }
}

#[derive(Clone)]
pub struct MirArgInfo {
    pub pos: Vec<(PrimType, ArgPos)>,
    pub arg_regs: Vec<(u32, PrimType, DispatchedReg)>,
    pub stack_size: usize,
}

impl Debug for MirArgInfo {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        #[derive(Clone, Copy)]
        struct ArgPosFmt(PrimType, ArgPos);
        impl Debug for ArgPosFmt {
            fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
                let Self(ty, pos) = *self;
                write!(f, "{ty:?} => {pos:?}")
            }
        }

        let pos = {
            let mut pos = Vec::with_capacity(self.pos.len());
            for (ty, arg_pos) in &self.pos {
                pos.push(ArgPosFmt(*ty, arg_pos.clone()));
            }
            pos
        };

        assert!(self.stack_size % 16 == 0, "Stack size must be aligned to 16 bytes");

        f.debug_struct("MirArgInfo")
            .field("stack_size", &self.stack_size)
            .field("pos", &pos)
            .finish()
    }
}

pub struct MirArgBuilder {
    pub pos: Vec<(PrimType, ArgPos)>,
    pub stack_top: u64,
    pub fp_count: usize,
    pub gp_count: usize,
}

impl MirArgBuilder {
    pub fn new() -> Self {
        Self { pos: Vec::new(), stack_top: 0, fp_count: 0, gp_count: 0 }
    }

    pub fn push_gpr32(&mut self) -> &mut Self {
        let i32ty = PrimType::Int(32);
        let gpid = self.gp_count;
        let dreg = DispatchedReg::G32(GPR32::new_raw(gpid as u32));
        if gpid < 8 {
            self.pos.push((i32ty, ArgPos::Reg(dreg)));
        } else {
            self.stack_top = self.stack_top.next_multiple_of(8);
            self.pos.push((
                i32ty,
                ArgPos::Stack(self.stack_top as u32, dreg, GPR64::new_empty()),
            ));
            self.stack_top += 4;
        }
        self.gp_count += 1;
        self
    }

    pub fn push_gpr64(&mut self, ty: PrimType) -> &mut Self {
        let gpid = self.gp_count;
        let dreg = DispatchedReg::G64(GPR64::new_raw(gpid as u32));
        if gpid < 8 {
            self.pos.push((ty, ArgPos::Reg(dreg)));
        } else {
            self.stack_top = self.stack_top.next_multiple_of(8);
            self.pos.push((
                ty,
                ArgPos::Stack(self.stack_top as u32, dreg, GPR64::new_empty()),
            ));
            self.stack_top += 8;
        }
        self.gp_count += 1;
        self
    }

    pub fn push_fp32(&mut self) -> &mut Self {
        let fpid = self.fp_count;
        let dreg = DispatchedReg::F32(FPR32::new_raw(fpid as u32));
        if fpid < 8 {
            self.pos
                .push((PrimType::Float(FPKind::Ieee32), ArgPos::Reg(dreg)));
        } else {
            self.stack_top = self.stack_top.next_multiple_of(8);
            self.pos.push((
                PrimType::Float(FPKind::Ieee32),
                ArgPos::Stack(self.stack_top as u32, dreg, GPR64::new_empty()),
            ));
            self.stack_top += 8;
        }
        self.fp_count += 1;
        self
    }

    pub fn push_fp64(&mut self) -> &mut Self {
        let fpid = self.fp_count;
        let dreg = DispatchedReg::F64(FPR64::new_raw(fpid as u32));
        if fpid < 8 {
            self.pos
                .push((PrimType::Float(FPKind::Ieee64), ArgPos::Reg(dreg)));
        } else {
            self.stack_top = self.stack_top.next_multiple_of(8);
            self.pos.push((
                PrimType::Float(FPKind::Ieee64),
                ArgPos::Stack(self.stack_top as u32, dreg, GPR64::new_empty()),
            ));
            self.stack_top += 8;
        }
        self.fp_count += 1;
        self
    }

    pub fn push(&mut self, ty: PrimType) -> &mut Self {
        match ty {
            PrimType::Int(32) => self.push_gpr32(),
            PrimType::Int(64) | PrimType::Ptr => self.push_gpr64(ty),
            PrimType::Float(FPKind::Ieee32) => self.push_fp32(),
            PrimType::Float(FPKind::Ieee64) => self.push_fp64(),
            _ => panic!("Unsupported type"),
        }
    }

    pub fn finish(mut self, vreg_alloc: &mut VirtRegAlloc) -> MirArgInfo {
        for (_, pos) in &mut self.pos {
            let ArgPos::Stack(_, vreg, gpr64) = pos else {
                continue;
            };
            *gpr64 = vreg_alloc.alloc_stackpos();
            *vreg = match *vreg {
                DispatchedReg::F32(fpr32) => DispatchedReg::F32(vreg_alloc.insert_fpr32(fpr32)),
                DispatchedReg::F64(fpr64) => DispatchedReg::F64(vreg_alloc.insert_fpr64(fpr64)),
                DispatchedReg::G32(gpr32) => DispatchedReg::G32(vreg_alloc.insert_gpr32(gpr32)),
                DispatchedReg::G64(gpr64) => DispatchedReg::G64(vreg_alloc.insert_gpr64(gpr64)),
            }
        }

        let mut arg_regs = Vec::new();
        for (id, &(ty, pos)) in self.pos.iter().enumerate() {
            let ArgPos::Reg(dreg) = pos else {
                continue;
            };
            arg_regs.push((id as u32, ty, dreg));
        }
        // Ensure stack size is aligned to 16 bytes
        let stack_size = self.stack_top.next_multiple_of(16) as usize;
        MirArgInfo { pos: self.pos, arg_regs, stack_size }
    }

    pub fn build_func(
        mut self,
        func_ty: FuncTypeRef,
        type_ctx: &TypeContext,
        vreg_alloc: &mut VirtRegAlloc,
    ) -> MirArgInfo {
        for &arg in &*func_ty.args(type_ctx) {
            let arg_ty = PrimType::from_ir(arg);
            self.push(arg_ty);
        }
        self.finish(vreg_alloc)
    }
}

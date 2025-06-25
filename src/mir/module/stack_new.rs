use crate::{
    mir::operand::reg::VirtReg,
    typing::{id::ValTypeID, types::FloatTypeKind},
};

#[derive(Debug, Clone)]
pub struct MirStackItem {
    pub irtype: ValTypeID,
    pub index: usize,
    pub virtreg: VirtReg,
    pub offset: i64,
    pub size: u64,
    pub size_with_padding: u64,
    pub align_log2: u8,
}

#[derive(Debug, Clone)]
pub struct MirStackLayout {
    pub vars: Vec<MirStackItem>,
    pub args: Vec<MirStackItem>,
    pub vars_size: u64,
    pub args_size: u64,
    finished_arg_build: bool,
}

#[derive(Debug, Clone)]
struct TopInfo {
    pos: u64,
    top: u64,
}

impl MirStackLayout {
    pub fn new() -> Self {
        Self {
            vars: Vec::new(),
            args: Vec::new(),
            vars_size: 0,
            args_size: 0,
            finished_arg_build: false,
        }
    }

    fn update_stack_top(curr_size: u64, align_log2: u8) -> TopInfo {
        let next_align = 1u64 << align_log2;
        let pmask = next_align - 1;
        let nmask = !pmask;
        let new_size_base = curr_size & nmask;
        let new_size = if curr_size & pmask != 0 {
            new_size_base + next_align
        } else {
            new_size_base
        };
        TopInfo {
            pos: curr_size,
            top: new_size,
        }
    }

    pub fn add_arg(&mut self, irtype: ValTypeID, vreg_alloc: VirtReg) -> &MirStackItem {
        assert!(
            self.finished_arg_build == false,
            "Cannot add more args after building the stack layout"
        );
        let (is_float, natural_size, natural_align_log2) = match irtype {
            ValTypeID::Ptr => (false, 8, 3),
            ValTypeID::Int(bits) => match bits {
                8 => (false, 1, 0),
                16 => (false, 2, 1),
                32 => (false, 4, 2),
                64 => (false, 8, 3),
                _ => panic!("Unsupported integer size: {}", bits),
            },
            ValTypeID::Float(fp_kind) => match fp_kind {
                FloatTypeKind::Ieee32 => (true, 4, 2),
                FloatTypeKind::Ieee64 => (true, 8, 3),
            },
            _ => panic!("Requires ptr/int/float as args but got `{irtype:?}`"),
        };

        let size = natural_size.max(8);
        let align_log2 = natural_align_log2.max(3);

        let top_info = Self::update_stack_top(self.args_size, align_log2);
    }
}

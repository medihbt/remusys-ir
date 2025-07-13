use std::{
    cell::{Cell, RefCell},
    fmt::Write,
    ops::Range,
    rc::Rc,
};

use crate::mir::{
    fmt::FuncFormatContext,
    inst::{IMirSubInst, MirInstCommon, opcode::MirOP},
    module::{block::MirBlockRef, func::MirFunc},
    operand::{IMirSubOperand, MirOperand},
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum TabSwitchCaseRange {
    /// (begin..end, step)
    Signed(Range<i64>, u64),
    /// (begin..end, step)
    Unsigned(Range<u64>, u64),
}

impl TabSwitchCaseRange {
    pub fn get_begin(&self) -> i128 {
        match self {
            TabSwitchCaseRange::Signed(range, _) => range.start as i128,
            TabSwitchCaseRange::Unsigned(range, _) => range.start as i128,
        }
    }
    pub fn get_end(&self) -> i128 {
        match self {
            TabSwitchCaseRange::Signed(range, _) => range.end as i128,
            TabSwitchCaseRange::Unsigned(range, _) => range.end as i128,
        }
    }
    pub fn get_step(&self) -> u64 {
        match self {
            TabSwitchCaseRange::Signed(_, step) => *step,
            TabSwitchCaseRange::Unsigned(_, step) => *step,
        }
    }

    fn do_index_to_case_value(&self, index: u64) -> i128 {
        match self {
            TabSwitchCaseRange::Signed(range, step) => {
                range.start as i128 + (index * *step as u64) as i128
            }
            TabSwitchCaseRange::Unsigned(range, step) => {
                range.start as i128 + (index * *step as u64) as i128
            }
        }
    }
    fn do_case_value_to_index(&self, value: i128) -> Option<u64> {
        match self {
            TabSwitchCaseRange::Signed(range, step) => {
                if value < range.start as i128 || value >= range.end as i128 {
                    None
                } else {
                    Some(((value - range.start as i128) / *step as i128) as u64)
                }
            }
            TabSwitchCaseRange::Unsigned(range, step) => {
                if value < range.start as i128 || value >= range.end as i128 {
                    None
                } else {
                    Some(((value - range.start as i128) / *step as i128) as u64)
                }
            }
        }
    }

    pub fn index_to_case<T: Into<u64>>(&self, index: T) -> i128 {
        self.do_index_to_case_value(index.into())
    }
    pub fn case_to_index<T: Into<i128>>(&self, value: T) -> Option<u64> {
        self.do_case_value_to_index(value.into())
    }
    pub fn has_case<T: Into<i128>>(&self, value: T) -> bool {
        self.do_case_value_to_index(value.into()).is_some()
    }

    pub fn num_cases(&self) -> u64 {
        let begin = self.get_begin();
        let end = self.get_end();
        let step = self.get_step();
        if step == 0 {
            return 0;
        }
        let delta = end - begin;
        if delta < 0 {
            return 0;
        }
        if delta % step as i128 != 0 {
            return (delta / step as i128) as u64 + 1;
        }
        (delta / step as i128) as u64
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct VecSwitchTab {
    pub left: i128,
    pub step: u64,
    pub cases: Box<[Cell<MirBlockRef>]>,
    pub tab_index: Cell<u32>,
}

impl VecSwitchTab {
    pub fn new(left: i128, step: u64, cases: impl Iterator<Item = MirBlockRef>) -> Self {
        Self {
            left,
            step,
            cases: cases.map(Cell::new).collect(),
            tab_index: Cell::new(0),
        }
    }

    pub fn ncases(&self) -> u64 {
        self.cases.len() as u64
    }

    pub fn get_case_range(&self) -> TabSwitchCaseRange {
        if self.left < i64::MIN as i128 || self.left > u64::MAX as i128 {
            panic!(
                "Left value out of bounds for TabSwitchCaseRange: {}",
                self.left
            );
        }
        if self.left < 0 {
            TabSwitchCaseRange::Signed(
                self.left as i64..(self.left as i64 + self.step as i64 * self.ncases() as i64),
                self.step,
            )
        } else {
            TabSwitchCaseRange::Unsigned(
                self.left as u64..(self.left as u64 + self.step * self.ncases()),
                self.step,
            )
        }
    }

    pub fn get_case_by_index(&self, index: u64) -> Option<MirBlockRef> {
        if index < self.ncases() {
            Some(self.cases[index as usize].get())
        } else {
            None
        }
    }
    pub fn get_case_by_value<T: Into<i128>>(&self, value: T) -> Option<MirBlockRef> {
        let value = value.into();
        if let Some(index) = self.get_case_range().case_to_index(value) {
            self.get_case_by_index(index)
        } else {
            None
        }
    }

    pub fn get_name(&self, mir_func: &MirFunc) -> String {
        let func_name = mir_func.get_name();
        let self_index = self.tab_index.get();
        format!(".switch.{func_name}.{self_index:x}",)
    }
}

#[derive(Debug, Clone)]
pub struct MirSwitch {
    _common: MirInstCommon,
    _operands: [Cell<MirOperand>; 3],
    switch_tab: RefCell<Rc<VecSwitchTab>>,
}

impl IMirSubInst for MirSwitch {
    fn get_common(&self) -> &MirInstCommon {
        &self._common
    }
    fn out_operands(&self) -> &[Cell<MirOperand>] {
        &[]
    }
    fn in_operands(&self) -> &[Cell<MirOperand>] {
        &self._operands
    }
    fn accepts_opcode(opcode: MirOP) -> bool {
        matches!(opcode, MirOP::MirSwitch)
    }
    fn new_empty(opcode: MirOP) -> Self {
        MirSwitch {
            _common: MirInstCommon::new(opcode),
            _operands: [
                Cell::new(MirOperand::None), // Switch value
                Cell::new(MirOperand::None), // Default case
                Cell::new(MirOperand::None), // Switch table
            ],
            switch_tab: RefCell::new(Rc::new(VecSwitchTab {
                left: 0,
                step: 0,
                cases: Box::new([]),
                tab_index: Cell::new(0),
            })),
        }
    }
}

impl MirSwitch {
    pub fn new(
        switch_value: MirOperand,
        default_case: MirBlockRef,
        switch_tab: Rc<VecSwitchTab>,
    ) -> Self {
        let mut inst = Self::new_empty(MirOP::MirSwitch);
        inst._operands[0].set(switch_value);
        inst._operands[1].set(MirOperand::Label(default_case));
        inst._operands[2].set(MirOperand::SwitchTab(switch_tab.tab_index.get()));
        inst.switch_tab = RefCell::new(switch_tab);
        inst
    }

    pub fn condition(&self) -> &Cell<MirOperand> {
        &self._operands[0]
    }
    pub fn default_case(&self) -> &Cell<MirOperand> {
        &self._operands[1]
    }
    pub fn get_default_case(&self) -> MirBlockRef {
        if let MirOperand::Label(block_ref) = self.default_case().get() {
            block_ref
        } else {
            panic!("Default case is not a label");
        }
    }
    pub fn set_default_case(&self, block_ref: MirBlockRef) {
        self.default_case().set(MirOperand::Label(block_ref));
    }

    pub fn get_switch_tab(&self) -> Rc<VecSwitchTab> {
        self.switch_tab.borrow().clone()
    }
    pub fn set_switch_tab(&self, switch_tab: Rc<VecSwitchTab>) {
        self._operands[2].set(MirOperand::SwitchTab(switch_tab.tab_index.get()));
        self.switch_tab.replace(switch_tab);
    }

    pub fn fmt_asm(&self, formatter: &mut FuncFormatContext<'_>) -> std::fmt::Result {
        let switch_value = self.condition().get();
        write!(formatter, "mir.switch ")?;
        switch_value.fmt_asm(formatter)?;
        write!(formatter, ", ")?;
        self.get_default_case().fmt_asm(formatter)?;
        write!(formatter, ", ")?;
        let switch_tab = self.get_switch_tab();
        write!(
            formatter,
            "{}",
            switch_tab.get_name(&formatter.get_current_func())
        )?;
        Ok(())
    }
}

use std::{cell::Cell, ops::Range, rc::Rc};

use crate::{
    base::NullableValue,
    mir::{
        inst::{MirInstCommon, opcode::MirOP},
        module::block::MirBlockRef,
        operand::MirOperand,
    },
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
}

#[derive(Debug, Clone)]
pub struct TabSwitch {
    pub(super) common: MirInstCommon,
    /// [%condition, <default label>, SwitchTab]
    pub operands: [Cell<MirOperand>; 3],
    pub switchtab_ref: Rc<VecSwitchTab>,
}

impl TabSwitch {
    pub fn new(
        condition: MirOperand,
        default_label: MirBlockRef,
        switchtab_ref: Rc<VecSwitchTab>,
    ) -> Self {
        Self {
            common: MirInstCommon::new(MirOP::TabSwitch),
            operands: [
                Cell::new(condition),
                Cell::new(MirOperand::Label(default_label)),
                Cell::new(MirOperand::VecSwitchTab(switchtab_ref.tab_index.get())),
            ],
            switchtab_ref,
        }
    }

    pub fn get_opcode(&self) -> MirOP {
        MirOP::TabSwitch
    }

    pub fn get_condition(&self) -> MirOperand {
        self.operands[0].get()
    }
    pub fn set_condition(&self, condition: MirOperand) {
        self.operands[0].set(condition);
    }
    pub fn get_default_label(&self) -> MirBlockRef {
        if let MirOperand::Label(label) = self.operands[1].get() {
            label
        } else {
            panic!("Expected a label operand for default label");
        }
    }
    pub fn set_default_label(&self, label: MirBlockRef) {
        self.operands[1].set(MirOperand::Label(label));
    }
    pub fn get_switchtab(&self) -> (u32, Rc<VecSwitchTab>) {
        if let MirOperand::VecSwitchTab(index) = self.operands[2].get() {
            (index, self.switchtab_ref.clone())
        } else {
            panic!("Expected a VecSwitchTab operand for switch table index");
        }
    }
    pub fn set_switchtab(&mut self, switch_tab: &Rc<VecSwitchTab>) {
        self.switchtab_ref = switch_tab.clone();
        self.operands[2].set(MirOperand::VecSwitchTab(switch_tab.tab_index.get()));
    }
}

#[derive(Debug, Clone)]
pub struct BinSwitchTab {
    pub cases: Box<[(MirBlockRef, u64)]>,
    pub tab_index: Cell<u32>,
    pub default_label: MirBlockRef,
}

impl BinSwitchTab {
    pub fn new(mut cases: Vec<(MirBlockRef, u64)>) -> Self {
        cases.sort_by(|a, b| a.1.cmp(&b.1));
        Self {
            cases: cases.into_boxed_slice(),
            tab_index: Cell::new(0),
            default_label: MirBlockRef::new_null(),
        }
    }
    pub fn from_iter<I: IntoIterator<Item = (MirBlockRef, u64)>>(cases: I) -> Self {
        Self::new(cases.into_iter().collect())
    }

    pub fn ncases(&self) -> u64 {
        self.cases.len() as u64
    }

    pub fn min_case_value(&self) -> u64 {
        self.cases.first().map_or(0, |(_, value)| *value)
    }
    pub fn max_case_value(&self) -> u64 {
        self.cases.last().map_or(0, |(_, value)| *value)
    }
    pub fn get_label_by_case(&self, case_value: u64) -> MirBlockRef {
        self.cases
            .binary_search_by(|(_, case)| case.cmp(&case_value))
            .map(|index| self.cases[index].0)
            .unwrap_or(self.default_label)
    }
}

#[derive(Debug, Clone)]
pub struct BinSwitch {
    pub(super) common: MirInstCommon,
    /// [%condition, SwitchTab]
    pub operands: [Cell<MirOperand>; 2],
    pub switchtab_ref: Rc<BinSwitchTab>,
}

impl BinSwitch {
    pub fn new(condition: MirOperand, switchtab_ref: Rc<BinSwitchTab>) -> Self {
        Self {
            common: MirInstCommon::new(MirOP::BinSwitch),
            operands: [
                Cell::new(condition),
                Cell::new(MirOperand::BinSwitchTab(switchtab_ref.tab_index.get())),
            ],
            switchtab_ref,
        }
    }

    pub fn get_opcode(&self) -> MirOP {
        MirOP::BinSwitch
    }

    pub fn get_condition(&self) -> MirOperand {
        self.operands[0].get()
    }
    pub fn set_condition(&self, condition: MirOperand) {
        self.operands[0].set(condition);
    }
    pub fn get_switchtab(&self) -> (u32, Rc<BinSwitchTab>) {
        if let MirOperand::BinSwitchTab(index) = self.operands[1].get() {
            (index, self.switchtab_ref.clone())
        } else {
            panic!("Expected a BinSwitchTab operand for switch table index");
        }
    }
    pub fn set_switchtab(&mut self, switch_tab: &Rc<BinSwitchTab>) {
        self.switchtab_ref = switch_tab.clone();
        self.operands[1].set(MirOperand::BinSwitchTab(switch_tab.tab_index.get()));
    }
}

use crate::mir::symbol::func::MachineFunc;

pub enum MachineGlobalData {
    Func(MachineFunc),
}
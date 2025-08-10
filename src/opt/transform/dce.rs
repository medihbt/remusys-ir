//! Dead Code Elimination

use crate::{
    base::SlabRef,
    ir::{ISubGlobal, ITraceableValue, Linkage, Module},
};

/// 使用引用计数法移除模块中未被使用的全局对象. 该方法不会移除循环引用的死亡对象.
pub fn roughly_remove_unused_globals(module: &Module) {
    module.globals.borrow_mut().retain(|_, &mut gref| {
        let allocs = module.borrow_allocs();
        let gdata = gref.to_data(&allocs.globals);
        gdata.get_linkage() == Linkage::DSOLocal || gdata.has_users()
    });
    module.gc_mark_sweep([]);
}

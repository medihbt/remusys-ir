//! Export IR CFG to Graphviz format.
//!
//! Well... Rust doesn't have a good Graphviz library, so we just hardcode the
//! Graphviz format.

use std::collections::BTreeMap;

use crate::{
    base::SlabRef,
    ir::{
        global::{GlobalData, GlobalRef},
        module::Module,
        util::numbering::{IRValueNumberMap, NumberOption},
    },
};

pub fn write_func_cfg(module: &Module, func: GlobalRef, writer: &mut dyn std::io::Write) {
    let func_data = module.get_global(func);
    let func_data = match &*func_data {
        GlobalData::Func(f) => f,
        _ => panic!("Expected a function"),
    };
    let blocks_range = match func_data.get_blocks() {
        Some(b) => b.load_range(),
        None => panic!("Function has no blocks"),
    };
    let value_number = IRValueNumberMap::from_func(module, func, NumberOption::ignore_all());

    writer
        .write_fmt(format_args!("digraph \"{}\" {{\n", func_data.get_name()))
        .unwrap();

    let alloc_value = module.borrow_value_alloc();
    let alloc_block = &alloc_value.alloc_block;
    let mut cfg_edges = Vec::new();
    let mut block_order_map = BTreeMap::new();
    for (order, (block_ref, block)) in blocks_range.view(alloc_block).into_iter().enumerate() {
        let block_id = value_number.block_get_number(block_ref).unwrap();
        block_order_map.insert(block_ref, order);
        writer
            .write_fmt(format_args!(
                "    {} [label=\"%{}({})\" shape=box];\n",
                order,
                block_id,
                block_ref.get_handle()
            ))
            .unwrap();
        let terminator = block.get_terminator_subref(module).unwrap();
        for succ in terminator.collect_jump_blocks_from_module(module) {
            cfg_edges.push((block_ref, succ));
        }
    }

    for (block_ref, succ) in cfg_edges {
        let block_order = block_order_map.get(&block_ref).unwrap();
        let succ_order = block_order_map.get(&succ).unwrap();
        writer
            .write_fmt(format_args!("    {} -> {};\n", block_order, succ_order))
            .unwrap();
    }

    writer.write(b"}\n").unwrap();
}

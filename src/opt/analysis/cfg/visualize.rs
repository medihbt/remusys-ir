//! Export IR CFG to Graphviz format.
//!
//! Well... Rust doesn't have a good Graphviz library, so we just hardcode the
//! Graphviz format.

use std::collections::BTreeMap;

use crate::{
    base::SlabRef,
    ir::{GlobalData, GlobalRef, IRValueNumberMap, ISubGlobal, Module, NumberOption},
};

pub fn write_func_cfg(module: &Module, func: GlobalRef, writer: &mut dyn std::io::Write) {
    let allocs = &module.allocs;
    let GlobalData::Func(func_data) = func.to_data(&allocs.globals) else {
        panic!("Expected a function");
    };
    let blocks_range = match func_data.get_body() {
        Some(b) => b.load_range(),
        None => panic!("Function has no blocks"),
    };
    let value_number = IRValueNumberMap::new(&allocs, func, NumberOption::ignore_all());

    writer
        .write_fmt(format_args!("digraph \"{}\" {{\n", func_data.get_name()))
        .unwrap();

    let mut cfg_edges = Vec::new();
    let mut block_order_map = BTreeMap::new();
    for (order, (block_ref, block)) in blocks_range.view(&allocs.blocks).into_iter().enumerate() {
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
        let terminator = block.get_terminator_from_alloc(&allocs.insts);
        for succ in &terminator.get_jts(&allocs.insts) {
            let succ = succ.get_block();
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

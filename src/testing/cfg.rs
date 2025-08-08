use crate::{
    ir::{IRValueNumberMap, NumberOption},
    opt::analysis::cfg::{
        dominance::DominatorTree, snapshot::CfgSnapshot, visualize::write_func_cfg,
    },
};

use super::cases::{test_case_cfg_deep_while_br, write_ir_to_file};

#[test]
fn test_case_build_dfs_seq() {
    let builder = test_case_cfg_deep_while_br();
    write_ir_to_file(&builder.module, "test_case_build_dfs_seq");

    let mut writer = std::fs::File::create("target/test_case_build_dfs_seq.dot").unwrap();
    write_func_cfg(
        &builder.module,
        builder.get_focus_full().function,
        &mut writer,
    );

    let func = builder.get_focus_full().function;
    let mut module = builder.module;
    let snapshot = CfgSnapshot::new(&module.allocs_mut(), func);
    let number_map = IRValueNumberMap::new(module.allocs_mut(), func, NumberOption::ignore_all());

    let dom_tree = DominatorTree::new_postdom_from_snapshot(&snapshot);
    let mut writer = std::fs::File::create("target/test_case_build_dfs_seq_postdom.dot").unwrap();
    dom_tree.write_to_graphviz(&number_map, &mut writer);
}

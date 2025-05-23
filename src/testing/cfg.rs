use crate::{
    ir::util::numbering::{IRValueNumberMap, NumberOption},
    opt::analysis::cfg::{
        dominance::DominatorTree, snapshot::CfgSnapshot, visualize::write_func_cfg,
    },
};

use super::cases::{test_case_cfg_deep_while_br, write_ir_to_file};

#[test]
fn test_case_build_dfs_seq() {
    let (module, builder) = test_case_cfg_deep_while_br();
    write_ir_to_file(module.as_ref(), "test_case_build_dfs_seq");

    let mut writer = std::fs::File::create("target/test_case_build_dfs_seq.dot").unwrap();
    write_func_cfg(&module, builder.get_focus_full().function, &mut writer);

    let func = builder.get_focus_full().function;
    let snapshot = CfgSnapshot::new_from_func(module.as_ref(), func);
    let number_map = IRValueNumberMap::from_func(
        module.as_ref(),
        builder.get_focus_full().function,
        NumberOption::ignore_all(),
    );

    let dom_tree = DominatorTree::new_postdom_from_snapshot(&snapshot);
    let mut writer = std::fs::File::create("target/test_case_build_dfs_seq_postdom.dot").unwrap();
    dom_tree.write_to_graphviz(&number_map, &mut writer);
}

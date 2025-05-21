use crate::opt::{analysis::cfg::{dfs::CfgDfsSeq, visualize::write_func_cfg}, util::DfsOrder};

use super::cases::{test_case_cfg_deep_while_br, write_ir_to_file};

#[test]
fn test_case_build_dfs_seq() {
    let (module, builder) = test_case_cfg_deep_while_br();
    let dfs_seq = CfgDfsSeq::new_from_func(
        module.as_ref(),
        builder.get_focus_full().function,
        DfsOrder::Pre,
    )
    .unwrap();

    write_ir_to_file(module.as_ref(), "test_case_build_dfs_seq");

    for (i, block) in dfs_seq.nodes.iter().enumerate() {
        println!("Block[{}] = {:?}", i, block);
    }

    let mut writer = std::fs::File::create("target/test_case_build_dfs_seq.dot").unwrap();
    write_func_cfg(&module, builder.get_focus_full().function, &mut writer);
}

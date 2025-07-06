//! Write MIR to assembly code.

use std::{
    cell::{Cell, RefCell},
    rc::Rc,
};

use crate::{
    base::slabref::SlabRef,
    mir::{
        inst::{
            IMirSubInst, MirInst,
            load_store::{AddressMode, ILoadStoreInst, LoadStoreRRI, LoadStoreRRR},
            opcode::MirOP,
            switch::{BinSwitchTab, VecSwitchTab},
        },
        module::{
            MirGlobal, MirModule,
            block::MirBlock,
            func::MirFunc,
            global::{Linkage, MirGlobalData, MirGlobalVariable, Section},
        },
        operand::MirOperand,
    },
};

pub struct AsmWriter<'a> {
    std_writer: RefCell<&'a mut dyn std::io::Write>,
    indent_level: Cell<usize>,
}

struct GlobalWriteStatus {
    last_section: Section,
    last_align_log2: u8,
}

#[derive(Debug, Clone)]
struct InstWriteStatus<'a> {
    parent_func: Rc<MirFunc>,
    module: &'a MirModule,
}

impl<'a> AsmWriter<'a> {
    pub fn new(writer: &'a mut dyn std::io::Write) -> Self {
        Self {
            std_writer: RefCell::new(writer),
            indent_level: Cell::new(0),
        }
    }

    fn wrap_indent(&self) {
        let mut mut_writer = self.std_writer.borrow_mut();
        mut_writer
            .write_all(b"\n")
            .expect("Failed to write newline");
        for _ in 0..self.indent_level.get() {
            mut_writer
                .write_all(b"    ")
                .expect("Failed to write indent");
        }
    }

    pub fn write_str(&self, s: &str) -> &Self {
        let mut mut_writer = self.std_writer.borrow_mut();
        mut_writer
            .write_all(s.as_bytes())
            .expect("Failed to write string");
        self
    }
    pub fn write_fmt(&self, args: std::fmt::Arguments) -> &Self {
        let mut mut_writer = self.std_writer.borrow_mut();
        mut_writer
            .write_fmt(args)
            .expect("Failed to write formatted string");
        self
    }
}

impl<'a> AsmWriter<'a> {
    pub fn write_module(&self, module: &MirModule) {
        let mut global_status = GlobalWriteStatus {
            last_section: Section::None,
            last_align_log2: 0,
        };

        let mut extern_globals = Vec::with_capacity(16);

        self.indent_level.set(1);
        for &mod_item_ref in &module.items {
            let mod_item = mod_item_ref.data_from_module(module);
            let common = mod_item.get_common();
            let section = common.section;
            let align_log2 = common.align_log2;
            if common.linkage == Linkage::Extern {
                extern_globals.push(mod_item_ref);
                continue;
            }
            if section != global_status.last_section {
                self.wrap_indent();
                self.write_str(&format!(".section {}", section.asm_name()));
                global_status.last_section = section;
            }
            if align_log2 != global_status.last_align_log2 {
                self.wrap_indent();
                self.write_str(&format!(".align {}", align_log2));
                global_status.last_align_log2 = align_log2;
            }
            match &*mod_item {
                MirGlobal::Variable(gvar) => self.write_variable(gvar),
                MirGlobal::UnnamedData(gdata) => self.write_global_data(gdata),
                MirGlobal::Function(func) => self.write_function(module, func),
                _ => {}
            }
        }
    }

    /// Syntax:
    ///
    /// * when gdata meets condition as an asciz string: `.asciz "string"`
    /// * or else: `<unit-kind> <unit>, <unit>, ...`
    fn write_global_data(&self, gdata: &MirGlobalData) {
        self.wrap_indent();
        if let Some(asciz_str) = gdata.as_asciz_string() {
            self.write_str(".asciz ").write_str(asciz_str.as_str());
        } else {
            let unit_kind = gdata.get_unit_kind_name();
            let nunits = gdata.get_nunits();
            self.write_str(unit_kind);
            for i in 0..nunits {
                if i > 0 {
                    self.write_str(", ");
                }
                let mut mut_writer = self.std_writer.borrow_mut();
                gdata
                    .format_unit_index(i, &mut *mut_writer)
                    .expect("Failed to write global data unit");
            }
        }
    }

    fn write_variable(&self, gvar: &MirGlobalVariable) {
        self.indent_level.set(0);
        self.wrap_indent();
        self.write_str(gvar.get_name()).write_str(":");
        self.indent_level.set(1);
        let mut size = 0;
        for initval in &gvar.initval {
            self.wrap_indent();
            self.write_global_data(initval);
            size += initval.data.len();
        }
        if matches!(gvar.common.linkage, Linkage::Extern | Linkage::Global) {
            // For extern/global variables, we need to write the size of the variable.
            self.wrap_indent();
            self.write_fmt(format_args!(".size {}, {}", gvar.get_name(), size));
        }
    }

    /// VecSwitchTab syntax:
    ///
    /// ```aarch64
    /// .dword <label1>
    /// .dword <label2>
    /// ...
    /// ```
    fn format_vec_switch_tab(&self, istat: &InstWriteStatus, switch_tab: &VecSwitchTab) {
        let alloc_block = istat.module.borrow_alloc_block();
        for label in switch_tab.cases.iter() {
            let label = label.get().to_slabref_unwrap(&alloc_block);
            self.wrap_indent();
            self.write_fmt(format_args!(".dword {}", label.name.as_str()));
        }
    }
    /// BinSwitchTab syntax:
    ///
    /// ```aarch64
    /// .dword <label1>, <case1>
    /// .dword <label2>, <case2>
    /// ...
    /// ```
    ///
    /// `default_label` is a pesudo field used for pesudo instructions, it is not written to assembly.
    fn format_bin_switch_tab(
        &self,
        istat: &InstWriteStatus,
        switch_tab: &BinSwitchTab,
        base_name: String,
    ) {
        self.indent_level.set(0);
        self.wrap_indent();
        self.write_str(&base_name).write_str(":");
        self.indent_level.set(1);
        let alloc_block = istat.module.borrow_alloc_block();
        for (label, case) in switch_tab.cases.iter() {
            let label = label.to_slabref_unwrap(&alloc_block);
            self.wrap_indent();
            self.write_fmt(format_args!(".dword {}, {}", label.name.as_str(), case));
        }
    }

    fn write_function(&self, module: &MirModule, func: &Rc<MirFunc>) {
        let func_name = func.common.name.as_str();
        let func_name_len = func_name.len();

        self.indent_level.set(0);
        self.wrap_indent();
        self.write_str(func_name).write_str(":");

        let istat = InstWriteStatus {
            parent_func: func.clone(),
            module,
        };

        // Write function body
        self.indent_level.set(1);
        let alloc_bb = module.borrow_alloc_block();
        for (index, (_, block)) in func.blocks.view(&alloc_bb).into_iter().enumerate() {
            let writes_name = index == 0;
            self.write_block(&istat, block, writes_name);
        }

        // For extern functions, we need to write the size of the function.
        if matches!(func.common.linkage, Linkage::Extern | Linkage::Global) {
            self.wrap_indent();
            self.write_fmt(format_args!(".size {}, .-{}", func_name, func_name));
        }

        for vec_switch_tab in &*func.borrow_vec_switch_tabs() {
            self.indent_level.set(0);
            self.wrap_indent();
            self.write_fmt(format_args!(
                "_Z{func_name_len}{func_name}10switch_tab.v{:x}:",
                vec_switch_tab.tab_index.get()
            ));
            self.indent_level.set(1);
            self.wrap_indent();
            self.format_vec_switch_tab(&istat, vec_switch_tab);
        }
        for bin_switch_tab in &*func.borrow_bin_switch_tabs() {
            self.wrap_indent();
            self.format_bin_switch_tab(
                &istat,
                bin_switch_tab,
                format!(
                    "_Z{func_name_len}{func_name}10switch_tab.b{:x}",
                    bin_switch_tab.tab_index.get()
                ),
            );
        }
    }

    fn write_block(&self, istat: &InstWriteStatus, block: &MirBlock, writes_name: bool) {
        if writes_name {
            self.indent_level.set(0);
            self.wrap_indent();
            self.write_str(".Lbb_")
                .write_str(block.name.as_str())
                .write_str(":");
            self.indent_level.set(1);
        }
        for (_, inst) in block.insts.view(&istat.module.borrow_alloc_inst()) {
            self.wrap_indent();
            self.write_inst(istat, inst);
        }
    }

    fn write_inst(&self, istat: &InstWriteStatus, inst: &MirInst) {
        if inst.get_opcode().is_mir_pseudo() {
            panic!("Cannot write pseudo instruction to assembly: {:?}", inst);
        }
        let opcode = inst.get_opcode();
        match inst {
            MirInst::Nullary(_) => {
                self.write_str(inst.get_opcode().asm_name());
            }
            MirInst::CondBr(cond_br) => {
                let cond = cond_br.cond.get_name();
                self.write_fmt(format_args!("{}{} ", opcode.asm_name(), cond))
                    .write_operand(istat, cond_br.label().get());
            }
            MirInst::UncondBr(uncond_br) => {
                self.write_fmt(format_args!("{} ", uncond_br.get_opcode().asm_name()))
                    .write_operand(istat, uncond_br.label().get());
            }
            MirInst::BLink(blink) => {
                self.write_str(opcode.asm_name())
                    .write_str(" ")
                    .write_operand(istat, blink.label().get());
            }
            MirInst::RegCondBr(reg_cond_br) => {
                self.write_str(opcode.asm_name())
                    .write_str(" ")
                    .write_operand(istat, reg_cond_br.reg().get())
                    .write_str(", ")
                    .write_operand(istat, reg_cond_br.label().get());
            }
            MirInst::LoadStoreRRR(load_store_rrr) => {
                self.write_load_store_rrr(istat, opcode, load_store_rrr);
            }
            MirInst::LoadStoreRRI(load_store_rri) => {
                self.write_load_store_rri(istat, opcode, load_store_rri);
            }
            MirInst::LoadStoreLiteral(load_store_literal) => {
                self.write_str(opcode.asm_name())
                    .write_str(" ")
                    .write_operand(istat, load_store_literal.rt().get())
                    .write_str(", ")
                    .write_load_store_immediate(istat, load_store_literal.literal().get());
            }
            MirInst::Bin(bin_op) => {
                self.write_str(opcode.asm_name())
                    .write_str(" ")
                    .write_operand(istat, bin_op.rd().get())
                    .write_str(", ")
                    .write_operand(istat, bin_op.rn().get())
                    .write_str(", ")
                    .write_operand(istat, bin_op.rhs().get());
                if let Some(modify) = bin_op.rhs_op {
                    self.write_str(", ").write_str(modify.to_string().as_str());
                }
            }
            MirInst::Unary(unary_op) => {
                self.write_str(opcode.asm_name())
                    .write_str(" ")
                    .write_operand(istat, unary_op.rd().get())
                    .write_str(", ")
                    .write_operand(istat, unary_op.rhs().get());
                if let Some(modify) = unary_op.rhs_op {
                    self.write_str(", ").write_str(modify.to_string().as_str());
                }
            }
            MirInst::BFM(bfmop) => {
                self.write_str(opcode.asm_name())
                    .write_str(" ")
                    .write_operand(istat, bfmop.rd().get())
                    .write_str(", ")
                    .write_operand(istat, bfmop.rn().get())
                    .write_str(", ")
                    .write_operand(istat, bfmop.immr().get())
                    .write_str(", ")
                    .write_operand(istat, bfmop.imms().get());
            }
            MirInst::ExtR(extr_op) => {
                self.write_str(opcode.asm_name())
                    .write_str(" ")
                    .write_operand(istat, extr_op.rd().get())
                    .write_str(", ")
                    .write_operand(istat, extr_op.rn().get())
                    .write_str(", ")
                    .write_operand(istat, extr_op.rm().get())
                    .write_str(", ")
                    .write_operand(istat, extr_op.imm().get());
            }
            MirInst::Tri(triop) => {
                self.write_str(opcode.asm_name())
                    .write_str(" ")
                    .write_operand(istat, triop.rd().get())
                    .write_str(", ")
                    .write_operand(istat, triop.rn().get())
                    .write_str(", ")
                    .write_operand(istat, triop.rm().get())
                    .write_str(", ")
                    .write_operand(istat, triop.ra().get());
            }
            MirInst::Cmp(cmp_op) => {
                self.write_str(opcode.asm_name())
                    .write_str(" ")
                    .write_operand(istat, cmp_op.rn().get())
                    .write_str(", ")
                    .write_operand(istat, cmp_op.rhs().get());
                if let Some(rhs_op) = &cmp_op.rhs_op {
                    self.write_str(", ").write_str(rhs_op.to_string().as_str());
                }
            }
            MirInst::CondSelect(csel) => {
                self.write_str(opcode.asm_name())
                    .write_str(" ")
                    .write_operand(istat, csel.rd().get())
                    .write_str(", ")
                    .write_operand(istat, csel.rn().get())
                    .write_str(", ")
                    .write_operand(istat, csel.rm().get())
                    .write_str(", ")
                    .write_str(csel.cond.get_name());
            }
            MirInst::CondUnary(cunary) => {
                self.write_str(opcode.asm_name())
                    .write_str(" ")
                    .write_operand(istat, cunary.rd().get())
                    .write_str(", ")
                    .write_operand(istat, cunary.rn().get())
                    .write_str(", ")
                    .write_str(cunary.cond.get_name());
            }
            MirInst::CondSet(cond_set) => {
                self.write_str(opcode.asm_name())
                    .write_str(" ")
                    .write_operand(istat, cond_set.rd().get())
                    .write_str(", ")
                    .write_str(cond_set.cond.get_name());
            }
            MirInst::CondCmp(ccmp) => {
                self.write_str(opcode.asm_name())
                    .write_str(" ")
                    .write_operand(istat, ccmp.rn().get())
                    .write_str(", ")
                    .write_operand(istat, ccmp.rhs().get())
                    .write_str(", #")
                    .write_fmt(format_args!("{}", ccmp.nzcv.bits()))
                    .write_str(", ")
                    .write_str(ccmp.cond.get_name());
            }
            MirInst::Call(_)
            | MirInst::MirReturn(_)
            | MirInst::TabSwitch(_)
            | MirInst::BinSwitch(_) => panic!(
                "Cannot write MIR pesudo instruction to assembly: {:?}",
                inst
            ),
            MirInst::GuideNode(_) => {}
            MirInst::LoadConst(ldrc) => {
                self.write_str("ldr ")
                    .write_operand(istat, ldrc.rt().get())
                    .write_fmt(format_args!(", =0x{:x}", ldrc.get_imm()));
            }
            MirInst::BinCSR(bin_csrop) => {
                self.write_str(opcode.asm_name())
                    .write_str(" ")
                    .write_operand(istat, bin_csrop.rd().get())
                    .write_str(", ")
                    .write_operand(istat, bin_csrop.rn().get())
                    .write_str(", ")
                    .write_operand(istat, bin_csrop.rhs().get());
                if let Some(rm_op) = &bin_csrop.rhs_op {
                    self.write_str(", ").write_fmt(format_args!("{}", rm_op));
                }
            }
            MirInst::UnaryCSR(una_csrop) => {
                self.write_str(opcode.asm_name())
                    .write_str(" ")
                    .write_operand(istat, una_csrop.rd().get())
                    .write_str(", ")
                    .write_operand(istat, una_csrop.rhs().get());
                if let Some(rm_op) = &una_csrop.rhs_op {
                    self.write_str(", ").write_fmt(format_args!("{}", rm_op));
                }
            },
        }
    }

    fn write_operand(&self, istat: &InstWriteStatus, operand: MirOperand) -> &Self {
        match operand {
            MirOperand::VReg(vreg) => self.write_str(&vreg.to_string()),
            MirOperand::PReg(preg) => self.write_str(&preg.to_string()),
            MirOperand::Imm(i) => self.write_fmt(format_args!("#0x{:x}", i)),
            MirOperand::Global(item) => {
                let name = item.get_name(istat.module);
                match name {
                    Some(name) => self.write_str(name.as_str()),
                    None => self.write_str("<unnamed>"),
                }
            }
            MirOperand::Label(bb) => {
                let alloc_bb = istat.module.borrow_alloc_block();
                self.write_str(bb.to_slabref_unwrap(&alloc_bb).name.as_str())
            }
            MirOperand::VecSwitchTab(index) => {
                let vec_switch_tab = istat
                    .parent_func
                    .get_vec_switch_tab(index as usize)
                    .expect("Invalid VecSwitchTab index in write_operand");
                self.write_fmt(format_args!(
                    "_Z{}switch_tab.v{}",
                    istat.parent_func.common.name.len(),
                    vec_switch_tab.tab_index.get()
                ))
            }
            MirOperand::BinSwitchTab(index) => {
                let bin_switch_tab = istat
                    .parent_func
                    .get_bin_switch_tab(index as usize)
                    .expect("Invalid BinSwitchTab index in write_operand");
                self.write_fmt(format_args!(
                    "_Z{}switch_tab.b{}",
                    istat.parent_func.common.name.len(),
                    bin_switch_tab.tab_index.get()
                ))
            }
            MirOperand::None => {
                panic!("Cannot write None operand to assembly");
            }
        };
        self
    }

    /// Load/store instruction, with all operands being registers.
    ///
    /// AArch64 + MIR assembly syntax:
    ///
    /// - `<load-store-op> <Rt>, [<Rn>, <Rm>]`
    /// - `<load-store-op> <Rt>, [<Rn>, <Rm>, <UXTW|SXTW|SXTX>]`
    /// - `<load-store-op> <Rt>, [<Rn>, <Rm>, LSL #<shift>]`
    ///
    /// Accepts opcode:
    ///
    /// ```aarch64
    /// ldr{b|h|sb|sh|sw}
    /// str{b|h|sb|sh|sw}
    /// ```
    fn write_load_store_rrr(
        &self,
        istat: &InstWriteStatus,
        opcode: MirOP,
        ldst_rrr: &LoadStoreRRR,
    ) {
        self.write_str(opcode.asm_name())
            .write_str(" ")
            .write_operand(istat, ldst_rrr.rt().get())
            .write_str(", [")
            .write_operand(istat, ldst_rrr.rn().get())
            .write_str(", ")
            .write_operand(istat, ldst_rrr.rm().get());
        if let Some(rm_op) = &ldst_rrr.rm_op {
            self.write_str(", ").write_fmt(format_args!("{}", rm_op));
        }
        self.write_str("]");
    }

    /// Load/store instruction, with its mem address made of a base register and an offset.
    ///
    /// AArch64 + MIR assembly syntax:
    ///
    /// - `<load-store-op> <Rt>, [<Rn>, #<imm>]` => AddressMode::BaseOffset
    /// - `<load-store-op> <Rt>, [<Rn>], #<imm>` => AddressMode::PreIndex
    /// - `<load-store-op> <Rt>, [<Rn>, #<imm>]!` => AddressMode::PostIndex
    ///
    /// Accepts opcode:
    ///
    /// ```aarch64
    /// ldr{b|h|sb|sh|sw}
    /// str{b|h|sb|sh|sw}
    /// ```
    fn write_load_store_rri(
        &self,
        istat: &InstWriteStatus,
        opcode: MirOP,
        ldst_rri: &LoadStoreRRI,
    ) {
        self.write_str(opcode.asm_name())
            .write_str(" ")
            .write_operand(istat, ldst_rri.rt().get())
            .write_str(", [")
            .write_operand(istat, ldst_rri.rn().get());
        match ldst_rri.get_addr_mode() {
            AddressMode::BaseOnly => self.write_str("]"),
            AddressMode::BaseOffset => self
                .write_str(", ")
                .write_load_store_immediate(istat, ldst_rri.offset().get())
                .write_str("]"),
            AddressMode::PreIndex => self
                .write_str("], ")
                .write_load_store_immediate(istat, ldst_rri.offset().get()),
            AddressMode::PostIndex => self
                .write_load_store_immediate(istat, ldst_rri.offset().get())
                .write_str("]!"),
            _ => panic!(
                "Cannot write literal address mode for load/store RRI instruction: {ldst_rri:?}",
            ),
        };
    }

    fn write_load_store_immediate(&self, istat: &InstWriteStatus, imm: MirOperand) -> &Self {
        type M = MirOperand;
        match imm {
            MirOperand::Imm(value) => self.write_fmt(format_args!("#{}", value)),
            MirOperand::Global(item) => {
                if item.is_extern(istat.module) {
                    self.write_str(":got");
                }
                self.write_str(":lo12:").write_str(
                    item.get_name(istat.module)
                        .unwrap_or("<unnamed>".into())
                        .as_str(),
                )
            }
            M::Label(label) => {
                let alloc_bb = istat.module.borrow_alloc_block();
                self.write_str(":lo12:")
                    .write_str(label.to_slabref_unwrap(&alloc_bb).name.as_str())
            }
            MirOperand::VecSwitchTab(index) => {
                let vec_switch_tab = istat
                    .parent_func
                    .get_vec_switch_tab(index as usize)
                    .expect("Invalid VecSwitchTab index in write_operand");
                self.write_fmt(format_args!(
                    ":lo12:_Z{}switch_tab.v{}",
                    istat.parent_func.common.name.len(),
                    vec_switch_tab.tab_index.get()
                ))
            }
            MirOperand::BinSwitchTab(index) => {
                let bin_switch_tab = istat
                    .parent_func
                    .get_bin_switch_tab(index as usize)
                    .expect("Invalid BinSwitchTab index in write_operand");
                self.write_fmt(format_args!(
                    ":lo12:_Z{}switch_tab.b{}",
                    istat.parent_func.common.name.len(),
                    bin_switch_tab.tab_index.get()
                ))
            }

            MirOperand::None | MirOperand::VReg(_) | MirOperand::PReg(_) => {
                panic!(
                    "Cannot write immediate operand for load/store RRI instruction: {:?}",
                    imm
                );
            }
        }
    }
}

#[cfg(test)]
mod testing {
    use crate::ir::global::{GlobalData, func::FuncStorage};

    use super::*;

    #[test]
    fn test_asm_writer() {
        let mut stdout = std::io::stdout();
        let writer = AsmWriter::new(&mut stdout);

        let (ir_module, _) = crate::testing::cases::test_case_cfg_deep_while_br();
        let type_ctx = ir_module.type_ctx.clone();

        let main_func = ir_module
            .global_defs
            .borrow_mut()
            .get("main")
            .unwrap()
            .clone();
        let main_functy = match &*ir_module.get_global(main_func) {
            GlobalData::Func(func_data) => func_data.get_stored_func_type(),
            GlobalData::Alias(_) | GlobalData::Var(_) => unreachable!(),
        };

        let mut module = MirModule::new("test_module".into());
        let main_func = MirFunc::new_define(
            "main".into(),
            main_functy,
            &type_ctx,
            &mut module.borrow_alloc_block_mut(),
        );
        module.add_item(MirGlobal::Function(Rc::new(main_func)));

        writer.write_module(&module);
    }
}

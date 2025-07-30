use std::{io::Write, rc::Rc};

use crate::{
    base::slabref::SlabRef,
    mir::{
        fmt::FuncFormatContext,
        inst::switch::VecSwitchTab,
        module::{
            MirGlobal, MirModule,
            block::MirBlock,
            func::MirFunc,
            global::{Linkage, MirGlobalData, MirGlobalVariable, Section},
        },
    },
};

pub struct AsmWriter<'a> {
    std_writer: &'a mut dyn std::io::Write,
    ident_level: usize,
}

struct GlobalWriteStatus {
    last_section: Section,
    last_align_log2: u8,
}

impl<'a> AsmWriter<'a> {
    pub fn new(std_writer: &'a mut dyn std::io::Write) -> Self {
        Self { std_writer, ident_level: 0 }
    }
    pub fn write(&mut self, content: &str) -> std::io::Result<()> {
        for _ in 0..self.ident_level {
            write!(self.std_writer, "    ")?; // 4 spaces for indentation
        }
        write!(self.std_writer, "{}", content)
    }
    pub fn inc_indent(&mut self) {
        self.ident_level += 1;
    }
    pub fn dec_indent(&mut self) {
        if self.ident_level > 0 {
            self.ident_level -= 1;
        }
    }

    pub fn wrap_indent(&mut self) {
        writeln!(self.std_writer).expect("Failed to write newline");
        for _ in 0..self.ident_level {
            write!(self.std_writer, "    ").expect("Failed to write indentation");
        }
    }
}

impl<'a> Write for AsmWriter<'a> {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        self.std_writer.write(buf)
    }

    fn flush(&mut self) -> std::io::Result<()> {
        self.std_writer.flush()
    }
}

impl<'a> AsmWriter<'a> {
    pub fn write_module(&mut self, module: &MirModule) {
        let mut global_status =
            GlobalWriteStatus { last_section: Section::None, last_align_log2: 0 };

        let mut has_main: bool = false;
        let mut extern_globals = Vec::new();
        self.inc_indent();
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
                write!(self, ".section {}", section.asm_name()).expect("Failed to write section");
                global_status.last_section = section;
                self.wrap_indent();
                write!(self, ".align {}", align_log2).expect("Failed to write alignment");
            } else if align_log2 != global_status.last_align_log2 {
                self.wrap_indent();
                write!(self, ".align {}", align_log2).expect("Failed to write alignment");
                global_status.last_align_log2 = align_log2;
            }
            match &*mod_item {
                MirGlobal::Variable(gvar) => self.write_variable(gvar),
                MirGlobal::UnnamedData(gdata) => self.write_global_data(gdata),
                MirGlobal::Function(func) => {
                    if func.get_name() == "main" {
                        has_main = true;
                    }
                    self.write_function(module, func)
                }
                _ => {}
            }
        }
        // Now write extern globals
        if !extern_globals.is_empty() {
            self.dec_indent();
            self.wrap_indent();
            self.wrap_indent();
            write!(self, "# External symbols").expect("Failed to write comment");
            self.inc_indent();
            for g in extern_globals {
                let mod_item = g.data_from_module(module);
                let name = mod_item.get_name().unwrap();
                self.wrap_indent();
                write!(self, ".extern {}", name).expect("Failed to write extern");
            }
        }
        if !has_main {
            panic!("No main function found in the module");
        } else {
            self.wrap_indent();
            write!(self, ".global main").expect("Failed to write global main");
        }
        self.dec_indent();
    }

    /// Syntax:
    ///
    /// * when gdata meets condition as an asciz string: `.asciz "string"`
    /// * or else: `<unit-kind> <unit>, <unit>, ...`
    fn write_global_data(&mut self, gdata: &MirGlobalData) {
        self.wrap_indent();
        if gdata.common.section == Section::Bss {
            // For BSS section, we don't write the data, just the size.
            write!(self, ".zero {}", gdata.data.len()).expect("Failed to write zero data");
        } else if let Some(asciz_str) = gdata.as_asciz_string() {
            write!(self, ".asciz {asciz_str}").expect("Failed to write asciz string");
        } else {
            let unit_kind = gdata.get_unit_kind_name();
            let nunits = gdata.get_nunits();
            write!(self, ".{unit_kind} ").expect("Failed to write unit kind");
            for i in 0..nunits {
                if i > 0 {
                    write!(self, ", ").expect("Failed to write comma");
                }
                gdata
                    .format_unit_index(i, self.std_writer)
                    .expect("Failed to write global data unit");
            }
        }
    }

    fn write_variable(&mut self, gvar: &MirGlobalVariable) {
        self.wrap_indent();
        self.wrap_indent();
        let gvar_name = gvar.get_name();
        write!(self, ".global {gvar_name}").expect("Failed to write global variable");
        self.wrap_indent();
        write!(self, ".type {gvar_name}, @object").expect("Failed to write variable type");
        self.dec_indent();
        self.wrap_indent();
        write!(self, "{gvar_name}:").unwrap();
        self.inc_indent();
        let mut size = 0;
        for initval in &gvar.initval {
            self.write_global_data(initval);
            size += initval.data.len();
        }
        if matches!(gvar.common.linkage, Linkage::Extern | Linkage::Global) {
            // For extern/global variables, we need to write the size of the variable.
            self.wrap_indent();
            write!(self, ".size {}, {}", gvar_name, size).expect("Failed to write variable size");
        }
    }

    /// VecSwitchTab syntax:
    ///
    /// ```aarch64
    /// .switch.$function.$index:
    ///     .dword <label1>
    ///     .dword <label2>
    ///     ...
    /// ```
    fn format_vec_switch_tab(&mut self, istat: &mut FuncFormatContext, switch_tab: &VecSwitchTab) {
        let cur_func = istat.get_current_func();
        let self_name = switch_tab.get_name(&cur_func);
        self.dec_indent();
        self.wrap_indent();
        write!(self, "{self_name}:").unwrap();

        self.inc_indent();
        let alloc_block = istat.mir_module.borrow_alloc_block();
        for label in switch_tab.cases.iter() {
            let label = label.get().to_data(&alloc_block);
            let name = label.name.as_str();
            self.wrap_indent();
            write!(self, ".dword {name}").unwrap();
        }
    }

    fn write_function(&mut self, module: &MirModule, func: &Rc<MirFunc>) {
        let func_name = func.common.name.as_str();
        self.dec_indent();
        self.wrap_indent();
        self.wrap_indent();
        write!(self, "{func_name}:").unwrap();
        self.inc_indent();

        let alloc_bb = module.borrow_alloc_block();
        for (index, (_, block)) in func.blocks.view(&alloc_bb).into_iter().enumerate() {
            self.write_block(module, func, block, index != 0);
        }

        // For extern functions, we need to write the size of the function.
        if matches!(func.common.linkage, Linkage::Extern | Linkage::Global) {
            self.wrap_indent();
            self.write_fmt(format_args!(".size {}, .-{}", func_name, func_name))
                .unwrap();
        }

        for vec_switch_tab in &*func.borrow_vec_switch_tabs() {
            self.wrap_indent();
            let mut switchtab_str = String::new();
            self.format_vec_switch_tab(
                &mut FuncFormatContext::new(&mut switchtab_str, func.clone(), module),
                vec_switch_tab,
            );
            write!(self, "{switchtab_str}").expect("Failed to write switch tab");
        }
    }

    fn write_block(
        &mut self,
        module: &MirModule,
        func: &Rc<MirFunc>,
        block: &MirBlock,
        writes_name: bool,
    ) {
        if writes_name {
            self.dec_indent();
            self.wrap_indent();
            let bb_name = block.name.as_str();
            write!(self, "{bb_name}:").unwrap();
            self.inc_indent();
        }
        for (_, inst) in block.insts.view(&module.borrow_alloc_inst()).into_iter() {
            let mut inst_str = String::new();
            let mut fmt_context = FuncFormatContext::new(&mut inst_str, func.clone(), module);
            fmt_context
                .format_inst(inst)
                .expect("Failed to format instruction");
            self.wrap_indent();
            write!(self, "{inst_str}").expect("Failed to write instruction");
        }
    }
}

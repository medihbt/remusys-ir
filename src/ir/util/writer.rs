//! IR Writer implementation.

use crate::{
    base::slabref::SlabRef,
    ir::{
        IValueVisitor, ValueSSA,
        block::{BlockData, BlockRef},
        constant::{
            data::{ConstData, IConstDataVisitor},
            expr::{Array, ConstExprRef, IConstExprVisitor, Struct},
        },
        global::{GlobalData, GlobalRef, IGlobalObjectVisitor, func::FuncData},
        inst::{
            InstData, InstDataCommon, InstRef,
            binop::BinOp,
            callop::CallOp,
            cast::CastOp,
            cmp::CmpOp,
            gep::IndexPtrOp,
            load_store::{LoadOp, StoreOp},
            phi::PhiOp,
            sundury_inst::SelectOp,
            terminator::{Br, Jump, Ret, Switch},
            visitor::IInstVisitor,
        },
        module::{Module, ModuleAllocatorInner},
    },
    typing::{id::ValTypeID, types::FloatTypeKind},
};

use std::{
    cell::RefCell,
    io::{Result as IoResult, Write as IoWrite},
};

pub fn write_ir_to(module: &Module, writer: &mut dyn IoWrite) -> IoResult<()> {
    Ok(())
}

struct ModuleValueWriter<'a> {
    module: &'a Module,
    writer: RefCell<&'a mut dyn IoWrite>,
    inst_id_map: RefCell<Vec<usize>>,
    block_id_map: RefCell<Vec<usize>>,
    live_func_def: RefCell<Vec<GlobalRef>>,
}

impl<'a> ModuleValueWriter<'a> {
    fn write_str(&self, s: &str) {
        self.writer.borrow_mut().write_all(s.as_bytes()).unwrap();
    }
    fn write_fmt(&self, fmtargs: std::fmt::Arguments) {
        self.writer.borrow_mut().write_fmt(fmtargs).unwrap();
    }

    fn add_live_func_def(&self, func: GlobalRef) {
        self.live_func_def.borrow_mut().push(func);
        todo!("Scan the function and number all instructions and blocks");
    }

    fn number_function(&self, func: &FuncData) {
        todo!("Number all instructions and blocks in the function");
    }
    fn number_block(&self, block: BlockRef, block_data: &BlockData, initial_id: usize) -> usize {
        todo!(
            "Number this block with initial id, and then traverse through all instructions to number them"
        );
    }

    fn write_value_by_ref(&self, value: ValueSSA) {
        let inner = self.module.borrow_value_alloc();
        let content = basic_value_formatting::format_value_by_ref(
            &inner,
            &self.inst_id_map.borrow(),
            &self.block_id_map.borrow(),
            value,
        );
        self.write_str(content.as_str());
    }
}

impl IValueVisitor for ModuleValueWriter<'_> {
    fn read_block(&self, block: BlockRef, block_data: &BlockData) {
        todo!()
    }

    fn read_func_arg(&self, func: GlobalRef, index: u32) {
        todo!()
    }
}

impl IConstDataVisitor for ModuleValueWriter<'_> {
    fn read_int_const(&self, _: u8, _: i128) {}
    fn read_float_const(&self, _: FloatTypeKind, _: f64) {}
    fn read_ptr_null(&self, _: ValTypeID) {}
    fn read_undef(&self, _: ValTypeID) {}
    fn read_zero(&self, _: ValTypeID) {}
}

impl IConstExprVisitor for ModuleValueWriter<'_> {
    fn read_array(&self, _: ConstExprRef, _: &Array) {}
    fn read_struct(&self, _: ConstExprRef, _: &Struct) {}
}

impl IGlobalObjectVisitor for ModuleValueWriter<'_> {
    /// Syntax: `@<name> = external|dso_local global <type> [initializer], align <align>`
    fn read_global_variable(&self, global_ref: GlobalRef, gvar: &crate::ir::global::Var) {
        todo!()
    }

    /// Syntax: `@<name> = alias <type>, <target>, align <align>`
    fn read_global_alias(&self, global_ref: GlobalRef, galias: &crate::ir::global::Alias) {
        todo!()
    }

    /// Function declaration syntax: `declare <type> @<name>(<arg types>)`
    /// Function definition will be collected and handled in the other place.
    fn read_func(&self, global_ref: GlobalRef, gfunc: &crate::ir::global::func::FuncData) {
        todo!()
    }
}

impl IInstVisitor for ModuleValueWriter<'_> {
    /// Hidden, no syntax
    fn read_phi_end(&self, _: InstRef) {}

    /// Syntax: `%<name> = phi <type> [<value>, %<block>], ...`
    fn read_phi_inst(&self, inst_ref: InstRef, common: &InstDataCommon, phi: &PhiOp) {
        todo!()
    }

    /// Syntax: `unreachable`
    fn read_unreachable_inst(&self, inst_ref: InstRef, common: &InstDataCommon) {
        todo!()
    }

    /// Syntax: `ret <type> <value>`
    fn read_ret_inst(&self, inst_ref: InstRef, common: &InstDataCommon, ret: &Ret) {
        todo!()
    }

    /// Syntax: `br label %<block>`
    fn read_jump_inst(&self, inst_ref: InstRef, common: &InstDataCommon, jump: &Jump) {
        todo!()
    }

    /// Syntax: `br <cond>, label %<true block>, label %<false block>`
    fn read_br_inst(&self, inst_ref: InstRef, common: &InstDataCommon, br: &Br) {
        todo!()
    }

    /// Syntax:
    /// ```llvm-ir
    /// %<name> = switch <type> <value>, label %<default block>, [
    ///     <value1>, label %<case block>,
    ///     <value2>, label %<case block>,
    ///     ...
    /// ]
    /// ```
    fn read_switch_inst(&self, inst_ref: InstRef, common: &InstDataCommon, switch: &Switch) {
        todo!()
    }

    /// WARNING: Not implemented yet.
    /// Syntax: `tail call <type> @<name>(<args>)`
    fn read_tail_call_inst(&self, inst_ref: InstRef, common: &InstDataCommon) {
        todo!()
    }

    /// Syntax: `%<name> = load <type>, ptr %<ptr>, align <align>`
    fn read_load_inst(&self, inst_ref: InstRef, common: &InstDataCommon, load: &LoadOp) {
        todo!()
    }

    /// Syntax: `store <type> <value>, ptr %<ptr>, align <align>`
    fn read_store_inst(&self, inst_ref: InstRef, common: &InstDataCommon, store: &StoreOp) {
        todo!()
    }

    /// Syntax: `%<name> = select <type>, <cond>, <true value>, <false value>`
    fn read_select_inst(&self, inst_ref: InstRef, common: &InstDataCommon, select: &SelectOp) {
        todo!()
    }

    /// Syntax: `%<name> = <op> <type> <value1>, <value2>`
    fn read_bin_op_inst(&self, inst_ref: InstRef, common: &InstDataCommon, bin_op: &BinOp) {
        todo!()
    }

    /// Syntax: `%<name> = <op> <type> <value1>, <value2>`
    fn read_cmp_inst(&self, inst_ref: InstRef, common: &InstDataCommon, cmp: &CmpOp) {
        todo!()
    }

    /// Syntax: `%<name> = <op> <type> <value> to <type>`
    fn read_cast_inst(&self, inst_ref: InstRef, common: &InstDataCommon, cast: &CastOp) {
        todo!()
    }

    /// Syntax: `%<name> = getelementptr <type>, ptr %<ptr>, <index type> <index>, ...`
    fn read_index_ptr_inst(
        &self,
        inst_ref: InstRef,
        common: &InstDataCommon,
        index_ptr: &IndexPtrOp,
    ) {
        todo!()
    }

    /// Syntax: `%<name> = call <type> @<name>(<args>)`
    fn read_call_inst(&self, inst_ref: InstRef, common: &InstDataCommon, call: &CallOp) {
        todo!()
    }
}

mod basic_value_formatting {
    use std::{
        cell::RefCell,
        io::{Cursor, Read, Write},
    };

    use super::*;
    use slab::Slab;

    pub fn format_value_by_ref(
        module: &ModuleAllocatorInner,
        inst_id_map: &[usize],
        block_id_map: &[usize],
        value: ValueSSA,
    ) -> String {
        let formatter = BasicValueFormatter {
            module,
            inst_id_map,
            block_id_map,
            str_writer: RefCell::new(Cursor::new(vec![])),
        };
        formatter.value_visitor_diapatch(value, module);
        formatter.extract_string()
    }

    struct BasicValueFormatter<'a> {
        pub module: &'a ModuleAllocatorInner,
        pub inst_id_map: &'a [usize],
        pub block_id_map: &'a [usize],
        pub str_writer: RefCell<Cursor<Vec<u8>>>,
    }
    impl<'a> BasicValueFormatter<'a> {
        fn write_str(&self, s: &str) {
            self.str_writer
                .borrow_mut()
                .write_all(s.as_bytes())
                .unwrap();
        }
        fn write_fmt(&self, fmtargs: std::fmt::Arguments) {
            self.str_writer.borrow_mut().write_fmt(fmtargs).unwrap();
        }
        pub fn extract_string(&self) -> String {
            let mut str_writer = self.str_writer.borrow_mut();
            let mut buf = vec![];
            str_writer.set_position(0);
            str_writer.read_to_end(&mut buf).unwrap();
            String::from_utf8(buf).unwrap()
        }
    }
    impl IValueVisitor for BasicValueFormatter<'_> {
        fn read_block(&self, block: BlockRef, _: &BlockData) {
            self.write_str("%");
            self.write_str(self.block_id_map[block.get_handle()].to_string().as_str());
        }
        fn read_func_arg(&self, _: GlobalRef, index: u32) {
            self.write_fmt(format_args!("%{}", index));
        }
    }
    impl IConstDataVisitor for BasicValueFormatter<'_> {
        fn read_int_const(&self, nbits: u8, value: i128) {
            let real_value = ConstData::iconst_value_get_real_signed(nbits, value);
            if nbits == 1 {
                /* boolean */
                self.write_str(if real_value == 0 { "false" } else { "true" });
            } else {
                /* normal number */
                self.write_str(real_value.to_string().as_str());
            }
        }

        fn read_float_const(&self, fp_kind: FloatTypeKind, value: f64) {
            match fp_kind {
                FloatTypeKind::Ieee32 => self.write_str((value as f32).to_string().as_str()),
                FloatTypeKind::Ieee64 => self.write_str(value.to_string().as_str()),
            }
        }

        fn read_ptr_null(&self, _: ValTypeID) {
            self.write_str("null");
        }

        fn read_undef(&self, _: ValTypeID) {
            self.write_str("undef");
        }

        fn read_zero(&self, ty: ValTypeID) {
            match ty {
                ValTypeID::Void => self.write_str("void"),
                ValTypeID::Ptr => self.write_str("null"),
                ValTypeID::Int(..) => self.write_str("0"),
                ValTypeID::Float(..) => self.write_str("0.0"),
                ValTypeID::Array(..) | ValTypeID::Struct(..) | ValTypeID::StructAlias(..) => {
                    self.write_str("zeroinitializer")
                }
                ValTypeID::Func(..) => panic!("Function is not instantiable"),
            }
        }
    }

    impl IConstExprVisitor for BasicValueFormatter<'_> {
        fn read_array(&self, _: ConstExprRef, array_data: &Array) {
            self.write_str("[");
            for (i, item) in array_data.elems.iter().enumerate() {
                if i > 0 {
                    self.write_str(", ");
                }
                self.value_visitor_diapatch(item.clone(), self.module);
            }
            self.write_str("]");
        }

        fn read_struct(&self, _: ConstExprRef, s: &Struct) {
            self.write_str("{");
            for (i, item) in s.elems.iter().enumerate() {
                if i > 0 {
                    self.write_str(", ");
                }
                self.value_visitor_diapatch(item.clone(), self.module);
            }
            self.write_str("}");
        }
    }

    impl IGlobalObjectVisitor for BasicValueFormatter<'_> {
        fn global_object_visitor_dispatch(&self, globl: GlobalRef, alloc: &Slab<GlobalData>) {
            self.write_str("@");
            self.write_str(globl.to_slabref_unwrap(alloc).get_common().name.as_str());
        }
        fn read_global_variable(&self, _: GlobalRef, _: &crate::ir::global::Var) {}
        fn read_global_alias(&self, _: GlobalRef, _: &crate::ir::global::Alias) {}
        fn read_func(&self, _: GlobalRef, _: &crate::ir::global::func::FuncData) {}
    }
    impl IInstVisitor for BasicValueFormatter<'_> {
        fn read_phi_end(&self, _: InstRef) {}
        fn read_phi_inst(&self, _: InstRef, _: &InstDataCommon, _: &PhiOp) {}
        fn read_unreachable_inst(&self, _: InstRef, _: &InstDataCommon) {}
        fn read_ret_inst(&self, _: InstRef, _: &InstDataCommon, _: &Ret) {}
        fn read_jump_inst(&self, _: InstRef, _: &InstDataCommon, _: &Jump) {}
        fn read_br_inst(&self, _: InstRef, _: &InstDataCommon, _: &Br) {}
        fn read_switch_inst(&self, _: InstRef, _: &InstDataCommon, _: &Switch) {}
        fn read_tail_call_inst(&self, _: InstRef, _: &InstDataCommon) {}
        fn read_load_inst(&self, _: InstRef, _: &InstDataCommon, _: &LoadOp) {}
        fn read_store_inst(&self, _: InstRef, _: &InstDataCommon, _: &StoreOp) {}
        fn read_select_inst(&self, _: InstRef, _: &InstDataCommon, _: &SelectOp) {}
        fn read_bin_op_inst(&self, _: InstRef, _: &InstDataCommon, _: &BinOp) {}
        fn read_cmp_inst(&self, _: InstRef, _: &InstDataCommon, _: &CmpOp) {}
        fn read_cast_inst(&self, _: InstRef, _: &InstDataCommon, _: &CastOp) {}
        fn read_index_ptr_inst(&self, _: InstRef, _: &InstDataCommon, _: &IndexPtrOp) {}
        fn read_call_inst(&self, _: InstRef, _: &InstDataCommon, _: &CallOp) {}

        fn inst_visitor_dispatch(&self, inst_ref: InstRef, _: &Slab<InstData>) {
            self.write_str("%");
            self.write_str(self.inst_id_map[inst_ref.get_handle()].to_string().as_str());
        }
    }
}

use std::cell::Cell;

use crate::{
    base::INullableValue,
    mir::module::MirGlobalRef,
    typing::{IValType, TypeContext, ValTypeID},
};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Section {
    Text,
    RoData,
    Data,
    Bss,
    None,
}

impl Section {
    pub fn asm_name(&self) -> &'static str {
        match self {
            Section::Text => ".text",
            Section::RoData => ".rodata",
            Section::Data => ".data",
            Section::Bss => ".bss",
            Section::None => panic!("Passed no section to MirGlobalCommon"),
        }
    }
    pub fn is_data(&self) -> bool {
        matches!(self, Section::Data | Section::Bss)
    }
    pub fn writable(&self) -> bool {
        matches!(self, Section::Data | Section::Bss)
    }
    pub fn executable(&self) -> bool {
        matches!(self, Section::Text)
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Linkage {
    Global,
    Weak,
    Private,
    Extern,
    NotSymbol,
}

#[derive(Debug, Clone)]
pub struct MirGlobalCommon {
    /// Useless when name is empty (e.g. when this global data is "unnamed global data" pesudo op).
    pub name: String,
    /// Section where this global data resides.
    pub section: Section,
    /// Linkage of this global data.
    pub linkage: Linkage,
    /// Log2 of the alignment of this global data.
    pub align_log2: u8,
    /// Size of the global data in bytes.
    pub size: usize,
    /// Index of the global data in the module. `u32::MAX` means not set.
    pub(super) self_ref: Cell<MirGlobalRef>,
}

impl MirGlobalCommon {
    pub fn new(name: String, section: Section, align_log2: u8, linkage: Linkage) -> Self {
        Self {
            name,
            section,
            linkage,
            align_log2,
            size: 0,
            self_ref: Cell::new(MirGlobalRef::new_null()), // Default to MAX to indicate not set
        }
    }
    pub fn get_align(&self) -> usize {
        1usize << self.align_log2
    }
    pub fn has_name(&self) -> bool {
        !self.name.is_empty()
    }
    pub fn get_self_ref(&self) -> MirGlobalRef {
        self.self_ref.get()
    }
    pub fn set_self_ref(&self, self_ref: MirGlobalRef) {
        self.self_ref.set(self_ref);
    }
}

/// Unnamed global data in MIR.
///
/// Syntax:
///
/// ```aarch64
///     ; when section of this differs from the section of the previous global data.
///     [.section <section>]
///
///     ; when alignment of this differs from the alignment of the previous global data.
///     [.align <align_log2>]
///
///     ; unit-kind = byte | half | word | dword
///     .<unit kind> data0, data1, ..., dataN
/// ```
///
/// NOTE: MirGlobalData `name` is unavailable, it is always empty.
/// This is because this global data is not a named global variable, but rather a collection of
/// unnamed data that is used for various purposes, such as storing constants.
#[derive(Debug, Clone)]
pub struct MirGlobalData {
    pub common: MirGlobalCommon,
    pub data: Vec<u8>,
    pub unit_bytes_log2: u8, // Log2 of the size of each unit in the data
}

impl MirGlobalData {
    fn new_raw_data<T: Sized + Copy, const T_SIZE: usize>(
        section: Section,
        data: &[T],
        unit_bytes_log2: u8,
        to_le_bytes: impl Fn(T) -> [u8; T_SIZE],
    ) -> Self {
        let mut bytes = Vec::with_capacity(data.len() << unit_bytes_log2);
        for &value in data {
            bytes.extend_from_slice(&to_le_bytes(value));
        }
        Self {
            common: MirGlobalCommon::new(
                String::new(),
                section,
                unit_bytes_log2,
                Linkage::NotSymbol,
            ),
            data: bytes,
            unit_bytes_log2,
        }
    }
    pub fn new_bytes_vec(section: Section, data: Vec<u8>) -> Self {
        let unit_bytes_log2 = 0; // 1 byte per unit
        Self {
            common: MirGlobalCommon::new(
                String::new(),
                section,
                unit_bytes_log2,
                Linkage::NotSymbol,
            ),
            data,
            unit_bytes_log2,
        }
    }
    pub fn new_bytes(section: Section, data: &[u8]) -> Self {
        Self::new_bytes_vec(section, data.to_vec())
    }
    pub fn new_half(section: Section, data: &[u16]) -> Self {
        Self::new_raw_data(section, data, 1, u16::to_le_bytes)
    }
    pub fn new_word(section: Section, data: &[u32]) -> Self {
        Self::new_raw_data(section, data, 2, u32::to_le_bytes)
    }
    pub fn new_dword(section: Section, data: &[u64]) -> Self {
        Self::new_raw_data(section, data, 3, u64::to_le_bytes)
    }

    pub fn get_unit_size(&self) -> usize {
        1usize << self.unit_bytes_log2
    }

    pub fn get_unit_kind_name(&self) -> &'static str {
        match self.unit_bytes_log2 {
            0 => "byte",
            1 => "short",
            2 => "long",
            3 => "quad",
            _ => panic!("Unsupported unit size"),
        }
    }

    pub fn get_nunits(&self) -> usize {
        self.data.len() >> self.unit_bytes_log2
    }
    pub fn get_unit(&self, index: usize) -> Option<&[u8]> {
        if index >= self.get_nunits() {
            return None;
        }
        let start = index << self.unit_bytes_log2;
        let end = start + self.get_unit_size();
        Some(&self.data[start..end])
    }
    /// Writes a unit to string by index and unit_bytes_log2.
    pub fn unit_to_string(&self, index: usize) -> Option<String> {
        let mut buffer = Vec::with_capacity(self.get_unit_size() * 2 + 2);
        self.get_unit(index).and_then(|unit| {
            if Self::format_unit_data(unit, self.unit_bytes_log2, &mut buffer).is_ok() {
                Some(String::from_utf8(buffer).ok()?)
            } else {
                None
            }
        })
    }
    /// Writes a unit to the writer by index and unit_bytes_log2.
    pub fn format_unit_index(
        &self,
        index: usize,
        writer: &mut dyn std::io::Write,
    ) -> std::io::Result<()> {
        if let Some(unit) = self.get_unit(index) {
            Self::format_unit_data(unit, self.unit_bytes_log2, writer)
        } else {
            Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Index out of bounds",
            ))
        }
    }
    /// Writes a unit to the writer by index and unit_bytes_log2.
    fn format_unit_data(
        unit: &[u8],
        unit_bytes_log2: u8,
        writer: &mut dyn std::io::Write,
    ) -> std::io::Result<()> {
        match unit_bytes_log2 {
            0 => write!(writer, "0x{:02x}", unit[0]),
            1 => write!(writer, "0x{:04x}", u16::from_le_bytes([unit[0], unit[1]])),
            2 => write!(
                writer,
                "0x{:08x}",
                u32::from_le_bytes([unit[0], unit[1], unit[2], unit[3]])
            ),
            3 => write!(
                writer,
                "0x{:016x}",
                u64::from_le_bytes([
                    unit[0], unit[1], unit[2], unit[3], unit[4], unit[5], unit[6], unit[7]
                ])
            ),
            _ => Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Unsupported unit size",
            )),
        }
    }

    pub fn can_make_asciz(&self) -> bool {
        self.unit_bytes_log2 == 0 && self.data.last() == Some(&0)
    }
    pub fn write_as_asciz(&self, mut writer: impl std::io::Write) -> Result<bool, std::io::Error> {
        if !self.can_make_asciz() {
            return Ok(false);
        }
        writer.write(b"\"")?;
        for &byte in &self.data[..self.data.len() - 1] {
            if byte == 0 {
                writer.write_all(b"\\0")?;
            } else if byte.is_ascii_graphic() {
                writer.write_all(&[byte])?;
            } else {
                write!(writer, "\\x{:02x}", byte)?;
            }
        }
        writer.write(b"\"")?;
        Ok(true)
    }
    pub fn as_asciz_string(&self) -> Option<String> {
        let mut buffer = Vec::with_capacity(self.data.len() + 2);
        if self.write_as_asciz(&mut buffer).ok()? { String::from_utf8(buffer).ok() } else { None }
    }
}

/// Represents a global variable in MIR.
///
/// Syntax:
///
/// ```aarch64
///     [.section <section>]
///     .global <name>, <linkage>
///     [.align <align_log2>]
///     (initvals...)
/// ```
#[derive(Debug, Clone)]
pub struct MirGlobalVariable {
    pub common: MirGlobalCommon,
    /// The type of the global variable.
    pub ty: ValTypeID,
    /// The initial value of the global variable.
    pub initval: Vec<MirGlobalData>,
}

impl MirGlobalVariable {
    pub fn new_extern(
        name: String,
        section: Section,
        ty: ValTypeID,
        type_ctx: &TypeContext,
    ) -> Self {
        let align_log2 = ty.get_align_log2(type_ctx).max(3);
        Self {
            common: MirGlobalCommon::new(name, section, align_log2, Linkage::Extern),
            ty,
            initval: Vec::new(),
        }
    }
    pub fn with_init(
        name: String,
        section: Section,
        ty: ValTypeID,
        initval: Vec<MirGlobalData>,
        type_ctx: &TypeContext,
    ) -> Self {
        let align_log2 = ty.get_align_log2(type_ctx).max(3);
        Self {
            common: MirGlobalCommon::new(name, section, align_log2, Linkage::Global),
            ty,
            initval,
        }
    }

    pub fn mark_defined(&mut self) {
        if self.common.linkage == Linkage::Extern {
            self.common.linkage = Linkage::Global;
        }
    }
    pub fn is_extern(&self) -> bool {
        self.common.linkage == Linkage::Extern
    }
    pub fn push_data(&mut self, mut data: MirGlobalData) {
        if self.common.linkage == Linkage::Extern {
            panic!("Cannot push data to an extern global variable {self:?}");
        }
        self.common.size += data.common.size;
        data.common.section = self.common.section;
        self.initval.push(data);
    }

    pub fn get_name(&self) -> &str {
        &self.common.name
    }
}

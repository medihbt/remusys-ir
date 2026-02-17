//! IR attributes and metadata.

use crate::typing::ValTypeID;
use smallvec::SmallVec;
use std::{fmt::Debug, str::FromStr};

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Attribute {
    NoUndef,
    IntExt(IntExtAttr),
    PtrReadOnly,
    PtrNoCapture,
    FuncNoReturn,
    FuncInline(InlineAttr),
    FuncAlignStack(u8 /* log2 of alignment in bytes */),
    FuncPure,
    ArgPtrTarget(PtrArgTargetAttr),
    /// slimilar to LLVM `dereferenceable` attribute
    ArgPtrDerefBytes(usize),
}
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum AttrClass {
    NoUndef,
    IntExt,
    PtrReadOnly,
    PtrNoCapture,
    FuncNoReturn,
    FuncInline,
    FuncAlignStack,
    FuncPure,
    ArgPtrTarget,
    ArgPtrDerefBytes,
}
impl Attribute {
    pub fn get_pos(&self) -> AttributePos {
        use Attribute::*;
        match self {
            NoUndef => AttributePos::ANYWHERE,
            IntExt(_) => AttributePos::FUNCARG,
            PtrReadOnly => AttributePos::FUNCARG | AttributePos::CALLARG,
            PtrNoCapture => AttributePos::FUNCARG | AttributePos::CALLARG,
            FuncNoReturn => AttributePos::FUNC,
            FuncInline(_) => AttributePos::FUNC,
            FuncAlignStack(_) => AttributePos::FUNC,
            FuncPure => AttributePos::FUNC,
            ArgPtrTarget(_) => AttributePos::FUNCARG,
            ArgPtrDerefBytes(_) => AttributePos::FUNCARG | AttributePos::CALLARG,
        }
    }

    pub fn get_class(&self) -> AttrClass {
        use Attribute::*;
        match self {
            NoUndef => AttrClass::NoUndef,
            IntExt(_) => AttrClass::IntExt,
            PtrReadOnly => AttrClass::PtrReadOnly,
            PtrNoCapture => AttrClass::PtrNoCapture,
            FuncNoReturn => AttrClass::FuncNoReturn,
            FuncInline(_) => AttrClass::FuncInline,
            FuncAlignStack(_) => AttrClass::FuncAlignStack,
            FuncPure => AttrClass::FuncPure,
            ArgPtrTarget(_) => AttrClass::ArgPtrTarget,
            ArgPtrDerefBytes(_) => AttrClass::ArgPtrDerefBytes,
        }
    }
}

bitflags::bitflags! {
    #[derive(Clone, Copy, PartialEq, Eq, Hash)]
    pub struct AttributePos: u8 {
        const BANNED    = 0b0000_0000;
        const FUNC      = 0b0000_0001;
        const FUNCARG   = 0b0000_0010;
        const CALLARG   = 0b0000_0100;
        const ANYWHERE  = 0b0000_0111;
    }
}
impl Debug for AttributePos {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut parts = SmallVec::<[&str; 4]>::new();
        if self.is_empty() {
            f.write_str("AttributePos(NONE)")?;
            return Ok(());
        }
        if self.contains(Self::ANYWHERE) {
            f.write_str("AttributePos(ANYWHERE)")?;
            return Ok(());
        }
        if self.contains(AttributePos::FUNC) {
            parts.push("FUNC");
        }
        if self.contains(AttributePos::FUNCARG) {
            parts.push("FUNCARG");
        }
        if self.contains(AttributePos::CALLARG) {
            parts.push("CALLARG");
        }
        write!(f, "AttributePos({})", parts.join("|"))
    }
}
impl Default for AttributePos {
    fn default() -> Self {
        Self::ANYWHERE
    }
}
impl AttributePos {
    pub fn allows(self, pos: AttributePos) -> bool {
        self.contains(pos)
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum InlineAttr {
    Never,
    #[default]
    Normal,
    Hint,
    Always,
}
impl FromStr for InlineAttr {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "never" => Ok(InlineAttr::Never),
            "normal" => Ok(InlineAttr::Normal),
            "inline" => Ok(InlineAttr::Hint),
            "always" => Ok(InlineAttr::Always),
            _ => Err(()),
        }
    }
}
impl InlineAttr {
    pub fn as_str(&self) -> &'static str {
        match self {
            InlineAttr::Never => "never",
            InlineAttr::Normal => "normal",
            InlineAttr::Hint => "inline",
            InlineAttr::Always => "always",
        }
    }

    pub fn from_bits(bits: u8) -> Option<Self> {
        match bits {
            0 => Some(InlineAttr::Never),
            1 => Some(InlineAttr::Normal),
            2 => Some(InlineAttr::Hint),
            3 => Some(InlineAttr::Always),
            _ => None,
        }
    }
    pub fn to_bits(&self) -> u8 {
        match self {
            InlineAttr::Never => 0,
            InlineAttr::Normal => 1,
            InlineAttr::Hint => 2,
            InlineAttr::Always => 3,
        }
    }

    pub fn should_inline(&self) -> bool {
        matches!(self, InlineAttr::Hint | InlineAttr::Always)
    }
    pub fn is_strict(&self) -> bool {
        matches!(self, InlineAttr::Never | InlineAttr::Always)
    }
}

#[derive(Default, Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IntExtAttr {
    #[default]
    NoExt,
    ZeroExt,
    SignExt,
}
impl FromStr for IntExtAttr {
    type Err = ();

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s {
            "zeroext" => Ok(IntExtAttr::ZeroExt),
            "signext" => Ok(IntExtAttr::SignExt),
            "noext" => Ok(IntExtAttr::NoExt),
            _ => Err(()),
        }
    }
}
impl IntExtAttr {
    pub fn as_str(&self) -> &'static str {
        match self {
            IntExtAttr::ZeroExt => "zeroext",
            IntExtAttr::SignExt => "signext",
            IntExtAttr::NoExt => "noext",
        }
    }

    pub fn from_bits(bits: u8) -> Option<Self> {
        match bits {
            1 => Some(IntExtAttr::NoExt),
            2 => Some(IntExtAttr::ZeroExt),
            3 => Some(IntExtAttr::SignExt),
            _ => None,
        }
    }
    pub fn to_bits(self) -> u8 {
        match self {
            IntExtAttr::NoExt => 1,
            IntExtAttr::ZeroExt => 2,
            IntExtAttr::SignExt => 3,
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PtrArgTargetAttr {
    ByRef(ValTypeID),
    ByVal(ValTypeID),
    DynArray(ValTypeID),
}

#[derive(Debug, Clone, thiserror::Error)]
pub enum AttrError {
    #[error("Invalid attribute position: required {required:?}, found {found:?}")]
    InvalidAttrPos { required: AttributePos, found: AttributePos },
}
pub type AttrRes<T = ()> = Result<T, AttrError>;

#[derive(Debug, Clone)]
pub struct AttrSet {
    bits: AttrSetBits,
    /// 3 by default
    func_align: u8,
    attr_pos: AttributePos,
    ptr_arg_target: ValTypeID,
    /// Some|None is stored in bits
    ptr_arg_deref_bytes: usize,
}

bitflags::bitflags! {
    /// See the definition of `AttrSet` for the meaning of each bit field.
    #[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
    struct AttrSetBits: u64 {
        const NONE          = 0x0000_0000;

        // single-bit section

        const NO_UNDEF      = 0x0000_0001;
        const PTR_READONLY  = 0x0000_0002;
        const PTR_NOCAPTURE = 0x0000_0004;
        const FUNC_NORETURN = 0x0000_0008;
        const FUNC_PURE     = 0x0000_0010;

        // multi-bit and discriminant section (starts from 2^32)

        /// 2 bits: `00=unset, 01=noext, 10=zext, 11=sext`
        const MASK_INT_EXT     = 3 << AttrSet::INTEXT_SHIFT; // 2 bits
        /// 2 bits: `00=never, 01=normal, 10=inline, 11=always`
        const MASK_FUNC_INLINE = 3 << AttrSet::FUNCINLINE_SHIFT; // 2 bits
        /// 2 bits: `00=None, 01=ByRef, 10=ByVal, 11=DynArray`
        const MASK_PTR_ARG_TARGET  = 3 << AttrSet::PTR_ARG_TARGET_SHIFT; // 2 bits
        /// 1 bit: whether `ArgPtrDerefBytes` is set
        const MASK_PTR_DEREF_BYTES = 0x0040_0000_0000;

        /// Default attribute set value
        const DEFAULT_VALUE    = 0x0004_0000_0000;
    }
}

impl Default for AttrSet {
    fn default() -> Self {
        Self {
            bits: AttrSetBits::DEFAULT_VALUE,
            attr_pos: AttributePos::ANYWHERE,
            func_align: 3,
            ptr_arg_target: ValTypeID::Void,
            ptr_arg_deref_bytes: 0,
        }
    }
}
impl AttrSet {
    pub fn new(pos: AttributePos) -> Self {
        Self { attr_pos: pos, ..Self::default() }
    }

    pub fn is_noundef(&self) -> bool {
        self.bits.contains(AttrSetBits::NO_UNDEF)
    }
    pub fn set_noundef(&mut self, val: bool) {
        self.bits.set(AttrSetBits::NO_UNDEF, val);
    }

    const INTEXT_SHIFT: u32 = 32;
    pub fn get_int_ext(&self) -> Option<IntExtAttr> {
        let bits = (self.bits & AttrSetBits::MASK_INT_EXT).bits() >> Self::INTEXT_SHIFT;
        IntExtAttr::from_bits(bits as u8)
    }
    pub fn set_int_ext(&mut self, attr: IntExtAttr) {
        self.bits.remove(AttrSetBits::MASK_INT_EXT);
        let bits = (attr.to_bits() as u64) << Self::INTEXT_SHIFT;
        self.bits.insert(AttrSetBits::from_bits_truncate(bits));
    }
    pub fn clean_int_ext(&mut self) {
        self.bits.remove(AttrSetBits::MASK_INT_EXT);
    }

    pub fn is_ptr_readonly(&self) -> bool {
        self.bits.contains(AttrSetBits::PTR_READONLY)
    }
    pub fn set_ptr_readonly(&mut self, val: bool) {
        self.bits.set(AttrSetBits::PTR_READONLY, val);
    }

    pub fn is_ptr_nocapture(&self) -> bool {
        self.bits.contains(AttrSetBits::PTR_NOCAPTURE)
    }
    pub fn set_ptr_nocapture(&mut self, val: bool) {
        self.bits.set(AttrSetBits::PTR_NOCAPTURE, val);
    }

    pub fn is_func_noreturn(&self) -> bool {
        self.bits.contains(AttrSetBits::FUNC_NORETURN)
    }
    pub fn set_func_noreturn(&mut self, val: bool) {
        self.bits.set(AttrSetBits::FUNC_NORETURN, val);
    }

    pub fn is_func_pure(&self) -> bool {
        self.bits.contains(AttrSetBits::FUNC_PURE)
    }
    pub fn set_func_pure(&mut self, val: bool) {
        self.bits.set(AttrSetBits::FUNC_PURE, val);
    }

    const FUNCINLINE_SHIFT: u32 = 34;
    pub fn get_func_inline(&self) -> InlineAttr {
        let bits = (self.bits & AttrSetBits::MASK_FUNC_INLINE).bits() >> Self::FUNCINLINE_SHIFT;
        InlineAttr::from_bits(bits as u8).unwrap_or(InlineAttr::Normal)
    }
    pub fn set_func_inline(&mut self, attr: InlineAttr) {
        self.bits.remove(AttrSetBits::MASK_FUNC_INLINE);
        let bits = (attr.to_bits() as u64) << Self::FUNCINLINE_SHIFT;
        self.bits.insert(AttrSetBits::from_bits_truncate(bits));
    }

    const PTR_ARG_TARGET_SHIFT: u32 = 48;
    pub fn get_ptr_arg_target(&self) -> Option<PtrArgTargetAttr> {
        let bits =
            (self.bits & AttrSetBits::MASK_PTR_ARG_TARGET).bits() >> Self::PTR_ARG_TARGET_SHIFT;
        match bits {
            1 => Some(PtrArgTargetAttr::ByRef(self.ptr_arg_target)),
            2 => Some(PtrArgTargetAttr::ByVal(self.ptr_arg_target)),
            3 => Some(PtrArgTargetAttr::DynArray(self.ptr_arg_target)),
            _ => None,
        }
    }
    pub fn set_ptr_arg_target(&mut self, attr: PtrArgTargetAttr) {
        self.bits.remove(AttrSetBits::MASK_PTR_ARG_TARGET);
        let (bits, val) = match attr {
            PtrArgTargetAttr::ByRef(ty) => (1u64, ty),
            PtrArgTargetAttr::ByVal(ty) => (2u64, ty),
            PtrArgTargetAttr::DynArray(ty) => (3u64, ty),
        };
        self.ptr_arg_target = val;
        let bits = bits << Self::PTR_ARG_TARGET_SHIFT;
        self.bits.insert(AttrSetBits::from_bits_truncate(bits));
    }
    pub fn clear_ptr_arg_target(&mut self) {
        self.bits.remove(AttrSetBits::MASK_PTR_ARG_TARGET);
        self.ptr_arg_target = ValTypeID::Void;
    }

    pub fn get_ptr_arg_deref_bytes(&self) -> Option<usize> {
        if self.bits.contains(AttrSetBits::MASK_PTR_DEREF_BYTES) {
            Some(self.ptr_arg_deref_bytes)
        } else {
            None
        }
    }
    pub fn set_ptr_arg_deref_bytes(&mut self, bytes: usize) {
        self.bits.insert(AttrSetBits::MASK_PTR_DEREF_BYTES);
        self.ptr_arg_deref_bytes = bytes;
    }
    pub fn clear_ptr_arg_deref_bytes(&mut self) {
        self.bits.remove(AttrSetBits::MASK_PTR_DEREF_BYTES);
        self.ptr_arg_deref_bytes = 0;
    }

    pub fn set_attr(&mut self, attr: Attribute) {
        use Attribute::*;
        match attr {
            NoUndef => self.set_noundef(true),
            IntExt(ext) => self.set_int_ext(ext),
            PtrReadOnly => self.set_ptr_readonly(true),
            PtrNoCapture => self.set_ptr_nocapture(true),
            FuncNoReturn => self.set_func_noreturn(true),
            FuncInline(inline) => self.set_func_inline(inline),
            FuncAlignStack(align) => self.func_align = align,
            FuncPure => self.set_func_pure(true),
            ArgPtrTarget(target) => self.set_ptr_arg_target(target),
            ArgPtrDerefBytes(bytes) => self.set_ptr_arg_deref_bytes(bytes),
        }
    }
    pub fn has_attr_class(&self, class: AttrClass) -> bool {
        use AttrClass::*;
        match class {
            NoUndef => self.is_noundef(),
            IntExt => self.get_int_ext().is_some(),
            PtrReadOnly => self.is_ptr_readonly(),
            PtrNoCapture => self.is_ptr_nocapture(),
            FuncNoReturn => self.is_func_noreturn(),
            FuncInline => {
                let inline = self.get_func_inline();
                inline != InlineAttr::Normal
            }
            FuncAlignStack => self.func_align != 3,
            FuncPure => self.is_func_pure(),
            ArgPtrTarget => self.get_ptr_arg_target().is_some(),
            ArgPtrDerefBytes => self.get_ptr_arg_deref_bytes().is_some(),
        }
    }
    pub fn clean_attr(&mut self, class: AttrClass) {
        use AttrClass::*;
        match class {
            NoUndef => self.set_noundef(false),
            IntExt => self.clean_int_ext(),
            PtrReadOnly => self.set_ptr_readonly(false),
            PtrNoCapture => self.set_ptr_nocapture(false),
            FuncNoReturn => self.set_func_noreturn(false),
            FuncInline => self.set_func_inline(InlineAttr::Normal),
            FuncAlignStack => self.func_align = 3,
            FuncPure => self.set_func_pure(false),
            ArgPtrTarget => self.clear_ptr_arg_target(),
            ArgPtrDerefBytes => self.clear_ptr_arg_deref_bytes(),
        }
    }

    /// Set the positional context of this attribute set (e.g. function / func arg / call arg).
    pub fn set_pos(&mut self, pos: AttributePos) {
        self.attr_pos = pos;
    }
    /// Get current positional context.
    pub fn pos(&self) -> AttributePos {
        self.attr_pos
    }

    /// Iterate over all currently active (logically present) attributes in a stable order.
    /// Attributes whose value is the implicit default (e.g. InlineAttr::Normal, func_align==3) are omitted.
    pub fn iter(&self) -> impl Iterator<Item = Attribute> + '_ {
        let mut list = SmallVec::<[Attribute; 12]>::new();
        if self.is_noundef() {
            list.push(Attribute::NoUndef);
        }
        if let Some(ext) = self.get_int_ext() {
            list.push(Attribute::IntExt(ext));
        }
        if self.is_ptr_readonly() {
            list.push(Attribute::PtrReadOnly);
        }
        if self.is_ptr_nocapture() {
            list.push(Attribute::PtrNoCapture);
        }
        if self.is_func_noreturn() {
            list.push(Attribute::FuncNoReturn);
        }
        let inline = self.get_func_inline();
        if inline != InlineAttr::Normal {
            list.push(Attribute::FuncInline(inline));
        }
        if self.func_align != 3 {
            list.push(Attribute::FuncAlignStack(self.func_align));
        }
        if self.is_func_pure() {
            list.push(Attribute::FuncPure);
        }
        if let Some(target) = self.get_ptr_arg_target() {
            list.push(Attribute::ArgPtrTarget(target));
        }
        if let Some(bytes) = self.get_ptr_arg_deref_bytes() {
            list.push(Attribute::ArgPtrDerefBytes(bytes));
        }
        list.into_iter()
    }

    /// Validate positional legality of all active attributes against this set's position.
    /// Returns first encountered error or Ok(()).
    pub fn validate(&self) -> AttrRes {
        for attr in self.iter() {
            let allowed = attr.get_pos();
            if !allowed.allows(self.attr_pos) {
                return Err(AttrError::InvalidAttrPos { required: allowed, found: self.attr_pos });
            }
        }
        Ok(())
    }
}

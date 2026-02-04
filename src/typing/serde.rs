//! All types are serialized as strings so that they can be easily compared by ID.

#![cfg(feature = "serde")]

use crate::typing::*;

use serde_core::{Deserialize, Deserializer, Serialize, Serializer, de::Error};
use smol_str::SmolStr;

use std::str::FromStr;

impl Serialize for FPKind {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(self.get_name())
    }
}
impl<'de> Deserialize<'de> for FPKind {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        match FPKind::from_str(s) {
            Ok(kind) => Ok(kind),
            Err(_) => Err(Error::custom(format!("unknown FPKind '{s}'"))),
        }
    }
}

impl Serialize for IntType {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use smol_str::format_smolstr;
        let s = format_smolstr!("i{}", self.0);
        serializer.serialize_str(&s)
    }
}
impl<'de> Deserialize<'de> for IntType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        match IntType::from_str(s) {
            Ok(ity) => Ok(ity),
            Err(_) => Err(Error::custom(format!("invalid IntType '{s}'"))),
        }
    }
}

impl Serialize for PtrType {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str("ptr")
    }
}
impl<'de> Deserialize<'de> for PtrType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        if s == "ptr" { Ok(PtrType) } else { Err(Error::custom(format!("invalid PtrType '{s}'"))) }
    }
}

/// Syntax:
///
/// ```remusys-ir-transport
/// vec:<elem_type>:<len_log2>
/// ```
impl Serialize for FixVecType {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use smol_str::format_smolstr;
        let elem_str = match self.0 {
            ScalarType::Ptr => SmolStr::new_inline("ptr"),
            ScalarType::Int(bits) => format_smolstr!("i{}", bits),
            ScalarType::Float(fpkind) => SmolStr::new_inline(fpkind.get_name()),
        };
        let s = format_smolstr!("vec:{}:{}", elem_str, self.1);
        serializer.serialize_str(&s)
    }
}
impl<'de> Deserialize<'de> for FixVecType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        FixVecType::from_str(s).map_err(Error::custom)
    }
}

impl FromStr for FixVecType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let errmsg = move || format!("invalid FixVecType '{s}'");

        let Some(rest) = s.strip_prefix("vec:") else {
            return Err(errmsg());
        };
        let mut parts = rest.split(':');
        let elem_str = parts.next().ok_or_else(errmsg)?;
        let len_log2_str = parts.next().ok_or_else(errmsg)?;
        if parts.next().is_some() {
            return Err(errmsg());
        }

        let len_log2: u8 = len_log2_str.parse().map_err(|_| errmsg())?;
        match elem_str {
            "ptr" => Ok(FixVecType(ScalarType::Ptr, len_log2)),
            "float" => Ok(FixVecType(ScalarType::Float(FPKind::Ieee32), len_log2)),
            "double" => Ok(FixVecType(ScalarType::Float(FPKind::Ieee64), len_log2)),
            _ if elem_str.starts_with('i') => {
                let bits_str = &elem_str[1..];
                let bits: u8 = bits_str.parse().map_err(|_| errmsg())?;
                Ok(FixVecType(ScalarType::Int(bits), len_log2))
            }
            _ => Err(errmsg()),
        }
    }
}

#[derive(Clone, Copy)]
struct RefCodec {
    id: u32,
    kind: &'static str,
}
impl Serialize for RefCodec {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        use smol_str::format_smolstr;
        let Self { id, kind } = *self;
        serializer.serialize_str(&format_smolstr!("{kind}:{id:x}"))
    }
}
impl<'de> Deserialize<'de> for RefCodec {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        RefCodec::from_str(s).map_err(Error::custom)
    }
}

impl FromStr for RefCodec {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let errmsg = move || format!("invalid RefCodec '{s}'");
        let mut parts = s.split(':');
        let kind = parts.next().ok_or_else(errmsg)?;
        let id_str = parts.next().ok_or_else(errmsg)?;
        if parts.next().is_some() {
            return Err(errmsg());
        }
        // make it static
        let kind: &'static str = match kind {
            "struct" => "struct",
            "arr" => "arr",
            "func" => "func",
            "alias" => "alias",
            _ => return Err(errmsg()),
        };

        let id = u32::from_str_radix(id_str, 16).map_err(|_| errmsg())?;
        Ok(RefCodec { id, kind })
    }
}

impl Serialize for ScalarType {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            ScalarType::Ptr => PtrType.serialize(serializer),
            ScalarType::Int(ity) => ity.serialize(serializer),
            ScalarType::Float(fpkind) => fpkind.serialize(serializer),
        }
    }
}
impl core::str::FromStr for ScalarType {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        // fast paths for common scalar types
        match s {
            "ptr" => Ok(ScalarType::Ptr),
            "float" => Ok(ScalarType::Float(FPKind::Ieee32)),
            "double" => Ok(ScalarType::Float(FPKind::Ieee64)),
            "i1" => Ok(ScalarType::Int(1)),
            "i8" => Ok(ScalarType::Int(8)),
            "i16" => Ok(ScalarType::Int(16)),
            "i32" => Ok(ScalarType::Int(32)),
            "i64" => Ok(ScalarType::Int(64)),
            "i128" => Ok(ScalarType::Int(128)),
            s if s.starts_with('i') => {
                let bits_str = &s[1..];
                let bits: u8 = bits_str
                    .parse()
                    .map_err(|e| format!("invalid ScalarType integer bits in '{s}': {e}"))?;
                Ok(ScalarType::Int(bits))
            }
            _ => Err(format!("invalid ScalarType '{s}'")),
        }
    }
}
impl<'de> Deserialize<'de> for ScalarType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        ScalarType::from_str(s).map_err(Error::custom)
    }
}

impl Serialize for ValTypeID {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        match self {
            ValTypeID::Void => serializer.serialize_str("void"),
            ValTypeID::Ptr => PtrType.serialize(serializer),
            // syntax: `i{bits}`
            ValTypeID::Int(ity) => ity.serialize(serializer),
            // syntax: `float` | `double`
            ValTypeID::Float(fpkind) => fpkind.serialize(serializer),
            // syntax: `arr:{id:x}`
            ValTypeID::Array(arrid) => RefCodec { id: arrid.0, kind: "arr" }.serialize(serializer),
            // syntax: `struct:{id:x}`
            ValTypeID::Struct(structid) => {
                RefCodec { id: structid.0, kind: "struct" }.serialize(serializer)
            }
            // syntax: `alias:{id:x}`
            ValTypeID::StructAlias(aliasid) => {
                RefCodec { id: aliasid.0, kind: "alias" }.serialize(serializer)
            }
            // syntax: `vec:{elem_type}:{len_log2}`
            ValTypeID::FixVec(fv) => fv.serialize(serializer),
            // syntax: `func:{id:x}`
            ValTypeID::Func(funcid) => {
                RefCodec { id: funcid.0, kind: "func" }.serialize(serializer)
            }
        }
    }
}
impl<'de> Deserialize<'de> for ValTypeID {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s: &str = Deserialize::deserialize(deserializer)?;
        ValTypeID::from_str(s).map_err(Error::custom)
    }
}

impl FromStr for ValTypeID {
    type Err = String;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if s == "void" {
            return Ok(ValTypeID::Void);
        }
        if let Ok(ty) = ScalarType::from_str(s) {
            return Ok(ty.into_ir());
        }
        if s.starts_with("vec:") {
            return FixVecType::from_str(s).map(ValTypeID::FixVec);
        }
        if let Ok(codec) = RefCodec::from_str(s) {
            return match codec.kind {
                "arr" => Ok(ValTypeID::Array(ArrayTypeID(codec.id))),
                "struct" => Ok(ValTypeID::Struct(StructTypeID(codec.id))),
                "alias" => Ok(ValTypeID::StructAlias(StructAliasID(codec.id))),
                "func" => Ok(ValTypeID::Func(FuncTypeID(codec.id))),
                _ => Err(format!("invalid ValTypeID kind '{}'", codec.kind)),
            };
        }
        Err(format!("invalid ValTypeID '{s}'"))
    }
}

impl Serialize for AggrType {
    fn serialize<S: Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        self.into_ir().serialize(serializer)
    }
}
impl<'de> Deserialize<'de> for AggrType {
    fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
    where
        D: Deserializer<'de>,
    {
        let tyid: ValTypeID = Deserialize::deserialize(deserializer)?;
        AggrType::try_from_ir(tyid).map_err(Error::custom)
    }
}

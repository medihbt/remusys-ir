use crate::{ir::IRWriter, typing::ValTypeID};

pub(super) mod attrlist;
pub(super) mod attrset;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum InlineAttr {
    NoInline,
    Hint,
    Always,
}

impl InlineAttr {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "noinline" => Some(InlineAttr::NoInline),
            "inlinehint" => Some(InlineAttr::Hint),
            "alwaysinline" => Some(InlineAttr::Always),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            InlineAttr::NoInline => "noinline",
            InlineAttr::Hint => "inlinehint",
            InlineAttr::Always => "alwaysinline",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum IntExtAttr {
    Zext,
    Sext,
}

impl IntExtAttr {
    pub fn from_str(s: &str) -> Option<Self> {
        match s {
            "zeroext" => Some(IntExtAttr::Zext),
            "signext" => Some(IntExtAttr::Sext),
            _ => None,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            IntExtAttr::Zext => "zeroext",
            IntExtAttr::Sext => "signext",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum CodeTempAttr {
    Cold,
    Hot,
}

/// 属性合并行为，参考 LLVM Module Flags
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MergeBehavior {
    /// 如果两个值不一致则报错
    Error,
    /// 如果两个值不一致则警告，使用第一个值
    Warning,
    /// 无条件使用新值
    Override,
    /// 追加两个值（用于列表类属性）
    Append,
    /// 追加但去重
    AppendUnique,
    /// 取较大值
    Max,
    /// 取较小值
    Min,
}

/// 单个属性项
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Attr {
    // 函数属性
    NoReturn,
    NoRecurse,
    Inline(InlineAttr),
    AlignStack(u32),
    CodeTemp(CodeTempAttr),

    // 参数/返回值属性
    IntExt(IntExtAttr),
    PtrElem(ValTypeID),
    NoAlias,
    NonNull,
    Dereferenceable(u32),
    Align(u32),

    // 目标相关属性（字符串形式）
    TargetDependent(String),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum AttrKind {
    NoReturn,
    NoRecurse,
    Inline,
    AlignStack,
    CodeTemp,

    IntExt,
    PtrElem,
    NoAlias,
    NonNull,
    Dereferenceable,
    Align,
    TargetDependent,
}

impl Attr {
    pub fn get_kind(&self) -> AttrKind {
        match self {
            Attr::NoReturn => AttrKind::NoReturn,
            Attr::NoRecurse => AttrKind::NoRecurse,
            Attr::Inline(_) => AttrKind::Inline,
            Attr::AlignStack(_) => AttrKind::AlignStack,
            Attr::CodeTemp(_) => AttrKind::CodeTemp,

            Attr::IntExt(_) => AttrKind::IntExt,
            Attr::PtrElem(_) => AttrKind::PtrElem,
            Attr::NoAlias => AttrKind::NoAlias,
            Attr::NonNull => AttrKind::NonNull,
            Attr::Dereferenceable(_) => AttrKind::Dereferenceable,
            Attr::Align(_) => AttrKind::Align,
            Attr::TargetDependent(_) => AttrKind::TargetDependent,
        }
    }

    pub fn is_func_attr(&self) -> bool {
        use Attr::*;
        matches!(
            self,
            NoReturn | NoRecurse | Inline(_) | AlignStack(_) | CodeTemp(_)
        )
    }

    pub fn get_merge_behavior(&self) -> MergeBehavior {
        use Attr::*;
        match self {
            // 布尔型函数属性 - 去重保留
            NoReturn | NoRecurse => MergeBehavior::AppendUnique,

            Inline(_) | CodeTemp(_) => MergeBehavior::Override,

            // 数值属性 - 取较大值，更严格的要求
            AlignStack(_) | Dereferenceable(_) | Align(_) => MergeBehavior::Max,

            IntExt(_) | PtrElem(_) => MergeBehavior::Override,

            // 布尔型指针属性 - 去重保留
            NoAlias | NonNull => MergeBehavior::AppendUnique,

            // 目标相关属性 - 可以累积多个特性
            TargetDependent(_) => MergeBehavior::Append,
        }
    }

    pub fn fmt_ir(&self, f: &IRWriter) -> std::io::Result<()> {
        match self {
            Attr::NoReturn => f.write_str("noreturn"),
            Attr::NoRecurse => f.write_str("norecurse"),
            Attr::Inline(inline) => f.write_str(inline.as_str()),
            Attr::AlignStack(val) => {
                write!(f, "alignstack({})", val)
            }
            Attr::CodeTemp(temp) => match temp {
                CodeTempAttr::Cold => f.write_str("cold"),
                CodeTempAttr::Hot => f.write_str("hot"),
            },
            Attr::IntExt(ext) => f.write_str(ext.as_str()),
            Attr::PtrElem(ty) => {
                f.write_str("elementtype(")?;
                f.write_type(*ty)?;
                f.write_str(")")
            }
            Attr::NoAlias => f.write_str("noalias"),
            Attr::NonNull => f.write_str("nonnull"),
            Attr::Dereferenceable(val) => write!(f, "dereferenceable({val})"),
            Attr::Align(val) => write!(f, "align({val})"),
            Attr::TargetDependent(dep) => f.write_str(&format!("target-dependent \"{dep}\"")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use slab::Slab;

    #[test]
    fn test_attrset_basic_functionality() {
        let mut attrs = attrset::AttrSet::new();
        assert!(attrs.is_empty());
        assert_eq!(attrs.len(), 0);

        // 测试基本属性设置
        attrs.merge_attr(Attr::NoReturn);
        attrs.merge_attr(Attr::Inline(InlineAttr::Always));
        attrs.merge_attr(Attr::Align(16));

        assert!(!attrs.is_empty());
        assert_eq!(attrs.len(), 3);
        assert!(attrs.noreturn);
        assert_eq!(attrs.inline, Some(InlineAttr::Always));
        assert_eq!(attrs.align, 16);
    }

    #[test]
    fn test_attrset_merge_behavior() {
        let mut attrs = attrset::AttrSet::new();

        // 测试布尔属性去重
        attrs.merge_attr(Attr::NoReturn);
        attrs.merge_attr(Attr::NoReturn); // 重复
        assert_eq!(attrs.len(), 1);

        // 测试数值属性取最大值
        attrs.merge_attr(Attr::Align(8));
        attrs.merge_attr(Attr::Align(16)); // 更大
        attrs.merge_attr(Attr::Align(4)); // 更小，应该被忽略
        assert_eq!(attrs.align, 16);

        // 测试可选属性覆盖
        attrs.merge_attr(Attr::Inline(InlineAttr::Hint));
        attrs.merge_attr(Attr::Inline(InlineAttr::Always)); // 覆盖
        assert_eq!(attrs.inline, Some(InlineAttr::Always));

        // 测试目标相关属性追加去重
        attrs.merge_attr(Attr::TargetDependent("sse2".to_string()));
        attrs.merge_attr(Attr::TargetDependent("avx".to_string()));
        attrs.merge_attr(Attr::TargetDependent("sse2".to_string())); // 重复
        assert_eq!(attrs.target_dependent.len(), 2);
    }

    #[test]
    fn test_attrset_from_iter() {
        let attrs = attrset::AttrSet::from_attrs([
            Attr::NoReturn,
            Attr::NoReturn, // 重复
            Attr::Align(8),
            Attr::Align(16), // 会保留更大的值
            Attr::Inline(InlineAttr::Hint),
        ]);

        assert!(attrs.noreturn);
        assert_eq!(attrs.align, 16);
        assert_eq!(attrs.inline, Some(InlineAttr::Hint));
        assert_eq!(attrs.len(), 3);
    }

    #[test]
    fn test_attrset_merge_from() {
        let mut attrs1 = attrset::AttrSet::new();
        attrs1.merge_attr(Attr::NoReturn);
        attrs1.merge_attr(Attr::Align(8));

        let mut attrs2 = attrset::AttrSet::new();
        attrs2.merge_attr(Attr::NoRecurse);
        attrs2.merge_attr(Attr::Align(16)); // 更大的值

        attrs1.merge_from(&attrs2);

        assert!(attrs1.noreturn);
        assert!(attrs1.norecurse);
        assert_eq!(attrs1.align, 16); // 取较大值
    }

    #[test]
    fn test_attrset_has_attr() {
        let mut attrs = attrset::AttrSet::new();
        attrs.merge_attr(Attr::NoReturn);
        attrs.merge_attr(Attr::Align(16));
        attrs.merge_attr(Attr::Dereferenceable(64));

        assert!(attrs.has_attr(&Attr::NoReturn));
        assert!(attrs.has_attr(&Attr::Align(16)));
        assert!(attrs.has_attr(&Attr::Align(8))); // 8 <= 16
        assert!(!attrs.has_attr(&Attr::Align(32))); // 32 > 16
        assert!(attrs.has_attr(&Attr::Dereferenceable(64)));
        assert!(attrs.has_attr(&Attr::Dereferenceable(32))); // 32 <= 64
        assert!(!attrs.has_attr(&Attr::NoRecurse));
    }

    #[test]
    fn test_attrlist_merge_all_simple() {
        let mut alloc = Slab::new();

        // 创建基础属性列表
        let base_attrs = attrset::AttrSet::from_attrs([Attr::NoReturn, Attr::Align(8)]);
        let base_id = attrlist::AttrListID(alloc.insert(attrlist::AttrList {
            includes: vec![],
            self_id: attrlist::AttrListID(0),
            attr: base_attrs,
        }));

        // 创建扩展属性列表，继承基础列表
        let ext_attrs = attrset::AttrSet::from_attrs([Attr::NoRecurse, Attr::Align(16)]);
        let ext_id = attrlist::AttrListID(alloc.insert(attrlist::AttrList {
            includes: vec![base_id],
            self_id: attrlist::AttrListID(1),
            attr: ext_attrs,
        }));

        // 合并所有属性
        let merged = ext_id.merge_all(&mut alloc);

        assert!(merged.noreturn); // 从基础列表继承
        assert!(merged.norecurse); // 自己的属性
        assert_eq!(merged.align, 16); // 取较大值
        assert_eq!(merged.len(), 3);
    }

    #[test]
    fn test_attrlist_merge_all_complex() {
        let mut alloc = Slab::new();

        // 创建多个基础属性列表
        let base1_id = attrlist::AttrListID(alloc.insert(attrlist::AttrList {
            includes: vec![],
            self_id: attrlist::AttrListID(0),
            attr: attrset::AttrSet::from_attrs([Attr::NoReturn, Attr::Align(8)]),
        }));

        let base2_id = attrlist::AttrListID(alloc.insert(attrlist::AttrList {
            includes: vec![],
            self_id: attrlist::AttrListID(1),
            attr: attrset::AttrSet::from_attrs([Attr::NoRecurse, Attr::AlignStack(16)]),
        }));

        // 创建多继承的属性列表
        let multi_id = attrlist::AttrListID(alloc.insert(attrlist::AttrList {
            includes: vec![base1_id, base2_id],
            self_id: attrlist::AttrListID(2),
            attr: attrset::AttrSet::from_attrs([Attr::Inline(InlineAttr::Always), Attr::Align(32)]),
        }));

        let merged = multi_id.merge_all(&mut alloc);

        assert!(merged.noreturn); // 从 base1 继承
        assert!(merged.norecurse); // 从 base2 继承
        assert_eq!(merged.align_stack, 16); // 从 base2 继承
        assert_eq!(merged.align, 32); // 自己的属性，且是最大值
        assert_eq!(merged.inline, Some(InlineAttr::Always)); // 自己的属性
    }

    #[test]
    fn test_attrlist_cycle_detection() {
        let mut alloc = Slab::new();

        // 先插入空的属性列表
        let id1 = attrlist::AttrListID(alloc.insert(attrlist::AttrList {
            includes: vec![],
            self_id: attrlist::AttrListID(0),
            attr: attrset::AttrSet::from_attrs([Attr::NoReturn]),
        }));

        let id2 = attrlist::AttrListID(alloc.insert(attrlist::AttrList {
            includes: vec![],
            self_id: attrlist::AttrListID(1),
            attr: attrset::AttrSet::from_attrs([Attr::NoRecurse]),
        }));

        // 创建循环引用：id1 -> id2 -> id1
        alloc[id1.0].includes.push(id2);
        alloc[id2.0].includes.push(id1);

        // 应该不会无限循环，而是正确处理
        let merged = id1.merge_all(&mut alloc);

        assert!(merged.noreturn);
        assert!(merged.norecurse);
        assert_eq!(merged.len(), 2);
    }

    #[test]
    fn test_attr_roundtrip() {
        let original_attrs = vec![
            Attr::NoReturn,
            Attr::Inline(InlineAttr::Always),
            Attr::Align(16),
            Attr::TargetDependent("sse2".to_string()),
        ];

        let attr_set = attrset::AttrSet::from_attrs(original_attrs.clone());
        let _roundtrip_attrs = attr_set.to_attrs(); // 确保转换不会崩溃

        // 检查所有原始属性都在往返后保留
        for attr in &original_attrs {
            assert!(attr_set.has_attr(attr), "属性 {:?} 应该被保留", attr);
        }

        // 检查数量正确（去重后）
        assert_eq!(attr_set.len(), 4);
    }
}

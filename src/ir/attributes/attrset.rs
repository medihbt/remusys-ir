use crate::{
    ir::{
        IRWriter,
        attributes::{Attr, CodeTempAttr, InlineAttr, IntExtAttr},
    },
    typing::ValTypeID,
};

#[derive(Debug, Clone, PartialEq, Eq, Default)]
pub struct AttrSet {
    pub noreturn: bool,
    pub norecurse: bool,
    pub inline: Option<InlineAttr>,
    pub align_stack: u32,
    pub code_temp: Option<CodeTempAttr>,

    pub int_ext: Option<IntExtAttr>,
    pub ptrelem: Option<ValTypeID>,
    pub noalias: bool,
    pub nonnull: bool,
    pub dereferenceable: Option<u32>,
    pub align: u32,
    pub target_dependent: Vec<String>,
}

impl AttrSet {
    pub fn new() -> Self {
        Self::default()
    }

    /// 将单个属性合并到 map 中
    pub fn merge_attr(&mut self, attr: Attr) {
        match attr {
            Attr::NoReturn => self.noreturn = true,
            Attr::NoRecurse => self.norecurse = true,
            Attr::Inline(inline) => self.inline = Some(inline),
            Attr::AlignStack(val) => self.align_stack = self.align_stack.max(val),
            Attr::CodeTemp(temp) => self.code_temp = Some(temp),

            Attr::IntExt(ext) => self.int_ext = Some(ext),
            Attr::PtrElem(ty) => self.ptrelem = Some(ty),
            Attr::NoAlias => self.noalias = true,
            Attr::NonNull => self.nonnull = true,
            Attr::Dereferenceable(val) => {
                self.dereferenceable = Some(match self.dereferenceable {
                    Some(existing) => existing.max(val),
                    None => val,
                });
            }
            Attr::Align(val) => self.align = self.align.max(val),
            Attr::TargetDependent(dep) => {
                let dep = dep.into();
                if !self.target_dependent.contains(&dep) {
                    self.target_dependent.push(dep);
                }
            }
        }
    }

    /// 从属性列表创建 AttrMergeMap
    pub fn from_attrs<'a>(attrs: impl IntoIterator<Item = Attr<'a>>) -> Self {
        let mut map = Self::default();
        for attr in attrs {
            map.merge_attr(attr);
        }
        map
    }

    pub fn merge_from(&mut self, other: &Self) {
        self.noreturn |= other.noreturn;
        self.norecurse |= other.norecurse;
        self.inline = self.inline.or(other.inline);
        self.align_stack = self.align_stack.max(other.align_stack);
        self.code_temp = self.code_temp.or(other.code_temp);

        self.int_ext = self.int_ext.or(other.int_ext);
        self.ptrelem = self.ptrelem.or(other.ptrelem);
        self.noalias |= other.noalias;
        self.nonnull |= other.nonnull;
        self.dereferenceable = self.dereferenceable.or(other.dereferenceable);
        self.align = self.align.max(other.align);
        self.target_dependent
            .extend(other.target_dependent.iter().cloned());
    }
    pub fn merge(self, other: &Self) -> Self {
        let mut new = self.clone();
        new.merge_from(other);
        new
    }

    /// 转换回属性列表（用于序列化等）
    pub fn to_attrs(&self) -> Vec<Attr<'_>> {
        let mut attrs = Vec::new();

        if self.noreturn {
            attrs.push(Attr::NoReturn);
        }
        if self.norecurse {
            attrs.push(Attr::NoRecurse);
        }
        if let Some(inline) = self.inline {
            attrs.push(Attr::Inline(inline));
        }
        if self.align_stack > 0 {
            attrs.push(Attr::AlignStack(self.align_stack));
        }
        if let Some(temp) = self.code_temp {
            attrs.push(Attr::CodeTemp(temp));
        }

        if let Some(ext) = self.int_ext {
            attrs.push(Attr::IntExt(ext));
        }
        if let Some(ty) = self.ptrelem {
            attrs.push(Attr::PtrElem(ty));
        }
        if self.noalias {
            attrs.push(Attr::NoAlias);
        }
        if self.nonnull {
            attrs.push(Attr::NonNull);
        }
        if let Some(val) = self.dereferenceable {
            attrs.push(Attr::Dereferenceable(val));
        }
        if self.align > 0 {
            attrs.push(Attr::Align(self.align));
        }

        for dep in &self.target_dependent {
            attrs.push(Attr::TargetDependent(dep.as_str()));
        }

        attrs
    }

    /// 检查是否包含特定属性
    pub fn has_attr(&self, attr: &Attr) -> bool {
        match attr {
            Attr::NoReturn => self.noreturn,
            Attr::NoRecurse => self.norecurse,
            Attr::Inline(inline) => self.inline == Some(*inline),
            Attr::AlignStack(val) => self.align_stack >= *val,
            Attr::CodeTemp(temp) => self.code_temp == Some(*temp),

            Attr::IntExt(ext) => self.int_ext == Some(*ext),
            Attr::PtrElem(ty) => self.ptrelem == Some(*ty),
            Attr::NoAlias => self.noalias,
            Attr::NonNull => self.nonnull,
            Attr::Dereferenceable(val) => self.dereferenceable.map_or(false, |d| d >= *val),
            Attr::Align(val) => self.align >= *val,
            Attr::TargetDependent(dep) => self.target_dependent.contains(&dep.to_string()),
        }
    }

    pub fn fmt_ir(&self, f: &IRWriter) -> std::io::Result<()> {
        let attrs = self.to_attrs();
        for (i, attr) in attrs.iter().enumerate() {
            if i > 0 {
                f.write_str(" ")?;
            }
            attr.fmt_ir(f)?;
        }
        Ok(())
    }

    /// 检查属性集是否为空
    pub fn is_empty(&self) -> bool {
        !self.noreturn
            && !self.norecurse
            && self.inline.is_none()
            && self.align_stack == 0
            && self.code_temp.is_none()
            && self.int_ext.is_none()
            && self.ptrelem.is_none()
            && !self.noalias
            && !self.nonnull
            && self.dereferenceable.is_none()
            && self.align == 0
            && self.target_dependent.is_empty()
    }

    /// 计算非空属性的数量
    pub fn len(&self) -> usize {
        let mut count = 0;
        if self.noreturn {
            count += 1;
        }
        if self.norecurse {
            count += 1;
        }
        if self.inline.is_some() {
            count += 1;
        }
        if self.align_stack > 0 {
            count += 1;
        }
        if self.code_temp.is_some() {
            count += 1;
        }
        if self.int_ext.is_some() {
            count += 1;
        }
        if self.ptrelem.is_some() {
            count += 1;
        }
        if self.noalias {
            count += 1;
        }
        if self.nonnull {
            count += 1;
        }
        if self.dereferenceable.is_some() {
            count += 1;
        }
        if self.align > 0 {
            count += 1;
        }
        count += self.target_dependent.len();
        count
    }
}

impl<'a> FromIterator<Attr<'a>> for AttrSet {
    fn from_iter<T: IntoIterator<Item = Attr<'a>>>(iter: T) -> Self {
        Self::from_attrs(iter)
    }
}

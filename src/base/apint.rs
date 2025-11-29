use std::ops::{
    Add, AddAssign, BitAnd, BitAndAssign, BitOr, BitOrAssign, BitXor, BitXorAssign, Div, DivAssign,
    Mul, MulAssign, Neg, Not, Rem, RemAssign, Shl, ShlAssign, Shr, Sub, SubAssign,
};

#[derive(Clone, Copy, PartialEq, Eq, Hash)]
pub struct APInt {
    value: [u32; 4],
    bits: u8,
}
impl std::fmt::Debug for APInt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let val = self.as_unsigned();
        let bits = self.bits;
        write!(f, "APInt({val:#x}:{bits})")
    }
}

impl APInt {
    pub fn new<T: IIntoU128>(value: T, bits: u8) -> Self {
        let value = value.into_u128();
        let bitmask = if bits == 128 { u128::MAX } else { (1u128 << bits) - 1 };
        Self { value: Self::split_u128(value & bitmask), bits }
    }
    pub const fn new_full(value: u128, bits: u8) -> Self {
        let bitmask = if bits == 128 { u128::MAX } else { (1u128 << bits) - 1 };
        Self { value: Self::split_u128(value & bitmask), bits }
    }
    pub const fn is_zero(&self) -> bool {
        self.value_raw() == 0
    }
    pub const fn is_nonzero(&self) -> bool {
        self.value_raw() != 0
    }

    const fn split_u128(x: u128) -> [u32; 4] {
        [x as u32, (x >> 32) as u32, (x >> 64) as u32, (x >> 96) as u32]
    }
    const fn join_u128(parts: [u32; 4]) -> u128 {
        let [p0, p1, p2, p3] = parts;
        let p0 = p0 as u128;
        let p1 = p1 as u128;
        let p2 = p2 as u128;
        let p3 = p3 as u128;
        p0 | (p1 << 32) | (p2 << 64) | (p3 << 96)
    }
    const fn value_raw(&self) -> u128 {
        Self::join_u128(self.value)
    }
    #[allow(dead_code)]
    fn set_value_raw(&mut self, value: u128) {
        self.value = Self::split_u128(value);
    }

    pub const fn as_unsigned(&self) -> u128 {
        self.value_raw()
    }
    pub const fn as_signed(&self) -> i128 {
        let value_raw = self.value_raw();
        match self.bits {
            8 => value_raw as i8 as i128,
            16 => value_raw as i16 as i128,
            32 => value_raw as i32 as i128,
            64 => value_raw as i64 as i128,
            128 => value_raw as i128,
            _ => {
                if value_raw & self.sign_bitmask() == 0 {
                    value_raw as i128
                } else {
                    (value_raw | self.signed_bitmask()) as i128
                }
            }
        }
    }

    pub const fn is_negative(&self) -> bool {
        let bits = self.as_unsigned();
        (bits & self.sign_bitmask()) != 0
    }
    /// 是否是该位宽下的最小负数
    pub const fn is_min_negative(&self) -> bool {
        self.as_unsigned() == self.sign_bitmask()
    }

    pub const fn signed_bitmask(&self) -> u128 {
        if self.bits == 0 || self.bits >= 128 {
            return 0;
        }
        // 创建符号扩展掩码，填充高位的1
        !((1u128 << self.bits) - 1)
    }
    pub const fn sign_bitmask(&self) -> u128 {
        if self.bits == 0 { 0 } else { 1 << (self.bits - 1) }
    }

    pub const fn bits(&self) -> u8 {
        self.bits
    }

    pub fn zext_to(&self, bits: u8) -> Self {
        Self::new(self.value_raw(), bits)
    }
    fn zext_as<T>(&self) -> Self {
        self.zext_to(core::mem::size_of::<T>() as u8 * 8)
    }

    pub fn sext_to(&self, bits: u8) -> Self {
        if bits <= self.bits {
            return *self;
        }
        let signed_value = self.as_signed();
        Self::new(signed_value as u128, bits)
    }
    fn sext_as<T>(&self) -> Self {
        self.sext_to(core::mem::size_of::<T>() as u8 * 8)
    }

    pub fn zext_with(&self, rhs: APInt) -> Self {
        self.zext_to(rhs.bits)
    }
    pub fn sext_with(&self, rhs: APInt) -> Self {
        self.sext_to(rhs.bits)
    }

    pub fn sdiv(self, other: APInt) -> Self {
        if self.bits != other.bits {
            panic!("Cannot divide APInts with different bit widths");
        }
        if other.is_zero() {
            panic!("Division by zero in APInt");
        }
        let result = self.as_signed() / other.as_signed();
        Self::new(result as u128, self.bits)
    }
    pub fn udiv(self, other: APInt) -> Self {
        if self.bits != other.bits {
            panic!("Cannot divide APInts with different bit widths");
        }
        if other.is_zero() {
            panic!("Division by zero in APInt");
        }
        let result = self.as_unsigned() / other.as_unsigned();
        Self::new(result, self.bits)
    }

    pub fn srem(self, other: APInt) -> Self {
        if self.bits != other.bits {
            panic!("Cannot divide APInts with different bit widths");
        }
        if other.is_zero() {
            panic!("Division by zero in APInt");
        }
        let result = self.as_signed() % other.as_signed();
        Self::new(result as u128, self.bits)
    }
    pub fn urem(self, other: APInt) -> Self {
        if self.bits != other.bits {
            panic!("Cannot divide APInts with different bit widths");
        }
        if other.is_zero() {
            panic!("Division by zero in APInt");
        }
        let result = self.as_unsigned() % other.as_unsigned();
        Self::new(result, self.bits)
    }
    pub fn is_boolean(&self) -> bool {
        self.bits == 1
    }

    pub fn lshr(self, shift: impl IIntoU128) -> Self {
        let shift = shift.into_u128() as u64;
        if shift >= 128 {
            return Self::new(0u128, self.bits);
        }
        let value = self.as_unsigned();
        Self::new(value.wrapping_shr(shift as u32), self.bits)
    }
    pub fn ashr(self, shift: impl IIntoU128) -> Self {
        let shift = shift.into_u128() as u64;
        if shift >= 128 {
            if self.is_negative() {
                return Self::new_full(self.signed_bitmask(), self.bits);
            } else {
                return Self::new(0u128, self.bits);
            }
        }
        let value = self.as_signed();
        Self::new((value.wrapping_shr(shift as u32)) as u128, self.bits)
    }

    pub fn lshr_with(self, rhs: impl Into<APInt>) -> Self {
        self.lshr(rhs.into().as_unsigned() as u8)
    }
    pub fn ashr_with(self, rhs: impl Into<APInt>) -> Self {
        self.ashr(rhs.into().as_unsigned() as u8)
    }

    pub fn rotate_left(self, shift: APInt) -> Self {
        let shift = shift.as_unsigned() % (self.bits as u128);
        if shift == 0 {
            return self;
        }
        let value = self.value_raw();
        let rotated = (value << shift) | (value >> (self.bits as u128 - shift));
        Self::new(rotated, self.bits)
    }
    pub fn rotate_right(self, shift: APInt) -> Self {
        let shift = shift.as_unsigned() % (self.bits as u128);
        if shift == 0 {
            return self;
        }
        let value = self.value_raw();
        let rotated = (value >> shift) | (value << (self.bits as u128 - shift));
        Self::new(rotated, self.bits)
    }

    pub const fn as_power_of_two(&self) -> Option<u32> {
        let value = self.as_unsigned();
        if value.is_power_of_two() { Some(value.trailing_zeros()) } else { None }
    }
    pub const fn is_power_of_two(&self) -> bool {
        self.as_unsigned().is_power_of_two()
    }

    pub const fn count_ones(&self) -> u32 {
        self.as_unsigned().count_ones()
    }
    pub const fn count_zeros(&self) -> u32 {
        self.bits as u32 - self.count_ones()
    }

    pub const fn trailing_zeros(&self) -> u32 {
        let value = self.as_unsigned();
        if value == 0 { self.bits as u32 } else { value.trailing_zeros() }
    }
    pub const fn trailing_ones(&self) -> u32 {
        self.as_unsigned().trailing_ones()
    }
    pub const fn leading_zeros(&self) -> u32 {
        let value = self.as_unsigned() << (128 - self.bits as u32);
        value.leading_zeros()
    }
    pub const fn leading_ones(&self) -> u32 {
        let value = self.as_unsigned() << (128 - self.bits as u32);
        value.leading_ones()
    }
}

impl From<bool> for APInt {
    fn from(value: bool) -> Self {
        Self::new(if value { 1u128 } else { 0u128 }, 1)
    }
}

macro_rules! impl_from_ints {
    ($($t:ty),+) => {
        $(
            impl From<$t> for APInt {
                fn from(value: $t) -> Self {
                    Self::new(value as u128, core::mem::size_of::<$t>() as u8 * 8)
                }
            }
        )+
    };
}

impl_from_ints!(
    u8, u16, u32, u64, u128, i8, i16, i32, i64, i128, usize, isize
);

impl std::fmt::Display for APInt {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let bits = self.bits();
        let value = self.as_unsigned();
        if bits == 0 { write!(f, "i0") } else { write!(f, "i{bits}({value})") }
    }
}

impl Add<APInt> for APInt {
    type Output = Self;

    fn add(self, other: APInt) -> APInt {
        if self.bits != other.bits {
            panic!("Cannot add APInts with different bit widths");
        }
        let lval = self.value_raw();
        let rval = other.value_raw();
        Self::new(lval.wrapping_add(rval), self.bits)
    }
}

impl AddAssign<APInt> for APInt {
    fn add_assign(&mut self, other: APInt) {
        *self = self.add(other);
    }
}

impl Sub<APInt> for APInt {
    type Output = Self;

    fn sub(self, other: APInt) -> APInt {
        if self.bits != other.bits {
            panic!("Cannot subtract APInts with different bit widths");
        }
        let lval = self.value_raw();
        let rval = other.value_raw();
        Self::new(lval.wrapping_sub(rval), self.bits)
    }
}

impl SubAssign<APInt> for APInt {
    fn sub_assign(&mut self, other: APInt) {
        *self = self.sub(other);
    }
}

impl Neg for APInt {
    type Output = Self;

    fn neg(self) -> Self {
        let [mut u0, mut u1, u2, u3] = self.value;
        match self.bits {
            1 => return self,
            8 => u0 = ((u0 as i8).wrapping_neg() as u8) as u32,
            16 => u0 = ((u0 as i16).wrapping_neg() as u16) as u32,
            32 => u0 = (u0 as i32).wrapping_neg() as u32,
            64 => {
                let d0 = ((u1 as u64) << 32) | (u0 as u64);
                let nh = (d0 as i64).wrapping_neg() as u64;
                u0 = nh as u32;
                u1 = (nh >> 32) as u32;
            }
            128 => {
                let q0 = self.as_unsigned() as i128;
                return Self::new_full(q0.wrapping_neg() as u128, self.bits);
            }
            _ => return Self::new((self.as_signed().wrapping_neg()) as u128, self.bits),
        }
        Self { value: [u0, u1, u2, u3], bits: self.bits }
    }
}

impl Mul<APInt> for APInt {
    type Output = Self;

    fn mul(self, other: APInt) -> APInt {
        if self.bits != other.bits {
            panic!("Cannot multiply APInts with different bit widths");
        }
        let lval = self.value_raw();
        let rval = other.value_raw();
        Self::new(lval.wrapping_mul(rval), self.bits)
    }
}

impl MulAssign<APInt> for APInt {
    fn mul_assign(&mut self, other: APInt) {
        *self = self.mul(other);
    }
}

impl BitAnd<APInt> for APInt {
    type Output = Self;

    fn bitand(self, other: APInt) -> Self {
        if self.bits != other.bits {
            panic!("Cannot bitwise AND APInts with different bit widths");
        }
        let [l0, l1, l2, l3] = self.value;
        let [r0, r1, r2, r3] = other.value;
        let res = [l0 & r0, l1 & r1, l2 & r2, l3 & r3];
        Self { value: res, bits: self.bits }
    }
}

impl BitAndAssign<APInt> for APInt {
    fn bitand_assign(&mut self, other: APInt) {
        *self = self.bitand(other);
    }
}

impl BitOr<APInt> for APInt {
    type Output = Self;

    fn bitor(self, other: APInt) -> Self {
        if self.bits != other.bits {
            panic!("Cannot bitwise OR APInts with different bit widths");
        }
        let [l0, l1, l2, l3] = self.value;
        let [r0, r1, r2, r3] = other.value;
        let res = [l0 | r0, l1 | r1, l2 | r2, l3 | r3];
        Self { value: res, bits: self.bits }
    }
}

impl BitOrAssign<APInt> for APInt {
    fn bitor_assign(&mut self, other: APInt) {
        *self = self.bitor(other);
    }
}

impl BitXor<APInt> for APInt {
    type Output = Self;

    fn bitxor(self, other: APInt) -> Self {
        if self.bits != other.bits {
            panic!("Cannot bitwise XOR APInts with different bit widths");
        }
        let [l0, l1, l2, l3] = self.value;
        let [r0, r1, r2, r3] = other.value;
        let res = [l0 ^ r0, l1 ^ r1, l2 ^ r2, l3 ^ r3];
        Self { value: res, bits: self.bits }
    }
}

impl BitXorAssign<APInt> for APInt {
    fn bitxor_assign(&mut self, other: APInt) {
        *self = self.bitxor(other);
    }
}

impl Not for APInt {
    type Output = APInt;

    fn not(self) -> Self {
        let Self { value: [v0, v1, v2, v3], bits } = self;
        Self { value: [!v0, !v1, !v2, !v3], bits }
    }
}

impl Shl<APInt> for APInt {
    type Output = Self;

    fn shl(self, other: APInt) -> Self {
        let shift = other.as_unsigned() as u64;
        if shift >= 128 {
            return Self::new(0u128, self.bits);
        }
        let value = self.value_raw();
        Self::new(value.wrapping_shl(shift as u32), self.bits)
    }
}
impl ShlAssign<APInt> for APInt {
    fn shl_assign(&mut self, other: APInt) {
        *self = self.shl(other);
    }
}

macro_rules! impl_add_from_uints {
    ($($t:ty),+) => {
        $(
            impl Add<APInt> for $t {
                type Output = APInt;

                fn add(self, other: APInt) -> APInt {
                    let rhs = other.zext_as::<$t>();
                    APInt::new(
                        (self as u128).wrapping_add(rhs.as_unsigned()),
                        rhs.bits
                    )
                }
            }

            impl Add<$t> for APInt {
                type Output = APInt;

                fn add(self, other: $t) -> APInt {
                    let rhs = APInt::from(other).zext_to(self.bits);
                    self + rhs
                }
            }

            impl AddAssign<$t> for APInt {
                fn add_assign(&mut self, other: $t) {
                    let rhs = APInt::from(other).zext_to(self.bits);
                    *self += rhs;
                }
            }

            impl Sub<APInt> for $t {
                type Output = APInt;

                fn sub(self, other: APInt) -> APInt {
                    let rhs = other.zext_as::<$t>();
                    APInt::new(
                        (self as u128).wrapping_sub(rhs.as_unsigned()),
                        rhs.bits
                    )
                }
            }

            impl Sub<$t> for APInt {
                type Output = APInt;

                fn sub(self, other: $t) -> APInt {
                    let rhs = APInt::from(other).zext_to(self.bits);
                    self - rhs
                }
            }

            impl SubAssign<$t> for APInt {
                fn sub_assign(&mut self, other: $t) {
                    let rhs = APInt::from(other).zext_to(self.bits);
                    *self -= rhs;
                }
            }

            impl Mul<APInt> for $t {
                type Output = APInt;

                fn mul(self, other: APInt) -> APInt {
                    let rhs = other.zext_as::<$t>();
                    APInt::new(
                        self as u128 * rhs.as_unsigned(),
                        rhs.bits
                    )
                }
            }

            impl Mul<$t> for APInt {
                type Output = APInt;

                fn mul(self, other: $t) -> APInt {
                    let rhs = APInt::from(other).zext_to(self.bits);
                    self * rhs
                }
            }

            impl MulAssign<$t> for APInt {
                fn mul_assign(&mut self, other: $t) {
                    let rhs = APInt::from(other).zext_to(self.bits);
                    *self *= rhs;
                }
            }

            impl Div<APInt> for $t {
                type Output = APInt;

                fn div(self, other: APInt) -> APInt {
                    let rhs = other.zext_as::<$t>();
                    if rhs.is_zero() {
                        panic!("Division by zero in APInt");
                    }
                    APInt::new_full(
                        (self as u128).wrapping_div(rhs.as_unsigned()),
                        rhs.bits
                    )
                }
            }
            impl Div<$t> for APInt {
                type Output = APInt;

                fn div(self, other: $t) -> APInt {
                    let rhs = APInt::from(other).zext_to(self.bits);
                    self.udiv(rhs)
                }
            }

            impl DivAssign<$t> for APInt {
                fn div_assign(&mut self, other: $t) {
                    let rhs = APInt::from(other).zext_to(self.bits);
                    *self = self.udiv(rhs);
                }
            }

            impl Rem<APInt> for $t {
                type Output = APInt;

                fn rem(self, other: APInt) -> APInt {
                    let rhs = other.zext_as::<$t>();
                    if rhs.is_zero() {
                        panic!("Division by zero in APInt");
                    }
                    APInt::new_full(
                        (self as u128).wrapping_rem(rhs.as_unsigned()),
                        rhs.bits
                    )
                }
            }

            impl Rem<$t> for APInt {
                type Output = APInt;

                fn rem(self, other: $t) -> APInt {
                    let rhs = APInt::from(other).zext_to(self.bits);
                    self.urem(rhs)
                }
            }

            impl RemAssign<$t> for APInt {
                fn rem_assign(&mut self, other: $t) {
                    let rhs = APInt::from(other).zext_to(self.bits);
                    *self = self.urem(rhs);
                }
            }

            impl BitAnd<APInt> for $t {
                type Output = APInt;

                fn bitand(self, other: APInt) -> APInt {
                    let rhs = other.zext_as::<$t>();
                    let lhs = APInt::from(self);
                    lhs.bitand(rhs)
                }
            }
            impl BitAnd<$t> for APInt {
                type Output = APInt;

                fn bitand(self, other: $t) -> APInt {
                    let rhs = APInt::from(other).zext_to(self.bits);
                    self.bitand(rhs)
                }
            }
            impl BitAndAssign<$t> for APInt {
                fn bitand_assign(&mut self, other: $t) {
                    let rhs = APInt::from(other).zext_to(self.bits);
                    *self &= rhs;
                }
            }

            impl BitOr<APInt> for $t {
                type Output = APInt;

                fn bitor(self, other: APInt) -> APInt {
                    let rhs = other.zext_as::<$t>();
                    let lhs = APInt::from(self);
                    lhs.bitor(rhs)
                }
            }
            impl BitOr<$t> for APInt {
                type Output = APInt;

                fn bitor(self, other: $t) -> APInt {
                    let rhs = APInt::from(other).zext_to(self.bits);
                    self.bitor(rhs)
                }
            }
            impl BitOrAssign<$t> for APInt {
                fn bitor_assign(&mut self, other: $t) {
                    let rhs = APInt::from(other).zext_to(self.bits);
                    *self |= rhs;
                }
            }

            impl BitXor<APInt> for $t {
                type Output = APInt;

                fn bitxor(self, other: APInt) -> APInt {
                    let rhs = other.zext_as::<$t>();
                    let lhs = APInt::from(self);
                    lhs.bitxor(rhs)
                }
            }
            impl BitXor<$t> for APInt {
                type Output = APInt;

                fn bitxor(self, other: $t) -> APInt {
                    let rhs = APInt::from(other).zext_to(self.bits);
                    self.bitxor(rhs)
                }
            }
            impl BitXorAssign<$t> for APInt {
                fn bitxor_assign(&mut self, other: $t) {
                    let rhs = APInt::from(other).zext_to(self.bits);
                    *self ^= rhs;
                }
            }

            impl Shl<APInt> for $t {
                type Output = $t;

                fn shl(self, other: APInt) -> $t {
                    let rhs = other.zext_as::<$t>();
                    let shift = rhs.as_unsigned() as u64;
                    if shift >= (core::mem::size_of::<$t>() as u64 * 8) {
                        return 0;
                    }
                    self.wrapping_shl(shift as u32)
                }
            }
            impl Shl<$t> for APInt {
                type Output = APInt;

                fn shl(self, other: $t) -> APInt {
                    let lhs = self.as_unsigned();
                    if other as u64 >= (core::mem::size_of::<$t>() as u64 * 8) {
                        return APInt::new(0u128, self.bits);
                    }
                    let shift = other as u32;
                    APInt::new(lhs.wrapping_shl(shift), self.bits)
                }
            }
            impl ShlAssign<$t> for APInt {
                fn shl_assign(&mut self, other: $t) {
                    *self = self.shl(other);
                }
            }
            impl ShlAssign<APInt> for $t {
                fn shl_assign(&mut self, other: APInt) {
                    *self = self.shl(other);
                }
            }

            impl Shr<APInt> for $t {
                type Output = $t;

                fn shr(self, other: APInt) -> $t {
                    let rhs = other.zext_as::<$t>();
                    let shift = rhs.as_unsigned() as u64;
                    if shift >= (core::mem::size_of::<$t>() as u64 * 8) {
                        return 0;
                    }
                    self.wrapping_shr(shift as u32)
                }
            }

            impl PartialEq<APInt> for $t {
                fn eq(&self, other: &APInt) -> bool {
                    other.as_unsigned() == (*self as u128)
                }
            }
            impl PartialEq<$t> for APInt {
                fn eq(&self, other: &$t) -> bool {
                    self.as_unsigned() == *other as u128
                }
            }
        )+
    };
}

impl_add_from_uints!(u8, u16, u32, u64, u128, usize);

macro_rules! impl_add_from_sints {
    ($($t:ty),+) => {
        $(
            impl Add<APInt> for $t {
                type Output = APInt;

                fn add(self, other: APInt) -> APInt {
                    let rhs = other.sext_as::<$t>();
                    let lhs = APInt::from(self);
                    lhs + rhs
                }
            }

            impl Add<$t> for APInt {
                type Output = APInt;

                fn add(self, other: $t) -> APInt {
                    let rhs = APInt::from(other).sext_to(self.bits);
                    self + rhs
                }
            }

            impl AddAssign<$t> for APInt {
                fn add_assign(&mut self, other: $t) {
                    let rhs = APInt::from(other).sext_to(self.bits);
                    *self += rhs;
                }
            }

            impl Sub<APInt> for $t {
                type Output = APInt;

                fn sub(self, other: APInt) -> APInt {
                    let rhs = other.sext_as::<$t>();
                    let lhs = APInt::from(self);
                    lhs - rhs
                }
            }

            impl Sub<$t> for APInt {
                type Output = APInt;

                fn sub(self, other: $t) -> APInt {
                    let rhs = APInt::from(other).sext_to(self.bits);
                    self - rhs
                }
            }

            impl SubAssign<$t> for APInt {
                fn sub_assign(&mut self, other: $t) {
                    let rhs = APInt::from(other).sext_to(self.bits);
                    *self -= rhs;
                }
            }

            impl Mul<APInt> for $t {
                type Output = APInt;

                fn mul(self, other: APInt) -> APInt {
                    let rhs = other.sext_as::<$t>();
                    let lhs = APInt::from(self);
                    lhs * rhs
                }
            }

            impl Mul<$t> for APInt {
                type Output = APInt;

                fn mul(self, other: $t) -> APInt {
                    let rhs = APInt::from(other).sext_to(self.bits);
                    self * rhs
                }
            }

            impl MulAssign<$t> for APInt {
                fn mul_assign(&mut self, other: $t) {
                    let rhs = APInt::from(other).sext_to(self.bits);
                    *self *= rhs;
                }
            }

            impl Div<APInt> for $t {
                type Output = APInt;

                fn div(self, other: APInt) -> APInt {
                    if other.is_zero() {
                        panic!("Division by zero in APInt");
                    }
                    let rhs = other.sext_as::<$t>();
                    let lhs = APInt::from(self);
                    lhs.sdiv(rhs)
                }
            }
            impl Div<$t> for APInt {
                type Output = APInt;

                fn div(self, other: $t) -> APInt {
                    let rhs = APInt::from(other).sext_to(self.bits);
                    self.sdiv(rhs)
                }
            }

            impl DivAssign<$t> for APInt {
                fn div_assign(&mut self, other: $t) {
                    let rhs = APInt::from(other).sext_to(self.bits);
                    *self = self.sdiv(rhs);
                }
            }

            impl BitAnd<APInt> for $t {
                type Output = APInt;

                fn bitand(self, other: APInt) -> APInt {
                    let rhs = other.sext_as::<$t>();
                    let lhs = APInt::from(self);
                    lhs.bitand(rhs)
                }
            }
            impl BitAnd<$t> for APInt {
                type Output = APInt;

                fn bitand(self, other: $t) -> APInt {
                    let rhs = APInt::from(other).sext_to(self.bits);
                    self.bitand(rhs)
                }
            }
            impl BitAndAssign<$t> for APInt {
                fn bitand_assign(&mut self, other: $t) {
                    let rhs = APInt::from(other).sext_to(self.bits);
                    *self &= rhs;
                }
            }

            impl BitOr<APInt> for $t {
                type Output = APInt;

                fn bitor(self, other: APInt) -> APInt {
                    let rhs = other.sext_as::<$t>();
                    let lhs = APInt::from(self);
                    lhs.bitor(rhs)
                }
            }
            impl BitOr<$t> for APInt {
                type Output = APInt;

                fn bitor(self, other: $t) -> APInt {
                    let rhs = APInt::from(other).sext_to(self.bits);
                    self.bitor(rhs)
                }
            }
            impl BitOrAssign<$t> for APInt {
                fn bitor_assign(&mut self, other: $t) {
                    let rhs = APInt::from(other).sext_to(self.bits);
                    *self |= rhs;
                }
            }

            impl BitXor<APInt> for $t {
                type Output = APInt;

                fn bitxor(self, other: APInt) -> APInt {
                    let rhs = other.sext_as::<$t>();
                    let lhs = APInt::from(self);
                    lhs.bitxor(rhs)
                }
            }
            impl BitXor<$t> for APInt {
                type Output = APInt;

                fn bitxor(self, other: $t) -> APInt {
                    let rhs = APInt::from(other).sext_to(self.bits);
                    self.bitxor(rhs)
                }
            }
            impl BitXorAssign<$t> for APInt {
                fn bitxor_assign(&mut self, other: $t) {
                    let rhs = APInt::from(other).sext_to(self.bits);
                    *self ^= rhs;
                }
            }

            impl Shl<APInt> for $t {
                type Output = $t;

                fn shl(self, other: APInt) -> $t {
                    let rhs = other.sext_as::<$t>();
                    let shift = rhs.as_unsigned() as u64;
                    if shift >= (core::mem::size_of::<$t>() as u64 * 8) {
                        return 0;
                    }
                    self.wrapping_shl(shift as u32)
                }
            }
            impl Shl<$t> for APInt {
                type Output = APInt;

                fn shl(self, other: $t) -> APInt {
                    let lhs = self.as_unsigned();
                    if other as u64 >= (core::mem::size_of::<$t>() as u64 * 8) {
                        return APInt::new(0u128, self.bits);
                    }
                    let shift = other as u32;
                    APInt::new(lhs.wrapping_shl(shift), self.bits)
                }
            }
            impl ShlAssign<$t> for APInt {
                fn shl_assign(&mut self, other: $t) {
                    *self = self.shl(other);
                }
            }
            impl ShlAssign<APInt> for $t {
                fn shl_assign(&mut self, other: APInt) {
                    *self = self.shl(other);
                }
            }

            impl Shr<APInt> for $t {
                type Output = $t;

                fn shr(self, other: APInt) -> $t {
                    let rhs = other.sext_as::<$t>();
                    let shift = rhs.as_unsigned() as u64;
                    if shift >= (core::mem::size_of::<$t>() as u64 * 8) {
                        return 0;
                    }
                    self.wrapping_shr(shift as u32)
                }
            }

            impl PartialEq<APInt> for $t {
                fn eq(&self, other: &APInt) -> bool {
                    other.as_signed() == (*self as i128)
                }
            }
            impl PartialEq<$t> for APInt {
                fn eq(&self, other: &$t) -> bool {
                    self.as_signed() == *other as i128
                }
            }
        )+
    };
}

impl_add_from_sints!(i8, i16, i32, i64, i128, isize);

pub trait IIntoU128 {
    fn into_u128(self) -> u128;
}

macro_rules! int_as_u128 {
    ($($t:ty),+) => {
        $(
            impl IIntoU128 for $t {
                fn into_u128(self) -> u128 {
                    self as u128
                }
            }
        )+
    };
}

int_as_u128!(
    i8, i16, i32, i64, i128, isize, u8, u16, u32, u64, u128, usize
);

#[cfg(test)]
mod testing {
    use super::APInt;

    #[test]
    fn test_apint_creation() {
        println!("=== APInt 创建测试 ===");

        // 从不同整数类型创建
        let a = APInt::from(42u8);
        println!(
            "APInt::from(42u8): {}, bits: {}, unsigned: {}, signed: {}",
            a,
            a.bits(),
            a.as_unsigned(),
            a.as_signed()
        );

        let b = APInt::from(-42i8);
        println!(
            "APInt::from(-42i8): {}, bits: {}, unsigned: {}, signed: {}",
            b,
            b.bits(),
            b.as_unsigned(),
            b.as_signed()
        );

        let c = APInt::from(1000u16);
        println!(
            "APInt::from(1000u16): {}, bits: {}, unsigned: {}, signed: {}",
            c,
            c.bits(),
            c.as_unsigned(),
            c.as_signed()
        );

        let d = APInt::from(-1000i16);
        println!(
            "APInt::from(-1000i16): {}, bits: {}, unsigned: {}, signed: {}",
            d,
            d.bits(),
            d.as_unsigned(),
            d.as_signed()
        );

        // 使用 new 方法创建
        let e = APInt::new(255u32, 8);
        println!(
            "APInt::new(255u32, 8): {}, bits: {}, unsigned: {}, signed: {}",
            e,
            e.bits(),
            e.as_unsigned(),
            e.as_signed()
        );

        let f = APInt::new(256u32, 8); // 会被截断
        println!(
            "APInt::new(256u32, 8): {}, bits: {}, unsigned: {}, signed: {}",
            f,
            f.bits(),
            f.as_unsigned(),
            f.as_signed()
        );

        // 布尔值
        let g = APInt::from(true);
        println!(
            "APInt::from(true): {}, bits: {}, unsigned: {}, signed: {}",
            g,
            g.bits(),
            g.as_unsigned(),
            g.as_signed()
        );

        let h = APInt::from(false);
        println!(
            "APInt::from(false): {}, bits: {}, unsigned: {}, signed: {}",
            h,
            h.bits(),
            h.as_unsigned(),
            h.as_signed()
        );
    }

    #[test]
    fn test_sign_interpretation() {
        println!("\n=== 有符号/无符号解释测试 ===");

        // 测试符号位解释
        let values = [0xFF_u128, 0x80_u128, 0x7F_u128, 0x00_u128, 0x01_u128];

        for &val in &values {
            let apint = APInt::new(val, 8);
            println!(
                "值 0x{:02X} (8位): unsigned = {}, signed = {}, is_negative = {}",
                val,
                apint.as_unsigned(),
                apint.as_signed(),
                apint.is_negative()
            );
        }

        // 测试不同位宽的符号解释
        println!("\n不同位宽的符号解释:");
        let val = 0xFFFF_u128;
        for bits in [8, 16, 32] {
            let apint = APInt::new(val, bits);
            println!(
                "0x{:04X} 作为 {}-bit: unsigned = {}, signed = {}",
                val,
                bits,
                apint.as_unsigned(),
                apint.as_signed()
            );
        }
    }

    #[test]
    fn test_zero_nonzero() {
        println!("\n=== 零值测试 ===");

        let zero = APInt::new(0, 32);
        let nonzero = APInt::new(1, 32);

        println!(
            "APInt::new(0, 32): is_zero = {}, is_nonzero = {}",
            zero.is_zero(),
            zero.is_nonzero()
        );
        println!(
            "APInt::new(1, 32): is_zero = {}, is_nonzero = {}",
            nonzero.is_zero(),
            nonzero.is_nonzero()
        );
    }

    #[test]
    fn test_extension() {
        println!("\n=== 扩展测试 ===");

        let a = APInt::new(0x80_u128, 8); // -128 as signed, 128 as unsigned
        println!(
            "原始值 (8-bit): {}, unsigned = {}, signed = {}",
            a,
            a.as_unsigned(),
            a.as_signed()
        );

        let zext = a.zext_to(16);
        println!(
            "零扩展到 16-bit: {}, unsigned = {}, signed = {}",
            zext,
            zext.as_unsigned(),
            zext.as_signed()
        );

        let sext = a.sext_to(16);
        println!(
            "符号扩展到 16-bit: {}, unsigned = {}, signed = {}",
            sext,
            sext.as_unsigned(),
            sext.as_signed()
        );
    }

    #[test]
    fn test_arithmetic_operations() {
        println!("\n=== 代数运算测试 ===");

        // 基本加法
        println!("--- 加法测试 ---");
        let a = APInt::new(100_u128, 8);
        let b = APInt::new(50_u128, 8);
        let sum = a + b;
        println!("{} + {} = {}", a, b, sum);

        // 溢出加法
        let c = APInt::new(200_u128, 8);
        let d = APInt::new(100_u128, 8);
        let overflow_sum = c + d;
        println!("{} + {} = {} (溢出情况)", c, d, overflow_sum);

        // 减法
        println!("\n--- 减法测试 ---");
        let e = APInt::new(100_u128, 8);
        let f = APInt::new(30_u128, 8);
        let diff = e - f;
        println!("{} - {} = {}", e, f, diff);

        // 下溢减法
        let g = APInt::new(10_u128, 8);
        let h = APInt::new(20_u128, 8);
        let underflow_diff = g - h;
        println!("{} - {} = {} (下溢情况)", g, h, underflow_diff);

        // 乘法
        println!("\n--- 乘法测试 ---");
        let i = APInt::new(15_u128, 8);
        let j = APInt::new(10_u128, 8);
        let product = i * j;
        println!("{} * {} = {}", i, j, product);

        // 溢出乘法
        let k = APInt::new(20_u128, 8);
        let l = APInt::new(20_u128, 8);
        let overflow_product = k * l;
        println!("{} * {} = {} (溢出情况)", k, l, overflow_product);
    }

    #[test]
    fn test_shift_operations() {
        println!("\n=== 位移运算测试 ===");

        let a = APInt::new(0b0001_1111_u128, 32);
        let left_shift = a << APInt::new(2_u128, 8);
        let right_shift = a.lshr_with(APInt::new(2_u128, 8));
        println!("{} << 2 = {}", a, left_shift);
        println!("{} >> 2 = {}", a, right_shift);

        let left_shift = a << 10u32;
        let right_shift = a.lshr(10u32);
        println!("{} << 10 = {}", a, left_shift);
        println!("{} >> 10 = {}", a, right_shift);

        let a = APInt::from(2u8);
        let left_shift = 10u8 << a;
        let right_shift = 10u8 >> a;
        println!("10u8 << {} = {}", a, left_shift);
        println!("10u8 >> {} = {}", a, right_shift);
    }

    #[test]
    fn test_division_operations() {
        println!("\n=== 除法运算测试 ===");

        // 无符号除法
        println!("--- 无符号除法测试 ---");
        let a = APInt::new(100_u128, 8);
        let b = APInt::new(7_u128, 8);
        let quotient = a.udiv(b);
        let remainder = a.urem(b);
        println!("{} udiv {} = {}", a, b, quotient);
        println!("{} urem {} = {}", a, b, remainder);

        // 有符号除法
        println!("\n--- 有符号除法测试 ---");
        let c = APInt::new(100_u128, 8);
        let d = APInt::new(7_u128, 8);
        let signed_quotient = c.sdiv(d);
        let signed_remainder = c.srem(d);
        println!(
            "{} sdiv {} = {} (作为有符号: {} / {} = {})",
            c,
            d,
            signed_quotient,
            c.as_signed(),
            d.as_signed(),
            signed_quotient.as_signed()
        );
        println!(
            "{} srem {} = {} (作为有符号: {} % {} = {})",
            c,
            d,
            signed_remainder,
            c.as_signed(),
            d.as_signed(),
            signed_remainder.as_signed()
        );

        // 负数除法
        println!("\n--- 负数除法测试 ---");
        let e = APInt::new(0xF0_u128, 8); // -16 as signed
        let f = APInt::new(3_u128, 8);
        let neg_quotient = e.sdiv(f);
        let neg_remainder = e.srem(f);
        println!(
            "{} (signed: {}) sdiv {} = {} (signed: {})",
            e,
            e.as_signed(),
            f,
            neg_quotient,
            neg_quotient.as_signed()
        );
        println!(
            "{} (signed: {}) srem {} = {} (signed: {})",
            e,
            e.as_signed(),
            f,
            neg_remainder,
            neg_remainder.as_signed()
        );
    }

    #[test]
    fn test_mixed_type_operations() {
        println!("\n=== 混合类型运算测试 ===");

        // APInt 与原生类型运算
        println!("--- APInt 与 u8 运算 ---");
        let a = APInt::new(100_u128, 16);
        let b = 50u8; // 使用 u8，这样 APInt::from(u8) 是 8-bit，可以扩展到 16-bit
        let sum = a + b;
        println!("{} + {} = {}", a, b, sum);

        println!("--- u8 与 APInt 运算 ---");
        let c = 200u8; // 使用 u8
        let d = APInt::new(30_u128, 8); // 匹配 u8 的位宽
        let sum2 = c + d;
        println!("{} + {} = {}", c, d, sum2);

        // 有符号类型运算
        println!("\n--- APInt 与 i8 运算 ---");
        let e = APInt::new(50_u128, 16);
        let f = -20i8; // 使用 i8
        let sum3 = e + f;
        println!("{} + {} = {} (signed: {})", e, f, sum3, sum3.as_signed());

        // 乘法
        println!("\n--- 混合乘法 ---");
        let g = APInt::new(15_u128, 8);
        let h = 3u8;
        let product = g * h;
        println!("{} * {} = {}", g, h, product);
    }

    #[test]
    fn test_edge_cases() {
        println!("\n=== 边界情况测试 ===");

        // 最大值和最小值
        println!("--- 8-bit 边界值 ---");
        let max_u8 = APInt::new(255_u128, 8);
        let min_u8 = APInt::new(0_u128, 8);
        println!(
            "max u8: {}, unsigned = {}, signed = {}",
            max_u8,
            max_u8.as_unsigned(),
            max_u8.as_signed()
        );
        println!(
            "min u8: {}, unsigned = {}, signed = {}",
            min_u8,
            min_u8.as_unsigned(),
            min_u8.as_signed()
        );

        // 符号位边界
        let sign_bit = APInt::new(128_u128, 8); // 0x80
        println!(
            "sign bit (0x80): {}, unsigned = {}, signed = {}, is_negative = {}",
            sign_bit,
            sign_bit.as_unsigned(),
            sign_bit.as_signed(),
            sign_bit.is_negative()
        );

        // 奇数位宽
        println!("\n--- 奇数位宽测试 ---");
        let odd_width = APInt::new(15_u128, 3); // 3-bit，最大值是 7
        println!(
            "APInt::new(15, 3): {}, unsigned = {}, signed = {}",
            odd_width,
            odd_width.as_unsigned(),
            odd_width.as_signed()
        );

        // 1-bit 值
        println!("\n--- 1-bit 值测试 ---");
        let one_bit_0 = APInt::new(0_u128, 1);
        let one_bit_1 = APInt::new(1_u128, 1);
        println!(
            "1-bit 0: {}, unsigned = {}, signed = {}",
            one_bit_0,
            one_bit_0.as_unsigned(),
            one_bit_0.as_signed()
        );
        println!(
            "1-bit 1: {}, unsigned = {}, signed = {}",
            one_bit_1,
            one_bit_1.as_unsigned(),
            one_bit_1.as_signed()
        );
    }
}

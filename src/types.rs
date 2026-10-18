use fast_posit::{RoundFrom, RoundInto};
use std::{iter::Sum, ops};

pub trait MathType:
    Default
    + Copy
    + ops::Mul<Output = Self>
    + ops::Div<Output = Self>
    + ops::AddAssign
    + ops::SubAssign
    + ops::MulAssign
    + ops::DivAssign
    + ops::Add<Output = Self>
    + ops::Sub<Output = Self>
    + ops::Neg<Output = Self>
    + PartialOrd
    + NumericConstants
{
    fn abs(self) -> Self;
}
// impl<T> MathType for T where
//     T: Default
//         + Copy
//         + ops::Mul<Output = Self>
//         + ops::Div<Output = Self>
//         + ops::AddAssign
//         + ops::SubAssign
//         + ops::Add<Output = Self>
//         + ops::Sub<Output = Self>
//         + ops::Neg<Output = Self>
//         + PartialOrd
//         + NumericConstants
// {
// }
macro_rules! impl_MathType {
($( $T:ident ),+) => {$(
    impl MathType for $T {
        #[inline]
        fn abs(self) -> Self {
            self.abs()
        }
    }
)+};
}
impl_MathType!(p8, p16, p32, p64, f16, f32, f64, f128);

pub trait NumericConstants {
    const ONE: Self;
    const ZERO: Self;
}
macro_rules! impl_float_constants {
($( $T:ident ),+) => {$(
    impl NumericConstants for $T {
        const ONE: Self = 1.0;
        const ZERO: Self = 0.0;
    }
)+};
}
impl_float_constants!(f16, f32, f64, f128);

macro_rules! define_posit_wrapper {
($( $pNN:ident $Int:ty ),+) => {$(

#[allow(non_camel_case_types)]
#[derive(Clone, Copy, Default)]
pub struct $pNN(pub fast_posit::$pNN);

impl_posit_wrapper_op!($pNN: Mul mul, Div div, Add add, Sub sub ; Neg neg ; MulAssign mul_assign, DivAssign div_assign, AddAssign add_assign, SubAssign sub_assign);
impl NumericConstants for $pNN {
    const ONE: Self = $pNN::ONE;
    const ZERO: Self = $pNN::ZERO;
}
impl PartialEq for $pNN {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        return self.0.eq(&other.0);
    }
}
impl Eq for $pNN {}
impl PartialOrd for $pNN {
    #[inline]
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.0.partial_cmp(&other.0)
    }
}
impl Ord for $pNN {
    #[inline]
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.0.cmp(&other.0)
    }
}
impl std::fmt::Display for $pNN {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        RoundInto::<f64>::round_into(self.0).fmt(f)
    }
}

impl $pNN {
    #[inline]
    pub fn sqrt(self) -> Self {
        Self(self.0.sqrt())
    }
    #[inline]
    pub fn from_float(x: f64) -> Self {
        Self(fast_posit::$pNN::round_from(x))
    }
    #[inline]
    pub fn round_into<T>(&self) -> T
    where
        fast_posit::$pNN: RoundInto<T>,
    {
        self.0.round_into()
    }
    #[inline]
    pub fn from_bits(bits: $Int) -> Self {
        Self(fast_posit::$pNN::from_bits(bits))
    }
    #[inline]
    pub fn abs(self) -> Self {
        Self(self.0.abs())
    }
    #[inline]
    pub fn ceil(self) -> Self {
        Self(self.0.ceil())
    }
    #[inline]
    pub fn floor(self) -> Self {
        Self(self.0.floor())
    }
    #[inline]
    pub fn round(self) -> Self {
        Self(self.0.nearest_int())
    }
    #[inline]
    pub fn max(self, other: Self) -> Self {
        Self(self.0.max(other.0))
    }
    #[inline]
    pub fn min(self, other: Self) -> Self {
        Self(self.0.min(other.0))
    }

    pub const ZERO: Self = Self(fast_posit::$pNN::ZERO);
    pub const ONE: Self = Self(fast_posit::$pNN::ONE);
    pub const MINUS_ONE: Self = Self(fast_posit::$pNN::MINUS_ONE);
    pub const MIN: Self = Self(fast_posit::$pNN::MIN);
    pub const MAX: Self = Self(fast_posit::$pNN::MAX);
    pub const NAR: Self = Self(fast_posit::$pNN::NAR);
    // pub const PI: Self = Self(fast_posit::$pNN {
    //     0: 0b1001100100100001111110110101010,
    // });
    // pub const TWO: Self = Self(fast_posit::$pNN {
    //     0: 0b1001000000000000000000000000000,
    // });
    // pub fn ln(self) -> Self {
    //     if self.0 == fast_posit::$pNN::NAR {
    //         return self;
    //     }
    //     if self.0 {}
    // }
}
// impl<T> From<T> for $pNN
// where
//     fast_posit::$pNN: RoundFrom<T>,
// {
//     #[inline]
//     fn from(val: T) -> $pNN {
//         Self(fast_posit::$pNN::round_from(val))
//     }
// }

impl_posit_from!($pNN: posit p8, p16, p32, p64);
impl_posit_from!($pNN: f32, f64, i8, i16, i32, i64);
impl_posit_into!($pNN: f32, f64, i8, i16, i32, i64);
impl_posit_into!($pNN: f16 <- f32, f128 <- f64, u8 <- i16, u16 <- i32, u32 <- i64);

impl Sum for $pNN {
    fn sum<I: Iterator<Item = Self>>(iter: I) -> Self {
        iter.fold(Self::ZERO, <Self as ops::Add>::add)
    }
}

)+};
}
macro_rules! impl_posit_wrapper_op {
    ($T:ident : $($Op:ident $op:ident),* ; $($UnaryOp:ident $unary_op:ident),* ; $($OpAssign:ident $op_assign:ident),*) => {
        $(
        impl ops::$Op for $T {
            type Output = $T;
            #[inline]
            fn $op(self, rhs: Self) -> Self {
                Self(self.0.$op(rhs.0))
            }
        }
        impl ops::$Op for &$T {
            type Output = $T;
            #[inline]
            fn $op(self, rhs: Self) -> $T {
                $T(self.0.$op(rhs.0))
            }
        }
        )*
        $(
        impl ops::$UnaryOp for $T {
            type Output = $T;
            #[inline]
            fn $unary_op(self) -> Self {
                Self(self.0.$unary_op())
            }
        }
        impl ops::$UnaryOp for &$T {
            type Output = $T;
            #[inline]
            fn $unary_op(self) -> $T {
                $T(self.0.$unary_op())
            }
        }
        )*
        $(
        impl ops::$OpAssign for $T {
            #[inline]
            fn $op_assign(&mut self, rhs: Self) {
                self.0.$op_assign(rhs.0);
            }
        }
        )*
    };
}

macro_rules! impl_posit_from {
    ($pNN:ident : $( $Type:ident ),+) => {$(
        impl From<$Type> for $pNN {
            #[inline]
            fn from(val: $Type) -> Self {
                Self(fast_posit::$pNN::round_from(val))
            }
        }
    )+};
    ($pNN:ident : posit $( $Type:ident ),+) => {$( impl_posit_from!(@ $pNN : posit $Type); )+};
    (@ p8 : posit p8) => {};
    (@ p16 : posit p16) => {};
    (@ p32 : posit p32) => {};
    (@ p64 : posit p64) => {};
    (@ $pNN:ident : posit $Type:ident) => {
        impl From<$Type> for $pNN {
            #[inline]
            fn from(val: $Type) -> Self {
                Self(val.0.convert())
            }
        }
    };
}

macro_rules! impl_posit_into {
    ($pNN:ty : $( $Type:ident ),+) => {$(
        impl Into<$Type> for $pNN {
            #[inline]
            fn into(self) -> $Type {
                self.0.round_into()
            }
        }
    )+};
    ($pNN:ty : $( $Type:ident <- $Base:ident ),+) => {$(
        impl Into<$Type> for $pNN {
            #[inline]
            fn into(self) -> $Type {
                <$pNN as Into<$Base>>::into(self) as _
            }
        }
    )+};
}

define_posit_wrapper!(p8 i8, p16 i16, p32 i32, p64 i64);

#[repr(transparent)]
pub struct SliceFormatter<'a, T: std::fmt::Display>(pub &'a [T]);
impl<T: std::fmt::Display> std::fmt::Display for SliceFormatter<'_, T> {
    fn fmt(&self, f: &mut std::fmt::Formatter) -> std::fmt::Result {
        f.write_str("[")?;
        let mut iter = self.0.iter();
        if let Some(first) = iter.next() {
            first.fmt(f)?;
            for elem in iter {
                f.write_str(", ")?;
                elem.fmt(f)?;
            }
        }
        f.write_str("]")
    }
}

use super::types::*;
use std::{default::Default, ops};

#[derive(Copy, Clone)]
pub struct Complex<T> {
    pub im: T,
    pub re: T,
}

impl<T: MathType> Complex<T> {
    pub fn new(re: T, im: T) -> Self {
        Self { im, re }
    }
    #[inline]
    pub fn norm_sqr(&self) -> T {
        self.re * self.re + self.im * self.im
    }
    #[inline]
    pub fn conj(&self) -> Self {
        Self {
            re: self.re.clone(),
            im: -self.im.clone(),
        }
    }
    #[inline]
    pub fn inv(&self) -> Self {
        let norm_sqr = self.norm_sqr();
        Self {
            re: self.re / norm_sqr,
            im: -self.im / norm_sqr,
        }
    }
    // #[inline]
    // pub fn exp(self) -> Self {
    //     // formula: e^(a + bi) = e^a (cos(b) + i*sin(b)) = from_polar(e^a, b)

    //     let Complex { re, im } = self;
    //     Self::from_polar(re.exp(), im)
    // }
}
impl<T> From<(T, T)> for Complex<T> {
    fn from((im, re): (T, T)) -> Self {
        Self { re, im }
    }
}
impl<T: Default> Default for Complex<T> {
    fn default() -> Self {
        Self {
            re: Default::default(),
            im: Default::default(),
        }
    }
}
impl<T: MathType + ops::Rem<Output = T> + NumericConstants> Complex<T> {
    fn div_trunc(self, divisor: Self) -> Self {
        let Self { re, im } = self / divisor;
        Self {
            re: re - re % T::ONE,
            im: im - im % T::ONE,
        }
    }
}
impl<T: MathType> ops::Mul for Complex<T> {
    type Output = Complex<T>;
    #[inline]
    fn mul(self, rhs: Self) -> Self {
        Self {
            re: self.re * rhs.re - self.im * rhs.im,
            im: self.re * rhs.im + self.im * rhs.re,
        }
    }
}
impl<T: MathType> ops::Mul for &Complex<T> {
    type Output = Complex<T>;
    #[inline]
    fn mul(self, rhs: Self) -> Complex<T> {
        *self * *rhs
    }
}
impl<T: MathType> ops::MulAssign for Complex<T> {
    #[inline]
    fn mul_assign(&mut self, rhs: Self) {
        *self = *self * rhs
    }
}
impl<T: MathType> ops::Div for Complex<T> {
    type Output = Complex<T>;
    #[inline]
    fn div(self, rhs: Self) -> Self {
        self * rhs.inv()
    }
}
impl<T: MathType> ops::Add for Complex<T> {
    type Output = Complex<T>;
    #[inline]
    fn add(self, rhs: Self) -> Self {
        Self {
            re: self.re + rhs.re,
            im: self.im + rhs.im,
        }
    }
}
impl<T: MathType> ops::Sub for Complex<T> {
    type Output = Complex<T>;
    #[inline]
    fn sub(self, rhs: Self) -> Self {
        Self {
            re: self.re - rhs.re,
            im: self.im - rhs.im,
        }
    }
}
impl<T: MathType> ops::Neg for Complex<T> {
    type Output = Complex<T>;
    #[inline]
    fn neg(self) -> Self {
        Self {
            re: -self.re,
            im: -self.im,
        }
    }
}
impl<T: MathType> ops::AddAssign for Complex<T> {
    #[inline]
    fn add_assign(&mut self, rhs: Self) {
        *self = *self + rhs;
    }
}
impl<T: ops::Rem<Output = T> + MathType + NumericConstants> ops::Rem for Complex<T> {
    type Output = Complex<T>;
    #[inline]
    fn rem(self, rhs: Self) -> Self {
        let gaussian = self.div_trunc(rhs);
        self - rhs * gaussian
    }
}
impl<T: PartialEq> PartialEq for Complex<T> {
    #[inline]
    fn eq(&self, other: &Self) -> bool {
        self.re == other.re && self.im == other.im
    }
}
impl<T: Eq> Eq for Complex<T> {}

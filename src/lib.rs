#![feature(f16, f128, associated_type_defaults, integer_casts)]
use std::iter::zip;

use fast_posit::RoundFrom;

pub mod types;
pub use types::*;

impl gnuplot::DataType for p32 {
    fn get(&self) -> f64 {
        return (*self).into();
    }
}
// impl<T> From<p32> for T
// where
//     fast_posit::p32: RoundInto<T>,
// {
//     fn from(val: p32) -> T {
//         val.0.round_into()
//     }
// }

pub trait Context<T: MathType> {
    // + From<usize>;
    type Acc: Default + Copy + From<T> + Into<T> + std::ops::Add<T, Output = Self::Acc> = T;
    /// (possibly lossy) convert to usize
    // fn from_usize(val: usize) -> T;
    fn from_usize(val: usize) -> T;
    /// computes `(sin(2π * value), cos(2π * value))`
    fn sin_cos_2pi(value: T) -> (T, T);
    /// computes `*acc += a * b`, possibly using some kind of fused-multiply-add
    fn mul_add(acc: &mut Self::Acc, a: T, b: T);
    // // computes e^(i * 2π * value)
    // fn euler_2pi(value: T) -> Self::Complex {
    //     let (im, re) = Self::sin_cos_2pi(value);
    //     (re, im).into()
    // }
}

/// indicates basic floating point calculations
pub struct SimpleFloatContext;
macro_rules! impl_SimpleFloatContext {
    ($( $Type:ident ),+) => {$(
        impl Context<$Type> for SimpleFloatContext {
            type Acc = $Type;
            fn sin_cos_2pi(value: $Type) -> ($Type, $Type) {
                (value * std::$Type::consts::PI * 2.).sin_cos()
            }
            fn from_usize(val: usize) -> $Type {
                val as $Type
            }
            fn mul_add(acc: &mut $Type, a: $Type, b: $Type) {
                *acc += a * b;
            }
        }
    )+};
}
impl_SimpleFloatContext!(f16, f32, f64, f128);

pub struct TaylorSeriesFloatContext<const DEGREE: u32 = 20>;
macro_rules! impl_TaylorSeriesFloatContext {
    ($( $Type:ident ),+) => {$(
        impl<const DEGREE: u32> Context<$Type> for TaylorSeriesFloatContext<DEGREE> {
            type Acc = $Type;
            fn sin_cos_2pi(value: $Type) -> ($Type, $Type) {
                taylor_series::<_, _, DEGREE>(value * std::$Type::consts::PI * 2., |x| x as $Type)
            }
            fn from_usize(val: usize) -> $Type {
                val as $Type
            }
            fn mul_add(acc: &mut $Type, a: $Type, b: $Type) {
                *acc += a * b;
            }
        }
    )+};
}
impl_TaylorSeriesFloatContext!(f16, f32, f64, f128);

/// indicates floating point calculations using a fused-multiply-add hardware instruction where applicable
pub struct FmaFloatContext;
macro_rules! impl_FmaContext {
    ($( $Type:ident ),+) => {$(
        impl Context<$Type> for FmaFloatContext {
            type Acc = $Type;
            fn sin_cos_2pi(value: $Type) -> ($Type, $Type) {
                (value * std::$Type::consts::PI * 2.).sin_cos()
            }
            fn from_usize(val: usize) -> $Type {
                val as $Type
            }
            fn mul_add(acc: &mut $Type, a: $Type, b: $Type) {
                *acc = $Type::mul_add(a, b, *acc)
            }
        }
    )+};
}
impl_FmaContext!(f16, f32, f64, f128);

fn taylor_series<T: MathType, Cast: Fn(i64) -> T, const DEGREE: u32>(x: T, cast: Cast) -> (T, T) {
    let mut sin = T::ZERO;
    let mut cos = T::ZERO;
    let mut factorial = T::ONE; //since n = 0, starts at (2n)! = 1
    let mut pow = T::ONE; //since n = 0, starts at x^{2n} = 1
    let mut last_sin = sin;
    let mut last_cos = cos;
    for n in 0..DEGREE {
        cos += if n & 1 == 0 { pow } else { -pow } / factorial;
        factorial *= cast((2 * n + 1) as i64); // calc (2n + 1)!
        pow *= x; // calc x^(2n + 1)
        sin += if n & 1 == 0 { pow } else { -pow } / factorial;
        if last_sin == sin && last_cos == cos {
            break;
        }
        factorial *= cast((2 * (n + 1)) as i64); // calc next (2n)!
        pow *= x; // calc next x^(2n)
        (last_sin, last_cos) = (sin, cos);
    }
    (sin, cos)
}

/// indicates posit calculations without a quire, using the taylor series approximation of sin and cos
// degree 15 seems to be fine for calculations up to x = 2π 0.5 for p32
pub struct PositNoQuireContextWithTaylorSeries<const DEGREE: u32 = 20>;
macro_rules! impl_ctx_taylor_no_quire_posit {
    ($( $pNN:ident ),+) => {$(

impl<const DEGREE: u32> Context<$pNN> for PositNoQuireContextWithTaylorSeries<DEGREE> {
    type Acc = $pNN;

    fn sin_cos_2pi(value: $pNN) -> ($pNN, $pNN) {
        // no remainder function available in fast_posit
        // let value = value % p32::ONE;
        // this function always called with 0.0 <= value <= 0.5 anyway

        // let two = p32::from_bits(0b1001000000000000000000000000000); // p32::from(2)
        // let pi = p32::from_bits(0b01001100100100001111110110101010); // p32::from(std::f64::consts::PI)
        // let two_pi = p32::from_bits(0b01010100100100001111110110101010);
        let two_pi = $pNN::from(std::f64::consts::TAU);
        let x = value * two_pi;
        taylor_series::<_, _, DEGREE>(x, $pNN::from)
    }
    fn from_usize(val: usize) -> $pNN {
        #[cfg(target_pointer_width = "64")]
        return $pNN(fast_posit::$pNN::round_from(val as i64));
        #[cfg(target_pointer_width = "32")]
        return $pNN(fast_posit::$pNN::round_from(val as i32));
    }
    fn mul_add(acc: &mut $pNN, a: $pNN, b: $pNN) {
        *acc += a * b;
    }
}
    )+};
}
impl_ctx_taylor_no_quire_posit!(p8, p16, p32, p64);

pub trait ComplexLike<T> {
    fn re(&self) -> T;
    fn im(&self) -> T;
}
impl<T: MathType> ComplexLike<T> for T {
    #[inline]
    fn re(&self) -> T {
        *self
    }
    #[inline]
    fn im(&self) -> T {
        T::ZERO
    }
}
impl<T: MathType> ComplexLike<T> for (T, T) {
    #[inline]
    fn re(&self) -> T {
        self.0
    }
    #[inline]
    fn im(&self) -> T {
        self.1
    }
}

pub fn fft<T: MathType, C: ComplexLike<T>, Ctx: Context<T>>(input: &[C]) -> Box<[(T, T)]> {
    let len = input.len();
    let big_n = len.next_power_of_two();
    let mut res = unsafe { Box::new_zeroed_slice(big_n).assume_init() };

    fn inner<T: MathType, C: ComplexLike<T>, Ctx: Context<T>>(
        input: &[C],
        res: &mut [(T, T)],
        big_n: usize,
        step: usize,
    ) {
        if big_n == 1 {
            if input.len() > 0 {
                res[0] = (input[0].re(), input[0].im());
            }
            return;
        }
        inner::<_, _, Ctx>(input, &mut res[..big_n / 2], big_n / 2, step * 2);
        inner::<_, _, Ctx>(
            if input.len() >= step {
                &input[step..]
            } else {
                input
            },
            &mut res[big_n / 2..],
            big_n / 2,
            step * 2,
        );

        for k in 0..big_n / 2 {
            let p_re = res[k].0;
            let p_im = res[k].1;
            // q = e^(-2pi i k / N) X_{k+N/2}
            // q = (cos(-2pi k / N) sin(2pi k / N)i) X_{k+N/2}
            let (sin, cos) = Ctx::sin_cos_2pi(Ctx::from_usize(k) / Ctx::from_usize(big_n));
            let (re, im) = (cos, -sin);
            // (x1 + y1 i) (x2 + y2 i) = x1x2 - y1y2 + (x1y2 + y1x2)i
            let q_re = re * res[k + big_n / 2].0 - im * res[k + big_n / 2].1;
            let q_im = re * res[k + big_n / 2].1 + im * res[k + big_n / 2].0;
            // X_k = p + q
            res[k].0 = q_re + p_re;
            res[k].1 = q_im + p_im;
            // X_{k + N/2} = p - q
            res[k + big_n / 2].0 = p_re - q_re;
            res[k + big_n / 2].1 = p_im - q_im;
        }
    }
    inner::<_, _, Ctx>(input, &mut *res, big_n, 1);
    res
}

pub fn inverse_ftt<T: MathType, Ctx: Context<T>>(input: &[(T, T)]) -> Box<[(T, T)]> {
    // avoid a memory allocation at the cost of another monomorphization of fft()
    /// imaginary component is negated
    #[repr(transparent)]
    struct Conjugate<T>((T, T));
    impl<T: MathType> ComplexLike<T> for Conjugate<T> {
        #[inline]
        fn re(&self) -> T {
            self.0.0
        }
        #[inline]
        fn im(&self) -> T {
            -self.0.1
        }
    }
    // safety: Conjugate<T> has the same layout as (T, T)
    let conjugate: &[Conjugate<T>] = unsafe { std::mem::transmute(input) };
    // let conjugate: Box<_> = input.iter().map(|&(re, im)| (re, -im)).collect();
    let mut res = fft::<_, _, Ctx>(&conjugate);
    let n = Ctx::from_usize(res.len());
    for (re, im) in &mut res {
        *re /= n;
        *im /= n;
    }
    res
}
#[derive(Debug)]
pub struct CrossCorrelateError;
impl std::error::Error for CrossCorrelateError {}
impl std::fmt::Display for CrossCorrelateError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "input sizes do not match")
    }
}
impl From<()> for CrossCorrelateError {
    fn from(_: ()) -> Self {
        Self
    }
}
pub fn cross_correlate<T>(a: &[T], b: &[T]) -> Result<Box<[T]>, CrossCorrelateError>
where
    T: std::ops::Mul + std::ops::AddAssign<T::Output> + std::default::Default + Copy,
{
    if a.len() != b.len() {
        return Err(CrossCorrelateError);
    }
    let len = a.len() as isize;
    return Ok((-len..len)
        .map(|m| {
            let mut sum = T::default();
            // for n in 0..m.max(0) {
            //     if len - m + n >= 0 && len - m + n < len {
            //         sum += a[n as usize] * b[(len - m + n) as usize];
            //     }
            // }
            for n in m.max(0)..len.min(len + m) {
                // if n - m >= 0 && n - m < len {
                sum += a[n as usize] * b[(n - m) as usize];
                // }
            }
            sum
        })
        .collect());
}

pub fn xf_correlate<T: MathType, Ctx: Context<T>>(
    a: &[T],
    b: &[T],
) -> Result<Box<[(T, T)]>, CrossCorrelateError> {
    let cross = cross_correlate(a.as_ref(), b.as_ref())?;
    Ok(fft::<_, _, Ctx>(&*cross))
}
// actually useful https://focisrc.github.io/eht-databook/intro/correlation.html

pub fn fx_correlate<T: MathType, Ctx: Context<T>>(a: &[T], b: &[T]) -> Box<[(T, T)]> {
    let a_freq = fft::<_, _, Ctx>(a);
    let b_freq = fft::<_, _, Ctx>(b);

    // (x1 + y1 i) (x2 + y2 i) = x1x2 - y1y2 + (x1y2 + y1x2)i
    zip(a_freq, b_freq)
        .map(|((re_a, im_a), (re_b, im_b))| (re_a * re_b - im_a * im_b, re_a * im_b + im_a * re_b))
        .collect()
}

pub fn ifft<T: MathType, Ctx: Context<T>>(input: &[(T, T)]) -> Box<[(T, T)]> {
    let len = input.len();
    let len_as_t = Ctx::from_usize(len);
    (0..len)
        .map(|n| {
            let mut iter = (0..len).map(|k| {
                let (im, re) = Ctx::sin_cos_2pi(Ctx::from_usize(k * n) / Ctx::from_usize(len));
                // (x1 + y1 i) (x2 + y2 i) = x1x2 - y1y2 + (x1y2 + y1x2)i
                let res_re = input[k].0 * re - input[k].1 * im;
                let res_im = input[k].0 * im + input[k].1 * re;
                (res_re, res_im)
            });
            let first = iter.next();
            // unwrap_unchecked is safe because if we are in the body of map, `len` >= 1, and the iterator has length `len` so it has at least 1 element
            let first = unsafe { first.unwrap_unchecked() };
            let first: (Ctx::Acc, Ctx::Acc) = (first.0.into(), first.1.into());
            let (re, im) = iter.fold(first, |(a_re, a_im), (b_re, b_im)| {
                (a_re + b_re, a_im + b_im)
            });
            (re.into() / len_as_t, im.into() / len_as_t)
        })
        .collect()
}

// pub fn rel_error<T: MathType + Default + Ord>(a: T, b: T) -> Option<T> {
//     if a == b {
//         Some(a - b)
//     } else if a.cmp(&T::default()) != b.cmp(&T::default()) {
//         None
//     } else {
//     }
// }

/// mean squared error
pub fn mse2<T, I>(iter: I) -> T
where
    T: MathType + From<i64>,
    I: ExactSizeIterator<Item = (T, T)>,
{
    mse(iter.map(|(a, b)| a - b))
}
/// mean squared error
pub fn mse<T, I>(iter: I) -> T
where
    T: MathType + From<i64>,
    I: ExactSizeIterator<Item = T>,
{
    let n = iter.len();
    if n == 0 {
        return T::ZERO;
    }
    iter.fold(T::ZERO, |acc, err| acc + err * err) / T::from(n.saturating_cast())
}

/// mean absolute error
pub fn mae2<T, I>(iter: I) -> T
where
    T: MathType + From<i64>,
    I: ExactSizeIterator<Item = (T, T)>,
{
    mae(iter.map(|(a, b)| a - b))
}
/// mean absolute error
pub fn mae<T, I>(iter: I) -> T
where
    T: MathType + From<i64>,
    I: ExactSizeIterator<Item = T>,
{
    let n = iter.len();
    if n == 0 {
        return T::ZERO;
    }
    iter.fold(T::ZERO, |acc, err| acc + err.abs()) / T::from(n.saturating_cast())
}

pub fn mean<T, I>(iter: I) -> T
where
    T: MathType + From<i64>,
    I: ExactSizeIterator<Item = T>,
{
    let n = iter.len();
    let Some(sum) = iter.reduce(T::add) else {
        return T::ZERO;
    };
    sum / T::from(n.saturating_cast())
}

// /// implementation of median that allocates and sorts
// pub fn median<T, I>(iter: I) -> T
// where
//     T: MathType + From<i32> + Ord,
//     I: ExactSizeIterator<Item = T>,
// {
//     if iter.len() == 0 {
//         return T::ZERO;
//     }
//     let mut items: Vec<_> = iter.collect();
//     items.sort_unstable();
//     if items.len() & 0b1 != 0 {
//         items[items.len() / 2]
//     } else {
//         (items[items.len() / 2 - 1] + items[items.len() / 2]) / T::from(2)
//     }
// }

// /// implementation of median that sorts its input
// pub fn median<T>(items: &mut [T]) -> T
// where
//     T: MathType + From<i32> + Ord,
// {
//     if items.len() == 0 {
//         return T::ZERO;
//     }
//     items.sort_unstable();
//     if items.len() & 0b1 != 0 {
//         items[items.len() / 2]
//     } else {
//         (items[items.len() / 2 - 1] + items[items.len() / 2]) / T::from(2)
//     }
// }

/// O(n^2) implementation of median.\
/// this technically wrong for even length lists since it should average the two middle elements, but whatever.
pub fn median<T>(items: &[T]) -> T
where
    T: MathType + Ord,
{
    for item in items {
        let count = items.iter().filter(|&x| x <= item).count();
        if count == items.len() / 2 + 1 {
            return *item;
        }
    }
    T::ZERO // empty list
}

pub fn nth_percentile_error<T, I, F, const N_RESULTS: usize>(
    iter: I,
    compare: F,
    percentiles: [u8; N_RESULTS],
) -> [T; N_RESULTS]
where
    T: MathType, // + Ord + From<i32>,
    I: ExactSizeIterator<Item = T>,
    F: Fn(&T, &T) -> std::cmp::Ordering,
{
    let mut res = [T::ZERO; N_RESULTS];
    if iter.len() == 0 {
        return res;
    }
    let mut items: Vec<_> = iter.collect();
    items.sort_unstable_by(compare);
    let items = items;
    for (i, percentile) in percentiles.into_iter().enumerate() {
        assert!(percentile <= 100);
        let index = usize::min(items.len() * percentile as usize / 100, items.len() - 1);
        // if items.len() & 0b1 != 0 {
        res[i] = items[index];
        // } else {
        //     res[i] = (items[index - 1] + items[index]) / T::from(2);
        // }
    }
    res
}

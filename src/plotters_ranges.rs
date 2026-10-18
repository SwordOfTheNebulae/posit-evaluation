use std::ops::Range;

use plotters::coord::ranged1d::{KeyPointHint, NoDefaultFormatting, Ranged, ValueFormatter};
use posit_project::{p8, p16, p32, p64};

// the rest of this file is based on, copied, and modified from plotters::coord::ranged1d::types::numeric
#[derive(Clone)]
pub struct RangedCoordf16(f16, f16);
impl From<Range<f16>> for RangedCoordf16 {
    fn from(range: Range<f16>) -> Self {
        return RangedCoordf16(range.start, range.end);
    }
}
impl Ranged for RangedCoordf16 {
    type FormatOption = NoDefaultFormatting;
    type ValueType = f16;
    #[allow(clippy::float_cmp)]
    fn map(&self, v: &f16, limit: (i32, i32)) -> i32 {
        // Corner case: If we have a range that have only one value,
        // then we just assign everything to the only point
        if self.1 == self.0 {
            return (limit.1 - limit.0) / 2;
        }

        let logic_length = (*v as f64 - self.0 as f64) / (self.1 as f64 - self.0 as f64);

        let actual_length = limit.1 - limit.0;

        if actual_length == 0 {
            return limit.1;
        }

        if logic_length.is_infinite() {
            if logic_length.is_sign_positive() {
                return limit.1;
            } else {
                return limit.0;
            }
        }

        if actual_length > 0 {
            return limit.0 + (actual_length as f64 * logic_length + 1e-3).floor() as i32;
        } else {
            return limit.0 + (actual_length as f64 * logic_length - 1e-3).ceil() as i32;
        }
    }
    fn key_points<Hint: KeyPointHint>(&self, hint: Hint) -> Vec<f16> {
        compute_f16_key_points((self.0, self.1), hint.max_num_points())
    }
    fn range(&self) -> Range<f16> {
        return self.0..self.1;
    }
}

impl ValueFormatter<f16> for RangedCoordf16 {
    fn format(value: &f16) -> String {
        plotters::data::float::FloatPrettyPrinter {
            allow_scientific: false,
            min_decimal: 1,
            max_decimal: 5,
        }
        .print(*value as f64)
    }
}

fn compute_f16_key_points(range: (f16, f16), max_points: usize) -> Vec<f16> {
    if max_points == 0 {
        return vec![];
    }

    let range = (range.0.min(range.1) as f64, range.1.max(range.0) as f64);

    assert!(!(range.0.is_nan() || range.1.is_nan()));

    if (range.0 - range.1).abs() < f64::EPSILON {
        return vec![range.0 as f16];
    }

    let mut scale = (10f64).powf((range.1 - range.0).log(10.0).floor());
    // The value granularity controls how we round the values.
    // To avoid generating key points like 1.00000000001, we round to the nearest multiple of the
    // value granularity.
    // By default, we make the granularity as the 1/10 of the scale.
    let mut value_granularity = scale / 10.0;
    fn rem_euclid(a: f64, b: f64) -> f64 {
        let ret = if b > 0.0 {
            a - (a / b).floor() * b
        } else {
            a - (a / b).ceil() * b
        };
        if (ret - b).abs() < f64::EPSILON {
            0.0
        } else {
            ret
        }
    }

    // At this point we need to make sure that the loop invariant:
    // The scale must yield number of points than requested
    if 1 + ((range.1 - range.0) / scale).floor() as usize > max_points {
        scale *= 10.0;
        value_granularity *= 10.0;
    }

    'outer: loop {
        let old_scale = scale;
        for nxt in [2.0, 5.0, 10.0].iter() {
            let mut new_left = range.0 - rem_euclid(range.0, old_scale / nxt);
            if new_left < range.0 {
                new_left += old_scale / nxt;
            }
            let new_right = range.1 - rem_euclid(range.1, old_scale / nxt);

            let npoints = 1.0 + ((new_right - new_left) / old_scale * nxt);

            if npoints.round() as usize > max_points {
                break 'outer;
            }

            scale = old_scale / nxt;
        }
        scale = old_scale / 10.0;
        value_granularity /= 10.0;
    }

    let mut ret = vec![];
    // In some extreme cases, left might be too big, so that (left + scale) - left == 0 due to
    // floating point error.
    // In this case, we may loop forever. To avoid this, we need to use two variables to store
    // the current left value. So we need keep a left_base and a left_relative.
    let left = {
        let mut value = range.0 - rem_euclid(range.0, scale);
        if value < range.0 {
            value += scale;
        }
        value
    };
    let left_base = (left / value_granularity).floor() * value_granularity;
    let mut left_relative = left - left_base;
    let right = range.1 - rem_euclid(range.1, scale);
    while (right - left_relative - left_base) >= -f64::EPSILON {
        let new_left_relative = (left_relative / value_granularity).round() * value_granularity;
        if new_left_relative < 0.0 {
            left_relative += value_granularity;
        }
        ret.push((left_relative + left_base) as f16);
        left_relative += scale;
    }
    return ret;
}

#[derive(Clone)]
pub struct RangedCoordf128(f128, f128);
impl From<Range<f128>> for RangedCoordf128 {
    fn from(range: Range<f128>) -> Self {
        return RangedCoordf128(range.start, range.end);
    }
}
impl Ranged for RangedCoordf128 {
    type FormatOption = NoDefaultFormatting;
    type ValueType = f128;
    #[allow(clippy::float_cmp)]
    fn map(&self, v: &f128, limit: (i32, i32)) -> i32 {
        // Corner case: If we have a range that have only one value,
        // then we just assign everything to the only point
        if self.1 == self.0 {
            return (limit.1 - limit.0) / 2;
        }

        let logic_length = (*v as f64 - self.0 as f64) / (self.1 as f64 - self.0 as f64);

        let actual_length = limit.1 - limit.0;

        if actual_length == 0 {
            return limit.1;
        }

        if logic_length.is_infinite() {
            if logic_length.is_sign_positive() {
                return limit.1;
            } else {
                return limit.0;
            }
        }

        if actual_length > 0 {
            return limit.0 + (actual_length as f64 * logic_length + 1e-3).floor() as i32;
        } else {
            return limit.0 + (actual_length as f64 * logic_length - 1e-3).ceil() as i32;
        }
    }
    fn key_points<Hint: KeyPointHint>(&self, hint: Hint) -> Vec<f128> {
        compute_f128_key_points((self.0, self.1), hint.max_num_points())
    }
    fn range(&self) -> Range<f128> {
        return self.0..self.1;
    }
}

impl ValueFormatter<f128> for RangedCoordf128 {
    fn format(&value: &f128) -> String {
        if value < f64::EPSILON as f128 || value > f64::MAX_EXACT_INTEGER as f128 {
            format!("{value:.5?}")
        } else {
            plotters::data::float::FloatPrettyPrinter {
                allow_scientific: false,
                min_decimal: 1,
                max_decimal: 5,
            }
            .print(value as f64)
        }
    }
}

fn compute_f128_key_points(range: (f128, f128), max_points: usize) -> Vec<f128> {
    if max_points == 0 {
        return vec![];
    }

    let range = (range.0.min(range.1) as f128, range.1.max(range.0) as f128);

    assert!(!(range.0.is_nan() || range.1.is_nan()));

    if (range.0 - range.1).abs() < f128::EPSILON {
        return vec![range.0 as f128];
    }

    let mut scale = (10f128).powf((range.1 - range.0).log(10.0).floor());
    // The value granularity controls how we round the values.
    // To avoid generating key points like 1.00000000001, we round to the nearest multiple of the
    // value granularity.
    // By default, we make the granularity as the 1/10 of the scale.
    let mut value_granularity = scale / 10.0;
    fn rem_euclid(a: f128, b: f128) -> f128 {
        let ret = if b > 0.0 {
            a - (a / b).floor() * b
        } else {
            a - (a / b).ceil() * b
        };
        if (ret - b).abs() < f128::EPSILON {
            0.0
        } else {
            ret
        }
    }

    // At this point we need to make sure that the loop invariant:
    // The scale must yield number of points than requested
    if 1 + ((range.1 - range.0) / scale).floor() as usize > max_points {
        scale *= 10.0;
        value_granularity *= 10.0;
    }

    'outer: loop {
        let old_scale = scale;
        for nxt in [2.0, 5.0, 10.0].iter() {
            let mut new_left = range.0 - rem_euclid(range.0, old_scale / nxt);
            if new_left < range.0 {
                new_left += old_scale / nxt;
            }
            let new_right = range.1 - rem_euclid(range.1, old_scale / nxt);

            let npoints = 1.0 + ((new_right - new_left) / old_scale * nxt);

            if npoints.round() as usize > max_points {
                break 'outer;
            }

            scale = old_scale / nxt;
        }
        scale = old_scale / 10.0;
        value_granularity /= 10.0;
    }

    let mut ret = vec![];
    // In some extreme cases, left might be too big, so that (left + scale) - left == 0 due to
    // floating point error.
    // In this case, we may loop forever. To avoid this, we need to use two variables to store
    // the current left value. So we need keep a left_base and a left_relative.
    let left = {
        let mut value = range.0 - rem_euclid(range.0, scale);
        if value < range.0 {
            value += scale;
        }
        value
    };
    let left_base = (left / value_granularity).floor() * value_granularity;
    let mut left_relative = left - left_base;
    let right = range.1 - rem_euclid(range.1, scale);
    while (right - left_relative - left_base) >= -f128::EPSILON {
        let new_left_relative = (left_relative / value_granularity).round() * value_granularity;
        if new_left_relative < 0.0 {
            left_relative += value_granularity;
        }
        ret.push((left_relative + left_base) as f128);
        left_relative += scale;
    }
    return ret;
}

////

macro_rules! make_ranged_coord_posit {
    ($( $pNN:ident, $RangedCoordpNN:ident, $compute_pNN_key_points:ident );+ $(;)?) => {$(

#[derive(Clone)]
pub struct $RangedCoordpNN($pNN, $pNN);
impl From<Range<$pNN>> for $RangedCoordpNN {
    fn from(range: Range<$pNN>) -> Self {
        return $RangedCoordpNN(range.start, range.end);
    }
}
impl Ranged for $RangedCoordpNN {
    type FormatOption = NoDefaultFormatting;
    type ValueType = $pNN;
    fn map(&self, v: &$pNN, limit: (i32, i32)) -> i32 {
        // Corner case: If we have a range that have only one value,
        // then we just assign everything to the only point
        if self.1 == self.0 {
            return (limit.1 - limit.0) / 2;
        }

        let logic_length = (*v - self.0) / (self.1 - self.0);

        let actual_length = limit.1 - limit.0;

        if actual_length == 0 {
            return limit.1;
        }

        if logic_length == $pNN::NAR {
            if self.1 > self.0 {
                return limit.1;
            } else {
                return limit.0;
            }
        }

        return limit.0
            + <$pNN as Into<i32>>::into(if actual_length > 0 {
                ($pNN::from(actual_length) * logic_length + $pNN::from(1e-3)).floor()
            } else {
                ($pNN::from(actual_length) * logic_length - $pNN::from(1e-3)).ceil()
            });
    }
    fn key_points<Hint: KeyPointHint>(&self, hint: Hint) -> Vec<$pNN> {
        $compute_pNN_key_points((self.0, self.1), hint.max_num_points())
    }
    fn range(&self) -> Range<$pNN> {
        return self.0..self.1;
    }
}

impl ValueFormatter<$pNN> for $RangedCoordpNN {
    fn format(&value: &$pNN) -> String {
        plotters::data::float::FloatPrettyPrinter {
            allow_scientific: false,
            min_decimal: 1,
            max_decimal: 5,
        }
        .print(value.into())
    }
}

fn $compute_pNN_key_points(range: ($pNN, $pNN), max_points: usize) -> Vec<$pNN> {
    if max_points == 0 {
        return vec![];
    }

    let range = (range.0.min(range.1), range.1.max(range.0));

    assert!(range.0 != $pNN::NAR && range.1 != $pNN::NAR);

    let mut scale = $pNN::from(
        (10f64).powf(
            <$pNN as Into<f64>>::into(range.1 - range.0)
                .log(10.0)
                .floor(),
        ),
    );
    // The value granularity controls how we round the values.
    // To avoid generating key points like 1.00000000001, we round to the nearest multiple of the
    // value granularity.
    // By default, we make the granularity as the 1/10 of the scale.
    let mut value_granularity = scale / $pNN::from(10.0);
    fn rem_euclid(a: $pNN, b: $pNN) -> $pNN {
        let ret = if b > $pNN::ZERO {
            a - (a / b).floor() * b
        } else {
            a - (a / b).ceil() * b
        };
        if ret == b { $pNN::ZERO } else { ret }
    }

    // At this point we need to make sure that the loop invariant:
    // The scale must yield number of points than requested
    if 1 + <$pNN as Into<i32>>::into(((range.1 - range.0) / scale).floor()) as usize > max_points {
        scale *= $pNN::from(10.0);
        value_granularity *= $pNN::from(10.0);
    }

    'outer: loop {
        let old_scale = scale;
        for &nxt in [$pNN::from(2.0), $pNN::from(5.0), $pNN::from(10.0)].iter() {
            let mut new_left = range.0 - rem_euclid(range.0, old_scale / nxt);
            if new_left < range.0 {
                new_left += old_scale / nxt;
            }
            let new_right = range.1 - rem_euclid(range.1, old_scale / nxt);

            let npoints = $pNN::ONE + ((new_right - new_left) / old_scale * nxt);

            if <$pNN as Into<i32>>::into(npoints.round()) as usize > max_points {
                break 'outer;
            }

            scale = old_scale / nxt;
        }
        scale = old_scale / $pNN::from(10);
        value_granularity /= $pNN::from(10);
    }

    let mut ret = vec![];
    // In some extreme cases, left might be too big, so that (left + scale) - left == 0 due to
    // floating point error.
    // In this case, we may loop forever. To avoid this, we need to use two variables to store
    // the current left value. So we need keep a left_base and a left_relative.
    let left = {
        let mut value = range.0 - rem_euclid(range.0, scale);
        if value < range.0 {
            value += scale;
        }
        value
    };
    let left_base = (left / value_granularity).floor() * value_granularity;
    let mut left_relative = left - left_base;
    let right = range.1 - rem_euclid(range.1, scale);
    while (right - left_relative - left_base) >= $pNN::ZERO {
        let new_left_relative = (left_relative / value_granularity).round() * value_granularity;
        if new_left_relative < $pNN::ZERO {
            left_relative += value_granularity;
        }
        ret.push((left_relative + left_base) as $pNN);
        left_relative += scale;
    }
    return ret;
}

    )+};
}

make_ranged_coord_posit!(
    p64, RangedCoordp64, compute_p64_key_points;
    p32, RangedCoordp32, compute_p32_key_points;
    p16, RangedCoordp16, compute_p16_key_points;
    // p8, RangedCoordp8, compute_p8_key_points;
);

#[derive(Clone)]
pub struct RangedCoordp8(p8, p8);
impl From<Range<p8>> for RangedCoordp8 {
    fn from(range: Range<p8>) -> Self {
        return RangedCoordp8(range.start, range.end);
    }
}
impl Ranged for RangedCoordp8 {
    type FormatOption = NoDefaultFormatting;
    type ValueType = p8;
    fn map(&self, &v: &p8, limit: (i32, i32)) -> i32 {
        let v = p16::from(v);
        // Corner case: If we have a range that have only one value,
        // then we just assign everything to the only point
        if self.1 == self.0 {
            return (limit.1 - limit.0) / 2;
        }

        let logic_length = (v - self.0.into()) / (self.1 - self.0).into();

        let actual_length = limit.1 - limit.0;

        if actual_length == 0 {
            return limit.1;
        }

        if logic_length == p16::NAR {
            if self.1 > self.0 {
                return limit.1;
            } else {
                return limit.0;
            }
        }

        return limit.0
            + <p16 as Into<i32>>::into(if actual_length > 0 {
                (p16::from(actual_length) * logic_length + p16::from(1e-3)).floor()
            } else {
                (p16::from(actual_length) * logic_length - p16::from(1e-3)).ceil()
            });
    }
    fn key_points<Hint: KeyPointHint>(&self, hint: Hint) -> Vec<p8> {
        compute_p16_key_points((self.0.into(), self.1.into()), hint.max_num_points())
            .into_iter()
            .map(p16::into)
            .collect()
    }
    fn range(&self) -> Range<p8> {
        return self.0..self.1;
    }
}

impl ValueFormatter<p8> for RangedCoordp8 {
    fn format(&value: &p8) -> String {
        plotters::data::float::FloatPrettyPrinter {
            allow_scientific: false,
            min_decimal: 1,
            max_decimal: 5,
        }
        .print(value.into())
    }
}

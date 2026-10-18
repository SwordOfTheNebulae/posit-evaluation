#![feature(
    float_conversions,
    iter_collect_into,
    const_trait_impl,
    array_try_map,
    const_result_trait_fn,
    f128,
    f16,
    float_exact_integer_constants
)]
use posit_project::{MathType, SliceFormatter, inverse_ftt, mae, mean, median, mse};
use posit_project::{
    PositNoQuireContextWithTaylorSeries, SimpleFloatContext, TaylorSeriesFloatContext,
    cross_correlate, fft, fx_correlate, nth_percentile_error, p8, p16, p32, p64, xf_correlate,
};
use rand::RngExt;
use rand::SeedableRng;
use rand::distr::Distribution;
use std::cell::UnsafeCell;
use std::io::Write;
use std::iter::zip;

pub mod colours;
use colours::Pallet;
pub mod plotters_ranges;

type Err = Box<dyn std::error::Error>;

#[inline]
fn deref<T: Copy>(x: &T) -> T {
    *x
}

fn rangef<R: Into<i64>>(end: R) -> impl Iterator<Item = f32> + Clone {
    (0..end.into()).map(|x| x as f32)
}
fn rangef64<R: Into<i64>>(end: R) -> impl Iterator<Item = f64> + Clone {
    (0..end.into()).map(|x| x as f64)
}
fn rangei<R: Into<i64>>(end: R) -> impl Iterator<Item = i32> + Clone {
    (0..end.into()).map(|x| x as i32)
}

fn with_x<'a, X, Y, I, S>(
    name: S,
    start: X,
    end: X,
    iter: I,
) -> (impl Iterator<Item = (GraphX, Y)> + Clone, S)
where
    X: Into<GraphX> + Copy,
    Y: Copy,
    I: Iterator<Item = &'a Y, Item: 'a> + Clone,
    S: Into<String>,
{
    (zip(start.into()..end.into(), iter.map(deref)), name)
}
fn with_x_by_val<X, Y, I, S>(
    name: S,
    start: X,
    end: X,
    iter: I,
) -> (impl Iterator<Item = (GraphX, Y)> + Clone, S)
where
    X: Into<GraphX> + Copy,
    Y: Copy,
    I: Iterator<Item = Y> + Clone,
    S: Into<String>,
{
    (zip(start.into()..end.into(), iter), name)
}

fn graph_cross_correlate_f32() -> Result<(), Err> {
    let f = |x: f32| {
        if x >= 0.33 && x < 0.66 { 1.0f32 } else { 0f32 }
    };
    let g = |x: f32| {
        if x >= 0.33 && x < 0.66 {
            (x - 0.33) * 2.0
        } else {
            0f32
        }
    };
    let a: Box<[f32]> = (0..64).map(|x| f(x as f32 / 64f32)).collect();
    let b: Box<[f32]> = (0..64).map(|x| g(x as f32 / 64f32)).collect();
    let res = cross_correlate(&a, &b).unwrap();
    // print!("{a:?}\n{b:?}\n{res:?}\n");

    make_graph(
        [
            (zip(0..64, a.iter().map(deref)), "a"),
            (zip(0..64, b.iter().map(deref)), "b"),
            (zip(-64..64, res.iter().map(deref)), "a * b"),
        ]
        .into_iter(),
        std::path::Path::new("graph_cross_f32.png"),
        (1024, 768),
    )
}

fn graph_fft_f32() -> Result<(), Err> {
    let c: Box<[f32]> = (0..512)
        .map(|x| f32::sin(x as f32 / 16. + std::f32::consts::PI / 2.) * 8.)
        .collect();
    let c_freq = fft::<f32, _, SimpleFloatContext>(&c);
    let mag: Box<[f32]> = c_freq
        .iter()
        .map(|(re, im)| (re * re + im * im).sqrt())
        .collect();

    let phase: Box<[f32]> = c_freq.iter().map(|(re, im)| im.atan2(*re)).collect();
    let range = || 0..512; //.map(f32::from);
    make_graph(
        [
            // (zip(range(), re.iter().map(deref)), "re"),
            // (zip(range(), im.iter().map(deref)), "im"),
            (zip(range(), mag.iter().map(deref)), "mag"),
            (zip(range(), phase.iter().map(deref)), "phase"),
            (zip(range(), c.iter().map(deref)), "c"),
        ]
        .into_iter(),
        std::path::Path::new("graph_fft_f32.png"),
        (2460, 1440),
    )
}

fn graph_fx_xf_f32() -> Result<(), Err> {
    // let range = || (0..512i16).map(f32::from);
    let size = 1u32 << 14;
    let s1: Box<_> = rangef(size)
        .map(|x| f32::sin(2. * std::f32::consts::PI * (x / 10.)))
        .collect();
    let s2: Box<_> = rangef(size)
        .map(|x| f32::sin(2. * std::f32::consts::PI * (x / 10.) - 1.2345))
        .collect();
    let xf: Box<_> = xf_correlate::<f32, SimpleFloatContext>(&*s1, &*s2)
        .unwrap()
        .iter()
        .map(|(re, im)| (re * re + im * im).sqrt())
        .collect();
    make_graph(
        [(zip(rangei(size), xf.to_vec().iter().map(deref)), "xf")].into_iter(),
        std::path::Path::new("graph_xf_f32.png"),
        (2460, 1440),
    )?;
    let fx: Box<_> = fx_correlate::<f32, SimpleFloatContext>(&*s1, &*s2)
        .iter()
        .map(|(re, im)| (re * re + im * im).sqrt())
        .collect();
    make_graph(
        [(
            zip(rangei(size / 2), fx[0..size as usize / 2].iter().map(deref)),
            "xf",
        )]
        .into_iter(),
        std::path::Path::new("graph_fx_f32.png"),
        (2460, 1440),
    )
}

fn graph_fx_xf_p32() -> Result<(), Err> {
    // let range = || (0..512i16).map(f32::from);
    let size = 1u32 << 14;
    let size_f = size as f64;
    let s1 = rangef64(size).map(|x| f64::sin(F64_TAU * (x / 10.)));
    let s2 = rangef64(size).map(|x| f64::sin(F64_TAU * (x / 10. - f64::sin(x / size_f * F64_PI))));
    let s1: Box<_> = s1.map(p32::from).collect();
    let s2: Box<_> = s2.map(p32::from).collect();
    let xf: Box<_> = xf_correlate::<p32, PositNoQuireContextWithTaylorSeries>(&*s1, &*s2)
        .unwrap()
        .iter()
        .map(|(re, im)| (re * re + im * im).sqrt())
        .collect();
    make_graph(
        [(zip(rangei(size), xf.to_vec().iter().map(deref)), "xf")].into_iter(),
        std::path::Path::new("graph_xf_p32.png"),
        (2460, 1440),
    )?;
    let fx: Box<_> = fx_correlate::<p32, PositNoQuireContextWithTaylorSeries>(&*s1, &*s2)
        .iter()
        .map(|(re, im)| (re * re + im * im).sqrt())
        .collect();
    make_graph(
        [(
            zip(rangei(size / 2), fx[0..size as usize / 2].iter().map(deref)),
            "xf",
        )]
        .into_iter(),
        std::path::Path::new("graph_fx_p32.png"),
        (2460, 1440),
    )
}

fn graph_cross_correlate_p32() -> Result<(), Err> {
    let f = |x: p32| {
        if x >= p32::from(0.33) && x < p32::from(0.66) {
            p32::ONE
        } else {
            p32::ZERO
        }
    };
    let g = |x: p32| {
        if x >= p32::from(0.33) && x < p32::from(0.66) {
            (x - p32::from(0.33)) * p32::from(2.0)
        } else {
            p32::ZERO
        }
    };
    let a: Box<[p32]> = (0..64).map(|x| f(p32::from(x) / p32::from(64))).collect();
    let b: Box<[p32]> = (0..64).map(|x| g(p32::from(x) / p32::from(64))).collect();
    let res = cross_correlate(&a, &b).unwrap();
    // print!("{a:?}\n{b:?}\n{res:?}\n");

    make_graph(
        [
            with_x("a", 0, 64, a.iter()),
            with_x("b", 0, 64, b.iter()),
            with_x("a * b", -64, 64, res.iter()),
        ]
        .into_iter(),
        std::path::Path::new("graph_cross_p32.png"),
        (1024, 768),
    )
}

fn graph_fft_p32() -> Result<(), Err> {
    let c: Box<[p32]> = (0..512)
        .map(|x| (f64::sin(x as f64 / 16. + std::f64::consts::PI / 2.) * 8.).into())
        .collect();
    let c_freq = fft::<p32, _, PositNoQuireContextWithTaylorSeries>(&c);
    let mag: Box<[p32]> = c_freq
        .iter()
        .map(|(re, im)| (re * re + im * im).sqrt())
        .collect();

    // let phase: Box<[p32]> = c_freq.iter().map(|(re, im)| im.atan2(*re)).collect();
    let range = || 0..512; //.map(f32::from);
    make_graph(
        [
            (zip(range(), mag.iter().map(deref)), "mag"),
            // (zip(range(), phase.iter().map(deref)), "phase"),
            (zip(range(), c.iter().map(deref)), "c"),
        ]
        .into_iter(),
        std::path::Path::new("graph_fft_p32.png"),
        (2460, 1440),
    )
}
// use std::f32::consts::PI as F32_PI;
use std::f64::consts::PI as F64_PI;
use std::f64::consts::TAU as F64_TAU;
fn graph_xf_f32_vs_p32() -> Result<(), Err> {
    // let range = || (0..512i16).map(f32::from);
    let size = 1 << 14;
    let size_f = size as f64;
    let s1 = rangef64(size).map(|x| f64::sin(F64_TAU * (x / 10.)));
    let s2 = rangef64(size).map(|x| f64::sin(F64_TAU * (x / 10. - f64::sin(x / size_f * F64_PI))));

    let f_xf = xf_correlate::<f32, SimpleFloatContext>(
        &*s1.clone().map(f64::cast).collect::<Box<_>>(),
        &*s2.clone().map(f64::cast).collect::<Box<_>>(),
    )
    .unwrap()
    .into_iter()
    .map(|(re, im)| (re * re + im * im).sqrt());

    let p_xf = xf_correlate::<p32, PositNoQuireContextWithTaylorSeries>(
        &*s1.map(p32::from).collect::<Box<_>>(),
        &*s2.map(p32::from).collect::<Box<_>>(),
    )
    .unwrap()
    .into_iter()
    .map(|(re, im)| (re * re + im * im).sqrt());

    let diff = zip(p_xf, f_xf).map(|(p, f)| <p32 as Into<f64>>::into(p) - f as f64);
    make_graph(
        [with_x_by_val("(p32 as f64) - (f32 as f64)", 0, size, diff)].into_iter(),
        std::path::Path::new("graph_xf_f32_vs_p32.png"),
        (2460, 1440),
    )
}
fn graph_fft_f32_vs_p32() -> Result<(), Err> {
    // let range = || (0..512i16).map(f32::from);
    let size = 1 << 14;
    let c = rangef64(size).map(|x| f64::sin(2. * std::f64::consts::PI * (x / 10.)));

    let mag_f32 =
        fft::<f32, f32, SimpleFloatContext>(&*c.clone().map(f64::cast).collect::<Box<_>>())
            .into_iter()
            .map(|(re, im)| (re * re + im * im).sqrt());
    let mag_p32 = fft::<p32, p32, PositNoQuireContextWithTaylorSeries>(
        &*c.map(f64::into).collect::<Box<_>>(),
    )
    .into_iter()
    .map(|(re, im)| (re * re + im * im).sqrt());

    let diff = zip(mag_p32, mag_f32).map(|(p, f)| <p32 as Into<f64>>::into(p) - f as f64);
    make_graph(
        [with_x_by_val("(p32 as f64) - (f32 as f64)", 0, size, diff)].into_iter(),
        std::path::Path::new("graph_fft_f32_vs_p32.png"),
        (2460, 1440),
    )
}

pub fn test_taylor_series() {
    let two_pi = p32::from(std::f64::consts::TAU);
    let one_hundred = p32::from(100);
    fn calc(x: p32, degree: usize) -> (p32, p32) {
        let mut sin = p32::ZERO;
        let mut cos = p32::ZERO;
        let mut factorial: i64 = 1; //since n = 0, starts at (2n)! = 1. also 20! < i64::MAX
        let mut pow: p32 = p32::ONE; //since n = 0, starts at x^{2n} = 1
        for n in 0..degree {
            cos += if n & 1 == 0 { pow } else { -pow } / p32::from(factorial);
            factorial *= (2 * n + 1) as i64; // calc (2n + 1)!
            pow *= x; // calc x^(2n + 1)
            sin += if n & 1 == 0 { pow } else { -pow } / p32::from(factorial);
            factorial *= (2 * (n + 1)) as i64; // calc next (2n)!
            pow *= x; // calc next x^(2n)
        }
        (sin, cos)
    }
    fn calc2(x: p32, degree: usize) -> (p32, p32) {
        let two = p32::from(2);
        let mut sin = p32::ZERO;
        let mut cos = p32::ZERO;
        let mut factorial: p32 = p32::ONE; //since n = 0, starts at (2n)! = 1
        let mut pow: p32 = p32::ONE; //since n = 0, starts at x^{2n} = 1
        for n in 0..degree {
            cos += if n & 1 == 0 { pow } else { -pow } / p32::from(factorial);
            factorial *= two * p32::from(n as i32) + p32::ONE; // calc (2n + 1)!
            pow *= x; // calc x^(2n + 1)
            sin += if n & 1 == 0 { pow } else { -pow } / p32::from(factorial);
            factorial *= two * (p32::from(n as i32) + p32::ONE); // calc next (2n)!
            pow *= x; // calc next x^(2n)
        }
        (sin, cos)
    }
    let mut histogram1 = [0u16; 20];
    let mut histogram2 = [0u16; 50];
    let mut fails1 = 0;
    let mut fails2 = 0;
    let mut values1 = [(false, p32::ZERO, p32::ZERO); 100];
    let mut values2 = [(false, p32::ZERO, p32::ZERO); 100];
    for index in 0..100 {
        let value = p32::from(index as i32) / one_hundred;
        let x = value * two_pi;
        let mut calc_done = false;
        let mut calc_done_sin = false;
        let mut calc_done_cos = false;
        let (mut last_sin, mut last_cos) = calc(x, 1);
        for degree in 2..20 {
            let last_degree = degree - 1;
            let (sin, cos) = calc(x, degree);
            if sin == last_sin && cos == last_cos {
                println!("(1) degree {last_degree} is sufficient at {x} (2π {value})");
                histogram1[last_degree] += 1;
                calc_done = true;
                values1[index].0 = true;
                break;
            }
            if !calc_done_sin && sin == last_sin {
                println!("(1) degree {last_degree} is sufficient at {x} (2π {value}) for sin only");
                calc_done_sin = true;
            }
            if !calc_done_cos && cos == last_cos {
                println!("(1) degree {last_degree} is sufficient at {x} (2π {value}) for cos only");
                calc_done_cos = true;
            }
            (last_sin, last_cos) = (sin, cos)
        }
        if !calc_done {
            println!("(1) degree 20 possibly insufficient at {x} (2π {value})");
            fails1 += 1;
        }
        (values1[index].1, values1[index].2) = (last_sin, last_cos);
    }
    for index in 0..100 {
        let value = p32::from(index as i32) / one_hundred;
        let x = value * two_pi;
        let mut calc_done = false;
        let mut calc_done_sin = false;
        let mut calc_done_cos = false;
        let (mut last_sin, mut last_cos) = calc2(x, 1);
        for degree in 2..50 {
            let last_degree = degree - 1;
            let (sin, cos) = calc2(x, degree);
            if sin == last_sin && cos == last_cos {
                println!("(2) degree {last_degree} is sufficient at {x} (2π {value})");
                histogram2[last_degree] += 1;
                calc_done = true;
                values2[index].0 = true;
                break;
            }
            if !calc_done_sin && sin == last_sin {
                println!("(2) degree {last_degree} is sufficient at {x} (2π {value}) for sin only");
                calc_done_sin = true;
            }
            if !calc_done_cos && cos == last_cos {
                println!("(2) degree {last_degree} is sufficient at {x} (2π {value}) for cos only");
                calc_done_cos = true;
            }
            (last_sin, last_cos) = (sin, cos)
        }
        if !calc_done {
            println!("(2) max degrees possibly insufficient at {x} (2π {value})");
            fails2 += 1;
        }
        (values2[index].1, values2[index].2) = (last_sin, last_cos);
    }
    println!(
        "iteration count histogram for using ints to calculate factorials (limited to 64 bit integer limit)\n{histogram1:?}\nfails: {fails1}"
    );
    println!(
        "iteration count histogram for using posits to calculate factorials\n{histogram2:?}\nfails: {fails2}"
    );
    let non_matching_count = values1
        .iter()
        .zip(values2.iter())
        .fold((0, 0), |(s, c), (a, b)| {
            (
                s + ((a.0 && b.0 && a.1 != b.1) as usize),
                c + ((a.0 && b.0 && a.2 != b.2) as usize),
            )
        });
    println!(
        "number of values that differ when the compution hit precision limit (sin, cos): {non_matching_count:?}"
    );

    // 2π 0.75 is problematic, lets see how many iterations it takes
    {
        const LIMIT: usize = u16::MAX as usize;
        let x = p32::from(3) / p32::from(4) * two_pi;
        let two = p32::from(2);
        let mut sin = p32::ZERO;
        let mut cos = p32::ZERO;
        let mut factorial: p32 = p32::ONE; //since n = 0, starts at (2n)! = 1
        let mut pow: p32 = p32::ONE; //since n = 0, starts at x^{2n} = 1
        let mut last_sin = sin;
        let mut last_cos = cos;
        for n in 0..LIMIT {
            cos += if n & 1 == 0 { pow } else { -pow } / p32::from(factorial);
            factorial *= two * p32::from(n as i32) + p32::ONE; // calc (2n + 1)!
            pow *= x; // calc x^(2n + 1)
            sin += if n & 1 == 0 { pow } else { -pow } / p32::from(factorial);
            factorial *= two * (p32::from(n as i32) + p32::ONE); // calc next (2n)!
            pow *= x; // calc next x^(2n)
            if last_sin == sin && last_cos == cos {
                println!("2π0.75 took {n} iterations to converge");
                break;
            }
            (last_sin, last_cos) = (sin, cos);
            if n >= LIMIT as usize {
                println!("2π0.75 didn't converge in {LIMIT} iterations");
                break;
            }
        }
        println!("its value is sin = {sin} cos = {cos}"); // this value is very wrong wtf
        // i suppose it doesnt matter i only care about 0 to π
    }

    let mut values_f64 = [(0f64, 0f64); 100];
    for index in 0..100 {
        let x = index as f64 / 100. * std::f64::consts::TAU;
        values_f64[index] = x.sin_cos();
    }
    let float_diff: (Vec<_>, Vec<_>) = values2
        .iter()
        .zip(values_f64.iter())
        .map(|(p, f)| (p.1 - p32::from(f.0), p.2 - p32::from(f.1)))
        .unzip();
    println!("{}", SliceFormatter(&*float_diff.0));
    // println!("{}", SliceFormatter(&*float_diff.1));
    println!("p32 vs f64 sin and cos");
    let mse_sin_f64 = mse(float_diff.0.iter().map(deref));
    let mse_cos_f64 = mse(float_diff.1.iter().map(deref));
    // MSE of 0.0097 is probably good enough for this??
    println!("MSE {mse_sin_f64} {mse_cos_f64}");
    let mae_sin_f64 = mae(float_diff.0.iter().map(deref));
    let mae_cos_f64 = mae(float_diff.1.iter().map(deref));
    println!("MAE {mae_sin_f64} {mae_cos_f64}");
    let mean_sin_f64 = mean(float_diff.0.iter().map(deref));
    let mean_cos_f64 = mean(float_diff.1.iter().map(deref));
    println!("mean {mean_sin_f64} {mean_cos_f64}");
    let median_sin_f64 = median(&*float_diff.0);
    let median_cos_f64 = median(&*float_diff.1);
    println!("median {median_sin_f64} {median_cos_f64}");
}

mod the_actual_thing;
use the_actual_thing::thereandbackagain;

const PALLET: Pallet<4> = Pallet::from([b"#3498DB", b"#DAF7A6", b"#FFC300", b"#FF5733"])
    .ok()
    .unwrap();

fn _remap<T: MathType + Ord>(val: T, old_min: T, old_max: T, new_min: T, new_max: T) -> T {
    debug_assert!(old_max > old_min);
    debug_assert!(new_max > new_min);
    let t = val - old_min / (old_max - old_min);
    t * (new_max - new_min) + new_min
}

pub fn foobarrenameme() -> Result<(), Err> {
    let n_steps: usize = 1024;
    let size = 1u32 << 10; //14; // samples per step
    let output_size = (size as usize).next_power_of_two();
    let size_f = size as f64;
    let time_per_step = 10000.; // lets say this is ms
    let _time_per_sample = size_f / time_per_step;
    const _ONE_DAY: f64 = 1000. * 60. * 60. * 24.;
    let total_time = n_steps as f64 * time_per_step;
    let base_freq = 1_000.; // 1 kz
    let rng = UnsafeCell::new(rand::rngs::SmallRng::seed_from_u64(0));
    // safety: rng is only ever referenced here, and the reference never leaves this function
    let random = || unsafe { &mut *rng.get() }.random::<f64>() - 0.5;
    let x_to_ms_at = move |t| move |x| x * (time_per_step / size_f) + t;
    let ms_to_signal_1 = move |x| f64::sin(F64_TAU * x * base_freq) + random() * 10.;
    let ms_to_signal_2 = move |x| {
        f64::sin(F64_TAU * x * base_freq - f64::sin(x / total_time * 50. * F64_PI)) + random() * 10.
    };
    let mk_data = move |t| {
        (
            rangef64(size).map(x_to_ms_at(t)).map(ms_to_signal_1),
            rangef64(size).map(x_to_ms_at(t)).map(ms_to_signal_2),
        )
    };
    let mut buffer1 = Vec::new();
    let mut buffer2 = Vec::new();
    let mut res = Vec::new();
    let start_time = std::time::Instant::now();
    let mut stderr = std::io::stderr().lock();
    for step in 0..n_steps {
        let time_elapsed = std::time::Instant::now() - start_time;
        write!(
            stderr,
            "\r\x1b[Kstep {}/{n_steps}\t{time_elapsed:.2?},\test. {:.2?} remaining",
            step + 1,
            time_elapsed * (n_steps - step) as u32 / (step + 1) as u32
        )?;
        std::io::stdout().flush()?;
        buffer1.clear();
        buffer2.clear();
        let (s1, s2) = mk_data(step as f64 * time_per_step);
        let data = fx_correlate::<p32, PositNoQuireContextWithTaylorSeries>(
            s1.map(p32::from).collect_into(&mut buffer1),
            s2.map(p32::from).collect_into(&mut buffer2),
        );
        res.extend(data.into_iter().map(|(re, im)| (re * re + im * im).sqrt()));
    }
    let time_elapsed = std::time::Instant::now() - start_time;
    writeln!(stderr, "\r\x1b[K{n_steps} steps took {time_elapsed:.2?}",)?;
    writeln!(stderr, "drawing graph")?;
    let (min, max) = res.iter().fold((p32::MAX, p32::MIN), |(min, max), val| {
        (min.min(*val), max.max(*val))
    });
    let scale = if max - min <= p32::ZERO {
        p32::ONE
    } else {
        max - min
    };
    println!("{min} {max}");
    let image = image::ImageBuffer::from_fn(n_steps as u32, output_size as u32, |x, y| {
        image::Rgb(
            PALLET.get(
                (((res[x as usize * output_size + y as usize] - min) / scale) * p32::from(255i16))
                    .into(),
            ),
        )
    });
    image.save("aaaaa.png")?;
    Ok(())
}

fn usage(argv0: &str) -> Result<(), std::io::Error> {
    writeln!(
        std::io::stderr(),
        "usage:
    {argv0} <command>... [<option>...]
available commands:
    roundtrip             ; generate a bunch of graphs
    (todo...)             ; there are a bunch of random things from testing that are still usable
available options:
    --image_size=<x>,<y>  ; set image size for graphs generated by `roundtrip` command (defaults to \"2024,2024\")
    --graphdir=<directory>  ; set out directory for graphs generated by `roundtrip` command (defaults to \"graphs\")
    --csvdir=<directory>  ; set out directory for csv files generated by `roundtrip` command (defaults to \"csv\")
    --help"
    )
}

pub fn main() -> Result<(), Err> {
    let mut args = std::env::args();
    let argv0 = args.next().unwrap_or_else(|| "program_name".into());
    let mut image_size = (2048, 2048);
    let mut graphdir = None;
    let mut datadir = None;
    let mut n_commands = 0usize;
    let mut n_fails = 0usize;
    for arg in &mut args {
        if arg.starts_with("--") {
            if let Some((key, val)) = arg[2..].split_once('=') {
                match key {
                    "image_size" => {
                        let split = val.split_once(',').map(|(x, y)| (x.parse(), y.parse()));
                        image_size = match split {
                            Some((Ok(x), Ok(y))) => (x, y),
                            Some((Err(e), _) | (_, Err(e))) => {
                                writeln!(
                                    std::io::stderr(),
                                    "expected \"<number>,<number>\" for image size, got \"{val}\", ({e})"
                                )?;
                                std::process::exit(2);
                            }
                            None => {
                                writeln!(
                                    std::io::stderr(),
                                    "expected \"<number>,<number>\" for image size, got \"{val}\" (missing ',')"
                                )?;
                                std::process::exit(2);
                            }
                        }
                    }
                    "graphdir" => graphdir = Some(val.to_owned()),
                    "csvdir" => datadir = Some(val.to_owned()),
                    _ => {
                        writeln!(std::io::stderr(), "unknown option \"--{key}\"")?;
                        std::process::exit(2);
                    }
                }
            } else {
                match &*arg {
                    "--help" => {
                        usage(&argv0)?;
                        return Ok(());
                    }
                    _ => {
                        writeln!(std::io::stderr(), "unknown option \"{arg}\"")?;
                        std::process::exit(2);
                    }
                }
            }
        } else {
            n_commands += 1;
            match &*arg {
                "-h" | "-?" | "/?" => {
                    usage(&argv0)?;
                    return Ok(());
                }
                "all" => {
                    println!("cross correlate f32");
                    graph_cross_correlate_f32()?;
                    println!("fft f32");
                    graph_fft_f32()?;
                    println!("fx xf f32");
                    graph_fx_xf_f32()?;
                    println!("fx xf p32");
                    graph_fx_xf_p32()?;
                    println!("cross correlate p32");
                    graph_cross_correlate_p32()?;
                    println!("xf f32 vs p32");
                    graph_xf_f32_vs_p32()?;
                    println!("fft f32 vs p32");
                    graph_fft_f32_vs_p32()?;
                    println!("fft p32");
                    graph_fft_p32()?;
                    break;
                }
                "cc_p32" => graph_cross_correlate_p32()?,
                "cc_f32" => graph_cross_correlate_f32()?,
                "fft_f32" => graph_fft_f32()?,
                "fx_xf_f32" => graph_fx_xf_f32()?,
                "fx_xf_p32" => graph_fx_xf_p32()?,
                "xf_f32_vs_p32" => graph_xf_f32_vs_p32()?,
                "fft_f32_vs_p32" => graph_fft_f32_vs_p32()?,
                "fft_p32" => graph_fft_p32()?,
                "foobar" => foobarrenameme()?,
                "tts" => test_taylor_series(),
                "roundtrip" => thereandbackagain(
                    datadir.as_deref().unwrap_or("csv"),
                    graphdir.as_deref().unwrap_or("graphs"),
                    image_size,
                )?,
                _ => {
                    writeln!(std::io::stderr(), "unknown command \"{arg}\"")?;
                    n_fails += 1
                }
            }
        }
    }
    if n_commands == 0 {
        usage(&argv0)?;
        std::process::exit(2);
    }
    if n_fails == n_commands {
        std::process::exit(2);
    }

    Ok(())
}

pub fn gplot<F, E>(filename: &std::path::Path, imgsize: (u32, u32), f: F) -> Result<(), Err>
where
    F: FnOnce(&mut gnuplot::Axes2D) -> Result<(), E>,
    E: Into<Err>,
{
    use std::ffi::OsStr;
    let mut fig = gnuplot::Figure::new();
    f(fig.axes2d()).map_err(E::into)?;
    match filename.extension().map(OsStr::to_str) {
        Some(Some("svg")) => fig.save_to_svg(filename, imgsize.0, imgsize.1)?,
        Some(Some("png")) => fig.save_to_png(filename, imgsize.0, imgsize.1)?,
        Some(Some("html")) => fig.save_to_canvas(filename, imgsize.0, imgsize.1)?,
        Some(Some("pdf")) => fig.save_to_pdf(filename, imgsize.0 as f32, imgsize.1 as f32)?,
        None => return Err("no file extension provided, cannot guess file type".into()),
        Some(None) => return Err("unknown file extension (bad utf8)".into()),
        Some(Some(ext)) => return Err(format!("supported file extension '{ext}'").into()),
    }
    Ok(())
}

type GraphX = i32;
type GraphY = f64;

pub fn make_graph<X, Y, T, I, S>(
    values: T,
    file_name: &std::path::Path,
    size: (u32, u32),
) -> Result<(), Box<dyn std::error::Error>>
where
    X: Into<GraphX> + Copy,
    Y: Into<GraphY> + Copy,
    T: Iterator<Item = (I, S)> + Clone,
    I: Iterator<Item = (X, Y)>,
    S: Into<String>,
{
    let (max_x, min_x, max_y, min_y) = values.clone().fold(
        (GraphX::MIN, GraphX::MAX, GraphY::MIN, GraphY::MAX),
        |(xmax, xmin, ymax, ymin), (data, _)| {
            data.map(|(x, y)| (x.into(), y.into())).fold(
                (xmax, xmin, ymax, ymin),
                |(xmax, xmin, ymax, ymin), (x, y)| {
                    (xmax.max(x), xmin.min(x), ymax.max(y), ymin.min(y))
                },
            )
        },
    );
    let (max_x, min_x, max_y, min_y) = (
        (max_x as f64 + 0.1 * (max_x - min_x) as f64) as GraphX,
        (min_x as f64 - 0.1 * (max_x - min_x) as f64) as GraphX,
        max_y + 0.1 * (max_y - min_y),
        min_y - 0.1 * (max_y - min_y),
    );
    use plotters::prelude::*;
    let drawing_area = BitMapBackend::new(file_name, size).into_drawing_area();

    drawing_area.fill(&WHITE)?;

    let mut chart = ChartBuilder::on(&drawing_area)
        .set_label_area_size(LabelAreaPosition::Left, 40)
        .set_label_area_size(LabelAreaPosition::Bottom, 40)
        .build_cartesian_2d(min_x..max_x, min_y..max_y)?;

    chart.configure_mesh().draw()?;

    for ((data, label), colour) in zip(values, [&RED, &BLUE, &BLACK].iter().cycle()) {
        chart
            .draw_series(LineSeries::new(
                data.map(|(x, y)| (x.into(), y.into())),
                colour,
            ))?
            .label(label);
    }
    Ok(())
}

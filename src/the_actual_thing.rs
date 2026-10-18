use std::convert::FloatToFloat;
use std::convert::identity;

use super::*;
use plotters_ranges::*;

const LENGTH: u32 = 1 << 12;
pub fn thereandbackagain(
    data_dir: &str,
    graph_dir: &str,
    image_size: (u32, u32),
) -> Result<(), Err> {
    let range = || rangef64(LENGTH);

    let graph_dir = if graph_dir.len() == 0 {
        "."
    } else {
        std::fs::create_dir_all(std::path::Path::new(graph_dir))?;
        graph_dir
    };
    let data_dir = if data_dir.len() == 0 {
        "."
    } else {
        std::fs::create_dir_all(std::path::Path::new(data_dir))?;
        data_dir
    };

    let signals: [(&str, Box<_>); _] = {
        let sin = f64::sin;
        let mut rng = rand::rngs::SmallRng::seed_from_u64(0);
        let rng_dist = rand::distr::Uniform::new(-1., 1.).unwrap();
        let small_rng_dist = rand::distr::Uniform::new(-0.4, 0.4).unwrap();
        [
            ("slow sine", range().map(|x| sin(x / 100.)).collect()),
            ("fast sine", range().map(|x| sin(x / 2.)).collect()),
            (
                "2 sines",
                range().map(|x| sin(x / 35.) + sin(x / 128.)).collect(),
            ),
            (
                "10 sines",
                range()
                    .map(|x| {
                        [17., 35., 51., 91., 111., 157., 191., 239., 281., 327., 359.]
                            .iter()
                            .map(|y| sin(x / y))
                            .sum()
                    })
                    .collect(),
            ),
            (
                "white noise",
                rng_dist
                    .sample_iter(&mut rng)
                    .take(LENGTH as usize)
                    .collect(),
            ),
            (
                "sine + white noise",
                range()
                    .map(|x| sin(x / 100.) + small_rng_dist.sample(&mut rng))
                    .collect(),
            ),
        ]
    };

    // f128 causes link errors on windows, oh well
    // it probably works on linux but i cant be bothered getting my laptop out

    // f128_simple(&signals, graph_dir, image_size)?;
    f64_simple(&signals, graph_dir, image_size, data_dir)?;
    f32_simple(&signals, graph_dir, image_size, data_dir)?;
    f16_simple(&signals, graph_dir, image_size, data_dir)?;

    // f128_tls(&signals,graph_dir, image_size,data_dir)?;
    f64_ts(&signals, graph_dir, image_size, data_dir)?;
    f32_ts(&signals, graph_dir, image_size, data_dir)?;
    f16_ts(&signals, graph_dir, image_size, data_dir)?;

    p64_ts(&signals, graph_dir, image_size, data_dir)?;
    p32_ts(&signals, graph_dir, image_size, data_dir)?;
    p16_ts(&signals, graph_dir, image_size, data_dir)?;
    p8_ts(&signals, graph_dir, image_size, data_dir)?;
    Ok(())
}
macro_rules! make_comparisons_for {
    ($($name:ident, $T:ident, $description:expr, $Ctx:ty, $signal_transform:expr, $atan2:expr, $cmp:expr, $range:ty);+ $(;)?) => {$(
fn $name(signals: &[(&str, Box<[f64]>)], graph_dir: &str, image_size: (u32, u32), data_dir: &str) -> Result<(), Err> {
    let max_signal_name_length = signals.iter().map(|(name, _)| name.len()).max().unwrap();
    let range = || 0..LENGTH;
    const TYPE: &str = concat!(stringify!($T), " ", $description);
    println!("{TYPE}:");
    for (signal_name, signal) in signals {
        let csv_name = format!("{data_dir}/({TYPE}) ({signal_name}) fft round trip test.csv");
        let mut csv_file = std::fs::File::create(csv_name)?;
        csv_file.write_all("mse,mae,min_err,median_err,p75_err,p90_err,p99_err,max_err\n".as_bytes())?;
        let signal = $signal_transform(signal);
        let mut last_signal = signal.clone();
        for i in 0..10 {
            let ffted = fft::<$T, _, $Ctx>(&last_signal);
            let iffted = inverse_ftt::<$T, $Ctx>(&ffted);

            let with_target = iffted.iter().zip(signal.iter().copied());
            let difference: Box<_> = with_target
                .map(|(&(res_re, res_im), target_re)| (res_re - target_re, res_im - cast!($T, 0)))
                .collect();
            let difference_length_squared = difference
                .iter()
                .map(|&(d_re, d_im)| d_re * d_re + d_im * d_im);
            let mse = difference_length_squared.clone().sum::<$T>() / cast!($T, signal.len() as i64);
            let difference_length: Vec<_> = difference_length_squared.map(|d| $T::sqrt(d)).collect();
            let mae = difference_length.iter().copied().sum::<$T>() / cast!($T, signal.len() as i64);
            let [min_err, median_err, p75_err, p90_err, p99_err, max_err] = nth_percentile_error(difference_length.iter().copied(), $cmp, [0, 50, 75, 90, 99, 100]);
            write!(csv_file, "{:.},{:.},{:.},{:.},{:.},{:.},{:.},{:.}\n", mse, mae, min_err, median_err, p75_err, p90_err, p99_err, max_err)?;
            // let max_err = difference_length.iter().copied().max_by($cmp).unwrap();
            // let min_err = difference_length.iter().copied().min_by($cmp).unwrap();
            if i == 0 || i == 4 || i == 9 {
                println!(
                    "{signal_name:>len$} {i}: mse {mse:.7e} mae {mae:.7e} min {min: <13.7e} max {max: <13.7e}",
                    len = max_signal_name_length,
                    mse = fmt_cast!($T, mse),
                    mae = fmt_cast!($T, mae),
                    min = fmt_cast!($T, min_err),
                    max = fmt_cast!($T, max_err),
                );
                let to_polar = |&(re, im)| ($T::sqrt(re * re + im * im), $atan2(im, re));
                let ffted_polar: Box<_> = ffted.iter().map(to_polar).collect();
                // let iffted_polar: Box<_> = iffted.iter().map(to_polar).collect();
                let [
                    (iffted_max, iffted_min),
                    (ffted_polar_max, ffted_polar_min),
                    (difference_max, difference_min),
                ] = [&iffted, &ffted_polar, &difference].map(|values| {
                    let ((max_re, max_im), (min_re, min_im)) = values.iter().fold(
                        (($T::MIN, $T::MIN), ($T::MAX, $T::MAX)),
                        |((re_max, im_max), (re_min, im_min)), &(re, im)| {
                            (
                                (re.max(re_max), im.max(im_max)),
                                (re.min(re_min), im.min(im_min)),
                            )
                        },
                    );
                    // remove infs and nans from floats
                    // let [max_re, max_im, min_re, min_im] = [max_re, max_im, min_re, min_im]
                    //     .map(|val| val.max($T::MIN).min($T::MAX));
                    fixup_floats!($T, [max_re, max_im, min_re, min_im]);
                    ((max_re, max_im), (min_re, min_im))
                });

                use plotters::prelude::*;
                use plotters::style::RelativeSize;
                let file_name = format!("{graph_dir}/({TYPE}) ({signal_name}) ({i}) fft round trip test.png",);
                let drawing_area = BitMapBackend::new(std::path::Path::new(&file_name), image_size)
                    .into_drawing_area();
                drawing_area.fill(&WHITE)?;

                let font = FontFamily::SansSerif;
                let text_style = (font, 20);

                let [drawing_area_top, drawing_area_middle, drawing_area_bottom] = drawing_area
                    .split_by_breakpoints(
                        [] as [RelativeSize; _],
                        [RelativeSize::Height(1. / 3.), RelativeSize::Height(2. / 3.)],
                    )
                    .try_into()
                    .ok()
                    .expect("returned vector should be length 3");
                let (drawing_area_top_left, drawing_area_top_right) =
                    drawing_area_top.split_horizontally(RelativeSize::Width(0.5));
                let (drawing_area_bottom_left, drawing_area_bottom_right) =
                    drawing_area_bottom.split_horizontally(RelativeSize::Width(0.5));

                let legend = |style| move |(x, y)| PathElement::new(vec![(x, y), (x + 20, y)], style);

                let signal_min = signal.iter().copied().min_by($cmp).unwrap();
                let signal_max = signal.iter().copied().max_by($cmp).unwrap();
                let mut top_left = ChartBuilder::on(&drawing_area_top_left)
                    .caption("Original Data ( x )", (font, 30))
                    .margin(20)
                    .set_label_area_size(LabelAreaPosition::Left, 40)
                    .set_label_area_size(LabelAreaPosition::Bottom, 40)
                    .build_cartesian_2d::<std::ops::Range<u32>,$range>(
                        0..LENGTH,
                        (signal_min.min(iffted_min.0)..signal_max.max(iffted_max.0)).into(),
                    )?;
                top_left.configure_mesh().label_style(text_style).draw()?;
                top_left.draw_series(LineSeries::new(range().zip(signal.iter().copied()), &BLACK))?;

                let mut top_right = ChartBuilder::on(&drawing_area_top_right)
                    .caption("Reconstructed Data ( IFFT(FFT(x)) )", (font, 30))
                    .margin(20)
                    .set_label_area_size(LabelAreaPosition::Right, 40)
                    .set_label_area_size(LabelAreaPosition::Bottom, 40)
                    .build_cartesian_2d::<std::ops::Range<u32>,$range>(
                        0..LENGTH,
                        (signal_min.min(iffted_min.0)..signal_max.max(iffted_max.0)).into(),
                    )?;
                top_right.configure_mesh().label_style(text_style).draw()?;
                top_right
                    .draw_series(LineSeries::new(
                        range().zip(iffted.iter().map(|&(re, _im)| re)),
                        &BLUE,
                    ))?
                    .label("Reconstructed (Real value)")
                    .legend(legend(&BLUE));
                top_right
                    .draw_series(LineSeries::new(
                        range().zip(iffted.iter().map(|&(_re, im)| im)),
                        &RED,
                    ))?
                    .label("Reconstructed (Imaginary value)")
                    .legend(legend(&RED));
                top_right
                    .configure_series_labels()
                    .border_style(BLACK)
                    .background_style(WHITE.mix(0.5))
                    .position(SeriesLabelPosition::UpperRight)
                    .label_font(text_style)
                    .draw()?;

                let mut middle = ChartBuilder::on(&drawing_area_middle)
                    .caption(
                        "Difference between Original and Reconstructed ( IFFT(FFT(x)) - x )",
                        (font, 30),
                    )
                    .margin(20)
                    .set_label_area_size(LabelAreaPosition::Left, 80)
                    .set_label_area_size(LabelAreaPosition::Bottom, 40)
                    .build_cartesian_2d::<std::ops::Range<u32>,$range>(
                        0..LENGTH,
                        (difference_min.0..difference_max.0).into()
                    )?;
                middle
                    .configure_mesh()
                    .y_label_formatter(&|y: &$T| format!("{y:.4e}", y=fmt_cast!($T, *y)))
                    .label_style(text_style)
                    .draw()?;

                middle
                    .draw_series(LineSeries::new(
                        range().zip(difference.iter().map(|&(re, _im)| re)),
                        &BLACK,
                    ))?
                    .label("Real Component")
                    .legend(legend(&BLACK));
                middle
                    .draw_series(LineSeries::new(
                        range().zip(difference.iter().map(|&(_re, im)| im)),
                        &RED,
                    ))?
                    .label("Imaginary Component")
                    .legend(legend(&RED));
                middle
                    .configure_series_labels()
                    .border_style(BLACK)
                    .background_style(WHITE.mix(0.5))
                    .position(SeriesLabelPosition::UpperRight)
                    .label_font(text_style)
                    .draw()?;

                // nyquist sampling theorem: we dont really care about showing anything after LENGTH / 2
                let ffted_polar = &ffted_polar[0..LENGTH as usize / 2];

                // make sure the y axis labels fit
                let n_y_axis_digits = f64::log10(ffted_polar_max.0.into()).ceil() as u32;

                // crop off anything to the right thats just basically 0 so we can see the actual fourier transform
                let threshold = ffted_polar_max.0 / cast!($T, 100);
                let x_size = ffted_polar.iter().enumerate().fold(0, |old, (index, &(mag, _phase))| {
                    if mag > threshold { index } else { old }
                }) + 100;
                let x_size = x_size.min(LENGTH as usize / 2);

                let mut bottom_left = ChartBuilder::on(&drawing_area_bottom_left)
                    .caption("FFT of Data (Magnitude) ( |FFT(x)| )", (font, 30))
                    .margin(20)
                    .set_label_area_size(LabelAreaPosition::Left, 10 * n_y_axis_digits)
                    .set_label_area_size(LabelAreaPosition::Bottom, 40)
                    .build_cartesian_2d::<std::ops::Range<u32>,$range>(
                        0..x_size as u32,
                        (ffted_polar_min.0..ffted_polar_max.0).into(),
                    )?;
                bottom_left
                    .configure_mesh()
                    .label_style(text_style)
                    .draw()?;

                let mut bottom_right = ChartBuilder::on(&drawing_area_bottom_right)
                    .caption("FFT of Data (Phase) ( arg(FFT(x)) )", (font, 30))
                    .margin(20)
                    .set_label_area_size(LabelAreaPosition::Right, 40)
                    .set_label_area_size(LabelAreaPosition::Bottom, 40)
                    .build_cartesian_2d::<std::ops::Range<u32>,$range>(
                        0..(LENGTH / 2),
                        (ffted_polar_min.1..ffted_polar_max.1).into(),
                    )?;
                bottom_right
                    .configure_mesh()
                    .label_style(text_style)
                    .draw()?;

                bottom_left.draw_series(LineSeries::new(
                    range().zip(ffted_polar[0..x_size].iter().map(|&(mag, _angle)| mag)),
                    BLACK,
                ))?;
                bottom_right.draw_series(LineSeries::new(
                    range().zip(ffted_polar.iter().map(|&(_mag, angle)| angle)),
                    BLUE,
                ))?;
            }
            last_signal = iffted.into_iter().map(|(re, _im)| re).collect();
        }
    };
    Ok(())
}
    )+};
    }

macro_rules! fmt_cast {
    (f128, $val:expr) => {
        $val as f64
    };
    (p64, $val:expr) => {
        <p64 as Into<f64>>::into($val)
    };
    (p32, $val:expr) => {
        <p32 as Into<f64>>::into($val)
    };
    (p16, $val:expr) => {
        <p16 as Into<f64>>::into($val)
    };
    (p8, $val:expr) => {
        <p8 as Into<f64>>::into($val)
    };
    ($_:ident, $val:expr) => {
        $val
    };
}
macro_rules! cast {
    (p64, $val:expr) => {
        p64::from($val)
    };
    (p32, $val:expr) => {
        p32::from($val)
    };
    (p16, $val:expr) => {
        p16::from($val)
    };
    (p8, $val:expr) => {
        p8::from($val)
    };
    ($T:ty, $val:expr) => {
        $val as $T
    };
}
macro_rules! fixup_floats {
    (f16, $val:tt) => {
        fixup_floats!(floating, f16, $val);
    };
    (f32, $val:tt) => {
        fixup_floats!(floating, f32, $val);
    };
    (f64, $val:tt) => {
        fixup_floats!(floating, f64, $val);
    };
    (f128, $val:tt) => {
        fixup_floats!(floating, f128, $val);
    };
    (floating, $T:ident, $val:tt) => {
        let $val = $val.map(|val| {
            if val.is_nan() {
                0.0
            } else {
                val.max($T::MIN).min($T::MAX)
            }
        });
    };
    ($_type:ident, $_val:expr) => {};
}

fn convert_data_using_from<T: From<f64>>(data: &Box<[f64]>) -> Box<[T]> {
    data.iter().copied().map(T::from).collect::<Box<[T]>>()
}

fn atan2_via_f64<T: From<f64> + Into<f64>>(y: T, x: T) -> T {
    (f64::atan2(y.into(), x.into())).into()
}

fn convert_data_using_cast<T>(data: &Box<[f64]>) -> Box<[T]>
where
    f64: FloatToFloat<T>,
{
    data.iter().copied().map(f64::cast).collect::<Box<[T]>>()
}

make_comparisons_for!(
 // f128_simple,f128,"simple", SimpleFloatContext, convert_data_using_cast, f128::atan2,f128::total_cmp,RangedCoordf128;
    f64_simple, f64, "simple", SimpleFloatContext, identity,                f64::atan2, f64::total_cmp, std::ops::Range<f64>;
    f32_simple, f32, "simple", SimpleFloatContext, convert_data_using_cast, f32::atan2, f32::total_cmp, std::ops::Range<f32>;
    f16_simple, f16, "simple", SimpleFloatContext, convert_data_using_cast, f16::atan2, f16::total_cmp, RangedCoordf16;

 // f128_ts,f128,"tayor series deg. 30", TaylorSeriesFloatContext<30>, convert_data_using_cast, f128::atan2,f128::total_cmp,RangedCoordf128;
    f64_ts, f64, "tayor series deg. 30", TaylorSeriesFloatContext<30>, identity,                f64::atan2, f64::total_cmp, std::ops::Range<f64>;
    f32_ts, f32, "tayor series deg. 20", TaylorSeriesFloatContext<20>, convert_data_using_cast, f32::atan2, f32::total_cmp, std::ops::Range<f32>;
    f16_ts, f16, "tayor series deg. 5",  TaylorSeriesFloatContext<5>,  convert_data_using_cast, f16::atan2, f16::total_cmp, RangedCoordf16;

    p64_ts, p64, "taylor series deg. 30", PositNoQuireContextWithTaylorSeries<30>, convert_data_using_from, atan2_via_f64, p64::cmp, RangedCoordp64;
    p32_ts, p32, "taylor series deg. 20", PositNoQuireContextWithTaylorSeries<20>, convert_data_using_from, atan2_via_f64, p32::cmp, RangedCoordp32;
    p16_ts, p16, "taylor series deg. 20", PositNoQuireContextWithTaylorSeries<20>, convert_data_using_from, atan2_via_f64, p16::cmp, RangedCoordp16;
    p8_ts,  p8,  "taylor series deg. 20", PositNoQuireContextWithTaylorSeries<20>, convert_data_using_from, atan2_via_f64, p8::cmp,  RangedCoordp8;
);

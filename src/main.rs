use image::png::PNGEncoder;
use image::ColorType;
use num::Complex;
use std::env;
use std::fs::File;
use std::io::Error;
use std::panic::catch_unwind;
use std::process::ExitCode;
use std::str::FromStr;

fn main() -> ExitCode {
    let args: Vec<String> = env::args().collect();
    let operation = catch_unwind(|| generate_mandelbrot_image(&args));

    if operation.is_err() {
        alert_error(&args[0]);
        return ExitCode::FAILURE;
    }

    ExitCode::SUCCESS
}

fn generate_mandelbrot_image(args: &Vec<String>) -> Result<(), Error> {
    let is_processor_strategy = match args.last().unwrap().as_str() {
        "-st" => true,
        _ => false,
    };

    if let [_, filename, bounds_input, pair_1, pair_2] = &args[..=4] {
        let bounds = parse_pair(&bounds_input, 'x').expect("Error parsing image dimensions");
        let upper_left = parse_complex(&pair_1).expect("Error parsing upper left corner point");
        let lower_right = parse_complex(&pair_2).expect("Error parsing lower right corner point");
        let mut pixels = vec![0; bounds.0 * bounds.1];
        let render_strategy = match is_processor_strategy {
            true => render_single_threaded,
            _ => render_multi_threaded,
        };
        render_strategy(&mut pixels, bounds, upper_left, lower_right);
        write_image(&filename, &pixels, bounds).expect("Error writing PNG file");
    }

    Ok(())
}

fn alert_error(target_path: &str) {
    eprint!("\n");
    eprint!("Usage: <target_path> <file_name> <resolution> <upper_left> <lower_right> \n",);
    eprint!(
        "Example: {} mandel.png 1000x750 -1.20,0.35 -1,0.20",
        target_path,
    );
}

fn render_single_threaded(
    pixels: &mut [u8],
    bounds: (usize, usize),
    upper_left: Complex<f64>,
    lower_right: Complex<f64>,
) {
    println!("Performing single-threaded computations");
    render(pixels, bounds, upper_left, lower_right);
}

fn render_multi_threaded(
    pixels: &mut [u8],
    bounds: (usize, usize),
    upper_left: Complex<f64>,
    lower_right: Complex<f64>,
) {
    let threads = num_cpus::get();
    let rows_per_band = bounds.1 / threads + 1;
    let bands: Vec<&mut [u8]> = pixels.chunks_mut(rows_per_band * bounds.0).collect();
    println!(
        "Performing multi-threaded computations across {} thread",
        threads
    );
    crossbeam::scope(|spawner| {
        for (i, band) in bands.into_iter().enumerate() {
            let top = rows_per_band * i;
            let height = band.len() / bounds.0;
            let band_bounds = (bounds.0, height);
            let band_upper_left = pixel_to_point(bounds, (0, top), upper_left, lower_right);
            let band_lower_right =
                pixel_to_point(bounds, (bounds.0, top + height), upper_left, lower_right);

            spawner.spawn(move |_| render(band, band_bounds, band_upper_left, band_lower_right));
        }
    })
    .unwrap();
}

fn escape_time(c: Complex<f64>, limit: usize) -> Option<usize> {
    let mut z = Complex { re: 0.0, im: 0.0 };
    for i in 0..limit {
        if z.norm_sqr() > 4.0 {
            return Some(i);
        }
        z = z * z + c;
    }

    None
}

fn parse_pair<T: FromStr>(s: &str, separator: char) -> Option<(T, T)> {
    match s.find(separator) {
        None => None,
        Some(index) => match (T::from_str(&s[..index]), T::from_str(&s[index + 1..])) {
            (Ok(l), Ok(r)) => Some((l, r)),
            _ => None,
        },
    }
}

fn parse_complex(s: &str) -> Option<Complex<f64>> {
    match parse_pair(s, ',') {
        Some((re, im)) => Some(Complex { re, im }),
        None => None,
    }
}

fn pixel_to_point(
    bounds: (usize, usize),
    pixel: (usize, usize),
    upper_left: Complex<f64>,
    lower_right: Complex<f64>,
) -> Complex<f64> {
    let (width, heigth) = (
        lower_right.re - upper_left.re,
        upper_left.im - lower_right.im,
    );
    Complex {
        re: upper_left.re + pixel.0 as f64 * width / bounds.0 as f64,
        im: upper_left.im - pixel.1 as f64 * heigth / bounds.1 as f64,
    }
}

fn render(
    pixels: &mut [u8],
    bounds: (usize, usize),
    upper_left: Complex<f64>,
    lower_right: Complex<f64>,
) {
    assert!(pixels.len() == bounds.0 * bounds.1);

    for row in 0..bounds.1 {
        for column in 0..bounds.0 {
            let point = pixel_to_point(bounds, (column, row), upper_left, lower_right);
            pixels[row * bounds.0 + column] = match escape_time(point, 255) {
                None => 0,
                Some(count) => 255 - count as u8,
            };
        }
    }
}

fn write_image(filename: &str, pixels: &[u8], bounds: (usize, usize)) -> Result<(), Error> {
    let output = File::create(filename)?;
    let encoder = PNGEncoder::new(output);
    encoder.encode(pixels, bounds.0 as u32, bounds.1 as u32, ColorType::Gray(8))?;

    Ok(())
}

#[test]
fn test_parse_pair() {
    assert_eq!(parse_pair::<i32>("", ','), None);
    assert_eq!(parse_pair::<i32>("10", ','), None);
    assert_eq!(parse_pair::<i32>(",10", ','), None);
    assert_eq!(parse_pair::<i32>("10,20", ','), Some((10, 20)));
    assert_eq!(parse_pair::<i32>("10,20xy", ','), None);
    assert_eq!(parse_pair::<f64>("0.5x", 'x'), None);
    assert_eq!(parse_pair::<f64>("0.5x1.5", 'x'), Some((0.5, 1.5)));
}

#[test]
fn test_parse_complex() {
    assert_eq!(
        parse_complex("1.25,-0.0625"),
        Some(Complex {
            re: 1.25,
            im: -0.0625
        })
    );
    assert_eq!(parse_complex(",-0.0625"), None);
}

#[test]
fn test_pixel_to_point() {
    assert_eq!(
        pixel_to_point(
            (100, 200),
            (25, 175),
            Complex { re: -1.0, im: 1.0 },
            Complex { re: 1.0, im: -1.0 }
        ),
        Complex {
            re: -0.5,
            im: -0.75
        }
    );
}

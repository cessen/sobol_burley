use std::fs::File;
use std::io::Write;

use sobol_burley::parts::{sobol_rev, u32_to_f32_norm};

const X_RES: usize = 512;
const Y_RES: usize = 512;
const POINTS: u32 = 1 << 10;

fn main() {
    // Plain sobol.
    plot(|i| sobol(i, 0), |i| sobol(i, 1), "sobol.pbm");
    for d in 0..20 {
        plot(
            |i| sobol(i, d),
            |i| sobol(i, d + 1),
            &format!("sobol_{}.pbm", d),
        );
    }

    // Scrambled shuffled sobol.
    for d in 0..20 {
        plot(
            |i| sobol_burley::sample(i, d, 0),
            |i| sobol_burley::sample(i, d + 1, 0),
            &format!("sobol_burley_{}.pbm", d),
        );
    }
}

fn sobol(i: u32, dimension: u32) -> f32 {
    let sobol_int = sobol_rev(i.reverse_bits(), dimension).reverse_bits();
    u32_to_f32_norm(sobol_int)
}

fn plot<F1, F2>(x_fn: F1, y_fn: F2, filename: &str)
where
    F1: Fn(u32) -> f32,
    F2: Fn(u32) -> f32,
{
    let mut image = vec![1u8; X_RES * Y_RES];
    for i in 0..POINTS {
        let x = (x_fn(i) * (X_RES - 1) as f32) as usize;
        let y = (y_fn(i) * (Y_RES - 1) as f32) as usize;
        image[y * X_RES + x] = 0;
    }

    let mut f = File::create(filename).unwrap();
    f.write(format!("P1\n{} {}\n\n", X_RES, Y_RES).as_bytes())
        .unwrap();
    for chunk in image.chunks(80) {
        for pixel in chunk.iter() {
            f.write(if *pixel == 0 { b"0" } else { b"1" }).unwrap();
        }
        f.write(b"\n").unwrap();
    }
}

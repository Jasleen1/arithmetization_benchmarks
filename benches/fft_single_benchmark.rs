// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.
// Modified version of code from github.com/facebook/winterfell

use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use std::time::Duration;
use winter_math::{
    fft,
    fields::{f128, f62, f64, CubeExtension, QuadExtension},
    FieldElement, StarkField,
};
use winter_rand_utils::rand_vector;

// const SIZES: [usize; 4] = [262_144, 524_288, 1_048_576, 2_097_152];
const AIR_FFT_INTERPOL_SIZES: [(usize, usize); 1] = [(16, 140)];

const AIR_FFT_EVAL_SIZES: [(usize, usize); 1] = [(16 * 64, 140)];

const R1CS_FFT_INTERPOL_SIZES: [(usize, usize); 1] = [(2048, 4)];
const R1CS_FFT_EVAL_SIZES: [(usize, usize); 1] = [(2048 * 4 * 2, 10)];

fn fft_evaluate_poly<B, E>(c: &mut Criterion, field_name: &str)
where
    B: StarkField,
    E: FieldElement<BaseField = B>,
{
    let mut group = c.benchmark_group(format!("{}/fft_evaluate_poly", field_name));
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(10));

    let blowup_factor = 8;

    for &size in AIR_FFT_EVAL_SIZES.iter() {
        let p: Vec<E> = rand_vector(size.0 / blowup_factor);
        let twiddles: Vec<B> = fft::get_twiddles(size.0);
        group.bench_function(BenchmarkId::new("air", size.0), |bench| {
            bench.iter_with_large_drop(|| {
                let mut result = vec![E::ZERO; size.0];
                result[..p.len()].copy_from_slice(&p);
                for _ in 0..size.1 {
                    fft::evaluate_poly(&mut result, &twiddles);
                }
                result
            });
        });
    }

    for &size in R1CS_FFT_EVAL_SIZES.iter() {
        let mut p: Vec<E> = rand_vector(size.0 / blowup_factor);
        let twiddles: Vec<B> = fft::get_twiddles(size.0);
        group.bench_function(BenchmarkId::new("r1cs", size.0), |bench| {
            bench.iter_with_large_drop(|| {
                let mut result = vec![E::ZERO; size.0];
                result[..p.len()].copy_from_slice(&p);
                // fft::evaluate_poly_with_offset(&p, &twiddles, B::GENERATOR, blowup_factor)
                for _ in 0..size.1 {
                    fft::evaluate_poly(&mut result, &twiddles);
                }
            });
        });
    }

    group.finish();
}

fn fft_interpolate_poly<B, E>(c: &mut Criterion, field_name: &str)
where
    B: StarkField,
    E: FieldElement<BaseField = B>,
{
    let mut group = c.benchmark_group(format!("{}/fft_interpolate_poly", field_name));
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(10));

    for &size in AIR_FFT_INTERPOL_SIZES.iter() {
        let p: Vec<E> = rand_vector(size.0);
        let inv_twiddles: Vec<B> = fft::get_inv_twiddles(size.0);
        group.bench_function(BenchmarkId::new("air", size.0), |bench| {
            bench.iter_batched_ref(
                || p.clone(),
                |mut p| {
                    for _ in 0..size.1 {
                        fft::interpolate_poly(p, &inv_twiddles);
                    }
                },
                BatchSize::LargeInput,
            );
        });
    }

    for &size in R1CS_FFT_INTERPOL_SIZES.iter() {
        let p: Vec<E> = rand_vector(size.0);
        let inv_twiddles: Vec<B> = fft::get_inv_twiddles(size.0);
        group.bench_function(BenchmarkId::new("r1cs", size.0), |bench| {
            bench.iter_batched_ref(
                || p.clone(),
                |mut p| {
                    for _ in 0..size.1 {
                        fft::interpolate_poly(p, &inv_twiddles);
                    }
                },
                BatchSize::LargeInput,
            );
        });
    }

    group.finish();
}

// fn get_twiddles(c: &mut Criterion) {
//     let mut group = c.benchmark_group("fft_get_twiddles");
//     group.sample_size(10);
//     for &size in SIZES.iter() {
//         group.bench_with_input(BenchmarkId::from_parameter(size), &size, |bench, &size| {
//             bench.iter(|| fft::get_twiddles::<f128::BaseElement>(size));
//         });
//     }
//     group.finish();
// }

fn bench_fft(c: &mut Criterion) {
    fft_evaluate_poly::<f62::BaseElement, f62::BaseElement>(c, "f62");
    // fft_evaluate_poly::<f64::BaseElement, f64::BaseElement>(c, "f64");
    // fft_evaluate_poly::<f128::BaseElement, f128::BaseElement>(c, "f128");

    // fft_evaluate_poly::<f62::BaseElement, QuadExtension<f62::BaseElement>>(c, "f62_quad");
    // fft_evaluate_poly::<f64::BaseElement, QuadExtension<f64::BaseElement>>(c, "f64_quad");
    // fft_evaluate_poly::<f128::BaseElement, QuadExtension<f128::BaseElement>>(c, "f128_quad");

    // fft_evaluate_poly::<f64::BaseElement, CubeExtension<f64::BaseElement>>(c, "f64_cube");

    fft_interpolate_poly::<f62::BaseElement, f62::BaseElement>(c, "f62");
    // fft_interpolate_poly::<f64::BaseElement, f64::BaseElement>(c, "f64");
    // fft_interpolate_poly::<f128::BaseElement, f128::BaseElement>(c, "f128");
}

criterion_group!(fft_group, bench_fft); //get_twiddles);
criterion_main!(fft_group);

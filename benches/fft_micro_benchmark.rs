// Copyright (c) Facebook, Inc. and its affiliates.
//
// This source code is licensed under the MIT license found in the
// LICENSE file in the root directory of this source tree.

use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use std::time::Duration;
use winter_math::{
    fft,
    fields::{f128, f62, f64, CubeExtension, QuadExtension},
    FieldElement, StarkField,
};
use winter_rand_utils::rand_vector;

const SIZES: [usize; 3] = [262_144, 524_288, 1_048_576];

fn get_vec_of_size<E: FieldElement>(s: u64) -> Vec<E> {
    (0..s).map(|x| E::from(x)).collect::<Vec<E>>()
}

fn fft_evaluate_poly<B, E>(c: &mut Criterion, field_name: &str)
where
    B: StarkField,
    E: FieldElement<BaseField = B>,
{
    let mut group = c.benchmark_group(format!("{field_name}/fft_evaluate_poly"));
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(10));

    let blowup_factor = 8usize;
    let sizes_for_bench = (1..25).map(|x| 1 << x);
    for size in sizes_for_bench {
        let p: Vec<E> = get_vec_of_size(size);
        let twiddles: Vec<B> = fft::get_twiddles(size.try_into().unwrap());
        group.bench_function(BenchmarkId::new("simple", size), |bench| {
            bench.iter_with_large_drop(|| {
                let mut result = vec![E::ZERO; size.try_into().unwrap()];
                result[..p.len()].copy_from_slice(&p);
                fft::evaluate_poly(&mut result, &twiddles);
                result
            });
        });
    }
    let sizes_for_bench = (1..25).map(|x| 1 << x);
    for size in sizes_for_bench {
        let p: Vec<E> = get_vec_of_size(size);
        let twiddles: Vec<B> = fft::get_twiddles(size.try_into().unwrap());
        group.bench_function(BenchmarkId::new("with_offset", size), |bench| {
            bench.iter_with_large_drop(|| {
                fft::evaluate_poly_with_offset(&p, &twiddles, B::GENERATOR, blowup_factor)
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
    let mut group = c.benchmark_group(format!("{field_name}/fft_interpolate_poly"));
    group.sample_size(10);
    group.measurement_time(Duration::from_secs(25));
    let sizes_for_bench = (1..25).map(|x| 1 << x);
    for size in sizes_for_bench {
        let p: Vec<E> = get_vec_of_size(size);
        let inv_twiddles: Vec<B> = fft::get_inv_twiddles(size.try_into().unwrap());
        group.bench_function(BenchmarkId::new("simple", size), |bench| {
            bench.iter_batched_ref(
                || p.clone(),
                |p| fft::interpolate_poly(p, &inv_twiddles),
                BatchSize::LargeInput,
            );
        });
    }
    let sizes_for_bench = (1..25).map(|x| 1 << x);
    for size in sizes_for_bench {
        let p: Vec<E> = get_vec_of_size(size);
        let inv_twiddles: Vec<B> = fft::get_inv_twiddles(size.try_into().unwrap());
        group.bench_function(BenchmarkId::new("with_offset", size), |bench| {
            bench.iter_batched_ref(
                || p.clone(),
                |p| fft::interpolate_poly_with_offset(p, &inv_twiddles, B::GENERATOR),
                BatchSize::LargeInput,
            );
        });
    }

    group.finish();
}

fn get_twiddles(c: &mut Criterion) {
    let mut group = c.benchmark_group("fft_get_twiddles");
    group.sample_size(10);
    let sizes_for_bench = (1..25).map(|x| 1 << x);
    for size in sizes_for_bench {
        group.bench_with_input(BenchmarkId::from_parameter(size), &size, |bench, &size| {
            bench.iter(|| fft::get_twiddles::<f128::BaseElement>(size));
        });
    }
    group.finish();
}

fn bench_fft(c: &mut Criterion) {
    fft_evaluate_poly::<f62::BaseElement, f62::BaseElement>(c, "f62");
    fft_evaluate_poly::<f64::BaseElement, f64::BaseElement>(c, "f64");
    fft_evaluate_poly::<f128::BaseElement, f128::BaseElement>(c, "f128");

    // fft_evaluate_poly::<f62::BaseElement, QuadExtension<f62::BaseElement>>(c, "f62_quad");
    // fft_evaluate_poly::<f64::BaseElement, QuadExtension<f64::BaseElement>>(c, "f64_quad");
    // fft_evaluate_poly::<f128::BaseElement, QuadExtension<f128::BaseElement>>(c, "f128_quad");

    // fft_evaluate_poly::<f64::BaseElement, CubeExtension<f64::BaseElement>>(c, "f64_cube");

    fft_interpolate_poly::<f62::BaseElement, f62::BaseElement>(c, "f62");
    fft_interpolate_poly::<f64::BaseElement, f64::BaseElement>(c, "f64");
    fft_interpolate_poly::<f128::BaseElement, f128::BaseElement>(c, "f128");
}

criterion_group!(fft_group, bench_fft, get_twiddles);
criterion_main!(fft_group);

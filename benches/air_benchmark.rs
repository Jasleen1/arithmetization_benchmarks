use std::cmp::max;

use criterion::{criterion_group, criterion_main, Criterion};
use examples::{fast_fourier_transform, fibonacci, ExampleOptions, ExampleType};

fn run_benchmarks(c: &mut Criterion, program: ExampleType) {
    let mut options = ExampleOptions {
        example: program,
        hash_fn: "blake3_256".to_string(),
        num_queries: Some(16),
        blowup_factor: Some(4),
        grinding_factor: 16,
        field_extension: 1,
        folding_factor: 8,
    };
    let testname;
    let example = match options.example {
        ExampleType::Fib { sequence_length } => {
            testname = format!("Fib-{sequence_length}");
            fibonacci::mulfib2::get_example(&options, sequence_length).unwrap()
        }
        ExampleType::FFT { degree } => {
            testname = format!("FFT-{degree}");
            let b = max(degree, 64);
            options.blowup_factor = Some(b);
            fast_fourier_transform::get_example(&options, degree).unwrap()
        }
        _ => {
            println!("Example type for STARKs not supported");
            return;
        }
    };

    let mut prover_bench = c.benchmark_group("prover");
    prover_bench.sample_size(10);
    prover_bench.bench_function(&format!("Air prover for {testname}"), |b| {
        b.iter(|| example.prove())
    });
    prover_bench.finish();

    let proof = example.prove();

    let mut verifier_bench = c.benchmark_group("verifier");
    verifier_bench.bench_function(&format!("Air verifier for {testname}"), |b| {
        b.iter(|| example.verify(proof.clone()))
    });
    verifier_bench.finish();
}
fn criterion_benchmark(c: &mut Criterion) {
    // This is how you instantiate a benchmark for AIR
    // run_benchmarks(c, ExampleType::Fib{sequence_length: 1 << 15});
    // run_benchmarks(c, ExampleType::Fib{sequence_length: 1 << 20});
    // run_benchmarks(c, ExampleType::Fib{sequence_length: 1 << 25});
    run_benchmarks(c, ExampleType::FFT { degree: 1 << 7 });
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

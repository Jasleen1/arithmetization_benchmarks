use criterion::{Criterion, criterion_main, criterion_group};
use examples::{fibonacci, ExampleType, ExampleOptions, fast_fourier_transform};

fn run_benchmarks(c: &mut Criterion, program: ExampleType){
    let options = ExampleOptions {
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
            fibonacci::mulfib2::get_example(options, sequence_length)
        }
        ExampleType::FFT { degree } =>{
            testname = format!("FFT-{degree}");
            fast_fourier_transform::get_example(options, degree)
        },
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
    run_benchmarks(c, ExampleType::Fib{sequence_length: 32});
    run_benchmarks(c, ExampleType::Fib{sequence_length: 64});
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);
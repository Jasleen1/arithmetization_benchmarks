// Copyright (c) 2023  Don Beaver, Harjasleen Malvai, Tom Jurek.

// Benchmark to run various proof systems applied to various programs.

use std::cmp::max;
// use structopt::StructOpt;

use criterion::{black_box, criterion_group, criterion_main, BenchmarkId, Criterion};

// Winterfell AIR
use examples::{fast_fourier_transform, fibonacci, Example, ExampleOptions, ExampleType};

// WinterFractal R1CS
use fractal_examples::r1cs_orchestrator::ProofSystemOrchestrator;
use winter_crypto::hashers::Blake3_256;
use winter_math::fields::f64::BaseElement;
use winter_math::FieldElement;
use winter_math::StarkField;

//use criterion_benchmarking::{euler1_par, euler1_series, euler1_simple};

// Verbose-controlled output.
macro_rules! println_if {
    ($verbose:expr, $($x:tt)*) => { if $verbose { println!($($x)*) } }
}

// Command line interface.
// #[derive(StructOpt, Debug)]
// #[structopt(name = "prog-by-ps", about = "Run combos of programs and proof systems")]
// struct ProgramOptions {
//     /// Which programs to run.  Comma-separated list.
//     #[structopt(short = "p", long = "programs", default_value = "fft")]
//     program_list: String,

//     /// Which proof systems to run.  Comma-separated list.
//     #[structopt(short = "s", long = "systems", default_value = "r1cs")]
//     system_list: String,

//     /// Which instance sizes to run.  Comma-separated list.
//     #[structopt(short = "i", long = "instances", default_value = "5,6,7")]
//     instance_list: String,

//     /// Verbose logging and reporting.
//     #[structopt(short = "v", long = "verbose")]
//     verbose: bool,
// }

// Programs to run.
#[derive(Debug)]
enum ProgramTag {
    FFT,
    Fibonacci,
    PtrChase,
    Sample,
}

// Proof systems to choose from.
#[derive(Debug)]
enum SystemTag {
    AIR,
    R1CS,
    R1CSPolyBatched,
}

fn get_program_tag(provided_name: &str) -> ProgramTag {
    match provided_name {
        "fft" | "fftexample" => ProgramTag::FFT,
        "fib" | "fibonacciexample" => ProgramTag::Fibonacci,
        "ptrchase" | "ptrchaseexample" => ProgramTag::PtrChase,
        "" | "default" | "sample" => ProgramTag::Sample,
        other => panic!("Unsupported program: {}", other),
    }
}

fn get_r1cs_source_stem(program_tag: &ProgramTag) -> String {
    match program_tag {
        ProgramTag::FFT => "fftexample".to_string(),
        ProgramTag::Fibonacci => "fibonacciexample".to_string(),
        ProgramTag::PtrChase => "ptrchaseexample".to_string(),
        ProgramTag::Sample => "sample".to_string(),
        other => panic!("Unsupported program: {:?}", other)
    }
}

fn get_system_tag(provided_name: &str) -> SystemTag {
    match provided_name {
        "air" => SystemTag::AIR,
        "r1cs" | "r1" | "r" => SystemTag::R1CS,
        "r1cs-batched" => SystemTag::R1CSPolyBatched,
        other => panic!("Unsupported proof system: {}", other),
    }
}

fn get_program_path(program_tag: ProgramTag, instance_size: u64) -> String {
    match program_tag {
        ProgramTag::FFT => format!("src/jsnark_outputs/fftexample_{}", instance_size),
        ProgramTag::Fibonacci => format!("src/jsnark_outputs/fibonacciexample_{}", instance_size),
        ProgramTag::PtrChase => format!("src/jsnark_outputs/_ptrchaseexample_{}", instance_size),
        ProgramTag::Sample => format!("src/jsnark_outputs/_sample_{}", instance_size),
    }
}

fn extract_setop_options() -> (bool, String, String, String) {
    // let options = ProgramOptions::from_args();
    //(options.verbose, options.program_list, options.system_list, options.instance_list)

    let (verbose, program_list, system_list, instance_list) = (
        false,
        "fft".to_string(),
        "r1cs".to_string(),
        "5,7,8,9,10,11,12".to_string()
    );
    (verbose, program_list, system_list, instance_list)
}

fn setup(
    verbose: bool,
    program_list: String,
    system_list: String,
    instance_list: String,
) -> (Vec<ProgramTag>, Vec<SystemTag>, Vec<u64>) {
    println_if!(verbose, "Programs {}", program_list);
    println_if!(verbose, "Systems {}", system_list);
    println_if!(verbose, "Sizes {}", instance_list);

    let program_tags: Vec<ProgramTag> = program_list
        .split(",")
        .map(|x| get_program_tag(x))
        .collect();
    let system_tags = system_list.split(",").map(|x| get_system_tag(x)).collect();
    let instance_sizes: Vec<u64> = instance_list
        .split(",")
        .map(|x| x.parse::<u64>().unwrap())
        .collect();

    for program_tag in program_tags.iter() {
        let supported_sizes = match program_tag {
            ProgramTag::FFT => 5u64..15u64,
            ProgramTag::Fibonacci => 5u64..21u64,
            ProgramTag::PtrChase => 5u64..10u64,
            ProgramTag::Sample => 1u64..2u64,
            other => panic!("Unsupported program: {:?}", other),
        };
        for instance_size in instance_sizes.iter() {
            assert!(
                supported_sizes.contains(instance_size),
                "Unsupported program size: {:?}@{}",
                program_tag,
                instance_size
            );
        }
    }

    (program_tags, system_tags, instance_sizes)
}

fn get_air_example(program_tag: &ProgramTag, instance_size: usize) -> Box<dyn Example> {
    let program = match program_tag {
        ProgramTag::FFT => ExampleType::FFT {
            degree: 2 << instance_size,
        },
        ProgramTag::Fibonacci => ExampleType::Fib {
            sequence_length: 2 << instance_size,
        },
        other => panic!("Unsupported program type {:?}", program_tag),
    };

    let mut air_example_options = ExampleOptions {
        example: program,
        hash_fn: "blake3_256".to_string(),
        num_queries: Some(16),
        blowup_factor: Some(4),
        grinding_factor: 16,
        field_extension: 1,
        folding_factor: 8,
    };

    match air_example_options.example {
        ExampleType::Fib { sequence_length } => {
            fibonacci::mulfib2::get_example(&air_example_options, sequence_length).unwrap()
        }
        ExampleType::FFT { degree } => {
            let b = max(degree, 64);
            air_example_options.blowup_factor = Some(b);
            fast_fourier_transform::get_example(&air_example_options, degree).unwrap()
        },
        other => {
            panic!("Example type {other:?} not supported");
        }
    }
}

fn get_r1cs_arith(program_tag: &ProgramTag, instance_size: u64) -> String {
    format!("src/jsnark_outputs/{}_{}.arith", get_r1cs_source_stem(program_tag), instance_size)
}

fn get_r1cs_wires(program_tag: &ProgramTag, instance_size: u64) -> String {
    format!("src/jsnark_outputs/{}_{}.wires", get_r1cs_source_stem(program_tag), instance_size)
}

// The benchmark runner.
fn program_by_proof_systems(crit: &mut Criterion) {
    let (verbose, program_list, system_list, instance_list) = extract_setop_options();

    println_if!(verbose, "ProgramByProofSystems");
    let (program_tags, system_tags, instance_sizes) =
        setup(verbose, program_list, system_list, instance_list);

    println_if!(verbose, "ProgramByProofSystems: iterate");
    for program_tag in program_tags.iter() {
        for system_tag in system_tags.iter() {
            for instance_size in instance_sizes.iter() {
                let prover_group_title = format!("{:?}-{:?}", program_tag, system_tag);
                let prover_bench_id = BenchmarkId::new(prover_group_title, instance_size);

                let verifier_group_title = format!("{:?}-{:?}", program_tag, system_tag);
                let verifier_bench_id = BenchmarkId::new(verifier_group_title, instance_size);

                println_if!(
                    verbose,
                    "Benching  {:?} x {:?} @ {}",
                    program_tag,
                    system_tag,
                    instance_size
                );

                match system_tag {
                    SystemTag::AIR => {
                        // Build and run the prover benchmarks.
                        let example = get_air_example(program_tag, *instance_size as usize);
                        let mut prover_group = crit.benchmark_group("ProverTime");
                        prover_group.bench_with_input(prover_bench_id, instance_size, |b, _| {
                            b.iter(|| example.prove())
                        });
                        prover_group.finish();
                        // Build and run the verifier benchmarks on a single proof.
                        let proof = example.prove();
                        // CHECK THIS
                        let mut verifier_group = crit.benchmark_group("VerifierTime");
                        verifier_group.bench_with_input(verifier_bench_id, instance_size, |b, _| {
                            b.iter(|| example.verify(proof.clone()))
                        });
                        verifier_group.finish();
                    },

                    SystemTag::R1CS => {
                        // Build and run the prover benchmarks.
                        let batched = false;
                        let orchestrator = ProofSystemOrchestrator::<BaseElement, BaseElement, Blake3_256<BaseElement>, 1>::new(
                            get_r1cs_arith(program_tag, *instance_size),
                            get_r1cs_wires(program_tag, *instance_size),
                            batched,
                            verbose);
                        let (prover_key, verifier_key, fractal_options, wires, prover_options) = orchestrator.prepare();
                        let pub_inputs_bytes = vec![0u8, 1u8, 2u8];
                        let mut prover_group = crit.benchmark_group("ProverTime");
                        prover_group.bench_with_input(prover_bench_id, &instance_size, |b, _| {
                            b.iter(|| orchestrator.prove(&pub_inputs_bytes, prover_key.clone(), &wires, &prover_options))
                        });
                        prover_group.finish();
                        // Build and run the verifier benchmarks on a single proof.
                        let proof = orchestrator.prove(&pub_inputs_bytes, prover_key.clone(), &wires, &prover_options);
                        let mut verifier_group = crit.benchmark_group("VerifierTime");
                        verifier_group.bench_with_input(verifier_bench_id, &instance_size, |b, _| {
                            b.iter(|| orchestrator.verify(&proof, &pub_inputs_bytes, &verifier_key, &fractal_options));
                        });
                        verifier_group.finish();
                    },

                    SystemTag::R1CSPolyBatched => {
                        // Build and run the prover benchmarks.
                        let batched = true;
                        let orchestrator = ProofSystemOrchestrator::<BaseElement, BaseElement, Blake3_256<BaseElement>, 1>::new(
                            get_r1cs_arith(program_tag, *instance_size),
                            get_r1cs_wires(program_tag, *instance_size),
                            batched,
                            verbose);
                        let (prover_key, verifier_key, fractal_options, wires, prover_options) = orchestrator.prepare();
                        let pub_inputs_bytes = vec![0u8, 1u8, 2u8];
                        let mut prover_group = crit.benchmark_group("ProverTime");
                        prover_group.bench_with_input(prover_bench_id, &instance_size, |b, _| {
                            b.iter(|| orchestrator.prove(&pub_inputs_bytes, prover_key.clone(), &wires, &prover_options))
                        });
                        prover_group.finish();
                        // Build and run the verifier benchmarks on a single proof.
                        let proof = orchestrator.prove(&pub_inputs_bytes, prover_key.clone(), &wires, &prover_options);
                        let mut verifier_group = crit.benchmark_group("VerifierTime");
                        verifier_group.bench_with_input(verifier_bench_id, &instance_size, |b, _| {
                            b.iter(|| orchestrator.verify(&proof, &pub_inputs_bytes, &verifier_key, &fractal_options));
                        });
                        verifier_group.finish();
                    },

                    other => {
                        panic!("Unsupported proof system: {:?}", other);
                    }
                }
            }
        }
    }
}

criterion_group!(benches, program_by_proof_systems);
criterion_main!(benches);

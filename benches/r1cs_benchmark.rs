use std::{cmp::max, sync::Arc};

use criterion::{criterion_group, criterion_main, Criterion};
use fractal_indexer::{
    index::{build_index_domains, Index, IndexParams},
    indexed_matrix::index_matrix,
    snark_keys::{generate_prover_and_verifier_keys, ProverKey},
};
use fractal_proofs::{
    fft, fields::f64::BaseElement, get_power_series, FieldElement, FractalOptions,
    FractalProverOptions, StarkField,
};
use fractal_prover::{prover::FractalProver, LayeredProver, LayeredSubProver};
use fractal_verifier::verifier::verify_layered_fractal_proof_from_top;
use winter_crypto::{hashers::Blake3_256, ElementHasher};
use winter_fri::FriOptions;
use winter_models::{
    jsnark_arith_parser::JsnarkArithReaderParser, jsnark_wire_parser::JsnarkWireReaderParser,
};

fn run_benchmarks<
    B: StarkField,
    E: FieldElement<BaseField = B>,
    H: ElementHasher + ElementHasher<BaseField = B>,
>(
    c: &mut Criterion,
    program: &str,
) {
    let arith_file = format!("src/jsnark_outputs/{program}.arith");
    let wire_file = format!("src/jsnark_outputs/{program}.wires");

    let mut arith_parser = JsnarkArithReaderParser::<B>::new().unwrap();
    arith_parser.parse_arith_file(&arith_file, false);
    let mut r1cs = arith_parser.r1cs_instance;

    let mut wires_parser = JsnarkWireReaderParser::<B>::new().unwrap();
    wires_parser.parse_wire_file(&wire_file, false);
    let wires = wires_parser.wires;

    // 1. Index this R1CS
    let num_input_variables = r1cs.num_cols().next_power_of_two();
    let num_non_zero = r1cs.max_num_nonzero().next_power_of_two();
    let num_constraints =
        max(max(r1cs.A.num_rows(), r1cs.B.num_rows()), r1cs.C.num_rows()).next_power_of_two();
    let max_degree = FractalProver::<B, E, H>::get_max_degree_constraint(
        num_input_variables,
        num_non_zero,
        num_constraints,
    );
    // TODO: make the calculation of eta automated
    let eta = B::GENERATOR.exp(B::PositiveInteger::from(2 * B::TWO_ADICITY));
    let eta_k = B::GENERATOR.exp(B::PositiveInteger::from(1337 * B::TWO_ADICITY));

    let index_params = IndexParams::<B> {
        num_input_variables,
        num_constraints,
        num_non_zero,
        max_degree,
        eta,
        eta_k,
    };

    let degree_fs = r1cs.num_cols();
    let index_domains = build_index_domains::<B>(index_params.clone());
    let indexed_a = index_matrix::<B>(&mut r1cs.A, &index_domains);
    let indexed_b = index_matrix::<B>(&mut r1cs.B, &index_domains);
    let indexed_c = index_matrix::<B>(&mut r1cs.C, &index_domains);
    // This is the index i.e. the pre-processed data for this r1cs
    let index = Index::new(index_params.clone(), indexed_a, indexed_b, indexed_c);

    // TODO: the IndexDomains should already guarantee powers of two, so why add extraneous bit or use next_power_of_two?

    let size_subgroup_h = index_domains.h_field.len().next_power_of_two();
    let size_subgroup_k = index_domains.k_field.len().next_power_of_two();

    let evaluation_domain = get_power_series(index_domains.l_field_base, index_domains.l_field_len);

    let summing_domain = index_domains.k_field;

    let h_domain = index_domains.h_field;
    let lde_blowup = 4;
    let num_queries = 16;
    let fri_options = FriOptions::new(lde_blowup, 4, 32);
    //println!("h_domain: {:?}, summing_domain: {:?}, evaluation_domain: {:?}", &h_domain, &summing_domain, &evaluation_domain);
    let options: FractalOptions<B> = FractalOptions::<B> {
        degree_fs,
        size_subgroup_h,
        size_subgroup_k,
        summing_domain: summing_domain.clone(),
        evaluation_domain: evaluation_domain.clone(),
        h_domain: h_domain.clone(),
        eta,
        eta_k,
        fri_options: fri_options.clone(),
        num_queries,
    };

    let h_domain_twiddles = fft::get_twiddles(size_subgroup_h);
    let h_domain_inv_twiddles = fft::get_inv_twiddles(size_subgroup_h);
    let k_domain_twiddles = fft::get_twiddles(size_subgroup_k);
    let k_domain_inv_twiddles = fft::get_inv_twiddles(size_subgroup_k);
    let l_domain_twiddles = fft::get_twiddles(evaluation_domain.len());
    let l_domain_inv_twiddles = fft::get_inv_twiddles(evaluation_domain.len());
    let prover_options: FractalProverOptions<B> = FractalProverOptions::<B> {
        degree_fs,
        size_subgroup_h,
        size_subgroup_k,
        summing_domain,
        evaluation_domain,
        h_domain,
        h_domain_twiddles,
        h_domain_inv_twiddles,
        k_domain_twiddles,
        k_domain_inv_twiddles,
        l_domain_twiddles,
        l_domain_inv_twiddles,
        eta,
        eta_k,
        fri_options: fri_options.clone(),
        num_queries,
    };

    let (prover_key_raw, verifier_key) =
        generate_prover_and_verifier_keys::<B, E, H>(index, &options).unwrap();
    let pub_inputs_bytes = vec![0u8, 1u8, 2u8];
    let prover_key: Arc<ProverKey<B, E, H>> = prover_key_raw.into();

    // create a benchmark group for the prover which runs fewer times
    let mut prover_bench = c.benchmark_group("prover");
    prover_bench.sample_size(10);

    prover_bench.bench_function(&format!("R1CS prover for {program}"), |b| {
        b.iter(|| {
            let mut b_prover = FractalProver::<B, E, H>::new(
                prover_key.clone(),
                vec![],
                wires.clone(),
                pub_inputs_bytes.clone(),
            );
            b_prover
                .generate_proof(&None, pub_inputs_bytes.clone(), &prover_options)
                .unwrap()
        })
    });
    prover_bench.finish();

    let mut prover =
        FractalProver::<B, E, H>::new(prover_key.clone(), vec![], wires, pub_inputs_bytes.clone());
    let proof = prover
        .generate_proof(&None, pub_inputs_bytes.clone(), &prover_options)
        .unwrap();

    // (optional) create a verifier group
    let mut verifier_bench = c.benchmark_group("verifier");
    verifier_bench.bench_function(&format!("R1CS verifier for {program}"), |b| {
        b.iter(|| {
            verify_layered_fractal_proof_from_top(
                &verifier_key,
                &proof,
                &pub_inputs_bytes,
                &options,
            )
            .unwrap()
        })
    });
    verifier_bench.finish();
}

fn criterion_benchmark(c: &mut Criterion) {
    // This is how you instantiate a benchmark for R1CS, make sure the corresponding .wires and .arith files exist in the ../src/jsnark_outputs directory.
    // run_benchmarks::<BaseElement, BaseElement, Blake3_256<BaseElement>>(c, "fibonacciexample_15");
    // run_benchmarks::<BaseElement, BaseElement, Blake3_256<BaseElement>>(c, "fibonacciexample_20");
    run_benchmarks::<BaseElement, BaseElement, Blake3_256<BaseElement>>(c, "fftexample_7");
    // run_benchmarks::<BaseElement, BaseElement, Blake3_256<BaseElement>>(c, "fftexample_10");
}

criterion_group!(benches, criterion_benchmark);
criterion_main!(benches);

use log::debug;
use std::time::Instant;
use structopt::StructOpt;
use winter_math::log2;

use examples::{fast_fourier_transform, fibonacci, ExampleOptions, ExampleType};
#[cfg(feature = "std")]
use winter_examples::{lamport, merkle};

fn get_num_main_trace_rows(num_fft_inputs: usize) -> usize {
    let log_num_fft_terms: usize = log2(num_fft_inputs).try_into().unwrap();
    log_num_fft_terms + 2
}

fn get_num_cols(num_fft_inputs: usize) -> usize {
    let log_num_fft_terms: usize = log2(num_fft_inputs).try_into().unwrap();
    // the first num_fft_inputs are for keeping the actual values at eachs step
    // the next value is for keeping the local omegas
    // Then, we store log_num_fft_terms + 1 bits which help select the function to apply
    // Finally, the additional position is to keep the power of 2 represented by the aforementioned bits
    num_fft_inputs + 1 + 1 + (log_num_fft_terms + 1) + 2
}

// EXAMPLE RUNNER
// ================================================================================================

fn main() {
    // read command-line args
    let options = ExampleOptions::from_args();

    println!("============================================================");

    // instantiate and prepare the example
    let example = match options.example {
        ExampleType::Fib { sequence_length } => {
            fibonacci::mulfib2::get_example(&options, sequence_length).unwrap()
        }
        ExampleType::FFT { degree } => {
            let num_cols = get_num_cols(degree);
            let num_rows = get_num_main_trace_rows(degree);
            println!(
                "FFT size {} has {} columns and {} rows",
                degree, num_cols, num_rows
            );
            fast_fourier_transform::get_example(&options, degree).unwrap()
        }
        _ => {
            println!("Example type for STARKs not supported");
            return;
        }
    };

    // generate stark proof
    let now = Instant::now();
    let proof = example.prove();
    println!(
        "---------------------\nProof generated in {} ms",
        now.elapsed().as_millis()
    );

    // verify the proof
    println!("---------------------");
    let now = Instant::now();
    match example.verify(proof) {
        Ok(_) => println!(
            "Proof verified in {:.1} ms",
            now.elapsed().as_micros() as f64 / 1000f64
        ),
        Err(msg) => debug!("Failed to verify proof: {}", msg),
    }
    println!("============================================================");
}

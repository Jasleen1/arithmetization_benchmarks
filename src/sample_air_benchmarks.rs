use log::debug;
use std::time::Instant;
use structopt::StructOpt;

use examples::{fast_fourier_transform, fibonacci, ExampleOptions, ExampleType};
#[cfg(feature = "std")]
use winter_examples::{lamport, merkle};

// EXAMPLE RUNNER
// ================================================================================================

fn main() {
    // read command-line args
    let options = ExampleOptions::from_args();

    println!("============================================================");

    // instantiate and prepare the example
    let example = match options.example {
        ExampleType::Fib { sequence_length } => {
            fibonacci::fib2::get_example(options, sequence_length)
        }
        ExampleType::FFT { degree } => fast_fourier_transform::get_example(options, degree),
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

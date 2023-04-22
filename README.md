# Introduction 
When using SNARK tools for practical applications, there exist heuristics for which arithmetization is best suited to a certain kind of computation. 

For example, it is known that it’s very difficult to implement hash tables in R1CS, due to the difficulty of random accesses. Usually random accesses end up requiring looping over an entire array, since the overhead of implementing a RAM-like solution is too high for reasonably sized circuits. For such an application, AIR with permutations would be a good fit. However, are there quantifiable properties of this computation that can tell you what to use?

Our goal with this project was to answer the following: Can we, in a principled way, categorize which arithmetization is best suited to which kind of computation?

In order to tease out the impact of the arithmetization on various metrics of a proof system, we need to build proof systems which are close to identical in every respect but the arithmetization. What this means is that the cryptographic assumptions and underlying polynomial commitments should be identical for the construction based on each arithmetization. To this end we would like to have (at minimum) R1CS and AIR-based implementations on top of the Rust Winterfell library’s existing poly-commit infrastructure. Then we wanted to test, experimentally, what applications yield better performance on what system. Ideally, we would also include a theoretical classification and proofs of this classification, for example, "how can this application be quantified in terms of its memory access patterns?" and we would have examples that fit the bill for each classification. 

We picked  FFT programs to illustrate the idea that the verification time for an AIR proof grows linearly in the number of distinct constraints known as transition constraints. R1CS verification is independent of the structure of the program and found that as expected, optimized R1CS performs better than AIR.We picked fibonacci to test if repeated structure in a program provides significant benefits when proving it in an AIR-based prover. So far, we haven’t even been able to run very large instances in R1CS because it runs out of memory. 

Note that the R1CS code still has some bugs at certain programs and for certain program sizes it fails to run due to running out of memory. These improvements for our implementation are a work in progress.

# TOOLING 
* We have a Rust implementation of Fractal, which is a highly optimized, FRI-based R1CS proof system, based on the backend from the Winterfell Rust crates. See [this branch](https://github.com/Jasleen1/winter_fractal/tree/incomplete_reorg).
* We have implemented an example FFT program in [Winterfell](https://github.com/Jasleen1/winterfell/tree/fft) and Winterfell also already has example Fibonacci implementations. This was the AIR examples component of our code.
* Next we optimized examples of Fibonacci and FFT implementations in [jsnark](https://github.com/Jasleen1/jsnark/tree/gen-arith).
* We had code to port the .arith and .in files representing R1CS inputs and instances generated by jsnark out (see the Dockerfile in [our jsnark fork](https://github.com/Jasleen1/jsnark/tree/gen-arith)) and used these as our examples for R1CS.
* Finally, we were able to modify the fields in jsnark to fit the fields needed by the underlying libraries of our [R1CS code](https://github.com/Jasleen1/winter_fractal/tree/incomplete_reorg), parse it in as a result and run benchmarks for different sized FFT and Fibonacci instances. 


# Including further examples
Here we have provided a blueprint for comparing two kinds of proof systems on two kinds of programs. To add more example programs, we suggest you 
* Generate R1CS examples using [jsnark](https://github.com/akosba/jsnark) (see our [implementation](https://github.com/Jasleen1/jsnark/tree/gen-arith) for examples of how to effectively extract the requisite files). 
* Generate AIR examples using [Winterfell](https://github.com/facebook/winterfell/)'s examples crate perhaps in your own fork, import that fork in the `Cargo.toml` and use the examples as we use them here. 

Once you have generated the examples themselves, to get them working, you will need to modify the `sample_air_benchmarks.rs` and `sample_r1cs_benchmarks.rs` files. 

Note that we suggest you try to transcribe your algorithms as closely as possible in each framework, in order to get an accurate idea of how the structure of your program impacts the performance of the proof system. 
# USAGE

### R1CS
To benchmark an R1CS program, you have the following options: 
* Program type `-p` which can be set to `fft` or `fib`
* Program size `-s` which can be set to a value between 5 and 10. Note that currently some of these values are causing crashes. We recommend trying 5 and 9 to get a sense of how things go. This runs the programs in the following way: if your size is `x`, and your program is `fib` you will see results for the fibonacci proof for the 2^x th fibonacci number. 

Run the following command to see the benchmarks:

```cargo run --release --package arithmetization_benchmarks --bin fractal-orchestrator -- -p=fft -s=9```


### AIR
To benchmark AIR programs the commands are similar the program types are `fft` or `fib` and you can run them without using them as commands exactly. Like so

`cargo run --release --package arithmetization_benchmarks --bin stark-orchestrator fft -n=32`

Note that in this case, your program size needs to be specified exactly, i.e. if you want to see an FFT of size 32 in the STARK context, you enter 32, as opposed to 5, which you would have entered in the R1CS case.

# Acknowledgements 
Many thanks to Don Beaver for writing the parser from jsnark's R1CS representation to our implementation's R1CS matrices, as well as greately beautifying our code. 

We are also very grateful to Bobbin Threadbare for help with implementing the examples for Winterfell as well as general support while building on the Winterfell repository. 

Thank you to Andrew Miller for extemely helpful guidance and helping us evolve the ideas for this work. 

# License

This project is [MIT licensed](LICENSE). 
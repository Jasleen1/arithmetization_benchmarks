## USAGE

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
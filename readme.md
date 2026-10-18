this is the code for my honours project

### what it does

it compares the posit number format with the IEEE 754 floating point format.

it performs a FFT then inverse FFT repeatedly for a few different signals, across all the number formats tested, and compares the inputs to the outputs, and generates some metrics and graphs.

### building and running

you need rust nightly
```sh
rustup default nightly
```

then build with
```sh
cargo build --profile=release
```

then run it with 
```sh
./target/posit-project roundtrip
# or
cargo run --profile=release -- roundtrip
```

see available options with `--help`

### data

csv files containing metrics are available in the `csv` directory.

graphs showing the actual FFTed and IFFTed data are in the `fft_graphs` directory

graphs showing some of the metrics are in the `metrics_graphs` directory
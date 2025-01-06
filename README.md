# Setup

[Install Rust](https://www.rust-lang.org/tools/install)  
Add target: `rustup target add x86_64-unknown-linux-musl`  
Add rust src (required for nightly musl): `rustup component add rust-src --toolchain nightly-x86_64-unknown-linux-gnu`

Download dataset and extract into `data` directory.

# Development

Commands to benchmark on g0. (scp only works like this if you have ssh setup with config)

```sh
RUSTFLAGS='-C target-cpu=icelake-client' cargo build -r
scp ./target/x86_64-unknown-linux-musl/release/TUW-EP g0.complang.tuwien.ac.at:~

ssh g0.complang.tuwien.ac.at

cd /localtmp/efficient24
LC_NUMERIC=en_US perf stat -e cycles ~/TUW-EP f1.csv f2.csv f3.csv f4.csv >/dev/null
```

Commands to run locally.

```sh
RUSTFLAGS='-C target-cpu=native' cargo build -r
./target/x86_64-unknown-linux-musl/release/TUW-EP data/f1.csv data/f2.csv data/f3.csv data/f4.csv | sort | diff - data/output.csv
```

# Benchmarks

Benchmarks can be found [here](stats.md)

To run 10 times and get the average cycles you can:

```
NUMERIC=en_US seq 10 | xargs -Iz perf stat -e cycles ~/TUW-EP f1.csv f2.csv f3.csv f4.csv >/dev/null 2> >(grep "cycles" | sed -r 's/\.//g' | awk '{s+=$1;c++} END {print s/c}' >&2)
```

# Tools

## Raw assembly

To view the raw assembly output you can run `RUSTFLAGS='-C target-cpu=icelake-client' cargo rustc --release -- --emit asm`.  
The directory `./target/x86_64-unknown-linux-musl/release/deps` then contains an `.asm` file.

## Flamegraph

To generate a flamegraph you have to first set `debug = true` in `./.cargo/config.toml`.  
Then build the binary to run locally (see above).  
Now run `flamegraph -- ./target/x86_64-unknown-linux-musl/release/TUW-EP data/f1.csv data/f2.csv data/f3.csv data/f4.csv >/dev/null`.  
There should now be a `flamegraph.svg` which you can open, navigate and search.

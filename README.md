# Setup

[Install Rust](https://www.rust-lang.org/tools/install)

Download dataset and extract into `data` directory.

# Development

Commands to benchmark on g0 (needs rust and git repo).

```sh
cargo build -r
cp ./target/release/TUW-EP ~/TUW-EP

cd /localtmp/efficient24
perf stat -e cycles ~/TUW-EP f1.csv f2.csv f3.csv f4.csv >/dev/null
```

Commands to run locally.

```sh
cargo build -r
./target/release/TUW-EP data/f1.csv data/f2.csv data/f3.csv data/f4.csv | sort | diff - data/output.csv
```

# Benchmarks

Benchmarks can be found [here](stats.md)

To run 10 times and get the average cycles you can:

```
seq 10 | xargs -Iz perf stat -e cycles ~/TUW-EP f1.csv f2.csv f3.csv f4.csv >/dev/null 2> >(grep "cycles" | sed -r 's/\.//g' | awk '{s+=$1;c++} END {print s/c}' >&2)
```

# Tools

## Raw assembly

To view the raw assembly output you can run `RUSTFLAGS='-C target-cpu=icelake-client' cargo rustc --release -- --emit asm`.  
The directory `./target/release/deps` then contains an `.asm` file.

## Flamegraph

To generate a flamegraph you have to first set `debug = true` in `./.cargo/config.toml`.  
Then build the binary to run locally (see above).  
Now run `flamegraph -- ./target/release/TUW-EP data/f1.csv data/f2.csv data/f3.csv data/f4.csv >/dev/null`.  
There should now be a `flamegraph.svg` which you can open, navigate and search.

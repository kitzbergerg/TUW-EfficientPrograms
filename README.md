# Setup

[Install Rust](https://www.rust-lang.org/tools/install)  
Add target: `rustup target add x86_64-unknown-linux-musl`

Download dataset and extract into `data` directory.

# Development

Commands to benchmark on g0. (scp only works like this if you have ssh setup with config)

```sh
cargo build --release --target=x86_64-unknown-linux-musl
scp ./target/x86_64-unknown-linux-musl/release/TUW-EP g0.complang.tuwien.ac.at:~

ssh g0.complang.tuwien.ac.at

cd /localtmp/efficient24
LC_NUMERIC=en_US perf stat -e cycles ~/TUW-EP f1.csv f2.csv f3.csv f4.csv|cat >/dev/null
```

Commands to run locally.

```sh
cargo run --release data/a.csv data/b.csv data/c.csv data/d.csv | sort | diff - data/abcd.csv
```

# Benchmarks

Benchmarks can be found [here](stats.md)

cargo build --release --target=x86_64-unknown-linux-musl
scp ./target/x86_64-unknown-linux-musl/release/TUW-EP g0.complang.tuwien.ac.at:~

cd /localtmp/efficient24
LC_NUMERIC=en_US perf stat -e cycles ~/TUW-EP f1.csv f2.csv f3.csv f4.csv|cat >/dev/null
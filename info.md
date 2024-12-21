# Info

# joins

```
a.csv 1-1 b.csv
            1
            |
            1
d.csv 1-2 c.csv
```

field length: 7-22 chars

num unique fields: 9996607

num unique bytes in fields: 37  
unique bytes in fields: 0x0 0123456789 ABCDEFGHIJKLMNOPQRSTUVWXYZ (first is null byte; there are no spaces, just for visualization)

# Commands

## Find key lengths

```sh
polars open data/f1.csv --no-header | polars select "column_1" | polars str-lengths | polars min | polars collect
polars open data/f1.csv --no-header | polars select "column_1" | polars str-lengths | polars max | polars collect
polars open data/f1.csv --no-header | polars select "column_2" | polars str-lengths | polars min | polars collect
polars open data/f1.csv --no-header | polars select "column_2" | polars str-lengths | polars max | polars collect

polars open data/f2.csv --no-header | polars select "column_1" | polars str-lengths | polars min | polars collect
polars open data/f2.csv --no-header | polars select "column_1" | polars str-lengths | polars max | polars collect
polars open data/f2.csv --no-header | polars select "column_2" | polars str-lengths | polars min | polars collect
polars open data/f2.csv --no-header | polars select "column_2" | polars str-lengths | polars max | polars collect

polars open data/f3.csv --no-header | polars select "column_1" | polars str-lengths | polars min | polars collect
polars open data/f3.csv --no-header | polars select "column_1" | polars str-lengths | polars max | polars collect
polars open data/f3.csv --no-header | polars select "column_2" | polars str-lengths | polars min | polars collect
polars open data/f3.csv --no-header | polars select "column_2" | polars str-lengths | polars max | polars collect

polars open data/f4.csv --no-header | polars select "column_1" | polars str-lengths | polars min | polars collect
polars open data/f4.csv --no-header | polars select "column_1" | polars str-lengths | polars max | polars collect
polars open data/f4.csv --no-header | polars select "column_2" | polars str-lengths | polars min | polars collect
polars open data/f4.csv --no-header | polars select "column_2" | polars str-lengths | polars max | polars collect
```

## Find num of unique keys

```sh
let f1_1 = (polars open data/f1.csv --no-header | polars select "column_1" | polars rename "column_1" "key")
let f1_2 = (polars open data/f1.csv --no-header | polars select "column_2" | polars rename "column_2" "key")

let f2_1 = (polars open data/f2.csv --no-header | polars select "column_1" | polars rename "column_1" "key")
let f2_2 = (polars open data/f2.csv --no-header | polars select "column_2" | polars rename "column_2" "key")

let f3_1 = (polars open data/f3.csv --no-header | polars select "column_1" | polars rename "column_1" "key")
let f3_2 = (polars open data/f3.csv --no-header | polars select "column_2" | polars rename "column_2" "key")

let f4_1 = (polars open data/f4.csv --no-header | polars select "column_1" | polars rename "column_1" "key")
let f4_2 = (polars open data/f4.csv --no-header | polars select "column_2" | polars rename "column_2" "key")

polars concat $f1_1 $f1_2 $f2_1 $f2_2 $f3_1 $f3_2 $f4_1 $f4_2 | polars n-unique
```

## Store unique keys

```sh
polars concat $f1_1 $f1_2 $f2_1 $f2_2 $f3_1 $f3_2 $f4_1 $f4_2 | polars unique | polars collect | polars save data/unique_keys.csv --csv-no-header
```

## Find unique key chars

commit hash b787e68632cf2723728d4202cb64815ac0c7a7ef

```rust
fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut reader1 = open_reader(&args[1]);
    let mut reader2 = open_reader(&args[2]);
    let mut reader3 = open_reader(&args[3]);
    let mut reader4 = open_reader(&args[4]);

    let unique = stream_data(&mut reader1)
        .chain(stream_data(&mut reader2))
        .chain(stream_data(&mut reader3))
        .chain(stream_data(&mut reader4))
        .flat_map(|(key1, key2)| key1.iter().chain(key2))
        .fold(HashSet::new(), |mut acc, el| {
            acc.insert(el);
            acc
        });
    let mut sorted_vec = unique.into_iter().collect::<Vec<_>>();
    sorted_vec.sort();
    println!("{} unique chars", sorted_vec.len());
    sorted_vec.iter().for_each(|&b| {
        print!("0x{b},");
    });
    println!("");
    sorted_vec.iter().for_each(|&b| {
        print!("{},", std::str::from_utf8(&[*b]).unwrap());
    });
    println!("");
}
```

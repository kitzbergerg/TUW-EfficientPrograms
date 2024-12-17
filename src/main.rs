use csv::ReaderBuilder;
use std::collections::HashMap;
use std::io::stdout;
use std::io::BufWriter;
use std::io::Write;

// a.csv 1-1 b.csv
//             1
//             |
//             1
// d.csv 1-2 c.csv

fn read(file: &str) -> Vec<(Vec<u8>, Vec<u8>)> {
    ReaderBuilder::new()
        .has_headers(false)
        .from_path(file)
        .unwrap()
        .into_byte_records()
        .map(Result::unwrap)
        .map(|result| {
            (
                result.get(0).unwrap().to_vec(),
                result.get(1).unwrap().to_vec(),
            )
        })
        .collect()
}

fn hash_join<'a>(
    left: &HashMap<&'a [u8], Vec<Vec<&'a [u8]>>>,
    right: &HashMap<&'a [u8], Vec<&'a [u8]>>,
    new_key: usize,
) -> HashMap<&'a [u8], Vec<Vec<&'a [u8]>>> {
    let mut result = HashMap::new();
    for (key, left_rows) in left {
        if let Some(right_rows) = right.get(key) {
            for left_row in left_rows {
                for right_row in right_rows {
                    let mut combined = left_row.to_vec();
                    combined.push(right_row);
                    result
                        .entry(combined[new_key])
                        .or_insert_with(Vec::new)
                        .push(combined);
                }
            }
        }
    }
    result
}

fn write_output<W: Write>(data: &HashMap<&[u8], Vec<Vec<&[u8]>>>, writer: &mut BufWriter<W>) {
    data.values().for_each(|rows| {
        rows.into_iter().for_each(|v| {
            writer.write_all(&v[3]).unwrap();
            writer.write(b",").unwrap();
            writer.write_all(&v[0]).unwrap();
            writer.write(b",").unwrap();
            writer.write_all(&v[1]).unwrap();
            writer.write(b",").unwrap();
            writer.write_all(&v[2]).unwrap();
            writer.write(b",").unwrap();
            writer.write_all(&v[4]).unwrap();
            writer.write(b"\n").unwrap();
        });
    });

    writer.flush().unwrap();
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut a = HashMap::new();
    let a_vec = read(&args[1]);
    for (key, value) in a_vec.iter() {
        a.entry(key.as_slice())
            .or_insert_with(Vec::new)
            .push(vec![key.as_slice(), value.as_slice()]);
    }

    let mut b = HashMap::new();
    let b_vec = read(&args[2]);
    for (key, value) in b_vec.iter() {
        b.entry(key.as_slice())
            .or_insert_with(Vec::new)
            .push(value.as_slice());
    }

    let mut c = HashMap::new();
    let c_vec = read(&args[3]);
    for (key, value) in c_vec.iter() {
        c.entry(key.as_slice())
            .or_insert_with(Vec::new)
            .push(value.as_slice());
    }

    let mut d = HashMap::new();
    let d_vec = read(&args[4]);
    for (key, value) in d_vec.iter() {
        d.entry(key.as_slice())
            .or_insert_with(Vec::new)
            .push(value.as_slice());
    }

    // Perform joins
    let map = hash_join(&a, &b, 0);
    let map = hash_join(&map, &c, 3);
    let map = hash_join(&map, &d, 0);

    // Write output
    let stdout = stdout();
    let mut writer = BufWriter::new(stdout.lock());
    write_output(&map, &mut writer);
}

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

type CsvField = Vec<u8>;

fn read(file: &str) -> impl Iterator<Item = (CsvField, CsvField)> {
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
}

fn hash_join(
    left: HashMap<CsvField, Vec<Vec<CsvField>>>,
    right: HashMap<CsvField, Vec<CsvField>>,
    new_key: usize,
) -> HashMap<CsvField, Vec<Vec<CsvField>>> {
    left.iter()
        .filter_map(|(key, left_rows)| {
            right.get(key).map(|right_matches| {
                // build cross product
                left_rows.iter().flat_map(|left_row| {
                    right_matches.iter().map(|right_element| {
                        let mut new_row = left_row.clone();
                        new_row.push(right_element.clone());
                        new_row
                    })
                })
            })
        })
        .fold(
            HashMap::new(),
            |mut acc: HashMap<CsvField, Vec<Vec<CsvField>>>, rows| {
                rows.into_iter().for_each(|value| {
                    acc.entry(value[new_key].clone()).or_default().push(value);
                });
                acc
            },
        )
}

fn write_output<W: Write>(data: HashMap<CsvField, Vec<Vec<CsvField>>>, writer: &mut BufWriter<W>) {
    data.into_values().for_each(|rows| {
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
    let mut a: HashMap<CsvField, Vec<Vec<CsvField>>> = HashMap::new();
    for (key, value) in read(&args[1]) {
        a.entry(key.clone())
            .or_default()
            .push(vec![key, value.clone()]);
    }

    let mut b: HashMap<CsvField, Vec<CsvField>> = HashMap::new();
    for (key, value) in read(&args[2]) {
        b.entry(key).or_default().push(value);
    }

    let mut c: HashMap<CsvField, Vec<CsvField>> = HashMap::new();
    for (key, value) in read(&args[3]) {
        c.entry(key).or_default().push(value);
    }

    let mut d: HashMap<CsvField, Vec<CsvField>> = HashMap::new();
    for (key, value) in read(&args[4]) {
        d.entry(key).or_default().push(value);
    }

    // Perform joins
    let map = hash_join(a, b, 0);
    let map = hash_join(map, c, 3);
    let map = hash_join(map, d, 0);

    // Write output
    let stdout = stdout();
    let mut writer = BufWriter::new(stdout.lock());
    write_output(map, &mut writer);
}

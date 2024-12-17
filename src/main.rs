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
fn hash_join(
    left: &HashMap<Vec<u8>, Vec<Vec<Vec<u8>>>>,
    right: Vec<(Vec<u8>, Vec<u8>)>,
    new_key: usize,
) -> HashMap<Vec<u8>, Vec<Vec<Vec<u8>>>> {
    // Build a hash index for the right table
    let mut right_map: HashMap<Vec<u8>, Vec<Vec<u8>>> = HashMap::new();
    right.into_iter().for_each(|(key, value)| {
        if let Some(first) = right_map.get_mut(&key) {
            first.push(value);
        } else {
            right_map.insert(key.clone(), vec![value]);
        }
    });

    // Perform the join
    left.iter()
        .filter_map(|(key, left_rows)| {
            right_map.get(key).map(|right_matches| {
                // build cross product
                let mut new_vec = Vec::with_capacity(left_rows.len() * right_matches.len());
                left_rows.iter().for_each(|left_row| {
                    right_matches.iter().for_each(|right_element| {
                        let mut new_row = left_row.clone();
                        new_row.push(right_element.clone());
                        new_vec.push(new_row);
                    })
                });
                new_vec
            })
        })
        .fold(
            HashMap::new(),
            |mut acc: HashMap<Vec<u8>, Vec<Vec<Vec<u8>>>>, rows| {
                rows.into_iter().for_each(|row| {
                    let key = row[new_key].clone();
                    if let Some(first) = acc.get_mut(&key) {
                        first.push(row);
                    } else {
                        acc.insert(key, vec![row]);
                    }
                });
                acc
            },
        )
}

fn write_output<W: Write>(data: &HashMap<Vec<u8>, Vec<Vec<Vec<u8>>>>, writer: &mut BufWriter<W>) {
    data.into_iter().for_each(|(_, value)| {
        value.into_iter().for_each(|v| {
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
    let a = read(&args[1]);
    let b = read(&args[2]);
    let c = read(&args[3]);
    let d = read(&args[4]);

    let mut map: HashMap<Vec<u8>, Vec<Vec<Vec<u8>>>> = HashMap::new();
    a.into_iter().for_each(|(key, value)| {
        if let Some(first) = map.get_mut(&key) {
            first.push(vec![key, value.clone()]);
        } else {
            map.insert(key.clone(), vec![vec![key, value]]);
        }
    });

    // Perform joins
    map = hash_join(&map, b, 0);
    map = hash_join(&map, c, 3);
    map = hash_join(&map, d, 0);

    // Write output
    let stdout = stdout();
    let mut writer = BufWriter::new(stdout.lock());
    write_output(&map, &mut writer);
}

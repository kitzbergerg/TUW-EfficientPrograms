use csv::ReaderBuilder;
use itertools::Itertools;
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

fn rebuild_index(
    map: HashMap<Vec<u8>, Vec<Vec<Vec<u8>>>>,
    col: usize,
) -> HashMap<Vec<u8>, Vec<Vec<Vec<u8>>>> {
    let mut new_map: HashMap<Vec<u8>, Vec<Vec<Vec<u8>>>> = HashMap::with_capacity(map.len());
    map.into_values()
        .into_iter()
        .flatten()
        .for_each(|mut line| {
            line.swap(0, col);
            if let Some(first) = new_map.get_mut(&line[0]) {
                first.push(line);
            } else {
                new_map.insert(line[0].clone(), vec![line]);
            }
        });
    return new_map;
}

fn write<W: Write>(map: HashMap<Vec<u8>, Vec<Vec<Vec<u8>>>>, writer: &mut BufWriter<W>) {
    map.into_iter().for_each(|(_, value)| {
        value.into_iter().for_each(|v| {
            writer.write_all(&v[0]).unwrap();
            writer.write(b",").unwrap();
            writer.write_all(&v[3]).unwrap();
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
            first.push(vec![key, value]);
        } else {
            map.insert(key.clone(), vec![vec![key, value]]);
        }
    });
    map = join(map, b);
    map = join(map, c);
    map = rebuild_index(map, 3);
    map = join(map, d);

    let mut writer = BufWriter::new(stdout());
    write(map, &mut writer);
}

fn join(
    mut map: HashMap<Vec<u8>, Vec<Vec<Vec<u8>>>>,
    b: Vec<(Vec<u8>, Vec<u8>)>,
) -> HashMap<Vec<u8>, Vec<Vec<Vec<u8>>>> {
    let mut new_map: HashMap<Vec<u8>, Vec<Vec<Vec<u8>>>> = HashMap::new();
    b.into_iter()
        .sorted_by(|(a, _), (b, _)| a.cmp(b))
        .chunk_by(|(key, _)| key.clone())
        .into_iter()
        .map(|(key, value)| (key, value.collect_vec()))
        .for_each(|(key, value)| {
            if let Some(first) = map.remove(&key) {
                first
                    .into_iter()
                    .map(|line| {
                        value
                            .clone()
                            .into_iter()
                            .map(|(_, to_add)| {
                                let mut line = line.clone();
                                line.push(to_add.clone());
                                line
                            })
                            .collect_vec()
                    })
                    .for_each(|v| {
                        if let Some(first) = new_map.get_mut(&key) {
                            first.extend(v);
                        } else {
                            new_map.insert(key.clone(), v);
                        }
                    });
            }
        });
    new_map
}

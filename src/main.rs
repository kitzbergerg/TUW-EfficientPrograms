use csv::ReaderBuilder;
use itertools::Itertools;
use serde::de::value;
use std::collections::HashMap;
use std::io::stdout;
use std::io::BufWriter;
use std::io::Write;

// a.csv 1-1 b.csv
//             1
//             |
//             1
// d.csv 1-2 c.csv

fn read(file: &str) -> Vec<(String, String)> {
    let mut reader = ReaderBuilder::new()
        .has_headers(false)
        .from_path(file)
        .unwrap();
    reader
        .records()
        .map(Result::unwrap)
        .map(|result| {
            (
                result.get(0).unwrap().to_string(),
                result.get(1).unwrap().to_string(),
            )
        })
        .collect()
}

fn rebuild_index(
    map: HashMap<String, Vec<Vec<String>>>,
    col: usize,
) -> HashMap<String, Vec<Vec<String>>> {
    let mut new_map: HashMap<String, Vec<Vec<String>>> = HashMap::with_capacity(map.len());
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

fn write<W: Write>(map: HashMap<String, Vec<Vec<String>>>, writer: &mut BufWriter<W>) {
    map.into_iter().for_each(|(_, value)| {
        value.into_iter().for_each(|v| {
            writeln!(writer, "{},{},{},{},{}", v[0], v[3], v[1], v[2], v[4]).unwrap();
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

    let mut map: HashMap<String, Vec<Vec<String>>> = HashMap::new();
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
    mut map: HashMap<String, Vec<Vec<String>>>,
    b: Vec<(String, String)>,
) -> HashMap<String, Vec<Vec<String>>> {
    let mut new_map: HashMap<String, Vec<Vec<String>>> = HashMap::new();
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

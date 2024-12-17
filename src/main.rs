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

fn read(file: &str) -> HashMap<String, Vec<Vec<String>>> {
    let mut reader = ReaderBuilder::new()
        .has_headers(false)
        .from_path(file)
        .unwrap();
    let mut map: HashMap<String, Vec<Vec<String>>> = HashMap::new();
    for result in reader.records().map(Result::unwrap) {
        let key = result.get(0).unwrap().to_string();
        let value = result.get(1).unwrap().to_string();

        if let Some(first) = map.get_mut(&key) {
            first.push(vec![key, value]);
        } else {
            map.insert(key.clone(), vec![vec![key, value]]);
        }
    }
    return map;
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

fn join_csv(
    map1: HashMap<String, Vec<Vec<String>>>,
    map2: HashMap<String, Vec<Vec<String>>>,
) -> HashMap<String, Vec<Vec<String>>> {
    let mut new_map: HashMap<String, Vec<Vec<String>>> = HashMap::new();
    map2.into_iter().for_each(|(key, value)| {
        if let Some(first) = map1.get(&key) {
            first.iter().for_each(|line| {
                value.iter().for_each(|to_add| {
                    let mut line = line.clone();
                    line.extend_from_slice(&to_add[1..]);
                    if let Some(first) = new_map.get_mut(&key) {
                        first.push(line);
                    } else {
                        new_map.insert(line[0].clone(), vec![line]);
                    }
                });
            });
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

    let joined_ab = join_csv(a, b);
    let joined_abc = join_csv(joined_ab, c);
    let joined_abc = rebuild_index(joined_abc, 3);
    let joined = join_csv(joined_abc, d);

    let mut writer = BufWriter::new(stdout());
    write(joined, &mut writer);
}

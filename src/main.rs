use fxhash::FxHashMap;
use memmap::Mmap;
use memmap::MmapOptions;
use std::fs::File;
use std::io::stdout;
use std::io::BufWriter;
use std::io::Write;

// a.csv 1-1 b.csv
//             1
//             |
//             1
// d.csv 1-2 c.csv

fn open_reader(file: &str) -> Mmap {
    let file = File::open(file).unwrap();
    unsafe { MmapOptions::new().map(&file).unwrap() }
}

fn read<'a>(reader: &'a Mmap) -> impl Iterator<Item = (&'a [u8], &'a [u8])> {
    reader
        .split(|b| b == &b'\n')
        .filter(|row| row.len() > 0)
        .map(|row| {
            let mut iter = row.split(|b| b == &b',');
            (iter.next().unwrap(), iter.next().unwrap())
        })
}

fn hash_join<'a>(
    left: FxHashMap<&'a [u8], Vec<Vec<&'a [u8]>>>,
    right: impl Iterator<Item = (&'a [u8], &'a [u8])>,
    new_key: usize,
) -> FxHashMap<&'a [u8], Vec<Vec<&'a [u8]>>> {
    let mut result = FxHashMap::default();
    for (key, value) in right {
        if let Some(left_rows) = left.get(key) {
            let mut left_rows_copy = left_rows.clone();
            left_rows_copy.iter_mut().for_each(|row| row.push(value));
            left_rows_copy.into_iter().for_each(|row| {
                result
                    .entry(row[new_key])
                    .or_insert_with(Vec::new)
                    .push(row)
            });
        }
    }
    result
}

fn write_output<W: Write>(data: &FxHashMap<&[u8], Vec<Vec<&[u8]>>>, writer: &mut BufWriter<W>) {
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

    let mut reader = open_reader(&args[1]);
    let mut map = FxHashMap::default();
    let a = read(&mut reader).collect::<Vec<_>>();
    for (key, value) in a.iter() {
        map.entry(*key)
            .or_insert_with(Vec::new)
            .push(vec![*key, *value]);
    }

    let mut reader = open_reader(&args[2]);
    let b = read(&mut reader);
    let map = hash_join(map, b, 0);

    let mut reader = open_reader(&args[3]);
    let c = read(&mut reader);
    let map = hash_join(map, c, 3);

    let mut reader = open_reader(&args[4]);
    let d = read(&mut reader);
    let map = hash_join(map, d, 0);

    // Write output
    let stdout = stdout();
    let mut writer = BufWriter::new(stdout.lock());
    write_output(&map, &mut writer);
}

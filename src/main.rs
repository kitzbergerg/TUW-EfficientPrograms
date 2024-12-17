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

type CsvField<'a> = &'a [u8];

fn open_reader(file: &str) -> Mmap {
    let file = File::open(file).unwrap();
    unsafe { MmapOptions::new().map(&file).unwrap() }
}

fn read<'a>(reader: &'a Mmap) -> impl Iterator<Item = (CsvField<'a>, CsvField<'a>)> {
    reader
        .split(|b| b == &b'\n')
        .filter(|row| row.len() > 0)
        .map(|row| {
            let mut iter = row.split(|b| b == &b',');
            (iter.next().unwrap(), iter.next().unwrap())
        })
}

fn hash_join<'a, const N0: usize, const N1: usize>(
    left: FxHashMap<CsvField<'a>, Vec<[CsvField<'a>; N0]>>,
    right: impl Iterator<Item = (CsvField<'a>, CsvField<'a>)>,
    new_key: usize,
) -> FxHashMap<CsvField<'a>, Vec<[CsvField<'a>; N1]>> {
    let mut result = FxHashMap::default();
    for (key, value) in right {
        if let Some(left_rows) = left.get(key) {
            left_rows
                .iter()
                .map(|row| {
                    let mut new_row: [&[u8]; N1] = [&[]; N1];
                    new_row[0..N0].copy_from_slice(row);
                    new_row[N0] = value;
                    new_row
                })
                .for_each(|row| {
                    result
                        .entry(row[new_key])
                        .or_insert_with(Vec::new)
                        .push(row)
                });
        }
    }
    result
}

fn write_output<W: Write>(data: &FxHashMap<&[u8], Vec<[&[u8]; 5]>>, writer: &mut BufWriter<W>) {
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
            .push([*key, *value]);
    }

    let mut reader = open_reader(&args[2]);
    let b = read(&mut reader);
    let map = hash_join::<2, 3>(map, b, 0);

    let mut reader = open_reader(&args[3]);
    let c = read(&mut reader);
    let map = hash_join::<3, 4>(map, c, 3);

    let mut reader = open_reader(&args[4]);
    let d = read(&mut reader);
    let map = hash_join::<4, 5>(map, d, 0);

    // Write output
    let stdout = stdout();
    let mut writer = BufWriter::new(stdout.lock());
    write_output(&map, &mut writer);
}

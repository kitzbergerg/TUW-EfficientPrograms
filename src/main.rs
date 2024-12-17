use fxhash::FxHashMap;
use memmap::Mmap;
use memmap::MmapOptions;
use smallvec::SmallVec;
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
type SV<T> = [T; 4];

fn open_reader(file: &str) -> Mmap {
    let file = File::open(file).unwrap();
    unsafe { MmapOptions::new().map(&file).unwrap() }
}

fn stream_data<'a>(reader: &'a Mmap) -> impl Iterator<Item = (CsvField<'a>, CsvField<'a>)> {
    reader
        .split(|&b| b == b'\n')
        .filter(|row| !row.is_empty())
        .map(|row| {
            let mut iter = row.split(|&b| b == b',');
            (iter.next().unwrap(), iter.next().unwrap())
        })
}

fn join<'a, const N0: usize, const N1: usize>(
    left: FxHashMap<CsvField<'a>, SmallVec<SV<[CsvField<'a>; N0]>>>,
    right: impl Iterator<Item = (CsvField<'a>, CsvField<'a>)>,
    new_key: usize,
) -> FxHashMap<CsvField<'a>, SmallVec<SV<[CsvField<'a>; N1]>>> {
    let mut result = FxHashMap::with_capacity_and_hasher(4000000, Default::default());
    right
        .filter_map(|(key, value)| left.get(key).map(|rows| (rows, value)))
        .for_each(|(rows, value)| {
            rows.iter()
                .map(|row| {
                    let mut new_row: [&[u8]; N1] = [&[]; N1];
                    new_row[0..N0].copy_from_slice(row);
                    new_row[N0] = value;
                    new_row
                })
                .for_each(|row| {
                    result
                        .entry(row[new_key])
                        .or_insert_with(SmallVec::<SV<_>>::new_const)
                        .push(row)
                });
        });
    result
}

fn write_output<W: Write>(
    data: &FxHashMap<&[u8], SmallVec<SV<[&[u8]; 5]>>>,
    writer: &mut BufWriter<W>,
) {
    data.values()
        .map(IntoIterator::into_iter)
        .flatten()
        .for_each(|v| {
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

    writer.flush().unwrap();
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut map = FxHashMap::with_capacity_and_hasher(7000000, Default::default());

    let mut reader = open_reader(&args[1]);
    stream_data(&mut reader).for_each(|(key, value)| {
        map.entry(key)
            .or_insert_with(SmallVec::<SV<_>>::new_const)
            .push([key, value]);
    });

    let mut reader = open_reader(&args[2]);
    let map = join::<2, 3>(map, stream_data(&mut reader), 0);

    let mut reader = open_reader(&args[3]);
    let map = join::<3, 4>(map, stream_data(&mut reader), 3);

    let mut reader = open_reader(&args[4]);
    let map = join::<4, 5>(map, stream_data(&mut reader), 0);

    // Write output
    let stdout = stdout();
    let mut writer = BufWriter::new(stdout.lock());
    write_output(&map, &mut writer);
}

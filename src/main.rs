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

fn stream_data<'a>(reader: &'a Mmap) -> impl Iterator<Item = (CsvField<'a>, CsvField<'a>)> {
    reader
        .split(|&b| b == b'\n')
        .filter(|row| !row.is_empty())
        .map(|row| {
            let mut iter = row.split(|&b| b == b',');
            (iter.next().unwrap(), iter.next().unwrap())
        })
}

fn join<'a>(
    left: FxHashMap<CsvField<'a>, Vec<[CsvField<'a>; 5]>>,
    right: impl Iterator<Item = (CsvField<'a>, CsvField<'a>)>,
    pos: usize,
    new_key: usize,
) -> FxHashMap<CsvField<'a>, Vec<[CsvField<'a>; 5]>> {
    let mut result = FxHashMap::default();
    right
        .filter_map(|(key, value)| left.get(key).map(|rows| (rows, value)))
        .for_each(|(rows, value)| {
            rows.iter().cloned().for_each(|mut row| {
                row[pos] = value;
                result
                    .entry(row[new_key])
                    .or_insert_with(Vec::new)
                    .push(row)
            });
        });
    result
}

fn write_output<W: Write>(data: &FxHashMap<&[u8], Vec<[&[u8]; 5]>>, writer: &mut BufWriter<W>) {
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
    let mut map = FxHashMap::default();

    let mut reader = open_reader(&args[1]);
    stream_data(&mut reader).for_each(|(key, value)| {
        map.entry(key)
            .or_insert_with(Vec::new)
            .push([key, value, &[], &[], &[]]);
    });

    let mut reader = open_reader(&args[2]);
    let map = join(map, stream_data(&mut reader), 2, 0);

    let mut reader = open_reader(&args[3]);
    let map = join(map, stream_data(&mut reader), 3, 3);

    let mut reader = open_reader(&args[4]);
    let map = join(map, stream_data(&mut reader), 4, 0);

    // Write output
    let stdout = stdout();
    let mut writer = BufWriter::new(stdout.lock());
    write_output(&map, &mut writer);
}

#![feature(portable_simd)]
use hash::MyHashMap;
use memmap2::Mmap;
use mimalloc::MiMalloc;
use simd_csv_reader::IntoCsvReader;
use smallvec::SmallVec;
use smallvec::smallvec;
use std::collections::HashMap;
use std::fs::File;
use std::hash::BuildHasher;
use std::io::BufWriter;
use std::io::Write;
use std::io::stdout;

mod hash;
mod simd_csv_reader;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

type CsvField<'a> = &'a [u8];
type SV2<T> = SmallVec<[T; 2]>;
type SV3<T> = SmallVec<[T; 3]>;

fn open_reader(file: &str) -> Mmap {
    let file = File::open(file).unwrap();
    unsafe { Mmap::map(&file).unwrap() }
}

fn stream_data(data: &Mmap) -> impl Iterator<Item = (CsvField<'_>, CsvField<'_>)> {
    data.parse_csv()
}

fn write_output<'a, W: Write, S: BuildHasher>(
    writer: &mut BufWriter<W>,
    abc: &HashMap<CsvField<'a>, SV3<SV2<CsvField<'a>>>, S>,
    d_map: &HashMap<CsvField<'a>, SV2<CsvField<'a>>, S>,
) {
    for (key, [a_cols2, b_cols2, c_cols2]) in abc {
        for c_col2 in c_cols2 {
            if let Some(d_cols2) = d_map.get(c_col2) {
                for d_col2 in d_cols2 {
                    for b_col2 in b_cols2 {
                        for a_col2 in a_cols2 {
                            writer.write_all(c_col2).unwrap();
                            writer.write_all(b",").unwrap();
                            writer.write_all(key).unwrap();
                            writer.write_all(b",").unwrap();
                            writer.write_all(a_col2).unwrap();
                            writer.write_all(b",").unwrap();
                            writer.write_all(b_col2).unwrap();
                            writer.write_all(b",").unwrap();
                            writer.write_all(d_col2).unwrap();
                            writer.write_all(b"\n").unwrap();
                        }
                    }
                }
            }
        }
    }
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut abc_map = MyHashMap::with_capacity_and_hasher(5_000_000, Default::default());
    let mut d_map = MyHashMap::with_capacity_and_hasher(5_000_000, Default::default());

    let reader1 = open_reader(&args[1]);
    let reader2 = open_reader(&args[2]);
    let reader3 = open_reader(&args[3]);
    let reader4 = open_reader(&args[4]);

    stream_data(&reader2).for_each(|(key, value)| {
        abc_map
            .entry(key)
            .and_modify(|vec: &mut SV3<SV2<_>>| vec[0].push(value))
            .or_insert({
                let mut sv: SV3<SV2<_>> = smallvec![SmallVec::with_capacity(2); 3];
                sv[0].push(value);
                sv
            });
    });
    stream_data(&reader1).for_each(|(key, value)| {
        if let Some(vec) = abc_map.get_mut(key) {
            vec[1].push(value);
        }
    });
    stream_data(&reader3).for_each(|(key, value)| {
        if let Some(vec) = abc_map.get_mut(key) {
            vec[2].push(value);
        }
    });
    stream_data(&reader4).for_each(|(key, value)| {
        d_map
            .entry(key)
            .and_modify(|vec: &mut SV2<_>| vec.push(value))
            .or_insert(smallvec![value]);
    });

    // Write output
    let stdout = stdout();
    let mut writer = BufWriter::with_capacity(256 * 1024, stdout.lock());
    write_output(&mut writer, &abc_map, &d_map);
}

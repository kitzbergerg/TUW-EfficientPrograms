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
    abc.iter()
        .filter(|(_, vec)| vec.len() == 3)
        .flat_map(|(key, abc_cols2)| {
            abc_cols2[2]
                .iter()
                .filter_map(|c_col2| d_map.get(c_col2).zip(Some(c_col2)))
                .flat_map(move |(d_cols2, c_col2)| {
                    d_cols2.iter().flat_map(move |d_col2| {
                        abc_cols2[0].iter().flat_map(move |a_col2| {
                            abc_cols2[1]
                                .iter()
                                .map(move |b_col2| (key, a_col2, b_col2, c_col2, d_col2))
                        })
                    })
                })
        })
        .for_each(|(abc_col1, a_col2, b_col2, c_col2, d_col2)| {
            writer.write_all(c_col2).unwrap();
            writer.write_all(b",").unwrap();
            writer.write_all(abc_col1).unwrap();
            writer.write_all(b",").unwrap();
            writer.write_all(a_col2).unwrap();
            writer.write_all(b",").unwrap();
            writer.write_all(b_col2).unwrap();
            writer.write_all(b",").unwrap();
            writer.write_all(d_col2).unwrap();
            writer.write_all(b"\n").unwrap();
        });
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut abc_map = MyHashMap::with_capacity_and_hasher(5_000_000, Default::default());
    let mut d_map = MyHashMap::with_capacity_and_hasher(5_000_000, Default::default());

    let reader1 = open_reader(&args[1]);
    let reader2 = open_reader(&args[2]);
    let reader3 = open_reader(&args[3]);
    let reader4 = open_reader(&args[4]);

    stream_data(&reader1).for_each(|(key, value)| {
        abc_map
            .entry(key)
            .and_modify(|vec: &mut SV3<SV2<_>>| vec[0].push(value))
            .or_insert({
                let mut sv: SV3<SV2<_>> = smallvec![SmallVec::with_capacity(2); 3];
                sv[0].push(value);
                sv
            });
    });
    stream_data(&reader2).for_each(|(key, value)| {
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

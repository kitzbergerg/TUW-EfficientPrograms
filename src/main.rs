#![feature(portable_simd)]
use hash::new_precomputed_hashmap;
use memmap2::Mmap;
use mimalloc::MiMalloc;
use simd_csv_reader::IntoCsvReader;
use smallvec::SmallVec;
use std::collections::HashMap;
use std::fs::File;
use std::hash::BuildHasher;
use std::io::stdout;
use std::io::BufWriter;
use std::io::Write;

mod hash;
mod simd_csv_reader;

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

type CsvField<'a> = &'a [u8];
type SvAbc<T> = [T; 3];
type SvMultiValue<T> = [T; 2];

fn open_reader(file: &str) -> Mmap {
    let file = File::open(file).unwrap();
    unsafe { Mmap::map(&file).unwrap() }
}

fn stream_data<'a>(data: &'a Mmap) -> impl Iterator<Item = (CsvField<'a>, CsvField<'a>)> {
    data.parse_csv()
}

fn write_output<'a, W: Write, S: BuildHasher>(
    writer: &mut BufWriter<W>,
    abc: HashMap<CsvField<'a>, SmallVec<SvAbc<SmallVec<SvMultiValue<CsvField<'a>>>>>, S>,
    d_map: HashMap<CsvField<'a>, SmallVec<SvMultiValue<CsvField<'a>>>, S>,
) {
    abc.iter()
        .filter(|(_, vec)| vec.len() == 3)
        .map(|(key, abc_cols2)| (key, &abc_cols2[0], &abc_cols2[1], &abc_cols2[2]))
        .flat_map(|(abc_col1, a_cols2, b_cols2, c_cols2)| {
            c_cols2
                .iter()
                .filter_map(|c_col2| d_map.get(c_col2).map(|d_cols2| (c_col2, d_cols2)))
                .flat_map(move |(c_col2, d_cols2)| {
                    d_cols2.iter().flat_map(move |d_col2| {
                        a_cols2.iter().flat_map(move |a_col2| {
                            b_cols2
                                .iter()
                                .map(move |b_col2| (abc_col1, a_col2, b_col2, c_col2, d_col2))
                        })
                    })
                })
        })
        .for_each(|(abc_col1, a_col2, b_col2, c_col2, d_col2)| {
            writer.write_all(c_col2).unwrap();
            writer.write(b",").unwrap();
            writer.write_all(abc_col1).unwrap();
            writer.write(b",").unwrap();
            writer.write_all(a_col2).unwrap();
            writer.write(b",").unwrap();
            writer.write_all(b_col2).unwrap();
            writer.write(b",").unwrap();
            writer.write_all(d_col2).unwrap();
            writer.write(b"\n").unwrap();
        });
}

fn main() {
    let args: Vec<String> = std::env::args().collect();

    let mut abc_map = new_precomputed_hashmap(2500000);
    let mut d_map = new_precomputed_hashmap(2500000);

    let mut reader1 = open_reader(&args[1]);
    let mut reader2 = open_reader(&args[2]);
    let mut reader3 = open_reader(&args[3]);
    let mut reader4 = open_reader(&args[4]);

    stream_data(&mut reader1).for_each(|(key, value)| {
        abc_map
            .entry(key)
            .and_modify(|vec: &mut SmallVec<SvAbc<SmallVec<SvMultiValue<_>>>>| vec[0].push(value))
            .or_insert_with(|| {
                let mut sv = SmallVec::with_capacity(3);
                sv.push(SmallVec::from_slice(&[value]));
                sv.push(SmallVec::new_const());
                sv.push(SmallVec::new_const());
                sv
            });
    });
    stream_data(&mut reader2).for_each(|(key, value)| {
        abc_map.entry(key).and_modify(|vec| vec[1].push(value));
    });
    stream_data(&mut reader3).for_each(|(key, value)| {
        abc_map.entry(key).and_modify(|vec| vec[2].push(value));
    });
    stream_data(&mut reader4).for_each(|(key, value)| {
        d_map
            .entry(key)
            .and_modify(|vec: &mut SmallVec<SvMultiValue<_>>| vec.push(value))
            .or_insert(SmallVec::from_slice(&[value]));
    });

    // Write output
    let stdout = stdout();
    let mut writer = BufWriter::with_capacity(256 * 1024, stdout.lock());
    write_output(&mut writer, abc_map, d_map);
}

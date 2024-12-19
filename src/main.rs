#![feature(portable_simd)]
use field::CsvField;
use fxhash::FxHashMap;
use mimalloc::MiMalloc;
use smallvec::SmallVec;
use std::fs::File;
use std::io::stdout;
use std::io::BufReader;
use std::io::BufWriter;
use std::io::Read;
use std::io::Write;

mod field;

// a.csv 1-1 b.csv
//             1
//             |
//             1
// d.csv 1-2 c.csv

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

type SvAbc<T> = [T; 3];
type SvMultiValue<T> = [T; 2];

fn open_reader(file: &str) -> Vec<u8> {
    let file = File::open(file).unwrap();
    let mut reader = BufReader::with_capacity(256 * 1024, file);
    let mut buffer = Vec::with_capacity(300 * 1024 * 1024);

    reader.read_to_end(&mut buffer).unwrap();
    buffer
}

fn stream_data(data: &Vec<u8>) -> impl Iterator<Item = (CsvField, CsvField)> + '_ {
    data.split(|&b| b == b'\n')
        .filter(|row| !row.is_empty())
        .map(|row| {
            let mut iter = row.splitn(2, |&b| b == b',');

            (
                CsvField::from_slice(iter.next().unwrap()),
                CsvField::from_slice(iter.next().unwrap()),
            )
        })
}

fn write_output<W: Write>(
    writer: &mut BufWriter<W>,
    abc: FxHashMap<CsvField, SmallVec<SvAbc<SmallVec<SvMultiValue<CsvField>>>>>,
    d_map: FxHashMap<CsvField, SmallVec<SvMultiValue<CsvField>>>,
) {
    // Pre-allocate output buffer
    let mut output = Vec::with_capacity(256);

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
            output.clear();

            c_col2.write_trimmed(&mut output);
            output.push(b',');
            abc_col1.write_trimmed(&mut output);
            output.push(b',');
            a_col2.write_trimmed(&mut output);
            output.push(b',');
            b_col2.write_trimmed(&mut output);
            output.push(b',');
            d_col2.write_trimmed(&mut output);
            output.push(b'\n');

            writer.write_all(&output).unwrap();
        });
}

fn main() {
    let args: Vec<String> = std::env::args().collect();
    let mut abc_map = FxHashMap::with_capacity_and_hasher(2500000, Default::default());

    let mut reader = open_reader(&args[1]);
    stream_data(&mut reader).for_each(|(key, value)| {
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

    let mut reader = open_reader(&args[2]);
    stream_data(&mut reader).for_each(|(key, value)| {
        abc_map.entry(key).and_modify(|vec| vec[1].push(value));
    });

    let mut reader = open_reader(&args[3]);
    stream_data(&mut reader).for_each(|(key, value)| {
        abc_map.entry(key).and_modify(|vec| vec[2].push(value));
    });

    let mut d_map = FxHashMap::with_capacity_and_hasher(2500000, Default::default());
    let mut reader = open_reader(&args[4]);
    stream_data(&mut reader).for_each(|(key, value)| {
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

use fxhash::FxHashMap;
use memmap::Mmap;
use memmap::MmapOptions;
use mimalloc::MiMalloc;
use rayon::iter::ParallelBridge;
use rayon::iter::ParallelIterator;
use smallvec::SmallVec;
use std::cmp::max;
use std::cmp::min;
use std::fs::File;
use std::io::stdout;
use std::io::BufWriter;
use std::io::Write;
use std::sync::mpsc::IntoIter;

// a.csv 1-1 b.csv
//             1
//             |
//             1
// d.csv 1-2 c.csv

#[global_allocator]
static GLOBAL: MiMalloc = MiMalloc;

type CsvField<'a> = &'a [u8];
type SV<T> = [T; 4];

pub struct IntoSeqIter<I>(IntoIter<I>);

impl<I> Iterator for IntoSeqIter<I> {
    type Item = I;
    fn next(&mut self) -> Option<Self::Item> {
        self.0.next()
    }
}

pub trait IntoSeqIterator: ParallelIterator {
    fn into_seq_iter(self: Self) -> IntoSeqIter<Self::Item> {
        let (send, recv) = std::sync::mpsc::channel();
        rayon::scope(|s| {
            s.spawn(|_| {
                self.for_each(|el| {
                    let _ = send.send(el);
                })
            })
        });
        IntoSeqIter(recv.into_iter())
    }
}

impl<P: ParallelIterator> IntoSeqIterator for P {}

fn open_reader(file: &str) -> Mmap {
    let file = File::open(file).unwrap();
    unsafe { MmapOptions::new().map(&file).unwrap() }
}

fn stream_data<'a>(reader: &'a Mmap) -> impl ParallelIterator<Item = (CsvField<'a>, CsvField<'a>)> {
    let length = reader.len();
    let num_splits = max(1, min(16, length / 100));
    let split_len = length / num_splits;

    let mut splits = Vec::with_capacity(num_splits);
    for i in 0..num_splits {
        splits.push(i * split_len);
    }

    splits.iter_mut().skip(1).for_each(|el| {
        while reader[*el] != b'\n' {
            *el += 1;
        }
        *el += 1;
    });
    let mut splits2 = splits.clone();
    splits2.push(length);

    let (send, recv) = std::sync::mpsc::channel();

    splits
        .into_iter()
        .zip(splits2.into_iter().skip(1))
        .for_each(|(start, end)| {
            rayon::scope(|s| {
                s.spawn(|_| {
                    reader[start..end]
                        .split(|&b| b == b'\n')
                        .filter(|row| !row.is_empty())
                        .for_each(|row| {
                            let mut iter = row.split(|&b| b == b',');
                            send.send((iter.next().unwrap(), iter.next().unwrap()))
                                .unwrap()
                        });
                })
            })
        });
    recv.into_iter().par_bridge()
}

fn join<'a, const N0: usize, const N1: usize>(
    left: FxHashMap<CsvField<'a>, SmallVec<SV<[CsvField<'a>; N0]>>>,
    right: impl ParallelIterator<Item = (CsvField<'a>, CsvField<'a>)>,
    new_key: usize,
) -> FxHashMap<CsvField<'a>, SmallVec<SV<[CsvField<'a>; N1]>>> {
    let mut map = FxHashMap::with_capacity_and_hasher(4000000, Default::default());
    right
        .filter_map(|(key, value)| left.get(key).map(|rows| (rows, value)))
        .into_seq_iter()
        .for_each(|(rows, value)| {
            rows.iter()
                .map(|row| {
                    let mut new_row: [&[u8]; N1] = [&[]; N1];
                    new_row[0..N0].copy_from_slice(row);
                    new_row[N0] = value;
                    new_row
                })
                .for_each(|row| {
                    map.entry(row[new_key])
                        .and_modify(|vec: &mut SmallVec<SV<_>>| vec.push(row))
                        .or_insert(SmallVec::<SV<_>>::from_slice(&[row]));
                });
        });
    map
}

fn write_output<W: Write>(
    data: &FxHashMap<&[u8], SmallVec<SV<[&[u8]; 5]>>>,
    writer: &mut BufWriter<W>,
) {
    data.values()
        .map(|s| s.into_iter())
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
    stream_data(&mut reader)
        .into_seq_iter()
        .for_each(|(key, value)| {
            map.entry(key)
                .and_modify(|vec: &mut SmallVec<SV<_>>| vec.push([key, value]))
                .or_insert(SmallVec::<SV<_>>::from_slice(&[[key, value]]));
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

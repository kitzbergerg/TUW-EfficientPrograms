#![feature(portable_simd)]
#![feature(iter_array_chunks)]
use hash::MyHashMap;
use memmap2::Mmap;
use simd_csv_reader::parse_csv;
use smallvec::SmallVec;
use smallvec::smallvec;
use std::alloc::GlobalAlloc;
use std::alloc::Layout;
use std::collections::HashMap;
use std::fs::File;
use std::hash::BuildHasher;
use std::io::BufWriter;
use std::io::Write;
use std::io::stdout;

mod hash;
mod simd_csv_reader;

struct NoDealloc;

unsafe impl GlobalAlloc for NoDealloc {
    #[inline]
    unsafe fn alloc(&self, layout: Layout) -> *mut u8 {
        unsafe { std::alloc::System.alloc(layout) }
    }

    #[inline]
    unsafe fn dealloc(&self, _ptr: *mut u8, _layout: Layout) {
        // No-op: We just run once, no need to dealloc
    }
}

#[global_allocator]
static GLOBAL: NoDealloc = NoDealloc;

type Field<'a> = &'a [u8];
type SV2<T> = SmallVec<[T; 2]>;

fn open_reader(file: &str) -> Mmap {
    let file = File::open(file).unwrap();
    unsafe { Mmap::map(&file).unwrap() }
}

fn stream_data<'a>(
    data: &'a [u8],
    fields: &mut Vec<Field<'a>>,
) -> impl Iterator<Item = [Field<'a>; 2]> {
    fields.clear();
    parse_csv(data, fields);
    fields.iter_mut().map(|x| *x).array_chunks()
}

fn write_output<'a, W: Write, S: BuildHasher>(
    writer: &mut BufWriter<W>,
    abc: &HashMap<Field<'a>, [SV2<Field<'a>>; 3], S>,
    d_map: &HashMap<Field<'a>, SV2<Field<'a>>, S>,
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

    let mut fields = Vec::with_capacity(10_000_000);
    let mut abc_map = MyHashMap::with_capacity_and_hasher(5_000_000, Default::default());
    let mut d_map = MyHashMap::with_capacity_and_hasher(5_000_000, Default::default());

    let reader1 = open_reader(&args[1]);
    let reader2 = open_reader(&args[2]);
    let reader3 = open_reader(&args[3]);
    let reader4 = open_reader(&args[4]);

    stream_data(&reader2, &mut fields).for_each(|[key, value]| {
        abc_map
            .entry(key)
            .and_modify(|vec: &mut [SV2<_>; 3]| vec[1].push(value))
            .or_insert([SmallVec::new(), smallvec![value], SmallVec::new()]);
    });
    stream_data(&reader1, &mut fields).for_each(|[key, value]| {
        if let Some(vec) = abc_map.get_mut(&key) {
            vec[0].push(value);
        }
    });
    stream_data(&reader3, &mut fields).for_each(|[key, value]| {
        if let Some(vec) = abc_map.get_mut(&key) {
            vec[2].push(value);
        }
    });
    stream_data(&reader4, &mut fields).for_each(|[key, value]| {
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

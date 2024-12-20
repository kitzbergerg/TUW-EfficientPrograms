// build.rs
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() {
    let dest_path = Path::new("src/codegen.rs");
    let mut f = File::create(&dest_path).unwrap();

    writeln!(
        f,
        "const SINGLE_MULTIPLIERS: [u64; 2] = [{}, {}];",
        37u64,
        37u64.pow(2)
    )
    .unwrap();

    // Single byte table for remaining bytes
    writeln!(
        f,
        "const CHAR_TO_INDEX: [u8; 256] = {{
            let mut indices = [0u8; 256];
            {}
            indices
        }};",
        ('A'..='Z')
            .chain('0'..='9')
            .enumerate()
            .map(|(i, c)| format!("indices[b'{}' as usize] = {};", c, i))
            .collect::<Vec<_>>()
            .join("\n")
    )
    .unwrap();

    // Generate position multipliers for 2-byte chunks
    writeln!(
        f,
        "const CHUNK_MULTIPLIERS: [u64; 11] = [{}];",
        (0..11)
            .map(|i| format!("{}", (37u64 * 37u64).pow(i as u32)))
            .collect::<Vec<_>>()
            .join(", ")
    )
    .unwrap();

    // Generate lookup tables for 2-byte combinations
    // This gives us 37^2 = 1369 possible combinations
    writeln!(
        f,
        "const TWO_BYTE_VALUES: [u16; 65536] = {{
            let mut values = [0u16; 65536];
            {}
            values
        }};",
        generate_two_byte_table()
    )
    .unwrap();
}

fn generate_two_byte_table() -> String {
    let valid_chars = ('A'..='Z').chain('0'..='9').collect::<Vec<_>>();
    let mut result = Vec::new();

    for b1 in 0u8..=255 {
        for b2 in 0u8..=255 {
            let idx1 = valid_chars.iter().position(|&c| c as u8 == b1);
            let idx2 = valid_chars.iter().position(|&c| c as u8 == b2);

            if let (Some(i1), Some(i2)) = (idx1, idx2) {
                result.push(format!(
                    "values[{}] = {};",
                    b1 as u16 * 256 + b2 as u16,
                    i1 as u16 * 37 + i2 as u16
                ));
            }
        }
    }

    result.join("\n")
}

#![deny(warnings)]

use arrayvec::ArrayVec;
use std::env::var_os;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;
use utf8_chars::BufReadCharsExt;

type Arr<T> = ArrayVec<T, 128>;

fn main() {
    let out_dir = var_os("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir);
    let rs = out_dir.join("generated.rs");
    let mut rs = File::create(rs).unwrap();
    for cp in [
        "CP437", "CP737",
        "CP850", "CP852",
        "CP855",
        "CP857",
        "CP860", "CP861", "CP862", "CP863", "CP864", "CP865", "CP866",
    ] {
        let mod_name = cp.to_ascii_lowercase();
        println!("cargo:rerun-if-changed={}", cp);
        rs.write(b"\n").unwrap();
        rs.write(format!("pub const {}: CP = CP {{\n", cp).as_bytes()).unwrap();
        rs.write(format!("    to_uni: {}::to_uni,\n", mod_name).as_bytes()).unwrap();
        rs.write(format!("    from_uni: {}::from_uni,\n", mod_name).as_bytes()).unwrap();
        rs.write(b"};\n").unwrap();
        rs.write(b"\n").unwrap();
        rs.write(format!("mod {} {{\n", mod_name).as_bytes()).unwrap();
        rs.write(b"    use core::hint::unreachable_unchecked;\n").unwrap();
        rs.write(b"    use core::num::NonZeroU8;\n").unwrap();
        rs.write(b"\n").unwrap();
        rs.write(b"    pub unsafe fn to_uni(c: u8) -> u32 {\n").unwrap();
        rs.write(b"        match c {\n").unwrap();
        let table = File::open(cp).expect("codepage table not found");
        let mut chars = BufReader::new(table)
            .chars()
            .map(|x| x.expect("invalid char in codepage table"))
            .enumerate()
            .map(|(i, u)| (u as u32, 128 + i))
            .collect::<Arr<_>>()
        ;
        assert_eq!(chars.len(), 128, "invalid codepage table");
        for (u, a) in chars.iter().copied().filter(|&(u, _)| u != '?' as u32) {
            rs.write(format!("            {} => {},\n", a - 128, u).as_bytes()).unwrap();
        }
        rs.write(b"            _ => unreachable_unchecked()\n").unwrap();
        rs.write(b"        }\n").unwrap();
        rs.write(b"    }\n").unwrap();
        rs.write(b"\n").unwrap();
        rs.write(b"    pub unsafe fn from_uni(c: u32) -> Option<NonZeroU8> {\n").unwrap();
        rs.write(b"        match c {\n").unwrap();
        chars.sort_by_key(|&(u, _)| u);
        for (u, a) in chars.into_iter().filter(|&(u, _)| u != '?' as u32) {
            rs.write(format!("            {} => Some(NonZeroU8::new_unchecked({})),\n", u, a).as_bytes()).unwrap();
        }
        rs.write(b"            _ => None\n").unwrap();
        rs.write(b"        }\n").unwrap();
        rs.write(b"    }\n").unwrap();
        rs.write(b"}\n").unwrap();
    }
}

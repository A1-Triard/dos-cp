#![deny(warnings)]

use arrayvec::ArrayVec;
use itertools::Itertools;
use std::env::var_os;
use std::fs::File;
use std::io::{BufReader, Write};
use std::path::Path;
use utf8_chars::BufReadCharsExt;

type Arr<T> = ArrayVec<T, 128>;

fn main() {
    let out_dir = var_os("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir);
    for cp in ["CP437", "CP857", "CP866"] {
        println!("cargo:rerun-if-changed={}", cp);
        let table = File::open(cp).expect("codepage table not found");
        let chars = BufReader::new(table)
            .chars()
            .map(|x| x.expect("invalid char in codepage table"))
            .enumerate()
            .map(|(i, u)| (u as u32, 128 + i))
            .sorted_by_key(|&(u, _)| u)
            .collect::<Arr<_>>()
        ;
        assert_eq!(chars.len(), 128, "invalid codepage table");
        let out = out_dir.join(cp).with_extension("rs");
        let mut out = File::create(out).unwrap();
        out.write(b"match c {\n").unwrap();
        for (u, a) in chars.into_iter().filter(|&(u, _)| u != '?' as u32) {
            out.write(format!("    {} => {},\n", u, a).as_bytes()).unwrap();
        }
        out.write(b"}\n").unwrap();
    }
}

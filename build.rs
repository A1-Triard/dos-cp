#![deny(warnings)]

use arrayvec::ArrayVec;
use itertools::Itertools;
use std::env::var_os;
use std::fs::File;
use std::io::{BufReader, Write};
use std::ops::RangeInclusive;
use std::path::Path;
use utf8_chars::BufReadCharsExt;

type Arr<T> = ArrayVec<T, 128>;

struct Ranges<I: Iterator<Item=u32>> {
    input: Option<I>,
    last: Option<u32>,
}

impl<I: Iterator<Item=u32>> Iterator for Ranges<I> {
    type Item = RangeInclusive<u32>;

    fn next(&mut self) -> Option<RangeInclusive<u32>> {
        let mut input = self.input.take()?;
        let (res, input) = if let Some(range_start) = self.last.or_else(|| input.next()) {
            let mut range_end = range_start;
            let input = loop {
                if let Some(item) = input.next() {
                    if item == range_end + 1 {
                        range_end = item;
                    } else {
                        self.last = Some(item);
                        break Some(input);
                    }
                } else {
                    break None;
                }
            };
            (Some(range_start ..= range_end), input)
        } else {
            (None, None)
        };
        self.input = input;
        res
    }
}

fn ranges<I: Iterator<Item=u32>>(input: I) -> Ranges<I> {
    Ranges { input: Some(input), last: None }
}

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
            .collect::<Arr<_>>()
        ;
        assert_eq!(chars.len(), 128, "invalid codepage table");
        let groups = chars.into_iter()
            .filter(|&(u, _)| u != '?' as u32)
            .map(|(u, a)| (u, a as i32 - u as i32))
            .sorted_by_key(|&(_, m)| m)
            .group_by(|&(_, m)| m)
        ;
        let mut ranges = groups
            .into_iter()
            .flat_map(|(m, group)| ranges(group.map(|(u, _)| u).sorted_by_key(|&u| u)).map(move |u| (u, m)))
            .collect::<Arr<_>>()
        ;
        ranges.sort_by_key(|(u, _)| *u.start());
        let out = out_dir.join(cp).with_extension("rs");
        let mut out = File::create(out).unwrap();
        out.write(b"match c {\n").unwrap();
        for (range, map) in ranges {
            if range.end() - range.start() <= 2 {
                for u in range {
                    out.write(format!("    {} => {},\n", u, (u as i32 + map) as u8).as_bytes()).unwrap();
                }
            } else {
                let map = if map < 0 {
                    format!(" - {}", -map)
                } else if map > 0 {
                    format!(" + {}", map)
                } else {
                    String::new()
                };
                out.write(format!("    {} ..= {} => (c as i32{}) as u8,\n",
                    range.start(), range.end(), map
                ).as_bytes()).unwrap();
            }
        }
        out.write(b"}\n").unwrap();
    }
}

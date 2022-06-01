use arrayvec::ArrayVec;
use std::cmp::max;
use std::fs::File;
use std::io::BufReader;
use utf8_chars::BufReadCharsExt;

fn hash(u: u16, add: u16) -> u8 {
    let u = u.wrapping_add(add);
    ((u ^ (u >> 8)) & 0x7F) as u8
}

pub fn find_hash(table: &[char]) -> u16 {
    (0 ..= u16::MAX).filter(|&p| p & 0x0080 != 0 && p & 0x8000 != 0).filter_map(|p| {
        let mut table = table.iter().copied().map(|c| hash(c as u32 as u16, p)).collect::<ArrayVec<_, 128>>();
        table.sort();
        let dups = table.into_iter().fold((false, 0usize, 0usize, None), |(zero, mut total, mut cur, prev_x), x| {
            if Some(x) == prev_x {
                cur += 1;
            } else {
                total = max(total, cur);
                cur = 1;
            }
            (if x == 0x7F { true } else { zero }, total, cur, Some(x))
        });
        if dups.0 { return None; }
        let dups = max(dups.1, dups.2);
        if dups > 2 { None } else { Some(p) }
    }).next().unwrap()
}

fn main() {
    /*
    for cp in [
        "CP720"
    ] {
        let table = File::open(cp).expect("codepage table not found");
        let chars = BufReader::new(table)
            .chars()
            .map(|x| x.expect("invalid char in codepage table"))
            .collect::<ArrayVec<_, 128>>()
        ;
        assert!(chars.len() == 128);
        for c in chars {
            println!("\\u{{{:04X}}}", c as u32);
        }
    }
    */
    for cp in [
        "CP437", "CP720", "CP737", "CP850", "CP852", "CP855", "CP857", "CP858", "CP860",
        "CP861", "CP862", "CP863", "CP864", "CP865", "CP866", "CP869", "CP874",
        "CP912", "CP915",
    ] {
        let table = File::open(cp).expect("codepage table not found");
        let chars = BufReader::new(table)
            .chars()
            .map(|x| x.expect("invalid char in codepage table"))
            .filter(|&u| u != '?')
            .collect::<ArrayVec<_, 128>>()
        ;
        let hash = find_hash(&chars[..]);

        println!("{}: {:04X}", cp, hash);
    }
}

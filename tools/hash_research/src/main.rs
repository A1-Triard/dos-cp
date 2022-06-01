use arrayvec::ArrayVec;
use std::fs::File;
use std::io::BufReader;
use utf8_chars::BufReadCharsExt;

fn hash(u: u16, add: u16, add_shift: u8, shift: u8) -> u8 {
    let u = u.wrapping_add(add);
    (((u ^ (u << add_shift)) >> shift) & 0x7F) as u8
}

pub fn find_hash(table: &[char]) -> (u16, u8, u8) {
    let adds = (0 ..= u16::MAX).into_iter();
    let shifted_adds = adds.flat_map(|add| (0 .. 16).into_iter().map(move |add_shift| (add, add_shift)));
    let hashes = shifted_adds.flat_map(|(add, add_shift)| (0 .. 16).into_iter().map(move |shift| (add, add_shift, shift)));
    hashes.filter_map(|p| {
        let mut table = table.iter().copied().map(|c| hash(c as u32 as u16, p.0, p.1, p.2)).collect::<ArrayVec<_, 128>>();
        table.sort();
        let dups = table.into_iter().fold((0usize, 0usize, 0usize, None), |(mut count, mut total, mut cur, prev_x), x| {
            if Some(x) == prev_x {
                cur += 1;
            } else {
                if cur > total {
                    total = cur;
                    count = 1;
                } else if cur == total {
                    count += 1;
                }
                cur = 1;
            }
            (count, total, cur, Some(x))
        });
        let (dups, count) = if dups.2 > dups.1 {
            (dups.2, 1)
        } else {
            (dups.1, dups.0)
        };
        if dups > 2 { return None; }
        Some((p, dups, count))
    }).next().unwrap().0
}

fn main() {
    /*
    for cp in [
        "CP874"
    ] {
        let table = File::open(cp).expect("codepage table not found");
        let chars = BufReader::new(table)
            .chars()
            .map(|x| x.expect("invalid char in codepage table"))
            .collect::<ArrayVec<_, 128>>()
        ;
        for c in chars {
            println!("\\u{{{:04X}}}", c as u32);
        }
    }
    */
    for cp in [
        "CP437", "CP737", "CP850", "CP852", "CP855", "CP857", "CP858", "CP860",
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

        println!("{}: add {:04X} shift {} and then shift {}", cp, hash.0, hash.1, hash.2);
    }
}

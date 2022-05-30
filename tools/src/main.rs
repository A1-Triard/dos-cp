use arrayvec::ArrayVec;
use std::fs::File;
use std::io::BufReader;
use utf8_chars::BufReadCharsExt;

pub fn find_mask(table: &[char]) -> u32 {
    let mut bits: ArrayVec<u8, 32> = ArrayVec::new();
    for bit in 0 .. 32 {
        let mask = 1u32 << bit;
        let value = (table[0] as u32) & mask;
        if table.iter().any(|&x| (x as u32) & mask != value) {
            bits.push(bit);
        }
    }
    assert!(bits.len() <= 16);
    let bit_index_range = 0 .. bits.len() as u8;
    let mut masks = bit_index_range.clone().into_iter()
        .flat_map(|x| bit_index_range.clone().into_iter().map(move |b| (x, b)))
        .flat_map(|x| bit_index_range.clone().into_iter().map(move |b| (x, b)))
        .flat_map(|x| bit_index_range.clone().into_iter().map(move |b| (x, b)))
        .flat_map(|x| bit_index_range.clone().into_iter().map(move |b| (x, b)))
        .flat_map(|x| bit_index_range.clone().into_iter().map(move |b| (x, b)))
        .flat_map(|x| bit_index_range.clone().into_iter().map(move |b| (x, b)))
        .map(|((((((a, b), c), d), e), f), g)| [a, b, c, d, e, f, g])
        .map(|mut x| { x.sort(); x })
        .filter(|[a, b, c, d, e, f, g]|
            a != b && a != c && a != d && a != e && a != f && a != g &&
            b != c && b != d && b != e && b != f && b != g &&
            c != d && c != e && c != f && c != g &&
            d != e && d != f && d != g &&
            e != f && e != g &&
            f != g
        )
        .map(|[a, b, c, d, e, f, g]|
            (1 << a) | (1 << b) | (1 << c) | (1 << d) | (1 << e) | (1 << f) | (1 << g)
        )
        .collect::<Vec<_>>()
    ;
    masks.sort();
    masks.dedup();
    let mut masks = masks.into_iter().map(|mask| {
        let mut table = table.iter().copied().map(|c| (c as u32) & mask).collect::<Vec<_>>();
        table.sort();
        let dups = table.into_iter().fold((0usize, 0usize, None), |(mut total, mut cur, prev_x), x| {
            if Some(x) == prev_x {
                cur += 1;
            } else {
                total = total.max(cur);
                cur = 0;
            }
            (total, cur, Some(x))
        });
        let dups = dups.0.max(dups.1);
        (mask, dups)
    }).collect::<Vec<_>>();
    masks.sort_by_key(|&(_, dups)| dups);
    let dups = masks[0].1;
    assert!(dups == 2 || dups == 1);
    let trim = masks.iter().take_while(|x| x.1 <= 2).count();
    masks.truncate(trim);
    for (ref mask, n) in &mut masks {
        let b00 = mask & 0x00000001; let b01 = mask & 0x00000002; let b02 = mask & 0x00000004; let b03 = mask & 0x00000008;
        let b04 = mask & 0x00000010; let b05 = mask & 0x00000020; let b06 = mask & 0x00000040; let b07 = mask & 0x00000080;
        let b08 = mask & 0x00000100; let b09 = mask & 0x00000200; let b10 = mask & 0x00000400; let b11 = mask & 0x00000800;
        let b12 = mask & 0x00001000; let b13 = mask & 0x00002000; let b14 = mask & 0x00004000; let b15 = mask & 0x00008000;
        let b16 = mask & 0x00010000; let b17 = mask & 0x00020000; let b18 = mask & 0x00040000; let b19 = mask & 0x00080000;
        let b20 = mask & 0x00100000; let b21 = mask & 0x00200000; let b22 = mask & 0x00400000; let b23 = mask & 0x00800000;
        let b24 = mask & 0x01000000; let b25 = mask & 0x02000000; let b26 = mask & 0x04000000; let b27 = mask & 0x08000000;
        let b28 = mask & 0x10000000; let b29 = mask & 0x20000000; let b30 = mask & 0x40000000; let b31 = mask & 0x80000000;
        let chunks = [
            b00, b01, b02, b03, b04, b05, b06, b07, b08, b09, b10, b11, b12, b13, b14, b15,
            b16, b17, b18, b19, b20, b21, b22, b23, b24, b25, b26, b27, b28, b29, b30, b31,
        ].iter().copied().fold((0, 0), |(mut chunks, prev_x), x| {
            if (prev_x == 0) != (x == 0) && prev_x == 0 {
                chunks += 1;
            }
            (chunks, x)
        }).0;
        *n = chunks;
    }
    masks.sort_by_key(|&(_, chunks)| chunks);
    masks[0].0
}

fn main() {
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
    /*/
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
        let mask = find_mask(&chars[..]);

        println!("{}: {:04X}", cp, mask);
    }
    */
}

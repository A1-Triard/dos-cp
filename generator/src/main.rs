use arrayvec::ArrayVec;

pub fn generate(table: &[char]) -> [u8; 1024] {
    assert_eq!(table.len(), 128);
    let mut bits: ArrayVec<u8, 32> = ArrayVec::new();
    for bit in 0 .. 32 {
        let mask = 1u32 << bit;
        let value = (table[0] as u32) & mask;
        if table.iter().any(|&x| (x as u32) & mask != value) {
            bits.push(bit);
        }
    }
    assert!(bits.len() <= 16);
    let mut bits_corr: ArrayVec<(u8, f32), 16> = ArrayVec::new();
    bits_corr.extend(bits.iter().copied().map(|x| (x, 0.0)));
    while bits_corr.len() > 7 {
        bits_corr = bits_corr.iter().copied().map(|(bit, _)| {
            let correlation = bits_corr.iter().copied().filter(|&(x, _)| x != bit).map(|(other_bit, _)| {
                let bit_average: f32 = table.iter().copied()
                    .map(|c| if (c as u32) & (1u32 << bit) != 0 { 1.0 } else { 0.0 })
                    .sum::<f32>() / table.len() as f32
                ;
                let other_bit_average: f32 = table.iter().copied()
                    .map(|c| if (c as u32) & (1u32 << other_bit) != 0 { 1.0 } else { 0.0 })
                    .sum::<f32>() / table.len() as f32
                ;
                let (n_1_sigma_2, n_1_other_sigma_2, n_1_covariance) = table.iter().copied().map(|c| {
                    let bit_delta = (if (c as u32) & (1u32 << bit) != 0 { 1.0 } else { 0.0 }) - bit_average;
                    let other_bit_delta = (if (c as u32) & (1u32 << other_bit) != 0 { 1.0 } else { 0.0 }) - other_bit_average;
                    (
                        bit_delta * bit_delta,
                        other_bit_delta * other_bit_delta,
                        bit_delta * other_bit_delta
                    )
                }).fold((0.0, 0.0, 0.0), |(s, u, v), (a, b, c)| (s + a, u + b, v + c));
                let sigma = (n_1_sigma_2 / (table.len() as f32 - 1.0)).sqrt();
                let other_sigma = (n_1_other_sigma_2 / (table.len() as f32 - 1.0)).sqrt();
                let covariance = n_1_covariance / (table.len() as f32 - 1.0);
                let correlation = covariance / (sigma * other_sigma);
                correlation.abs()
            }).fold(0.0f32, |m, x| m.max(x));
            (bit, correlation)
        }).collect::<ArrayVec<_, 16>>();
        bits_corr.sort_by(|(_, a), (_, b)| a.partial_cmp(b).unwrap());
        println!("{:?}", bits_corr);
        bits_corr.pop();
    }
    let mask = bits_corr.iter().copied().map(|(x, _)| 1u32 << x).fold(0u32, |m, x| m | x);
    let mut table_hash = table.iter().copied().map(|c| (c as u32) & mask).collect::<ArrayVec<_, 128>>();
    table_hash.sort();
    println!("{:?}", table_hash);
    [0; 1024]
}

const CP857: &str = "\
    ÇüéâäàåçêëèïîıÄÅÉæÆôöòûùİÖÜø£ØŞşáíóúñÑĞğ¿®¬\
    ½¼¡«»░▒▓│┤ÁÂÀ©╣║╗╝¢¥┐└┴┬├─┼ãÃ╚╔╩╦╠═╬¤ºªÊËÈ?\
    ÍÎÏ┘┌█▄¦Ì▀ÓßÔÒõÕµ?×ÚÛÙìÿ¯´­±?¾¶§÷¸°¨·¹³²■ \
";

fn main() {
    let table = CP857.chars().collect::<ArrayVec<_, 128>>();
    generate(&table);
}

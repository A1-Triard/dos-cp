#![deny(warnings)]

use dos_cp::{CodePage, hash};
use std::mem::{MaybeUninit, transmute};

pub const KNOWN_CODE_PAGES: &[u16] = &[
    437, 720, 737, 850, 852, 855, 857, 858, 860,
    861, 862, 863, 864, 865, 866, 869, 874,
    912, 915,
];

#[derive(Copy, Clone)]
enum Filled { None, Half, Full }

pub trait CodePageGenExt {
    fn generate(code_page: u16) -> Self;
}

impl CodePageGenExt for CodePage {
    fn generate(code_page: u16) -> CodePage {
        let (base_table, hash_param) = base_table_and_hash_param(code_page);
        let mut res: [MaybeUninit<u8>; 512] = unsafe { MaybeUninit::uninit().assume_init() };
        res[510].write(hash_param as u8);
        res[511].write((hash_param >> 8) as u8);
        let base_table = base_table.iter().copied().map(|c| {
            if c == '?' { return 0; }
            let c: u16 = (c as u32).try_into()
                .expect("too big char, bit needs to be preremapped");
            c
        });
        let mut first_part = &mut res[.. 256];
        for byte in base_table.clone().flat_map(|w| [(w >> 8) as u8, w as u8]) {
            first_part[0].write(byte);
            first_part = &mut first_part[1 ..];
        }
        let second_part = &mut res[256 ..];
        let mut filled = [Filled::None; 127];
        for (i, w) in base_table.into_iter().enumerate().filter(|&(_, w)| w != 0) {
            let hash = hash(w, hash_param);
            debug_assert!((hash >> 7) == 0 && hash != 0x7F, "invalid hash");
            let filled = &mut filled[hash as usize];
            match filled {
                Filled::None => {
                    second_part[(2 * hash) as usize].write(i as u8);
                    *filled = Filled::Half;
                },
                Filled::Half => {
                    second_part[(2 * hash + 1) as usize].write(i as u8);
                    *filled = Filled::Full;
                },
                Filled::Full => unreachable!("invalid hash / base table"),
            }
        }
        for (i, filled) in filled.into_iter().enumerate() {
            match filled {
                Filled::None => {
                    second_part[2 * i].write(128);
                    second_part[2 * i + 1].write(128);
                },
                Filled::Half => {
                    second_part[2 * i + 1].write(128);
                },
                Filled::Full => { }
            }
        }
        CodePage(unsafe { transmute(res) })
    }
}

fn base_table_and_hash_param(code_page: u16) -> (&'static [char; 128], u16) {
    match code_page {
        437 => (&CP437, 0x9F8D),
        720 => (&CP720, 0x8EAC),
        737 => (&CP737, 0x9AF8),
        850 => (&CP850, 0x80F1),
        852 => (&CP852, 0xF5F4),
        855 => (&CP855, 0x808F),
        857 => (&CP857, 0x80EE),
        858 => (&CP858, 0x80F1),
        860 => (&CP860, 0x9FCD),
        861 => (&CP861, 0x83AD),
        862 => (&CP862, 0xBB95),
        863 => (&CP863, 0xEAC8),
        864 => (&CP864, 0x80A2),
        865 => (&CP865, 0x9F8D),
        866 => (&CP866, 0x8398),
        869 => (&CP869, 0x8B8F),
        874 => (&CP874, 0x8080),
        912 => (&CP912, 0x8086),
        915 => (&CP915, 0x8080),
        _ => panic!("unknow code page"),
    }
}
 
const CP437: [char; 128] = [
    'Ç', 'ü', 'é', 'â', 'ä', 'à', 'å', 'ç',
    'ê', 'ë', 'è', 'ï', 'î', 'ì', 'Ä', 'Å',
    'É', 'æ', 'Æ', 'ô', 'ö', 'ò', 'û', 'ù',
    'ÿ', 'Ö', 'Ü', '¢', '£', '¥', '₧', 'ƒ',
    'á', 'í', 'ó', 'ú', 'ñ', 'Ñ', 'ª', 'º',
    '¿', '⌐', '¬', '½', '¼', '¡', '«', '»',
    '░', '▒', '▓', '│', '┤', '╡', '╢', '╖',
    '╕', '╣', '║', '╗', '╝', '╜', '╛', '┐',
    '└', '┴', '┬', '├', '─', '┼', '╞', '╟',
    '╚', '╔', '╩', '╦', '╠', '═', '╬', '╧',
    '╨', '╤', '╥', '╙', '╘', '╒', '╓', '╫',
    '╪', '┘', '┌', '█', '▄', '▌', '▐', '▀',
    'α', 'ß', 'Γ', 'π', 'Σ', 'σ', 'µ', 'τ',
    'Φ', 'Θ', 'Ω', 'δ', '∞', 'φ', 'ε', '∩',
    '≡', '±', '≥', '≤', '⌠', '⌡', '÷', '≈',
    '°', '∙', '·', '√', 'ⁿ', '²', '■', ' ',
];

const CP720: [char; 128] = [
    '\u{003F}', '\u{003F}', '\u{00E9}', '\u{00E2}', '\u{003F}', '\u{00E0}', '\u{003F}', '\u{00E7}',
    '\u{00EA}', '\u{00EB}', '\u{00E8}', '\u{00EF}', '\u{00EE}', '\u{003F}', '\u{003F}', '\u{003F}',
    '\u{003F}', '\u{0651}', '\u{0652}', '\u{00F4}', '\u{00A4}', '\u{0640}', '\u{00FB}', '\u{00F9}',
    '\u{0621}', '\u{0622}', '\u{0623}', '\u{0624}', '\u{00A3}', '\u{0625}', '\u{0626}', '\u{0627}',
    '\u{0628}', '\u{0629}', '\u{062A}', '\u{062B}', '\u{062C}', '\u{062D}', '\u{062E}', '\u{062F}',
    '\u{0630}', '\u{0631}', '\u{0632}', '\u{0633}', '\u{0634}', '\u{0635}', '\u{00AB}', '\u{00BB}',
    '\u{2591}', '\u{2592}', '\u{2593}', '\u{2502}', '\u{2524}', '\u{2561}', '\u{2562}', '\u{2556}',
    '\u{2555}', '\u{2563}', '\u{2551}', '\u{2557}', '\u{255D}', '\u{255C}', '\u{255B}', '\u{2510}',
    '\u{2514}', '\u{2534}', '\u{252C}', '\u{251C}', '\u{2500}', '\u{253C}', '\u{255E}', '\u{255F}',
    '\u{255A}', '\u{2554}', '\u{2569}', '\u{2566}', '\u{2560}', '\u{2550}', '\u{256C}', '\u{2567}',
    '\u{2568}', '\u{2564}', '\u{2565}', '\u{2559}', '\u{2558}', '\u{2552}', '\u{2553}', '\u{256B}',
    '\u{256A}', '\u{2518}', '\u{250C}', '\u{2588}', '\u{2584}', '\u{258C}', '\u{2590}', '\u{2580}',
    '\u{0636}', '\u{0637}', '\u{0638}', '\u{0639}', '\u{063A}', '\u{0641}', '\u{00B5}', '\u{0642}',
    '\u{0643}', '\u{0644}', '\u{0645}', '\u{0646}', '\u{0647}', '\u{0648}', '\u{0649}', '\u{064A}',
    '\u{2261}', '\u{064B}', '\u{064C}', '\u{064D}', '\u{064E}', '\u{064F}', '\u{0650}', '\u{2248}',
    '\u{00B0}', '\u{2219}', '\u{00B7}', '\u{221A}', '\u{207F}', '\u{00B2}', '\u{25A0}', '\u{00A0}',
];

const CP737: [char; 128] = [
    'Α', 'Β', 'Γ', 'Δ', 'Ε', 'Ζ', 'Η', 'Θ',
    'Ι', 'Κ', 'Λ', 'Μ', 'Ν', 'Ξ', 'Ο', 'Π',
    'Ρ', 'Σ', 'Τ', 'Υ', 'Φ', 'Χ', 'Ψ', 'Ω',
    'α', 'β', 'γ', 'δ', 'ε', 'ζ', 'η', 'θ',
    'ι', 'κ', 'λ', 'μ', 'ν', 'ξ', 'ο', 'π',
    'ρ', 'σ', 'ς', 'τ', 'υ', 'φ', 'χ', 'ψ',
    '░', '▒', '▓', '│', '┤', '╡', '╢', '╖',
    '╕', '╣', '║', '╗', '╝', '╜', '╛', '┐',
    '└', '┴', '┬', '├', '─', '┼', '╞', '╟',
    '╚', '╔', '╩', '╦', '╠', '═', '╬', '╧',
    '╨', '╤', '╥', '╙', '╘', '╒', '╓', '╫',
    '╪', '┘', '┌', '█', '▄', '▌', '▐', '▀',
    'ω', 'ά', 'έ', 'ή', 'ϊ', 'ί', 'ό', 'ύ',
    'ϋ', 'ώ', 'Ά', 'Έ', 'Ή', 'Ί', 'Ό', 'Ύ',
    'Ώ', '±', '≥', '≤', 'Ϊ', 'Ϋ', '÷', '≈',
    '°', '∙', '·', '√', 'ⁿ', '²', '■', ' ',
];

const CP850: [char; 128] = [
    'Ç', 'ü', 'é', 'â', 'ä', 'à', 'å', 'ç',
    'ê', 'ë', 'è', 'ï', 'î', 'ì', 'Ä', 'Å',
    'É', 'æ', 'Æ', 'ô', 'ö', 'ò', 'û', 'ù',
    'ÿ', 'Ö', 'Ü', 'ø', '£', 'Ø', '×', 'ƒ',
    'á', 'í', 'ó', 'ú', 'ñ', 'Ñ', 'ª', 'º',
    '¿', '®', '¬', '½', '¼', '¡', '«', '»',
    '░', '▒', '▓', '│', '┤', 'Á', 'Â', 'À',
    '©', '╣', '║', '╗', '╝', '¢', '¥', '┐',
    '└', '┴', '┬', '├', '─', '┼', 'ã', 'Ã',
    '╚', '╔', '╩', '╦', '╠', '═', '╬', '¤',
    'ð', 'Ð', 'Ê', 'Ë', 'È', 'ı', 'Í', 'Î',
    'Ï', '┘', '┌', '█', '▄', '¦', 'Ì', '▀',
    'Ó', 'ß', 'Ô', 'Ò', 'õ', 'Õ', 'µ', 'þ',
    'Þ', 'Ú', 'Û', 'Ù', 'ý', 'Ý', '¯', '´',
    '­', '±', '‗', '¾', '¶', '§', '÷', '¸',
    '°', '¨', '·', '¹', '³', '²', '■', ' ',
];

const CP852: [char; 128] = [
    'Ç', 'ü', 'é', 'â', 'ä', 'ů', 'ć', 'ç',
    'ł', 'ë', 'Ő', 'ő', 'î', 'Ź', 'Ä', 'Ć',
    'É', 'Ĺ', 'ĺ', 'ô', 'ö', 'Ľ', 'ľ', 'Ś',
    'ś', 'Ö', 'Ü', 'Ť', 'ť', 'Ł', '×', 'č',
    'á', 'í', 'ó', 'ú', 'Ą', 'ą', 'Ž', 'ž',
    'Ę', 'ę', '¬', 'ź', 'Č', 'ş', '«', '»',
    '░', '▒', '▓', '│', '┤', 'Á', 'Â', 'Ě',
    'Ş', '╣', '║', '╗', '╝', 'Ż', 'ż', '┐',
    '└', '┴', '┬', '├', '─', '┼', 'Ă', 'ă',
    '╚', '╔', '╩', '╦', '╠', '═', '╬', '¤',
    'đ', 'Đ', 'Ď', 'Ë', 'ď', 'Ň', 'Í', 'Î',
    'ě', '┘', '┌', '█', '▄', 'Ţ', 'Ů', '▀',
    'Ó', 'ß', 'Ô', 'Ń', 'ń', 'ň', 'Š', 'š',
    'Ŕ', 'Ú', 'ŕ', 'Ű', 'ý', 'Ý', 'ţ', '´',
    '­', '˝', '˛', 'ˇ', '˘', '§', '÷', '¸',
    '°', '¨', '˙', 'ű', 'Ř', 'ř', '■', ' ',
];

const CP855: [char; 128] = [
    'ђ', 'Ђ', 'ѓ', 'Ѓ', 'ё', 'Ё', 'є', 'Є',
    'ѕ', 'Ѕ', 'і', 'І', 'ї', 'Ї', 'ј', 'Ј',
    'љ', 'Љ', 'њ', 'Њ', 'ћ', 'Ћ', 'ќ', 'Ќ',
    'ў', 'Ў', 'џ', 'Џ', 'ю', 'Ю', 'ъ', 'Ъ',
    'а', 'А', 'б', 'Б', 'ц', 'Ц', 'д', 'Д',
    'е', 'Е', 'ф', 'Ф', 'г', 'Г', '«', '»',
    '░', '▒', '▓', '│', '┤', 'х', 'Х', 'и',
    'И', '╣', '║', '╗', '╝', 'й', 'Й', '┐',
    '└', '┴', '┬', '├', '─', '┼', 'к', 'К',
    '╚', '╔', '╩', '╦', '╠', '═', '╬', '¤',
    'л', 'Л', 'м', 'М', 'н', 'Н', 'о', 'О',
    'п', '┘', '┌', '█', '▄', 'П', 'я', '▀',
    'Я', 'р', 'Р', 'с', 'С', 'т', 'Т', 'у',
    'У', 'ж', 'Ж', 'в', 'В', 'ь', 'Ь', '№',
    '­', 'ы', 'Ы', 'з', 'З', 'ш', 'Ш', 'э',
    'Э', 'щ', 'Щ', 'ч', 'Ч', '§', '■', ' ',
];

const CP857: [char; 128] = [
    'Ç', 'ü', 'é', 'â', 'ä', 'à', 'å', 'ç',
    'ê', 'ë', 'è', 'ï', 'î', 'ı', 'Ä', 'Å',
    'É', 'æ', 'Æ', 'ô', 'ö', 'ò', 'û', 'ù',
    'İ', 'Ö', 'Ü', 'ø', '£', 'Ø', 'Ş', 'ş',
    'á', 'í', 'ó', 'ú', 'ñ', 'Ñ', 'Ğ', 'ğ',
    '¿', '®', '¬', '½', '¼', '¡', '«', '»',
    '░', '▒', '▓', '│', '┤', 'Á', 'Â', 'À',
    '©', '╣', '║', '╗', '╝', '¢', '¥', '┐',
    '└', '┴', '┬', '├', '─', '┼', 'ã', 'Ã',
    '╚', '╔', '╩', '╦', '╠', '═', '╬', '¤',
    'º', 'ª', 'Ê', 'Ë', 'È', '?', 'Í', 'Î',
    'Ï', '┘', '┌', '█', '▄', '¦', 'Ì', '▀',
    'Ó', 'ß', 'Ô', 'Ò', 'õ', 'Õ', 'µ', '?',
    '×', 'Ú', 'Û', 'Ù', 'ì', 'ÿ', '¯', '´',
    '­', '±', '?', '¾', '¶', '§', '÷', '¸',
    '°', '¨', '·', '¹', '³', '²', '■', ' ',
];

const CP858: [char; 128] = [
    'Ç', 'ü', 'é', 'â', 'ä', 'à', 'å', 'ç',
    'ê', 'ë', 'è', 'ï', 'î', 'ì', 'Ä', 'Å',
    'É', 'æ', 'Æ', 'ô', 'ö', 'ò', 'û', 'ù',
    'ÿ', 'Ö', 'Ü', 'ø', '£', 'Ø', '×', 'ƒ',
    'á', 'í', 'ó', 'ú', 'ñ', 'Ñ', 'ª', 'º',
    '¿', '®', '¬', '½', '¼', '¡', '«', '»',
    '░', '▒', '▓', '│', '┤', 'Á', 'Â', 'À',
    '©', '╣', '║', '╗', '╝', '¢', '¥', '┐',
    '└', '┴', '┬', '├', '─', '┼', 'ã', 'Ã',
    '╚', '╔', '╩', '╦', '╠', '═', '╬', '¤',
    'ð', 'Ð', 'Ê', 'Ë', 'È', '€', 'Í', 'Î',
    'Ï', '┘', '┌', '█', '▄', '¦', 'Ì', '▀',
    'Ó', 'ß', 'Ô', 'Ò', 'õ', 'Õ', 'µ', 'þ',
    'Þ', 'Ú', 'Û', 'Ù', 'ý', 'Ý', '¯', '´',
    '­', '±', '‗', '¾', '¶', '§', '÷', '¸',
    '°', '¨', '·', '¹', '³', '²', '■', ' ',
];

const CP860: [char; 128] = [
    'Ç', 'ü', 'é', 'â', 'ã', 'à', 'Á', 'ç',
    'ê', 'Ê', 'è', 'Í', 'Ô', 'ì', 'Ã', 'Â',
    'É', 'À', 'È', 'ô', 'õ', 'ò', 'Ú', 'ù',
    'Ì', 'Õ', 'Ü', '¢', '£', 'Ù', '₧', 'Ó',
    'á', 'í', 'ó', 'ú', 'ñ', 'Ñ', 'ª', 'º',
    '¿', 'Ò', '¬', '½', '¼', '¡', '«', '»',
    '░', '▒', '▓', '│', '┤', '╡', '╢', '╖',
    '╕', '╣', '║', '╗', '╝', '╜', '╛', '┐',
    '└', '┴', '┬', '├', '─', '┼', '╞', '╟',
    '╚', '╔', '╩', '╦', '╠', '═', '╬', '╧',
    '╨', '╤', '╥', '╙', '╘', '╒', '╓', '╫',
    '╪', '┘', '┌', '█', '▄', '▌', '▐', '▀',
    'α', 'ß', 'Γ', 'π', 'Σ', 'σ', 'µ', 'τ',
    'Φ', 'Θ', 'Ω', 'δ', '∞', 'φ', 'ε', '∩',
    '≡', '±', '≥', '≤', '⌠', '⌡', '÷', '≈',
    '°', '∙', '·', '√', 'ⁿ', '²', '■', ' ',
];

const CP861: [char; 128] = [
    'Ç', 'ü', 'é', 'â', 'ä', 'à', 'å', 'ç',
    'ê', 'ë', 'è', 'Ð', 'ð', 'Þ', 'Ä', 'Å',
    'É', 'æ', 'Æ', 'ô', 'ö', 'þ', 'û', 'Ý',
    'ý', 'Ö', 'Ü', 'ø', '£', 'Ø', '₧', 'ƒ',
    'á', 'í', 'ó', 'ú', 'Á', 'Í', 'Ó', 'Ú',
    '¿', '⌐', '¬', '½', '¼', '¡', '«', '»',
    '░', '▒', '▓', '│', '┤', '╡', '╢', '╖',
    '╕', '╣', '║', '╗', '╝', '╜', '╛', '┐',
    '└', '┴', '┬', '├', '─', '┼', '╞', '╟',
    '╚', '╔', '╩', '╦', '╠', '═', '╬', '╧',
    '╨', '╤', '╥', '╙', '╘', '╒', '╓', '╫',
    '╪', '┘', '┌', '█', '▄', '▌', '▐', '▀',
    'α', 'ß', 'Γ', 'π', 'Σ', 'σ', 'µ', 'τ',
    'Φ', 'Θ', 'Ω', 'δ', '∞', 'φ', 'ε', '∩',
    '≡', '±', '≥', '≤', '⌠', '⌡', '÷', '≈',
    '°', '∙', '·', '√', 'ⁿ', '²', '■', ' ',
];

const CP862: [char; 128] = [
    'א', 'ב', 'ג', 'ד', 'ה', 'ו', 'ז', 'ח',
    'ט', 'י', 'ך', 'כ', 'ל', 'ם', 'מ', 'ן',
    'נ', 'ס', 'ע', 'ף', 'פ', 'ץ', 'צ', 'ק',
    'ר', 'ש', 'ת', '¢', '£', '¥', '₧', 'ƒ',
    'á', 'í', 'ó', 'ú', 'ñ', 'Ñ', 'ª', 'º',
    '¿', '⌐', '¬', '½', '¼', '¡', '«', '»',
    '░', '▒', '▓', '│', '┤', '╡', '╢', '╖',
    '╕', '╣', '║', '╗', '╝', '╜', '╛', '┐',
    '└', '┴', '┬', '├', '─', '┼', '╞', '╟',
    '╚', '╔', '╩', '╦', '╠', '═', '╬', '╧',
    '╨', '╤', '╥', '╙', '╘', '╒', '╓', '╫',
    '╪', '┘', '┌', '█', '▄', '▌', '▐', '▀',
    'α', 'ß', 'Γ', 'π', 'Σ', 'σ', 'µ', 'τ',
    'Φ', 'Θ', 'Ω', 'δ', '∞', 'φ', 'ε', '∩',
    '≡', '±', '≥', '≤', '⌠', '⌡', '÷', '≈',
    '°', '∙', '·', '√', 'ⁿ', '²', '■', ' ',
];

const CP863: [char; 128] = [
    'Ç', 'ü', 'é', 'â', 'Â', 'à', '¶', 'ç',
    'ê', 'ë', 'è', 'ï', 'î', '‗', 'À', '§',
    'É', 'È', 'Ê', 'ô', 'Ë', 'Ï', 'û', 'ù',
    '¤', 'Ô', 'Ü', '¢', '£', 'Ù', 'Û', 'ƒ',
    '¦', '´', 'ó', 'ú', '¨', '¸', '³', '¯',
    'Î', '⌐', '¬', '½', '¼', '¾', '«', '»',
    '░', '▒', '▓', '│', '┤', '╡', '╢', '╖',
    '╕', '╣', '║', '╗', '╝', '╜', '╛', '┐',
    '└', '┴', '┬', '├', '─', '┼', '╞', '╟',
    '╚', '╔', '╩', '╦', '╠', '═', '╬', '╧',
    '╨', '╤', '╥', '╙', '╘', '╒', '╓', '╫',
    '╪', '┘', '┌', '█', '▄', '▌', '▐', '▀',
    'α', 'ß', 'Γ', 'π', 'Σ', 'σ', 'µ', 'τ',
    'Φ', 'Θ', 'Ω', 'δ', '∞', 'φ', 'ε', '∩',
    '≡', '±', '≥', '≤', '⌠', '⌡', '÷', '≈',
    '°', '∙', '·', '√', 'ⁿ', '²', '■', ' ',
];

const CP864: [char; 128] = [
    '\u{00B0}', '\u{00B7}', '\u{2219}', '\u{221A}', '\u{2592}', '\u{2500}', '\u{2502}', '\u{253C}',
    '\u{2524}', '\u{252C}', '\u{251C}', '\u{2534}', '\u{2510}', '\u{250C}', '\u{2514}', '\u{2518}',
    '\u{03B2}', '\u{221E}', '\u{03C6}', '\u{00B1}', '\u{00BD}', '\u{00BC}', '\u{2248}', '\u{00AB}',
    '\u{00BB}', '\u{FEF7}', '\u{FEF8}', '\u{003F}', '\u{003F}', '\u{FEFB}', '\u{FEFC}', '\u{003F}',
    '\u{00A0}', '\u{00AD}', '\u{FE82}', '\u{00A3}', '\u{00A4}', '\u{FE84}', '\u{003F}', '\u{003F}',
    '\u{FE8E}', '\u{FE8F}', '\u{FE95}', '\u{FE99}', '\u{060C}', '\u{FE9D}', '\u{FEA1}', '\u{FEA5}',
    '\u{0660}', '\u{0661}', '\u{0662}', '\u{0663}', '\u{0664}', '\u{0665}', '\u{0666}', '\u{0667}',
    '\u{0668}', '\u{0669}', '\u{FED1}', '\u{061B}', '\u{FEB1}', '\u{FEB5}', '\u{FEB9}', '\u{061F}',
    '\u{00A2}', '\u{FE80}', '\u{FE81}', '\u{FE83}', '\u{FE85}', '\u{FECA}', '\u{FE8B}', '\u{FE8D}',
    '\u{FE91}', '\u{FE93}', '\u{FE97}', '\u{FE9B}', '\u{FE9F}', '\u{FEA3}', '\u{FEA7}', '\u{FEA9}',
    '\u{FEAB}', '\u{FEAD}', '\u{FEAF}', '\u{FEB3}', '\u{FEB7}', '\u{FEBB}', '\u{FEBF}', '\u{FEC1}',
    '\u{FEC5}', '\u{FECB}', '\u{FECF}', '\u{00A6}', '\u{00AC}', '\u{00F7}', '\u{00D7}', '\u{FEC9}',
    '\u{0640}', '\u{FED3}', '\u{FED7}', '\u{FEDB}', '\u{FEDF}', '\u{FEE3}', '\u{FEE7}', '\u{FEEB}',
    '\u{FEED}', '\u{FEEF}', '\u{FEF3}', '\u{FEBD}', '\u{FECC}', '\u{FECE}', '\u{FECD}', '\u{FEE1}',
    '\u{FE7D}', '\u{0651}', '\u{FEE5}', '\u{FEE9}', '\u{FEEC}', '\u{FEF0}', '\u{FEF2}', '\u{FED0}',
    '\u{FED5}', '\u{FEF5}', '\u{FEF6}', '\u{FEDD}', '\u{FED9}', '\u{FEF1}', '\u{25A0}', '\u{003F}',
];

const CP865: [char; 128] = [
    'Ç', 'ü', 'é', 'â', 'ä', 'à', 'å', 'ç',
    'ê', 'ë', 'è', 'ï', 'î', 'ì', 'Ä', 'Å',
    'É', 'æ', 'Æ', 'ô', 'ö', 'ò', 'û', 'ù',
    'ÿ', 'Ö', 'Ü', 'ø', '£', 'Ø', '₧', 'ƒ',
    'á', 'í', 'ó', 'ú', 'ñ', 'Ñ', 'ª', 'º',
    '¿', '⌐', '¬', '½', '¼', '¡', '«', '¤',
    '░', '▒', '▓', '│', '┤', '╡', '╢', '╖',
    '╕', '╣', '║', '╗', '╝', '╜', '╛', '┐',
    '└', '┴', '┬', '├', '─', '┼', '╞', '╟',
    '╚', '╔', '╩', '╦', '╠', '═', '╬', '╧',
    '╨', '╤', '╥', '╙', '╘', '╒', '╓', '╫',
    '╪', '┘', '┌', '█', '▄', '▌', '▐', '▀',
    'α', 'ß', 'Γ', 'π', 'Σ', 'σ', 'µ', 'τ',
    'Φ', 'Θ', 'Ω', 'δ', '∞', 'φ', 'ε', '∩',
    '≡', '±', '≥', '≤', '⌠', '⌡', '÷', '≈',
    '°', '∙', '·', '√', 'ⁿ', '²', '■', ' ',
];

const CP866: [char; 128] = [
    'А', 'Б', 'В', 'Г', 'Д', 'Е', 'Ж', 'З',
    'И', 'Й', 'К', 'Л', 'М', 'Н', 'О', 'П',
    'Р', 'С', 'Т', 'У', 'Ф', 'Х', 'Ц', 'Ч',
    'Ш', 'Щ', 'Ъ', 'Ы', 'Ь', 'Э', 'Ю', 'Я',
    'а', 'б', 'в', 'г', 'д', 'е', 'ж', 'з',
    'и', 'й', 'к', 'л', 'м', 'н', 'о', 'п',
    '░', '▒', '▓', '│', '┤', '╡', '╢', '╖',
    '╕', '╣', '║', '╗', '╝', '╜', '╛', '┐',
    '└', '┴', '┬', '├', '─', '┼', '╞', '╟',
    '╚', '╔', '╩', '╦', '╠', '═', '╬', '╧',
    '╨', '╤', '╥', '╙', '╘', '╒', '╓', '╫',
    '╪', '┘', '┌', '█', '▄', '▌', '▐', '▀',
    'р', 'с', 'т', 'у', 'ф', 'х', 'ц', 'ч',
    'ш', 'щ', 'ъ', 'ы', 'ь', 'э', 'ю', 'я',
    'Ё', 'ё', 'Є', 'є', 'Ї', 'ї', 'Ў', 'ў',
    '°', '∙', '·', '√', '№', '¤', '■', ' ',
];

const CP869: [char; 128] = [
    '?', '?', '?', '?', '?', '?', 'Ά', '?',
    '·', '¬', '¦', '‘', '’', 'Έ', '―', 'Ή',
    'Ί', 'Ϊ', 'Ό', '?', '?', 'Ύ', 'Ϋ', '©',
    'Ώ', '²', '³', 'ά', '£', 'έ', 'ή', 'ί',
    'ϊ', 'ΐ', 'ό', 'ύ', 'Α', 'Β', 'Γ', 'Δ',
    'Ε', 'Ζ', 'Η', '½', 'Θ', 'Ι', '«', '»',
    '░', '▒', '▓', '│', '┤', 'Κ', 'Λ', 'Μ',
    'Ν', '╣', '║', '╗', '╝', 'Ξ', 'Ο', '┐',
    '└', '┴', '┬', '├', '─', '┼', 'Π', 'Ρ',
    '╚', '╔', '╩', '╦', '╠', '═', '╬', 'Σ',
    'Τ', 'Υ', 'Φ', 'Χ', 'Ψ', 'Ω', 'α', 'β',
    'γ', '┘', '┌', '█', '▄', 'δ', 'ε', '▀',
    'ζ', 'η', 'θ', 'ι', 'κ', 'λ', 'μ', 'ν',
    'ξ', 'ο', 'π', 'ρ', 'σ', 'ς', 'τ', '΄',
    '­', '±', 'υ', 'φ', 'χ', '§', 'ψ', '΅',
    '°', '¨', 'ω', 'ϋ', 'ΰ', 'ώ', '■', ' ',
];

const CP874: [char; 128] = [
    '\u{20AC}', '\u{003F}', '\u{003F}', '\u{003F}', '\u{003F}', '\u{2026}', '\u{003F}', '\u{003F}',
    '\u{003F}', '\u{003F}', '\u{003F}', '\u{003F}', '\u{003F}', '\u{003F}', '\u{003F}', '\u{003F}',
    '\u{003F}', '\u{2018}', '\u{2019}', '\u{201C}', '\u{201D}', '\u{2022}', '\u{2013}', '\u{2014}',
    '\u{003F}', '\u{003F}', '\u{003F}', '\u{003F}', '\u{003F}', '\u{003F}', '\u{003F}', '\u{003F}',
    '\u{00A0}', '\u{0E01}', '\u{0E02}', '\u{0E03}', '\u{0E04}', '\u{0E05}', '\u{0E06}', '\u{0E07}',
    '\u{0E08}', '\u{0E09}', '\u{0E0A}', '\u{0E0B}', '\u{0E0C}', '\u{0E0D}', '\u{0E0E}', '\u{0E0F}',
    '\u{0E10}', '\u{0E11}', '\u{0E12}', '\u{0E13}', '\u{0E14}', '\u{0E15}', '\u{0E16}', '\u{0E17}',
    '\u{0E18}', '\u{0E19}', '\u{0E1A}', '\u{0E1B}', '\u{0E1C}', '\u{0E1D}', '\u{0E1E}', '\u{0E1F}',
    '\u{0E20}', '\u{0E21}', '\u{0E22}', '\u{0E23}', '\u{0E24}', '\u{0E25}', '\u{0E26}', '\u{0E27}',
    '\u{0E28}', '\u{0E29}', '\u{0E2A}', '\u{0E2B}', '\u{0E2C}', '\u{0E2D}', '\u{0E2E}', '\u{0E2F}',
    '\u{0E30}', '\u{0E31}', '\u{0E32}', '\u{0E33}', '\u{0E34}', '\u{0E35}', '\u{0E36}', '\u{0E37}',
    '\u{0E38}', '\u{0E39}', '\u{0E3A}', '\u{003F}', '\u{003F}', '\u{003F}', '\u{003F}', '\u{0E3F}',
    '\u{0E40}', '\u{0E41}', '\u{0E42}', '\u{0E43}', '\u{0E44}', '\u{0E45}', '\u{0E46}', '\u{0E47}',
    '\u{0E48}', '\u{0E49}', '\u{0E4A}', '\u{0E4B}', '\u{0E4C}', '\u{0E4D}', '\u{0E4E}', '\u{0E4F}',
    '\u{0E50}', '\u{0E51}', '\u{0E52}', '\u{0E53}', '\u{0E54}', '\u{0E55}', '\u{0E56}', '\u{0E57}',
    '\u{0E58}', '\u{0E59}', '\u{0E5A}', '\u{0E5B}', '\u{003F}', '\u{003F}', '\u{003F}', '\u{003F}',
];

const CP912: [char; 128] = [
    '', '', '', '', '', '', '', '',
    '', '', '', '', '', '', '', '',
    '', '', '', '', '', '', '', '',
    '', '', '', '', '', '', '', '',
    ' ', 'Ą', '˘', 'Ł', '¤', 'Ľ', 'Ś', '§',
    '¨', 'Š', 'Ş', 'Ť', 'Ź', '­', 'Ž', 'Ż',
    '°', 'ą', '˛', 'ł', '´', 'ľ', 'ś', 'ˇ',
    '¸', 'š', 'ş', 'ť', 'ź', '˝', 'ž', 'ż',
    'Ŕ', 'Á', 'Â', 'Ă', 'Ä', 'Ĺ', 'Ć', 'Ç',
    'Č', 'É', 'Ę', 'Ë', 'Ě', 'Í', 'Î', 'Ď',
    'Đ', 'Ń', 'Ň', 'Ó', 'Ô', 'Ő', 'Ö', '×',
    'Ř', 'Ů', 'Ú', 'Ű', 'Ü', 'Ý', 'Ţ', 'ß',
    'ŕ', 'á', 'â', 'ă', 'ä', 'ĺ', 'ć', 'ç',
    'č', 'é', 'ę', 'ë', 'ě', 'í', 'î', 'ď',
    'đ', 'ń', 'ň', 'ó', 'ô', 'ő', 'ö', '÷',
    'ř', 'ů', 'ú', 'ű', 'ü', 'ý', 'ţ', '˙',
];

const CP915: [char; 128] = [
    '', '', '', '', '', '', '', '',
    '', '', '', '', '', '', '', '',
    '', '', '', '', '', '', '', '',
    '', '', '', '', '', '', '', '',
    ' ', 'Ё', 'Ђ', 'Ѓ', 'Є', 'Ѕ', 'І', 'Ї',
    'Ј', 'Љ', 'Њ', 'Ћ', 'Ќ', '­', 'Ў', 'Џ',
    'А', 'Б', 'В', 'Г', 'Д', 'Е', 'Ж', 'З',
    'И', 'Й', 'К', 'Л', 'М', 'Н', 'О', 'П',
    'Р', 'С', 'Т', 'У', 'Ф', 'Х', 'Ц', 'Ч',
    'Ш', 'Щ', 'Ъ', 'Ы', 'Ь', 'Э', 'Ю', 'Я',
    'а', 'б', 'в', 'г', 'д', 'е', 'ж', 'з',
    'и', 'й', 'к', 'л', 'м', 'н', 'о', 'п',
    'р', 'с', 'т', 'у', 'ф', 'х', 'ц', 'ч',
    'ш', 'щ', 'ъ', 'ы', 'ь', 'э', 'ю', 'я',
    '№', 'ё', 'ђ', 'ѓ', 'є', 'ѕ', 'і', 'ї',
    'ј', 'љ', 'њ', 'ћ', 'ќ', '§', 'ў', 'џ',
];

#[cfg(test)]
mod test {
    use crate::*;
    use quickcheck::{Arbitrary, Gen, TestResult};
    use quickcheck_macros::quickcheck;

    #[derive(Debug, Copy, Clone)]
    struct KnownCodePage(u8);

    impl Arbitrary for KnownCodePage {
        fn arbitrary(g: &mut Gen) -> Self {
            KnownCodePage(unsafe {
                (g.choose(KNOWN_CODE_PAGES).unwrap() as *const u16)
                    .offset_from(KNOWN_CODE_PAGES.as_ptr())
            } as u8)
        }

        fn shrink(&self) -> Box<dyn Iterator<Item=Self>> {
            Box::new((0 .. self.0).into_iter().rev().map(KnownCodePage))
        }
    }

    #[quickcheck]
    fn low_half_transform_is_trivial(code_page: KnownCodePage) -> bool {
        let code_page = CodePage::generate(KNOWN_CODE_PAGES[code_page.0 as usize]);
        (0 .. 128).all(|a| {
            let u = a as char;
            let u_ = char::from_u32(a as u32).unwrap();
            let a_t = code_page.to_char(a).unwrap();
            let u_t = code_page.from_char(u).unwrap();
            u == u_ && u == a_t && a == u_t
        })
    }

    #[quickcheck]
    fn to_char_is_from_char_inverse(c: char, code_page: KnownCodePage) -> TestResult {
        let code_page = CodePage::generate(KNOWN_CODE_PAGES[code_page.0 as usize]);
        if let Some(b) = code_page.from_char(c) {
            TestResult::from_bool(code_page.to_char(b) == Some(c))
        } else {
            TestResult::discard()
        }
    }

    #[quickcheck]
    fn from_char_is_to_char_inverse(c: u8, code_page: KnownCodePage) -> TestResult {
        let code_page = CodePage::generate(KNOWN_CODE_PAGES[code_page.0 as usize]);
        if let Some(u) = code_page.to_char(c) {
            TestResult::from_bool(code_page.from_char(u) == Some(c))
        } else {
            TestResult::discard()
        }
    }
}

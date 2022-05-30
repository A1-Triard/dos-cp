#![deny(warnings)]

#![no_std]

use core::num::NonZeroU32;

#[doc(hidden)]
#[inline]
pub fn hash(w: u16, add: u16, shift: u8, mut mask: u16) -> u8 {
    let mut w = w ^ (w.wrapping_add(add) << shift);
    let mut res = 0;
    for _ in 0 .. 16 {
        res <<= (mask & 0x0001) as u8;
        res |= (w & mask & 0x0001) as u8;
        w >>= 1;
        mask >>= 1;
    }
    res
}

fn bits_count(mask: u16) -> u8 {
    (0 .. 16).map(|b| ((mask >> b) & 0x0001) as u8).sum()
}

const CODE_PAGE_SIZE: usize = 528;

#[derive(Debug, Clone)]
#[repr(C, align(8))]
pub struct CodePage([u8; CODE_PAGE_SIZE]);

impl CodePage {
    pub const unsafe fn new_unchecked(bytes: [u8; CODE_PAGE_SIZE]) -> Self {
        CodePage(bytes)
    }

    pub fn new(bytes: [u8; CODE_PAGE_SIZE]) -> Option<Self> {
        if bytes[0] != b'C' { return None; }
        if bytes[1] != b'D' { return None; }
        if bytes[2] != b'P' { return None; }
        if bytes[3] != b'G' { return None; }
        if bytes[4] != 1 { return None; }
        if bytes[5] != 0 { return None; }
        if bytes[10] >= 16 { return None; }
        if bytes[11] != 0 { return None; }
        let mask = (bytes[12] as u16) | ((bytes[13] as u16) << 8);
        if bits_count(mask) != 7 { return None; }
        Some(unsafe { Self::new_unchecked(bytes) })
    }

    pub fn into_bytes(self) -> [u8; CODE_PAGE_SIZE] {
        self.0
    }

    fn to_upper_half_char(&self, c: u8) -> Option<char> {
        let offset = 16 + 2 * c as usize;
        let hb = self.0[offset];
        let lb = self.0[offset + 1];
        NonZeroU32::new(((hb as u32) << 8) | (lb as u32))
            .map(|x| unsafe { char::from_u32_unchecked(x.get()) })
    }

    pub fn to_char(&self, c: u8) -> Option<char> {
        if c >> 7 == 0 {
            Some(c as char)
        } else {
            self.to_upper_half_char(c & 0x7F)
        }
    }

    pub fn from_char(&self, c: char) -> Option<u8> {
        if (c as u32) >> 7 == 0 {
            Some(c as u32 as u8)
        } else if (c as u32) >> 16 != 0 {
            None
        } else {
            let w = (c as u32) as u16;
            let add = (self.0[8] as u16) | ((self.0[9] as u16) << 8);
            let shift = self.0[10];
            let mask = (self.0[12] as u16) | ((self.0[13] as u16) << 8);
            let offset = 16 + 256 + 2 * hash(w, add, shift, mask) as usize;
            let try_1 = self.0[offset];
            if try_1 >> 7 != 0 { return None; }
            if self.to_upper_half_char(try_1) == Some(c) {
                return Some(0x80 | try_1);
            }
            let try_2 = self.0[offset + 1];
            if try_2 >> 7 != 0 { return None; }
            if self.to_upper_half_char(try_2) == Some(c) {
                return Some(0x80 | try_2);
            }
            None
        }
    }
}

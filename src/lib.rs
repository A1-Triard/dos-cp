#![deny(warnings)]

#![no_std]

use core::mem::align_of;
use core::num::NonZeroU32;

#[doc(hidden)]
#[inline]
pub fn hash(w: u16, mask: &[u8; 8]) -> u8 {
    let add = (mask[0] as u16) | ((mask[1] as u16) << 8);
    let shift = mask[2];
    let mut mask = (mask[4] as u16) | ((mask[5] as u16) << 8);
    let mut w = w ^ ((w + add) << shift);
    let mut res = 0;
    for _ in 0 .. 16 {
        res <<= (mask & 0x0001) as u8;
        res |= (w & mask & 0x0001) as u8;
        w >>= 1;
        mask >>= 1;
    }
    res
}

const CODE_PAGE_SIZE: usize = 528;

#[derive(Debug, Clone)]
#[repr(C, align(8))]
pub struct CodePage([u8; CODE_PAGE_SIZE]);

impl CodePage {
    pub unsafe fn new_unchecked(bytes: [u8; CODE_PAGE_SIZE]) -> Self {
        CodePage(bytes)
    }

    pub fn into_bytes(self) -> [u8; CODE_PAGE_SIZE] {
        self.0
    }

    fn to_upper_char(&self, c: u8) -> Option<char> {
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
            self.to_upper_char(c & 0x7F)
        }
    }

    pub fn from_char(&self, c: char) -> Option<u8> {
        if (c as u32) >> 7 == 0 {
            Some(c as u32 as u8)
        } else if (c as u32) >> 16 != 0 {
            None
        } else {
            let w = (c as u32) as u16;
            assert!(align_of::<[u8; 8]>() <= align_of::<CodePage>());
            let mask = unsafe { &*(self.0.as_ptr() as *const [u8; 8]).offset(1) };
            let offset = 16 + 256 + 2 * hash(w, mask) as usize;
            let try_1 = self.0[offset];
            if try_1 >> 7 != 0 { return None; }
            if self.to_upper_char(try_1) == Some(c) {
                return Some(0x80 | try_1);
            }
            let try_2 = self.0[offset + 1];
            if try_2 >> 7 != 0 { return None; }
            if self.to_upper_char(try_2) == Some(c) {
                return Some(0x80 | try_2);
            }
            None
        }
    }
}

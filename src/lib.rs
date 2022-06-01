#![deny(warnings)]

#![no_std]

use core::num::NonZeroU32;

#[doc(hidden)]
#[inline]
pub fn hash(w: u16, p: u16) -> u8 {
    let w = w.wrapping_add(p);
    ((w ^ (w >> 8)) & 0x007F) as u8
}

const CODE_PAGE_SIZE: usize = 520;

#[derive(Debug, Clone)]
#[repr(C, align(8))]
pub struct CodePage([u8; CODE_PAGE_SIZE]);

impl CodePage {
    /// # Safety
    ///
    /// This function may not be called with bytes
    /// are not obtained from the [`into_bytes`](CodePage::into_bytes) method.
    pub const unsafe fn new_unchecked(bytes: [u8; CODE_PAGE_SIZE]) -> Self {
        CodePage(bytes)
    }

    pub fn new(bytes: [u8; CODE_PAGE_SIZE]) -> Option<Self> {
        if bytes[0] != b'C' { return None; }
        if bytes[1] != b'P' { return None; }
        if bytes[2] != 1 { return None; }
        if bytes[3] != 0 { return None; }
        Some(unsafe { Self::new_unchecked(bytes) })
    }

    pub fn into_bytes(self) -> [u8; CODE_PAGE_SIZE] {
        self.0
    }

    fn to_upper_half_char(&self, c: u8) -> Option<char> {
        let offset = (CODE_PAGE_SIZE - 512) + 2 * c as usize;
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
            let hash_param = (self.0[6] as u16) | ((self.0[7] as u16) << 8);
            let offset = (CODE_PAGE_SIZE - 512) + 256 + 2 * hash(w, hash_param) as usize;
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

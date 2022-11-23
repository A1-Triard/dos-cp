//! **Crate features**
//!
//! * `"nightly"`
//! Enabled by default. Disable to make the library compatible with stable and beta Rust channels.

#![cfg_attr(feature="nightly", feature(const_char_from_u32_unchecked))]

#![deny(warnings)]

#![no_std]

use core::num::NonZeroU32;

#[doc(hidden)]
#[inline]
pub const fn hash(w: u16, p: u16) -> u8 {
    let w = w.wrapping_add(p);
    ((w ^ (w >> 8)) & 0x007F) as u8
}

const CODE_PAGE_SIZE: usize = 512;

#[derive(Debug, Clone)]
#[repr(C, align(8))]
pub struct CodePage(pub [u8; CODE_PAGE_SIZE]);

impl CodePage {
    #[cfg(feature="nightly")]
    const fn to_upper_half_char(&self, c: u8) -> Option<char> {
        let offset = 2 * c as usize;
        let hb = self.0[offset];
        let lb = self.0[offset + 1];
        if let Some(c) = NonZeroU32::new(((hb as u32) << 8) | (lb as u32)) {
            Some(unsafe { char::from_u32_unchecked(c.get()) })
        } else {
            None
        }
    }

    #[cfg(not(feature="nightly"))]
    fn to_upper_half_char(&self, c: u8) -> Option<char> {
        let offset = 2 * c as usize;
        let hb = self.0[offset];
        let lb = self.0[offset + 1];
        NonZeroU32::new(((hb as u32) << 8) | (lb as u32)).map(|x|
            unsafe { char::from_u32_unchecked(x.get()) }
        )
    }

    #[cfg(feature="nightly")]
    pub const fn to_char(&self, c: u8) -> Option<char> {
        let half = c & 0x7F;
        if c == half {
            Some(c as char)
        } else {
            self.to_upper_half_char(half)
        }
    }

    #[cfg(not(feature="nightly"))]
    pub fn to_char(&self, c: u8) -> Option<char> {
        let half = c & 0x7F;
        if c == half {
            Some(c as char)
        } else {
            self.to_upper_half_char(half)
        }
    }

    #[cfg(feature="nightly")]
    pub const fn from_char(&self, c: char) -> Option<u8> {
        if (c as u32) >> 7 == 0 {
            Some(c as u32 as u8)
        } else if (c as u32) >> 16 != 0 {
            None
        } else {
            let w = (c as u32) as u16;
            let hash_param = (self.0[510] as u16) | ((self.0[511] as u16) << 8);
            let offset = 256 + 2 * hash(w, hash_param) as usize;
            let try_1 = self.0[offset];
            if try_1 >> 7 != 0 { return None; }
            if let Some(x) = self.to_upper_half_char(try_1) {
                if x == c { return Some(0x80 | try_1); }
            }
            let try_2 = self.0[offset + 1];
            if try_2 >> 7 != 0 { return None; }
            if let Some(x) = self.to_upper_half_char(try_2) {
                if x == c { return Some(0x80 | try_2); }
            }
            None
        }
    }

    #[cfg(not(feature="nightly"))]
    pub fn from_char(&self, c: char) -> Option<u8> {
        if (c as u32) >> 7 == 0 {
            Some(c as u32 as u8)
        } else if (c as u32) >> 16 != 0 {
            None
        } else {
            let w = (c as u32) as u16;
            let hash_param = (self.0[510] as u16) | ((self.0[511] as u16) << 8);
            let offset = 256 + 2 * hash(w, hash_param) as usize;
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

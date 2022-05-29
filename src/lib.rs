#![deny(warnings)]

#![no_std]

use core::num::NonZeroU8;

#[derive(Clone, Copy, Eq, PartialEq, Ord, PartialOrd, Hash)]
pub struct CP {
    to_uni: unsafe fn(u8) -> u32,
    from_uni: unsafe fn(u32) -> Option<NonZeroU8>,
}

impl CP {
    pub fn to_char(&self, c: u8) -> char {
        if c < 128 {
            c as char
        } else {
            unsafe { char::from_u32_unchecked((self.to_uni)(c & 0x7F)) }
        }
    }

    pub fn from_char(&self, c: char) -> u8 {
        if (c as u32) < 128 {
            c as u32 as u8
        } else {
            unsafe { (self.from_uni)(c as u32) }.map_or(0, |x| x.get())
        }
    }
}

include!(concat!(env!("OUT_DIR"), "/generated.rs"));

#[cfg(test)]
mod tests {
    #[test]
    fn it_works() {
        let result = 2 + 2;
        assert_eq!(result, 4);
    }
}

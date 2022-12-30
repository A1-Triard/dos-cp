//! **Crate features**
//!
//! * `"nightly"`
//! Enabled by default. Disable to make the library compatible with stable and beta Rust channels.

#![cfg_attr(feature="nightly", feature(const_char_from_u32_unchecked))]

#![deny(warnings)]

#![no_std]

#[cfg(feature="load")]
use core::mem::{MaybeUninit, forget, transmute};
use core::num::NonZeroU32;
#[cfg(feature="load")]
use core::slice::{self};
#[cfg(feature="load")]
use errno_no_std::Errno;
#[cfg(feature="load")]
use pc_ints::*;

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

    #[cfg(feature="load")]
    pub fn load() -> Result<&'static CodePage, CodePageLoadError> {
        let dos_ver = int_21h_ah_30h_dos_ver();
        if dos_ver.al_major < 3 || dos_ver.al_major == 3 && dos_ver.ah_minor < 30 {
            return Err(CodePageLoadError::Dos33Required);
        }
        let code_page_memory = int_31h_ax_0100h_rm_alloc(8)
            .map_err(|e| CodePageLoadError::CanNotAlloc(Errno(e.ax_err.into())))?;
        let code_page_selector = RmAlloc { selector: code_page_memory.dx_selector };
        let code_page_memory = unsafe { slice::from_raw_parts_mut(
            ((code_page_memory.ax_segment as u32) << 4) as *mut u8,
            512
        ) };
        let code_page_n = int_21h_ax_6601h_code_page()
            .map_err(|e| CodePageLoadError::CanNotGetSelectedCodePage(Errno(e.ax_err.into())))?
            .bx_active;
        if code_page_n > 999 {
            return Err(CodePageLoadError::UnsupportedCodePage(code_page_n));
        }
        let mut code_page: [MaybeUninit<u8>; 13] = unsafe { MaybeUninit::uninit().assume_init() };
        code_page[.. 9].copy_from_slice(unsafe { transmute(&b"CODEPAGE\\"[..]) });
        code_page[9].write(b'0' + (code_page_n / 100) as u8);
        code_page[10].write(b'0' + ((code_page_n % 100) / 10) as u8);
        code_page[11].write(b'0' + (code_page_n % 10) as u8);
        code_page[12].write(0);
        let code_page: [u8; 13] = unsafe { transmute(code_page) };
        let code_page = int_21h_ah_3Dh_open(code_page.as_ptr(), 0x00)
            .map_err(|e| CodePageLoadError::CanNotOpenCodePageFile(code_page_n, Errno(e.ax_err.into())))?
            .ax_handle;
        let mut code_page_buf: &mut [MaybeUninit<u8>] = unsafe { transmute(&mut code_page_memory[..]) };
        loop {
            if code_page_buf.is_empty() {
                let mut byte: MaybeUninit<u8> = MaybeUninit::uninit();
                let read = int_21h_ah_3Fh_read(code_page, slice::from_mut(&mut byte))
                    .map_err(|e| CodePageLoadError::CanNotReadCodePageFile(code_page_n, Errno(e.ax_err.into())))?
                    .ax_read;
                if read != 0 {
                    return Err(CodePageLoadError::InvalidCodePageFile(code_page_n));
                }
                break;
            }
            let read = int_21h_ah_3Fh_read(code_page, code_page_buf)
                .map_err(|e| CodePageLoadError::CanNotReadCodePageFile(code_page_n, Errno(e.ax_err.into())))?
                .ax_read;
            if read == 0 { break; }
            code_page_buf = &mut code_page_buf[read as usize ..];
        }
        if !code_page_buf.is_empty() {
            return Err(CodePageLoadError::InvalidCodePageFile(code_page_n));
        }
        let code_page = unsafe { &*(code_page_memory.as_ptr() as *const CodePage) };
        forget(code_page_selector);
        Ok(code_page)
    }
}

#[cfg(feature="load")]
pub enum CodePageLoadError {
    Dos33Required,
    CanNotAlloc(Errno),
    CanNotGetSelectedCodePage(Errno),
    UnsupportedCodePage(u16),
    CanNotOpenCodePageFile(u16, Errno),
    CanNotReadCodePageFile(u16, Errno),
    InvalidCodePageFile(u16),
}

#[cfg(feature="load")]
struct RmAlloc {
    selector: u16,
}

#[cfg(feature="load")]
impl Drop for RmAlloc {
    fn drop(&mut self) {
       let _ = int_31h_ax_0101h_rm_free(self.selector);
    }
}

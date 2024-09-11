//! ## Feature flags
#![doc=document_features::document_features!()]

#![deny(warnings)]

#![no_std]

#[cfg(feature="load")]
use core::fmt::{self, Debug, Display, Formatter};
#[cfg(feature="load")]
use core::mem::{MaybeUninit, forget, transmute};
use core::num::NonZeroU32;
#[cfg(feature="load")]
use core::ptr::{self};
#[cfg(feature="load")]
use core::slice::{self};
#[cfg(feature="load")]
use core::sync::atomic::{AtomicBool, Ordering};
#[cfg(feature="load")]
use either::{Either, Left, Right};
#[cfg(feature="load")]
use exit_no_std::exit;
#[cfg(feature="load")]
use iter_identify_first_last::IteratorIdentifyFirstLastExt;
#[cfg(feature="load")]
use panicking::panicking;
#[cfg(feature="load")]
use pc_ints::*;

#[doc(hidden)]
pub use core::write as std_write;
#[doc(hidden)]
pub use core::writeln as std_writeln;

#[doc(hidden)]
#[inline]
pub const fn hash(w: u16, p: u16) -> u8 {
    let w = w.wrapping_add(p);
    ((w ^ (w >> 8)) & 0x007F) as u8
}

const CODE_PAGE_SIZE: u16 = 512;

#[derive(Debug, Clone)]
#[repr(C, align(8))]
pub struct CodePage(pub [u8; CODE_PAGE_SIZE as _]);

impl CodePage {
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

    pub const fn to_char(&self, c: u8) -> Option<char> {
        let half = c & 0x7F;
        if c == half {
            Some(c as char)
        } else {
            self.to_upper_half_char(half)
        }
    }

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

    #[cfg(feature="load")]
    pub fn load_or_exit_with_msg(exit_code: u8) -> &'static CodePage {
        match Self::load() {
            Ok(cp) => cp,
            Err(e) => {
                write!(DosLastChanceWriter, "Error: {e}.").unwrap();
                exit(exit_code);
            },
        }
    }

    #[cfg(feature="load")]
    pub fn load() -> Result<&'static CodePage, CodePageLoadError> {
        let mut loaded = LoadedCodePageGuard::acquire();
        let loaded_code_page = loaded.code_page();
        if let Some(code_page) = loaded_code_page {
            return Ok(code_page);
        }
        let dos_ver = int_21h_ah_30h_dos_ver();
        if dos_ver.al_major < 3 || dos_ver.al_major == 3 && dos_ver.ah_minor < 30 {
            return Err(CodePageLoadError::Dos33Required);
        }
        let code_page_memory = int_31h_ax_0100h_rm_alloc(CODE_PAGE_SIZE.checked_add(15).unwrap() / 16)
            .map_err(|e| CodePageLoadError::CanNotAlloc { err_code: e.ax_err })?;
        let code_page_selector = RmAlloc { selector: code_page_memory.dx_selector };
        let code_page_memory = unsafe { slice::from_raw_parts_mut(
            ((code_page_memory.ax_segment as u32) << 4) as *mut u8,
            CODE_PAGE_SIZE.into()
        ) };
        let code_page_n = int_21h_ax_6601h_code_page()
            .map_err(|e| CodePageLoadError::CanNotGetSelectedCodePage { err_code: e.ax_err })?
            .bx_active;
        if !(100 ..= 999).contains(&code_page_n) {
            return Err(CodePageLoadError::UnsupportedCodePage { code_page: code_page_n });
        }
        let mut code_page: [MaybeUninit<u8>; 13] = unsafe { MaybeUninit::uninit().assume_init() };
        code_page[.. 9].copy_from_slice(unsafe { transmute::<&[u8], &[MaybeUninit<u8>]>(&b"CODEPAGE\\"[..]) });
        code_page[9].write(b'0' + (code_page_n / 100) as u8);
        code_page[10].write(b'0' + ((code_page_n % 100) / 10) as u8);
        code_page[11].write(b'0' + (code_page_n % 10) as u8);
        code_page[12].write(0);
        let code_page: [u8; 13] = unsafe { transmute(code_page) };
        let code_page = int_21h_ah_3Dh_open(code_page.as_ptr(), 0x00)
            .map_err(|e| CodePageLoadError::CanNotOpenCodePageFile { code_page: code_page_n, err_code: e.ax_err })?
            .ax_handle;
        let code_page = File(code_page);
        let mut code_page_buf: &mut [MaybeUninit<u8>] = unsafe { transmute(&mut code_page_memory[..]) };
        loop {
            if code_page_buf.is_empty() {
                let mut byte: MaybeUninit<u8> = MaybeUninit::uninit();
                let read = int_21h_ah_3Fh_read(code_page.0, slice::from_mut(&mut byte))
                    .map_err(|e| CodePageLoadError::CanNotReadCodePageFile { code_page: code_page_n, err_code: e.ax_err })?
                    .ax_read;
                if read != 0 {
                    return Err(CodePageLoadError::InvalidCodePageFile { code_page: code_page_n });
                }
                break;
            }
            let read = int_21h_ah_3Fh_read(code_page.0, code_page_buf)
                .map_err(|e| CodePageLoadError::CanNotReadCodePageFile { code_page: code_page_n, err_code: e.ax_err })?
                .ax_read;
            if read == 0 { break; }
            code_page_buf = &mut code_page_buf[read as usize ..];
        }
        if !code_page_buf.is_empty() {
            return Err(CodePageLoadError::InvalidCodePageFile { code_page: code_page_n });
        }
        let code_page = unsafe { &*(code_page_memory.as_ptr() as *const CodePage) };
        forget(code_page_selector);
        loaded_code_page.replace(code_page);
        Ok(code_page)
    }

    #[cfg(feature="load")]
    pub fn inkey(&self) -> Result<Option<Either<u8, char>>, InkeyErr> {
        let c = int_21h_ah_06h_dl_FFh_inkey().map_err(|_| InkeyErr)?;
        let c = match c {
            Some(x) => x.al_char,
            None => return Ok(None),
        };
        if c == 0 {
            let c = int_21h_ah_06h_dl_FFh_inkey().map_err(|_| InkeyErr)?;
            let c = c.ok_or(InkeyErr)?.al_char;
            Ok(Some(Left(c)))
        } else {
            Ok(self.to_char(c).map(Right))
        }
    }
}

#[cfg(feature="load")]
pub fn inkey() -> Result<Option<Either<u8, char>>, InkeyErr> {
    let cp = CodePage::load().map_err(|_| InkeyErr)?;
    cp.inkey()
}

#[cfg(feature="load")]
#[derive(Debug)]
pub struct InkeyErr;

#[cfg(feature="load")]
struct DosLastChanceWriter;

#[cfg(feature="load")]
impl DosLastChanceWriter {
    pub fn write_fmt(&mut self, args: fmt::Arguments) -> fmt::Result {
        <Self as fmt::Write>::write_fmt(self, args)
    }
}

#[cfg(feature="load")]
impl fmt::Write for DosLastChanceWriter {
    fn write_char(&mut self, c: char) -> fmt::Result {
        let c = c as u32;
        let c = if c > 0x7F || c == '\r' as u32 {
            b'?'
        } else {
            c as u8
        };
        if c == b'\n' {
            int_21h_ah_02h_out_ch(b'\r');
        }
        int_21h_ah_02h_out_ch(c);
        Ok(())
    }

    fn write_str(&mut self, s: &str) -> fmt::Result {
        for c in s.chars() {
            self.write_char(c)?;
        }
        Ok(())
    }
}

#[cfg(feature="load")]
struct File(u16);

#[cfg(feature="load")]
impl Drop for File {
    fn drop(&mut self) {
        let r = int_21h_ah_3Eh_close(self.0);
        if r.is_err() && !panicking() {
            #[allow(clippy::panicking_unwrap)]
            r.unwrap();
        }
    }
}

#[cfg(feature="load")]
struct LoadedCodePageGuard;

#[cfg(feature="load")]
static LOADED_CODE_PAGE_GUARD: AtomicBool = AtomicBool::new(false);

#[cfg(feature="load")]
static mut LOADED_CODE_PAGE: Option<&'static CodePage> = None;

#[cfg(feature="load")]
impl LoadedCodePageGuard {
    fn acquire() -> Self {
        loop {
            if LOADED_CODE_PAGE_GUARD.compare_exchange_weak(false, true, Ordering::SeqCst, Ordering::Relaxed).is_ok() {
                break;
            }
        }
        LoadedCodePageGuard
    }

    fn code_page(&mut self) -> &mut Option<&'static CodePage> {
        unsafe { &mut *ptr::addr_of_mut!(LOADED_CODE_PAGE) }
    }
}

#[cfg(feature="load")]
impl Drop for LoadedCodePageGuard {
    fn drop(&mut self) {
        LOADED_CODE_PAGE_GUARD.store(false, Ordering::SeqCst);
    }
}

#[cfg(feature="load")]
pub enum CodePageLoadError {
    Dos33Required,
    CanNotAlloc { err_code: u16 },
    CanNotGetSelectedCodePage { err_code: u16 },
    UnsupportedCodePage { code_page: u16 },
    CanNotOpenCodePageFile { code_page: u16, err_code: u16 },
    CanNotReadCodePageFile { code_page: u16, err_code: u16 },
    InvalidCodePageFile { code_page: u16 },
}

#[cfg(feature="load")]
impl CodePageLoadError {
    pub fn code_page(&self) -> Option<u16> {
        match self {
            CodePageLoadError::Dos33Required => None,
            CodePageLoadError::CanNotAlloc { .. } => None,
            CodePageLoadError::CanNotGetSelectedCodePage { .. } => None,
            &CodePageLoadError::UnsupportedCodePage { code_page } => Some(code_page),
            &CodePageLoadError::CanNotOpenCodePageFile { code_page, .. } => Some(code_page),
            &CodePageLoadError::CanNotReadCodePageFile { code_page, .. } => Some(code_page),
            &CodePageLoadError::InvalidCodePageFile { code_page } => Some(code_page),
        }
    }
}

#[cfg(feature="load")]
impl Display for CodePageLoadError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        match self {
            CodePageLoadError::Dos33Required => write!(f, "DOS >= 3.3 reequired"),
            CodePageLoadError::CanNotAlloc { err_code } =>
                write!(f, "cannot allocate real-mode memory for code page ({err_code:04X}h)"),
            CodePageLoadError::CanNotGetSelectedCodePage { err_code } =>
                write!(f, "cannon get selected code page ({err_code:04X}h)"),
            CodePageLoadError::UnsupportedCodePage { code_page } => write!(f, "unsupported code page {code_page}"),
            CodePageLoadError::CanNotOpenCodePageFile { code_page, err_code } =>
                write!(f, "cannot open code page file 'CODEPAGE\\{code_page}' ({err_code:04X}h)"),
            CodePageLoadError::CanNotReadCodePageFile { code_page, err_code } =>
                write!(f, "cannot read code page file 'CODEPAGE\\{code_page}' ({err_code:04X}h)"),
            CodePageLoadError::InvalidCodePageFile { code_page } => write!(f, "invalid code page file 'CODEPAGE\\{code_page}'"),
        }
    }
}

#[cfg(feature="load")]
impl Debug for CodePageLoadError {
    fn fmt(&self, f: &mut Formatter) -> fmt::Result {
        <Self as Display>::fmt(self, f)
    }
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

#[cfg(feature="load")]
pub struct DosStdout { pub panic: bool }

#[cfg(feature="load")]
impl DosStdout {
    pub fn write_fmt(&mut self, args: fmt::Arguments) -> fmt::Result {
        <Self as fmt::Write>::write_fmt(self, args)
    }
}

#[cfg(feature="load")]
impl fmt::Write for DosStdout {
    fn write_char(&mut self, c: char) -> fmt::Result {
        let cp = CodePage::load();
        let cp = if self.panic { cp.unwrap() } else { cp.map_err(|_| fmt::Error)? };
        let c = cp.from_char(c).unwrap_or(b'?');
        match c {
            b'\r' => Ok(()),
            b'\n' => match int_21h_ah_40h_write(1, b"\r\n") {
                Err(_) => Err(fmt::Error),
                Ok(AxWritten { ax_written }) if ax_written < 2 => Err(fmt::Error),
                _ => Ok(()),
            },
            c => match int_21h_ah_40h_write(1, &[c]) {
                Err(_) | Ok(AxWritten { ax_written: 0 }) => Err(fmt::Error),
                _ => Ok(()),
            }
        }
    }

    fn write_str(&mut self, s: &str) -> fmt::Result {
        let cp = CodePage::load();
        let cp = if self.panic { cp.unwrap() } else { cp.map_err(|_| fmt::Error)? };
        let mut buf = [0; 128];
        for (skip_newline, s) in s.split('\n').identify_last() {
            let mut i = 0;
            for (is_last, c) in s.chars().identify_last() {
                buf[i] = cp.from_char(c).unwrap_or(b'?');
                i += 1;
                if is_last || i == buf.len() {
                    match int_21h_ah_40h_write(1, &buf[.. i]) {
                        Err(_) => return Err(fmt::Error),
                        Ok(AxWritten { ax_written }) if usize::from(ax_written) < i => return Err(fmt::Error),
                        _ => { },
                    }
                    i = 0;
                }
            }
            if !skip_newline {
                match int_21h_ah_40h_write(1, b"\r\n") {
                    Err(_) => return Err(fmt::Error),
                    Ok(AxWritten { ax_written }) if ax_written < 2 => return Err(fmt::Error),
                    _ => { },
                }
            }
        }
        Ok(())
    }
}

#[cfg(feature="load")]
#[macro_export]
macro_rules! print {
    (
        $($arg:tt)*
    ) => {
        $crate::std_write!($crate::DosStdout { panic: true }, $($arg)*).unwrap()
    };
}

#[cfg(feature="load")]
#[macro_export]
macro_rules! println {
    (
    ) => {
        $crate::std_writeln!($crate::DosStdout { panic: true }).unwrap()
    };
    (
        $($arg:tt)*
    ) => {
        $crate::std_writeln!($crate::DosStdout { panic: true }, $($arg)*).unwrap()
    };
}

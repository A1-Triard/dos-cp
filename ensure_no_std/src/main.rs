#![deny(warnings)]

#![no_std]
#![no_main]

#[cfg(windows)]
#[link(name="msvcrt")]
extern "C" { }

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    exit_no_std::exit(99)
}

use core::ffi::{c_char, c_int};
use dos_cp::CodePage;

include!(concat!(env!("OUT_DIR"), "/cp852.rs"));

#[unsafe(no_mangle)]
extern "C" fn main(_argc: c_int, _argv: *mut *mut c_char) -> c_int {
    assert_eq!(CP852.to_char(0xF8), Some('\u{00B0}'));
    0
}

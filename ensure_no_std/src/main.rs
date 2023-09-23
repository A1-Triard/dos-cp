#![feature(start)]

#![deny(warnings)]

#![no_std]

#[cfg(windows)]
#[link(name="msvcrt")]
extern { }

#[panic_handler]
fn panic(_info: &core::panic::PanicInfo) -> ! {
    exit_no_std::exit(99)
}

use dos_cp::CodePage;

include!(concat!(env!("OUT_DIR"), "/cp852.rs"));

#[start]
pub fn main(_argc: isize, _argv: *const *const u8) -> isize {
    assert_eq!(CP852.to_char(0xF8), Some('\u{00B0}'));
    0
}

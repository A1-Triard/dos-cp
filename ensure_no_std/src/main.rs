#![feature(allocator_api)]
#![feature(default_alloc_error_handler)]
#![feature(explicit_generic_args_with_impl_trait)]
#![feature(iter_collect_into)]
#![feature(start)]

#![deny(warnings)]

#![no_std]

use core::panic::PanicInfo;
#[cfg(not(windows))]
use libc::exit;
#[cfg(windows)]
use winapi::shared::minwindef::UINT;
#[cfg(windows)]
use winapi::um::processthreadsapi::ExitProcess;

#[cfg(windows)]
#[link(name="msvcrt")]
extern { }

#[cfg(windows)]
unsafe fn exit(code: UINT) -> ! {
    ExitProcess(code);
    loop { }
}

#[panic_handler]
pub extern fn panic(_info: &PanicInfo) -> ! {
    unsafe { exit(99) }
}

use dos_cp::CodePage;

include!(concat!(env!("OUT_DIR"), "/cp852.rs"));

#[start]
pub fn main(_argc: isize, _argv: *const *const u8) -> isize {
    assert_eq!(CP852.to_char(0xF8), Some('\u{00B0}'));
    0
}

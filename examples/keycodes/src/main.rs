#![feature(extern_types)]

#![deny(warnings)]

#![windows_subsystem="console"]
#![no_std]
#![no_main]

extern crate pc_atomics;
extern crate rlibc_ext;

mod no_std {
    #[panic_handler]
    fn panic_handler(info: &core::panic::PanicInfo) -> ! { panic_no_std::panic(info, b'P') }
}

extern {
    type PEB;
}

use dos_cp::{CodePage, inkey, println};
use either::Right;

#[allow(non_snake_case)]
#[no_mangle]
extern "stdcall" fn mainCRTStartup(_: *const PEB) -> u64 {
    CodePage::load_or_exit_with_msg(1);
    loop {
        if let Some(c) = inkey().unwrap() {
            println!("{c:?}");
            if c == Right(' ') { break; }
        }
    }
    0
}

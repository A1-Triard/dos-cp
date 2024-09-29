#![feature(extern_types)]

#![deny(warnings)]

#![no_std]
#![no_main]
#![windows_subsystem="console"]

extern crate rlibc_ext;

mod no_std {
    #[panic_handler]
    fn panic_handler(info: &core::panic::PanicInfo) -> ! { panic_no_std::panic(info, b'P') }
}

use dos_cp::{CodePage, println};
use exit_no_std::exit;

#[allow(non_snake_case)]
#[no_mangle]
extern "C" fn mainCRTStartup() -> ! {
    CodePage::load_or_exit_with_msg(1);
    println!("Hello, DOS!");
    exit(0)
}

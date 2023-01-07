#![feature(extern_types)]

#![deny(warnings)]

#![windows_subsystem="console"]
#![no_std]
#![no_main]

extern crate dos_errno_and_panic;
extern crate pc_atomics;
extern crate rlibc;

mod no_std {
    #[no_mangle]
    extern "C" fn _aulldiv() -> ! { panic!("10") }
    #[no_mangle]
    extern "C" fn _aullrem() -> ! { panic!("11") }
    #[no_mangle]
    extern "C" fn _chkstk() { }
    #[no_mangle]
    extern "C" fn _fltused() -> ! { panic!("13") }
    #[no_mangle]
    extern "C" fn strlen() -> ! { panic!("14") }
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

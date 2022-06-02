#![deny(warnings)]
use dos_cp::CodePage;
use dos_cp_generator::{CodePageGenExt};

use std::env::var_os;
use std::fs::File;
use std::io::Write;
use std::path::Path;

fn main() {
    let out_dir = var_os("OUT_DIR").unwrap();
    let out_dir = Path::new(&out_dir);
    let cp852 = out_dir.join("cp852.rs");
    let mut cp852 = File::create(cp852).unwrap();
    cp852.write_all(b"const CP852: CodePage = CodePage::new(\n").unwrap();
    cp852.write_all(format!("{:?}", CodePage::generate(852).into_bytes()).as_bytes()).unwrap();
    cp852.write_all(b");\n").unwrap();
}

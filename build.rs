mod gen;

use {
    gen::{codegen::*, parser::*},
    std::{env, fs::File, io, path::Path},
};

const CODE: &str = include_str!("gen/codes.txt");

fn main() -> io::Result<()> {
    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("code.rs");

    let mut file = File::create(&dest_path)?;

    let codes = Code::from_iter(CODE.split('\n'));

    gen_code(&codes, &mut file)?;

    Ok(())
}

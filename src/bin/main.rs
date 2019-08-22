//! frothy
//!
//! TODO: finish parse functions
//! TODO: add 'near x' parse error messages by keeping token position and last token

use std::env;
use std::fs::File;
use std::io::prelude::*;

fn main() {
    // read the file at the path argument
    let path = env::args().nth(1).expect("usage: frothy <path>");
    let mut file = File::open(path).expect("file not found");
    let mut program = String::new();
    file.read_to_string(&mut program)
        .expect("failed to read file");

    match frothy::exec(&program) {
        Ok(_) => {}
        Err(e) => eprintln!("error = {:?}", e),
    }
}

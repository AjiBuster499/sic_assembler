/* Rust Implementation of SIC Assembler
* Original C code by Samuel Mikell (AjiBuster499)
*/
#![allow(dead_code, unused)]
mod symbols;

use std::{env, process};

use sic_assembler::{self, Config};

fn main() {
    // establishing variables
    let config = Config::new(env::args()).unwrap_or_else(|err| {
        eprintln!("Error: {}", err);
        process::exit(1);
    });

    if let Err(e) = sic_assembler::run(config) {
        eprintln!("Error during execution: {}", e);

        process::exit(1);
    }
}

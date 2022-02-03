/* Logic for the Rusty SIC Assembler
*/

#![allow(unused, dead_code)]
mod data_records;
mod symbols;

use data_records::ObjectData;
use std::{
    collections::HashMap,
    env,
    fs::File,
    io::{BufRead, BufReader, Seek},
    vec,
};
use symbols::Symbol;

const MAX_MEMORY: i32 = 0x7FFF;
// Holds inputted args (for now, just filename)
// maybe future uses can include flags
pub struct Config {
    filename: String,
}

impl Config {
    pub fn new(mut args: env::Args) -> Result<Config, &'static str> {
        args.next(); // discard the program itself

        // get the fileName from the next argument
        let fname = match args.next() {
            Some(args) => args,
            None => return Err("Usage is sic_assembler <filename>"),
        };

        Ok(Config { filename: fname })
    }

    pub fn filename(&self) -> &str {
        &self.filename
    }
}

pub struct AssemblyLine {
    symbol: Option<String>,
    directive: Option<String>,
    operand: Option<String>,
}

impl AssemblyLine {
    pub fn new(sym: Option<String>, dir: Option<String>, op: Option<String>) -> Self {
        AssemblyLine {
            symbol: sym,
            directive: dir,
            operand: op,
        }
    }

    pub fn symbol(&self) -> Option<&String> {
        self.symbol.as_ref()
    }

    pub fn directive(&self) -> Option<&String> {
        self.directive.as_ref()
    }

    pub fn operand(&self) -> Option<&String> {
        self.operand.as_ref()
    }
}

// Connection between main.rs and lib.rs
pub fn run(config: Config) -> Result<(), &'static str> {
    let mut address_counter: i32 = 0; // address counter for symbols
    let mut symbol_table: Vec<Symbol> = vec![]; // symbol table, initially empty.

    let sic_asm_file = File::open(config.filename);

    let sic_asm_file = match sic_asm_file {
		    Ok(file) => file,
	      _ => return Err("could not open file. Please ensure that file exists and that you have permission to open it."),
	  };

    let mut reader = BufReader::new(sic_asm_file);
    let mut buffer = String::new();

    // Key is instruction, value is opcode
    let mut opcodes_lists: HashMap<&str, &str> = HashMap::new();

    initalize_opcodes(&mut opcodes_lists);

    for (k, v) in opcodes_lists {
        println!("Instruction: {:}, Opcode: {:}", k, v);
    }
    // main loop
    'pass1: loop {
        buffer.clear();
        // read a line
        if let Ok(line) = reader.read_line(&mut buffer) {
            // EOF Encountered
            if line == 0 {
                break 'pass1; // exit out of 'pass1 loop
            }

            // memory exceeds maximum
            if is_memory_out_of_bounds(&address_counter) {
                return Err("Memory out of Bounds");
            }

            // check for symbol line
            if is_symbol_line(&buffer, &mut address_counter) != 1 {
                // not a symbol line
                continue 'pass1;
            }

            // add the symbol to the symbol table
            let broken_line: Vec<&str> = buffer.split_ascii_whitespace().collect();
            let line = AssemblyLine::new(
                Some(broken_line[0].to_string()),
                Some(broken_line[1].to_string()),
                Some(broken_line[2].to_string()),
            );

            // START directive, aka first line.
            // This means that we have to set the address and move on.
            if line.directive().unwrap() == "START" {
                // address comes in as a hex string, need to convert to decimal
                let address_as_hex = i32::from_str_radix(line.operand().unwrap(), 16).unwrap();

                address_counter += address_as_hex;
                continue;
            }
            // Add new symbol to symbol table
            if let Some(symbol_name) = line.symbol() {
                symbol_table.push(Symbol::new(symbol_name.to_string(), address_counter));
            }

            // Call function to determine address increment here
            let address_increment =
                get_address_increment(line.directive().unwrap(), line.operand().unwrap());

            address_counter += address_increment;
        }
    }

    // pass 2 loop: Creating the records
    let _starting_address = 0; // preserve the old starting address
    let mut _length = 3; // default length of object code
    let _text_index = 0; // used for text records (maybe unneeded with Rust)

    reader.rewind().unwrap(); // reset the reader

    'pass2: loop {
        buffer.clear();
        if let Ok(line) = reader.read_line(&mut buffer) {
            if line == 0 {
                break 'pass2;
            }

            match is_symbol_line(&buffer, &mut address_counter) {
                0 => { // symbol line
                     // just fall through to the symbol
                }
                1 => {
                    // non-symbol assembly line
                    // tokenize the line
                    let broken_line: Vec<&str> = buffer.split_ascii_whitespace().collect();
                    let line = AssemblyLine::new(
                        None,
                        Some(broken_line[1].to_string()),
                        Some(broken_line[2].to_string()),
                    );

                    let increment =
                        get_address_increment(line.directive().unwrap(), line.operand().unwrap());

                    address_counter += increment;
                    continue 'pass2;
                }
                _ => {
                    // comment line
                    continue 'pass2;
                }
            }

            // tokenize the line containing a symbol
            let broken_line: Vec<&str> = buffer.split_ascii_whitespace().collect();
            let line = AssemblyLine::new(
                Some(broken_line[0].to_string()),
                Some(broken_line[1].to_string()),
                Some(broken_line[2].to_string()),
            );

            let increment =
                get_address_increment(line.directive().unwrap(), line.operand().unwrap());
        }
    }

    Ok(())
}

// checks if memory is out of bounds (SIC Max memory is 0x0000 to 0x7FFF)
fn is_memory_out_of_bounds(current_counter: &i32) -> bool {
    if *current_counter >= MAX_MEMORY {
        return true;
    }

    false
}

// Checks if line has a symbol
// returns an i32 as follows:
// 0: Symbol Line
// 1: Non-symbol assembly Line
// -1: Non-assembly line
fn is_symbol_line(buffer: &str, counter: &mut i32) -> i32 {
    if buffer.starts_with('\t') {
        *counter += 3;
        return 1;
    } else if buffer.starts_with('#') {
        // do nothing
    } else {
        // symbol line
        return 0;
    }

    -1
}

// returns the address increment
fn get_address_increment(directive: &String, operand: &String) -> i32 {
    let address_increment;
    match directive.as_str() {
        "RESB" => {
            address_increment = operand.parse::<i32>().unwrap();
        }
        "RESW" => {
            address_increment = (operand.parse::<i32>().unwrap()) * 3;
        }
        "BYTE" if operand.starts_with('C') => {
            let len = operand.len() - 4;
            address_increment = len as i32;
        }
        "BYTE" if operand.starts_with('X') => {
            let len = operand.len() - 4;
            address_increment = len as i32 / 2;
        }
        "END" => address_increment = 0,
        _ => {
            address_increment = 3;
        }
    }

    address_increment
}

// initializes the opcodes for the SIC machine
// This WILL be painful to read
fn initalize_opcodes(opcodes_lists: &mut HashMap<&str, &str>) {
    let instructions = vec![
        "ADD", "AND", "COMP", "DIV", "J", "JEQ", "JGT", "JLT", "JSUB", "LDA", "LDCH", "LDL", "LDX",
        "MUL", "OR", "RD", "RSUB", "STA", "STCH", "STL", "STSW", "STX", "SUB", "TD", "TIX", "WD",
    ];
    let opcodes = vec![
        "18", "40", "28", "24", "3C", "30", "34", "38", "48", "00", "50", "08", "04", "20", "44",
        "D8", "4C", "0C", "54", "14", "E8", "10", "1C", "E0", "2C", "DC",
    ];

    for index in 0..instructions.len() {
        opcodes_lists.insert(opcodes[index], instructions[index]);
    }
}

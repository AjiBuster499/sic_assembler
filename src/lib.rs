/* Logic for the Rusty SIC Assembler
*/

#![allow(unused, dead_code)]
mod data_records;
mod directives;
mod instructions;
mod symbols;

use data_records::ObjectData;
use directives::is_directive;
use instructions::Instruction;
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

pub struct AssemblyLine<'a> {
    symbol: Option<&'a str>,
    directive: Option<&'a str>,
    operand: Option<&'a str>,
}

impl<'a> AssemblyLine<'a> {
    pub fn new(sym: Option<&'a str>, dir: Option<&'a str>, op: Option<&'a str>) -> Self {
        AssemblyLine {
            symbol: sym,
            directive: dir,
            operand: op,
        }
    }

    pub fn symbol(&self) -> Option<&str> {
        self.symbol
    }

    pub fn directive(&self) -> Option<&str> {
        self.directive
    }

    pub fn operand(&self) -> Option<&str> {
        self.operand
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
    let mut opcodes_list: Vec<Instruction> = vec![];

    initalize_opcodes(&mut opcodes_list);

    // main loop
    'pass1: loop {
        buffer.clear();
        // read a line
        if let Ok(line) = reader.read_line(&mut buffer) {
            print!("{:}", &buffer);
            // EOF Encountered
            if line == 0 {
                break 'pass1; // exit out of 'pass1 loop
            }

            // memory exceeds maximum
            if is_memory_out_of_bounds(&address_counter) {
                return Err("Memory out of Bounds");
            }

            // check for symbol line
            if is_symbol_line(&buffer, &mut address_counter) != 0 {
                // not a symbol line
                continue 'pass1;
            }

            // add the symbol to the symbol table
            let mut broken_line = buffer.split_ascii_whitespace();
            let line =
                AssemblyLine::new(broken_line.next(), broken_line.next(), broken_line.next());

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
                get_address_increment(line.directive().unwrap(), line.operand());

            address_counter += address_increment;
        }

        //for symbol in &symbol_table {
        //    println!("Symbol: {:}, Address: {:}", symbol.name(), symbol.address());
        //}
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
                    let mut broken_line = buffer.split_ascii_whitespace();
                    let line = AssemblyLine::new(None, broken_line.next(), broken_line.next());

                    // Need to write textRecords here

                    let increment =
                        get_address_increment(line.directive().unwrap(), line.operand());

                    address_counter += increment;
                    continue 'pass2;
                }
                _ => {
                    // comment line
                    continue 'pass2;
                }
            }

            // tokenize the line containing a symbol
            let mut broken_line = buffer.split_ascii_whitespace();
            let line =
                AssemblyLine::new(broken_line.next(), broken_line.next(), broken_line.next());

            let increment = get_address_increment(line.directive().unwrap(), line.operand());

            // write text records
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

// Creates text records
// arguments: records holder, symbol table,
// opcodes, directive for the line, operand for the line
// length of the record, starting address, and text index
// Some of these may be unnecessary with Rust
fn write_text_record(
    symtable: &Vec<Symbol>,
    opcodes: &Vec<Instruction>,
    directive: &str,
    operand: &str,
    length: &i32,
) -> String {
    // this function wil return text_data
    let mut local_length = *length; // take a local copy of length
    let mut text_data: String = String::new();
    let mut symbol_address = 0;
    // need to locate the symbol in the symbol table
    // as well as the instruction
    let symbol = find_symbol(symtable, operand);
    let opcode = find_instruction(opcodes, directive, operand);

    // If we didn't find the symbol, we need to error
    if let Some(symbol) = symbol {
        // we found the symbol
        // need to set the symbol address
        symbol_address = *symbol.address();
    }

    if let Some(opcode) = opcode {
        if !is_directive(directive) {
            // instruction, so the object code is OP and ADDR
            text_data = format!("{:#02X}{:#04X}", opcode.opcode(), symbol_address);
        } else {
            // It's just a directive
            // BYTE directives can exceed the 60 character object code limit
            // Please look forward to it (tm)
            if operand.len() > 60 {
                todo!(); // please do look forward to it (tm)
            } else {
                if opcode.name() == "WORD" {
                    // word format is %06X (using format from C code as template
                    text_data = format!("{:#06X}", opcode.opcode());
                } else {
                    // This was uncommented in C Code
                    // Dangit past me
                    text_data = format!("{}", operand);
                    local_length /= 2;
                }
            }
        }
    }

    format!("T{}\n", text_data)
}

fn find_symbol<'a>(symtable: &'a Vec<Symbol>, operand: &str) -> Option<&'a Symbol> {
    let found_symbol = symtable.iter().position(|r| r.name() == operand);

    if let Some(index) = found_symbol {
        return symtable.get(index);
    }

    None
}
fn find_instruction<'a>(
    opcodes: &Vec<Instruction>,
    directive: &'a str,
    operand: &'a str,
) -> Option<Instruction<'a>> {
    let mut result = None;
    if is_directive(directive) {
        match directive {
            "RESB" | "RESW" => {
                // do nothing
            }
            "WORD" => {
                result = Some(Instruction::new(
                    directive,
                    i32::from_str_radix(operand, 10).unwrap(),
                ));
            }
            "BYTE" => {
                result = Some(Instruction::new(
                    directive,
                    i32::from_str_radix(operand, 16).unwrap(),
                ));
            }
            _ => {}
        }
    }
    result
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
fn get_address_increment(directive: &str, operand: Option<&str>) -> i32 {
    let mut address_increment = 3;
    if let Some(op) = operand {
        match directive {
            "RESB" => {
                address_increment = op.parse::<i32>().unwrap();
            }
            "RESW" => {
                address_increment = (op.parse::<i32>().unwrap()) * 3;
            }
            "BYTE" if op.starts_with('C') => {
                let len = op.len() - 4;
                address_increment = len as i32;
            }
            "BYTE" if op.starts_with('X') => {
                let len = op.len() - 4;
                address_increment = len as i32 / 2;
            }
            "END" => address_increment = 0,
            _ => {}
        }
    }

    address_increment
}

// initializes the opcodes for the SIC machine
// This WILL be painful to read
fn initalize_opcodes(opcodes_list: &mut Vec<Instruction>) {
    let instructions = vec![
        "ADD", "AND", "COMP", "DIV", "J", "JEQ", "JGT", "JLT", "JSUB", "LDA", "LDCH", "LDL", "LDX",
        "MUL", "OR", "RD", "RSUB", "STA", "STCH", "STL", "STSW", "STX", "SUB", "TD", "TIX", "WD",
    ];
    let opcodes = vec![
        "18", "40", "28", "24", "3C", "30", "34", "38", "48", "00", "50", "08", "04", "20", "44",
        "D8", "4C", "0C", "54", "14", "E8", "10", "1C", "E0", "2C", "DC",
    ];

    // because instructions and opcodes are both the same length,
    // we just need the length of one for this loop to connect them
    for i in 0..instructions.len() {
        opcodes_list.push(Instruction::new(
            instructions[i],
            i32::from_str_radix(opcodes[i], 16).ok().unwrap(),
        ));
    }
}

/* Logic for the Rusty SIC Assembler
*/
mod symbols;

use std::{
    env,
    fs::File,
    io::{BufRead, BufReader},
};
use symbols::Symbol;

const MAX_MEMORY: i32 = 0x7FFF;
// Holds inputted args (for now, just filename)
pub struct Config {
    filename: String,
}

impl Config {
    pub fn new(mut args: env::Args) -> Result<Config, &'static str> {
        args.next(); // discard the program itself

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
    pub fn new(sym: String, dir: String, op: String) -> Self {
        AssemblyLine {
            symbol: Some(sym),
            directive: Some(dir),
            operand: Some(op),
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

    // main loop
    /*
     * The first pass of the Assembler generates the symbol table
     * The Symbol Table consists of two parts: The symbol itself, and the address in memory
     * The rule of thumb is that each line increases the address by 0x03, but certain directives
     * and instructions can change that.
     * Those are as follows;
     * * START: Sets the initial address
     * * RESB: Adds the operand to the address (as its reserving bytes, not words)
     * * RESW: Adds 3 times the operand to the address (a word in SIC is 3 bytes)
     * * BYTE: Varies depending on whether it is a char or a hex byte. Hex BYTE
     * directives are 1 byte per two hex characters (BYTE can only take even hex values).
     * Char BYTE is 1 byte per character (following ASCII).
     * * Every Instruction is 3 bytes (even the instructions that take no operands, like RSUB)
     * * END does not add to the address.
     */
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
            if !(is_symbol_line(&buffer, &mut address_counter)) {
                // not a symbol line
                continue 'pass1;
            }

            // add the symbol to the symbol table
            let broken_line: Vec<&str> = buffer.split_ascii_whitespace().collect();
            let line = AssemblyLine::new(
                broken_line[0].to_string(),
                broken_line[1].to_string(),
                broken_line[2].to_string(),
            );

            if line.directive().unwrap() == "START" {
                address_counter += line.operand.unwrap().parse::<i32>().unwrap();
                continue;
            }

            if let Some(symbol_name) = line.symbol() {
                symbol_table.push(Symbol::new(symbol_name.to_string(), address_counter));
            }

            address_counter += 3;
        }
    }

    // print the symbol table
    for entry in symbol_table {
        println!("{}\t{:X}", entry.name(), entry.address());
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
fn is_symbol_line(buffer: &str, counter: &mut i32) -> bool {
    if buffer.starts_with('\t') {
        *counter += 3;
    } else if buffer.starts_with('#') {
        // do nothing
    } else {
        return true;
    }

    false
}

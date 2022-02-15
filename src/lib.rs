/* Logic for the Rusty SIC Assembler
*/

/* Big list of TODO
* Error Handling in other places
* * Maybe propagate it up to main.rs?
* Records aren't being given to the ObjectData struct correctly.
*/

mod data_records;
mod directives;
mod instructions;
mod symbols;

use crate::data_records::ModRecordData;
use ascii_to_hex::ascii_to_hex;
use data_records::ObjectData;
use directives::is_directive;
use instructions::Instruction;
use std::{
    env,
    fs::{self, File},
    io::{BufRead, BufReader, Result as ioResult, Seek, Write},
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

    let sic_asm_file = File::open(&config.filename);

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
    }

    // pass 2 loop: Creating the records
    let mut starting_address: Option<i32> = None; // preserve the old starting address
    let _length = 3; // default length of object code
    let mut object_data = ObjectData::new();
    let mut mod_records: Vec<ModRecordData> = vec![];

    reader.rewind().unwrap(); // reset the reader

    'pass2: loop {
        // empty out the buffer
        buffer.clear();
        if let Ok(line) = reader.read_line(&mut buffer) {
            if line == 0 {
                // hit EOF
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
                    write_text_record(
                        &mut object_data,
                        &symbol_table,
                        line.directive().unwrap(),
                        line.operand(),
                    );

                    if !is_directive(line.directive().unwrap())
                        && line.directive().unwrap() != "RSUB"
                    {
                        add_mod_record(
                            &mut mod_records,
                            &starting_address.unwrap(),
                            &4,
                            line.symbol(),
                        );
                    }
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

            address_counter += increment;

            // START and END handling
            if line.directive().unwrap() == "START" {
                // starting_address has not been set yet, meaning
                // this is the first (and only valid) call of START
                if let None = starting_address {
                    starting_address = Some(
                        // Set starting_address
                        i32::from_str_radix(
                            line.operand()
                                .expect("ERROR: Starting address not included."),
                            16,
                        )
                        .expect("ERROR: Starting address is not a valid hex number!"),
                    );
                    // Define an ending address
                    let end_address = address_counter - starting_address.unwrap();
                    // write the head record
                    write_head_record(
                        &mut object_data,
                        line.symbol().expect("ERROR: No program name included."),
                        &end_address,
                        starting_address.unwrap(),
                    );
                } else {
                    // starting address has been defined already, which means START has been called
                    // twice!
                    return Err("ERROR: Starting address was already defined!\n Maybe you called START twice?");
                }
            // The first (and only valid call) of END
            } else if line.directive().unwrap() == "END" {
                // In theory, END comes after START
                if let Some(start_address) = starting_address {
                    write_end_record(&mut object_data, &start_address);
                    write_mod_record(&mut object_data, &mut mod_records);
                    // write to file
                    match write_to_file(&object_data, config.filename().to_string()) {
                        Ok(_) => {
                            break;
                        }
                        Err(_) => return Err("Error writing to file."),
                    }
                } else {
                    return Err(
                        // but sometimes humans err, and that's why we handle such cases
                        "ERROR: Starting Address not assigned.\n Maybe you didn't use START?",
                    );
                }
            }

            // write text records
            write_text_record(
                &mut object_data,
                &symbol_table,
                line.directive().unwrap(),
                line.operand(),
            );
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
    object_data: &mut ObjectData,
    symtable: &Vec<Symbol>,
    directive: &str,
    operand: Option<&str>,
) {
    // this function wil return text_data
    let mut text_data: String = String::new();
    let mut symbol_address = 0;

    // The operand does not exit only when the directive is RSUB
    if let Some(operand) = operand {
        // need to locate the symbol in the symbol table
        // as well as the instruction
        let symbol = find_symbol(symtable, operand);
        let opcode = find_instruction(directive, operand);

        // If we didn't find the symbol, we need to error
        if let Some(symbol) = symbol {
            // we found the symbol
            // need to set the symbol address
            symbol_address = *symbol.address();
        }

        if let Some(instruction) = opcode {
            if !is_directive(directive) {
                // instruction, so the object code is OP and ADDR
                text_data = format!("{:02X}{:04X}", instruction.opcode(), symbol_address);
            } else {
                // It's just a directive
                // BYTE directives can exceed the 60 character object code limit
                // Please look forward to it (tm)
                if operand.len() > 60 {
                    todo!(); // please do look forward to it (tm)
                } else {
                    if instruction.name() == "WORD" {
                        // word format is %06X
                        text_data = format!("{:06X}", instruction.opcode());
                    } else {
                        // This was uncommented in C Code
                        // Dangit past me
                        text_data = format!("{}", operand);
                    }
                }
            }
        }
    }
    object_data.add_text_records(format!("T{}\n", text_data));
}

// writes head record
fn write_head_record(
    object_data: &mut ObjectData,
    start_symbol: &str,
    start_address: &i32,
    length: i32,
) {
    object_data.set_head_record(format!(
        "H{}{:06X}{:06X}\n",
        start_symbol, start_address, length
    ));
}

// Writes end record
fn write_end_record(object_data: &mut ObjectData, start_address: &i32) {
    object_data.set_end_record(format!("E{:06X}\n", start_address));
}

/*
* Writes the modification records
* If I recall correctly, modification records
* happen on every instruction that isn't RSUB.
* At least, that's what I can gather from my
* poorly commented C code.
* I forget the reasoning why, but there's two mod record functions
* I may be able to condense them down with Rust powers.
*/
// This takes a collection of records, and fills them out
fn add_mod_record(
    mod_records: &mut Vec<ModRecordData>,
    starting_address: &i32,
    mod_length: &i32,
    symbol: Option<&str>,
) {
    if let Some(symbol) = symbol {
        mod_records.push(ModRecordData::new(
            *starting_address,
            *mod_length,
            symbol.to_string(),
        ));
    }
}

fn write_mod_record(object_data: &mut ObjectData, mod_records: &mut Vec<ModRecordData>) {
    for record in mod_records {
        object_data.add_mod_records(format!(
            "M{:06X}{:02X}+{}\n",
            record.starting_address(),
            record.mod_length(),
            record.symbol()
        ));
    }
}

fn find_symbol<'a>(symtable: &'a Vec<Symbol>, operand: &str) -> Option<&'a Symbol> {
    let found_symbol = symtable.iter().position(|r| r.name() == operand);

    if let Some(index) = found_symbol {
        return symtable.get(index);
    }

    None
}
fn find_instruction<'a>(directive: &'a str, operand: &'a str) -> Option<Instruction<'a>> {
    if is_directive(directive) {
        match directive {
            "RESB" | "RESW" => {
                // This can be removed in favor of the wildcard
                // do nothing
            }
            "WORD" => {
                return Some(Instruction::new(
                    directive,
                    i32::from_str_radix(operand, 10).unwrap(),
                ));
            }
            "BYTE" if operand.starts_with('X') => {
                let len = operand.len() - 1;
                let clean_operand = &operand[2..len];
                return Some(Instruction::new(
                    directive,
                    i32::from_str_radix(clean_operand, 16).unwrap(),
                ));
            }
            "BYTE" if operand.starts_with('C') => {
                // Remove the stuff unneeded from the operand
                let len = operand.len() - 1;
                let clean_operand = &operand[2..len];
                return Some(Instruction::new(
                    directive,
                    <i32>::from_str_radix(ascii_to_hex::get_hex_string(clean_operand).as_str(), 16)
                        .unwrap(),
                ));
            }
            _ => {}
        }
    }
    None
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

fn write_to_file(object_data: &ObjectData, filename: String) -> ioResult<()> {
    let mut output_file = fs::File::create(format!("{}.obj", filename))?;
    output_file.write_all(object_data.head_record().as_bytes())?;
    print!("{}", object_data.head_record());
    for t_record in object_data.text_records() {
        output_file.write_all(t_record.as_bytes())?;
        print!("{}", t_record);
    }
    for m_record in object_data.mod_records() {
        output_file.write_all(m_record.as_bytes())?;
        print!("{}", m_record);
    }
    output_file.write_all(object_data.end_record().as_bytes())?;
    print!("{}", object_data.end_record());
    Ok(())
}

// Data Record structs and methods
#[allow(dead_code)]
#[derive(Default, Debug)]
pub struct ObjectData {
    head_record: String,
    end_record: String,
    text_records: Vec<String>,
    mod_records: Vec<String>,
}

impl ObjectData {
    pub fn new() -> Self {
        Self {
            head_record: String::new(),
            end_record: String::new(),
            text_records: Vec::new(),
            mod_records: Vec::new(),
        }
    }
    pub fn head_record(&self) -> &str {
        &self.head_record
    }
    pub fn end_record(&self) -> &str {
        &self.end_record
    }
    pub fn mod_records(&self) -> &Vec<String> {
        &self.mod_records
    }
    pub fn text_records(&self) -> &Vec<String> {
        &self.text_records
    }
    pub fn head_record_mut(&mut self) -> &mut str {
        &mut self.head_record
    }
    pub fn end_record_mut(&mut self) -> &mut str {
        &mut self.end_record
    }
    pub fn mod_records_mut(&mut self) -> &mut Vec<String> {
        &mut self.mod_records
    }
    pub fn text_records_mut(&mut self) -> &mut Vec<String> {
        &mut self.text_records
    }
}

pub struct ModRecordData {
    starting_address: i32,
    mod_length: i32,
    symbol: String,
}

impl ModRecordData {
    pub fn new(starting_address: i32, mod_length: i32, symbol: String) -> Self {
        Self {
            starting_address,
            mod_length,
            symbol,
        }
    }

    pub fn starting_address(&self) -> &i32 {
        &self.starting_address
    }
    pub fn mod_length(&self) -> &i32 {
        &self.mod_length
    }
    pub fn symbol(&self) -> &str {
        &self.symbol
    }
}

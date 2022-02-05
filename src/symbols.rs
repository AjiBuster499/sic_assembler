// Symbol structure
#[derive(Debug)]
pub struct Symbol {
    name: String,
    address: i32,
}

impl Symbol {
    // standard new method
    pub fn new(name: String, address: i32) -> Self {
        Symbol { name, address }
    }

    // get name
    pub fn name(&self) -> &str {
        &self.name
    }

    // get address
    pub fn address(&self) -> &i32 {
        &self.address
    }
}

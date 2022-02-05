pub struct Instruction {
    name: String,
    opcode: i32,
}

impl Instruction {
    pub fn new(name: String, opcode: i32) -> Self {
        Instruction { name, opcode }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn opcode(&self) -> &i32 {
        &self.opcode
    }
}

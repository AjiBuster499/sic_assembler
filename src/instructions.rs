pub struct Instruction<'a> {
    name: &'a str,
    opcode: i32,
}

impl<'a> Instruction<'a> {
    pub fn new(name: &'a str, opcode: i32) -> Self {
        Instruction { name, opcode }
    }

    pub fn name(&self) -> &str {
        &self.name
    }

    pub fn opcode(&self) -> &i32 {
        &self.opcode
    }
}

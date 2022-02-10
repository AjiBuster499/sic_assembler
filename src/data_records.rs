// Data Record structs and methods

#[derive(Default, Debug)]
pub struct ObjectData<'a> {
    head_record: &'a str,
    end_record: &'a str,
    text_record: &'a str,
    mod_record: &'a str,
}

impl ObjectData<'_> {
    pub fn head_record(&self) -> &'_ str {
        self.head_record
    }
    pub fn end_record(&self) -> &'_ str {
        self.end_record
    }
    pub fn text_record(&self) -> &'_ str {
        self.text_record
    }
    pub fn mod_record(&self) -> &'_ str {
        self.mod_record
    }
}

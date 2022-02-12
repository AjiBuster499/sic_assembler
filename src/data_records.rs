// Data Record structs and methods
#[allow(dead_code)]
#[derive(Default, Debug)]
pub struct ObjectData<'a> {
    head_record: &'a str,
    end_record: &'a str,
    text_record: &'a str,
    mod_record: &'a str,
}

impl<'a> ObjectData<'a> {
    pub fn new(
        head_record: &'a str,
        end_record: &'a str,
        text_record: &'a str,
        mod_record: &'a str,
    ) -> Self {
        Self {
            head_record,
            end_record,
            text_record,
            mod_record,
        }
    }
}

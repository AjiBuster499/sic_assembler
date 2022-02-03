// Data Record structs and methods

#[derive(Default, Debug)]
pub struct ObjectData<'a> {
    head_record: &'a str,
    end_record: &'a str,
    text_record: &'a str,
    mod_record: &'a str,
}

impl ObjectData<'_> {}

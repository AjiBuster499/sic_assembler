pub fn is_directive(directive: &str) -> bool {
    match directive {
        "START" => {
            return true;
        }
        "END" => {
            return true;
        }
        "RESB" => {
            return true;
        }
        "RESW" => {
            return true;
        }
        "RESR" => {
            return true;
        }
        "BYTE" => {
            return true;
        }
        "WORD" => {
            return true;
        }
        "EXPORTS" => {
            return true;
        }
        _ => {
            return false;
        }
    }
}

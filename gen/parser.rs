fn capfirst(s: &str) -> String {
    s[..1].to_ascii_uppercase() + &s[1..].to_ascii_lowercase()
}

pub struct Code {
    pub code: String,
    pub value: String,
    pub is_reply: bool,
    pub is_error: bool,
    pub format_code: String,
    pub format_value: String,
}

impl Code {
    pub fn from_iter<'a>(mut iter: impl Iterator<Item = &'a str>) -> Vec<Self> {
        let mut codes = Vec::new();
        loop {
            let code = if let Some(code) = iter.next() {
                code.to_string()
            } else {
                break;
            };
            let value = if let Some(value) = iter.next() {
                value.to_string()
            } else {
                break;
            };

            let is_reply = code.starts_with("RPL_");
            let is_error = code.starts_with("ERR_");

            let format_code = Self::format_code(&code, is_reply, is_error);
            let format_value = Self::format_value(&value);

            codes.push(Code {
                code,
                value,
                is_reply,
                is_error,
                format_code,
                format_value,
            });
        }

        codes
    }

    fn format_code(code: &str, is_reply: bool, is_error: bool) -> String {
        if is_reply || is_error {
            let mut parts = code.split('_');
            let left = parts.next().unwrap();
            let right = parts.next().unwrap();
            capfirst(&left) + &capfirst(right)
        } else {
            capfirst(&code)
        }
    }

    fn format_value(value: &str) -> String {
        format!("\"{}\"", value.to_ascii_uppercase())
    }
}

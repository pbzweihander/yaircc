use {
    super::parser::*,
    std::{
        fs::File,
        io::{self, Write},
    },
};

fn gen_header(file: &mut File) -> io::Result<()> {
    writeln!(file, "// This file was generated automatically.")?;
    writeln!(
        file,
        "// See `gen/` folder and `build.rs` at the project root."
    )?;
    writeln!(file)?;
    writeln!(file, "use std::{{convert::Infallible, fmt, str}};")?;

    Ok(())
}

fn gen_enum(file: &mut File, codes: &[Code]) -> io::Result<()> {
    writeln!(
        file,
        "/// Representation of IRC commands, replies and errors."
    )?;
    writeln!(file, "#[derive(Clone, Debug, Eq, PartialEq)]")?;
    writeln!(file, "pub enum Code {{")?;
    for code in codes {
        writeln!(file, "    /// {} = \"{}\"", code.code, code.value)?;
        writeln!(file, "    {},", code.format_code)?;
    }
    writeln!(file, "    /// Codes that are unknown end up in here.")?;
    writeln!(file, "    Unknown(String),")?;
    writeln!(file, "}}")?;

    Ok(())
}

fn gen_methods(file: &mut File, codes: &[Code]) -> io::Result<()> {
    writeln!(file, "impl Code {{")?;
    writeln!(file, "    /// Checks if the code is a reply.")?;
    writeln!(file, "    pub fn is_reply(&self) -> bool {{")?;
    writeln!(file, "        match *self {{")?;
    for code in codes {
        if code.is_reply {
            writeln!(file, "            Code::{} => true,", code.format_code)?;
        }
    }
    writeln!(file, "            _  => false,")?;
    writeln!(file, "        }}")?;
    writeln!(file, "    }}")?;
    writeln!(file)?;
    writeln!(file, "    /// Check if the code is en error.")?;
    writeln!(file, "    pub fn is_error(&self) -> bool {{")?;
    writeln!(file, "        match *self {{")?;
    for code in codes {
        if code.is_error {
            writeln!(file, "            Code::{} => true,", code.format_code)?;
        }
    }
    writeln!(file, "            _  => false,")?;
    writeln!(file, "        }}")?;
    writeln!(file, "    }}")?;
    writeln!(file, "}}")?;

    Ok(())
}

fn gen_display(file: &mut File, codes: &[Code]) -> io::Result<()> {
    writeln!(file, "impl fmt::Display for Code {{")?;
    writeln!(file)?;
    writeln!(
        file,
        "    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {{"
    )?;
    writeln!(file, "        match *self {{")?;
    for code in codes {
        writeln!(
            file,
            "            Code::{} => write!(f, {}),",
            code.format_code, code.format_value
        )?;
    }
    writeln!(
        file,
        "            Code::Unknown(ref text) => write!(f, \"{{}}\", text),"
    )?;
    writeln!(file, "        }}")?;
    writeln!(file, "    }}")?;
    writeln!(file, "}}")?;

    Ok(())
}

fn gen_fromstr(file: &mut File, codes: &[Code]) -> io::Result<()> {
    writeln!(file, "impl str::FromStr for Code {{")?;
    writeln!(file, "    type Err = Infallible;")?;
    writeln!(file)?;
    writeln!(
        file,
        "    fn from_str(s: &str) -> Result<Self, Self::Err> {{"
    )?;
    writeln!(file, "        let code = match s {{")?;
    for code in codes {
        writeln!(
            file,
            "            {} => Code::{},",
            code.format_value, code.format_code
        )?;
    }
    writeln!(file, "            _ => Code::Unknown(s.to_string()),")?;
    writeln!(file, "        }};")?;
    writeln!(file, "        Ok(code)")?;
    writeln!(file, "    }}")?;
    writeln!(file, "}}")?;

    Ok(())
}

pub fn gen_code(codes: &[Code], file: &mut File) -> io::Result<()> {
    gen_header(file)?;
    writeln!(file)?;
    gen_enum(file, codes)?;
    writeln!(file)?;
    gen_methods(file, codes)?;
    writeln!(file)?;
    gen_display(file, codes)?;
    writeln!(file)?;
    gen_fromstr(file, codes)?;

    Ok(())
}

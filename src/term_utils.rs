use std::process::{Command, Stdio};
pub type DimType = u16;

#[derive(Debug)]
pub struct TermSize {
    pub lines: DimType,
    pub cols: DimType,
}

/// Returns the size of the terminal, or None if the size cannot be determined.
pub fn get_termsize() -> Option<TermSize> {
    // Call "stty size" to get output in form of "[LINES] [COLUMNS]\n"
    let output = match Command::new("stty")
        .arg("size")
        .stdin(Stdio::inherit())
        .output() {
            Ok(out) => String::from_utf8(out.stdout).ok(),
            Err(_) => None,
    }?;
    let mut iter = output.split(char::is_whitespace);
    // Parse lines and cols from output
    let lines = iter.next()?.parse().ok()?;
    let cols = iter.next()?.parse().ok()?;
    Some(TermSize{ lines, cols })
}

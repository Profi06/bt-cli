// vim: cc=81
use std::process::{Command, Stdio};
pub type DimType = u16;

/*
#[macro_export]
macro_rules! setter {
    ($setx:ident, $x:ident, $t:ty) => {
        /// Setter function for $x
        pub fn $setx(&mut self, val:$t) {
            self.$x = val;
        }
    };
}
*/

#[macro_export]
macro_rules! dict {
    {$($k:expr => $v:expr),+} => {
        [$(($k, $v)),*].into()
    };
}

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
        .output()
    {
        Ok(out) => String::from_utf8(out.stdout).ok(),
        Err(_) => None,
    }?;
    let mut iter = output.split(char::is_whitespace);
    // Parse lines and cols from output
    let lines = iter.next()?.parse().ok()?;
    let cols = iter.next()?.parse().ok()?;
    Some(TermSize { lines, cols })
}

pub mod ansi {
    pub const ANSI_RESET: &str = "\x1b[0m";
    // pub const ANSI_BLACK: &str = "\x1b[30m";
    pub const ANSI_RED: &str = "\x1b[31m";
    pub const ANSI_GREEN: &str = "\x1b[32m";
    pub const ANSI_YELLOW: &str = "\x1b[33m";
    // pub const ANSI_BLUE: &str = "\x1b[34m";
    // pub const ANSI_MAGENTA: &str = "\x1b[35m";
    // pub const ANSI_CYAN: &str = "\x1b[36m";
    // pub const ANSI_WHITE: &str = "\x1b[37m";
    // pub const ANSI_DEFAULT: &str = "\x1b[39m";
    // pub const ANSI_BLACK_BG: &str = "\x1b[40m";
    // pub const ANSI_RED_BG: &str = "\x1b[41m";
    // pub const ANSI_GREEN_BG: &str = "\x1b[42m";
    // pub const ANSI_YELLOW_BG: &str = "\x1b[43m";
    // pub const ANSI_BLUE_BG: &str = "\x1b[44m";
    // pub const ANSI_MAGENTA_BG: &str = "\x1b[45m";
    // pub const ANSI_CYAN_BG: &str = "\x1b[46m";
    // pub const ANSI_WHITE_BG: &str = "\x1b[47m";
    // pub const ANSI_DEFAULT_BG: &str = "\x1b[49m";
}

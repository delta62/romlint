use std::fmt::Display;
use std::io::{self, Write};

pub fn red(s: impl Display) {
    print!("\x1b[31m{}\x1b[0m", s);
    io::stdout().lock().flush().unwrap();
}

pub fn green(s: impl Display) {
    print!("\x1b[32m{}\x1b[0m", s);
    io::stdout().lock().flush().unwrap();
}

pub fn blue(s: impl Display) {
    print!("\x1b[34m{}\x1b[0m", s);
    io::stdout().lock().flush().unwrap();
}

pub fn move_to_line_start() {
    print!("\x1b[1G");
    io::stdout().lock().flush().unwrap_or_default();
}

pub fn print_status<D: Display>(s: D) {
    print!("\x1b[1G{}", s);
    io::stdout().lock().flush().unwrap_or_default();
}

pub fn clear_line() {
    print!("\x1b[2K");
    io::stdout().lock().flush().unwrap_or_default();
}

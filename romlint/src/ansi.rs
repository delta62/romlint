use std::fmt::Display;
use std::io::{self, Result, Write};

pub fn move_up_lines(lines: usize) -> Result<()> {
    print!("\x1b[{}A", lines);
    io::stdout().lock().flush()?;
    flush()
}

pub fn move_to_line_start() -> Result<()> {
    print!("\x1b[1G");
    flush()
}

pub fn print_status<D: Display>(s: D) -> Result<()> {
    print!("\x1b[1G{}", s);
    flush()
}

pub fn clear_line() -> Result<()> {
    print!("\x1b[2K");
    flush()
}

fn flush() -> Result<()> {
    io::stdout().lock().flush()
}

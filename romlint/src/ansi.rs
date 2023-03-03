use std::fmt::Display;
use std::io::{self, Result, Write};

pub fn move_up_lines(lines: usize) -> Result<()> {
    print!("\x1b[{}A", lines);
    io::stdout().lock().flush()?;
    move_to_line_start()
}

pub fn move_to_line_start() -> Result<()> {
    print!("\x1b[1G");
    io::stdout().lock().flush()
}

pub fn print_status<D: Display>(s: D) -> Result<()> {
    print!("\x1b[1G{}", s);
    io::stdout().lock().flush()
}

pub fn clear_line() -> Result<()> {
    print!("\x1b[2K");
    io::stdout().lock().flush()
}

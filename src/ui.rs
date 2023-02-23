use crate::ansi::{clear_line, green, move_to_line_start, print_status, red};
use crate::linter::Diagnostic;
use std::{sync::mpsc::Receiver, time::Duration};

#[derive(Debug)]
pub struct Report {
    pub diagnostics: Vec<Diagnostic>,
    pub path: String,
}

impl Report {
    fn ok(&self) -> bool {
        self.diagnostics.is_empty()
    }
}

pub enum Message {
    Finished,
    SetStatus(String),
    Report(Report),
}

pub struct Ui {
    channel: Receiver<Message>,
}

impl Ui {
    pub fn new(channel: Receiver<Message>) -> Self {
        Self { channel }
    }

    pub fn run(self) {
        let mut icons = ['⣾', '⣽', '⣻', '⢿', '⡿', '⣟', '⣯', '⣷'].iter().cycle();
        let mut status = "".to_string();

        'outer: loop {
            while let Ok(message) = self.channel.try_recv() {
                clear_line();
                match message {
                    Message::Finished => {
                        clear_line();
                        break 'outer;
                    }
                    Message::SetStatus(s) => status = s,
                    Message::Report(report) => {
                        move_to_line_start();
                        print_report(&report);
                    }
                }
            }

            let message = format!(" {} > {}", icons.next().unwrap(), status);
            print_status(message);
            std::thread::sleep(Duration::from_millis(100));
        }
    }
}

fn print_report(report: &Report) {
    if report.ok() {
        green(format!(" {}\n", report.path));
    } else {
        red(format!("❌{}\n", report.path));
        for (i, diag) in report.diagnostics.iter().enumerate() {
            let last = i == report.diagnostics.len() - 1;

            if last {
                println!("  └─ {}", diag.message);
            } else {
                println!("  ├─ {}", diag.message);
            }

            for hint in &diag.hints {
                println!("       {}", hint);
            }
        }
    }
}

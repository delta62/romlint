use crate::ansi::{clear_line, move_to_line_start, print_status};
use crate::error::{IoErr, Result};
use crate::linter::Diagnostic;
use nu_ansi_term::Color::{Green, Red};
use snafu::prelude::*;
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

    pub fn run(self) -> Result<()> {
        let mut icons = ['⣾', '⣽', '⣻', '⢿', '⡿', '⣟', '⣯', '⣷'].iter().cycle();
        let mut status = "".to_string();

        'outer: loop {
            while let Ok(message) = self.channel.try_recv() {
                clear_line().context(IoErr { path: "stdout" })?;
                match message {
                    Message::Finished => {
                        clear_line().context(IoErr { path: "stdout" })?;
                        break 'outer;
                    }
                    Message::SetStatus(s) => status = s,
                    Message::Report(report) => {
                        move_to_line_start().context(IoErr { path: "stdout" })?;
                        print_report(&report);
                    }
                }
            }

            let message = format!(" {} > {}", icons.next().unwrap(), status);
            print_status(message).context(IoErr { path: "stdout" })?;
            std::thread::sleep(Duration::from_millis(100));
        }

        Ok(())
    }
}

fn print_report(report: &Report) {
    if report.ok() {
        println!(" {}", Green.paint(report.path.as_str()));
    } else {
        println!("{}", Red.paint(format!("❌ {}", report.path.as_str())));
        for (i, diag) in report.diagnostics.iter().enumerate() {
            let last = i == report.diagnostics.len() - 1;

            if last {
                println!("  └─ {}", diag.message);
            } else {
                println!("  ├─ {}", diag.message);
            }

            if let Some(hints) = &diag.hints {
                for hint in hints {
                    if last {
                        println!("       {}", hint);
                    } else {
                        println!("  │    {}", hint);
                    }
                }
            }
        }
    }
}

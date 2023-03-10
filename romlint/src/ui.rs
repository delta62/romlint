use crate::ansi::{clear_line, move_to_line_start, move_up_lines, print_status};
use crate::error::{IoErr, Result};
use crate::linter::Diagnostic;
use nu_ansi_term::Color::{Blue, Green, Red};
use snafu::prelude::*;
use std::collections::HashMap;
use std::time::Instant;
use std::{sync::mpsc::Receiver, time::Duration};

#[derive(Default)]
pub struct SystemSummary {
    pass_count: usize,
    fail_count: usize,
}

pub struct Summary {
    systems: HashMap<String, SystemSummary>,
    total_passes: usize,
    total_fails: usize,
    start_time: Instant,
    end_time: Option<Instant>,
}

impl Summary {
    pub fn new(start_time: Instant) -> Self {
        Self {
            systems: HashMap::new(),
            total_passes: 0,
            total_fails: 0,
            start_time,
            end_time: None,
        }
    }

    pub fn add_success<S: Into<String>>(&mut self, system: S) {
        self.total_passes += 1;
        self.systems.entry(system.into()).or_default().pass_count += 1;
    }

    pub fn add_failure<S: Into<String>>(&mut self, system: S) {
        self.total_fails += 1;
        self.systems.entry(system.into()).or_default().fail_count += 1;
    }

    pub fn mark_ended(&mut self) {
        self.end_time = Some(Instant::now());
    }
}

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
    Finished(Summary),
    SetStatus(String),
    StartProgress(usize, String),
    EndProgress(usize),
    Report(Report),
}

pub struct Ui {
    channel: Receiver<Message>,
    show_passes: bool,
}

impl Ui {
    pub fn new(channel: Receiver<Message>, show_passes: bool) -> Self {
        Self {
            channel,
            show_passes,
        }
    }

    pub fn run(self) -> Result<()> {
        let mut icons = ['⣷', '⣯', '⣟', '⡿', '⢿', '⣻', '⣽', '⣾'].iter().cycle();
        let mut status = "Initializing...".to_string();
        let mut loading_items = Vec::new();
        let mut last_load_printed = false;

        'outer: loop {
            while let Ok(message) = self.channel.try_recv() {
                clear_line().context(IoErr { path: "stdout" })?;
                match message {
                    Message::StartProgress(_id, name) => {
                        println!();
                        loading_items.push((name, true))
                    }
                    Message::EndProgress(id) => {
                        loading_items
                            .get_mut(id)
                            .map(|(_, is_loading)| *is_loading = false);
                    }
                    Message::Finished(summary) => {
                        clear_line().context(IoErr { path: "stdout" })?;
                        println!();
                        print_summary(&summary);
                        break 'outer;
                    }
                    Message::SetStatus(s) => status = s,
                    Message::Report(report) => {
                        move_to_line_start().context(IoErr { path: "stdout" })?;
                        print_report(&report, self.show_passes);
                    }
                }
            }

            let done_loading = loading_items.iter().all(|(_, is_loading)| !is_loading);
            if !done_loading || !last_load_printed {
                print_loading_status(&loading_items);
            }

            last_load_printed = done_loading;

            let message = format!("{} >> {}", icons.next().unwrap(), Blue.paint(&status));
            print_status(message).context(IoErr { path: "stdout" })?;
            std::thread::sleep(Duration::from_millis(100));
        }

        Ok(())
    }
}

fn print_loading_status(items: &[(String, bool)]) {
    move_up_lines(items.len()).unwrap();
    for (system_name, is_loading) in items {
        let done_text = if *is_loading { "" } else { "✓" };
        println!(
            "Loading {} rom db... {}",
            system_name,
            Green.paint(done_text)
        );
    }
}

fn print_summary(summary: &Summary) {
    let duration = summary.end_time.unwrap().duration_since(summary.start_time);
    let duration = format_duration(&duration);

    println!("          Passed   Failed");
    for (system, summary) in &summary.systems {
        println!(
            "{:10}{:6}   {:6}",
            system,
            Green.paint(format!("{:6}", summary.pass_count)),
            Red.paint(format!("{:6}", summary.fail_count))
        );
    }
    println!("-------------------------");
    println!(
        "Total     {:6}   {:6}",
        Green.paint(format!("{:6}", summary.total_passes)),
        Red.paint(format!("{:6}", summary.total_fails)),
    );
    println!();
    println!(
        "Scanned {} items in {}",
        summary.total_passes + summary.total_fails,
        Blue.paint(duration)
    );
}

fn format_duration(duration: &Duration) -> String {
    let secs = duration.as_secs();
    if secs >= 60 {
        format!("{}m {:02}s", secs / 60, secs % 60)
    } else {
        let millis = duration.as_millis();
        format!("{}.{:03}s", millis / 1000, millis % 1000)
    }
}

fn print_report(report: &Report, show_passes: bool) {
    if report.ok() {
        if show_passes {
            println!("{}", Green.paint(format!(" {}", report.path.as_str())));
        }
    } else {
        println!("{}", Red.paint(format!("❌ {}", report.path.as_str())));
        for (i, diag) in report.diagnostics.iter().enumerate() {
            let last = i == report.diagnostics.len() - 1;

            if last {
                println!("   └─ {}", diag.message);
            } else {
                println!("   ├─ {}", diag.message);
            }

            if let Some(hints) = &diag.hints {
                for hint in hints {
                    if last {
                        println!("        {}", hint);
                    } else {
                        println!("   │    {}", hint);
                    }
                }
            }
        }
    }
}

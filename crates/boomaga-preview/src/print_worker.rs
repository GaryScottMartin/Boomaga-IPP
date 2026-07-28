//! Background CUPS command bridge for printer discovery and submission.
use crate::app::AppData;
use crate::submission_plan::SubmissionPlan;
use boomaga_core::{DuplexMode, PrintOptions};
use std::{
    path::{Path, PathBuf},
    process::Command,
};
use tokio::sync::mpsc::{UnboundedReceiver, UnboundedSender};
use xilem::{
    core::{MessageProxy, NoElement, View},
    view::worker,
    ViewCtx,
};

#[derive(Debug)]
pub enum PrintCommand {
    Discover,
    Submit {
        printer: String,
        document: PathBuf,
        page_count: usize,
        options: PrintOptions,
    },
}
#[derive(Debug, PartialEq, Eq)]
pub enum PrintEvent {
    Printers(Vec<String>),
    Submitted(String),
    Failed(String),
}
pub type PrintSender = UnboundedSender<PrintCommand>;

pub fn print_worker() -> impl View<AppData, (), ViewCtx, Element = NoElement> {
    worker(
        run_worker,
        |d: &mut AppData, s| d.install_print_worker(s),
        |d: &mut AppData, e| d.handle_print_event(e),
    )
}
async fn run_worker(proxy: MessageProxy<PrintEvent>, receiver: UnboundedReceiver<PrintCommand>) {
    let failed = proxy.clone();
    if let Err(error) = std::thread::Builder::new()
        .name("boomaga-cups-client".into())
        .spawn(move || run_loop(proxy, receiver))
    {
        let _ = failed.message(PrintEvent::Failed(format!(
            "failed to start print worker: {error}"
        )));
    }
}
fn run_loop(proxy: MessageProxy<PrintEvent>, mut receiver: UnboundedReceiver<PrintCommand>) {
    while let Some(command) = receiver.blocking_recv() {
        let event = match command {
            PrintCommand::Discover => run_discovery(),
            PrintCommand::Submit {
                printer,
                document,
                page_count,
                options,
            } => run_submit(&printer, &document, page_count, &options),
        };
        if proxy.message(event).is_err() {
            break;
        }
    }
}
fn run_discovery() -> PrintEvent {
    match Command::new("lpstat").arg("-p").output() {
        Ok(out) if out.status.success() => PrintEvent::Printers(parse_printers(&out.stdout)),
        Ok(out) => PrintEvent::Failed(command_error("lpstat", &out.stderr)),
        Err(error) => PrintEvent::Failed(format!("unable to run lpstat: {error}")),
    }
}
fn run_submit(
    printer: &str,
    document: &Path,
    page_count: usize,
    options: &PrintOptions,
) -> PrintEvent {
    let plan = match SubmissionPlan::new(page_count, options) {
        Ok(plan) => plan,
        Err(error) => return PrintEvent::Failed(error.to_string()),
    };
    let total = plan.jobs.len();
    let mut responses = Vec::with_capacity(total);
    for (index, job) in plan.jobs.iter().enumerate() {
        let batch = PrintOptions {
            copies: job.copies,
            collate: false,
            page_range: Some((*job.page_range.start(), *job.page_range.end())),
            ..options.clone()
        };
        match Command::new("lp")
            .args(lp_arguments(printer, document, &batch))
            .output()
        {
            Ok(out) if out.status.success() => {
                responses.push(String::from_utf8_lossy(&out.stdout).trim().to_owned());
            }
            Ok(out) => {
                return PrintEvent::Failed(format!(
                    "submitted {index} of {total} copies; {}",
                    command_error("lp", &out.stderr)
                ));
            }
            Err(error) => {
                return PrintEvent::Failed(format!(
                    "submitted {index} of {total} copies; unable to run lp: {error}"
                ));
            }
        }
    }
    PrintEvent::Submitted(responses.join(" · "))
}
fn command_error(command: &str, stderr: &[u8]) -> String {
    let detail = String::from_utf8_lossy(stderr);
    format!(
        "{command} failed{}",
        if detail.trim().is_empty() {
            String::new()
        } else {
            format!(": {}", detail.trim())
        }
    )
}
fn parse_printers(output: &[u8]) -> Vec<String> {
    String::from_utf8_lossy(output)
        .lines()
        .filter_map(|line| line.strip_prefix("printer "))
        .filter_map(|line| line.split_whitespace().next())
        .map(str::to_owned)
        .collect()
}
fn lp_arguments(printer: &str, document: &Path, options: &PrintOptions) -> Vec<String> {
    let sides = match options.duplex {
        DuplexMode::None => "one-sided",
        DuplexMode::LongEdge => "two-sided-long-edge",
        DuplexMode::ShortEdge => "two-sided-short-edge",
    };
    let mut args = vec![
        "-d".into(),
        printer.into(),
        "-n".into(),
        options.copies.to_string(),
        "-o".into(),
        format!("collate={}", if options.collate { "true" } else { "false" }),
        "-o".into(),
        format!("sides={sides}"),
        "-o".into(),
        format!("number-up={}", options.pages_per_sheet as u8),
        "-o".into(),
        format!("scaling={:.0}", options.scale * 100.0),
    ];
    if let Some((first, last)) = options.page_range {
        args.extend(["-P".into(), format!("{first}-{last}")]);
    }
    args.push(document.to_string_lossy().into_owned());
    args
}

#[cfg(test)]
mod tests {
    use super::*;
    use boomaga_core::PagesPerSheet;
    #[test]
    fn parses_lpstat_output() {
        assert_eq!(
            parse_printers(b"printer Office is idle\nprinter PDF disabled\n"),
            ["Office", "PDF"]
        );
    }
    #[test]
    fn maps_options_to_lp_arguments() {
        let options = PrintOptions {
            copies: 3,
            collate: true,
            duplex: DuplexMode::LongEdge,
            pages_per_sheet: PagesPerSheet::Four,
            page_range: Some((2, 7)),
            ..PrintOptions::default()
        };
        let args = lp_arguments("Office", Path::new("doc.pdf"), &options);
        for expected in [
            "Office",
            "3",
            "collate=true",
            "sides=two-sided-long-edge",
            "number-up=4",
            "2-7",
            "doc.pdf",
        ] {
            assert!(args.contains(&expected.to_owned()));
        }
    }
}

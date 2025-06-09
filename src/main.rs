use std::path::Path;

use clap::Parser;
use serde::{Deserialize, Serialize};

use crate::{bench_settings::IoMethodSettings, report::ReportCommand, run_benchmark::RunCommand};

mod bench_settings;
mod buffered_io;
mod direct_async_io;
mod direct_io;
mod direct_io_uring;
mod io_data;

mod report;
mod run_benchmark;

fn main() {
    let args = Args::parse();
    match &args.command {
        Command::Run(run_command) => run_benchmark::run_benchmark(run_command),
        Command::Report(report_command) => report::run_report(report_command),
    }
}

#[derive(Debug, Clone, clap::Parser)]
struct Args {
    #[clap(subcommand)]
    command: Command,
}

#[derive(Debug, Clone, clap::Subcommand)]
enum Command {
    Run(RunCommand),
    Report(ReportCommand),
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum IoSequence {
    Sequential,
    Random,
}

pub trait IoMethod {
    fn write_file(&self, path: &Path, file_size: u64, sequence: IoSequence);
    fn read_file(&self, path: &Path, file_size: u64, sequence: IoSequence);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ReportItem {
    method: IoMethodSettings,
    sequence: IoSequence,
    write_tput_mbps: f64,
    read_tput_mbps: f64,
}

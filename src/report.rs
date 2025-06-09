use minijinja::{Environment, context};

use crate::ReportItem;
use std::{cmp::Reverse, path::PathBuf};

#[derive(Debug, Clone, clap::Args)]
pub struct ReportCommand {
    #[clap(long, value_parser, default_value = "target/report.json")]
    report_file: PathBuf,
    #[clap(long, value_parser, default_value = "target/report.html")]
    report_html_file: PathBuf,
}

pub fn run_report(report_command: &ReportCommand) {
    let mut report_items: Vec<ReportItem> =
        serde_json::from_slice(&std::fs::read(&report_command.report_file).unwrap()).unwrap();
    report_items.sort_by_key(|ri| {
        Reverse((
            ri.method.block_size(),
            ri.sequence,
            float_ord::FloatOrd(ri.write_tput_mbps + ri.read_tput_mbps),
        ))
    });
    let mut env = Environment::new();
    let template = std::fs::read_to_string("report.jinja.html").unwrap();
    env.add_template("report", &template).unwrap();
    let tmpl = env.get_template("report").unwrap();
    let html = tmpl.render(context!(report_items => report_items)).unwrap();
    std::fs::write(&report_command.report_html_file, html).unwrap();
}

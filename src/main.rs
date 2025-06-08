use std::{
    fs::File,
    io::ErrorKind,
    path::Path,
    time::{Duration, Instant},
};

use serde::{Deserialize, Serialize};

use crate::bench_settings::{IoMethodSettings, read_bench_settings};

mod bench_settings;
mod buffered_io;
mod direct_async_io;
mod direct_io;
mod io_data;

fn main() {
    let settings = read_bench_settings();
    println!("{settings:?}");
    let mut report_items = Vec::<ReportItem>::new();
    for m in &settings.methods {
        for sequence in [
            IoSequence::Sequential,
            IoSequence::Random,
            // IoSequence::Sequential,
        ] {
            let duration = measure_write_file(settings.file_size, m, IoSequence::Sequential);
            let write_tput_mbps =
                settings.file_size as f64 / 1024.0 / 1024.0 / duration.as_secs_f64();
            println!(
                "{m:?} {sequence:?} => {d:.3} sec {write_tput_mbps:.2} MiB/sec",
                d = duration.as_secs_f64()
            );
            report_items.push(ReportItem {
                method: m.clone(),
                sequence,
                write_tput_mbps,
            });
        }
    }

    std::fs::write(
        "target/report.json",
        serde_json::to_string(&report_items).unwrap(),
    )
    .unwrap();
}

fn measure_write_file(
    file_size: u64,
    io_method: &IoMethodSettings,
    sequence: IoSequence,
) -> Duration {
    let path = Path::new("target").join("test_file");
    remove_file_maybe(&path);
    let file = File::create_new(&path).unwrap();
    file.set_len(file_size).unwrap();
    file.sync_all().unwrap();
    drop(file);
    let start = Instant::now();
    let mut iters = 0;
    while iters <= 10 && start.elapsed() < Duration::from_secs(3) {
        match io_method {
            IoMethodSettings::Buffered(buffered) => buffered.write_file(&path, file_size, sequence),
            IoMethodSettings::Direct(direct) => direct.write_file(&path, file_size, sequence),
            IoMethodSettings::DirectAsync(direct_async) => {
                direct_async.write_file(&path, file_size, sequence)
            }
            IoMethodSettings::DirectUring(_direct_uring) => todo!(),
        }
        iters += 1;
    }
    let duration = start.elapsed() / iters;
    remove_file_maybe(&path);
    duration
}

fn remove_file_maybe(path: &std::path::PathBuf) {
    match std::fs::remove_file(path) {
        Ok(()) => {}
        Err(e) if e.kind() == ErrorKind::NotFound => {}
        Err(e) => panic!("error: {e}"),
    }
}

#[derive(Clone, Copy, Debug, Serialize, Deserialize)]
pub enum IoSequence {
    Sequential,
    Random,
}

pub trait IoMethod {
    fn write_file(&self, path: &Path, file_size: u64, sequence: IoSequence);
    // TODO: `echo 3 > /proc/sys/vm/drop_caches`
    // fn read_file(&self, path: &Path, sequence: IoSequence);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ReportItem {
    method: IoMethodSettings,
    sequence: IoSequence,
    write_tput_mbps: f64,
}

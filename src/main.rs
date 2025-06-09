use std::{
    fs::File,
    io::ErrorKind,
    path::Path,
    process::Command,
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
    let mut report_items = Vec::<ReportItem>::new();
    drop_caches();
    for m in &settings.methods {
        for sequence in [
            IoSequence::Sequential,
            IoSequence::Random,
            // IoSequence::Sequential,
        ] {
            let path = Path::new("target").join("test_file");
            let write_duration =
                measure_write_file(&path, settings.file_size, m, IoSequence::Sequential);
            let write_tput_mbps =
                settings.file_size as f64 / 1024.0 / 1024.0 / write_duration.as_secs_f64();
            println!(
                "write {m:?} {sequence:?} => {d:.3} sec {write_tput_mbps:.2} MiB/sec",
                d = write_duration.as_secs_f64()
            );
            let read_duration =
                measure_read_file(&path, settings.file_size, m, IoSequence::Sequential);
            let read_tput_mbps =
                settings.file_size as f64 / 1024.0 / 1024.0 / read_duration.as_secs_f64();
            println!(
                "read {m:?} {sequence:?} => {d:.3} sec {read_tput_mbps:.2} MiB/sec",
                d = read_duration.as_secs_f64()
            );
            report_items.push(ReportItem {
                method: m.clone(),
                sequence,
                write_tput_mbps,
                read_tput_mbps,
            });
            remove_file_maybe(&path);
        }
    }

    std::fs::write(
        "target/report.json",
        serde_json::to_string(&report_items).unwrap(),
    )
    .unwrap();
}

fn measure_write_file(
    path: &Path,
    file_size: u64,
    io_method: &IoMethodSettings,
    sequence: IoSequence,
) -> Duration {
    remove_file_maybe(path);
    let file = File::create_new(path).unwrap();
    file.set_len(file_size).unwrap();
    file.sync_all().unwrap();
    drop(file);
    let start = Instant::now();
    let mut iters = 0;
    while iters <= 10 && start.elapsed() < Duration::from_secs(3) {
        match io_method {
            IoMethodSettings::Buffered(buffered) => buffered.write_file(path, file_size, sequence),
            IoMethodSettings::Direct(direct) => direct.write_file(path, file_size, sequence),
            IoMethodSettings::DirectAsync(direct_async) => {
                direct_async.write_file(path, file_size, sequence)
            }
            IoMethodSettings::DirectUring(_direct_uring) => todo!(),
        }
        iters += 1;
    }

    start.elapsed() / iters
}

fn measure_read_file(
    path: &Path,
    file_size: u64,
    io_method: &IoMethodSettings,
    sequence: IoSequence,
) -> Duration {
    let start = Instant::now();
    let mut iters = 0;
    while iters <= 10 && start.elapsed() < Duration::from_secs(3) {
        drop_caches();
        match io_method {
            IoMethodSettings::Buffered(buffered) => buffered.read_file(path, file_size, sequence),
            IoMethodSettings::Direct(direct) => direct.read_file(path, file_size, sequence),
            IoMethodSettings::DirectAsync(direct_async) => {
                direct_async.read_file(path, file_size, sequence)
            }
            IoMethodSettings::DirectUring(_direct_uring) => todo!(),
        }
        iters += 1;
    }

    start.elapsed() / iters
}

fn drop_caches() {
    let rc = Command::new("sudo")
    .args(["sh", "-c", "sync && (echo 3 > /proc/sys/vm/drop_caches) && sync && (echo 3 > /proc/sys/vm/drop_caches)"])
    .spawn().unwrap().wait().unwrap();
    assert!(rc.success());
}

fn remove_file_maybe(path: &Path) {
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
    fn read_file(&self, path: &Path, file_size: u64, sequence: IoSequence);
}

#[derive(Debug, Clone, Serialize, Deserialize)]
struct ReportItem {
    method: IoMethodSettings,
    sequence: IoSequence,
    write_tput_mbps: f64,
    read_tput_mbps: f64,
}

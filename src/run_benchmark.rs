use std::{
    fs::File,
    io::ErrorKind,
    path::{Path, PathBuf},
    time::{Duration, Instant},
};

use crate::{
    IoMethod, IoSequence, ReportItem,
    bench_settings::{IoMethodSettings, read_bench_settings},
};

#[derive(Debug, Clone, clap::Args)]
pub struct RunCommand {
    #[clap(long, value_parser, default_value = "benchmark.json")]
    pub settings_file: PathBuf,
    #[clap(long, value_parser, default_value = "target/report.json")]
    pub report_file: PathBuf,
}

pub fn run_benchmark(run_command: &RunCommand) {
    let settings = read_bench_settings(run_command);
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
        &run_command.report_file,
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
            IoMethodSettings::BufferedUring(buffered_uring) => {
                buffered_uring.write_file(path, file_size, sequence)
            }
            IoMethodSettings::Direct(direct) => direct.write_file(path, file_size, sequence),
            IoMethodSettings::DirectAsync(direct_async) => {
                direct_async.write_file(path, file_size, sequence)
            }
            IoMethodSettings::DirectUring(direct_uring) => {
                direct_uring.write_file(path, file_size, sequence)
            }
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
    let mut iters = 0;
    let mut duration = Duration::ZERO;
    while iters <= 10 && duration < Duration::from_secs(3) {
        drop_caches();
        let start = Instant::now();
        match io_method {
            IoMethodSettings::Buffered(buffered) => buffered.read_file(path, file_size, sequence),
            IoMethodSettings::BufferedUring(buffered_uring) => {
                buffered_uring.read_file(path, file_size, sequence)
            }
            IoMethodSettings::Direct(direct) => direct.read_file(path, file_size, sequence),
            IoMethodSettings::DirectAsync(direct_async) => {
                direct_async.read_file(path, file_size, sequence)
            }
            IoMethodSettings::DirectUring(direct_uring) => {
                direct_uring.read_file(path, file_size, sequence);
            }
        }
        duration += start.elapsed();
        iters += 1;
    }

    duration / iters
}

fn drop_caches() {
    use std::process::Command;
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

use std::path::PathBuf;

use clap::Parser;
use serde::{Deserialize, Serialize};

use crate::{buffered_io::Buffered, direct_async_io::DirectAsync, direct_io::Direct};

pub fn read_bench_settings() -> BenchSettings {
    let args = Args::parse();
    let json = std::fs::read(&args.settings_file).unwrap();
    serde_json::from_slice(&json).unwrap()
}

#[derive(Debug, Clone, clap::Parser)]
struct Args {
    #[clap(long, value_parser, default_value = "benchmark.json")]
    settings_file: PathBuf,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BenchSettings {
    pub file_size: u64,
    pub methods: Vec<IoMethodSettings>,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum IoMethodSettings {
    Buffered(Buffered),
    Direct(Direct),
    DirectAsync(DirectAsync),
    DirectUring(DirectUring),
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DirectUring {
    pub block_size: u32,
    pub concurrency: u32,
}

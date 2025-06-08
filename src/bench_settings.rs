use serde::{Deserialize, Serialize};

use crate::{buffered_io::Buffered, direct_async_io::DirectAsync, direct_io::Direct};

pub fn read_bench_settings() -> BenchSettings {
    let json = std::fs::read("benchmark.json").unwrap();
    serde_json::from_slice(&json).unwrap()
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

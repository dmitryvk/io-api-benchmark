use serde::{Deserialize, Serialize};

use crate::{
    buffered_io::Buffered, buffered_io_uring::BufferedUring, direct_async_io::DirectAsync,
    direct_io::Direct, direct_io_uring::DirectUring, run_benchmark::RunCommand,
};

pub fn read_bench_settings(args: &RunCommand) -> BenchSettings {
    let json = std::fs::read(&args.settings_file).unwrap();
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
    BufferedUring(BufferedUring),
    Direct(Direct),
    DirectAsync(DirectAsync),
    DirectUring(DirectUring),
}

impl IoMethodSettings {
    pub(crate) fn block_size(&self) -> u32 {
        match self {
            IoMethodSettings::Buffered(buffered) => buffered.block_size,
            IoMethodSettings::BufferedUring(buffered_uring) => buffered_uring.block_size,
            IoMethodSettings::Direct(direct) => direct.block_size,
            IoMethodSettings::DirectAsync(direct_async) => direct_async.block_size,
            IoMethodSettings::DirectUring(direct_uring) => direct_uring.block_size,
        }
    }
}

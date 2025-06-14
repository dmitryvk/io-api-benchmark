use std::{
    fs::OpenOptions,
    io::Write,
    os::{fd::AsRawFd, unix::fs::OpenOptionsExt},
    sync::{Arc, LazyLock, Mutex},
};

use aiofut::{AIOBuilder, AIOManager};
use futures::StreamExt;
use libc::O_DIRECT;
use serde::{Deserialize, Serialize};
use tokio::runtime::Runtime;

use crate::{
    IoMethod, IoSequence,
    io_data::{access_seq, aligned_vec, buf_data},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct DirectAsync {
    pub block_size: u32,
    pub concurrency: u32,
}

static TOKIO_RUNTIME: LazyLock<Runtime> = LazyLock::new(create_runtime);
static AIO_MGR: LazyLock<Mutex<AIOManager>> = LazyLock::new(create_aiomgr);

fn create_runtime() -> Runtime {
    tokio::runtime::Builder::new_multi_thread()
        .enable_all()
        .build()
        .expect("Failed building the Runtime")
}

fn create_aiomgr() -> Mutex<AIOManager> {
    Mutex::new(AIOBuilder::default().build().unwrap())
}

impl IoMethod for DirectAsync {
    fn write_file(&self, path: &std::path::Path, file_size: u64, sequence: IoSequence) {
        TOKIO_RUNTIME.block_on(self.write_file_inner(path, file_size, sequence))
    }
    fn read_file(&self, path: &std::path::Path, file_size: u64, sequence: IoSequence) {
        TOKIO_RUNTIME.block_on(self.read_file_inner(path, file_size, sequence))
    }
}

impl DirectAsync {
    async fn write_file_inner(&self, path: &std::path::Path, file_size: u64, sequence: IoSequence) {
        assert_eq!(file_size % self.block_size as u64, 0);
        let template_buf = Arc::new(buf_data(self.block_size as usize));
        let num_pages = file_size / self.block_size as u64;
        let mut file = OpenOptions::new()
            .write(true)
            .read(true)
            .custom_flags(O_DIRECT)
            .open(path)
            .unwrap();
        let fd = file.as_raw_fd();

        let block_pool = Arc::new(Mutex::new(Vec::<Box<[u8]>>::new()));

        let _results: Vec<()> = futures::stream::iter(access_seq(sequence, num_pages))
            .map(|page_idx| {
                let template_buf = template_buf.clone();
                let block_pool = block_pool.clone();
                async move {
                    let offset = page_idx * self.block_size as u64;
                    let buf = {
                        let mut pool = block_pool.lock().unwrap();
                        if let Some(buf) = pool.pop() {
                            buf
                        } else {
                            let mut buf = aligned_vec(template_buf.len());
                            buf.copy_from_slice(&template_buf);
                            buf.into_boxed_slice()
                        }
                    };
                    let (rc, buf) = { AIO_MGR.lock().unwrap().write(fd, offset, buf, None) }.await;
                    {
                        let mut pool = block_pool.lock().unwrap();
                        pool.push(buf);
                    }
                    let written = rc.unwrap();
                    assert_eq!(written as u32, self.block_size);
                }
            })
            .buffer_unordered(self.concurrency as usize)
            .collect()
            .await;

        file.flush().unwrap();
        file.sync_all().unwrap();
    }

    async fn read_file_inner(&self, path: &std::path::Path, file_size: u64, sequence: IoSequence) {
        assert_eq!(file_size % self.block_size as u64, 0);
        let num_pages = file_size / self.block_size as u64;
        let mut file = OpenOptions::new()
            .write(true)
            .read(true)
            .custom_flags(O_DIRECT)
            .open(path)
            .unwrap();
        let fd = file.as_raw_fd();

        let _results: Vec<()> = futures::stream::iter(access_seq(sequence, num_pages))
            .map(|page_idx| async move {
                let block_size = self.block_size;
                let offset = page_idx * self.block_size as u64;
                let (rc, _buf) = {
                    AIO_MGR.lock().unwrap().read(
                        fd,
                        offset,
                        aligned_vec(block_size as usize).into_boxed_slice(),
                        None,
                    )
                }
                .await;
                let written = rc.unwrap();
                assert_eq!(written as u32, self.block_size);
            })
            .buffer_unordered(self.concurrency as usize)
            .collect()
            .await;

        file.flush().unwrap();
        file.sync_all().unwrap();
    }
}

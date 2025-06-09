use std::{
    fs::OpenOptions,
    io::Write,
    os::fd::AsRawFd,
    sync::{LazyLock, Mutex},
};

use io_uring::{IoUring, opcode, types};
use itertools::Itertools;
use serde::{Deserialize, Serialize};

use crate::{
    IoMethod,
    io_data::{access_seq, buf_data},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct BufferedUring {
    pub block_size: u32,
    pub concurrency: u32,
}

static URING: LazyLock<Mutex<IoUring>> = LazyLock::new(create_uring);

fn create_uring() -> Mutex<IoUring> {
    Mutex::new(IoUring::new(1024).unwrap())
}

impl IoMethod for BufferedUring {
    fn write_file(&self, path: &std::path::Path, file_size: u64, sequence: crate::IoSequence) {
        assert_eq!(file_size % self.block_size as u64, 0);
        let buf = buf_data(self.block_size as usize);
        let num_pages = file_size / self.block_size as u64;
        let mut file = OpenOptions::new()
            .write(true)
            .read(true)
            // .custom_flags(O_DIRECT)
            .open(path)
            .unwrap();
        let mut remaining_pages = access_seq(sequence, num_pages).collect_vec().into_iter();
        let binding = &*URING;
        let mut uring = binding.lock().unwrap();
        let mut pending_writes = 0;
        for page in (&mut remaining_pages).take(self.concurrency as usize) {
            let entry =
                opcode::Write::new(types::Fd(file.as_raw_fd()), buf.as_ptr(), buf.len() as u32)
                    .offset(page * self.block_size as u64)
                    .build()
                    .user_data(1);
            unsafe {
                // SAFETY: fd and buffer are valid for the duration of the operation
                uring.submission().push(&entry).unwrap();
            }
            pending_writes += 1;
        }
        uring.submit().unwrap();
        while pending_writes > 0 {
            uring.submit().unwrap();
            uring.submit_and_wait(1).unwrap();
            uring.completion().sync();
            while let Some(entry) = { uring.completion().next() } {
                assert_eq!(entry.result(), buf.len() as i32);
                pending_writes -= 1;
                if let Some(page) = remaining_pages.next() {
                    let entry = opcode::Write::new(
                        types::Fd(file.as_raw_fd()),
                        buf.as_ptr(),
                        buf.len() as u32,
                    )
                    .offset(page * self.block_size as u64)
                    .build()
                    .user_data(1);
                    unsafe {
                        // SAFETY: fd and buffer are valid for the duration of the operation
                        uring.submission().push(&entry).unwrap();
                    }
                    pending_writes += 1;
                    uring.submit().unwrap();
                }
            }
        }
        file.flush().unwrap();
        file.sync_all().unwrap();
    }

    fn read_file(&self, path: &std::path::Path, file_size: u64, sequence: crate::IoSequence) {
        assert_eq!(file_size % self.block_size as u64, 0);
        let mut bufs = (0..self.concurrency)
            .map(|_| vec![0u8; self.block_size as usize])
            .collect_vec();
        let mut available_buf_idx = (0..self.concurrency as usize).collect_vec();
        let num_pages = file_size / self.block_size as u64;
        let file = OpenOptions::new()
            .write(true)
            .read(true)
            // .custom_flags(O_DIRECT)
            .open(path)
            .unwrap();
        let mut remaining_pages = access_seq(sequence, num_pages).collect_vec().into_iter();
        let binding = &*URING;
        let mut uring = binding.lock().unwrap();
        let mut pending_reads = 0;
        for page in (&mut remaining_pages).take(self.concurrency as usize) {
            let buf_idx = available_buf_idx.pop().unwrap();
            let buf = &mut bufs[buf_idx];
            let entry = opcode::Read::new(
                types::Fd(file.as_raw_fd()),
                buf.as_mut_ptr(),
                buf.len() as u32,
            )
            .offset(page * self.block_size as u64)
            .build()
            .user_data(buf_idx as u64);
            unsafe {
                // SAFETY: fd and buffer are valid for the duration of the operation
                uring.submission().push(&entry).unwrap();
            }
            pending_reads += 1;
        }
        uring.submit().unwrap();
        while pending_reads > 0 {
            uring.submit().unwrap();
            uring.submit_and_wait(1).unwrap();
            uring.completion().sync();
            while let Some(entry) = { uring.completion().next() } {
                assert_eq!(entry.result(), self.block_size as i32);
                let buf_idx = entry.user_data() as usize;
                let buf = &mut bufs[buf_idx];
                pending_reads -= 1;
                if let Some(page) = remaining_pages.next() {
                    let entry = opcode::Read::new(
                        types::Fd(file.as_raw_fd()),
                        buf.as_mut_ptr(),
                        buf.len() as u32,
                    )
                    .offset(page * self.block_size as u64)
                    .build()
                    .user_data(buf_idx as u64);
                    unsafe {
                        // SAFETY: fd and buffer are valid for the duration of the operation
                        uring.submission().push(&entry).unwrap();
                    }
                    pending_reads += 1;
                    uring.submit().unwrap();
                }
            }
        }
    }
}

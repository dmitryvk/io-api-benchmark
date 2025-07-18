use std::{fs::OpenOptions, io::Write, os::unix::fs::FileExt};

use serde::{Deserialize, Serialize};

use crate::{
    IoMethod, IoSequence,
    io_data::{access_seq, aligned_vec, buf_data},
};

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct Buffered {
    pub block_size: u32,
}

impl IoMethod for Buffered {
    fn write_file(&self, path: &std::path::Path, file_size: u64, sequence: crate::IoSequence) {
        assert_eq!(file_size % self.block_size as u64, 0);
        let buf = buf_data(self.block_size as usize);
        let num_pages = file_size / self.block_size as u64;
        let mut file = OpenOptions::new()
            .write(true)
            .read(true)
            .open(path)
            .unwrap();
        for page_idx in access_seq(sequence, num_pages) {
            file.write_all_at(&buf, page_idx * self.block_size as u64)
                .unwrap();
        }
        file.flush().unwrap();
        file.sync_all().unwrap();
    }

    fn read_file(&self, path: &std::path::Path, file_size: u64, sequence: IoSequence) {
        assert_eq!(file_size % self.block_size as u64, 0);
        let mut buf = aligned_vec(self.block_size as usize);
        let num_pages = file_size / self.block_size as u64;
        let file = OpenOptions::new()
            .write(true)
            .read(true)
            .open(path)
            .unwrap();
        for page_idx in access_seq(sequence, num_pages) {
            file.read_exact_at(&mut buf, page_idx * self.block_size as u64)
                .unwrap();
        }
    }
}

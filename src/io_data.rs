use std::{
    alloc::{Layout, alloc},
    ptr::write_bytes,
};

use rand::{RngCore, rng, seq::SliceRandom};

use crate::IoSequence;

pub fn access_seq(sequence: IoSequence, n: u64) -> impl Iterator<Item = u64> {
    let mut result: Vec<_> = (0..n).collect();
    match sequence {
        IoSequence::Sequential => {}
        IoSequence::Random => result.shuffle(&mut rng()),
    }
    result.into_iter()
}

pub fn aligned_vec(buf_size: usize) -> Vec<u8> {
    let layout = Layout::from_size_align(buf_size, 4096).unwrap();
    // SAFETY: layout is correct
    let allocation = unsafe { alloc(layout) };
    // SAFETY: the allocation is allocation with buf_size
    unsafe {
        write_bytes(allocation, 0, buf_size);
    }
    // SAFETY: the allocation contains buf_size bytes
    unsafe { Vec::from_raw_parts(allocation, buf_size, buf_size) }
}

pub fn buf_data(buf_size: usize) -> Vec<u8> {
    let mut buf = aligned_vec(buf_size);
    rng().fill_bytes(&mut buf);
    buf
}

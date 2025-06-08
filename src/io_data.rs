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

pub fn buf_data(buf_size: usize) -> Vec<u8> {
    let mut buf = vec![0u8; buf_size];
    rng().fill_bytes(&mut buf);
    buf
}

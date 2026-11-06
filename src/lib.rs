#![doc = include_str!("../README.md")]
#![no_std]
#![forbid(unsafe_code)]
#![warn(missing_docs)]

mod error;
mod ring_buffer;

pub use error::Full;
pub use ring_buffer::RingBuffer;

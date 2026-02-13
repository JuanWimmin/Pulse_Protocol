#![no_std]

pub mod contract;
pub mod errors;
pub mod storage;
pub mod types;

#[cfg(test)]
mod test;

pub use contract::DocumentRegistryContract;
pub use errors::DocumentError;
pub use types::*;

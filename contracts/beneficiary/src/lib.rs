#![no_std]

pub mod contract;
pub mod errors;
pub mod storage;
pub mod types;

#[cfg(test)]
mod test;

pub use contract::BeneficiaryContract;
pub use errors::BeneficiaryError;
pub use types::*;

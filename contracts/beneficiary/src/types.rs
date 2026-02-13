use soroban_sdk::{contracttype, Address};

#[contracttype]
#[derive(Clone, Debug)]
pub struct Beneficiary {
    pub address: Address,
    pub percentage: u32,
    pub claimed: bool,
}

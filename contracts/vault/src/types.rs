use soroban_sdk::{contracttype, Address};

pub type VaultId = u64;

#[contracttype]
#[derive(Clone, Debug, PartialEq)]
pub enum VaultStatus {
    Active,
    Alert,
    GracePeriod,
    Triggered,
    Distributed,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct Beneficiary {
    pub address: Address,
    pub percentage: u32,
    pub claimed: bool,
}

#[contracttype]
#[derive(Clone, Debug)]
pub struct VaultInfo {
    pub id: VaultId,
    pub owner: Address,
    pub token: Address,
    pub status: VaultStatus,
    pub balance: i128,
    pub created_at: u64,
    pub last_updated: u64,
}

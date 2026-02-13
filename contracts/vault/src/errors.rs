use soroban_sdk::contracterror;

#[contracterror]
#[derive(Clone, Debug, PartialEq)]
pub enum VaultError {
    NotAuthorized = 1,
    VaultNotFound = 2,
    InsufficientBalance = 3,
    InvalidStatus = 4,
    InvalidBeneficiaries = 5,
    PercentageSumInvalid = 6,
    NoBeneficiaries = 7,
    ZeroAmount = 8,
    AlreadyInitialized = 9,
    NotInitialized = 10,
    ProofOfLifeNotLinked = 11,
}

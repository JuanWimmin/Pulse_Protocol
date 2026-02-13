use soroban_sdk::contracterror;

#[contracterror]
#[derive(Clone, Debug, PartialEq)]
pub enum BeneficiaryError {
    NotAuthorized = 1,
    VaultNotTriggered = 2,
    AlreadyClaimed = 3,
    NotABeneficiary = 4,
    InvalidPercentage = 5,
    VaultNotFound = 6,
    AlreadyInitialized = 7,
    NotInitialized = 8,
    PercentageSumInvalid = 9,
    NoBeneficiaries = 10,
}

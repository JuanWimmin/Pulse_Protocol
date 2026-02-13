use soroban_sdk::contracterror;

#[contracterror]
#[derive(Clone, Debug, PartialEq)]
pub enum ProofOfLifeError {
    NotAuthorized = 1,
    UserNotRegistered = 2,
    InvalidScore = 3,
    InvalidWeights = 4,
    OracleNotRegistered = 5,
    AlreadyInitialized = 6,
    NotInitialized = 7,
    VaultNotLinked = 8,
}

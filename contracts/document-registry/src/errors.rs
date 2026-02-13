use soroban_sdk::contracterror;

#[contracterror]
#[derive(Clone, Debug, PartialEq)]
pub enum DocumentError {
    NotAuthorized = 1,
    DocumentNotFound = 2,
    AlreadyLinked = 3,
    AccessDenied = 4,
    AlreadyInitialized = 5,
    NotInitialized = 6,
    KeyNotFound = 7,
}

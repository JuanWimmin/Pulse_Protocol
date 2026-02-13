#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, Env, Vec as SorobanVec};
use contract::{BeneficiaryContract, BeneficiaryContractClient};
use types::Beneficiary;

// ---------------------------------------------------------------------------
// 1. test_initialize
// ---------------------------------------------------------------------------
#[test]
fn test_initialize() {
    let env = Env::default();
    let contract_id = env.register(BeneficiaryContract, ());
    let client = BeneficiaryContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);
    client.initialize(&admin);
}

// ---------------------------------------------------------------------------
// 2. test_initialize_twice_fails
// ---------------------------------------------------------------------------
#[test]
fn test_initialize_twice_fails() {
    let env = Env::default();
    let contract_id = env.register(BeneficiaryContract, ());
    let client = BeneficiaryContractClient::new(&env, &contract_id);
    let admin = Address::generate(&env);

    client.initialize(&admin);

    let result = client.try_initialize(&admin);
    assert!(result.is_err() || result.unwrap().is_err());
}

// ---------------------------------------------------------------------------
// 3. test_set_beneficiaries_valid
// ---------------------------------------------------------------------------
#[test]
fn test_set_beneficiaries_valid() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(BeneficiaryContract, ());
    let client = BeneficiaryContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let b1 = Beneficiary {
        address: Address::generate(&env),
        percentage: 5000,
        claimed: false,
    };
    let b2 = Beneficiary {
        address: Address::generate(&env),
        percentage: 5000,
        claimed: false,
    };

    let mut beneficiaries = SorobanVec::new(&env);
    beneficiaries.push_back(b1.clone());
    beneficiaries.push_back(b2.clone());

    client.set_beneficiaries(&0u64, &beneficiaries);

    let result = client.get_beneficiaries(&0u64);
    assert_eq!(result.len(), 2);
}

// ---------------------------------------------------------------------------
// 4. test_set_beneficiaries_invalid_sum
// ---------------------------------------------------------------------------
#[test]
fn test_set_beneficiaries_invalid_sum() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(BeneficiaryContract, ());
    let client = BeneficiaryContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let b1 = Beneficiary {
        address: Address::generate(&env),
        percentage: 3000,
        claimed: false,
    };
    let b2 = Beneficiary {
        address: Address::generate(&env),
        percentage: 3000,
        claimed: false,
    };

    // Sum is 6000, not 10000
    let mut beneficiaries = SorobanVec::new(&env);
    beneficiaries.push_back(b1);
    beneficiaries.push_back(b2);

    let result = client.try_set_beneficiaries(&0u64, &beneficiaries);
    assert!(result.is_err() || result.unwrap().is_err());
}

// ---------------------------------------------------------------------------
// 5. test_set_beneficiaries_empty
// ---------------------------------------------------------------------------
#[test]
fn test_set_beneficiaries_empty() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(BeneficiaryContract, ());
    let client = BeneficiaryContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    client.initialize(&admin);

    let beneficiaries: SorobanVec<Beneficiary> = SorobanVec::new(&env);

    let result = client.try_set_beneficiaries(&0u64, &beneficiaries);
    assert!(result.is_err() || result.unwrap().is_err());
}

// ---------------------------------------------------------------------------
// 6. test_can_claim
// ---------------------------------------------------------------------------
#[test]
fn test_can_claim() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(BeneficiaryContract, ());
    let client = BeneficiaryContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let claimer = Address::generate(&env);
    let non_beneficiary = Address::generate(&env);

    client.initialize(&admin);

    let b1 = Beneficiary {
        address: claimer.clone(),
        percentage: 10000,
        claimed: false,
    };

    let mut beneficiaries = SorobanVec::new(&env);
    beneficiaries.push_back(b1);
    client.set_beneficiaries(&0u64, &beneficiaries);

    assert_eq!(client.can_claim(&0u64, &claimer), true);
    assert_eq!(client.can_claim(&0u64, &non_beneficiary), false);
}

// ---------------------------------------------------------------------------
// 7. test_record_claim
// ---------------------------------------------------------------------------
#[test]
fn test_record_claim() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(BeneficiaryContract, ());
    let client = BeneficiaryContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let claimer = Address::generate(&env);

    client.initialize(&admin);

    let b1 = Beneficiary {
        address: claimer.clone(),
        percentage: 7000,
        claimed: false,
    };
    let b2 = Beneficiary {
        address: Address::generate(&env),
        percentage: 3000,
        claimed: false,
    };

    let mut beneficiaries = SorobanVec::new(&env);
    beneficiaries.push_back(b1);
    beneficiaries.push_back(b2);
    client.set_beneficiaries(&0u64, &beneficiaries);

    let percentage = client.record_claim(&0u64, &claimer);
    assert_eq!(percentage, 7000);

    // After claiming, can_claim should return false
    assert_eq!(client.can_claim(&0u64, &claimer), false);
}

// ---------------------------------------------------------------------------
// 8. test_record_claim_already_claimed
// ---------------------------------------------------------------------------
#[test]
fn test_record_claim_already_claimed() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(BeneficiaryContract, ());
    let client = BeneficiaryContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let claimer = Address::generate(&env);

    client.initialize(&admin);

    let b1 = Beneficiary {
        address: claimer.clone(),
        percentage: 10000,
        claimed: false,
    };

    let mut beneficiaries = SorobanVec::new(&env);
    beneficiaries.push_back(b1);
    client.set_beneficiaries(&0u64, &beneficiaries);

    // First claim succeeds
    client.record_claim(&0u64, &claimer);

    // Second claim should fail with AlreadyClaimed
    let result = client.try_record_claim(&0u64, &claimer);
    assert!(result.is_err() || result.unwrap().is_err());
}

// ---------------------------------------------------------------------------
// 9. test_record_claim_not_beneficiary
// ---------------------------------------------------------------------------
#[test]
fn test_record_claim_not_beneficiary() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(BeneficiaryContract, ());
    let client = BeneficiaryContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let claimer = Address::generate(&env);
    let stranger = Address::generate(&env);

    client.initialize(&admin);

    let b1 = Beneficiary {
        address: claimer.clone(),
        percentage: 10000,
        claimed: false,
    };

    let mut beneficiaries = SorobanVec::new(&env);
    beneficiaries.push_back(b1);
    client.set_beneficiaries(&0u64, &beneficiaries);

    // Stranger is not a beneficiary
    let result = client.try_record_claim(&0u64, &stranger);
    assert!(result.is_err() || result.unwrap().is_err());
}

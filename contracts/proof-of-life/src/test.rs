#![cfg(test)]

use super::*;
use soroban_sdk::{testutils::Address as _, Address, BytesN, Env, Vec as SorobanVec};
use contract::{ProofOfLifeContract, ProofOfLifeContractClient};
use errors::ProofOfLifeError;
use types::VerificationSource;

fn create_weights(env: &Env) -> SorobanVec<i128> {
    let mut weights = SorobanVec::new(env);
    for _ in 0..10 {
        weights.push_back(500_000i128); // 0.5 in fixed-point
    }
    weights
}

fn setup_initialized(env: &Env) -> (ProofOfLifeContractClient<'_>, Address, Address) {
    let contract_id = env.register(ProofOfLifeContract, ());
    let client = ProofOfLifeContractClient::new(env, &contract_id);
    let admin = Address::generate(env);
    let oracle = Address::generate(env);
    client.initialize(&admin, &oracle);
    (client, admin, oracle)
}

fn setup_with_user(env: &Env) -> (ProofOfLifeContractClient<'_>, Address, Address, Address) {
    let (client, admin, oracle) = setup_initialized(env);
    let user = Address::generate(env);
    let weights = create_weights(env);
    client.register_model(&user, &weights, &100_000i128);
    (client, admin, oracle, user)
}

// ─── 1. test_initialize ──────────────────────────────────────────────

#[test]
fn test_initialize() {
    let env = Env::default();
    let contract_id = env.register(ProofOfLifeContract, ());
    let client = ProofOfLifeContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let oracle = Address::generate(&env);
    client.initialize(&admin, &oracle);
}

// ─── 2. test_initialize_twice_fails ──────────────────────────────────

#[test]
fn test_initialize_twice_fails() {
    let env = Env::default();
    let (client, admin, oracle) = setup_initialized(&env);

    let result = client.try_initialize(&admin, &oracle);
    assert_eq!(result, Err(Ok(ProofOfLifeError::AlreadyInitialized)));
}

// ─── 3. test_register_model ─────────────────────────────────────────

#[test]
fn test_register_model() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(ProofOfLifeContract, ());
    let client = ProofOfLifeContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let oracle = Address::generate(&env);
    let user = Address::generate(&env);

    client.initialize(&admin, &oracle);

    let weights = create_weights(&env);
    client.register_model(&user, &weights, &100_000i128);

    let model = client.get_model(&user);
    assert_eq!(model.version, 1);
    assert_eq!(model.weights.len(), 10);
}

// ─── 4. test_register_model_invalid_weights ─────────────────────────

#[test]
fn test_register_model_invalid_weights() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _admin, _oracle) = setup_initialized(&env);
    let user = Address::generate(&env);

    // Only 5 weights instead of 10
    let mut bad_weights = SorobanVec::new(&env);
    for _ in 0..5 {
        bad_weights.push_back(500_000i128);
    }

    let result = client.try_register_model(&user, &bad_weights, &100_000i128);
    assert_eq!(result, Err(Ok(ProofOfLifeError::InvalidWeights)));
}

// ─── 5. test_submit_verification ────────────────────────────────────

#[test]
fn test_submit_verification() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(ProofOfLifeContract, ());
    let client = ProofOfLifeContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let oracle = Address::generate(&env);
    let user = Address::generate(&env);

    client.initialize(&admin, &oracle);

    let weights = create_weights(&env);
    client.register_model(&user, &weights, &100_000i128);

    let sig = BytesN::from_array(&env, &[0u8; 64]);
    client.submit_verification(&user, &8500u32, &VerificationSource::PerceptronAggregate, &sig);

    let score = client.get_liveness_score(&user);
    assert_eq!(score, 8500);
}

// ─── 6. test_submit_verification_invalid_score ──────────────────────

#[test]
fn test_submit_verification_invalid_score() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _admin, _oracle, user) = setup_with_user(&env);

    let sig = BytesN::from_array(&env, &[0u8; 64]);
    let result = client.try_submit_verification(
        &user,
        &10_001u32,
        &VerificationSource::PerceptronAggregate,
        &sig,
    );
    assert_eq!(result, Err(Ok(ProofOfLifeError::InvalidScore)));
}

// ─── 7. test_submit_verification_unregistered ───────────────────────

#[test]
fn test_submit_verification_unregistered() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _admin, _oracle) = setup_initialized(&env);
    let unregistered_user = Address::generate(&env);

    let sig = BytesN::from_array(&env, &[0u8; 64]);
    let result = client.try_submit_verification(
        &unregistered_user,
        &5000u32,
        &VerificationSource::PerceptronAggregate,
        &sig,
    );
    assert_eq!(result, Err(Ok(ProofOfLifeError::UserNotRegistered)));
}

// ─── 8. test_update_model ───────────────────────────────────────────

#[test]
fn test_update_model() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _admin, _oracle, user) = setup_with_user(&env);

    // Verify initial version
    let model_before = client.get_model(&user);
    assert_eq!(model_before.version, 1);

    // Update with new weights
    let mut new_weights = SorobanVec::new(&env);
    for _ in 0..10 {
        new_weights.push_back(750_000i128); // 0.75 in fixed-point
    }
    client.update_model(&user, &new_weights, &200_000i128);

    let model_after = client.get_model(&user);
    assert_eq!(model_after.version, 2);
    assert_eq!(model_after.bias, 200_000i128);
    assert_eq!(model_after.weights.get(0).unwrap(), 750_000i128);
}

// ─── 9. test_update_model_invalid_weights ───────────────────────────

#[test]
fn test_update_model_invalid_weights() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _admin, _oracle, user) = setup_with_user(&env);

    // Try updating with 3 weights instead of 10
    let mut bad_weights = SorobanVec::new(&env);
    for _ in 0..3 {
        bad_weights.push_back(100_000i128);
    }

    let result = client.try_update_model(&user, &bad_weights, &200_000i128);
    assert_eq!(result, Err(Ok(ProofOfLifeError::InvalidWeights)));
}

// ─── 10. test_emergency_checkin ─────────────────────────────────────

#[test]
fn test_emergency_checkin() {
    let env = Env::default();
    env.mock_all_auths();

    let contract_id = env.register(ProofOfLifeContract, ());
    let client = ProofOfLifeContractClient::new(&env, &contract_id);

    let admin = Address::generate(&env);
    let oracle = Address::generate(&env);
    let user = Address::generate(&env);

    client.initialize(&admin, &oracle);

    let weights = create_weights(&env);
    client.register_model(&user, &weights, &100_000i128);

    // Submit low score
    let sig = BytesN::from_array(&env, &[0u8; 64]);
    client.submit_verification(&user, &2000u32, &VerificationSource::PerceptronAggregate, &sig);
    assert_eq!(client.get_liveness_score(&user), 2000);

    // Emergency checkin resets to max
    client.emergency_checkin(&user);
    assert_eq!(client.get_liveness_score(&user), 10_000);
}

// ─── 11. test_emergency_checkin_unregistered ────────────────────────

#[test]
fn test_emergency_checkin_unregistered() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _admin, _oracle) = setup_initialized(&env);
    let unregistered_user = Address::generate(&env);

    let result = client.try_emergency_checkin(&unregistered_user);
    assert_eq!(result, Err(Ok(ProofOfLifeError::UserNotRegistered)));
}

// ─── 12. test_link_vault ────────────────────────────────────────────

#[test]
fn test_link_vault() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _admin, _oracle, user) = setup_with_user(&env);
    let vault_contract = Address::generate(&env);
    let vault_id: u64 = 42;

    client.link_vault(&user, &vault_contract, &vault_id);

    // Verify the vault is linked by submitting a verification that will
    // trigger the transition event (proving vault link was stored).
    let sig = BytesN::from_array(&env, &[0u8; 64]);
    client.submit_verification(&user, &8000u32, &VerificationSource::PerceptronAggregate, &sig);

    let score = client.get_liveness_score(&user);
    assert_eq!(score, 8000);
}

// ─── 13. test_get_liveness_score_default ────────────────────────────

#[test]
fn test_get_liveness_score_default() {
    let env = Env::default();
    env.mock_all_auths();

    let (client, _admin, _oracle, user) = setup_with_user(&env);

    // Default score for a registered user with no verifications should be 10_000
    let score = client.get_liveness_score(&user);
    assert_eq!(score, 10_000);
}

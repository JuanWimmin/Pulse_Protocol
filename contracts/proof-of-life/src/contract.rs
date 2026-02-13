use soroban_sdk::{contract, contractimpl, symbol_short, Address, BytesN, Env, Vec};

use crate::errors::ProofOfLifeError;
use crate::storage;
use crate::types::{LifeModel, VerificationRecord, VerificationSource};

const DEFAULT_ALERT_THRESHOLD: u32 = 7_000;
const DEFAULT_CRITICAL_THRESHOLD: u32 = 3_000;
const DEFAULT_GRACE_PERIOD_DAYS: u32 = 30;
const MAX_SCORE: u32 = 10_000;
const EXPECTED_WEIGHTS_COUNT: u32 = 10;

#[contract]
pub struct ProofOfLifeContract;

#[allow(deprecated)]
#[contractimpl]
impl ProofOfLifeContract {
    /// Initialize the contract with admin and oracle addresses.
    pub fn initialize(env: Env, admin: Address, oracle: Address) -> Result<(), ProofOfLifeError> {
        if storage::is_initialized(&env) {
            return Err(ProofOfLifeError::AlreadyInitialized);
        }
        storage::set_admin(&env, &admin);
        storage::set_oracle(&env, &oracle);
        storage::set_initialized(&env);
        Ok(())
    }

    /// Register a perceptron model for a user.
    pub fn register_model(
        env: Env,
        user: Address,
        initial_weights: Vec<i128>,
        bias: i128,
    ) -> Result<(), ProofOfLifeError> {
        user.require_auth();

        if initial_weights.len() != EXPECTED_WEIGHTS_COUNT {
            return Err(ProofOfLifeError::InvalidWeights);
        }

        let model = LifeModel {
            weights: initial_weights,
            bias,
            version: 1,
            last_updated: env.ledger().timestamp(),
            calibration_complete: false,
            total_verifications: 0,
            avg_confidence: 0,
            alert_threshold: DEFAULT_ALERT_THRESHOLD,
            critical_threshold: DEFAULT_CRITICAL_THRESHOLD,
            grace_period_days: DEFAULT_GRACE_PERIOD_DAYS,
        };

        storage::set_model(&env, &user, &model);

        env.events()
            .publish((symbol_short!("pol"), symbol_short!("model")), user);

        Ok(())
    }

    /// Submit a verification score. Only callable by the oracle.
    pub fn submit_verification(
        env: Env,
        user: Address,
        score: u32,
        source: VerificationSource,
        oracle_sig: BytesN<64>,
    ) -> Result<(), ProofOfLifeError> {
        let oracle = storage::get_oracle(&env);
        oracle.require_auth();

        if score > MAX_SCORE {
            return Err(ProofOfLifeError::InvalidScore);
        }

        if storage::get_model(&env, &user).is_none() {
            return Err(ProofOfLifeError::UserNotRegistered);
        }

        let record = VerificationRecord {
            timestamp: env.ledger().timestamp(),
            liveness_score: score,
            source,
            oracle_signature: oracle_sig,
        };

        let mut verifications = storage::get_verifications(&env, &user);
        verifications.push_back(record);
        storage::set_verifications(&env, &user, &verifications);
        storage::set_last_score(&env, &user, score);

        let mut model = storage::get_model(&env, &user).unwrap();
        model.total_verifications += 1;
        storage::set_model(&env, &user, &model);

        env.events()
            .publish((symbol_short!("pol"), symbol_short!("verify")), (&user, score));

        // Check if a vault is linked and emit status transition event
        if let Some(_vault_contract) = storage::get_linked_vault(&env, &user) {
            if let Some(vault_id) = storage::get_linked_vault_id(&env, &user) {
                let new_status: u32 = if score < model.critical_threshold {
                    2u32 // GracePeriod
                } else if score < model.alert_threshold {
                    1u32 // Alert
                } else {
                    0u32 // Active
                };
                env.events().publish(
                    (symbol_short!("pol"), symbol_short!("transit")),
                    (&user, vault_id, new_status),
                );
            }
        }

        Ok(())
    }

    /// Update model weights after re-training.
    pub fn update_model(
        env: Env,
        user: Address,
        new_weights: Vec<i128>,
        new_bias: i128,
    ) -> Result<(), ProofOfLifeError> {
        user.require_auth();

        if new_weights.len() != EXPECTED_WEIGHTS_COUNT {
            return Err(ProofOfLifeError::InvalidWeights);
        }

        let mut model = storage::get_model(&env, &user)
            .ok_or(ProofOfLifeError::UserNotRegistered)?;

        model.weights = new_weights;
        model.bias = new_bias;
        model.version += 1;
        model.last_updated = env.ledger().timestamp();

        storage::set_model(&env, &user, &model);

        Ok(())
    }

    /// Get current liveness score for a user.
    pub fn get_liveness_score(env: Env, user: Address) -> Result<u32, ProofOfLifeError> {
        if storage::get_model(&env, &user).is_none() {
            return Err(ProofOfLifeError::UserNotRegistered);
        }
        Ok(storage::get_last_score(&env, &user))
    }

    /// Emergency check-in. Resets score to maximum.
    pub fn emergency_checkin(env: Env, user: Address) -> Result<(), ProofOfLifeError> {
        user.require_auth();

        if storage::get_model(&env, &user).is_none() {
            return Err(ProofOfLifeError::UserNotRegistered);
        }

        storage::set_last_score(&env, &user, MAX_SCORE);

        env.events()
            .publish((symbol_short!("pol"), symbol_short!("checkin")), &user);

        Ok(())
    }

    /// Get model data for a user.
    pub fn get_model(env: Env, user: Address) -> Result<LifeModel, ProofOfLifeError> {
        storage::get_model(&env, &user).ok_or(ProofOfLifeError::UserNotRegistered)
    }

    /// Link a vault contract to this user.
    pub fn link_vault(
        env: Env,
        user: Address,
        vault_contract: Address,
        vault_id: u64,
    ) -> Result<(), ProofOfLifeError> {
        user.require_auth();
        storage::set_linked_vault(&env, &user, &vault_contract);
        storage::set_linked_vault_id(&env, &user, vault_id);
        Ok(())
    }
}

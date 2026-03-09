use ed25519_dalek::{Signer, SigningKey};
use sha2::{Digest, Sha256};
use stellar_strkey::ed25519::PrivateKey as StrPrivateKey;
use tracing::{error, info};

use crate::services::aggregator::ValidatedScore;
use crate::services::contracts::proof_of_life::ProofOfLifeContractClient;

/// On-chain publisher: signs scores with the oracle key and submits to Soroban.
#[derive(Clone)]
pub struct Publisher {
    pol_client: ProofOfLifeContractClient,
    signing_key: SigningKey,
}

impl Publisher {
    /// Create a new publisher with a ProofOfLife contract client and oracle secret key.
    pub fn new(
        pol_client: ProofOfLifeContractClient,
        oracle_secret_key: &str,
    ) -> Result<Self, String> {
        let decoded = StrPrivateKey::from_string(oracle_secret_key)
            .map_err(|e| format!("Invalid oracle secret key: {}", e))?;
        let signing_key = SigningKey::from_bytes(&decoded.0);

        Ok(Self {
            pol_client,
            signing_key,
        })
    }

    /// Sign a score and publish it on-chain.
    ///
    /// Flow:
    /// 1. Sign the score payload with the oracle's ed25519 key
    /// 2. Call submit_verification() on the ProofOfLife contract
    /// 3. Return the transaction hash
    pub async fn publish_score(
        &self,
        validated: &ValidatedScore,
    ) -> Result<String, String> {
        info!(
            "Publishing score: user={}, score={}, source={}",
            validated.user_stellar_address, validated.score, validated.source
        );

        // Build signature payload: hash(user_address | score | source)
        let signature = self.sign_score(
            &validated.user_stellar_address,
            validated.score,
            &validated.source,
        );

        // Submit on-chain
        let tx_hash = self
            .pol_client
            .submit_verification(
                &validated.user_stellar_address,
                validated.score,
                &validated.source,
                &signature,
            )
            .await
            .map_err(|e| {
                error!("Failed to publish score on-chain: {}", e);
                e
            })?;

        info!(
            "Score published on-chain: tx={}, user={}, score={}",
            tx_hash, validated.user_stellar_address, validated.score
        );

        Ok(tx_hash)
    }

    /// Sign a score payload with the oracle key.
    /// Returns the 64-byte ed25519 signature.
    fn sign_score(&self, user_address: &str, score: u32, source: &str) -> Vec<u8> {
        // Deterministic payload: sha256(address || score_be || source)
        let mut hasher = Sha256::new();
        hasher.update(user_address.as_bytes());
        hasher.update(score.to_be_bytes());
        hasher.update(source.as_bytes());
        let payload_hash = hasher.finalize();

        let signature = self.signing_key.sign(&payload_hash);
        signature.to_bytes().to_vec()
    }
}

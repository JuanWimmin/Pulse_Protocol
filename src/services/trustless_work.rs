use crate::services::soroban::{
    address_to_scval, i128_to_scval, symbol_to_scval, u32_to_scval, SorobanClient,
};
use stellar_xdr::curr::{ScMapEntry, ScVal};
use tracing::info;

/// Backend-orchestrated Trustless Work escrow integration.
///
/// Instead of C2C (contract-to-contract) calls from Vault Contract,
/// the backend orchestrates all escrow operations directly via Soroban RPC.
///
/// Flow:
///   createVault  → backend calls TW factory to create escrow
///   deposit      → backend calls TW escrow to fund it
///   TRIGGERED    → backend calls TW escrow to approve milestones
///   claim        → backend calls TW escrow to release funds to beneficiary
#[derive(Clone)]
pub struct TrustlessWorkOrchestrator {
    soroban: SorobanClient,
    /// TW factory contract address on testnet (creates new escrows).
    factory_contract_id: Option<String>,
}

/// Escrow creation result.
#[derive(Debug, Clone)]
pub struct EscrowInfo {
    pub escrow_contract_id: String,
    pub tx_hash: String,
}

/// Milestone definition for a beneficiary.
#[derive(Debug, Clone)]
pub struct Milestone {
    pub beneficiary_address: String,
    pub percentage: u32,
}

impl TrustlessWorkOrchestrator {
    pub fn new(soroban: SorobanClient, factory_contract_id: Option<String>) -> Self {
        Self {
            soroban,
            factory_contract_id,
        }
    }

    /// Check if TW integration is available (factory configured).
    pub fn is_available(&self) -> bool {
        self.factory_contract_id.is_some()
    }

    /// Create a new escrow for a vault via TW factory.
    ///
    /// Maps Pulse Protocol roles to TW roles:
    /// - service_provider = oracle address (Pulse Oracle)
    /// - approver = oracle address (acts as PoL proxy)
    /// - release_signer = oracle address (releases when TRIGGERED)
    /// - receiver = beneficiary (set later via milestones)
    /// - platform_address = oracle address (Pulse Protocol)
    pub async fn create_escrow(
        &self,
        vault_id: u32,
        oracle_address: &str,
        token_address: &str,
        milestones: &[Milestone],
    ) -> Result<EscrowInfo, String> {
        let factory_id = self
            .factory_contract_id
            .as_ref()
            .ok_or("TW factory contract not configured")?;

        info!(
            "Creating TW escrow for vault {} with {} milestones",
            vault_id,
            milestones.len()
        );

        // Build milestone ScVal Vec
        let milestone_vals: Result<Vec<ScVal>, String> = milestones
            .iter()
            .map(|m| {
                Ok(ScVal::Map(Some(
                    vec![
                        ScMapEntry {
                            key: ScVal::Symbol(stellar_xdr::curr::ScSymbol("receiver".try_into().map_err(|_| "sym")?)),
                            val: address_to_scval(&m.beneficiary_address)?,
                        },
                        ScMapEntry {
                            key: ScVal::Symbol(stellar_xdr::curr::ScSymbol("amount_percentage".try_into().map_err(|_| "sym")?)),
                            val: u32_to_scval(m.percentage),
                        },
                    ]
                    .try_into()
                    .map_err(|_| "map entries")?,
                )))
            })
            .collect();

        let title = format!("Pulse Protocol Inheritance - Vault #{}", vault_id);

        let args = vec![
            // engagement_id
            symbol_to_scval(&format!("vault_{}", vault_id))?,
            // title
            ScVal::String(stellar_xdr::curr::ScString(
                title.try_into().map_err(|_| "title too long")?,
            )),
            // service_provider (oracle)
            address_to_scval(oracle_address)?,
            // approver (oracle acts as PoL proxy)
            address_to_scval(oracle_address)?,
            // release_signer (oracle releases when TRIGGERED)
            address_to_scval(oracle_address)?,
            // platform_address
            address_to_scval(oracle_address)?,
            // token
            address_to_scval(token_address)?,
            // milestones
            ScVal::Vec(Some(milestone_vals?.try_into().map_err(|_| "milestones vec")?)),
        ];

        let tx_hash = self
            .soroban
            .invoke_contract(factory_id, "create_escrow", args)
            .await?;

        // For MVP, the escrow contract ID would be parsed from the tx result.
        // For now, we use the factory_id as a reference.
        let escrow_contract_id = format!("escrow_vault_{}", vault_id);

        info!("TW escrow created: tx={}, escrow={}", tx_hash, escrow_contract_id);

        Ok(EscrowInfo {
            escrow_contract_id,
            tx_hash,
        })
    }

    /// Fund an escrow with deposited tokens.
    pub async fn fund_escrow(
        &self,
        escrow_contract_id: &str,
        funder: &str,
        amount: i128,
        token: &str,
    ) -> Result<String, String> {
        if !self.is_available() {
            return Err("TW not configured".into());
        }

        info!(
            "Funding TW escrow {} with {} from {}",
            escrow_contract_id, amount, funder
        );

        let args = vec![
            address_to_scval(funder)?,
            i128_to_scval(amount),
            address_to_scval(token)?,
        ];

        self.soroban
            .invoke_contract(escrow_contract_id, "fund", args)
            .await
    }

    /// Approve all milestones when vault transitions to TRIGGERED.
    /// This signals that the inheritance condition is met.
    pub async fn approve_milestones(
        &self,
        escrow_contract_id: &str,
        approver: &str,
    ) -> Result<String, String> {
        if !self.is_available() {
            return Err("TW not configured".into());
        }

        info!(
            "Approving TW milestones on escrow {} by {}",
            escrow_contract_id, approver
        );

        let args = vec![address_to_scval(approver)?];

        self.soroban
            .invoke_contract(escrow_contract_id, "approve_milestones", args)
            .await
    }

    /// Release funds to a specific beneficiary from the escrow.
    pub async fn release_to_beneficiary(
        &self,
        escrow_contract_id: &str,
        release_signer: &str,
        beneficiary: &str,
    ) -> Result<String, String> {
        if !self.is_available() {
            return Err("TW not configured".into());
        }

        info!(
            "Releasing TW escrow {} to beneficiary {}",
            escrow_contract_id, beneficiary
        );

        let args = vec![
            address_to_scval(release_signer)?,
            address_to_scval(beneficiary)?,
        ];

        self.soroban
            .invoke_contract(escrow_contract_id, "release", args)
            .await
    }
}

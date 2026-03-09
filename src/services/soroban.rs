use base64::{engine::general_purpose::STANDARD as BASE64, Engine};
use ed25519_dalek::{Signer, SigningKey};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};
use stellar_strkey::ed25519::PublicKey as StrPublicKey;
use stellar_xdr::curr::{
    AccountId, ContractId, Hash, HostFunction, InvokeContractArgs, InvokeHostFunctionOp,
    Limits, Memo, MuxedAccount, Operation, OperationBody, Preconditions, PublicKey,
    ScAddress, ScSymbol, ScVal, SequenceNumber, SorobanAuthorizationEntry,
    SorobanTransactionData, Transaction, TransactionEnvelope,
    TransactionExt, TransactionSignaturePayload,
    TransactionSignaturePayloadTaggedTransaction, TransactionV1Envelope, Uint256, WriteXdr,
    ReadXdr, DecoratedSignature, Signature, SignatureHint,
};
use tracing::{debug, error, info, warn};

/// Soroban JSON-RPC client for interacting with deployed contracts.
#[derive(Clone)]
pub struct SorobanClient {
    rpc_url: String,
    network_passphrase: String,
    oracle_keypair: Option<SigningKey>,
    oracle_public: Option<[u8; 32]>,
    http: Client,
}

// ── JSON-RPC request / response types ──

#[derive(Serialize)]
struct RpcRequest<'a> {
    jsonrpc: &'a str,
    id: u64,
    method: &'a str,
    params: serde_json::Value,
}

#[derive(Deserialize, Debug)]
struct RpcResponse {
    result: Option<serde_json::Value>,
    error: Option<RpcError>,
}

#[derive(Deserialize, Debug)]
struct RpcError {
    code: i64,
    message: String,
}

// ── Public response types ──

#[derive(Debug, Clone)]
pub struct SimulateResult {
    pub transaction_data: String,
    pub min_resource_fee: i64,
    pub results: Vec<SimResultEntry>,
    pub auth: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SimResultEntry {
    pub xdr: String,
    pub auth: Vec<String>,
}

#[derive(Debug, Clone)]
pub struct SendResult {
    pub hash: String,
    pub status: String,
}

#[derive(Debug, Clone)]
pub struct GetTxResult {
    pub status: String,
    pub return_value: Option<String>,
    pub envelope_xdr: Option<String>,
    pub result_xdr: Option<String>,
}

#[derive(Debug, Clone)]
pub struct AccountInfo {
    pub sequence: i64,
}

impl SorobanClient {
    /// Create a new Soroban RPC client.
    pub fn new(
        rpc_url: &str,
        network_passphrase: &str,
        oracle_secret_key: Option<&str>,
    ) -> Self {
        let (oracle_keypair, oracle_public) = oracle_secret_key
            .and_then(|sk| {
                let decoded = stellar_strkey::ed25519::PrivateKey::from_string(sk).ok()?;
                let signing = SigningKey::from_bytes(&decoded.0);
                let public = signing.verifying_key().to_bytes();
                Some((Some(signing), Some(public)))
            })
            .unwrap_or((None, None));

        Self {
            rpc_url: rpc_url.to_string(),
            network_passphrase: network_passphrase.to_string(),
            oracle_keypair,
            oracle_public,
            http: Client::new(),
        }
    }

    /// Get the oracle's Stellar address (G... format).
    pub fn oracle_address(&self) -> Option<String> {
        self.oracle_public.map(|pk| StrPublicKey(pk).to_string())
    }

    /// Call a Soroban RPC method.
    async fn rpc_call(&self, method: &str, params: serde_json::Value) -> Result<serde_json::Value, String> {
        let req = RpcRequest {
            jsonrpc: "2.0",
            id: 1,
            method,
            params,
        };

        let resp = self
            .http
            .post(&self.rpc_url)
            .json(&req)
            .send()
            .await
            .map_err(|e| format!("RPC request failed: {}", e))?;

        let rpc_resp: RpcResponse = resp
            .json()
            .await
            .map_err(|e| format!("RPC response parse error: {}", e))?;

        if let Some(err) = rpc_resp.error {
            return Err(format!("RPC error {}: {}", err.code, err.message));
        }

        rpc_resp.result.ok_or_else(|| "Empty RPC result".to_string())
    }

    /// Get account sequence number.
    pub async fn get_account(&self, address: &str) -> Result<AccountInfo, String> {
        let result = self
            .rpc_call("getAccount", serde_json::json!({ "address": address }))
            .await;

        match result {
            Ok(val) => {
                let seq_str = val["sequence"]
                    .as_str()
                    .ok_or("Missing sequence in account response")?;
                let sequence: i64 = seq_str.parse().map_err(|e| format!("Bad sequence: {}", e))?;
                Ok(AccountInfo { sequence })
            }
            Err(e) => {
                // Fallback: try getLedgerEntries for account
                warn!("getAccount failed ({}), trying getLedgerEntries", e);
                Err(e)
            }
        }
    }

    /// Simulate a transaction to get resource estimates and auth.
    pub async fn simulate_transaction(&self, tx_xdr: &str) -> Result<SimulateResult, String> {
        let result = self
            .rpc_call(
                "simulateTransaction",
                serde_json::json!({ "transaction": tx_xdr }),
            )
            .await?;

        let tx_data = result["transactionData"]
            .as_str()
            .unwrap_or("")
            .to_string();
        let min_fee = result["minResourceFee"]
            .as_str()
            .and_then(|s| s.parse::<i64>().ok())
            .unwrap_or(0);

        let mut results = Vec::new();
        if let Some(arr) = result["results"].as_array() {
            for r in arr {
                let xdr = r["xdr"].as_str().unwrap_or("").to_string();
                let auth = r["auth"]
                    .as_array()
                    .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
                    .unwrap_or_default();
                results.push(SimResultEntry { xdr, auth });
            }
        }

        let auth = result["results"]
            .as_array()
            .and_then(|arr| arr.first())
            .and_then(|r| r["auth"].as_array())
            .map(|a| a.iter().filter_map(|v| v.as_str().map(String::from)).collect())
            .unwrap_or_default();

        Ok(SimulateResult {
            transaction_data: tx_data,
            min_resource_fee: min_fee,
            results,
            auth,
        })
    }

    /// Send a signed transaction.
    pub async fn send_transaction(&self, tx_xdr: &str) -> Result<SendResult, String> {
        let result = self
            .rpc_call(
                "sendTransaction",
                serde_json::json!({ "transaction": tx_xdr }),
            )
            .await?;

        Ok(SendResult {
            hash: result["hash"].as_str().unwrap_or("").to_string(),
            status: result["status"].as_str().unwrap_or("").to_string(),
        })
    }

    /// Poll transaction status until final.
    pub async fn get_transaction(&self, hash: &str) -> Result<GetTxResult, String> {
        let result = self
            .rpc_call(
                "getTransaction",
                serde_json::json!({ "hash": hash }),
            )
            .await?;

        Ok(GetTxResult {
            status: result["status"].as_str().unwrap_or("NOT_FOUND").to_string(),
            return_value: result["returnValue"].as_str().map(String::from),
            envelope_xdr: result["envelopeXdr"].as_str().map(String::from),
            result_xdr: result["resultXdr"].as_str().map(String::from),
        })
    }

    /// Build an `InvokeContractArgs` ScVal from contract ID, function name, and args.
    pub fn build_invoke_args(
        contract_id: &str,
        function_name: &str,
        args: Vec<ScVal>,
    ) -> Result<InvokeContractArgs, String> {
        let contract_bytes = stellar_strkey::Contract::from_string(contract_id)
            .map_err(|e| format!("Invalid contract ID: {}", e))?;
        let contract_address = ScAddress::Contract(ContractId(Hash(contract_bytes.0)));

        Ok(InvokeContractArgs {
            contract_address,
            function_name: ScSymbol(function_name.try_into().map_err(|_| "Function name too long")?),
            args: args.try_into().map_err(|_| "Too many args")?,
        })
    }

    /// Build, simulate, sign, and send a contract invocation.
    /// Returns the tx hash on success.
    pub async fn invoke_contract(
        &self,
        contract_id: &str,
        function_name: &str,
        args: Vec<ScVal>,
    ) -> Result<String, String> {
        let keypair = self.oracle_keypair.as_ref().ok_or("No oracle keypair configured")?;
        let public_bytes = self.oracle_public.ok_or("No oracle public key")?;
        let source_address = StrPublicKey(public_bytes).to_string();

        info!("Invoking {}.{}() on contract {}", contract_id, function_name, &contract_id[..8]);

        // 1. Get account sequence
        let account = self.get_account(&source_address).await?;
        let sequence = account.sequence + 1;

        // 2. Build the invoke host function operation
        let invoke_args = Self::build_invoke_args(contract_id, function_name, args)?;

        let op = Operation {
            source_account: None,
            body: OperationBody::InvokeHostFunction(InvokeHostFunctionOp {
                host_function: HostFunction::InvokeContract(invoke_args),
                auth: Vec::new().try_into().map_err(|_| "auth vec")?,
            }),
        };

        // 3. Build unsigned transaction (for simulation)
        let source_account = MuxedAccount::Ed25519(Uint256(public_bytes));

        let tx = Transaction {
            source_account: source_account.clone(),
            fee: 100_000, // will be updated after simulation
            seq_num: SequenceNumber(sequence),
            cond: Preconditions::None,
            memo: Memo::None,
            operations: vec![op].try_into().map_err(|_| "ops vec")?,
            ext: TransactionExt::V0,
        };

        let tx_xdr = tx.to_xdr(Limits::none()).map_err(|e| format!("XDR encode: {}", e))?;
        let tx_b64 = BASE64.encode(&tx_xdr);

        // 4. Simulate
        debug!("Simulating transaction...");
        let sim = self.simulate_transaction(&tx_b64).await?;

        if sim.transaction_data.is_empty() {
            return Err("Simulation returned empty transaction data (contract call likely failed)".into());
        }

        // 5. Rebuild transaction with simulation results
        let soroban_data_bytes = BASE64.decode(&sim.transaction_data)
            .map_err(|e| format!("Decode soroban data: {}", e))?;
        let soroban_data = SorobanTransactionData::from_xdr(soroban_data_bytes, Limits::none())
            .map_err(|e| format!("Parse soroban data: {}", e))?;

        // Parse auth entries from simulation
        let auth_entries: Vec<SorobanAuthorizationEntry> = sim
            .auth
            .iter()
            .filter_map(|a| {
                let bytes = BASE64.decode(a).ok()?;
                SorobanAuthorizationEntry::from_xdr(bytes, Limits::none()).ok()
            })
            .collect();

        let invoke_args2 = Self::build_invoke_args(contract_id, function_name, Vec::new())?;
        let op2 = Operation {
            source_account: None,
            body: OperationBody::InvokeHostFunction(InvokeHostFunctionOp {
                host_function: HostFunction::InvokeContract(invoke_args2),
                auth: auth_entries.try_into().map_err(|_| "auth entries vec")?,
            }),
        };

        let fee = 100_000u32.saturating_add(sim.min_resource_fee as u32);

        let tx2 = Transaction {
            source_account: source_account.clone(),
            fee,
            seq_num: SequenceNumber(sequence),
            cond: Preconditions::None,
            memo: Memo::None,
            operations: vec![op2].try_into().map_err(|_| "ops vec")?,
            ext: TransactionExt::V1(soroban_data),
        };

        // 6. Sign the transaction
        let network_hash = Sha256::digest(self.network_passphrase.as_bytes());

        let payload = TransactionSignaturePayload {
            network_id: Hash(network_hash.into()),
            tagged_transaction: TransactionSignaturePayloadTaggedTransaction::Tx(tx2.clone()),
        };

        let payload_bytes = payload.to_xdr(Limits::none())
            .map_err(|e| format!("Sign payload XDR: {}", e))?;
        let payload_hash = Sha256::digest(&payload_bytes);

        let sig = keypair.sign(&payload_hash);

        let hint_bytes = &public_bytes[28..32];
        let mut hint_arr = [0u8; 4];
        hint_arr.copy_from_slice(hint_bytes);

        let envelope = TransactionEnvelope::Tx(TransactionV1Envelope {
            tx: tx2,
            signatures: vec![DecoratedSignature {
                hint: SignatureHint(hint_arr),
                signature: Signature(sig.to_bytes().to_vec().try_into().map_err(|_| "sig bytes")?),
            }]
            .try_into()
            .map_err(|_| "sigs vec")?,
        });

        let env_xdr = envelope.to_xdr(Limits::none())
            .map_err(|e| format!("Envelope XDR: {}", e))?;
        let env_b64 = BASE64.encode(&env_xdr);

        // 7. Send
        debug!("Sending transaction...");
        let send_result = self.send_transaction(&env_b64).await?;

        if send_result.status == "ERROR" {
            return Err(format!("Transaction send error: {:?}", send_result));
        }

        info!("Transaction sent: hash={}", send_result.hash);

        // 8. Poll for confirmation
        let tx_hash = send_result.hash.clone();
        for attempt in 0..30 {
            tokio::time::sleep(std::time::Duration::from_secs(1)).await;
            let tx_result = self.get_transaction(&tx_hash).await?;

            match tx_result.status.as_str() {
                "SUCCESS" => {
                    info!("Transaction confirmed: {}", tx_hash);
                    return Ok(tx_hash);
                }
                "FAILED" => {
                    error!("Transaction failed: {:?}", tx_result);
                    return Err(format!("Transaction failed: {:?}", tx_result.result_xdr));
                }
                "NOT_FOUND" => {
                    debug!("Tx pending (attempt {}/30)...", attempt + 1);
                    continue;
                }
                other => {
                    warn!("Unknown tx status: {}", other);
                    continue;
                }
            }
        }

        Err(format!("Transaction {} did not confirm after 30s", tx_hash))
    }

    /// Read contract data (getLedgerEntries) — used for queries.
    pub async fn get_contract_data(
        &self,
        contract_id: &str,
        key: ScVal,
    ) -> Result<Option<ScVal>, String> {
        let contract_bytes = stellar_strkey::Contract::from_string(contract_id)
            .map_err(|e| format!("Invalid contract ID: {}", e))?;

        let ledger_key = stellar_xdr::curr::LedgerKey::ContractData(
            stellar_xdr::curr::LedgerKeyContractData {
                contract: ScAddress::Contract(ContractId(Hash(contract_bytes.0))),
                key: key.clone(),
                durability: stellar_xdr::curr::ContractDataDurability::Persistent,
            },
        );

        let key_xdr = ledger_key.to_xdr(Limits::none())
            .map_err(|e| format!("Ledger key XDR: {}", e))?;
        let key_b64 = BASE64.encode(&key_xdr);

        let result = self
            .rpc_call(
                "getLedgerEntries",
                serde_json::json!({ "keys": [key_b64] }),
            )
            .await?;

        let entries = result["entries"].as_array();
        if let Some(entries) = entries {
            if let Some(entry) = entries.first() {
                let xdr_str = entry["xdr"]
                    .as_str()
                    .ok_or("Missing xdr in entry")?;
                let xdr_bytes = BASE64.decode(xdr_str)
                    .map_err(|e| format!("Decode entry: {}", e))?;
                let ledger_entry = stellar_xdr::curr::LedgerEntryData::from_xdr(xdr_bytes, Limits::none())
                    .map_err(|e| format!("Parse entry: {}", e))?;

                if let stellar_xdr::curr::LedgerEntryData::ContractData(data) = ledger_entry {
                    return Ok(Some(data.val));
                }
            }
        }

        Ok(None)
    }
}

// ── ScVal conversion helpers ──

/// Convert a Stellar address string to an ScVal::Address.
pub fn address_to_scval(address: &str) -> Result<ScVal, String> {
    if address.starts_with('G') {
        let pk = stellar_strkey::ed25519::PublicKey::from_string(address)
            .map_err(|e| format!("Invalid Stellar address: {}", e))?;
        Ok(ScVal::Address(ScAddress::Account(AccountId(
            PublicKey::PublicKeyTypeEd25519(Uint256(pk.0)),
        ))))
    } else if address.starts_with('C') {
        let contract = stellar_strkey::Contract::from_string(address)
            .map_err(|e| format!("Invalid contract address: {}", e))?;
        Ok(ScVal::Address(ScAddress::Contract(ContractId(Hash(contract.0)))))
    } else {
        Err(format!("Unknown address format: {}", address))
    }
}

/// Convert a u32 to ScVal::U32.
pub fn u32_to_scval(val: u32) -> ScVal {
    ScVal::U32(val)
}

/// Convert an i128 to ScVal::I128.
pub fn i128_to_scval(val: i128) -> ScVal {
    ScVal::I128(stellar_xdr::curr::Int128Parts {
        hi: (val >> 64) as i64,
        lo: val as u64,
    })
}

/// Convert a string to ScVal::Symbol.
pub fn symbol_to_scval(s: &str) -> Result<ScVal, String> {
    Ok(ScVal::Symbol(ScSymbol(
        s.try_into().map_err(|_| format!("Symbol too long: {}", s))?,
    )))
}

/// Convert a byte slice to ScVal::Bytes.
pub fn bytes_to_scval(b: &[u8]) -> Result<ScVal, String> {
    Ok(ScVal::Bytes(stellar_xdr::curr::ScBytes(
        b.to_vec().try_into().map_err(|_| "Bytes too long")?,
    )))
}

/// Extract a u32 from ScVal.
pub fn scval_to_u32(val: &ScVal) -> Option<u32> {
    match val {
        ScVal::U32(v) => Some(*v),
        _ => None,
    }
}

/// Extract an i128 from ScVal.
pub fn scval_to_i128(val: &ScVal) -> Option<i128> {
    match val {
        ScVal::I128(parts) => Some(((parts.hi as i128) << 64) | (parts.lo as i128)),
        _ => None,
    }
}

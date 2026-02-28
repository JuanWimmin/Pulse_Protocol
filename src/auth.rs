use actix_web::{web, HttpRequest, HttpResponse};
use ed25519_dalek::{Signature, Verifier, VerifyingKey};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;
use uuid::Uuid;

/// Authenticated user info injected into GraphQL context.
#[derive(Clone, Debug)]
pub struct AuthenticatedUser {
    pub user_id: Uuid,
    pub stellar_address: String,
}

/// In-memory session store.
pub type SessionStore = Arc<RwLock<HashMap<String, SessionData>>>;

#[derive(Clone, Debug)]
pub struct SessionData {
    pub stellar_address: String,
    pub user_id: Uuid,
}

#[derive(Deserialize)]
pub struct AuthPayload {
    pub stellar_address: String,
    pub message: String,
    pub signature: String,
}

#[derive(Serialize)]
pub struct AuthResponse {
    pub token: String,
    pub stellar_address: String,
}

/// POST /auth — Verify Stellar signature and create session.
pub async fn auth_handler(
    pool: web::Data<sqlx::PgPool>,
    sessions: web::Data<SessionStore>,
    payload: web::Json<AuthPayload>,
) -> HttpResponse {
    let payload = payload.into_inner();

    match verify_stellar_signature(
        &payload.stellar_address,
        &payload.message,
        &payload.signature,
    ) {
        Ok(true) => {}
        Ok(false) => {
            return HttpResponse::Unauthorized().json(serde_json::json!({
                "error": "Invalid signature"
            }));
        }
        Err(e) => {
            return HttpResponse::BadRequest().json(serde_json::json!({
                "error": e
            }));
        }
    }

    let user =
        match crate::models::user::User::find_or_create(pool.get_ref(), &payload.stellar_address)
            .await
        {
            Ok(u) => u,
            Err(e) => {
                return HttpResponse::InternalServerError().json(serde_json::json!({
                    "error": format!("Database error: {}", e)
                }));
            }
        };

    let token = generate_token();

    sessions.write().await.insert(
        token.clone(),
        SessionData {
            stellar_address: payload.stellar_address.clone(),
            user_id: user.id,
        },
    );

    HttpResponse::Ok().json(AuthResponse {
        token,
        stellar_address: payload.stellar_address,
    })
}

/// Extract AuthenticatedUser from request using Bearer token.
pub async fn extract_auth(req: &HttpRequest, sessions: &SessionStore) -> Option<AuthenticatedUser> {
    let auth_header = req.headers().get("Authorization")?.to_str().ok()?;
    let token = auth_header.strip_prefix("Bearer ")?;
    let sessions = sessions.read().await;
    let session = sessions.get(token)?;
    Some(AuthenticatedUser {
        user_id: session.user_id,
        stellar_address: session.stellar_address.clone(),
    })
}

fn generate_token() -> String {
    use rand::Rng;
    let mut rng = rand::thread_rng();
    let bytes: [u8; 32] = rng.gen();
    hex::encode(bytes)
}

/// Verify ed25519 signature from a Stellar keypair.
fn verify_stellar_signature(
    stellar_address: &str,
    message: &str,
    signature_hex: &str,
) -> Result<bool, String> {
    let pubkey_bytes = decode_stellar_address(stellar_address)?;

    let verifying_key = VerifyingKey::from_bytes(&pubkey_bytes)
        .map_err(|e| format!("Invalid public key: {}", e))?;

    let sig_bytes =
        hex::decode(signature_hex).map_err(|e| format!("Invalid signature hex: {}", e))?;

    if sig_bytes.len() != 64 {
        return Err(format!(
            "Signature must be 64 bytes, got {}",
            sig_bytes.len()
        ));
    }

    let mut sig_array = [0u8; 64];
    sig_array.copy_from_slice(&sig_bytes);
    let signature = Signature::from_bytes(&sig_array);

    let hashed = message.as_bytes();
    match verifying_key.verify(hashed, &signature) {
        Ok(()) => Ok(true),
        Err(_) => Ok(false),
    }
}

/// Decode a Stellar G... address to 32-byte ed25519 public key.
fn decode_stellar_address(address: &str) -> Result<[u8; 32], String> {
    if !address.starts_with('G') || address.len() != 56 {
        return Err("Invalid Stellar address: must start with G and be 56 chars".into());
    }

    let decoded = base32_decode(address).ok_or("Failed to base32 decode address")?;

    if decoded.len() != 35 {
        return Err(format!(
            "Decoded address should be 35 bytes, got {}",
            decoded.len()
        ));
    }

    // Version byte 6 << 3 = 48 for ed25519 public key (G... addresses)
    if decoded[0] != 6 << 3 {
        return Err(format!("Invalid version byte: {}", decoded[0]));
    }

    let mut pubkey = [0u8; 32];
    pubkey.copy_from_slice(&decoded[1..33]);

    // Verify CRC16-XMODEM checksum
    let expected = crc16_xmodem(&decoded[0..33]);
    let actual = u16::from_le_bytes([decoded[33], decoded[34]]);
    if expected != actual {
        return Err("Address checksum mismatch".into());
    }

    Ok(pubkey)
}

/// RFC 4648 base32 decode (alphabet: A-Z, 2-7).
fn base32_decode(input: &str) -> Option<Vec<u8>> {
    let alphabet = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    let mut lookup = [255u8; 256];
    for (i, &c) in alphabet.iter().enumerate() {
        lookup[c as usize] = i as u8;
    }

    let input = input.trim_end_matches('=');
    let mut bits: u64 = 0;
    let mut bit_count: u32 = 0;
    let mut output = Vec::new();

    for &b in input.as_bytes() {
        let val = lookup[b as usize];
        if val == 255 {
            return None;
        }
        bits = (bits << 5) | val as u64;
        bit_count += 5;
        if bit_count >= 8 {
            bit_count -= 8;
            output.push((bits >> bit_count) as u8);
            bits &= (1 << bit_count) - 1;
        }
    }

    Some(output)
}

/// CRC16-XMODEM checksum.
fn crc16_xmodem(data: &[u8]) -> u16 {
    let mut crc: u16 = 0;
    for &byte in data {
        crc ^= (byte as u16) << 8;
        for _ in 0..8 {
            if crc & 0x8000 != 0 {
                crc = (crc << 1) ^ 0x1021;
            } else {
                crc <<= 1;
            }
        }
    }
    crc
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_decode_stellar_address() {
        // Known Stellar test address
        let address = "GAAZI4TCR3TY5OJHCTJC2A4QSY6CJWJH5IAJTGKIN2ER7LBNVKOCCWN7";
        let result = decode_stellar_address(address);
        assert!(result.is_ok());
        assert_eq!(result.unwrap().len(), 32);
    }

    #[test]
    fn test_invalid_address() {
        assert!(decode_stellar_address("INVALID").is_err());
        assert!(
            decode_stellar_address("S1234567890123456789012345678901234567890123456789012345")
                .is_err()
        );
    }
}

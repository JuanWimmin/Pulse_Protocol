use ed25519_dalek::{SigningKey, Signer, SECRET_KEY_LENGTH};
use rand::rngs::OsRng;
use serde_json::json;
use std::fs;

// CRC16-XMODEM implementation
fn crc16_xmodem(data: &[u8]) -> u16 {
    let mut crc: u32 = 0;
    for byte in data {
        crc ^= (*byte as u32) << 8;
        for _ in 0..8 {
            crc <<= 1;
            if crc & 0x10000 != 0 {
                crc ^= 0x1021;
            }
        }
    }
    (crc & 0xFFFF) as u16
}

fn encode_stellar_address(public_key: &[u8; 32]) -> String {
    const VERSION_BYTE: u8 = 48; // Account address
    let mut data = vec![VERSION_BYTE];
    data.extend_from_slice(public_key);
    
    let checksum = crc16_xmodem(&data);
    data.extend_from_slice(&checksum.to_le_bytes());
    
    // Base32 encode
    const ALPHABET: &[u8] = b"ABCDEFGHIJKLMNOPQRSTUVWXYZ234567";
    let mut result = String::new();
    let mut bits = 0u32;
    let mut bit_count = 0;
    
    for byte in &data {
        bits = (bits << 8) | (*byte as u32);
        bit_count += 8;
        
        while bit_count >= 5 {
            bit_count -= 5;
            let index = ((bits >> bit_count) & 0x1F) as usize;
            result.push(ALPHABET[index] as char);
        }
    }
    
    if bit_count > 0 {
        let index = ((bits << (5 - bit_count)) & 0x1F) as usize;
        result.push(ALPHABET[index] as char);
    }
    
    result
}

fn main() {
    // Generate random secret key bytes
    let mut secret_bytes = [0u8; SECRET_KEY_LENGTH];
    rand::Rng::fill(&mut OsRng, &mut secret_bytes);
    
    // Create signing key from bytes
    let signing_key = SigningKey::from_bytes(&secret_bytes);
    
    let public_key_bytes: [u8; 32] = signing_key.verifying_key().to_bytes();
    let message = "pulse-auth:1234567890";
    
    // Sign the message
    let signature = signing_key.sign(message.as_bytes());
    
    // Create Stellar address
    let stellar_address = encode_stellar_address(&public_key_bytes);
    
    // Output results
    println!("=== ED25519 Signature Generation ===\n");
    println!("Public Key (hex): {}", hex::encode(&public_key_bytes));
    println!("Stellar Address: {}\n", stellar_address);
    println!("Message: {}\n", message);
    println!("Signature (hex): {}\n", hex::encode(signature.to_bytes()));
    
    // Create JSON payload
    let payload = json!({
        "stellar_address": stellar_address,
        "message": message,
        "signature": hex::encode(signature.to_bytes())
    });
    
    println!("=== JSON Payload ===");
    println!("{}\n", serde_json::to_string_pretty(&payload).unwrap());
    
    // Save to file for curl
    let output_path = "auth_payload.json";
    fs::write(output_path, payload.to_string()).expect("Failed to write payload file");
    println!("Payload saved to {}", output_path);
}
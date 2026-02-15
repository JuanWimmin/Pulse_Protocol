pub fn crc16_xmodem(data: &[u8]) -> u16 {
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

pub fn encode_stellar_address(public_key: &[u8; 32]) -> String {
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

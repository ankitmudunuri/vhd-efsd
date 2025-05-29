use std::fs::{File};
use std::io::{Read, Write, BufReader, BufWriter};
use aes_gcm::{Aes256Gcm, Key, Nonce};
use aes_gcm::aead::{Aead, KeyInit};
use sha2::{Sha256, Digest};

// Derive a 32-byte key from password and KEY
fn derive_key(password: &str, key_const: &[u8]) -> [u8; 32] {
    let mut hasher = Sha256::new_with_prefix(password.as_bytes());
    hasher.update(key_const);
    let result = hasher.finalize();
    let mut key = [0u8; 32];
    key.copy_from_slice(&result[..32]);
    key
}

pub fn encrypt_file(input_path: &str, output_path: &str, password: &str, key_const: &[u8]) -> std::io::Result<()> {
    let key_bytes = derive_key(password, key_const);
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    // Use a 12-byte nonce as required by AES-GCM
    let nonce = Nonce::from_slice(b"nonce_aesgcm"); // 12 bytes

    let mut input = BufReader::new(File::open(input_path)?);
    let mut buffer = Vec::new();
    input.read_to_end(&mut buffer)?;
    let ciphertext = cipher.encrypt(nonce, buffer.as_ref()).expect("encryption failure!");
    let mut output = BufWriter::new(File::create(output_path)?);
    output.write_all(&ciphertext)?;
    Ok(())
}

pub fn decrypt_file(input_path: &str, output_path: &str, password: &str, key_const: &[u8]) -> std::io::Result<()> {
    let key_bytes = derive_key(password, key_const);
    let key = Key::<Aes256Gcm>::from_slice(&key_bytes);
    let cipher = Aes256Gcm::new(key);
    let nonce = Nonce::from_slice(b"nonce_aesgcm");
    let mut input = BufReader::new(File::open(input_path)?);
    let mut buffer = Vec::new();
    input.read_to_end(&mut buffer)?;
    match cipher.decrypt(nonce, buffer.as_ref()) {
        Ok(plaintext) => {
            let mut output = BufWriter::new(File::create(output_path)?);
            output.write_all(&plaintext)?;
            Ok(())
        },
        Err(e) => {
            panic!("decryption failure!: {:?}", e);
        }
    }
}

// Encrypt a file (e.g., fragment_info.json) with the same key logic as VHD
pub fn encrypt_json(input_path: &str, output_path: &str, password: &str, key_const: &[u8]) -> std::io::Result<()> {
    encrypt_file(input_path, output_path, password, key_const)
}

// Decrypt a file (e.g., fragment_info.json) with the same key logic as VHD
pub fn decrypt_json(input_path: &str, output_path: &str, password: &str, key_const: &[u8]) -> std::io::Result<()> {
    decrypt_file(input_path, output_path, password, key_const)
}

use std::io::{self, Write};
use std::fs;
use std::path::Path;
use std::process::exit;
use bcrypt::{hash, verify, DEFAULT_COST};
use rpassword::read_password;
use serde::{Deserialize, Serialize};
use rand::rngs::OsRng;
use argon2::{Argon2, PasswordHash, PasswordHasher, PasswordVerifier};
use hmac::{Hmac, Mac};
use sha2::Sha256;
use hex;

type HmacSha256 = Hmac<Sha256>;

const KEY: &[u8] = b"secretkey";

#[derive(Serialize, Deserialize)]
struct PassData {
    password_hash: String,
    salt: String,
}

#[derive(Serialize, Deserialize)]
pub struct LoginAttempts {
    pub attempts: u32,
    pub mac: String,
}

fn prompt_password(prompt: &str) -> String{
    print!("{}", prompt);
    io::stdout().flush().unwrap();
    return read_password().unwrap();
}

fn write_to_file(pass_file: &str, pass_data: &PassData) -> (){
    let json_data = serde_json::to_string_pretty(pass_data).unwrap();
    fs::write(pass_file, json_data).expect("Could not write to file");
}

fn read_from_file(pass_file: &str) -> PassData {
    let json_data = fs::read_to_string(pass_file).expect("Could not read from file");
    let pass_data: PassData = serde_json::from_str(&json_data).expect("Could not cast to PassData");
    return pass_data;
}

pub fn compute_mac(attempts: u32, key: &[u8]) -> String {
    let mut mac = HmacSha256::new_from_slice(key).expect("HMAC can take key of any size");
    mac.update(&attempts.to_le_bytes());
    let result = mac.finalize();
    let code_bytes = result.into_bytes();
    hex::encode(code_bytes)
}

fn read_attempts(attempts_file: &str) -> Result<LoginAttempts, Box<dyn std::error::Error>> {
    let data = fs::read_to_string(attempts_file).expect("Could not read data from file");
    let attempts: LoginAttempts = serde_json::from_str(&data).expect("Could not read login attempts/some tampering has occured");
    Ok(attempts)
}

pub fn write_attempts(attempts_file: &str, attempts: &LoginAttempts) -> () {
    let data = serde_json::to_string_pretty(attempts).expect("Could not get data");
    fs::write(attempts_file, data).expect("Could not write data");
}

fn increment_attempts(attempts_file: &str) -> bool {
    let mut attempts: LoginAttempts = read_attempts(attempts_file).unwrap_or(LoginAttempts { attempts: 5, mac: compute_mac(5, KEY), });
    if attempts.mac != compute_mac(attempts.attempts, KEY) {
        println!("Tampering detected in attempts file! Starting self-destruct sequence.");
        // Do self-destruct sequence
        exit(1);
    }

    attempts.attempts += 1;
    if attempts.attempts >= 5 {
        attempts.attempts = 0;
        attempts.mac = compute_mac(attempts.attempts, KEY);
        write_attempts(attempts_file, &attempts);
        return true;
    }
    attempts.mac = compute_mac(attempts.attempts, KEY);
    write_attempts(attempts_file, &attempts);
    println!("Failed login attempts: {}", attempts.attempts);

    return false;
}

fn reset_attempts(attempts_file: &str) -> () {
    let mut attempts: LoginAttempts = LoginAttempts { attempts: 0, mac: compute_mac(0, KEY), };

    write_attempts(attempts_file, &attempts);
}

pub fn setup_password(pass_file: &str) -> (){


    let mut password: String = String::new();
    let mut confirm: String = String::new();

    loop {
        password = prompt_password("Enter new password: ");
        confirm = prompt_password("Confirm new password: ");

        if password == confirm {
            break;
        }
        else {
            println!("Passwords do not match!");
            continue;
        }
    }
    
    let salt = argon2::password_hash::SaltString::generate(&mut OsRng);

    let argon2 = Argon2::default();
    
    let hashed = argon2.hash_password(password.as_bytes(), &salt).expect("Failed to hash password");

    let pass_data = PassData {password_hash: hashed.to_string(), salt: salt.as_str().to_string()};

    write_to_file(pass_file, &pass_data);
    println!("Setup successful!");

}

pub fn login(pass_file: &str, attempts_file: &str) -> bool {
    let pass_data = read_from_file(pass_file);
    let parsed_hash = PasswordHash::new(&pass_data.password_hash).expect("Stored hash is invalid");
    let argon2obj = Argon2::default();
    let mut inp_password: String = String::new();


    loop {
        inp_password = prompt_password("Enter password: ");

        if argon2obj.verify_password(inp_password.as_bytes(), &parsed_hash).is_ok() {
            reset_attempts(attempts_file);
            break true;
        } else {
            println!("Incorrect password entered!");
            if increment_attempts(attempts_file){
                // Do self-destruct sequence
                println!("Too many failed attempts! Self-destruct initiated.");
                exit(1);
            }
        }
    }
}
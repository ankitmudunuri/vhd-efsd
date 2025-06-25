mod auth;
mod filesys;
mod json_manip;
mod keysetup;
mod crypto;

use std::fs;
use std::path::Path;
use std::env;

const KEY: &[u8] = b"thisisatest";

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let current_dir = env::current_dir().expect("Couldn't get current directory");
    let pass_file = "files/pass.json";
    let attempts_file = "files/attempts.json";
    let fragment_info_file = "files/fragment_info.json";
    let locker_pathbuf = current_dir.join(["files", "locker.vhd"].iter().collect::<std::path::PathBuf>());
    let locker = locker_pathbuf.to_str().expect("Couldn't cast to string");
    fs::create_dir_all("files").expect("Couldn't create files directory");
    if !Path::new(pass_file).exists() {
        use std::io::{self, Write};
        auth::setup_password(pass_file);
        auth::write_attempts(attempts_file, &auth::LoginAttempts {
            attempts: 0,
            mac: auth::compute_mac(0, KEY)
        });
        print!("Enter number of VHD fragments: ");
        io::stdout().flush()?;
        let mut frag_input = String::new();
        io::stdin().read_line(&mut frag_input)?;
        let fragment_count: usize = frag_input.trim().parse().expect("Invalid number");
        print!("Enter max number of binary chunks per file: ");
        io::stdout().flush()?;
        let mut chunk_input = String::new();
        io::stdin().read_line(&mut chunk_input)?;
        let max_chunks: usize = chunk_input.trim().parse().expect("Invalid number");
        let random_dirs = filesys::get_random_directories(fragment_count, "C:\\");
        
        println!("\nFragment directories selected:");
        for (i, dir) in random_dirs.iter().enumerate() {
            println!("  Fragment {}: {}", i, dir);
        }
        println!();
        
        let (key, fragments) = keysetup::generate_key_and_fragments(random_dirs.clone(), max_chunks);
        
        println!("Generated filenames:");
        for (i, fragment) in fragments.iter().enumerate() {
            println!("  File {}: {} in {} (chunks: {:?})", i, fragment.filename, fragment.directory, fragment.chunk_indices);
        }
        println!("Assembly key: {}", key);
        println!();
        
        let fragment_info = serde_json::json!({
            "fragment_count": fragment_count,
            "max_chunks": max_chunks,
            "dirs": random_dirs,
            "key": key,
            "fragments": fragments
        });
        fs::write(fragment_info_file, serde_json::to_string_pretty(&fragment_info)?).expect("Couldn't write fragment info");
        let passphrase = auth::prompt_password("Enter passphrase for fragment info encryption: ");
        let fragment_info_enc = current_dir.join(["files", "fragment_info.json.enc"].iter().collect::<std::path::PathBuf>());
        let fragment_info_enc_str = fragment_info_enc.to_str().unwrap();
        crypto::encrypt_json(fragment_info_file, fragment_info_enc_str, &passphrase, KEY)?;
        let _ = fs::remove_file(fragment_info_file);
        filesys::attach_drive(locker)?;
        return Ok(());
    }
    
    if Path::new(locker).exists() && filesys::is_vhd_attached(locker) {
        filesys::detach_drive(locker)?;
        let fragment_info_enc = current_dir.join(["files", "fragment_info.json.enc"].iter().collect::<std::path::PathBuf>());
        let fragment_info_enc_str = fragment_info_enc.to_str().unwrap();
        let fragment_info_dec = current_dir.join(["files", "fragment_info.json"].iter().collect::<std::path::PathBuf>());
        let fragment_info_dec_str = fragment_info_dec.to_str().unwrap();
        let passphrase = auth::prompt_password("Enter passphrase for fragment info decryption: ");
        crypto::decrypt_json(fragment_info_enc_str, fragment_info_dec_str, &passphrase, KEY)?;
        let fragment_info: serde_json::Value = serde_json::from_str(&fs::read_to_string(fragment_info_dec_str)?)?;
        let _ = fs::remove_file(fragment_info_dec_str);
        let key = fragment_info["key"].as_str().unwrap();
        let fragments: Vec<keysetup::FragmentInfo> = serde_json::from_value(fragment_info["fragments"].clone())?;
        
        let total_chunks = key.len();
        
        let enc_vhd = current_dir.join(["files", "locker_encrypted.vhd"].iter().collect::<std::path::PathBuf>());
        let enc_vhd_str = enc_vhd.to_str().unwrap();
        let password = auth::get_password_from_user();
        crypto::encrypt_file(locker, enc_vhd_str, &password, KEY).expect("Encryption failed");
        let _ = fs::remove_file(locker);
        filesys::split_binary_with_key(enc_vhd_str, &fragments, total_chunks).expect("Failed to split binary");
        let _ = fs::remove_file(enc_vhd_str);
        return Ok(());
    }
    
    if Path::new(locker).exists() && !filesys::is_vhd_attached(locker) {
        println!("VHD file found but not attached. Attaching drive...");
        filesys::attach_drive(locker)?;
        return Ok(());
    }
    
    if !Path::new(locker).exists() {
        let password = match auth::login_and_get_password(pass_file, attempts_file) {
            Some(pwd) => pwd,
            None => return Ok(()),
        };
        
        let fragment_info_enc = current_dir.join(["files", "fragment_info.json.enc"].iter().collect::<std::path::PathBuf>());
        let fragment_info_enc_str = fragment_info_enc.to_str().unwrap();
        let fragment_info_dec = current_dir.join(["files", "fragment_info.json"].iter().collect::<std::path::PathBuf>());
        let fragment_info_dec_str = fragment_info_dec.to_str().unwrap();
        crypto::decrypt_json(fragment_info_enc_str, fragment_info_dec_str, &password, KEY)?;
        let fragment_info: serde_json::Value = serde_json::from_str(&fs::read_to_string(fragment_info_dec_str)?)?;
        let _ = fs::remove_file(fragment_info_dec_str);
        let key = fragment_info["key"].as_str().unwrap();
        let fragments: Vec<keysetup::FragmentInfo> = serde_json::from_value(fragment_info["fragments"].clone())?;
        
        let mut missing = false;
        for fragment in &fragments {
            let fpath = std::path::Path::new(&fragment.directory).join(&fragment.filename);
            if !fpath.exists() {
                missing = true;
                break;
            }
        }
        if missing {
            println!("Error: Some fragments are missing. Cannot reassemble VHD.");
            return Ok(());
        }
        
        println!("Reassembling VHD from fragments...");
        println!("Fragment files:");
        for (i, fragment) in fragments.iter().enumerate() {
            println!("  File {}: {} in {} (chunks: {:?})", i, fragment.filename, fragment.directory, fragment.chunk_indices);
        }
        println!("Assembly key: {}", key);
        
        let enc_vhd = current_dir.join(["files", "locker_encrypted.vhd"].iter().collect::<std::path::PathBuf>());
        let enc_vhd_str = enc_vhd.to_str().unwrap();
        filesys::assemble_binary_with_key(&fragments, key, enc_vhd_str).expect("Failed to assemble binary");
        let vhd_password = auth::get_password_from_user();
        crypto::decrypt_file(enc_vhd_str, locker, &vhd_password, KEY).expect("Decryption failed");
        let _ = fs::remove_file(enc_vhd_str);
        filesys::attach_drive(locker)?;
        return Ok(());
    }
    
    println!("Error: Unexpected system state. Please check your files.");
    Ok(())
}

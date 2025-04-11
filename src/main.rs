mod auth;
mod filesys;
mod json_manip;
mod keysetup;
use std::fs;
use std::path::Path;
use std::env;


use auth::login;

const KEY: &[u8] = b"secretkey";

fn main() {
    let current_dir = env::current_dir().expect("Couldn't get current directory");
    let pass_file = "files/pass.json";
    let attempts_file = "files/attempts.json";
    let direc_file = "files/directories.json";
    let locker_pathbuf = current_dir.join("files\\locker.vhd");

    let locker = locker_pathbuf.to_str().expect("Couldn't cast to string");

    if !Path::new(pass_file).exists() {
        auth::setup_password(pass_file);

        let attempts = auth::LoginAttempts {
            attempts: 0,
            mac: auth::compute_mac(0, KEY)
        };
        auth::write_attempts(attempts_file, &attempts);

    }
    
    let test = filesys::get_random_directories(5, "C:");
    keysetup::generate_key(test, 7);


    // match filesys::detach_drive(locker) {
    //     Ok(()) => {},
    //     Err(_why) => {
    //         login(pass_file, attempts_file);
    //         filesys::attach_drive(locker);
    //     }
    // }

}

use vhdrs;
use windows;
use core::num;
use std::vec;
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Read, SeekFrom, Seek, Write};
use std::path::{Path, PathBuf};
use rand::Rng;
use walkdir::WalkDir;
use std::process::Command;
use std::env;
use std::thread;
use std::time::Duration;

pub fn main() -> Result<(), Box<dyn std::error::Error>> {
    let vhd_path = "C:\\Users\\ankit_nk\\Documents\\sdfs\\test.vhd";
    println!("Attempting to attach/detach VHD at: {}", vhd_path);
    
    attach_drive(vhd_path)?;
    println!("VHD operation completed successfully!");
    
    Ok(())
}

pub fn create_drive(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    // Create parent directory if it doesn't exist
    if let Some(parent) = Path::new(path).parent() {
        if !parent.exists() {
            fs::create_dir_all(parent)?;
        }
    }

    let mut input = String::new();
    let mut letter = String::new();
    print!("Enter disk size in MB: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut input)?;

    print!("Enter drive letter: ");
    io::stdout().flush()?;
    io::stdin().read_line(&mut letter)?;

    let disk_mb: u64 = input.trim().parse()?;
    let letterstr: &str = letter.trim();
      let diskpart_script = format!(
        "create vdisk file=\"{}\" maximum={} type=fixed
select vdisk file=\"{}\"
attach vdisk
create partition primary
format fs=ntfs label=\"Locker\" quick
assign letter={}
detach vdisk
",
        path, disk_mb, path, letterstr
    );

    let mut script_path = env::temp_dir();
    script_path.push("diskpart_script.txt");

    fs::write(&script_path, diskpart_script)?;

    let output = Command::new("diskpart")
        .arg("/s")
        .arg(&script_path)
        .output()?;

    if !output.status.success() {
        return Err(format!(
            "Failed to create VHD. DiskPart error: {}",
            String::from_utf8_lossy(&output.stderr)
        ).into());
    }

    // Clean up the temporary script file
    let _ = fs::remove_file(script_path);

    Ok(())
}

pub fn is_vhd_attached(path: &str) -> bool {
    let script = format!(
        r#"select vdisk file="{}"
detail vdisk"#,
        path
    );

    let mut script_path = env::temp_dir();
    script_path.push("diskpart_check.txt");

    if let Ok(_) = fs::write(&script_path, script) {
        if let Ok(output) = Command::new("diskpart")
            .arg("/s")
            .arg(&script_path)
            .output() {
            let _ = fs::remove_file(script_path);
            let output_str = String::from_utf8_lossy(&output.stdout);
            
            // VHD is mounted if we have a disk number and state isn't Added or Detached
            let has_disk = !output_str.contains("Associated disk#: Not found");
            let state_added = output_str.contains("State : Added");
            let state_detached = output_str.contains("State : Detached");
            
            return has_disk && !state_added && !state_detached;
        }
    }
    false
}

pub fn attach_drive(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    if !Path::new(path).exists() {
        create_drive(path)?;
    }
    // Always try to detach first, in case it's in 'Added' state
    let _ = vhdrs::Vhd::detach(path);
    std::thread::sleep(std::time::Duration::from_millis(500));
    let mut vhd = vhdrs::Vhd::new(path, vhdrs::OpenMode::ReadWrite, None)?;
    vhd.attach(true)?;
    Ok(())
}

pub fn detach_drive(path: &str) -> Result<(), Box<dyn std::error::Error>> {
    vhdrs::Vhd::detach(path)?;
    Ok(())
}

pub fn split_binary(filepaths: Vec<(&str, &str)>, vhdname: &str) -> (){
    let numchunks: usize = filepaths.len();

    let vhdpath = Path::new(vhdname);

    let metadata = fs::metadata(&vhdpath).unwrap();
    let filesize = metadata.len() as usize;

    let chunksize = filesize / numchunks;

    let file = File::open(vhdpath).unwrap();
    let mut reader = BufReader::new(file);

    for i in 0..numchunks {
        let bytestoread = if i < numchunks - 1 {
            chunksize
        } else {
            filesize - chunksize * (numchunks - 1)
        };

        let mut chunk_buff = vec![0u8; bytestoread];

        reader.read_exact(&mut chunk_buff).unwrap();

        let chunkfilename = format!("{}\\{}", filepaths[i].1, filepaths[i].0);
        let mut chunkfile = File::create(&chunkfilename).expect(&format!("Failed to write to {}", chunkfilename));
        chunkfile.write_all(&chunk_buff).expect("Couldn't write to buffer");
    }

    // Delete the original VHD file after splitting
    if let Err(e) = fs::remove_file(vhdpath) {
        eprintln!("Warning: failed to delete VHD file after splitting: {}", e);
    }
}

pub fn assemble_binary(directories: Vec<(&str, &str)>, vhdname: &str) -> (){
    let outputfile = File::create(vhdname).expect("Couldn't create file");
    let mut writer = BufWriter::new(outputfile);

    for (filename, directory) in directories {
        let chunkpath: PathBuf = Path::new(directory).join(filename);

        let chunkfile = File::open(&chunkpath).unwrap();
        let mut reader = BufReader::new(chunkfile);

        io::copy(&mut reader, &mut writer).expect("Couldn't copy");
    }

    writer.flush().expect("Couldn't flush write buffer");
}

fn normalize_path(path: &str) -> String {
    if path.len() >= 2 && path.chars().nth(1) == Some(':') {
        if path.len() == 2 || path.chars().nth(2) != Some('\\') {
            let (drive, rest) = path.split_at(2);
            let rest = rest.trim_start();
            return format!("{}\\{}", drive, rest);
        }
    }
    path.to_string()
}

pub fn get_random_directories(n: usize, base_path: &str) -> Vec<String> {
    let mut rng = rand::thread_rng();
    let mut result = Vec::with_capacity(n);
    let accept_prob = 0.05;
    let temp_dir = std::env::temp_dir().to_string_lossy().to_lowercase();
    for entry in WalkDir::new(base_path)
        .min_depth(1)
        .max_depth(8)
        .into_iter()
        .filter_map(Result::ok)
    {
        let path = entry.path();
        if path.is_dir() {
            if let Some(s) = path.to_str() {
                let norm = normalize_path(s);
                let norm_lower = norm.to_lowercase();
                if norm_lower.contains(&temp_dir)
                    || norm_lower.contains("windows")
                    || norm_lower.contains("program files")
                    || norm_lower.contains("programdata")
                    || norm_lower.contains("system volume information")
                    || norm_lower.contains("recycle.bin")
                    || norm_lower.contains("appdata")
                    || norm_lower.contains("users\\default")
                    || norm_lower.contains("users\\public")
                    || norm_lower.contains("$")
                {
                    continue;
                }
                if rng.gen_bool(accept_prob) {
                    result.push(norm);
                    if result.len() == n {
                        break;
                    }
                }
            }
        }
    }
    result
}

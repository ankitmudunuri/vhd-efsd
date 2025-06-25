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

pub fn split_binary_with_key(vhd_path: &str, fragments: &[crate::keysetup::FragmentInfo], total_chunks: usize) -> Result<(), Box<dyn std::error::Error>> {
    let vhd_file = File::open(vhd_path)?;
    let metadata = vhd_file.metadata()?;
    let total_size = metadata.len() as usize;
    
    let chunk_size = if total_chunks > 0 {
        (total_size + total_chunks - 1) / total_chunks
    } else {
        return Err("Total chunks must be greater than 0".into());
    };
    
    let mut reader = BufReader::new(vhd_file);
    let mut chunks = Vec::new();
    
    for i in 0..total_chunks {
        let bytes_to_read = if i < total_chunks - 1 {
            chunk_size
        } else {
            total_size - chunk_size * (total_chunks - 1)
        };
        
        let mut chunk_buffer = vec![0u8; bytes_to_read];
        reader.read_exact(&mut chunk_buffer)?;
        chunks.push(chunk_buffer);
    }
    
    for fragment in fragments {
        let file_path = Path::new(&fragment.directory).join(&fragment.filename);
        let mut output_file = BufWriter::new(File::create(&file_path)?);
        
        for &chunk_index in &fragment.chunk_indices {
            if chunk_index < chunks.len() {
                output_file.write_all(&chunks[chunk_index])?;
            }
        }
        
        output_file.flush()?;
    }
    
    if let Err(e) = fs::remove_file(vhd_path) {
        eprintln!("Warning: failed to delete VHD file after splitting: {}", e);
    }
    
    Ok(())
}

pub fn assemble_binary_with_key(fragments: &[crate::keysetup::FragmentInfo], key: &str, output_path: &str) -> Result<(), Box<dyn std::error::Error>> {
    let total_chunks = key.len();
    let mut chunks = vec![Vec::new(); total_chunks];
    
    let mut total_size = 0;
    for fragment in fragments {
        let file_path = Path::new(&fragment.directory).join(&fragment.filename);
        if let Ok(metadata) = fs::metadata(&file_path) {
            total_size += metadata.len() as usize;
        }
    }
    
    let chunk_size = if total_chunks > 0 {
        (total_size + total_chunks - 1) / total_chunks 
    } else {
        return Err("No chunks to assemble".into());
    };
    
    for fragment in fragments {
        let file_path = Path::new(&fragment.directory).join(&fragment.filename);
        let fragment_file = File::open(&file_path)?;
        let mut reader = BufReader::new(fragment_file);
        
        if fragment.chunk_indices.is_empty() {
            continue;
        }
        
        let mut fragment_data = Vec::new();
        reader.read_to_end(&mut fragment_data)?;
        
        let mut offset = 0;
        for &global_chunk_index in &fragment.chunk_indices {
            let bytes_for_this_chunk = if global_chunk_index < total_chunks - 1 {
                chunk_size
            } else {
                total_size - chunk_size * (total_chunks - 1)
            };
            
            if offset + bytes_for_this_chunk <= fragment_data.len() && global_chunk_index < chunks.len() {
                chunks[global_chunk_index] = fragment_data[offset..offset + bytes_for_this_chunk].to_vec();
                offset += bytes_for_this_chunk;
            }
        }
    }
    
    let output_file = File::create(output_path)?;
    let mut writer = BufWriter::new(output_file);
    
    for chunk in chunks {
        if !chunk.is_empty() {
            writer.write_all(&chunk)?;
        }
    }
    
    writer.flush()?;
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
    let accept_prob = 0.1;
    let temp_dir = std::env::temp_dir().to_string_lossy().to_lowercase();
    
    let preferred_patterns = [
        "documents", "pictures", "videos", "music", "projects", "work", "data", 
        "files", "archive", "storage", "shared", "public", "media", "games"
    ];
    
    for entry in WalkDir::new(base_path)
        .min_depth(2)
        .max_depth(6)
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
                    || norm_lower.contains("temp")
                    || norm_lower.contains("tmp")
                    || norm_lower.contains("cache")
                    || norm_lower.contains("logs")
                    || norm_lower.contains("log")
                    || norm_lower.contains("backup")
                    || norm_lower.contains("downloads")
                    || norm_lower.contains("desktop")
                    || norm_lower.contains("recent")
                    || norm_lower.contains("history")
                    || norm_lower.contains("cookies")
                    || norm_lower.contains("temporary")
                    || norm_lower.contains("msocache")
                    || norm_lower.contains("prefetch")
                    || norm_lower.contains("intel")
                    || norm_lower.contains("microsoft")
                    || norm_lower.contains("mozilla")
                    || norm_lower.contains("google")
                    || norm_lower.contains("chrome")
                    || norm_lower.contains("firefox")
                    || norm_lower.contains("edge")
                    || norm_lower.contains("system32")
                    || norm_lower.contains("syswow64")
                    || norm_lower.contains("winsxs")
                    || norm_lower.contains("recovery")
                    || norm_lower.contains("perflogs")
                    || norm_lower.starts_with("c:\\users")
                    || norm_lower.starts_with("c:\\windows")
                    || norm_lower.starts_with("c:\\program")
                    || norm.len() < 10 
                {
                    continue;
                }
                
                let is_preferred = preferred_patterns.iter().any(|&pattern| norm_lower.contains(pattern));
                let probability = if is_preferred { accept_prob * 3.0 } else { accept_prob };
                
                if rng.gen_bool(probability) {
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

use vhdrs;
use windows;
use core::num;
use std::vec;
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Read, SeekFrom, Seek, Write};
use std::path::{Path, PathBuf};
use rand::seq::SliceRandom;
use walkdir::WalkDir;
use std::process::Command;
use std::env;
use std::thread;
use std::time::Duration;

pub fn main(){

    let test = get_random_directories(127, "C:");
    println!("{:?}", test);
}

fn create_drive(path: &str) -> (){

    let mut input = String::new();
    let mut letter = String::new();
    print!("Enter disk size in MB: ");
    io::stdout().flush();
    io::stdin().read_line(&mut input).expect("Could not read I/O");

    print!("Enter drive letter: ");
    io::stdout().flush();
    io::stdin().read_line(&mut letter).expect("Could not read I/O");

    let disk_mb: u64 = input.trim().parse().expect("Could not case to integer");
    let letterstr: &str = letter.trim();
    
    let diskpart_script = format!(
        "
        create vdisk file=\"{}\" maximum={} type=fixed\n\
        select vdisk file=\"{}\"\n\
        attach vdisk\n\
        create partition primary\n\
        format fs=ntfs label=\"Locker\" quick\n\
        assign letter={}\n\
        detach vdisk\n
        ",
                path, disk_mb, path, letterstr
            );

    let mut script_path = env::temp_dir();
    script_path.push("diskpart_script.txt");

    fs::write(&script_path, diskpart_script).expect("Couldn't write to tempfile");

    let output = Command::new("diskpart")
        .arg("/s")
        .arg(&script_path)
        .output().expect("Could not run command");

}


pub fn attach_drive(path: &str) -> Result<(), Box<dyn std::error::Error>>{

    if Path::new(path).exists() {
        let _ = vhdrs::Vhd::detach(path);
    } else {
        create_drive(path);
    }

    let mut vhd = vhdrs::Vhd::new(path, vhdrs::OpenMode::ReadWrite, None)?;
    let drive_letter = vhd.attach(true)?;

    let disk_info = vhd.get_size().unwrap();

    Ok(())
}

pub fn detach_drive(path: &str) -> Result<(), Box<dyn std::error::Error>>{
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
    let mut dirs: Vec<String> = Vec::new();

    for entry in WalkDir::new(base_path)
        .min_depth(1)
        .max_depth(125)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.is_dir() {
            if let Some(path_str) = path.to_str() {
                let normalized = normalize_path(path_str);
                dirs.push(normalized);
            }
        }
    }

    let mut rng = rand::thread_rng();
    dirs.shuffle(&mut rng);

    if dirs.len() > n {
        dirs.truncate(n);
    }

    dirs
}

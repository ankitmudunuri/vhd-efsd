use vhdrs;
use windows;
use core::num;
use std::vec;
use std::fs::{self, File};
use std::io::{self, BufReader, BufWriter, Read, Write};
use std::path::{Path, PathBuf};

// main function only for testing purposes

// pub fn main(){
//     //let vhd = attach_drive("test.vhd");
//     //detach_drive("test.vhd");

//     let mappings = vec![
//         ("ae.bin", "C:\\Users\\ankit\\Documents\\selfdestruct\\sdfs\\testing"),
//         ("os.bin", "C:\\Users\\ankit\\Documents\\selfdestruct\\sdfs\\testing\\test1")
//     ];

//     assemble_binary(mappings);
// }

pub fn attach_drive(path: &str) -> vhdrs::Vhd{
    let mut vhd = vhdrs::Vhd::new(path, vhdrs::OpenMode::ReadWrite, None).expect("Failed to open VHD");
    let drive_letter = vhd.attach(true).expect("FAILED TO MOUNT DRIVE");

    let disk_info = vhd.get_size().unwrap();
    println!("Disk {} info: {:?}", drive_letter, disk_info);

    return vhd;
}

pub fn detach_drive(path: &str) {
    vhdrs::Vhd::detach(path);
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
use serde::Serialize;
use serde_json::{json, Value};
use std::vec;
use std::fs::File;

#[derive(Serialize)]
struct OrderKey<'a>{
    order: &'a str,
    length: usize,
    num_segments: usize,
    chunk_map: Vec<ChunkMapping<'a>>
}

#[derive(Serialize)]
struct ChunkMapping<'a>{
    file: &'a str,
    directory: &'a str
}

pub fn main(){

    let mappings = vec![
        ChunkMapping {file: "ae5874.bin", directory: "C:\\Users\\ankit\\Documents\\selfdestruct\\sdfs\\testing"},
        ChunkMapping {file: "os54.bin", directory: "C:\\Users\\ankit\\Documents\\selfdestruct\\sdfs\\testing\\test1"},
        ChunkMapping {file: "diegbeodiosbe.bin", directory: "C:\\Users\\ankit\\Documents\\selfdestruct\\sdfs\\testing\\test1"},
    ];

    to_json(mappings);

}

pub fn to_json(pairings: Vec<ChunkMapping>){
    let file = File::create("files/directories.json").unwrap();

    let mut ord_str: String = String::new();
    let mut length: usize = 0;
    let mut seg_len: usize = 0;
    let mut chunk = "";

    for i in &pairings{

        chunk = i.file.strip_suffix(".bin").unwrap();

        seg_len += 1;

        length += chunk.len();
        ord_str.push_str(chunk);

    }

    let ord_key = OrderKey {order: &ord_str, length: length, num_segments: seg_len, chunk_map: pairings};

    serde_json::to_writer_pretty(&file, &ord_key).expect("Couldn't write to JSON file");
}


use serde_json::{json, Value};
use std::vec;
use std::fs::File;



pub fn to_json(pairings: Vec<(&str, &str)>){
    let file = File::create("files/directories.json").unwrap();

    serde_json::to_writer_pretty(file, &pairings);
}
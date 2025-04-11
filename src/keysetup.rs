use rand::{distributions::Alphanumeric, Rng};
use rand::prelude::SliceRandom;
use crate::filesys;
use std::vec;
use std::collections::VecDeque;
use std::collections::HashMap;



struct Filename {
    original: String,
    queue: VecDeque<char>,
    idx: usize,
}

pub fn generate_names(n: usize) -> Filename{

    let mut rngen = rand::thread_rng();

    let len = rngen.gen_range(1..=n);

    let filename: String = rngen.sample_iter(&Alphanumeric).take(len).map(char::from).collect();

    let filename = filename.trim();
    let mut char_queue: VecDeque<char> = filename.chars().collect();

    println!("Filename: {}", filename.to_string());

    return Filename {original: filename.to_string(), queue: char_queue, idx: 0};
}

fn adjust_conflicts(filevec: &mut [Filename]) {
    loop {
        let front_letters: Vec<(usize, char)> = filevec
            .iter()
            .enumerate()
            .filter_map(|(i, file)| file.queue.front().cloned().map(|c| (i, c)))
            .collect();

        let mut freq: HashMap<char, Vec<usize>> = HashMap::new();
        for (i, ch) in front_letters.iter() {
            freq.entry(*ch).or_default().push(*i);
        }

        let mut conflict_found = false;
        for (_ch, indices) in &freq {
            if indices.len() > 1 {
                conflict_found = true;
                break;
            }
        }
        if !conflict_found {
            break;
        }

        for (_ch, indices) in freq {
            if indices.len() > 1 {
                for i in indices {
                    let current = match filevec[i].queue.front().cloned() {
                        Some(c) => c,
                        None => continue,
                    };
                    let mut new_char = current;
                    let mut found = false;
                    for offset in 1..10 {
                        for sign in [1, -1] {
                            let candidate_val = (current as i32) + sign * (offset as i32);
                            if let Some(candidate) = std::char::from_u32(candidate_val as u32) {
                                let mut unique = true;
                                for (j, file) in filevec.iter().enumerate() {
                                    if j == i {
                                        continue;
                                    }
                                    if let Some(other) = file.queue.front() {
                                        if candidate == *other {
                                            unique = false;
                                            break;
                                        }
                                    }
                                }
                                if unique {
                                    new_char = candidate;
                                    found = true;
                                    break;
                                }
                            }
                        }
                        if found {
                            break;
                        }
                    }
                    if found {
                        filevec[i].queue.pop_front();
                        filevec[i].queue.push_front(new_char);
                        let idx = filevec[i].idx;
                        unsafe {
                            let bytes = filevec[i].original.as_bytes_mut();
                            if idx < bytes.len() {
                                bytes[idx] = new_char as u8;
                            }
                        }
                    }
                }
            }
        }
    }
}

pub fn generate_key(direc_vec: Vec<String>, max_len: usize){
    let mut filevec: Vec<Filename> = Vec::new();
    for _ in direc_vec.iter() {
        filevec.push(generate_names(max_len));
    }
    
    let mut result_key = String::new();
    let mut rng = rand::thread_rng();

    while filevec.iter().any(|f| !f.queue.is_empty()) {
        adjust_conflicts(&mut filevec);
        
        let non_empty_indices: Vec<usize> = filevec
            .iter()
            .enumerate()
            .filter_map(|(i, f)| if !f.queue.is_empty() { Some(i) } else { None })
            .collect();
        
        if let Some(&random_index) = non_empty_indices.choose(&mut rng) {
            if let Some(ch) = filevec[random_index].queue.pop_front() {
                result_key.push(ch);
                filevec[random_index].idx += 1;
            }
        }
    }
    
    println!("{}", result_key);

    for (i, filename) in filevec.iter().enumerate() {
        println!("Filename {}: {}", i + 1, filename.original);
    }
}